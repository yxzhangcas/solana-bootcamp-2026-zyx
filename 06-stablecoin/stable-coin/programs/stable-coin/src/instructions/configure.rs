use anchor_lang::prelude::*;

use crate::{Config, MinterConfig, error::StablecoinError};

/// Configure a minter with a specific allowance
/// Only the admin can call this instruction
/// If the minter already exists, this updates their allowance
pub fn handle_configure_minter(ctx: Context<ConfigureMinter>, allowance: u64) -> Result<()> {
    let minter_config = &mut ctx.accounts.minter_config;

    // If not initialized, set the minter address
    if !minter_config.is_initialized {
        minter_config.minter = ctx.accounts.minter.key();
        minter_config.amount_minted = 0;
        minter_config.is_initialized = true;
        minter_config.bump = ctx.bumps.minter_config;
    }

    minter_config.allowance = allowance;    // 挖矿上限可以更新（上调或者下调）

    msg!(
        "Configured minter {} with allowance {}",
        ctx.accounts.minter.key(),
        allowance
    );

    Ok(())
}

#[derive(Accounts)]
pub struct ConfigureMinter<'info> {
    /// Only the admin can configure minters
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

    /// The minter being configured
    /// CHECK: This can be any account that will be authorized to mint
    pub minter: UncheckedAccount<'info>,

    /// The minter's configuration account
    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + MinterConfig::INIT_SPACE,
        seeds = [b"minter", minter.key().as_ref()],
        bump
    )]
    pub minter_config: Account<'info, MinterConfig>,

    pub system_program: Program<'info, System>,
}
