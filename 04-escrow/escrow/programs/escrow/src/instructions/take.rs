use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::TransferChecked,
    token_interface::{self, CloseAccount, Mint, TokenAccount, TokenInterface},
};

use crate::{error::EscrowError, state::Escrow};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>, // owner是SystemProgram，也就是SOL主账户
    #[account(
      mut,
      close = maker,
      seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
      bump = escrow.bump,
      has_one = maker @ EscrowError::InvalidMaker,
      has_one = mint_a @ EscrowError::InvalidMintA,
      has_one = mint_b @ EscrowError::InvalidMintB,
    )]
    pub escrow: Account<'info, Escrow>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
      mut,
      associated_token::mint = mint_a,
      associated_token::authority = escrow,
      associated_token::token_program = token_program,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>, // Box用于应对栈空间不足问题，把对象分配到堆
    #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = mint_a,  // 初始化ata需要指定的参数
      associated_token::authority = taker,
      associated_token::token_program = token_program,
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      mut,
      associated_token::mint = mint_b,  // 非初始化情况是约束条件？
      associated_token::authority = taker,
      associated_token::token_program = token_program,
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
      init_if_needed,
      payer = taker,
      associated_token::mint = mint_b,
      associated_token::authority = maker,
      associated_token::token_program = token_program,
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    fn transfer_to_maker(&mut self) -> Result<()> {
        token_interface::transfer_checked(
            CpiContext::new(
                self.token_program.key(),
                TransferChecked {
                    from: self.taker_ata_b.to_account_info(),
                    to: self.maker_ata_b.to_account_info(),
                    mint: self.mint_b.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            self.escrow.receive,
            self.mint_b.decimals,
        )?;
        Ok(())
    }
    fn withdraw_and_close_vault(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];
        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.taker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;
        token_interface::close_account(CpiContext::new_with_signer(
            self.token_program.key(),
            CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.maker.to_account_info(),
                authority: self.escrow.to_account_info(),
            },
            &signer_seeds,
        ))?;
        Ok(())
    }
}

pub fn take_handler(ctx: Context<Take>) -> Result<()> {
    ctx.accounts.transfer_to_maker()?;
    ctx.accounts.withdraw_and_close_vault()?;
    Ok(())
}
