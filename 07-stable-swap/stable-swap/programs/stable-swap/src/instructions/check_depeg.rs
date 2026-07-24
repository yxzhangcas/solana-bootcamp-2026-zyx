use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{error::StableSwapError, oracle::load_pair_status, Pool, DEFAULT_MAX_PRICE_AGE_SEC};

#[derive(Accounts)]
pub struct CheckDepeg<'info> {
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
    /// CHECK
    pub oracle_price_feed_a: UncheckedAccount<'info>,
    /// CHECK
    pub oracle_price_feed_b: UncheckedAccount<'info>,
}

pub fn check_depeg_handler(ctx: Context<CheckDepeg>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let oracle_status = load_pair_status(
        &pool.oracle_config.oracle_a,
        &pool.oracle_config.oracle_b,
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        DEFAULT_MAX_PRICE_AGE_SEC,
        pool.oracle_config.max_depeg_bps,
    )?;
    // 更新pool的状态，并非只有查询
    pool.is_paused = pool.oracle_config.enabled && oracle_status.should_pause;
    msg!(
        "Check depeg: oracle_a={}bps oracle_b={}bps threshold={}bps paused={}",
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps,
        pool.oracle_config.max_depeg_bps,
        pool.is_paused
    );
    Ok(())
}
