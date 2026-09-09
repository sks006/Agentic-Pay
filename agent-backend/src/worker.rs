use crate::error::EngineError;
use crate::solana_rpc::{AsyncSolanaProvider, SolanaRpcAdapter};
use crate::state::{NoncePhase, ServerState};
pub use crate::voucher::VoucherPayload;
use solana_sdk::{
    message::Message,
    transaction::VersionedTransaction,
};
use solana_system_interface::instruction::transfer;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct WorkerConfig {
    pub batch_interval: Duration, // How often to sweep for pending vouchers
    pub max_retries: usize,
    pub retry_backoff_base: Duration,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            batch_interval: Duration::from_secs(10),
            max_retries: 3,
            retry_backoff_base: Duration::from_secs(2),
        }
    }
}

/// The settlement worker daemon.
pub struct SettlementWorker {
    pub state: Arc<ServerState>,
    pub rpc: Arc<AsyncSolanaProvider>,
    pub config: WorkerConfig,
}

impl SettlementWorker {
    pub fn new(state: Arc<ServerState>, rpc: Arc<AsyncSolanaProvider>, config: WorkerConfig) -> Self {
        Self { state, rpc, config }
    }

    /// Start the worker loop.
    pub async fn run(&self) {
        let mut interval = time::interval(self.config.batch_interval);
        loop {
            interval.tick().await;
            if let Err(e) = self.process_cycle().await {
                error!("Worker cycle error: {}", e);
            }
        }
    }

    /// One processing cycle adhering to the topological state machine:
    /// 1. Extraction & Lock: Atomically transition IntentSigned -> BatchedForSettlement
    /// 2. Execution: Chunk MTU payload and broadcast via RPC
    /// 3. Recovery (Success): BatchedForSettlement -> Finalized
    /// 4. Recovery (Failure): Increment retry_count, revert to IntentSigned or fail
    pub async fn process_cycle(&self) -> Result<(), EngineError> {
        // 1. Extraction & Lock: Sweep DashMap and immediately transition IntentSigned -> BatchedForSettlement
        let vouchers = self.state.extract_and_lock_intent_vouchers();
        if !vouchers.is_empty() {
            info!("Extracted and locked {} vouchers for settlement", vouchers.len());
        }

        // 2. Execution: Chunk and settle each batch
        let chunks = Self::chunk_vouchers(&vouchers);
        for chunk in chunks {
            if let Err(e) = self.settle_batch(chunk).await {
                error!("Batch settlement failed: {}", e);
            }
        }

        Ok(())
    }

    /// Chunk a list of vouchers into batches (MTU-safe).
    pub fn chunk_vouchers(vouchers: &[VoucherPayload]) -> Vec<Vec<VoucherPayload>> {
        const MAX_TX_SIZE: usize = 1232;
        // Conservative base overhead: signature, blockhash, headers, and ALT index space.
        let base_overhead = 128;

        let mut chunks = Vec::new();
        let mut current = Vec::new();
        let mut projected_size = base_overhead;
        let mut alt_count = 0;

        for v in vouchers {
            // Marginal cost per transfer instruction
            let marginal_cost = 1 + 2 + 8 + 2; // ~13 bytes per transfer
            let new_size = projected_size + marginal_cost;

            if new_size > MAX_TX_SIZE || alt_count + 2 > 256 {
                if !current.is_empty() {
                    chunks.push(std::mem::take(&mut current));
                }
                projected_size = base_overhead;
                alt_count = 0;
            }

            current.push(v.clone());
            projected_size += marginal_cost;
            alt_count += 2; // Two accounts per transfer (from, to)
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }

    /// Settle a single batch of vouchers.
    pub async fn settle_batch(&self, batch: Vec<VoucherPayload>) -> Result<(), EngineError> {
        if batch.is_empty() {
            return Ok(());
        }

        // Build a VersionedTransaction with ALTs/transfers
        let tx = self.build_batch_transaction(&batch)?;

        // Send transaction
        let tx_bytes = bincode::serialize(&tx).map_err(|_| EngineError::PayloadFormatInvalid)?;
        match self.rpc.send_transaction(&tx_bytes).await {
            Ok(sig) => {
                // 3. Recovery (Success): BatchedForSettlement -> Finalized
                info!("Batch transaction sent: {}", hex::encode(sig));
                for v in &batch {
                    if let Err(e) = self.state.promote_phase_sync(v.nonce, NoncePhase::Finalized) {
                        error!("Failed to promote nonce {}: {}", v.nonce, e);
                    }
                }
                Ok(())
            }
            Err(e) => {
                // 4. Recovery (Failure): The RPC returns an error. Increment retry_count += 1
                warn!("Batch submission failed: {}", e);
                for mut v in batch {
                    v.retry_count = v.retry_count.saturating_add(1);
                    if (v.retry_count as usize) < self.config.max_retries {
                        // Revert the state BatchedForSettlement -> IntentSigned (picked up on next sweep)
                        if let Err(err) = self.state.promote_phase_with_voucher_sync(
                            v.nonce,
                            NoncePhase::IntentSigned(v.signature),
                            v.clone(),
                        ) {
                            error!("Failed to revert nonce {} to IntentSigned: {}", v.nonce, err);
                        }
                    } else {
                        // Transition BatchedForSettlement -> Failed. Capital is freed.
                        if let Err(err) = self.state.mark_voucher_failed(v.nonce, &v.agent) {
                            error!("Failed to mark nonce {} as Failed: {}", v.nonce, err);
                        }
                    }
                }
                Err(EngineError::ProviderFault(format!("Batch failed: {}", e)))
            }
        }
    }

    /// Build a transaction for a batch using modular transfer instructions.
    pub fn build_batch_transaction(&self, batch: &[VoucherPayload]) -> Result<VersionedTransaction, EngineError> {
        let from_pubkey = VoucherPayload::pubkey_from_bytes(&batch[0].agent);
        let mut instructions = Vec::with_capacity(batch.len());
        for v in batch {
            let to = VoucherPayload::pubkey_from_bytes(&v.provider);
            instructions.push(transfer(&from_pubkey, &to, v.amount_lamports));
        }

        let message = Message::new(&instructions, Some(&from_pubkey));
        let tx = VersionedTransaction::from(solana_sdk::transaction::Transaction::new_unsigned(message));
        Ok(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::EngineConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_extraction_lock_prevents_duplicate_sweep() {
        let state = Arc::new(ServerState::new(Some(EngineConfig::default())));
        let agent = [1u8; 32];
        let provider = [2u8; 32];
        let sig = [3u8; 64];

        let voucher = VoucherPayload::new(1001, agent, provider, 50_000, sig);

        // Acquire nonce and commit with voucher to IntentSigned
        let guard = state.try_acquire_nonce_sync(agent, 1001).unwrap();
        guard.commit_with_voucher(NoncePhase::IntentSigned(sig), voucher.clone()).unwrap();

        // Verify initial state
        assert_eq!(
            state.pending_locks.get(&1001).unwrap().state,
            NoncePhase::IntentSigned(sig)
        );

        // First sweep: extracts voucher and immediately transitions to BatchedForSettlement
        let swept = state.extract_and_lock_intent_vouchers();
        assert_eq!(swept.len(), 1);
        assert_eq!(swept[0].nonce, 1001);
        assert_eq!(swept[0].retry_count, 0);

        // In-flight state must now be BatchedForSettlement
        assert_eq!(
            state.pending_locks.get(&1001).unwrap().state,
            NoncePhase::BatchedForSettlement
        );

        // Second sweep while in flight: MUST return empty! (Prevents duplicate broadcast / double spend)
        let swept_again = state.extract_and_lock_intent_vouchers();
        assert!(swept_again.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_failure_revert_and_terminal_failed() {
        let state = Arc::new(ServerState::new(Some(EngineConfig::default())));
        let agent = [1u8; 32];
        let provider = [2u8; 32];
        let sig = [3u8; 64];

        let voucher = VoucherPayload::new(2001, agent, provider, 50_000, sig);
        let guard = state.try_acquire_nonce_sync(agent, 2001).unwrap();
        guard.commit_with_voucher(NoncePhase::IntentSigned(sig), voucher).unwrap();

        // 1. First sweep: locked to BatchedForSettlement
        let mut swept = state.extract_and_lock_intent_vouchers();
        assert_eq!(swept.len(), 1);

        // Simulate failure #1 (retry 0 -> 1 < 3): revert to IntentSigned
        swept[0].retry_count += 1;
        state.promote_phase_with_voucher_sync(
            swept[0].nonce,
            NoncePhase::IntentSigned(swept[0].signature),
            swept[0].clone(),
        ).unwrap();

        // 2. Next sweep picks it up with retry_count = 1
        let mut swept_2 = state.extract_and_lock_intent_vouchers();
        assert_eq!(swept_2.len(), 1);
        assert_eq!(swept_2[0].retry_count, 1);

        // Simulate failure #2 (retry 1 -> 2 < 3)
        swept_2[0].retry_count += 1;
        state.promote_phase_with_voucher_sync(
            swept_2[0].nonce,
            NoncePhase::IntentSigned(swept_2[0].signature),
            swept_2[0].clone(),
        ).unwrap();

        // 3. Next sweep picks it up with retry_count = 2
        let mut swept_3 = state.extract_and_lock_intent_vouchers();
        assert_eq!(swept_3.len(), 1);
        assert_eq!(swept_3[0].retry_count, 2);

        // Simulate failure #3 (retry 2 -> 3 >= 3): Terminal failure!
        swept_3[0].retry_count += 1;
        state.mark_voucher_failed(swept_3[0].nonce, &swept_3[0].agent).unwrap();

        // State is now Failed permanently
        assert_eq!(
            state.pending_locks.get(&2001).unwrap().state,
            NoncePhase::Failed
        );

        // Next sweep MUST ignore it
        let swept_terminal = state.extract_and_lock_intent_vouchers();
        assert!(swept_terminal.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_success_finalized() {
        let state = Arc::new(ServerState::new(Some(EngineConfig::default())));
        let agent = [1u8; 32];
        let provider = [2u8; 32];
        let sig = [3u8; 64];

        let voucher = VoucherPayload::new(3001, agent, provider, 50_000, sig);
        let guard = state.try_acquire_nonce_sync(agent, 3001).unwrap();
        guard.commit_with_voucher(NoncePhase::IntentSigned(sig), voucher).unwrap();

        let swept = state.extract_and_lock_intent_vouchers();
        assert_eq!(swept.len(), 1);

        // Settle success: promote to Finalized
        state.promote_phase_sync(swept[0].nonce, NoncePhase::Finalized).unwrap();

        assert_eq!(
            state.pending_locks.get(&3001).unwrap().state,
            NoncePhase::Finalized
        );

        // Subsequent sweep ignores Finalized
        assert!(state.extract_and_lock_intent_vouchers().is_empty());
    }
}
