use anchor_lang::{prelude::*, system_program};

use crate::{
    error::PrivateTransfersError, event::DepositEvent, state::Pool, MAX_LEAVES, MIN_DEPOSIT_AMOUNT,
    ROOT_HISTORY_SIZE,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Client computes commitment and new_root offchain.
/// Invalid roots will cause withdrawal proofs to fail.
fn deposit(
    ctx: Context<Deposit>,
    commitment: [u8; 32],
    new_root: [u8; 32],
    amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(
        pool.next_leaf_index < MAX_LEAVES,
        PrivateTransfersError::TreeFull
    );

    require!(
        amount >= MIN_DEPOSIT_AMOUNT,
        PrivateTransfersError::DepositTooSmall
    );

    let cpi_context = CpiContext::new(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.depositor.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
        },
    );
    system_program::transfer(cpi_context, amount)?;

    let leaf_index = pool.next_leaf_index;
    let new_root_index = ((pool.current_root_index + 1) % ROOT_HISTORY_SIZE as u64) as usize;
    pool.current_root_index = new_root_index as u64;
    pool.roots[new_root_index] = new_root;

    emit!(DepositEvent {
        commitment,
        leaf_index,
        timestamp: Clock::get()?.unix_timestamp,
        new_root,
    });

    pool.next_leaf_index += 1;
    pool.total_deposits += 1;

    msg!(
        "Deposit successful: {} lamports at leaf index {}",
        amount,
        leaf_index
    );
    Ok(())
}

pub fn handle_deposit(
    ctx: Context<Deposit>,
    commitment: [u8; 32],
    new_root: [u8; 32],
    amount: u64,
) -> Result<()> {
  deposit(ctx, commitment, new_root, amount)
}