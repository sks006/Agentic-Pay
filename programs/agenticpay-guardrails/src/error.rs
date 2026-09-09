

#[error_code]
pub enum GuardrailError {
    #[msg("Emergency pause active: all agent payments are currently halted")]
    ProgramPaused,

    #[msg("Payment amount exceeds max per-transaction limit")]
    MaxTxLimitExceeded,

    #[msg("Payment amount exceeds 24-hour daily spend limit")]
    DailyLimitExceeded,

    #[msg("Payment amount exceeds lifetime overall spend cap")]
    OverallCapExceeded,

    #[msg("Payment amount exceeds session-key allowance")]
    SessionAllowanceExceeded,

    #[msg("Session key has expired")]
    SessionKeyExpired,

    #[msg("Session key has been revoked")]
    SessionKeyRevoked,

    #[msg("Invalid spend limits: must satisfy max_tx <= daily_limit <= overall_cap")]
    InvalidLimits,

    #[msg("Invalid expiry time: must be in the future")]
    InvalidExpiry,

    #[msg("Numerical overflow calculating spend accumulation")]
    NumericalOverflow,
}
