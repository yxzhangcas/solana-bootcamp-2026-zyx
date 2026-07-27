use std::error::Error;

use crate::utils::{TestFixture, INITIAL_DEPOSIT, ONE_TOKEN};

mod utils;

const DEPEGGED_PRICE: i64 = 90_000_000;
const SWAP_AMOUNT: u64 = 10_000 * ONE_TOKEN;

#[test]
fn check_depeg_pauses_pool_and_blocks_swaps() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();
    fixture.create_user_lp_token();
    fixture.add_liquidity(INITIAL_DEPOSIT, INITIAL_DEPOSIT, 0);

    fixture.overwrite_oracle(fixture.oracle_b, DEPEGGED_PRICE);
    fixture.check_depeg().assert_success();

    let pool = fixture.pool_state();
    assert!(pool.is_paused);

    fixture
        .swap(SWAP_AMOUNT, 0, 0, 1)
        .assert_anchor_error("PoolPaused");

    Ok(())
}
