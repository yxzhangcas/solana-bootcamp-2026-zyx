use anchor_lang::prelude::*;

use crate::constants::MAX_QUESTION_LEN;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub creator: Pubkey,
    pub market_id: u64,
    #[max_len(MAX_QUESTION_LEN)]
    pub question: String,
    pub resolution_time: i64,
    pub yes_pool_lamports: u64,
    pub no_pool_lamports: u64,
    pub resolved: bool,
    pub outcome: Option<bool>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserPosition {
    pub market: Pubkey,
    pub user: Pubkey,
    pub yes_amount: u64,
    pub no_amount: u64,
    pub claimed: bool,
    pub bump: u8,
}
