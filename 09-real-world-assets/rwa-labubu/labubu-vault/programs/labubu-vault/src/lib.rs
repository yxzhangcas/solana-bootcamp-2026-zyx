pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8ZgzUhriQhkKLsp6kr4rrW68UYvjifHiUr4WrPCZJJ3Q");

#[program]
pub mod labubu_vault {
    use super::*;

    pub fn initialize_collection(ctx: Context<InitializeCollection>) -> Result<()> {
        crate::instructions::initialize_collection::handle_initialize_collection(ctx)
    }
    pub fn create_mint(ctx: Context<CreateLabubuMint>, labubu_id: u8) -> Result<()> {
        crate::instructions::create_mint::handle_create_mint(ctx, labubu_id)
    }
    pub fn mint_random(ctx: Context<MintRandom>, labubu_id: u8) -> Result<()> {
        crate::instructions::mint_random::handle_mint_random(ctx, labubu_id)
    }
}
