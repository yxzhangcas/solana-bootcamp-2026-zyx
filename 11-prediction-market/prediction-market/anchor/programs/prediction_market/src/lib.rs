pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("23yw1EYRAvDAfX9t4LAzpkbMLYnqAKPE1SY2MAn2y8Ao");

#[program]
pub mod prediction_market {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        question: String,
        resolution_time: i64,
    ) -> Result<()> {
        crate::instructions::create_market::handle_create_market(
            ctx,
            market_id,
            question,
            resolution_time,
        )
    }
    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, bet_yes: bool) -> Result<()> {
        crate::instructions::place_bet::handle_place_bet(ctx, amount, bet_yes)
    }
    pub fn resolve_market(ctx: Context<ResolveMarket>, outcome: bool) -> Result<()> {
        crate::instructions::resolve_market::handle_resolve_market(ctx, outcome)
    }
    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        crate::instructions::claim_winnings::handle_claim_winnings(ctx)
    }
}
