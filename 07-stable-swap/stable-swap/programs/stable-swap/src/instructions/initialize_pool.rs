use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::{Mint, Token, TokenAccount},
};

use crate::{
    error::StableSwapError, oracle::load_pair_status, OracleConfig, Pool, MAX_APP,
    MAX_DEPEG_THRESHOLD_BPS, MAX_FEE_BPS, NUM_TOKENS,
};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>, // 创建池子和Account
    pub token_mint_a: Account<'info, Mint>, // 提前创建
    pub token_mint_b: Account<'info, Mint>, // 提前创建
    #[account(
      init,
      payer = admin,
      space = Pool::LEN,
      seeds = [b"pool", lp_mint.key().as_ref()],
      bump,
    )]
    pub pool: Account<'info, Pool>, // pool依赖lp_mint才能计算出地址
    #[account(
      init,
      payer = admin,
      mint::decimals = token_mint_a.decimals,
      mint::authority = pool,
    )]
    pub lp_mint: Account<'info, Mint>, // 地址提前获取，此处根据已知地址进行创建
    #[account(
      init,
      payer = admin,
      associated_token::mint = token_mint_a,
      associated_token::authority = pool,
    )]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(
      init,
      payer = admin,
      associated_token::mint = token_mint_b,
      associated_token::authority = pool,
    )]
    pub vault_b: Account<'info, TokenAccount>,
    /// CHECK
    pub oracle_price_feed_a: UncheckedAccount<'info>,
    /// CHECK
    pub oracle_price_feed_b: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
impl<'info> InitializePool<'info> {
    pub fn validate(&self, amplification: u64, fee_bps: u16) -> Result<()> {
        require!(
            amplification > 0 && amplification <= MAX_APP,
            StableSwapError::InvalidAmplification
        );
        require!(fee_bps <= MAX_FEE_BPS, StableSwapError::InvalidFee);
        Ok(())
    }
}

pub fn initialize_pool_handler(
    ctx: Context<InitializePool>,
    amplification: u64,
    base_fee_bps: u16,
    max_dynamic_fee_bps: u16,
    depeg_threshold_bps: u16,
    max_price_age_sec: u64,
) -> Result<()> {
    /* 参数合法性校验 */
    ctx.accounts.validate(amplification, base_fee_bps)?; // ?表示不使用返回值
    require!(
        max_dynamic_fee_bps <= MAX_FEE_BPS,
        StableSwapError::InvalidFee
    );
    require!(
        base_fee_bps <= max_dynamic_fee_bps,
        StableSwapError::InvalidFeeConfig
    );
    require!(
        depeg_threshold_bps > 0 && depeg_threshold_bps <= MAX_DEPEG_THRESHOLD_BPS,
        StableSwapError::InvalidDepegThreshold
    );
    require!(max_price_age_sec > 0, StableSwapError::InvalidOracleAge);
    require!(
        ctx.accounts.token_mint_a.key() != ctx.accounts.token_mint_b.key(),
        StableSwapError::InvalidMint
    );
    require!(
        ctx.accounts.token_mint_a.decimals == ctx.accounts.token_mint_b.decimals,
        StableSwapError::InvalidMintDecimals
    );
    require!(
        ctx.accounts.oracle_price_feed_a.key() != ctx.accounts.oracle_price_feed_b.key(),
        StableSwapError::InvalidOracleAccount
    );
    // 原始代码为什么不直接在定义中使用对应的类型，而是使用Unchecked然后在这里单独进行check？
    require_keys_eq!(
        ctx.accounts.system_program.key(),
        System::id(),
        StableSwapError::InvalidSystemProgram
    );
    require_keys_eq!(
        ctx.accounts.token_program.key(),
        Token::id(),
        StableSwapError::InvalidTokenProgram
    );
    require_keys_eq!(
        ctx.accounts.associated_token_program.key(),
        AssociatedToken::id(),
        StableSwapError::InvalidAssociatedTokenProgram
    );
    let expected_vault_a =
        get_associated_token_address(&ctx.accounts.pool.key(), &ctx.accounts.token_mint_a.key());
    require!(
        ctx.accounts.vault_a.key() == expected_vault_a,
        StableSwapError::InvalidVault
    );
    let expected_vault_b =
        get_associated_token_address(&ctx.accounts.pool.key(), &ctx.accounts.token_mint_b.key());
    require!(
        ctx.accounts.vault_b.key() == expected_vault_b,
        StableSwapError::InvalidVault
    );
    let oracle_status = load_pair_status(
        &ctx.accounts.oracle_price_feed_a.key(),
        &ctx.accounts.oracle_price_feed_b.key(),
        &ctx.accounts.oracle_price_feed_a.to_account_info(),
        &ctx.accounts.oracle_price_feed_b.to_account_info(),
        max_price_age_sec,
        depeg_threshold_bps,
    )?;
    require!(!oracle_status.should_pause, StableSwapError::PoolPaused);

    // 直接对整个PDA数据进行赋值，不用逐个处理
    ctx.accounts.pool.set_inner(Pool {
        admin: ctx.accounts.admin.key(),
        lp_mint: ctx.accounts.lp_mint.key(),
        amplification,
        fee_bps: base_fee_bps,
        token_mints: [
            ctx.accounts.token_mint_a.key(),
            ctx.accounts.token_mint_b.key(),
        ],
        bump: ctx.bumps.pool,
        oracle_config: OracleConfig {
            oracle_a: ctx.accounts.oracle_price_feed_a.key(),
            oracle_b: ctx.accounts.oracle_price_feed_b.key(),
            max_depeg_bps: depeg_threshold_bps,
            emergency_fee_bps: max_dynamic_fee_bps,
            enabled: true,
        },
        is_paused: false,
    });
    msg!(
        "StableSwap pool initialized: tokens={}, A={}, fee={}bps, emergency_fee={}bps, depeg_threshold={}bps",
        NUM_TOKENS,
        amplification,
        base_fee_bps,
        max_dynamic_fee_bps,
        depeg_threshold_bps
    );
    Ok(())
}
