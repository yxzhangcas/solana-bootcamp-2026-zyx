mod utils;

use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::vault::accounts::Market;
use crate::vault::accounts::UserPosition;
use crate::utils::create_market;
use crate::utils::place_bet;

declare_program!(vault);

#[test]
pub fn test_place_bet() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        vault::ID,
        include_bytes!("../../../target/deploy/vault.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let user = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = 1_000;
    let amount: u64 = 10 * 1_000_000_000;
    let bet_yes: bool = true;
    create_market(&mut ctx, &creator, market_id, question, alive_time);
    place_bet(&mut ctx, &creator, &user, market_id, amount, bet_yes);

    let market_pda = ctx.svm.get_pda(
        &[
            b"market",
            creator.pubkey().as_ref(),
            &market_id.to_le_bytes(),
        ],
        &vault::ID,
    );
    let user_pda = ctx.svm.get_pda(
        &[b"position", market_pda.as_ref(), user.pubkey().as_ref()],
        &vault::ID,
    );

    ctx.svm.assert_account_exists(&market_pda);
    let market = ctx.get_account::<Market>(&market_pda).unwrap();
    assert!(market.creator == creator.pubkey());
    assert!(market.market_id == market_id);
    assert!(market.question == question.to_string());
    assert!(market.yes_pool_lamports == amount);
    assert!(market.no_pool_lamports == 0);
    let market_balance = ctx.svm.get_balance(&market_pda).unwrap();
    assert!(market_balance >= amount); // 还有分配空间需要的lamports需要考虑

    ctx.svm.assert_account_exists(&user_pda);
    let user_position = ctx.get_account::<UserPosition>(&user_pda).unwrap();
    assert!(user_position.user == user.pubkey());
    assert!(user_position.market == market_pda);
    assert!(user_position.yes_amount == amount);
    assert!(user_position.no_amount == 0);
}

#[test]
pub fn test_place_bet_after_deadline() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        vault::ID,
        include_bytes!("../../../target/deploy/vault.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let user = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = 1;
    let amount: u64 = 10 * 1_000_000_000;
    let bet_yes: bool = true;
    create_market(&mut ctx, &creator, market_id, question, alive_time);

    let mut clock = ctx.svm.get_sysvar::<Clock>();
    clock.unix_timestamp += alive_time + 1;
    ctx.svm.set_sysvar(&clock);

    let market_pda = ctx.svm.get_pda(
        &[
            b"market",
            creator.pubkey().as_ref(),
            &market_id.to_le_bytes(),
        ],
        &vault::ID,
    );
    let user_pda = ctx.svm.get_pda(
        &[b"position", market_pda.as_ref(), user.pubkey().as_ref()],
        &vault::ID,
    );
    let ix = ctx
        .program()
        .accounts(vault::client::accounts::PlaceBet {
            user: user.pubkey(),
            market: market_pda,
            user_position: user_pda,
            system_program: system_program::ID,
        })
        .args(vault::client::args::PlaceBet {
            amount: amount,
            bet_yes: bet_yes,
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&user]);

    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Should not place bet after deadline"
    );
}
