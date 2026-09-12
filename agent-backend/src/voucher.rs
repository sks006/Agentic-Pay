//! Voucher (deferred payment intent) handling.
//!
//! A voucher is an off‑chain, cryptographically signed promise to pay.
//! The agent signs an intent‑to‑pay, the resource server verifies it locally,
//! grants immediate data access, and batches the voucher for later on‑chain
//! settlement via `worker.rs`.

use base64::Engine;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::CryptoFault;
/// Domain separator for the signature scheme (prevents cross‑protocol replay).
pub const DOMAIN_SEPARATOR: &[u8] = b"agenticpay_x402v1";

/// Default voucher TTL (60 seconds).
pub const DEFAULT_VOUCHER_TTL_SECS: i64 = 60;

/// A voucher representing a deferred payment intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoucherPayload {
    pub nonce: u64,
    pub agent: [u8; 32],
    pub provider: [u8; 32],
    pub amount_lamports: u64,
    /// Unix timestamp (seconds) when the voucher expires.
    pub expires_at: i64,
    /// Ed25519 signature over the canonical bytes.
    pub signature: [u8; 64],
    /// Number of settlement retries (incremented by the worker on failure).
    pub retry_count: u8,
}


impl VoucherPayload {
    /// Construct a new voucher (unsigned). Use `sign_voucher()` to sign it.
    pub fn new(
        nonce: u64,
        agent: [u8; 32],
        provider: [u8; 32],
        amount_lamports: u64,
        ttl_secs: i64,
    ) -> Self {
        let now = current_timestamp_secs();
        Self {
            nonce,
            agent,
            provider,
            amount_lamports,
            expires_at: now + ttl_secs,
            signature: [0u8; 64],
            retry_count: 0,
        }
    }

    /// Convert a 32‑byte array into a Solana `Pubkey`.
    pub fn pubkey_from_bytes(bytes: &[u8; 32]) -> Pubkey {
        Pubkey::new_from_array(*bytes)
    }

    /// Convert the voucher's agent into a Solana `Pubkey`.
    pub fn agent_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.agent)
    }

    /// Convert the voucher's provider into a Solana `Pubkey`.
    pub fn provider_pubkey(&self) -> Pubkey {
        Pubkey::new_from_array(self.provider)
    }
}

// ---------- Cryptographic Trait ----------

pub trait VoucherCryptography {
    /// Serialize the voucher into a deterministic, fixed‑width byte array.
    fn canonical_bytes(&self) -> Vec<u8>;

    /// Sign the voucher with the agent's keypair and store the signature.
    fn sign_voucher(&mut self, agent_keypair: &Keypair) -> Result<(), CryptoFault>;

    /// Verify the signature against the canonical bytes and the agent's pubkey.
    fn verify_voucher(&self) -> Result<(), CryptoFault>;

    /// Check whether the voucher has expired relative to the given timestamp.
    fn is_expired(&self, current_timestamp: i64) -> bool;
}

impl VoucherCryptography for VoucherPayload {
    fn canonical_bytes(&self) -> Vec<u8> {
        // Exact capacity:
        //   17 (domain separator)
        // +  8 (nonce)
        // + 32 (agent)
        // + 32 (provider)
        // +  8 (amount_lamports)
        // +  8 (expires_at)
        // = 105 bytes
        let mut buf = Vec::with_capacity(105);
        buf.extend_from_slice(DOMAIN_SEPARATOR);
        buf.extend_from_slice(&self.nonce.to_le_bytes());
        buf.extend_from_slice(&self.agent);
        buf.extend_from_slice(&self.provider);
        buf.extend_from_slice(&self.amount_lamports.to_le_bytes());
        buf.extend_from_slice(&self.expires_at.to_le_bytes());
        buf
    }

    fn sign_voucher(&mut self, agent_keypair: &Keypair) -> Result<(), CryptoFault> {
        // Ensure the agent field matches the keypair's pubkey.
        let expected = agent_keypair.pubkey().to_bytes();
        if expected != self.agent {
            return Err(CryptoFault::SigningFault(
                "Agent pubkey does not match keypair".into(),
            ));
        }

        let message = self.canonical_bytes();
        let signature = agent_keypair.sign_message(&message);
        self.signature = signature.into();
        Ok(())
    }

    fn verify_voucher(&self) -> Result<(), CryptoFault> {
        // 1. Reconstruct the pubkey.
        let pubkey = Pubkey::new_from_array(self.agent);
        // 2. Reconstruct the signature.
        let signature = Signature::from(self.signature);
        // 3. Canonical bytes.
        let message = self.canonical_bytes();
        // 4. Verify.
        if signature.verify(pubkey.as_ref(), &message) {
            Ok(())
        } else {
            Err(CryptoFault::InvalidSignature)
        }
    }

    fn is_expired(&self, current_timestamp: i64) -> bool {
        current_timestamp > self.expires_at
    }
}

// ---------- Time Utilities ----------

/// Return the current Unix timestamp in seconds.
pub fn current_timestamp_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ---------- HTTP Header Encoding ----------

/// Encode a voucher into a compact Base64 string for the `PAYMENT-SIGNATURE` header.
pub fn encode_voucher_header(voucher: &VoucherPayload) -> String {
    let mut bytes = Vec::with_capacity(105 + 64 + 1);
    bytes.extend_from_slice(&voucher.canonical_bytes());
    bytes.extend_from_slice(&voucher.signature);
    bytes.push(voucher.retry_count);
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Decode a voucher from a `PAYMENT-SIGNATURE` header value.
pub fn decode_voucher_header(encoded: &str) -> Result<VoucherPayload, CryptoFault> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| CryptoFault::SerializationFault(e.to_string()))?;

    if bytes.len() != 105 + 64 + 1 {
        return Err(CryptoFault::SerializationFault(format!(
            "Unexpected voucher length: {}",
            bytes.len()
        )));
    }

    let mut nonce_bytes = [0u8; 8];
    nonce_bytes.copy_from_slice(&bytes[17..25]);
    let mut agent = [0u8; 32];
    agent.copy_from_slice(&bytes[25..57]);
    let mut provider = [0u8; 32];
    provider.copy_from_slice(&bytes[57..89]);
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&bytes[89..97]);
    let mut expires_bytes = [0u8; 8];
    expires_bytes.copy_from_slice(&bytes[97..105]);
    let mut signature = [0u8; 64];
    signature.copy_from_slice(&bytes[105..169]);
    let retry_count = bytes[169];

    Ok(VoucherPayload {
        nonce: u64::from_le_bytes(nonce_bytes),
        agent,
        provider,
        amount_lamports: u64::from_le_bytes(amount_bytes),
        expires_at: i64::from_le_bytes(expires_bytes),
        signature,
        retry_count,
    })
}

// ---------- Unit Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_keypair() -> Keypair {
        Keypair::new()
    }

    #[test]
    fn test_sign_and_verify_ok() {
        let keypair = make_keypair();
        let agent = keypair.pubkey().to_bytes();
        let provider = [2u8; 32];

        let mut voucher = VoucherPayload::new(42, agent, provider, 50_000, 60);
        voucher.sign_voucher(&keypair).unwrap();
        assert!(voucher.verify_voucher().is_ok());
    }

    #[test]
    fn test_verify_tampered_payload() {
        let keypair = make_keypair();
        let agent = keypair.pubkey().to_bytes();
        let provider = [2u8; 32];

        let mut voucher = VoucherPayload::new(42, agent, provider, 50_000, 60);
        voucher.sign_voucher(&keypair).unwrap();
        voucher.amount_lamports = 999_999_999;
        assert!(matches!(
            voucher.verify_voucher(),
            Err(CryptoFault::InvalidSignature)
        ));
    }

    #[test]
    fn test_expiry() {
        let keypair = make_keypair();
        let agent = keypair.pubkey().to_bytes();
        let voucher = VoucherPayload::new(42, agent, [2u8; 32], 50_000, 60);
        let now = current_timestamp_secs();
        assert!(!voucher.is_expired(now));
        assert!(voucher.is_expired(now + 120));
    }

    #[test]
    fn test_header_roundtrip() {
        let keypair = make_keypair();
        let agent = keypair.pubkey().to_bytes();
        let mut voucher = VoucherPayload::new(99, agent, [3u8; 32], 1_000_000, 60);
        voucher.sign_voucher(&keypair).unwrap();

        let encoded = encode_voucher_header(&voucher);
        let decoded = decode_voucher_header(&encoded).unwrap();
        assert_eq!(decoded.nonce, voucher.nonce);
        assert_eq!(decoded.agent, voucher.agent);
        assert_eq!(decoded.provider, voucher.provider);
        assert_eq!(decoded.amount_lamports, voucher.amount_lamports);
        assert_eq!(decoded.expires_at, voucher.expires_at);
        assert_eq!(decoded.signature, voucher.signature);
    }

    #[test]
    fn test_canonical_bytes_deterministic() {
        let keypair = make_keypair();
        let agent = keypair.pubkey().to_bytes();
        let voucher = VoucherPayload::new(1, agent, [2u8; 32], 100, 60);
        let bytes1 = voucher.canonical_bytes();
        let bytes2 = voucher.canonical_bytes();
        assert_eq!(bytes1, bytes2);
        assert_eq!(bytes1.len(), 105);
        // Verify domain separator present.
        assert_eq!(&bytes1[..17], DOMAIN_SEPARATOR);
    }
}