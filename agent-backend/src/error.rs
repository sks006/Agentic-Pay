//! Centralised error types for the agent backend.

use solana_client::client_error::{ClientError, ClientErrorKind};

// Do not derive PartialEq or Eq here; ClientError and underlying network faults 
// are not easily comparable and do not belong in equality checks.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Rate limit exceeded for agent {}", hex::encode(.0))]
    RateLimitExceeded([u8; 32]),
    
    #[error("Agent {} has reached in-flight limit of {1}", hex::encode(.0))]
    InFlightLimitReached([u8; 32], usize),
    
    #[error("Nonce {0} already in use for agent {}", hex::encode(.1))]
    NonceAlreadyInUse(u64, [u8; 32]),
    
    #[error("Nonce {0} not found")]
    NonceNotFound(u64),
    
    // Abstracted to avoid circular dependencies with state.rs
    #[error("Invalid phase transition for nonce {0}")]
    InvalidPhaseTransition(u64),
    
    #[error("Internal sweep error: {0}")]
    SweepError(String),
    
    #[error("Invalid pubkey format: {0}")]
    InvalidPubkey(String),
    
    #[error("Invalid signature format: {0}")]
    InvalidSignature(String),
    

    // ---------- Network & Upstream Faults ----------
    
    #[error("Upstream rate limit exceeded (HTTP 429). Backoff required.")]
    NodeRateLimited,
    
    #[error("Transaction serialization/deserialization failed.")]
    PayloadFormatInvalid,
    
    #[error("Transaction size exceeds the Solana MTU limit (1232 bytes).")]
    TransactionTooLarge,
    
    #[error("RPC node timeout after {0} ms")]
    NetworkTimeout(u64),
    
    #[error("Node internal error: {0}")]
    ProviderFault(String),
}

/// Mechanically maps raw Solana RPC faults into the Agentic-Pay failure domain.
impl From<ClientError> for EngineError {
    fn from(err: ClientError) -> Self {
        match err.kind() {
            // Trap HTTP 429 Too Many Requests to trigger localized backoff
            ClientErrorKind::Reqwest(req_err) if req_err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) => {
                EngineError::NodeRateLimited
            }
            ClientErrorKind::Reqwest(_) => {
                EngineError::NetworkTimeout(0) // Explicit timeout fallback
            }
            ClientErrorKind::TransactionError(tx_err) => {
                EngineError::ProviderFault(format!("Transaction rejected by validator: {:?}", tx_err))
            }
            _ => EngineError::ProviderFault(err.to_string()),
        }
    }
}

// ---------- RPC Adapter Errors ----------
#[derive(Debug, thiserror::Error)]
pub enum RpcAdapterFault {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),
    #[error("Rate limited (HTTP 429) – retry after {0:?}")]
    RateLimited(std::time::Duration),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Blockchain error: {0}")]
    Blockchain(String),
}

// We can derive From<ClientError> for convenience.
impl From<ClientError> for RpcAdapterFault {
    fn from(err: ClientError) -> Self {
        match err.kind() {
            ClientErrorKind::TransactionError(e) => RpcAdapterFault::Blockchain(e.to_string()),
            ClientErrorKind::Reqwest(e) => RpcAdapterFault::Transport(e.to_string()),
            _ => RpcAdapterFault::Transport(err.to_string()),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignalFault {
    #[error("Confidence interval too wide (risk threshold exceeded)")]
    ConfidenceTooWide,
    #[error("Expected profit margin is negative or below network fee")]
    NegativeExpectedValue,
    #[error("Network fee exceeds maximum allowed cap")]
    FeeExceedsMaxCap,
}

// ---------- Crypto-Economic Faults ----------

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CryptoFault {
    #[error("Invalid Ed25519 signature")]
    InvalidSignature,
    #[error("Voucher has expired")]
    ExpiredVoucher,
    #[error("Serialization fault: {0}")]
    SerializationFault(String),
    #[error("Signing fault: {0}")]
    SigningFault(String),
}
