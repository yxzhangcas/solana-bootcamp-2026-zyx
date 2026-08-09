use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[cfg(test)]
mod tests;

mod constants;
mod error;
mod instructions;
mod state;
use instructions::*;

declare_id!("4W88mTBdwLUv3RsheJGJhpTp1Vm6CEGvWyYsA2c8KWvt");

#[program]
pub mod vault {
    use crate::error::MarketError;

    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        question: String,
        resolution_time: i64,
    ) -> Result<()> {
        create_market::handle_create_market(ctx, market_id, question, resolution_time)
    }
    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, bet_yes: bool) -> Result<()> {
        place_bet::handle_place_bet(ctx, amount, bet_yes)
    }
    pub fn resolve_market(ctx: Context<ResolveMarket>, outcome: bool) -> Result<()> {
        resolve_market::handle_resolve_market(ctx, outcome)
    }
    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        claim_winnings::handle_claim_winnings(ctx)
    }

    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        require!(
            ctx.accounts.vault.lamports() == 0,
            MarketError::VaultAlreadyExists
        );

        let rent = Rent::get()?.minimum_balance(0);
        require!(amount > rent, MarketError::InvalidAmount);

        transfer(
            CpiContext::new(
                System::id(),
                Transfer {
                    from: ctx.accounts.signer.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        require!(
            ctx.accounts.vault.lamports() > 0,
            MarketError::InvalidAmount
        );

        let bump = ctx.bumps.vault;
        let signer_key = ctx.accounts.signer.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", signer_key.as_ref(), &[bump]]];

        transfer(
            CpiContext::new_with_signer(
                System::id(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.signer.to_account_info(),
                },
                signer_seeds,
            ),
            ctx.accounts.vault.lamports(),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct VaultAction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
