mod utils;

use std::error::Error;

use anchor_litesvm::{AssertionHelpers, Signer};
use stable_swap::Pool;

use crate::utils::*;

#[test]
fn initialize_pool_creates_pool_state_and_vaults() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();

    let pool = fixture.ctx.get_account::<Pool>(&fixture.pool).unwrap();
    assert_eq!(pool.admin, fixture.user.pubkey());
    assert_eq!(pool.lp_mint, fixture.lp_mint.pubkey());
    assert_eq!(pool.amplification, AMPLIFICATION);
    assert_eq!(pool.fee_bps, BASE_FEE_BPS);
    assert_eq!(pool.token_mints[0], fixture.token_mint_a.pubkey());
    assert_eq!(pool.token_mints[1], fixture.token_mint_b.pubkey());
    assert_eq!(pool.oracle_config.oracle_a, fixture.oracle_a);
    assert_eq!(pool.oracle_config.oracle_b, fixture.oracle_b);
    assert!(!pool.is_paused);

    fixture.ctx.svm.assert_account_exists(&fixture.pool);
    fixture.ctx.svm.assert_account_exists(&fixture.vault_a);
    fixture.ctx.svm.assert_account_exists(&fixture.vault_b);
    fixture
        .ctx
        .svm
        .assert_mint_supply(&fixture.lp_mint.pubkey(), 0);

    Ok(())
}
