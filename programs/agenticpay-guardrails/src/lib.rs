use anchor_lang::prelude::*;


declare_id!("Guard11111111111111111111111111111111111111");

pub const SECONDS_PER_DAY: i64 = 86_400;

#[program]
pub mod agenticpay_guardrails {
    use super::*;

    /// Initialize spending guardrails for an agent wallet.
    pub fn initialize_guardrails(
        ctx: Context<InitializeGuardrails>,
        daily_spend_limit: u64,
        max_tx_limit: u64,
        overall_spend_cap: u64,
    ) -> Result<()> {
        require!(max_tx_limit <= daily_spend_limit, GuardrailError::InvalidLimits);
        require!(daily_spend_limit <= overall_spend_cap, GuardrailError::InvalidLimits);

        let guardrails = &mut ctx.accounts.guardrails;
        let clock = Clock::get()?;

        guardrails.authority = ctx.accounts.authority.key();
        guardrails.agent = ctx.accounts.agent.key();
        guardrails.daily_spend_limit = daily_spend_limit;
        guardrails.max_tx_limit = max_tx_limit;
        guardrails.overall_spend_cap = overall_spend_cap;
        guardrails.current_daily_spend = 0;
        guardrails.current_total_spend = 0;
        guardrails.last_window_reset = clock.unix_timestamp;
        guardrails.is_paused = false;
        guardrails.bump = ctx.bumps.guardrails;

        emit!(GuardrailsInitialized {
            agent: guardrails.agent,
            authority: guardrails.authority,
            daily_limit: daily_spend_limit,
            max_tx_limit,
            overall_cap: overall_spend_cap,
        });

        Ok(())
    }

    /// Validate and record a payment transaction against the agent's on-chain guardrails.
    pub fn validate_and_record_payment(
        ctx: Context<ValidateAndRecordPayment>,
        amount: u64,
    ) -> Result<()> {
        let guardrails = &mut ctx.accounts.guardrails;
        let clock = Clock::get()?;

        // 1. Emergency circuit breaker check
        require!(!guardrails.is_paused, GuardrailError::ProgramPaused);

        // 2. Per-transaction limit check
        require!(
            amount <= guardrails.max_tx_limit,
            GuardrailError::MaxTxLimitExceeded
        );

        // 3. Rolling 24-hour daily window check and auto-reset
        if clock.unix_timestamp >= guardrails.last_window_reset.saturating_add(SECONDS_PER_DAY) {
            guardrails.current_daily_spend = 0;
            guardrails.last_window_reset = clock.unix_timestamp;
        }

        // 4. Daily spend ceiling check
        let new_daily_spend = guardrails
            .current_daily_spend
            .checked_add(amount)
            .ok_or(GuardrailError::NumericalOverflow)?;
        require!(
            new_daily_spend <= guardrails.daily_spend_limit,
            GuardrailError::DailyLimitExceeded
        );

        // 5. Lifetime overall spend cap check
        let new_total_spend = guardrails
            .current_total_spend
            .checked_add(amount)
            .ok_or(GuardrailError::NumericalOverflow)?;
        require!(
            new_total_spend <= guardrails.overall_spend_cap,
            GuardrailError::OverallCapExceeded
        );

        // Commit state updates
        guardrails.current_daily_spend = new_daily_spend;
        guardrails.current_total_spend = new_total_spend;

        emit!(PaymentRecorded {
            agent: guardrails.agent,
            amount,
            current_daily_spend: new_daily_spend,
            current_total_spend: new_total_spend,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Register a bounded ephemeral session key for low-latency deferred voucher signing.
    pub fn register_session_key(
        ctx: Context<RegisterSessionKey>,
        session_key: Pubkey,
        allowance: u64,
        valid_until: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        require!(valid_until > clock.unix_timestamp, GuardrailError::InvalidExpiry);

        let session_account = &mut ctx.accounts.session_account;
        session_account.authority = ctx.accounts.authority.key();
        session_account.guardrails = ctx.accounts.guardrails.key();
        session_account.session_key = session_key;
        session_account.allowance = allowance;
        session_account.spent = 0;
        session_account.valid_until = valid_until;
        session_account.is_revoked = false;
        session_account.bump = ctx.bumps.session_account;

        emit!(SessionKeyRegistered {
            guardrails: session_account.guardrails,
            session_key,
            allowance,
            valid_until,
        });

        Ok(())
    }

    /// Settle a batched session voucher against session key allowance and master spend caps.
    pub fn settle_voucher(
        ctx: Context<SettleVoucher>,
        amount: u64,
        voucher_nonce: u64,
    ) -> Result<()> {
        let guardrails = &mut ctx.accounts.guardrails;
        let session_account = &mut ctx.accounts.session_account;
        let clock = Clock::get()?;

        require!(!guardrails.is_paused, GuardrailError::ProgramPaused);
        require!(!session_account.is_revoked, GuardrailError::SessionKeyRevoked);
        require!(
            clock.unix_timestamp <= session_account.valid_until,
            GuardrailError::SessionKeyExpired
        );

        // Enforce session allowance cap
        let new_session_spent = session_account
            .spent
            .checked_add(amount)
            .ok_or(GuardrailError::NumericalOverflow)?;
        require!(
            new_session_spent <= session_account.allowance,
            GuardrailError::SessionAllowanceExceeded
        );

        // Daily window auto-reset on guardrails
        if clock.unix_timestamp >= guardrails.last_window_reset.saturating_add(SECONDS_PER_DAY) {
            guardrails.current_daily_spend = 0;
            guardrails.last_window_reset = clock.unix_timestamp;
        }

        // Daily limit check
        let new_daily_spend = guardrails
            .current_daily_spend
            .checked_add(amount)
            .ok_or(GuardrailError::NumericalOverflow)?;
        require!(
            new_daily_spend <= guardrails.daily_spend_limit,
            GuardrailError::DailyLimitExceeded
        );

        // Overall cap check
        let new_total_spend = guardrails
            .current_total_spend
            .checked_add(amount)
            .ok_or(GuardrailError::NumericalOverflow)?;
        require!(
            new_total_spend <= guardrails.overall_spend_cap,
            GuardrailError::OverallCapExceeded
        );

        // Commit both account updates
        session_account.spent = new_session_spent;
        guardrails.current_daily_spend = new_daily_spend;
        guardrails.current_total_spend = new_total_spend;

        emit!(VoucherSettled {
            guardrails: guardrails.key(),
            session_key: session_account.session_key,
            amount,
            voucher_nonce,
            session_spent: new_session_spent,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Trigger emergency pause (circuit breaker) stopping all agent payments immediately.
    pub fn pause(ctx: Context<ManageGuardrails>) -> Result<()> {
        let guardrails = &mut ctx.accounts.guardrails;
        guardrails.is_paused = true;

        emit!(EmergencyPauseToggled {
            guardrails: guardrails.key(),
            is_paused: true,
        });
        Ok(())
    }

    /// Resume payments after emergency pause.
    pub fn unpause(ctx: Context<ManageGuardrails>) -> Result<()> {
        let guardrails = &mut ctx.accounts.guardrails;
        guardrails.is_paused = false;

        emit!(EmergencyPauseToggled {
            guardrails: guardrails.key(),
            is_paused: false,
        });
        Ok(())
    }

    /// Adjust spending limits by program authority.
    pub fn update_limits(
        ctx: Context<ManageGuardrails>,
        new_daily_limit: u64,
        new_max_tx_limit: u64,
        new_overall_cap: u64,
    ) -> Result<()> {
        require!(new_max_tx_limit <= new_daily_limit, GuardrailError::InvalidLimits);
        require!(new_daily_limit <= new_overall_cap, GuardrailError::InvalidLimits);

        let guardrails = &mut ctx.accounts.guardrails;
        guardrails.daily_spend_limit = new_daily_limit;
        guardrails.max_tx_limit = new_max_tx_limit;
        guardrails.overall_spend_cap = new_overall_cap;

        emit!(LimitsUpdated {
            guardrails: guardrails.key(),
            daily_limit: new_daily_limit,
            max_tx_limit: new_max_tx_limit,
            overall_cap: new_overall_cap,
        });
        Ok(())
    }

    /// Revoke a session key before its expiration.
    pub fn revoke_session_key(ctx: Context<RevokeSessionKey>) -> Result<()> {
        let session_account = &mut ctx.accounts.session_account;
        session_account.is_revoked = true;

        emit!(SessionKeyRevokedEvent {
            guardrails: session_account.guardrails,
            session_key: session_account.session_key,
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Account Contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct InitializeGuardrails<'info> {
    #[account(
        init,
        payer = authority,
        space = GuardrailAccount::SPACE,
        seeds = [b"guardrails", agent.key().as_ref()],
        bump
    )]
    pub guardrails: Account<'info, GuardrailAccount>,

    /// CHECK: Agent operational wallet address
    pub agent: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ValidateAndRecordPayment<'info> {
    #[account(
        mut,
        seeds = [b"guardrails", guardrails.agent.as_ref()],
        bump = guardrails.bump,
        has_one = agent
    )]
    pub guardrails: Account<'info, GuardrailAccount>,

    pub agent: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(session_key: Pubkey)]
pub struct RegisterSessionKey<'info> {
    #[account(
        has_one = authority
    )]
    pub guardrails: Account<'info, GuardrailAccount>,

    #[account(
        init,
        payer = authority,
        space = SessionKeyAccount::SPACE,
        seeds = [b"session_key", guardrails.key().as_ref(), session_key.as_ref()],
        bump
    )]
    pub session_account: Account<'info, SessionKeyAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettleVoucher<'info> {
    #[account(
        mut,
        seeds = [b"guardrails", guardrails.agent.as_ref()],
        bump = guardrails.bump
    )]
    pub guardrails: Account<'info, GuardrailAccount>,

    #[account(
        mut,
        seeds = [b"session_key", guardrails.key().as_ref(), session_account.session_key.as_ref()],
        bump = session_account.bump,
        has_one = guardrails
    )]
    pub session_account: Account<'info, SessionKeyAccount>,

    /// The recipient/resource-server submitting the signed voucher for settlement
    pub settler: Signer<'info>,
}

#[derive(Accounts)]
pub struct ManageGuardrails<'info> {
    #[account(
        mut,
        has_one = authority
    )]
    pub guardrails: Account<'info, GuardrailAccount>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeSessionKey<'info> {
    #[account(
        has_one = authority
    )]
    pub guardrails: Account<'info, GuardrailAccount>,

    #[account(
        mut,
        seeds = [b"session_key", guardrails.key().as_ref(), session_account.session_key.as_ref()],
        bump = session_account.bump,
        has_one = guardrails
    )]
    pub session_account: Account<'info, SessionKeyAccount>,

    pub authority: Signer<'info>,
}

// ---------------------------------------------------------------------------
// Account State Definitions
// ---------------------------------------------------------------------------

#[account]
pub struct GuardrailAccount {
    pub authority: Pubkey,
    pub agent: Pubkey,
    pub daily_spend_limit: u64,
    pub max_tx_limit: u64,
    pub overall_spend_cap: u64,
    pub current_daily_spend: u64,
    pub current_total_spend: u64,
    pub last_window_reset: i64,
    pub is_paused: bool,
    pub bump: u8,
}

impl GuardrailAccount {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1;
}

#[account]
pub struct SessionKeyAccount {
    pub authority: Pubkey,
    pub guardrails: Pubkey,
    pub session_key: Pubkey,
    pub allowance: u64,
    pub spent: u64,
    pub valid_until: i64,
    pub is_revoked: bool,
    pub bump: u8,
}

impl SessionKeyAccount {
    pub const SPACE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 1 + 1;
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct GuardrailsInitialized {
    pub agent: Pubkey,
    pub authority: Pubkey,
    pub daily_limit: u64,
    pub max_tx_limit: u64,
    pub overall_cap: u64,
}

#[event]
pub struct PaymentRecorded {
    pub agent: Pubkey,
    pub amount: u64,
    pub current_daily_spend: u64,
    pub current_total_spend: u64,
    pub timestamp: i64,
}

#[event]
pub struct SessionKeyRegistered {
    pub guardrails: Pubkey,
    pub session_key: Pubkey,
    pub allowance: u64,
    pub valid_until: i64,
}

#[event]
pub struct VoucherSettled {
    pub guardrails: Pubkey,
    pub session_key: Pubkey,
    pub amount: u64,
    pub voucher_nonce: u64,
    pub session_spent: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPauseToggled {
    pub guardrails: Pubkey,
    pub is_paused: bool,
}

#[event]
pub struct LimitsUpdated {
    pub guardrails: Pubkey,
    pub daily_limit: u64,
    pub max_tx_limit: u64,
    pub overall_cap: u64,
}

#[event]
pub struct SessionKeyRevokedEvent {
    pub guardrails: Pubkey,
    pub session_key: Pubkey,
}

// ---------------------------------------------------------------------------
// Custom Errors
// ---------------------------------------------------------------------------

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


