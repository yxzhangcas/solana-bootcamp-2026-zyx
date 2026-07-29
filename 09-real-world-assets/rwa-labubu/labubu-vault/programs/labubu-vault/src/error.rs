use anchor_lang::prelude::*;

#[error_code]
pub enum LabubuError {
    #[msg("All Labubu sold out")]
    SoldOut,
    #[msg("Invalid Labubu ID (must be 1-11)")]
    InvalidLabubuId,
}
