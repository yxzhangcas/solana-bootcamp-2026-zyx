use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{self, Token, TokenAccount, Transfer},
};

use crate::{
    error::StableSwapError, math::calculate_swap_output, oracle::load_pair_status, Pool,
    DEFAULT_MAX_PRICE_AGE_SEC, NUM_TOKENS,
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    /// CHECK
    pub oracle_price_feed_a: UncheckedAccount<'info>,
    /// CHECK
    pub oracle_price_feed_b: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    // 实际传入的account不只这些，其它的通过remaining_account传入
}
impl<'info> Swap<'info> {
    // 用于解析处理remaining_account的解析
    fn read_token_account(&self, account: &AccountInfo<'info>) -> Result<TokenAccount> {
        let data = &mut &account.try_borrow_data()?[..];
        TokenAccount::try_deserialize(data)
            .map_err(|_| error!(StableSwapError::InvalidRemainingAccounts))
    }
    // vault也通过remaining_acccounts传入
    pub fn read_reserves(&self, vaults: &[&AccountInfo<'info>]) -> Result<Vec<u128>> {
        let mut reserves = Vec::with_capacity(vaults.len());
        for vault in vaults {
            let account = self.read_token_account(vault)?;
            reserves.push(account.amount as u128);
        }
        Ok(reserves)
    }
    // 仅用于进行账户参数的合法性校验，无业务逻辑
    pub fn validate_remaining_accounts(
        &self,
        remaining: &[AccountInfo<'info>],
        input_index: u8,
        output_index: u8,
    ) -> Result<()> {
        require!(
            input_index < NUM_TOKENS as u8 && output_index < NUM_TOKENS as u8,
            StableSwapError::InvalidTokenIndex
        );
        require!(input_index != output_index, StableSwapError::SameTokenSwap);
        // 每个token都对应一个pool_vault和user_token_account
        require!(
            remaining.len() == NUM_TOKENS * 2,
            StableSwapError::InvalidRemainingAccounts
        );

        // 有顺序要求：显式所有token对应的pool_vault，再是所有token对应的user_token_account
        let vault_a = self.read_token_account(&remaining[0])?;
        let vault_b = self.read_token_account(&remaining[1])?;
        let user_input = self.read_token_account(&remaining[2])?;
        let user_output = self.read_token_account(&remaining[3])?;

        let expected_vault_a =
            get_associated_token_address(&self.pool.key(), &self.pool.token_mints[0]);
        let expected_vault_b =
            get_associated_token_address(&self.pool.key(), &self.pool.token_mints[1]);

        require_keys_eq!(
            remaining[0].key(),
            expected_vault_a,
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            remaining[1].key(),
            expected_vault_b,
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_a.owner,
            self.pool.key(),
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_b.owner,
            self.pool.key(),
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_a.mint,
            self.pool.token_mints[0],
            StableSwapError::InvalidVault
        );
        require_keys_eq!(
            vault_b.mint,
            self.pool.token_mints[1],
            StableSwapError::InvalidVault
        );

        require_keys_eq!(
            user_input.owner,
            self.user.key(),
            StableSwapError::InvalidRemainingAccounts
        );
        require_keys_eq!(
            user_output.owner,
            self.user.key(),
            StableSwapError::InvalidRemainingAccounts
        );
        require_keys_eq!(
            user_input.mint,
            self.pool.token_mints[input_index as usize],
            StableSwapError::InvalidMint
        );
        require_keys_eq!(
            user_output.mint,
            self.pool.token_mints[output_index as usize],
            StableSwapError::InvalidMint
        );
        Ok(())
    }
    pub fn transfer_in(
        &self,
        from: &AccountInfo<'info>,
        vault: &AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                self.token_program.key(),
                Transfer {
                    from: from.clone(),
                    to: vault.clone(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )
    }
    pub fn transfer_out(
        &self,
        vault: &AccountInfo<'info>,
        to: &AccountInfo<'info>,
        amount: u64,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.key(),
                Transfer {
                    from: vault.clone(),
                    to: to.clone(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )
    }
}

pub fn swap_handler<'info>(
    ctx: Context<'info, Swap<'info>>,
    amount_in: u64,
    min_amount_out: u64,
    input_index: u8,
    output_index: u8,
) -> Result<()> {
    require!(amount_in > 0, StableSwapError::ZeroAmount);
    let pool = &ctx.accounts.pool;
    let remaining = ctx.remaining_accounts;
    ctx.accounts
        .validate_remaining_accounts(remaining, input_index, output_index)?;
    require!(!pool.is_paused, StableSwapError::PoolPaused);

    let oracle_status = load_pair_status(
        &pool.oracle_config.oracle_a,
        &pool.oracle_config.oracle_b,
        &ctx.accounts.oracle_price_feed_a,
        &ctx.accounts.oracle_price_feed_b,
        DEFAULT_MAX_PRICE_AGE_SEC,
        pool.oracle_config.max_depeg_bps,
    )?;

    let reserves = ctx
        .accounts
        .read_reserves(&[&remaining[0], &remaining[1]])?;
    let reserve_a = reserves[0];
    let reserve_b = reserves[1];
    require!(reserve_a > 0 && reserve_b > 0, StableSwapError::EmptyPool);

    let amp = pool.amplification as u128;
    let base_fee_bps = pool.fee_bps;
    let max_dynamic_fee_bps = if pool.oracle_config.enabled {
        pool.oracle_config.emergency_fee_bps
    } else {
        base_fee_bps
    };

    let reserve_by_index = [reserve_a, reserve_b];
    let price_by_index = [oracle_status.price_a, oracle_status.price_b];
    let reserve_in = reserve_by_index[input_index as usize];
    let reserve_out = reserve_by_index[output_index as usize];
    let oracle_price_in = price_by_index[input_index as usize];
    let oracle_price_out = price_by_index[output_index as usize];

    let quote = calculate_swap_output(
        reserve_in,
        reserve_out,
        amount_in as u128,
        amp,
        base_fee_bps,
        max_dynamic_fee_bps,
        oracle_price_in,
        oracle_price_out,
        pool.oracle_config.max_depeg_bps,
    )?;
    require!(
        quote.amount_out >= min_amount_out as u128,
        StableSwapError::SlippageExceeded
    );

    let seeds: &[&[u8]] = &[b"pool", pool.lp_mint.as_ref(), &[pool.bump]];
    ctx.accounts
        .transfer_in(&remaining[2], &remaining[input_index as usize], amount_in)?;
    ctx.accounts.transfer_out(
        &remaining[output_index as usize],
        &remaining[3],
        quote.amount_out as u64,
        &[seeds],
    )?;
    msg!(
        "Swap {}→{}: {} in → {} out (fee: {}, dynamic_fee={}bps, oracle_a={}bps, oracle_b={}bps)",
        input_index,
        output_index,
        amount_in,
        quote.amount_out,
        quote.fee_amount,
        quote.dynamic_fee_bps,
        oracle_status.peg_delta_a_bps,
        oracle_status.peg_delta_b_bps
    );
    Ok(())
}
