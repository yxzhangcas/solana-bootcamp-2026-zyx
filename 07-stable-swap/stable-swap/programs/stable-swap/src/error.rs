use anchor_lang::prelude::*;

#[error_code]
pub enum StableSwapError {
    #[msg("Amplification parameter must be between 1 and 1,000,000")]
    InvalidAmplification,
    #[msg("Fee exceed maximum of 100%")]
    InvalidFee,
    #[msg("Dynamic fee configuration is invalid")]
    InvalidFeeConfig,
    #[msg("Depeg threshold is outside the supported range")]
    InvalidDepegThreshold,
    #[msg("Oracle price maximum age must be greater than zero")]
    InvalidOracleAge,
    #[msg("Slippage exceeded: output less than minimum")]
    SlippageExceeded,
    #[msg("Insufficient liquidity in pool")]
    InsufficientLiquidity,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Zero amount not allowed")]
    ZeroAmount,
    #[msg("Convergence failed in Newton's method")]
    ConvergenceFailed,
    #[msg("Pool is empty")]
    EmptyPool,
    #[msg("Initial liquidity too small")]
    InsufficientInitialLiquidity,
    #[msg("Single-sided withdrawals are not supported; LP exits must stay proportional")]
    SingleSidedWithdrawalNotAllowed,
    #[msg("Invalid token index")]
    InvalidTokenIndex,
    #[msg("Input and output token must be different")]
    SameTokenSwap,
    #[msg("Invalid remaining accounts for swap instruction")]
    InvalidRemainingAccounts,
    #[msg("Invalid vault account")]
    InvalidVault,
    #[msg("Invalid token mint")]
    InvalidMint,
    #[msg("Both token mints must use the same decimals for this StableSwap pool")]
    InvalidMintDecimals,
    #[msg("Invalid oracle account")]
    InvalidOracleAccount,
    #[msg("Invalid system program account")]
    InvalidSystemProgram,
    #[msg("Invalid token program account")]
    InvalidTokenProgram,
    #[msg("Invalid associated token program account")]
    InvalidAssociatedTokenProgram,
    #[msg("Oracle price is stale")]
    StaleOraclePrice,
    #[msg("Oracle price is invalid")]
    InvalidOraclePrice,
    #[msg("Pool is paused because one of the stablecoins is outside the peg band")]
    PoolPaused,
}
