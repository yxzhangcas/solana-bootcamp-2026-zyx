use anchor_lang::prelude::*;

use crate::{
    state::{NullifierSet, Pool},
    EMPTY_ROOT,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = authority,
        space = 8 + NullifierSet::INIT_SPACE,
        seeds = [b"nullifiers", pool.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSet>,

    #[account(seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.next_leaf_index = 0;
    pool.total_deposits = 0;
    pool.current_root_index = 0;
    pool.roots[0] = EMPTY_ROOT;

    let nullifiers = &mut ctx.accounts.nullifier_set;
    nullifiers.pool = pool.key();

    msg!("Pool initialized");
    Ok(())
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    initialize(ctx)
}
