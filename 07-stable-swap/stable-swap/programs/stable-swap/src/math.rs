use anchor_lang::prelude::*;

use crate::{error::StableSwapError, BASIS_POINTS_DIVISOR, MAX_ITERATIONS};

/*
   常量和不变量：D = sum(alpha * reserve_i) (所有代币的加权数量都相同时的值，alpha为权重系数)
   [为简化计算逻辑，此处只考虑严格1:1兑换的场景，也就是alpha===1]
   常量和不变式：sum(reserve_i) = D [任意时刻池子中代币数量的总和都为不变量D]

   常量积不变量：C = mul(reserve_i) (在所有代币加权数量都相同时：C = mul(D / n) = (D/n)^n)
   常量积不变式：mul(reserve_i) = (D/n)^n [任意时刻池子中代币数量的总乘积都为不变量(D/n)^n]

   两者组合出新的不变式：k * D^(n-1) * sum(reserve_i) + mul(reserve_i) = k * D^n + (D/n)^n
   参数k的不同取值会对应不同的曲线，为0时对应x*y=c曲线，无穷大时对应x+y=c曲线
   将k设置为动态值：k = A * mul(reserve_i) / (D/n)^n
   当代币数量都相同时：k=A，靠近常量和；当代币数量偏离时，k减少，向常量积靠拢；（具体曲线也与A的取值有关）

   不变式变为：
   A * mul(reserve_i) * D^(n-1) / (D/n)^n * sum(reserve_i) + mul(reserve_i) = A * mul(reserve_i) / (D/n)^n * D^n + (D/n)^n
   => A * mul(reserve_i) * n^n * sum(reserve_i) / D + mul(reserve_i) = A * mul(reserve_i) * n^n + (D/n)^n
   => A * n^n * sum(reserve_i) + D = A * D * n^n + D^(n+1) / (n^n * mul(reserve_i)) [其中reserve_i是变量]
   => A*n^n * SUM + D = A*D*n^n + D^(n+1)/n^n/MUL
   => 当n=2时，4A(x+y) + D = 4AD + D^3/4xy (A->0，常量积；A->Infinity，常量和)

   对于上述的不变式直接进行数学求解，故使用Newton-Raphson迭代进行求值
   https://rareskills.io/post/curve-get-d-get-y
   f(D) = 4AD + D^3/4xy - D - 4A(x+y)
   f'(D) = 4A + 3D^2/4xy - 1
   D_next = D - f(D) / f'(D) -> 选取一个初始值，不断迭代，直至误差满足最小值
   令ann = 4A, dp = D^3/4xy，D_next = (ann * (SUM) + 2dp) * D / ((ann-1)*D + 2dp)

   在计算出D之后，就可以计算y(swap_out or swap_in)了，使用相同的不变式，同样使用牛顿法进行迭代计算。
*/

pub fn calculate_lp_mint_amount(
    reserve_a_before: u128,
    reserve_b_before: u128,
    reserve_a_after: u128,
    reserve_b_after: u128,
    lp_supply: u128,
    amp: u128,
    minimun_liquidity: u64, // 池子中最少流动性，这部分流动性无对应的lp_mint，不能撤回
) -> Result<u64> {
    // 新池子首次添加流动性，返回的lp_mint数量会去掉最小流动性部分
    if lp_supply == 0 {
        let d = compute_d(reserve_a_after, reserve_b_after, amp)?; // 牛顿迭代计算d
        require!(
            d > minimun_liquidity as u128,
            StableSwapError::InsufficientInitialLiquidity
        );
        let lp_to_mint = (d - minimun_liquidity as u128).min(u64::MAX as u128) as u64;
        return Ok(lp_to_mint);
    }
    let d_before = compute_d(reserve_a_before, reserve_b_before, amp)?;
    let d_after = compute_d(reserve_a_after, reserve_b_after, amp)?;
    require!(d_after >= d_before, StableSwapError::InsufficientLiquidity);

    let d_diff = d_after
        .checked_sub(d_before)
        .ok_or(StableSwapError::MathOverflow)?;
    let lp_to_mint = lp_supply
        .checked_mul(d_diff)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(d_before)
        .ok_or(StableSwapError::MathOverflow)?
        .min(u64::MAX as u128) as u64;

    Ok(lp_to_mint)
}

pub fn compute_d(reserve_a: u128, reserve_b: u128, amp: u128) -> Result<u128> {
    require!(reserve_a > 0 && reserve_b > 0, StableSwapError::EmptyPool);
    let sum = reserve_a
        .checked_add(reserve_b)
        .ok_or(StableSwapError::MathOverflow)?;
    // f(D) = 4AD + D^3/4xy - D - 4A(x+y)
    let ann = amp.checked_mul(4).ok_or(StableSwapError::MathOverflow)?; // 4A
    let mut d = sum;
    for _ in 0..MAX_ITERATIONS {
        // 迭代255次
        let d_prev = d;
        let dp = d // D^3/4xy
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(
                reserve_a
                    .checked_mul(2)
                    .ok_or(StableSwapError::MathOverflow)?,
            )
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(
                reserve_b
                    .checked_mul(2)
                    .ok_or(StableSwapError::MathOverflow)?,
            )
            .ok_or(StableSwapError::MathOverflow)?;
        // 分子：(ann * (SUM) + 2dp) * D
        let numernator = ann
            .checked_mul(sum)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(dp.checked_mul(2).ok_or(StableSwapError::MathOverflow)?)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?;
        // 分母：(ann-1)*D + 2dp
        let denominator = ann
            .checked_sub(1)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(dp.checked_mul(3).ok_or(StableSwapError::MathOverflow)?)
            .ok_or(StableSwapError::MathOverflow)?;
        d = numernator
            .checked_div(denominator)
            .ok_or(StableSwapError::MathOverflow)?;
        if d.abs_diff(d_prev) <= 1 {
            return Ok(d);
        }
    }
    Err(StableSwapError::ConvergenceFailed.into())
}

pub fn calculate_withdraw_amounts(
    reserves: &[u128],
    lp_amount: u128,
    pool_liquidity: u128,
) -> Result<Vec<u64>> {
    require!(reserves.len() == 2, StableSwapError::InvalidVault);
    require!(lp_amount > 0, StableSwapError::ZeroAmount);
    require!(pool_liquidity > 0, StableSwapError::EmptyPool);
    require!(
        reserves.iter().all(|reserve| *reserve > 0),
        StableSwapError::EmptyPool
    );

    let mut withdraw_amounts = Vec::with_capacity(reserves.len());
    for reserve in reserves {
        let amount = reserve
            .checked_mul(lp_amount)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(pool_liquidity)
            .ok_or(StableSwapError::MathOverflow)?;
        withdraw_amounts.push(amount.min(u64::MAX as u128) as u64); // 队列，压到队尾
    }
    require!(
        withdraw_amounts.iter().all(|amount| *amount > 0),
        StableSwapError::SingleSidedWithdrawalNotAllowed
    );
    Ok(withdraw_amounts)
}

pub struct SwapQuote {
    pub amount_out: u128,
    pub fee_amount: u128,
    pub dynamic_fee_bps: u16,
}
pub fn calculate_swap_output(
    reserve_in: u128,
    reserve_out: u128,
    amount_in: u128,
    amp: u128,
    base_fee_bps: u16,
    max_dynamic_fee_bps: u16,
    oracle_price_in: u128,
    oracle_price_out: u128,
    depeg_threshold_bps: u16,
) -> Result<SwapQuote> {
    require!(amount_in > 0, StableSwapError::ZeroAmount);
    let d = compute_d(reserve_in, reserve_out, amp)?;
    let new_reserve_in = reserve_in
        .checked_add(amount_in)
        .ok_or(StableSwapError::MathOverflow)?;
    let new_reserve_out = compute_y(new_reserve_in, d, amp)?;
    let amount_out_before_fee = reserve_out
        .checked_sub(new_reserve_out)
        .ok_or(StableSwapError::MathOverflow)?;
    let dynamic_fee_bps = calculate_dynamic_fee_bps(
        base_fee_bps,
        max_dynamic_fee_bps,
        new_reserve_in,
        new_reserve_out,
        oracle_price_in,
        oracle_price_out,
        depeg_threshold_bps,
    )?;
    let fee_amount = amount_out_before_fee
        .checked_mul(dynamic_fee_bps as u128)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?;
    let amount_out = amount_out_before_fee
        .checked_sub(fee_amount)
        .ok_or(StableSwapError::MathOverflow)?;

    Ok(SwapQuote {
        amount_out,
        fee_amount,
        dynamic_fee_bps,
    })
}

// https://rareskills.io/post/curve-get-d-get-y#newtons-method-formula
pub fn compute_y(reserve_other: u128, d: u128, amp: u128) -> Result<u128> {
    require!(reserve_other > 0, StableSwapError::EmptyPool);
    let ann = amp.checked_mul(4).ok_or(StableSwapError::MathOverflow)?;
    let b = reserve_other
        .checked_add(d.checked_div(ann).ok_or(StableSwapError::MathOverflow)?)
        .ok_or(StableSwapError::MathOverflow)?;
    let c = d
        .checked_mul(d)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_mul(d)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(
            reserve_other
                .checked_mul(4)
                .ok_or(StableSwapError::MathOverflow)?
                .checked_mul(ann)
                .ok_or(StableSwapError::MathOverflow)?,
        )
        .ok_or(StableSwapError::MathOverflow)?;
    let mut y = d;
    for _ in 0..MAX_ITERATIONS {
        let y_prev = y;
        let numerator = y
            .checked_mul(y)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(c)
            .ok_or(StableSwapError::MathOverflow)?;
        let denominator = y
            .checked_mul(2)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(b)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_sub(d)
            .ok_or(StableSwapError::MathOverflow)?;
        y = numerator
            .checked_div(denominator)
            .ok_or(StableSwapError::MathOverflow)?;
        if y.abs_diff(y_prev) <= 1 {
            return Ok(y);
        }
    }
    Err(StableSwapError::ConvergenceFailed.into())
}

pub fn calculate_dynamic_fee_bps(
    base_fee_bps: u16,
    max_dynamic_fee_bps: u16,
    new_reserve_in: u128,
    new_reserve_out: u128,
    oracle_price_in: u128,
    oracle_price_out: u128,
    depeg_threshold_bps: u16,
) -> Result<u16> {
    require!(
        base_fee_bps <= max_dynamic_fee_bps,
        StableSwapError::InvalidFeeConfig
    );
    require!(
        depeg_threshold_bps > 0,
        StableSwapError::InvalidDepegThreshold
    );
    let post_value_in = new_reserve_in
        .checked_mul(oracle_price_in)
        .ok_or(StableSwapError::MathOverflow)?;
    let post_value_out = new_reserve_out
        .checked_mul(oracle_price_out)
        .ok_or(StableSwapError::MathOverflow)?;

    let imbalance_bps = calculate_value_imbalance_bps(post_value_in, post_value_out)?;
    let oracle_ratio_bps = oracle_price_in
        .checked_mul(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(oracle_price_out)
        .ok_or(StableSwapError::MathOverflow)?;
    let oracle_deviation_bps = oracle_ratio_bps.abs_diff(BASIS_POINTS_DIVISOR);
    let stress_bps = imbalance_bps.max(oracle_deviation_bps);
    let stress_cap = stress_bps.min(depeg_threshold_bps as u128);
    let dynamic_range = (max_dynamic_fee_bps - base_fee_bps) as u128;

    let effective_fee = (base_fee_bps as u128)
        .checked_add(
            dynamic_range
                .checked_mul(stress_cap)
                .ok_or(StableSwapError::MathOverflow)?
                .checked_div(depeg_threshold_bps as u128)
                .ok_or(StableSwapError::MathOverflow)?,
        )
        .ok_or(StableSwapError::MathOverflow)?;

    Ok(effective_fee.min(max_dynamic_fee_bps as u128) as u16)
}

fn calculate_value_imbalance_bps(value_a: u128, value_b: u128) -> Result<u128> {
    let total_value = value_a
        .checked_add(value_b)
        .ok_or(StableSwapError::MathOverflow)?;

    if total_value == 0 {
        return Ok(0);
    }
    Ok(value_a
        .abs_diff(value_b)
        .checked_mul(BASIS_POINTS_DIVISOR)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(total_value)
        .ok_or(StableSwapError::MathOverflow)?)
}
