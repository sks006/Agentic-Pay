use anchor_lang::prelude::*;

use crate::contexts::OwnerOnly;

pub fn handler(ctx: Context<OwnerOnly>, paused: bool) -> Result<()> {
    ctx.accounts.escrow.paused = paused;
    Ok(())
}