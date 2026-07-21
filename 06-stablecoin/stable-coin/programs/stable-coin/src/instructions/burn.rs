use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{burn, Burn, Token2022},
    token_interface::{Mint, TokenAccount},
};

use crate::Config;

/// Burn stablecoins from the caller's account
/// Anyone can burn their own tokens
/// In a real stablecoin, this would be called when users redeem for fiat
pub fn handle_burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
    burn(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!(
        "Burned {} tokens from {}",
        amount,
        ctx.accounts.token_account.key()
    );

    Ok(())
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    /// The owner of the token account burning tokens
    pub owner: Signer<'info>,

    /// The config account
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The Token-2022 stablecoin mint
    #[account(
        mut,
        seeds = [b"mint"],
        bump = config.mint_bump
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// The Token-2022 token account to burn from
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}
