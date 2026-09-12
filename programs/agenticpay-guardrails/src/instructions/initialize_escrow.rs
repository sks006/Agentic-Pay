use anchor_lang::prelude::*;

use crate::contexts::InitializeEscrow;

pub fn handler(
    ctx: Context<InitializeEscrow>,
    daily_cap_lamports: u64,
    per_tx_cap_lamports: u64,
) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;
    escrow.owner = ctx.accounts.owner.key();
    escrow.agent = ctx.accounts.agent.key();
    escrow.daily_cap_lamports = daily_cap_lamports;
    escrow.per_tx_cap_lamports = per_tx_cap_lamports;
    escrow.spent_today_lamports = 0;
    escrow.last_reset_slot = Clock::get()?.slot;
    escrow.paused = false;
    escrow.bump = ctx.bumps.escrow;
    Ok(())
}