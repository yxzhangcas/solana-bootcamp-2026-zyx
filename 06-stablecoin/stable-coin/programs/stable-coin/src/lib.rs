pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("6ddbQXeryFeCtwuNi6H2dK1gEvDWZYiNYe13xd5gZo1s");

#[program]
pub mod stable_coin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx)
    }
    pub fn configure_minter(ctx: Context<ConfigureMinter>, allowance: u64) -> Result<()> {
        crate::instructions::configure::handle_configure_minter(ctx, allowance)
    }
    pub fn remove_minter(ctx: Context<RemoveMinter>) -> Result<()> {
        crate::instructions::remove::handle_remove_minter(ctx)
    }
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        crate::instructions::mint::handle_mint_tokens(ctx, amount)
    }
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        crate::instructions::burn::handle_burn_tokens(ctx, amount)
    }
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        crate::instructions::pause::handle_pause(ctx)
    }
    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        crate::instructions::pause::handle_unpause(ctx)
    }
}
