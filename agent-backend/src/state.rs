use dashmap::DashMap;
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use std::sync::atomic::{AtomicUsize,Ordering};
use std::sync::Arc;
use std::time::{Duration,Instant};
use tracing::{debug,info,warn};

// Trait for solana RPC (dependency inversion)----------

/// Abstraction over the solana RPC client to allow mocking in tests

pub trait SolanaRpcAdapter:Send+Sync{
    fn get_balance(&self, pubkey:&[u8;32])-> Result<u64,Box<dyn std::error::Error + Send + Sync>>;
    fn send_transaction(&self,tx_data:& )
}

