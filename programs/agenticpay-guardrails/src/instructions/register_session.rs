use anchor_lang::prelude::*;

use crate::contexts::RegisterSession;

pub fn handler(
    ctx: Context<RegisterSession>,
    session_pubkey: Pubkey,
    per_tx_limit: u64,
    expires_at: i64,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    session.escrow = ctx.accounts.escrow.key();
    session.session_key = session_pubkey;
    session.per_tx_limit = per_tx_limit;
    session.expires_at = expires_at;
    session.bump = ctx.bumps.session;
    Ok(())
}