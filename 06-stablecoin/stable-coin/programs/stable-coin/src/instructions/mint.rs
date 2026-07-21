use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to, MintTo, Token2022},
    token_interface::{Mint, TokenAccount},
};

use crate::{error::StablecoinError, Config, MinterConfig};

/// Mint new stablecoins to a user
/// Only authorized minters can call this instruction
/// The minter must have sufficient allowance remaining
pub fn handle_mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    let config = &ctx.accounts.config;

    // Check not paused
    require!(!config.paused, StablecoinError::Paused);

    // Check and update minter allowance
    let minter_config = &mut ctx.accounts.minter_config;
    let remaining = minter_config
        .allowance
        .checked_sub(minter_config.amount_minted)   // 需要用安全的数学运算
        .ok_or(StablecoinError::ExceedsAllowance)?;
    require!(amount <= remaining, StablecoinError::ExceedsAllowance);

    minter_config.amount_minted = minter_config
        .amount_minted
        .checked_add(amount)    // 先修改数量，再进行铸币
        .ok_or(StablecoinError::Overflow)?;

    // Create the signer seeds for the mint authority PDA
    let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[config.bump]]];

    // Mint tokens to the destination account via Token-2022
    mint_to(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    msg!(
        "Minted {} tokens to {}",
        amount,
        ctx.accounts.destination.key()
    );

    Ok(())
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    /// The minter calling this instruction
    #[account(mut)]
    pub minter: Signer<'info>,

    /// The config account
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The minter's configuration - verifies they are authorized
    #[account(
        mut,
        seeds = [b"minter", minter.key().as_ref()],
        bump = minter_config.bump,
        constraint = minter_config.is_initialized @ StablecoinError::NotMinter
    )]
    pub minter_config: Account<'info, MinterConfig>,

    /// The Token-2022 stablecoin mint
    #[account(
        mut,
        seeds = [b"mint"],
        bump = config.mint_bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// The destination Token-2022 token account (ATA) to mint to
    #[account(
        init_if_needed,
        payer = minter,
        associated_token::mint = mint,
        associated_token::authority = destination_owner,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>, // ATA

    /// CHECK: The owner of the destination token account
    pub destination_owner: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
