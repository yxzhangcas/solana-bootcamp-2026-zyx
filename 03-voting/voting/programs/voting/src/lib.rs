pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("2hq8buG1yUxcLpYxSYrTB4BgTD66K7WvAgLT5iYnYUd3");

#[program]
pub mod voting {
    use super::*;

    pub fn init_poll(
        ctx: Context<InitPoll>,
        poll_id: u64,
        name: String,
        description: String,
        start_time: u64,
        stop_time: u64,
    ) -> Result<()> {
        poll::init_poll_(ctx, poll_id, name, description, start_time, stop_time)
    }

    pub fn init_candidate(
        ctx: Context<InitCandidate>,
        poll_id: u64,
        candidate: String,
    ) -> Result<()> {
        candidate::init_candidate_(ctx, poll_id, candidate)
    }

    pub fn vote(ctx: Context<Vote>, poll_id: u64, candidate: String) -> Result<()> {
        vote::vote_(ctx, poll_id, candidate)
    }
}
