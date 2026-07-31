use anchor_lang::prelude::*;

use crate::{error::MarketError, state::Market};

#[derive(Accounts)]
pub struct ResolveMarket<'info> {
    #[account(
      constraint = creator.key() == market.creator
    )]
    pub creator: Signer<'info>,
    #[account(
      mut,
      seeds = [b"market", market.creator.as_ref(), &market.market_id.to_le_bytes()],
      bump = market.bump,
    )]
    pub market: Account<'info, Market>,
}

pub fn handle_resolve_market(ctx: Context<ResolveMarket>, outcome: bool) -> Result<()> {
    let clock = Clock::get()?;
    let market = &ctx.accounts.market;
    require!(
        clock.unix_timestamp >= market.resolution_time,
        MarketError::ResolutionTooEarly
    );
    require!(!market.resolved, MarketError::AlreadyResolved);

    let market = &mut ctx.accounts.market;
    market.resolved = true;
    market.outcome = Some(outcome);

    Ok(())
}
