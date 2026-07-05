use anchor_lang::prelude::*;

#[error_code]
pub enum PrivateTransfersError {
    #[msg("Merkle tree is full")]
    TreeFull,
    #[msg("Invalid Merkle root")]
    InvalidRoot,
    #[msg("Nullifier already used")]
    NullifierUsed,
    #[msg("Deposit amount too small (minimum 0.001 SOL)")]
    DepositTooSmall,
    #[msg("Nullifier set is full")]
    NullifierSetFull,
    #[msg("Recipient account does not match recipient parameter")]
    RecipientMismatch,
    #[msg("Invalid verifier program")]
    InvalidVerifier,
    #[msg("Insufficient vault balance for withdrawal")]
    InsufficientVaultBalance,
}
