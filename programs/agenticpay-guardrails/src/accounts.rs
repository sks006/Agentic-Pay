use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ID;

use crate::state::{Escrow, SessionKey};

#[derive(Accounts)]
pub struct InitializeEscrow<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Escrow::LEN,
        seeds = [b"escrow", owner.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: agent pubkey is stored but not a signer.
    pub agent: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OwnerOnly<'info> {
    #[account(mut, has_one = owner)]
    pub escrow: Account<'info, Escrow>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterSession<'info> {
    #[account(mut, has_one = owner)]
    pub escrow: Account<'info, Escrow>,
    #[account(
        init,
        payer = owner,
        space = 8 + SessionKey::LEN,
        seeds = [b"session", escrow.key().as_ref(), session_key.key().as_ref()],
        bump
    )]
    pub session: Account<'info, SessionKey>,
    /// CHECK: session key pubkey stored in account.
    pub session_key: UncheckedAccount<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchSettleVouchers<'info> {
    #[account(
        mut,
        seeds = [b"escrow", escrow.owner.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    /// CHECK: provider receives lamports.
    #[account(mut)]
    pub provider: UncheckedAccount<'info>,
    /// CHECK: instructions sysvar for Ed25519 verification.
    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}