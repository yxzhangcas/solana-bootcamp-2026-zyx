pub mod constants;
pub mod dynamic_fees;
pub mod error;
pub mod instructions;
pub mod math;
pub mod oracle;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("3PTUwmwAKadewk4owwcpm84uMjz1QduZ5UuEGqYqisDm");

#[program]
pub mod stable_swap {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        amplification: u64,
        base_fee_bps: u16,
        max_dynamic_fee_bps: u16,
        depeg_threshold_bps: u16,
        max_price_age_sec: u64,
    ) -> Result<()> {
        initialize_pool::initialize_pool_handler(
            ctx,
            amplification,
            base_fee_bps,
            max_dynamic_fee_bps,
            depeg_threshold_bps,
            max_price_age_sec,
        )
    }

    // 可以自定义汇率进行流动性添加（偏移通常意味着亏损，除非知道后面的走势）
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_out: u64,
    ) -> Result<()> {
        add_liquidity::add_liquidity_handler(ctx, amount_a, amount_b, min_lp_out)
    }

    // 流动性撤回只能根据当前池子中的兑换比例进行
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_amount: u64,
        min_a_out: u64,
        min_b_out: u64,
    ) -> Result<()> {
        remove_liquidity::remove_liquidity_handler(ctx, lp_amount, min_a_out, min_b_out)
    }

    pub fn check_depeg(ctx: Context<CheckDepeg>) -> Result<()> {
        check_depeg::check_depeg_handler(ctx)
    }

    pub fn swap<'info>(
        ctx: Context<'info, Swap<'info>>,
        amount_in: u64,
        min_amount_out: u64,
        input_index: u8,
        output_index: u8,
    ) -> Result<()> {
        swap::swap_handler(ctx, amount_in, min_amount_out, input_index, output_index)
    }
}
