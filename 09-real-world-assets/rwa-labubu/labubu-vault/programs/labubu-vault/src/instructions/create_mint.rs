use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;

use anchor_spl::token_interface::Mint;
// use anchor_spl::token_interface;
// use anchor_lang::system_program;

use crate::{error::LabubuError, LabubuCollection};

#[derive(Accounts)]
#[instruction(labubu_id: u8)]
pub struct CreateLabubuMint<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
      seeds = [b"collection"],
      bump,
      has_one = authority
    )]
    pub collection: Account<'info, LabubuCollection>,
    /// CHECK: 代码中进行创建和初始化(此处尚未创建，跳过Check)
    // #[account(
    //   mut,
    //   seeds = [b"labubu_mint", &[labubu_id]],
    //   bump
    // )]
    // pub labubu_mint: UncheckedAccount<'info>,
    /// 此处代码可以自动创建正确的mint账户，无需写代码，比上面的方式好
    #[account(
      init,
      payer = authority,
      mint::decimals = 0,
      mint::authority = collection.key(),
      seeds = [b"labubu_mint".as_ref(), &[labubu_id]],
      bump
    )]
    pub labubu_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_create_mint(ctx: Context<CreateLabubuMint>, labubu_id: u8) -> Result<()> {
    require!(
        labubu_id >= 1 && labubu_id <= 11,
        LabubuError::InvalidLabubuId
    );
    // let rent = Rent::get()?;
    // let mint_size = 82; // 如何得来？
    // let lamports = rent.minimum_balance(mint_size);

    // let seeds: &[&[&[u8]]] = &[&[
    //     b"labubu_mint".as_ref(),
    //     &[labubu_id],
    //     &[ctx.bumps.labubu_mint],
    // ]];
    // system_program::create_account(
    //     CpiContext::new_with_signer(
    //         ctx.accounts.system_program.key(),
    //         system_program::CreateAccount {
    //             from: ctx.accounts.authority.to_account_info(),
    //             to: ctx.accounts.labubu_mint.to_account_info(),
    //         },
    //         seeds,
    //     ),
    //     lamports,
    //     mint_size as u64,
    //     &ctx.accounts.token_program.key(),
    // )?;
    // token_interface::initialize_mint2(
    //     CpiContext::new(
    //         ctx.accounts.token_program.key(),
    //         token_interface::InitializeMint2 {
    //             mint: ctx.accounts.labubu_mint.to_account_info(),
    //         },
    //     ),
    //     0,
    //     &ctx.accounts.collection.key(),
    //     None,
    // )?;

    msg!(
        "Created Labubu #{} mint: {}",
        labubu_id,
        ctx.accounts.labubu_mint.key()
    );
    Ok(())
}
