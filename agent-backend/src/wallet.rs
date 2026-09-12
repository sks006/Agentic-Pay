//! Agent wallet and key management.
//!
//! Provides cryptographic signing and public key management for agents
//! using Ed25519 (`ed25519-dalek` / Solana Keypair).

pub use ed25519_dalek::{Signer, SigningKey};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;

use crate::error::EngineError;

/// The agent's local cryptographic wallet.
#[derive(Clone)]
pub struct AgentWallet {
    signing_key: SigningKey,
}

impl AgentWallet {
    /// Create an `AgentWallet` from a hex-encoded or base58-encoded secret string.
    pub fn from_hex_secret(secret_str: &str) -> Result<Self, EngineError> {
        let trimmed = secret_str.trim();

        // 1. Try decoding as hex string first
        if let Ok(bytes) = hex::decode(trimmed) {
            if bytes.len() == 32 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes);
                let signing_key = SigningKey::from_bytes(&seed);
                return Ok(Self { signing_key });
            } else if bytes.len() == 64 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes[0..32]);
                let signing_key = SigningKey::from_bytes(&seed);
                return Ok(Self { signing_key });
            }
        }

        // 2. Try decoding as base58 string
        if let Ok(bytes) = bs58::decode(trimmed).into_vec() {
            if bytes.len() == 32 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes);
                let signing_key = SigningKey::from_bytes(&seed);
                return Ok(Self { signing_key });
            } else if bytes.len() == 64 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes[0..32]);
                let signing_key = SigningKey::from_bytes(&seed);
                return Ok(Self { signing_key });
            }
        }

        Err(EngineError::InvalidPubkey(format!(
            "Failed to parse secret key as 32/64 byte hex or base58: length={}",
            trimmed.len()
        )))
    }

    /// Create an `AgentWallet` directly from an existing `SigningKey`.
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    /// Create an `AgentWallet` from a Solana `Keypair`.
    pub fn from_keypair(keypair: &Keypair) -> Self {
        let bytes = keypair.to_bytes();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes[0..32]);
        let signing_key = SigningKey::from_bytes(&seed);
        Self { signing_key }
    }

    /// Generate a fresh random `AgentWallet`.
    pub fn new_random() -> Self {
        let keypair = Keypair::new();
        Self::from_keypair(&keypair)
    }

    /// Reference to the underlying Ed25519 `SigningKey`.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Returns the 32-byte public key as raw bytes.
    pub fn pubkey_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Returns the agent's public key as a Solana `Pubkey`.
    pub fn pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.pubkey_bytes())
    }

    /// Converts into a Solana `Keypair`.
    pub fn keypair(&self) -> Keypair {
        let bytes = self.signing_key.to_keypair_bytes();
        Keypair::try_from(bytes.as_ref()).unwrap_or_else(|_| Keypair::new())
    }

    /// Sign arbitrary message bytes with the agent's Ed25519 key.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing_key.sign(message).to_bytes()
    }
}
