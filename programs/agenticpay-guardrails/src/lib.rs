//! Agentic-Pay guardrails: on-chain escrow with spending caps,
//! session keys, and batch settlement with Ed25519 verification.

use anchor_lang::prelude::*;

#[path = "accounts.rs"]
pub mod contexts;
pub use contexts::*;
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod verification;

declare_id!("Grd1111111111111111111111111111111111111111");

#[program]
pub mod agenticpay_guardrails {
    use super::instructions;
    use super::contexts::{InitializeEscrow, OwnerOnly, RegisterSession, BatchSettleVouchers};
    use anchor_lang::prelude::*;

    /// Initialize the escrow for a given agent.
    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        daily_cap_lamports: u64,
        per_tx_cap_lamports: u64,
    ) -> Result<()> {
        instructions::initialize_escrow::handler(ctx, daily_cap_lamports, per_tx_cap_lamports)
    }

    /// Pause or unpause the escrow (emergency).
    pub fn set_paused(ctx: Context<OwnerOnly>, paused: bool) -> Result<()> {
        instructions::set_paused::handler(ctx, paused)
    }

    /// Register a session key for the agent.
    pub fn register_session(
        ctx: Context<RegisterSession>,
        session_pubkey: Pubkey,
        per_tx_limit: u64,
        expires_at: i64,
    ) -> Result<()> {
        instructions::register_session::handler(ctx, session_pubkey, per_tx_limit, expires_at)
    }

    /// Batch-settle vouchers: transfers lamports from escrow PDA to provider.
    pub fn batch_settle_vouchers(
        ctx: Context<BatchSettleVouchers>,
        canonical_message: Vec<u8>,
        provider: Pubkey,
        amount_lamports: u64,
        nonce: u64,
        expires_at: i64,
    ) -> Result<()> {
        instructions::batch_settle_vouchers::handler(
            ctx,
            canonical_message,
            provider,
            amount_lamports,
            nonce,
            expires_at,
        )
    }
}