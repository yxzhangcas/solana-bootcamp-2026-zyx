use anchor_lang::prelude::*;
use anchor_spl::{token_2022::Token2022, token_interface::Mint};

use crate::Config;

/// Initialize the stablecoin mint and config
/// This creates a new Token-2022 mint with the program PDA as the mint authority
pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.mint = ctx.accounts.mint.key();
    config.paused = false;
    config.bump = ctx.bumps.config;
    config.mint_bump = ctx.bumps.mint;

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The config account that stores stablecoin settings
    #[account(
        init,
        payer = admin,
        space = 8 + Config::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    /// The Token-2022 stablecoin mint
    /// The config PDA is set as both mint authority and freeze authority
    /// 合约调用时自动创建Mint，必须设置decimal和authority参数，可选设置freeze_authority
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        mint::freeze_authority = config,
        seeds = [b"mint"],
        bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,    /// 这是个PDA Mint，没有私钥，权限由Program的唯一config控制

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
