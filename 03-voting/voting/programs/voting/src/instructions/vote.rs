use anchor_lang::prelude::*;

use crate::{error::ErrorCode, CandidateState, PollState};

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_state: Account<'info, PollState>,
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_state: Account<'info, CandidateState>,
}

pub fn vote_(ctx: Context<Vote>, _poll_id: u64, _candidate: String) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let poll_state = &ctx.accounts.poll_state;
    if current_time > (poll_state.voting_stop as i64) {
        return Err(ErrorCode::VotingStopped.into());
    }
    if current_time < (poll_state.voting_start as i64) {
        return Err(ErrorCode::VotingNotStarted.into());
    }
    ctx.accounts.candidate_state.votes += 1;
    Ok(())
}
