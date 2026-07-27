use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{error::StableSwapError, math::calculate_withdraw_amounts, Pool, MINIMUM_LIQUIDITY};

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint_a: Box<Account<'info, Mint>>,
    pub token_mint_b: Box<Account<'info, Mint>>,
    #[account(
      mut,
      seeds = [b"pool", lp_mint.key().as_ref()],
      bump,
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
      mut,
      constraint = lp_mint.key() == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub lp_mint: Box<Account<'info, Mint>>,
    #[account(
      mut,
      constraint = vault_a.mint == token_mint_a.key() @ StableSwapError::InvalidVault,
      constraint = vault_a.owner == pool.key() @ StableSwapError::InvalidVault,
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(
      mut,
      constraint = vault_b.mint == token_mint_b.key() @ StableSwapError::InvalidVault,
      constraint = vault_b.owner == pool.key() @ StableSwapError::InvalidVault,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,
    #[account(
      mut,
      constraint = user_token_a.mint == token_mint_a.key() @ StableSwapError::InvalidMint,
    )]
    pub user_token_a: Box<Account<'info, TokenAccount>>,
    #[account(
      mut,
      constraint = user_token_b.mint == token_mint_b.key() @ StableSwapError::InvalidMint,
    )]
    pub user_token_b: Box<Account<'info, TokenAccount>>,
    #[account(
      mut,
      constraint = user_lp_token.mint == pool.lp_mint @ StableSwapError::InvalidMint,
    )]
    pub user_lp_token: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn remove_liquidity_handler(
    ctx: Context<RemoveLiquidity>,
    lp_amount: u64,
    min_a_out: u64,
    min_b_out: u64,
) -> Result<()> {
    require!(lp_amount > 0, StableSwapError::ZeroAmount);

    let pool = &ctx.accounts.pool;
    let reserve_a = ctx.accounts.vault_a.amount as u128;
    let reserve_b = ctx.accounts.vault_b.amount as u128;
    let lp_supply = ctx.accounts.lp_mint.supply as u128;
    // 池子有最小流动性限制，这部分流动性不铸造lp_mint
    let pool_liquidity = (lp_supply as u128)
        .checked_add(MINIMUM_LIQUIDITY as u128)
        .ok_or(StableSwapError::MathOverflow)?;
    require!(pool_liquidity > 0, StableSwapError::EmptyPool);
    require!(
        lp_amount as u128 <= lp_supply as u128,
        StableSwapError::InsufficientLiquidity
    );

    let withdraw_amounts =
        calculate_withdraw_amounts(&[reserve_a, reserve_b], lp_amount as u128, pool_liquidity)?;
    let amount_a = withdraw_amounts[0];
    let amount_b = withdraw_amounts[1];
    require!(amount_a >= min_a_out, StableSwapError::SlippageExceeded);
    require!(amount_b >= min_b_out, StableSwapError::SlippageExceeded);

    // 进行币转移
    token::burn(
        CpiContext::new(
            token::ID,
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;
    let seeds: &[&[u8]] = &[b"pool", pool.lp_mint.as_ref(), &[pool.bump]];
    if amount_a > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                token::ID,
                Transfer {
                    from: ctx.accounts.vault_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            amount_a,
        )?;
    }
    if amount_b > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                token::ID,
                Transfer {
                    from: ctx.accounts.vault_b.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            amount_b,
        )?;
    }
    msg!(
        "Removed liquidity: lp_burned={} a_out={} b_out={}",
        lp_amount,
        amount_a,
        amount_b
    );
    Ok(())
}
