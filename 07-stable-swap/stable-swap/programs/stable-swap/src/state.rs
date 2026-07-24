use anchor_lang::prelude::*;

use crate::NUM_TOKENS;

// 自定义结构体，作为PDA数据成员使用
#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct OracleConfig {
    pub oracle_a: Pubkey,
    pub oracle_b: Pubkey,
    pub max_depeg_bps: u16,
    pub emergency_fee_bps: u16,
    pub enabled: bool,
}
impl OracleConfig {
    pub const LEN: usize = 32 + 32 + 2 + 2 + 1;
}

#[account]
pub struct Pool {
    pub admin: Pubkey,
    pub lp_mint: Pubkey,
    pub amplification: u64,
    pub fee_bps: u16,
    pub token_mints: [Pubkey; NUM_TOKENS],
    pub bump: u8,
    pub oracle_config: OracleConfig,
    pub is_paused: bool,
}
impl Pool {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 2 + (32 * NUM_TOKENS) + 1 + OracleConfig::LEN + 1; 
    pub fn mints(&self) -> &[Pubkey; NUM_TOKENS] {
        &self.token_mints
    }
    pub fn find_mint_index(&self, mint: &Pubkey) -> Option<usize> {
        self.token_mints.iter().position(|stored_mint| stored_mint == mint)
    }
}