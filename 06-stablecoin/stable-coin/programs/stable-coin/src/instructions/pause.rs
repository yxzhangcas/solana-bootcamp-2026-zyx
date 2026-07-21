use anchor_lang::prelude::*;

use crate::{error::StablecoinError, Config};

/// Pause all minting operations
/// Only the admin can call this instruction
pub fn handle_pause(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.config.paused = true;
    msg!("Stablecoin paused");
    Ok(())
}

/// Unpause minting operations
/// Only the admin can call this instruction
pub fn handle_unpause(ctx: Context<Unpause>) -> Result<()> {
    ctx.accounts.config.paused = false;
    msg!("Stablecoin unpaused");
    Ok(())
}

#[derive(Accounts)]
pub struct Pause<'info> {
    /// Only the admin can pause
    #[account(
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}

#[derive(Accounts)]
pub struct Unpause<'info> {
    /// Only the admin can unpause
    #[account(
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
}
