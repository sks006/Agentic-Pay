//! Centralised error types for the agent backend.

use crate::state::NoncePhase;

/// All possible errors that can occur in the state management layer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("Rate limit exceeded for agent {0:x}")]
    RateLimitExceeded([u8; 32]),

    #[error("Agent {0:x} has reached in‑flight limit of {1}")]
    InFlightLimitReached([u8; 32], usize),

    #[error("Nonce {0} already in use for agent {1:x}")]
    NonceAlreadyInUse(u64, [u8; 32]),

    #[error("Nonce {0} not found")]
    NonceNotFound(u64),

    #[error("Invalid phase transition for nonce {0}: expected {1:?}, got {2:?}")]
    InvalidPhaseTransition(u64, NoncePhase, NoncePhase),

    #[error("Internal sweep error: {0}")]
    SweepError(String),
}