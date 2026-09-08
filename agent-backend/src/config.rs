//! Environment and runtime configuration for agent-backend.

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub max_in_flight_per_agent: usize,
    pub nonce_ttl_secs: u64,
    pub max_rate_limit_units: u32,
    pub rate_limit_per_sec: f64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        let max_in_flight_per_agent = env::var("MAX_IN_FLIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let nonce_ttl_secs = env::var("NONCE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        let max_rate_limit_units = env::var("MAX_RATE_LIMIT_UNITS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);
        let rate_limit_per_sec = env::var("RATE_LIMIT_PER_SEC")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10.0);

        Ok(Self {
            rpc_url,
            max_in_flight_per_agent,
            nonce_ttl_secs,
            max_rate_limit_units,
            rate_limit_per_sec,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            max_in_flight_per_agent: 10,
            nonce_ttl_secs: 30,
            max_rate_limit_units: 100,
            rate_limit_per_sec: 10.0,
        }
    }
}