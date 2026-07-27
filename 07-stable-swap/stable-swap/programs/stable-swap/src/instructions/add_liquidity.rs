use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::{
    error::StableSwapError, math::calculate_lp_mint_amount, oracle::load_pair_status, Pool,
    DEFAULT_MAX_PRICE_AGE_SEC, MINIMUM_LIQUIDITY,
};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
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
    /// CHECK
    pub oracle_price_feed_a: UncheckedAccount<'info>,
    /// CHECK
    pub oracle_price_feed_b: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn add_liquidity_handler(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_lp_out: u64,
) -> Result<()> {
    // 至少添加一种token
    require!(amount_a > 0 || amount_b > 0, StableSwapError::ZeroAmount);

    let pool = &ctx.accounts.pool;
    require!(!pool.is_paused, StableSwapError::PoolPaused);

    let oracle_status = load_pair_status(
        &pool.oracle_config.oracle_a,
        &pool.oracle_config.oracle_b,
        &ctx.accounts.oracle_price_feed_a,
        &ctx.accounts.oracle_price_feed_b,
        DEFAULT_MAX_PRICE_AGE_SEC,
        pool.oracle_config.max_depeg_bps,
    )?;
    require!(
        !pool.oracle_config.enabled || !oracle_status.should_pause,
        StableSwapError::PoolPaused
    );

    let reserve_a = ctx.accounts.vault_a.amount as u128;
    let reserve_b = ctx.accounts.vault_b.amount as u128;
    let lp_supply = ctx.accounts.lp_mint.supply as u128;
    let amp = pool.amplification as u128;

    let new_reserve_a = reserve_a
        .checked_add(amount_a as u128)
        .ok_or(StableSwapError::MathOverflow)?;
    let new_reserve_b = reserve_b
        .checked_add(amount_b as u128)
        .ok_or(StableSwapError::MathOverflow)?;
    let lp_to_mint = calculate_lp_mint_amount(
        reserve_a,
        reserve_b,
        new_reserve_a,
        new_reserve_b,
        lp_supply,
        amp,
        MINIMUM_LIQUIDITY,
    )?;
    require!(lp_to_mint >= min_lp_out, StableSwapError::SlippageExceeded);
    require!(lp_to_mint > 0, StableSwapError::ZeroAmount);

    // 进行币转移(mint_a, mint_b, lp_mint)
    if amount_a > 0 {
        token::transfer(
            CpiContext::new(
                token::ID,
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }
    if amount_b > 0 {
        token::transfer(
            CpiContext::new(
                token::ID,
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }
    let seeds: &[&[u8]] = &[b"pool", pool.lp_mint.as_ref(), &[pool.bump]];
    token::mint_to(
        CpiContext::new_with_signer(
            token::ID,
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            &[seeds],
        ),
        lp_to_mint,
    )?;
    msg!(
        "Added liquidity: a={} b={} lp_minted={} oracle_a={}bps oracle_b={}bps",
        amount_a,
        amount_b,
        lp_to_mint,
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps
    );
    Ok(())
}
