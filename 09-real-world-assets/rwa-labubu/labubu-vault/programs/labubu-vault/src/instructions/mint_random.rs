use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount},
};

use crate::{error::LabubuError, LabubuCollection};

#[derive(Accounts)]
#[instruction(labubu_id: u8)]
pub struct MintRandom<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
      mut,
      seeds = [b"collection"],
      bump,
    )]
    pub collection: Account<'info, LabubuCollection>,
    #[account(
      mut,
      seeds = [b"labubu_mint", &[labubu_id]],
      bump
    )]
    pub labubu_mint: InterfaceAccount<'info, Mint>,
    #[account(
      init_if_needed,
      payer = user,
      associated_token::mint = labubu_mint,
      associated_token::authority = user,
      associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handle_mint_random(ctx: Context<MintRandom>, labubu_id: u8) -> Result<()> {
    let collection = &mut ctx.accounts.collection;
    require!(
        labubu_id >= 1 && labubu_id <= 11,
        LabubuError::InvalidLabubuId
    );
    let index = (labubu_id - 1) as usize;
    require!(collection.remaining_supply[index] > 0, LabubuError::SoldOut);
    collection.remaining_supply[index] -= 1;

    let seeds: &[&[&[u8]]] = &[&[b"collection".as_ref(), &[ctx.bumps.collection]]];
    token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            token_interface::MintTo {
                mint: ctx.accounts.labubu_mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: collection.to_account_info(),
            },
            seeds,
        ),
        1,
    )?;
    collection.total_minted += 1;

    msg!(
        "User {} minted Labubu #{} ({})",
        ctx.accounts.user.key(),
        labubu_id,
        ctx.accounts.labubu_mint.key()
    );
    Ok(())
}
