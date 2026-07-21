use anchor_lang::prelude::*;

/// Config account that stores the stablecoin configuration
#[account]
#[derive(InitSpace)]
/// 每个Program唯一(init中初始化)
pub struct Config {
    /// The admin who can configure minters
    pub admin: Pubkey,  // 不可变
    /// The mint address of the stablecoin
    pub mint: Pubkey,   // 不可变
    /// Whether minting is paused
    pub paused: bool,   // 可变(pause.rs)
    /// Bump seed for the config PDA
    pub bump: u8,       // 不可变
    /// Bump seed for the mint PDA
    pub mint_bump: u8,  // 不可变
}

/// Minter configuration account
/// Each authorized minter has their own config with an allowance
#[account]
#[derive(InitSpace)]
pub struct MinterConfig {
    /// The minter's public key
    pub minter: Pubkey, // 被授权的矿工（不一定是wallet，也可能是program）
    /// Maximum amount the minter can mint (total)
    pub allowance: u64,
    /// Amount already minted by this minter
    pub amount_minted: u64,
    /// Whether this account has been initialized
    pub is_initialized: bool,   // 初始化标记（首次创建需要初始化，后续复用）
    /// Bump seed for this PDA
    pub bump: u8,
}