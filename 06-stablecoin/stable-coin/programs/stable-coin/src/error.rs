use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinError {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Minting is currently paused")]
    Paused,
    #[msg("Mint amount exceeds minter's remaining allowance")]
    ExceedsAllowance,
    #[msg("Account is not an authorized minter")]
    NotMinter,
    #[msg("Arithmetic overflow")]
    Overflow,
}
