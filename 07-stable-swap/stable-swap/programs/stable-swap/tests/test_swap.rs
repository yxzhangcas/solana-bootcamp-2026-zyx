use std::error::Error;

use anchor_litesvm::AssertionHelpers;
use stable_swap::math::calculate_swap_output;

use crate::utils::{
    TestFixture, AMPLIFICATION, BASE_FEE_BPS, DEPEG_THRESHOLD_BPS, INITIAL_DEPOSIT, INITIAL_MINT,
    MAX_DYNAMIC_FEE_BPS, NORMALIZED_ONE_DOLLAR, SWAP_AMOUNT,
};

mod utils;

#[test]
fn swap_uses_remaining_accounts_and_matches_quote() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();
    fixture.create_user_lp_token();
    fixture.add_liquidity(INITIAL_DEPOSIT, INITIAL_DEPOSIT, 0);

    let quote = calculate_swap_output(
        INITIAL_DEPOSIT as u128,
        INITIAL_DEPOSIT as u128,
        SWAP_AMOUNT as u128,
        AMPLIFICATION as u128,
        BASE_FEE_BPS,
        MAX_DYNAMIC_FEE_BPS,
        NORMALIZED_ONE_DOLLAR,
        NORMALIZED_ONE_DOLLAR,
        DEPEG_THRESHOLD_BPS,
    )?;

    fixture
        .swap(SWAP_AMOUNT, quote.amount_out as u64, 0, 1)
        .assert_success();

    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_a,
        INITIAL_MINT - INITIAL_DEPOSIT - SWAP_AMOUNT,
    );
    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_b,
        INITIAL_MINT - INITIAL_DEPOSIT + quote.amount_out as u64,
    );

    Ok(())
}
