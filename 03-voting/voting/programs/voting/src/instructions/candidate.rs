use anchor_lang::prelude::*;

use crate::{CandidateState, PollState};

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct InitCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_state: Account<'info, PollState>,
    #[account(
        init,
        payer = signer,
        space = 8 + CandidateState::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_state: Account<'info, CandidateState>,
    pub system_program: Program<'info, System>,
}

pub fn init_candidate_(
    ctx: Context<InitCandidate>,
    _poll_id: u64,
    candidate: String,
) -> Result<()> {
    ctx.accounts.candidate_state.name = candidate;
    ctx.accounts.poll_state.option_index += 1;
    Ok(())
}
