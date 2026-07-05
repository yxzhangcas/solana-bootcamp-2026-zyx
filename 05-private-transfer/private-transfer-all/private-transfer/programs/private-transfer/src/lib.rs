pub mod constants;
pub mod error;
pub mod event;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("3CHgWxcuX4zx4kZqQiQodd3db7qut5jcpTyfq8uUu7um");

#[program]
pub mod private_transfer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handle_initialize(ctx)
    }
    pub fn deposit(
        ctx: Context<Deposit>,
        commitment: [u8; 32],
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        deposit::handle_deposit(ctx, commitment, new_root, amount)
    }
    pub fn withdraw(
        ctx: Context<Withdraw>,
        proof: Vec<u8>,
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        recipient: Pubkey,
        amount: u64,
    ) -> Result<()> {
        withdraw::handle_withdraw(ctx, proof, nullifier_hash, root, recipient, amount)
    }
}
