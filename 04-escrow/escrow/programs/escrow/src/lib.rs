pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("CBm7pwVU1tcwBgWtHyJaEF5qeJkns6iQSvL2A6wkcrxo");

#[program]
pub mod escrow {
    use super::*;

    pub fn exec_make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        make::make_handler(ctx, seed, receive, amount)
    }
    pub fn exec_take(ctx: Context<Take>) -> Result<()> {
        take::take_handler(ctx)
    }
}
