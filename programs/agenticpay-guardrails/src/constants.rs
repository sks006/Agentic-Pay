/// Domain separator – must match the off-chain `voucher.rs`.
pub const DOMAIN_SEPARATOR: &[u8] = b"agenticpay_x402v1";

/// Length of the canonical message (bytes).
pub const CANONICAL_MESSAGE_LEN: usize = 105;

/// Slots per day (~24 hours at 400ms per slot).
pub const SLOTS_PER_DAY: u64 = 216_000;