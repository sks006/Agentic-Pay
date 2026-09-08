use dashmap::DashMap;
use governor::{Quota, RateLimiter};
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicUsize,Ordering};
use std::sync::Arc;
use std::time::{Duration,Instant};
use tracing::{debug,info,warn};

// Import the error type from the dedicated module.
use crate::error::{EngineError,RpcAdapterFault};
use serde::{Deserialize, Serialize};


// Identity Hasher for [u8,32]

#[derive(Default)]

pub struct PubkeyIdentityHasher{
    hash:u64,
}

impl Hasher for PubkeyIdentityHasher{
    fn write(&mut self, bytes:&[u8]){
        if bytes.len()>=8{
            let mut arr=[0u8;8];
            arr.copy_from_slice(&bytes[..8]);
            self.hash=u64::from_le_bytes(arr);
        }
    }
    fn write_u8(&mut self,i:u8){self.hash=i as u64;}
    fn write_u16(&mut self,i:u16){self.hash=i as u64;}
    fn write_u32(&mut self,i:u32){self.hash=i as u64;}
    fn write_u64(&mut self,i:u64){self.hash=i as u64;}
    fn write_usize(&mut self, i: usize) { self.hash = i as u64; }
    fn finish(&self) -> u64 { self.hash }
}

#[derive(Clone,Default)]
pub struct PubkeyBuildHasher;

impl BuildHasher for PubkeyBuildHasher {
    type Hasher = PubkeyIdentityHasher;
    fn build_hasher(&self) -> Self::Hasher {
        PubkeyIdentityHasher::default()
    }
}

// ---------- Core State Structures ----------

pub struct ServerState {
    pub pending_locks: Arc<DashMap<u64, NonceRecord>>,
    pub agent_inflight: Arc<DashMap<[u8; 32], AtomicUsize, PubkeyBuildHasher>>,
    pub rate_limiter: Arc<RateLimiterState>,
    pub execution_config: EngineConfig,
}

#[derive(Debug, Clone)]
pub struct NonceRecord {
    pub agent_pubkey: [u8; 32],
    pub state: NoncePhase,
    pub initiated_timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoncePhase {
    Acquired,
    IntentSigned([u8; 64]),
    BatchedForSettlement,
    Broadcasted([u8; 64]),
    Finalized,
    Failed,
}

/// Status of a submitted transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub confirmed: bool,
    pub finalized: bool,
    pub slot: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct EngineConfig {
    pub max_in_flight_per_agent: usize,
    pub nonce_ttl: Duration,
    pub max_rate_limit_units: u32,
    pub rate_limit_per_sec: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_in_flight_per_agent: 10,
            nonce_ttl: Duration::from_secs(30),
            max_rate_limit_units: 100,
            rate_limit_per_sec: 10.0,
        }
    }
}

//----------- Rate Limiter State ----------

pub struct RateLimiterState {
    limiters: Arc<DashMap<[u8; 32], Arc<governor::DefaultDirectRateLimiter>, PubkeyBuildHasher>>,
    quota: Quota,
}

impl RateLimiterState {
    pub fn new(max_burst: u32, per_sec: f64) -> Self {
        let burst = std::num::NonZeroU32::new(max_burst)
            .unwrap_or_else(|| std::num::NonZeroU32::new(1).unwrap());
        let quota = Quota::with_period(Duration::from_secs_f64(1.0 / per_sec))
            .unwrap()
            .allow_burst(burst);
        Self {
            limiters: Arc::new(DashMap::with_hasher(PubkeyBuildHasher)),
            quota,
        }
    }

    pub fn check_add_consume(&self, agent: &[u8; 32]) -> bool {
        let limiter = self
            .limiters
            .entry(*agent)
            .or_insert_with(|| Arc::new(RateLimiter::direct(self.quota)));
        limiter.check().is_ok()
    }
}


// ---------- IdempotencyStore Trait ----------

pub trait IdempotencyStore: Send +Sync{
    type StoreFault;
    fn try_acquire_nonce(&self,agent:&[u8;32],nonce:u64)->Result<NonceGuard, Self::StoreFault>;
    fn promote_phase(&self, nonce: u64, next_phase: NoncePhase) -> Result<(), Self::StoreFault>;
    fn sweep_expired(&self, ttl_threshold: Duration) -> Result<usize, Self::StoreFault>;
}


// ---------- RAII Guard (Panic‑safe, Idempotent) ----------

pub struct NonceGuard {
    state: ServerState, // cheap clone (internal Arcs)

    agent: [u8; 32],
    nonce: u64,
    completed: bool, // true if commit() was called
}

impl NonceGuard {
    /// Commit the nonce with a final phase (e.g., IntentSigned, Broadcasted)
    /// and release the concurrency budget, but **leave the nonce in the map**
    /// to block replay attacks until TTL expiry.
    pub fn commit(mut self, phase: NoncePhase) -> Result<(), EngineError> {
        self.completed = true;
        self.state.promote_phase_sync(self.nonce, phase)?;
        // Free concurrency budget
        if let Some(counter) = self.state.agent_inflight.get(&self.agent) {
            counter.fetch_sub(1, Ordering::Release);
        }
        Ok(())
    }
}


impl Drop for NonceGuard {
    fn drop(&mut self) {
        if !self.completed {
            // The task panicked or timed out before commit().
            // Mark as Failed and free the budget.

            let _ =self.state.promote_phase_sync(self.nonce, NoncePhase::Failed);
            if let Some(counter)=self.state.agent_inflight.get(&self.agent){
                counter.fetch_sub(1,Ordering::Release);
            }
        }
    }
}

//------------ ServerState Implementation ------

impl ServerState{
    pub fn new(config:Option<EngineConfig>)->Self{
        let config=config.unwrap_or_default();
        let rate_limiter= Arc::new(RateLimiterState::new(config.max_rate_limit_units,config.rate_limit_per_sec));
        Self{
            pending_locks: Arc::new(DashMap::new()),
            agent_inflight:Arc::new(DashMap::with_hasher(PubkeyBuildHasher)),
            rate_limiter,
            execution_config:config,
        }
    }
    pub fn try_acquire_nonce_sync(&self, agent:[u8;32],nonce:u64)->Result<NonceGuard,EngineError>{
        if !self.rate_limiter.check_add_consume(&agent){
            return Err(EngineError::RateLimitExceeded(agent))
        }
        let counter=self
            .agent_inflight
            .entry(agent)
            .or_insert_with(||AtomicUsize::new(0));
        let current =counter.load(Ordering::Acquire);
        if current>=self.execution_config.max_in_flight_per_agent{
             return Err(EngineError::InFlightLimitReached(agent, self.execution_config.max_in_flight_per_agent));
        }

        //insert nonce(reject if already exists)
        let record=NonceRecord{
            agent_pubkey:agent,
            state:NoncePhase::Acquired,
            initiated_timestamp:Instant::now(),
        };
        if self.pending_locks.insert(nonce,record).is_some(){
               return Err(EngineError::NonceAlreadyInUse(nonce, agent)); 
        }

        counter.fetch_add(1, Ordering::Release);
        debug!("Acquired nonce {} for agent {}", nonce, hex::encode(&agent[..4]));
        Ok(NonceGuard{
                state: self.clone(),
                agent,
                nonce,
                completed: false,
        })

    }
    pub fn promote_phase_sync(&self, nonce: u64, next_phase: NoncePhase) -> Result<(), EngineError> {
        if let Some(mut entry) = self.pending_locks.get_mut(&nonce) {
            let record = entry.value_mut();
            // Simple transition validation (you may extend this)
            record.state = next_phase;
            debug!("Nonce {} promoted to {:?}", nonce, record.state);
            Ok(())
        } else {
            Err(EngineError::NonceNotFound(nonce))
        }
    }

    /// Sweep expired nonces without cross-locking.
    pub fn sweep_expired_sync(&self) -> Result<usize, EngineError> {
        let ttl = self.execution_config.nonce_ttl;
        let now = Instant::now();
        let mut decrements = Vec::new();

        // Pass 1: Remove expired nonces and collect agents that need decrementing
        self.pending_locks.retain(|_nonce, record| {
            let expired = now.duration_since(record.initiated_timestamp) >= ttl;
            if expired {
                // Only decrement if it was never successfully finalized/broadcasted
                // (i.e., it's still Acquired or Failed)
                if record.state == NoncePhase::Acquired || record.state == NoncePhase::Failed {
                    decrements.push(record.agent_pubkey);
                }
                false // remove from map
            } else {
                true // keep
            }
        });

        // Pass 2: Decrement in-flight counters **outside** the retain lock
        for agent in decrements.iter() {
            if let Some(counter) = self.agent_inflight.get(agent) {
                counter.fetch_sub(1, Ordering::Release);
            }
        }

        let count = decrements.len();
        if count > 0 {
            info!("Cleaned {} expired nonces", count);
        }
        Ok(count)
    }

    pub fn spawn_cleanup_task(self: Arc<Self>, interval: Duration) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                match self.sweep_expired_sync() {
                    Ok(count) if count > 0 => debug!("Cleaned {} expired nonces", count),
                    Ok(_) => {}
                    Err(e) => warn!("Cleanup error: {}", e),
                }
            }
        })
    }

    pub fn in_flight_count(&self, agent: &[u8; 32]) -> usize {
        self.agent_inflight
            .get(agent)
            .map(|c| c.load(Ordering::Acquire))
            .unwrap_or(0)
    }
}

impl Clone for ServerState {
    fn clone(&self) -> Self {
        Self {
            pending_locks: Arc::clone(&self.pending_locks),
            agent_inflight: Arc::clone(&self.agent_inflight),
            rate_limiter: Arc::clone(&self.rate_limiter),
            execution_config: self.execution_config.clone(),
        }
    }
}

// ---------- IdempotencyStore Implementation ----------
impl IdempotencyStore for ServerState {
    type StoreFault = EngineError;

    fn try_acquire_nonce(&self, agent: &[u8; 32], nonce: u64) -> Result<NonceGuard, Self::StoreFault> {
        self.try_acquire_nonce_sync(*agent, nonce)
    }

    fn promote_phase(&self, nonce: u64, next_phase: NoncePhase) -> Result<(), Self::StoreFault> {
        self.promote_phase_sync(nonce, next_phase)
    }

    fn sweep_expired(&self, _ttl_threshold: Duration) -> Result<usize, Self::StoreFault> {
        self.sweep_expired_sync()
    }
}

#[async_trait::async_trait]
pub trait SolanaRpcAdapter: Send + Sync {
    async fn get_balance(&self, pubkey: &[u8; 32]) -> Result<u64, RpcAdapterFault>;
    async fn send_transaction(&self, tx_data: &[u8]) -> Result<[u8; 64], RpcAdapterFault>;
    async fn get_transaction_status(
        &self,
        sig: &[u8; 64],
    ) -> Result<Option<TransactionStatus>, RpcAdapterFault>;
}

// ---------- Tests ----------
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_acquire_and_commit() {
        let state = Arc::new(ServerState::new(None));
        let agent = [1u8; 32];

        let guard = state.try_acquire_nonce_sync(agent, 42).unwrap();
        // Commit to IntentSigned – nonce stays in map
        guard.commit(NoncePhase::IntentSigned([2u8; 64])).unwrap();

        // Duplicate acquisition should still fail (nonce is still in map)
        assert!(state.try_acquire_nonce_sync(agent, 42).is_err());

        // Wait for TTL and sweep
        tokio::time::sleep(Duration::from_millis(100)).await;
        let removed = state.sweep_expired_sync().unwrap();
        // The nonce is expired and will be removed, but it was not in Acquired/Failed
        // so it won't decrement the in-flight counter (it was already decremented on commit)
        assert_eq!(removed, 1);

        // Now we can re‑acquire
        assert!(state.try_acquire_nonce_sync(agent, 42).is_ok());
    }

    #[tokio::test]
    async fn test_guard_drop_on_panic() {
        let state = Arc::new(ServerState::new(None));
        let agent = [1u8; 32];

        {
            let guard = state.try_acquire_nonce_sync(agent, 99).unwrap();
            // Simulate panic by dropping without commit
            std::mem::drop(guard);
        }
        // Guard drop marked as Failed and freed the concurrency budget
        // Nonce remains in map until TTL
        assert!(state.try_acquire_nonce_sync(agent, 100).is_ok()); // different nonce works
        // but same nonce is still blocked
        assert!(state.try_acquire_nonce_sync(agent, 99).is_err());

        // After sweep, it will be removed
        tokio::time::sleep(Duration::from_millis(100)).await;
        let removed = state.sweep_expired_sync().unwrap();
        assert_eq!(removed, 1);
        assert!(state.try_acquire_nonce_sync(agent, 99).is_ok());
    }
}