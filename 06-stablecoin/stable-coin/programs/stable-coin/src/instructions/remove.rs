use anchor_lang::prelude::*;

use crate::{error::StablecoinError, Config, MinterConfig};

/// Remove a minter's authorization
/// Only the admin can call this instruction
/// This closes the minter config account and returns rent to admin
pub fn handle_remove_minter(_ctx: Context<RemoveMinter>) -> Result<()> {
    msg!("Minter removed");
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveMinter<'info> {
    /// Only the admin can remove minters
    #[account(
        mut,
        constraint = admin.key() == config.admin @ StablecoinError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The minter being removed
    /// CHECK: This is the minter whose config is being closed
    pub minter: UncheckedAccount<'info>,

    /// The minter's configuration account to close
    #[account(
        mut,
        close = admin,
        seeds = [b"minter", minter.key().as_ref()],
        bump = minter_config.bump
    )]
    pub minter_config: Account<'info, MinterConfig>,    // 通过close属性实现自动销毁
}
