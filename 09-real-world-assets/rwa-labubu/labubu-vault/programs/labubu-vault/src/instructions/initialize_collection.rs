use anchor_lang::prelude::*;

use crate::{constants::*, LabubuCollection};

#[derive(Accounts)]
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 22 + 4,
        seeds = [b"collection"],
        bump
    )]
    pub collection: Account<'info, LabubuCollection>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_collection(ctx: Context<InitializeCollection>) -> Result<()> {
    let collection = &mut ctx.accounts.collection;

    for i in 0..TOTAL_LABUBU_TYPES as usize {
        collection.remaining_supply[i] = if i < 10 { NORMAL_SUPPLY } else { RARE_SUPPLY };
    }
    collection.total_minted = 0;
    collection.authority = ctx.accounts.authority.key();

    msg!("Labubu Collection initialized");
    Ok(())
}
