use std::{error::Error, fs, path::PathBuf};

use anchor_lang::{solana_program::rent, system_program};
// use anchor_lang::prelude::*; // 不要全都引入，否则会由于别名Result<T>覆盖而无法使用Result<T,E>结构
use anchor_litesvm::{
    AccountMeta, AnchorContext, AnchorLiteSVM, Keypair, Pubkey, Signer, TestHelpers,
    TransactionResult,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token,
};
use bytemuck::bytes_of;
use solana_sdk::{account::Account, clock::Clock, native_loader, native_token::LAMPORTS_PER_SOL};
use stable_swap::{
    oracle::{PythPriceAccount, PythPriceComp, PythPriceInfo, PythRational},
    Pool,
};

const PROGRAM_SO_PATH: &str = "../../target/deploy/stable_swap.so";

pub const DECIMALS: u8 = 6;
pub const ONE_TOKEN: u64 = 1_000_000;
pub const INITIAL_MINT: u64 = 2_000_000 * ONE_TOKEN;

pub const ONE_DOLLAR_PRICE: i64 = 100_000_000;

pub const PYTH_MAGIC: u32 = 0xa1b2c3d4;
pub const PYTH_VERSION_2: u32 = 2;
pub const PYTH_ACCOUNT_TYPE_PRICE: u32 = 3;
pub const PYTH_STATUS_TRADING: u8 = 1;
pub const PYTH_NUM_COMPONENTS: usize = 32;
pub const PYTH_EXPONENT: i32 = -8;

pub const AMPLIFICATION: u64 = 100;
pub const BASE_FEE_BPS: u16 = 4;
pub const MAX_DYNAMIC_FEE_BPS: u16 = 100;
pub const DEPEG_THRESHOLD_BPS: u16 = 500;
pub const MAX_PRICE_AGE_SEC: u64 = 60;

pub const INITIAL_DEPOSIT: u64 = 1_000_000 * ONE_TOKEN;
pub const SWAP_AMOUNT: u64 = 10_000 * ONE_TOKEN;
pub const NORMALIZED_ONE_DOLLAR: u128 = 1_000_000_000;

pub struct TestFixture {
    pub ctx: AnchorContext,
    pub user: Keypair,
    pub token_mint_a: Keypair,
    pub token_mint_b: Keypair,
    pub lp_mint: Keypair,
    pub pool: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub user_token_a: Pubkey,
    pub user_token_b: Pubkey,
    pub user_lp_token: Option<Pubkey>,
    pub oracle_a: Pubkey,
    pub oracle_b: Pubkey,
}
impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut ctx = AnchorLiteSVM::build_with_program(stable_swap::ID, &read_program_bytes()?);
        // 对system_program的data进行填写有意义吗？回头注释掉看看有没有问题
        ctx.svm.set_account(
            system_program::ID,
            Account {
                lamports: 1,
                data: b"system_program".to_vec(),
                owner: native_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        )?;
        // 创建用户相关的token和account
        let user = ctx.create_funded_account(20 * LAMPORTS_PER_SOL)?;
        let token_mint_a = ctx.svm.create_token_mint(&user, DECIMALS)?;
        let token_mint_b = ctx.svm.create_token_mint(&user, DECIMALS)?;
        let user_token_a = ctx
            .svm
            .create_associated_token_account(&token_mint_a.pubkey(), &user)?;
        let user_token_b = ctx
            .svm
            .create_associated_token_account(&token_mint_b.pubkey(), &user)?;
        ctx.svm
            .mint_to(&token_mint_a.pubkey(), &user_token_a, &user, INITIAL_MINT)?;
        ctx.svm
            .mint_to(&token_mint_b.pubkey(), &user_token_b, &user, INITIAL_MINT)?;
        // 创建oracle account
        let oracle_a = Pubkey::new_unique();
        let oracle_b = Pubkey::new_unique();
        write_pyth_price_account(&mut ctx, oracle_a, ONE_DOLLAR_PRICE)?;
        write_pyth_price_account(&mut ctx, oracle_b, ONE_DOLLAR_PRICE)?;
        // 计算pool相关的地址（并未创建）
        let lp_mint = Keypair::new();
        let pool =
            Pubkey::find_program_address(&[b"pool", lp_mint.pubkey().as_ref()], &stable_swap::ID).0;
        let vault_a = get_associated_token_address(&pool, &token_mint_a.pubkey());
        let vault_b = get_associated_token_address(&pool, &token_mint_b.pubkey());
        Ok(Self {
            ctx,
            user,
            token_mint_a,
            token_mint_b,
            lp_mint,
            pool,
            vault_a,
            vault_b,
            user_token_a,
            user_token_b,
            user_lp_token: None,
            oracle_a,
            oracle_b,
        })
    }

    pub fn initialize_pool(&mut self) {
        let _system_program = self
            .ctx
            .svm
            .get_account(&system_program::ID)
            .expect("system program account must exist in LiteSVM");
        assert!(
            _system_program.executable,
            "system program account must be executable"
        );
        let ix = self
            .ctx
            .program()
            .accounts(stable_swap::accounts::InitializePool {
                admin: self.user.pubkey(),
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                pool: self.pool,
                lp_mint: self.lp_mint.pubkey(),
                vault_a: self.vault_a,
                vault_b: self.vault_b,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
                system_program: anchor_lang::system_program::ID,
                token_program: token::ID,
                associated_token_program: associated_token::ID,
                rent: rent::ID,
            })
            .args(stable_swap::instruction::InitializePool {
                amplification: AMPLIFICATION,
                base_fee_bps: BASE_FEE_BPS,
                max_dynamic_fee_bps: MAX_DYNAMIC_FEE_BPS,
                depeg_threshold_bps: DEPEG_THRESHOLD_BPS,
                max_price_age_sec: MAX_PRICE_AGE_SEC,
            })
            .instruction()
            .unwrap();

        assert_eq!(ix.accounts[9].pubkey, system_program::ID);
        assert!(!ix.accounts[9].is_signer);
        assert!(!ix.accounts[9].is_writable);
        assert_eq!(ix.accounts[10].pubkey, token::ID);
        assert!(!ix.accounts[10].is_signer);
        assert!(!ix.accounts[10].is_writable);
        assert_eq!(ix.accounts[11].pubkey, associated_token::ID);
        assert!(!ix.accounts[11].is_signer);
        assert!(!ix.accounts[11].is_writable);
        assert_eq!(ix.accounts[12].pubkey, rent::ID);
        assert!(!ix.accounts[12].is_signer);
        assert!(!ix.accounts[12].is_writable);

        self.ctx
            .execute_instruction(ix, &[&self.user, &self.lp_mint])
            .unwrap()
            .assert_success();
    }
    pub fn add_liquidity(&mut self, amount_a: u64, amount_b: u64, min_lp_out: u64) {
        let user_lp_token = self.user_lp_token.expect("LP ATA must exist");
        let ix = self
            .ctx
            .program()
            .accounts(stable_swap::accounts::AddLiquidity {
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                pool: self.pool,
                vault_a: self.vault_a,
                vault_b: self.vault_b,
                lp_mint: self.lp_mint.pubkey(),
                user_token_a: self.user_token_a,
                user_token_b: self.user_token_b,
                user_lp_token,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
                user: self.user.pubkey(),
                token_program: token::ID,
            })
            .args(stable_swap::instruction::AddLiquidity {
                amount_a,
                amount_b,
                min_lp_out,
            })
            .instruction()
            .unwrap();
        self.ctx
            .execute_instruction(ix, &[&self.user])
            .unwrap()
            .assert_success();
    }
    pub fn remove_liquidity(&mut self, lp_amount: u64, min_a_out: u64, min_b_out: u64) {
        let user_lp_token = self.user_lp_token.expect("LP ATA must exist");
        let ix = self
            .ctx
            .program()
            .accounts(stable_swap::accounts::RemoveLiquidity {
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                pool: self.pool,
                vault_a: self.vault_a,
                vault_b: self.vault_b,
                lp_mint: self.lp_mint.pubkey(),
                user_token_a: self.user_token_a,
                user_token_b: self.user_token_b,
                user_lp_token,
                user: self.user.pubkey(),
                token_program: token::ID,
            })
            .args(stable_swap::instruction::RemoveLiquidity {
                lp_amount,
                min_a_out,
                min_b_out,
            })
            .instruction()
            .unwrap();
        self.ctx
            .execute_instruction(ix, &[&self.user])
            .unwrap()
            .assert_success();
    }
    pub fn check_depeg(&mut self) -> TransactionResult {
        let ix = self
            .ctx
            .program()
            .accounts(stable_swap::accounts::CheckDepeg {
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                lp_mint: self.lp_mint.pubkey(),
                pool: self.pool,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
            })
            .args(stable_swap::instruction::CheckDepeg {})
            .instruction()
            .unwrap();
        self.ctx.execute_instruction(ix, &[&self.user]).unwrap()
    }
    pub fn swap(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
        input_index: u8,
        output_index: u8,
    ) -> TransactionResult {
        let mut ix = self
            .ctx
            .program()
            .accounts(stable_swap::accounts::Swap {
                pool: self.pool,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
                user: self.user.pubkey(),
                token_program: token::ID,
            })
            .args(stable_swap::instruction::Swap {
                amount_in,
                min_amount_out,
                input_index,
                output_index,
            })
            .instruction()
            .unwrap();
        ix.accounts.extend([
            AccountMeta::new(self.vault_a, false),
            AccountMeta::new(self.vault_b, false),
            AccountMeta::new(self.user_token_a, false),
            AccountMeta::new(self.user_token_b, false),
        ]);
        self.ctx.execute_instruction(ix, &[&self.user]).unwrap()
    }

    pub fn create_user_lp_token(&mut self) -> Pubkey {
        let user_lp_token = self
            .ctx
            .svm
            .create_associated_token_account(&self.lp_mint.pubkey(), &self.user)
            .unwrap();
        self.user_lp_token = Some(user_lp_token);
        user_lp_token
    }
    pub fn pool_state(&self) -> Pool {
        self.ctx.get_account::<Pool>(&self.pool).unwrap()
    }
    pub fn overwrite_oracle(&mut self, oracle: Pubkey, price: i64) {
        write_pyth_price_account(&mut self.ctx, oracle, price).unwrap()
    }
}

fn read_program_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PROGRAM_SO_PATH);
    Ok(fs::read(&path).map_err(|err| {
        format!(
            "failed to read compiled program at {}: {}. Run `anchor build` first.",
            path.display(),
            err
        )
    })?)
}

fn write_pyth_price_account(
    ctx: &mut AnchorContext,
    oracle: Pubkey,
    price: i64,
) -> Result<(), Box<dyn Error>> {
    let clock = ctx.ctx_svm_clock();
    let price_account = PythPriceAccount {
        magic: PYTH_MAGIC,
        version: PYTH_VERSION_2,
        account_type: PYTH_ACCOUNT_TYPE_PRICE,
        size: size_of::<PythPriceAccount>() as u32,
        price_type: 0,
        exponent: PYTH_EXPONENT,
        num: 1,
        num_qt: 1,
        last_slot: clock.slot,
        valid_slot: clock.slot,
        ema_price: PythRational {
            value: price,
            numerator: price,
            denominator: 1,
        },
        ema_conf: PythRational {
            value: 0,
            numerator: 0,
            denominator: 1,
        },
        timestamp: clock.unix_timestamp,
        min_pub: 1,
        drv2: 0,
        drv3: 0,
        drv4: 0,
        product: Pubkey::new_from_array([0; 32]),
        next: Pubkey::new_from_array([0; 32]),
        prev_slot: clock.slot,
        prev_price: price,
        prev_conf: 0,
        prev_timestamp: clock.unix_timestamp,
        aggregate: PythPriceInfo {
            price,
            conf: 0,
            status: PYTH_STATUS_TRADING,
            corp_act: 0,
            padding: [0; 6],
            pub_slot: clock.slot,
        },
        comp: [PythPriceComp::default(); PYTH_NUM_COMPONENTS],
    };
    ctx.svm.set_account(
        oracle,
        Account {
            lamports: ctx
                .svm
                .minimum_balance_for_rent_exemption(size_of::<PythPriceAccount>()),
            data: bytes_of(&price_account).to_vec(),
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        },
    )?;
    Ok(())
}

trait ClockAccess {
    fn ctx_svm_clock(&self) -> Clock;
}
impl ClockAccess for AnchorContext {
    fn ctx_svm_clock(&self) -> Clock {
        self.svm.get_sysvar::<Clock>()
    }
}
