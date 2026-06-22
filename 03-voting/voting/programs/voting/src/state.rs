use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PollState {
  #[max_len(32)]
  pub name: String,
  #[max_len(280)]
  pub description: String,
  pub voting_start: u64,
  pub voting_stop: u64,
  pub option_index: u64,
}

#[account]
#[derive(InitSpace)]
pub struct CandidateState {
  #[max_len(32)]
  pub name: String,
  pub votes: u64,
}