//! x402 HTTP client: fetches premium data from resource servers
//! by signing vouchers and retrying with payment headers.

use crate::decision::PythPriceFeed;
use crate::error::EngineError;
use crate::voucher::{VoucherCryptography, VoucherPayload};
use crate::wallet::{AgentWallet, Signer};

use base64::Engine;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Payment terms returned by the resource server in a 402 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerms {
    pub price: u64,
    pub network: String,
    pub recipient: String,
    pub ttl: u64,
    pub scheme: String,
}

/// Full 402 response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaymentRequiredResponse {
    error: String,
    payment_required: PaymentTerms,
}

/// Response body from a successful price fetch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResponse {
    pub symbol: String,
    pub price: i64,
    #[serde(alias = "conf")]
    pub confidence: u64,
    #[serde(alias = "expo")]
    pub exponent: i32,
    pub publish_time: u64,
    pub source: String,
}

impl From<PriceResponse> for PythPriceFeed {
    fn from(r: PriceResponse) -> Self {
        PythPriceFeed {
            price: r.price,
            conf: r.confidence,
            expo: r.exponent,
            publish_time: r.publish_time,
        }
    }
}

/// x402 client for a single resource server.
pub struct X402Client {
    http: Client,
    base_url: String,
    wallet: Arc<AgentWallet>,
    /// Nonce generator: monotonically increasing.
    nonce_counter: std::sync::atomic::AtomicU64,
}

impl X402Client {
    pub fn new(base_url: impl Into<String>, wallet: Arc<AgentWallet>) -> Result<Self, EngineError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| EngineError::ProviderFault(format!("HTTP client build failed: {}", e)))?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            wallet,
            nonce_counter: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Fetch a price for the given symbol from the resource server.
    /// Handles the 402 challenge automatically.
    pub async fn fetch_price(&self, symbol: &str) -> Result<PythPriceFeed, EngineError> {
        let url = format!("{}/price/{}", self.base_url, symbol);

        // 1. Attempt request without payment.
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| EngineError::ProviderFault(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| EngineError::ProviderFault(format!("Failed to read response body: {}", e)))?;

        // 2. If 200, parse directly.
        if status == StatusCode::OK {
            let body: PriceResponse = serde_json::from_str(&text)
                .map_err(|e| EngineError::ProviderFault(format!("JSON parse failed: {} - body was: {}", e, text)))?;
            return Ok(body.into());
        }

        // 3. If 402, parse payment terms and retry.
        if status == StatusCode::PAYMENT_REQUIRED {
            let body: PaymentRequiredResponse = serde_json::from_str(&text)
                .map_err(|e| EngineError::ProviderFault(format!("402 parse failed: {} - body was: {}", e, text)))?;

            let terms = body.payment_required;

            // 4. Build and sign a voucher.
            let voucher = self.sign_voucher_for_terms(&terms)?;

            // 5. Retry with PAYMENT-SIGNATURE header.
            return self.fetch_with_voucher(&url, &voucher, &terms).await;
        }

        Err(EngineError::ProviderFault(format!(
            "Unexpected status: {} - body: {}",
            status, text
        )))
    }

    /// Sign a voucher for the given payment terms.
    pub fn sign_voucher_for_terms(&self, terms: &PaymentTerms) -> Result<VoucherPayload, EngineError> {
        // 1. Parse the recipient pubkey.
        let recipient_bytes = bs58::decode(&terms.recipient)
            .into_vec()
            .map_err(|e| EngineError::InvalidPubkey(format!("Recipient decode failed: {}", e)))?;
        if recipient_bytes.len() != 32 {
            return Err(EngineError::InvalidPubkey(
                "Recipient must be a 32-byte pubkey".into(),
            ));
        }
        let mut provider = [0u8; 32];
        provider.copy_from_slice(&recipient_bytes);

        // 2. Generate a unique nonce.
        let nonce = self
            .nonce_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // 3. Build the unsigned voucher.
        let agent = self.wallet.pubkey_bytes();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut voucher = VoucherPayload::new(
            nonce,
            agent,
            provider,
            terms.price,
            terms.ttl as i64,
        );
        // Override expiry to match the server's TTL.
        voucher.expires_at = now + terms.ttl as i64;

        // 4. Sign with the agent's keypair.
        let signing_key = self.wallet.signing_key();
        let canonical = voucher.canonical_bytes();
        let signature = signing_key.sign(&canonical);
        voucher.signature = signature.to_bytes();

        Ok(voucher)
    }

    /// Retry the request with a signed voucher in the header.
    async fn fetch_with_voucher(
        &self,
        url: &str,
        voucher: &VoucherPayload,
        _terms: &PaymentTerms,
    ) -> Result<PythPriceFeed, EngineError> {
        let encoded = encode_voucher(voucher)?;

        let response = self
            .http
            .get(url)
            .header("PAYMENT-SIGNATURE", encoded)
            .header("PAYMENT-SCHEME", "agenticpay_x402v1")
            .send()
            .await
            .map_err(|e| EngineError::ProviderFault(format!("HTTP retry failed: {}", e)))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| EngineError::ProviderFault(format!("Failed to read retry response body: {}", e)))?;

        if status == StatusCode::OK {
            let body: PriceResponse = serde_json::from_str(&text)
                .map_err(|e| EngineError::ProviderFault(format!("JSON parse failed: {} - body was: {}", e, text)))?;
            return Ok(body.into());
        }

        Err(EngineError::ProviderFault(format!(
            "Voucher rejected: {} — {}",
            status, text
        )))
    }
}

/// Encode a voucher for transmission in the PAYMENT-SIGNATURE header.
///
/// Layout (Base64-encoded):
///   canonical_bytes (105 bytes) || signature (64 bytes) || retry_count (1 byte)
pub fn encode_voucher(voucher: &VoucherPayload) -> Result<String, EngineError> {
    let mut bytes = Vec::with_capacity(105 + 64 + 1);
    bytes.extend_from_slice(&voucher.canonical_bytes());
    bytes.extend_from_slice(&voucher.signature);
    bytes.push(voucher.retry_count);
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

// ---------- Tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_voucher_layout() {
        let wallet = AgentWallet::from_hex_secret(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let agent = wallet.pubkey_bytes();
        let provider = [2u8; 32];

        let mut voucher = VoucherPayload::new(42, agent, provider, 1000, 60);
        let canonical = voucher.canonical_bytes();
        voucher.signature = wallet.signing_key().sign(&canonical).to_bytes();

        let encoded = encode_voucher(&voucher).unwrap();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        assert_eq!(decoded.len(), 170);
        assert_eq!(&decoded[0..105], &voucher.canonical_bytes());
        assert_eq!(&decoded[105..169], &voucher.signature);
        assert_eq!(decoded[169], 0);
    }
}
