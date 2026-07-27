use std::error::Error;

use anchor_litesvm::{AssertionHelpers, Signer};
use stable_swap::{
    math::{calculate_lp_mint_amount, calculate_withdraw_amounts},
    MINIMUM_LIQUIDITY,
};

use crate::utils::{TestFixture, AMPLIFICATION, INITIAL_DEPOSIT, INITIAL_MINT};

mod utils;

#[test]
fn add_and_remove_liquidity_use_proportional_lp_accounting() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();
    let user_lp_token = fixture.create_user_lp_token();

    let expected_lp = calculate_lp_mint_amount(
        0,
        0,
        INITIAL_DEPOSIT as u128,
        INITIAL_DEPOSIT as u128,
        0,
        AMPLIFICATION as u128,
        MINIMUM_LIQUIDITY,
    )?;

    fixture.add_liquidity(INITIAL_DEPOSIT, INITIAL_DEPOSIT, expected_lp);

    fixture
        .ctx
        .svm
        .assert_token_balance(&fixture.vault_a, INITIAL_DEPOSIT);
    fixture
        .ctx
        .svm
        .assert_token_balance(&fixture.vault_b, INITIAL_DEPOSIT);
    fixture
        .ctx
        .svm
        .assert_token_balance(&user_lp_token, expected_lp);
    fixture
        .ctx
        .svm
        .assert_mint_supply(&fixture.lp_mint.pubkey(), expected_lp);

    let burn_amount = expected_lp / 2;
    let withdraw_amounts = calculate_withdraw_amounts(
        &[INITIAL_DEPOSIT as u128, INITIAL_DEPOSIT as u128],
        burn_amount as u128,
        expected_lp as u128 + MINIMUM_LIQUIDITY as u128,
    )?;
    fixture.remove_liquidity(burn_amount, withdraw_amounts[0], withdraw_amounts[1]);

    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_a,
        INITIAL_MINT - INITIAL_DEPOSIT + withdraw_amounts[0],
    );
    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_b,
        INITIAL_MINT - INITIAL_DEPOSIT + withdraw_amounts[1],
    );
    fixture
        .ctx
        .svm
        .assert_token_balance(&user_lp_token, expected_lp - burn_amount);

    Ok(())
}
