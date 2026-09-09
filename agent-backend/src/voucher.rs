use solana_sdk::pubkey::Pubkey;

/// A voucher representing a deferred payment intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoucherPayload {
    pub nonce: u64,
    pub agent: [u8; 32],
    pub provider: [u8; 32],
    pub amount_lamports: u64,
    pub signature: [u8; 64],
    pub retry_count: u8, // The mandated structural boundary
}

impl VoucherPayload {
    pub fn new(
        nonce: u64,
        agent: [u8; 32],
        provider: [u8; 32],
        amount_lamports: u64,
        signature: [u8; 64],
    ) -> Self {
        Self {
            nonce,
            agent,
            provider,
            amount_lamports,
            signature,
            retry_count: 0,
        }
    }

    pub fn pubkey_from_bytes(bytes: &[u8; 32]) -> Pubkey {
        Pubkey::new_from_array(*bytes)
    }
}
