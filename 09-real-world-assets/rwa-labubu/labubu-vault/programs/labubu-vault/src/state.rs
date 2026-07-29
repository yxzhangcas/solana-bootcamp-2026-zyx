use anchor_lang::prelude::*;

use crate::TOTAL_LABUBU_TYPES;

#[account]
#[derive(InitSpace)]
pub struct LabubuCollection {
    pub authority: Pubkey,
    pub remaining_supply: [u16; TOTAL_LABUBU_TYPES as usize],
    pub total_minted: u32,
}
