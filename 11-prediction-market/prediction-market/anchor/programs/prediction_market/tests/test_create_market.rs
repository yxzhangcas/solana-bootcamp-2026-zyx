mod utils;

use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::prediction_market::accounts::Market;
use crate::utils::create_market;

declare_program!(prediction_market);

#[test]
pub fn test_create_market() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        prediction_market::ID,
        include_bytes!("../../../target/deploy/prediction_market.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = 1_000;
    create_market(&mut ctx, &creator, market_id, question, alive_time);

    let market_pda = ctx.svm.get_pda(
        &[
            b"market",
            creator.pubkey().as_ref(),
            &market_id.to_le_bytes(),
        ],
        &prediction_market::ID,
    );
    ctx.svm.assert_account_exists(&market_pda);
    let market = ctx.get_account::<Market>(&market_pda).unwrap();
    assert!(market.creator == creator.pubkey());
    assert!(market.market_id == market_id);
    assert!(market.question == question.to_string());
}

#[test]
pub fn test_create_market_outoftime() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        prediction_market::ID,
        include_bytes!("../../../target/deploy/prediction_market.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = -1_000;
    let resolution_time = ctx.svm.get_sysvar::<Clock>().unix_timestamp + alive_time;
    let market_pda = ctx.svm.get_pda(
        &[
            b"market",
            creator.pubkey().as_ref(),
            &market_id.to_le_bytes(),
        ],
        &prediction_market::ID,
    );
    let ix = ctx
        .program()
        .accounts(prediction_market::client::accounts::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        })
        .args(prediction_market::client::args::CreateMarket {
            market_id,
            question: question.to_string(),
            resolution_time,
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&creator]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Should not create out-of-time market"
    );
}
