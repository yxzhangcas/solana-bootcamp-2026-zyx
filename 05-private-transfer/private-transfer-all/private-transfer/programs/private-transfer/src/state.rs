use anchor_lang::prelude::*;

use crate::{error::PrivateTransfersError, ROOT_HISTORY_SIZE};

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub next_leaf_index: u64,
    pub total_deposits: u64,
    pub current_root_index: u64,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
}

impl Pool {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
}

#[account]
#[derive(InitSpace)]
pub struct NullifierSet {
    pub pool: Pubkey,
    #[max_len(256)]
    pub nullifiers: Vec<[u8; 32]>,
}

impl NullifierSet {
    pub fn is_nullifier_used(&self, nullifier_hash: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier_hash)
    }

    pub fn mark_nullifier_used(&mut self, nullifier_hash: [u8; 32]) -> Result<()> {
        require!(
            self.nullifiers.len() < 256,
            PrivateTransfersError::NullifierSetFull
        );
        self.nullifiers.push(nullifier_hash);
        Ok(())
    }
}
