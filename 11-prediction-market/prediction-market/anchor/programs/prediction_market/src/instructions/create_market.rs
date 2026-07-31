use anchor_lang::prelude::*;

use crate::{constants::MAX_QUESTION_LEN, error::MarketError, state::Market};

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
      init,
      payer = creator,
      space = 8 + Market::INIT_SPACE,
      seeds = [b"market", creator.key().as_ref(), &market_id.to_le_bytes()],
      bump
    )]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_market(
    ctx: Context<CreateMarket>,
    market_id: u64,
    question: String,
    resolution_time: i64,
) -> Result<()> {
    require!(question.len() <= MAX_QUESTION_LEN, MarketError::Overflow);
    let clock = Clock::get()?;
    require!(
        resolution_time > clock.unix_timestamp,
        MarketError::ResolutionTimeInPast
    );

    let market = &mut ctx.accounts.market;
    market.creator = ctx.accounts.creator.key();
    market.market_id = market_id;
    market.question = question;
    market.resolution_time = resolution_time;
    market.yes_pool_lamports = 0;
    market.no_pool_lamports = 0;
    market.resolved = false;
    market.outcome = None;
    market.bump = ctx.bumps.market;

    Ok(())
}
