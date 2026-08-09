use anchor_lang::{prelude::*, system_program};

use crate::{
    error::MarketError,
    state::{Market, UserPosition},
};

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
      mut,
      seeds = [b"market", market.creator.as_ref(), &market.market_id.to_le_bytes()],
      bump = market.bump,
    )]
    pub market: Account<'info, Market>,
    #[account(
      init_if_needed,
      payer = user,
      space = 8 + UserPosition::INIT_SPACE,
      seeds = [b"position", market.key().as_ref(), user.key().as_ref()],
      bump
    )]
    pub user_position: Account<'info, UserPosition>,
    pub system_program: Program<'info, System>,
}

pub fn handle_place_bet(ctx: Context<PlaceBet>, amount: u64, bet_yes: bool) -> Result<()> {
    require!(amount > 0, MarketError::InvalidBetAmount);

    let market = &ctx.accounts.market;

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp < market.resolution_time,
        MarketError::BettingClosed
    );

    // 此处直接把资金发送给了market账户，这是个PDA，不是普通SOL账户。
    // 无法通过transfer函数反向将资金转回，不是seed的问题，是from不能有数据部分，必须是纯粹的SOL账户。
    // try_borrow_mut_lamports可以跳过CPI直接操作账户中的SOL，可以用于PDA资金转出。
    system_program::transfer(
        CpiContext::new(
            system_program::ID,
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.market.to_account_info(),
            },
        ),
        amount,
    )?;

    // 此处重新进行引用，前面只能使用不可变引用，后面必须使用可变引用
    let market = &mut ctx.accounts.market;
    if bet_yes {
        market.yes_pool_lamports = market
            .yes_pool_lamports
            .checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    } else {
        market.no_pool_lamports = market
            .no_pool_lamports
            .checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    }

    let position = &mut ctx.accounts.user_position;
    // 首次创建，进行初始化
    if position.market == Pubkey::default() {
        position.market = market.key();
        position.user = ctx.accounts.user.key();
        position.bump = ctx.bumps.user_position;
    }
    if bet_yes {
        position.yes_amount = position
            .yes_amount
            .checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    } else {
        position.no_amount = position
            .no_amount
            .checked_add(amount)
            .ok_or(MarketError::Overflow)?;
    }
    Ok(())
}
