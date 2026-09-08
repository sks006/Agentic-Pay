//! Async Solana RPC adapter using non‑blocking client and VersionedTransaction.

use crate::error::RpcAdapterFault;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    transaction::VersionedTransaction,
};
use solana_transaction_status_client_types::TransactionConfirmationStatus;
use std::sync::Arc;
use std::time::Duration;

// Re‑export the trait from `state` – unchanged.
pub use crate::state::SolanaRpcAdapter;

/// Async, non‑blocking Solana RPC client with timeout support.
pub struct AsyncSolanaProvider {
    pub client: Arc<RpcClient>,
    pub tx_timeout: Duration,
}

impl AsyncSolanaProvider {
    pub fn new(client: Arc<RpcClient>, timeout: Duration) -> Self {
        Self {
            client,
            tx_timeout: timeout,
        }
    }

    /// Helper to parse a pubkey from bytes.
    fn pubkey_from_bytes(bytes: &[u8; 32]) -> Pubkey {
        Pubkey::new_from_array(*bytes)
    }

    /// Helper to parse a signature from bytes.
    fn signature_from_bytes(bytes: &[u8; 64]) -> Signature {
        Signature::from(*bytes)
    }
}

#[async_trait::async_trait]
impl SolanaRpcAdapter for AsyncSolanaProvider {
    async fn get_balance(&self, pubkey: &[u8; 32]) -> Result<u64, RpcAdapterFault> {
        let pk = Self::pubkey_from_bytes(pubkey);
        self.client
            .get_balance(&pk)
            .await
            .map_err(RpcAdapterFault::from)
    }

    async fn send_transaction(&self, tx_data: &[u8]) -> Result<[u8; 64], RpcAdapterFault> {
        // 1. Deserialize into VersionedTransaction (supports ALTs)
        let tx: VersionedTransaction = bincode::deserialize(tx_data)
            .map_err(|e| RpcAdapterFault::Serialization(e.to_string()))?;

        // 2. Send with timeout
        let result = tokio::time::timeout(self.tx_timeout, self.client.send_transaction(&tx))
            .await
            .map_err(|_| RpcAdapterFault::Timeout(self.tx_timeout))?;

        // 3. Map result
        match result {
            Ok(sig) => Ok(sig.into()),
            Err(e) => Err(RpcAdapterFault::from(e)),
        }
    }

    async fn get_transaction_status(
        &self,
        sig: &[u8; 64],
    ) -> Result<Option<crate::state::TransactionStatus>, RpcAdapterFault> {
        let signature = Self::signature_from_bytes(sig);
        let statuses = self
            .client
            .get_signature_statuses(&[signature])
            .await
            .map_err(RpcAdapterFault::from)?;

        if let Some(Some(status)) = statuses.value.first() {
            Ok(Some(crate::state::TransactionStatus {
                confirmed: status.confirmation_status.is_some(),
                finalized: status.confirmation_status == Some(TransactionConfirmationStatus::Finalized),
                slot: Some(status.slot),
                error: status.err.as_ref().map(|e| e.to_string()),
            }))
        } else {
            Ok(None)
        }
    }
}