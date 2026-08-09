use anchor_lang::prelude::*;
// use anchor_lang::system_program;

use crate::{
    error::MarketError,
    state::{Market, UserPosition},
};

#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
      mut,
      seeds = [b"market", market.creator.as_ref(), &market.market_id.to_le_bytes()],
      bump = market.bump,
    )]
    pub market: Account<'info, Market>,
    #[account(
      mut,
      seeds = [b"position", market.key().as_ref(), user.key().as_ref()],
      bump
    )]
    pub user_position: Account<'info, UserPosition>,
    pub system_program: Program<'info, System>,
}

pub fn handle_claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
    let market = &ctx.accounts.market;
    let position = &ctx.accounts.user_position;
    require!(market.resolved, MarketError::NotResolved);
    require!(!position.claimed, MarketError::AlreadyClaimed);

    let outcome = market.outcome.unwrap();
    let (user_winning_bet, total_winning_pool, total_losing_pool) = if outcome {
        (
            position.yes_amount,
            market.yes_pool_lamports,
            market.no_pool_lamports,
        )
    } else {
        (
            position.no_amount,
            market.no_pool_lamports,
            market.yes_pool_lamports,
        )
    };
    require!(user_winning_bet > 0, MarketError::NoWinnings);

    let winnings = (user_winning_bet as u128)
        .checked_mul(total_losing_pool as u128)
        .ok_or(MarketError::Overflow)?
        .checked_div(total_winning_pool as u128)
        .ok_or(MarketError::Overflow)? as u64;
    let total_payout = user_winning_bet
        .checked_add(winnings)
        .ok_or(MarketError::Overflow)?;

    ctx.accounts.user_position.claimed = true;

    let market_account_info = ctx.accounts.market.to_account_info();
    let user_account_info = ctx.accounts.user.to_account_info();

    **market_account_info.try_borrow_mut_lamports()? -= total_payout;
    **user_account_info.try_borrow_mut_lamports()? += total_payout;

    // let seeds: &[&[&[u8]]] = &[&[
    //     b"market".as_ref(),
    //     market.creator.as_ref(),
    //     &market.market_id.to_le_bytes()[..],
    //     &[market.bump]
    // ]];
    // system_program::transfer(
    //     CpiContext::new_with_signer(
    //         ctx.accounts.system_program.key(),
    //         system_program::Transfer {
    //             from: ctx.accounts.market.to_account_info(),
    //             to: ctx.accounts.user.to_account_info(),
    //         },
    //         seeds,
    //     ),
    //     total_payout,
    // )?;

    Ok(())
}
