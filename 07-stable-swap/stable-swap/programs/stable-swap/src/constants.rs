use anchor_lang::prelude::*;

pub const NUM_TOKENS: usize = 2;
pub const MAX_APP: u64 = 1_000_000;
pub const MAX_FEE_BPS: u16 = 10_000;
pub const MAX_DEPEG_THRESHOLD_BPS: u16 = 5_000;

pub const ORACLE_PRICE_SCALE: u128 = 1_000_000_000;
pub const ORACLE_TARGET_EXPONENT: i32 = -9;
pub const TARGET_STABLE_PRICE: u128 = ORACLE_PRICE_SCALE;
pub const BASIS_POINTS_DIVISOR: u128 = 10_000;

pub const DEFAULT_MAX_PRICE_AGE_SEC: u64 = 60;
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

pub const MAX_ITERATIONS: u8 = 255;