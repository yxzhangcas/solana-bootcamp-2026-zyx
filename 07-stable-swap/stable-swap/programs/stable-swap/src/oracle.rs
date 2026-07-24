use anchor_lang::prelude::*;
use bytemuck::{try_from_bytes, Pod, Zeroable};

use crate::{
    error::StableSwapError, BASIS_POINTS_DIVISOR, ORACLE_PRICE_SCALE, TARGET_STABLE_PRICE,
};

const PYTH_NUM_COMPONENTS: usize = 32;
const PYTH_MAGIC: u32 = 0xa1b2c3d4;
const PYTH_VERSION_2: u32 = 2;
const PYTH_ACCOUNT_TYPE_PRICE: u32 = 3;
const PYTH_STATUS_TRADING: u8 = 1;
pub const ORACLE_TARGET_EXPONENT: i32 = -9;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct PythPriceInfo {
    price: i64,
    conf: u64,
    status: u8,
    corp_act: u8,
    padding: [u8; 6],
    pub_slot: u64,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct PythRational {
    value: u64,
    numerator: i64,
    denominator: i64,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct PythPriceComp {
    publisher: Pubkey,
    aggregate: PythPriceInfo,
    latest: PythPriceInfo,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct PythPriceAccount {
    magic: u32,
    version: u32,
    account_type: u32,
    size: u32,
    price_type: u32,
    exponent: i32,
    num: u32,
    num_qt: u32,
    last_slot: u64,
    valid_slot: u64,
    ema_price: PythRational,
    ema_conf: PythRational,
    timestamp: i64,
    min_pub: u8,
    drv2: u8,
    drv3: u16,
    drv4: u32,
    product: Pubkey,
    next: Pubkey,
    prev_slot: u64,
    prev_price: i64,
    prev_conf: u64,
    prev_timestamp: i64,
    aggregate: PythPriceInfo,
    comp: [PythPriceComp; PYTH_NUM_COMPONENTS],
}

struct PythPrice {
    price: i64,
    publish_time: i64,
}

pub struct OracleStatus {
    pub price_a: u128,
    pub price_b: u128,
    pub peg_delta_a_bps: u128,
    pub peg_delta_b_bps: u128,
    pub should_pause: bool,
}

pub fn load_pair_status(
    expected_price_feed_a: &Pubkey,
    expected_price_feed_b: &Pubkey,
    price_feed_a: &AccountInfo,
    price_feed_b: &AccountInfo,
    max_price_age_sec: u64,
    depeg_threshold_bps: u16,
) -> Result<OracleStatus> {
    // 参数检查
    require_keys_eq!(
        *price_feed_a.key,
        *expected_price_feed_a,
        StableSwapError::InvalidOracleAccount
    );
    require_keys_eq!(
        *price_feed_b.key,
        *expected_price_feed_b,
        StableSwapError::InvalidOracleAccount
    );
    // 数据计算
    let price_a = load_scaled_price(price_feed_a, max_price_age_sec)?;
    let price_b = load_scaled_price(price_feed_b, max_price_age_sec)?;
    let peg_delta_a_bps = calculate_peg_delta_bps(price_a)?;
    let peg_delta_b_bps = calculate_peg_delta_bps(price_b)?;
    let should_pause = check_depeg(price_a, price_b, depeg_threshold_bps);
    // 返回结果
    Ok(OracleStatus {
        price_a,  // decimal=9 表示的价格
        price_b,
        peg_delta_a_bps,  // 价格相对于基准值偏移 单位：万分之一
        peg_delta_b_bps,
        should_pause, // 价格偏移是否超出上下限，是否需要暂停工作
    })
}

// 获取token的实际价格
fn load_scaled_price(price_account_info: &AccountInfo, max_price_age_sec: u64) -> Result<u128> {
    let clock = Clock::get()?;
    let price_account = load_price_account(price_account_info)?;
    let price = select_recent_price(&price_account, clock.unix_timestamp, max_price_age_sec)?;
    // 返回值
    scale_price(price.price, price_account.exponent)
}

// 从AccountInfo中解析出PythPriceAccount
fn load_price_account(price_account_info: &AccountInfo) -> Result<PythPriceAccount> {
    // AccountInfo -> data -> bytes -> price_account
    let data = price_account_info
        .try_borrow_data()
        .map_err(|_| error!(StableSwapError::InvalidOracleAccount))?;
    let bytes = data
        .get(..size_of::<PythPriceAccount>())
        .ok_or_else(|| error!(StableSwapError::InvalidOracleAccount))?;
    let price_account = *try_from_bytes::<PythPriceAccount>(bytes)
        .map_err(|_| error!(StableSwapError::InvalidOracleAccount))?;
    require!(
        price_account.magic == PYTH_MAGIC,
        StableSwapError::InvalidOracleAccount
    );
    require!(
        price_account.version == PYTH_VERSION_2,
        StableSwapError::InvalidOracleAccount
    );
    require!(
        price_account.account_type == PYTH_ACCOUNT_TYPE_PRICE,
        StableSwapError::InvalidOracleAccount
    );
    Ok(price_account)
}

fn select_recent_price(
    price_account: &PythPriceAccount,
    current_time: i64,
    max_price_age_sec: u64,
) -> Result<PythPrice> {
    // 根据PythPriceAccount中的PythPriceInfo中的状态，判断使用当前的价格还是历史价格
    let aggregate_price = if price_account.aggregate.status == PYTH_STATUS_TRADING {
        PythPrice {
            price: price_account.aggregate.price,
            publish_time: price_account.timestamp,
        }
    } else {
        PythPrice {
            price: price_account.prev_price,
            publish_time: price_account.prev_timestamp,
        }
    };
    // 计算时间戳偏移，确保不能偏移太多
    let age = aggregate_price.publish_time.abs_diff(current_time);
    require!(age <= max_price_age_sec, StableSwapError::StaleOraclePrice);
    require!(
        aggregate_price.price > 0,
        StableSwapError::InvalidOraclePrice
    );
    Ok(aggregate_price)
}

// 将价格统一转换到小数点后面9位decimal
fn scale_price(price: i64, exponent: i32) -> Result<u128> {
    require!(price > 0, StableSwapError::InvalidOraclePrice);
    let mut normalized = price as u128;
    if exponent > ORACLE_TARGET_EXPONENT {
        let scale = pow10((exponent - ORACLE_TARGET_EXPONENT) as u32)?;
        normalized = normalized
            .checked_mul(scale)
            .ok_or(StableSwapError::MathOverflow)?;
    } else if exponent < ORACLE_TARGET_EXPONENT {
        let scale = pow10((ORACLE_TARGET_EXPONENT - exponent) as u32)?;
        normalized = normalized
            .checked_div(scale)
            .ok_or(StableSwapError::MathOverflow)?;
    }
    Ok(normalized)
}

fn pow10(exponent: u32) -> Result<u128> {
    let mut value = 1u128;
    for _ in 0..exponent {
        value = value.checked_mul(10).ok_or(StableSwapError::MathOverflow)?;
    }
    Ok(value)
}

// 计算token的实际价格相对目标价格偏离比例(单位：万分之一)
fn calculate_peg_delta_bps(price: u128) -> Result<u128> {
    Ok(price
        .abs_diff(TARGET_STABLE_PRICE)
        .checked_mul(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(ORACLE_PRICE_SCALE)
        .ok_or(StableSwapError::MathOverflow)?)
}

// 判断两个token的实际价格相对目标价格是否偏移出了允许的范围（超出范围需要停止swap）
fn check_depeg(price_a_normalized: u128, price_b_normalized: u128, max_deviation_bps: u16) -> bool {
    // 最大运行的便宜价格
    let max_deviation = TARGET_STABLE_PRICE
        .saturating_mul(max_deviation_bps as u128)
        .saturating_div(BASIS_POINTS_DIVISOR);
    // 允许的价格区间（上下限）
    let lower_bound = TARGET_STABLE_PRICE.saturating_sub(max_deviation);
    let upper_bound = TARGET_STABLE_PRICE.saturating_add(max_deviation);
    // 判断两个token的价格是否存在超限
    let a_depegged = price_a_normalized < lower_bound || price_a_normalized > upper_bound;
    let b_depegged = price_b_normalized < lower_bound || price_b_normalized > upper_bound;
    a_depegged || b_depegged
}
