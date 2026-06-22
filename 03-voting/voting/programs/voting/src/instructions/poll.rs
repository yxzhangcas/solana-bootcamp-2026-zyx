use anchor_lang::prelude::*;

use crate::PollState;

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitPoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + PollState::INIT_SPACE,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_state: Account<'info, PollState>,
    pub system_program: Program<'info, System>,
}

pub fn init_poll_(
    ctx: Context<InitPoll>,
    _poll_id: u64,
    name: String,
    description: String,
    start_time: u64,
    stop_time: u64,
) -> Result<()> {
    ctx.accounts.poll_state.name = name;
    ctx.accounts.poll_state.description = description;
    ctx.accounts.poll_state.voting_start = start_time;
    ctx.accounts.poll_state.voting_stop = stop_time;
    Ok(())
}
