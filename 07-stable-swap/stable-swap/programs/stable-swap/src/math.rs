use anchor_lang::prelude::*;

use crate::{
    dynamic_fees::calculate_dynamic_fee_bps, error::StableSwapError, BASIS_POINTS_DIVISOR,
    MAX_ITERATIONS,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_d_balanced() {
        let reserve = 1_000_000_000u128; // 1000 USDC (6 decimals)
        let d = compute_d(reserve, reserve, 100).unwrap();
        // For balanced pool D ≈ 2 * reserve
        assert!(d > 1_900_000_000u128);
        assert!(d < 2_100_000_000u128);
    }
    #[test]
    fn test_swap_low_slippage() {
        let reserve = 1_000_000_000_000u128; // 1M USDC (6 decimals)
        let amount_in = 1_000_000u128; // 1 USDC
        let quote = calculate_swap_output(
            reserve,
            reserve,
            amount_in,
            100,
            4,
            100,
            1_000_000_000,
            1_000_000_000,
            500,
        )
        .unwrap();
        // With A=100 and tiny trade vs huge pool: almost 1:1
        assert!(quote.amount_out > 990_000u128);
        assert!(quote.amount_out <= amount_in);
        assert_eq!(quote.dynamic_fee_bps, 4);
    }
    #[test]
    fn test_swap_large_amount_low_slippage() {
        let reserve = 1_000_000_000_000u128; // 1M USDC
        let amount_in = 100_000_000_000u128; // 100k USDC swap (10% of pool)
        let quote = calculate_swap_output(
            reserve,
            reserve,
            amount_in,
            100,
            4,
            100,
            1_000_000_000,
            1_000_000_000,
            500,
        )
        .unwrap();
        // StableSwap should give >99% output for 10% of pool swap
        let ratio = quote.amount_out * 100 / amount_in;
        assert!(ratio >= 98, "Expected >=98% output, got {}%", ratio);
        assert!(quote.dynamic_fee_bps > 4);
    }
    #[test]
    fn test_lp_mint_first_deposit() {
        let amount = 1_000_000_000u128; // 1000 tokens each
        let lp = calculate_lp_mint_amount(0, 0, amount, amount, 0, 100, 1_000).unwrap();
        // LP ≈ D - MINIMUM_LIQUIDITY ≈ 2_000_000_000 - 1_000
        assert!(lp > 1_999_000_000u64);
    }
    #[test]
    fn test_lp_mint_subsequent_deposit() {
        let reserve = 1_000_000_000u128;
        let lp_supply = 2_000_000_000u128;
        // Doubling reserves should double LP supply
        let lp = calculate_lp_mint_amount(
            reserve,
            reserve,
            reserve * 2,
            reserve * 2,
            lp_supply,
            100,
            1_000,
        )
        .unwrap();
        // LP minted should be approximately equal to current supply
        assert!(lp > 1_900_000_000u64);
        assert!(lp < 2_100_000_000u64);
    }
    #[test]
    fn test_calculate_withdraw_amounts_proportional() {
        let withdraw_amounts =
            calculate_withdraw_amounts(&[1_000_000u128, 1_000_000u128], 100_000, 1_000_000)
                .unwrap();

        assert_eq!(withdraw_amounts, vec![100_000u64, 100_000u64]);
    }
    #[test]
    fn test_calculate_withdraw_amounts_rejects_single_sided_rounding() {
        let err = calculate_withdraw_amounts(&[1u128, 1_000_000u128], 1, 1_000_000).unwrap_err();

        assert!(err
            .to_string()
            .contains("Single-sided withdrawals are not supported"));
    }
}
