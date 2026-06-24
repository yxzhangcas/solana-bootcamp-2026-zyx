use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::TransferChecked,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{error::EscrowError, state::Escrow};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
      init,
      payer = maker,
      space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(), // 分配的空间可以更大但不能更小
      seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
      bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
      mint::token_program = token_program,  // 可选约束条件，对传入的mint账户类型进行校验
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account(
      mint::token_program = token_program,
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
      mut,
      associated_token::mint = mint_a,  // 可选约束条件，入参校验
      associated_token::authority = maker,
      associated_token::token_program = token_program,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    // 对于token_account的约束有两类：token::*, associated_token::*
    // 后者只针对ata进行约束，前者更加通用，可以对pda和带私钥的地址进行约束
    // 通常只使用ATA，针对某些特殊场景需要使用PDA（比如同一个Mint创建多个不同的TokenAccount）
    #[account(
      init,
      payer = maker,
      associated_token::mint = mint_a,  // 必选初始化参数
      associated_token::authority = escrow, // 必选初始化参数
      associated_token::token_program = token_program,   // 可选初始化参数
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>, // TokenProgram or TokenExtensionProgram
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    // 可以看做是setter方法(这里的self对应的是Make结构，对应到Context中是ctx.accounts)
    fn populate_escrow(&mut self, seed: u64, amount: u64, bump: u8) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive: amount,
            bump,
        });
        Ok(())
    }
    fn deposit_tokens(&mut self, amount: u64) -> Result<()> {
        // 此处不需要额外签名，maker的签名就足够了
        token_interface::transfer_checked(
            CpiContext::new(
                self.token_program.key(),
                TransferChecked {
                    from: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;
        Ok(())
    }
}

pub fn make_handler(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
    require_gt!(receive, 0, EscrowError::InvalidAmount);
    require_gt!(amount, 0, EscrowError::InvalidAmount);
    ctx.accounts
        .populate_escrow(seed, receive, ctx.bumps.escrow)?;
    ctx.accounts.deposit_tokens(amount)?;
    Ok(())
}
