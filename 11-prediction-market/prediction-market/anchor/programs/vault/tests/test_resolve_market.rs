mod utils;

use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::vault::accounts::Market;
use crate::utils::create_market;
use crate::utils::resolve_market;

declare_program!(vault);

#[test]
pub fn test_resolve_market() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        vault::ID,
        include_bytes!("../../../target/deploy/vault.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = 1_000;
    let outcome: bool = true;

    create_market(&mut ctx, &creator, market_id, question, alive_time);

    let mut clock = ctx.svm.get_sysvar::<Clock>();
    clock.unix_timestamp += alive_time + 1;
    ctx.svm.set_sysvar(&clock);

    resolve_market(&mut ctx, &creator, market_id, outcome);

    let market_pda = ctx.svm.get_pda(
        &[
            b"market",
            creator.pubkey().as_ref(),
            &market_id.to_le_bytes(),
        ],
        &vault::ID,
    );

    let market = ctx.get_account::<Market>(&market_pda).unwrap();
    assert!(market.resolved == true);
    assert!(market.outcome.unwrap() == outcome);
}

#[test]
pub fn test_resolve_market_before_deadline() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        vault::ID,
        include_bytes!("../../../target/deploy/vault.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = 1_000;
    let outcome: bool = true;

    create_market(&mut ctx, &creator, market_id, question, alive_time);

    let market_pda = ctx.svm.get_pda(
        &[
            b"market",
            creator.pubkey().as_ref(),
            &market_id.to_le_bytes(),
        ],
        &vault::ID,
    );
    let ix = ctx
        .program()
        .accounts(vault::client::accounts::ResolveMarket {
            creator: creator.pubkey(),
            market: market_pda,
        })
        .args(vault::client::args::ResolveMarket { outcome })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&creator]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Should not resolve before market deadline"
    );
}

#[test]
pub fn test_resolve_market_without_creator() {
    let mut ctx = AnchorLiteSVM::build_with_program(
        vault::ID,
        include_bytes!("../../../target/deploy/vault.so"),
    );
    let creator = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let user = ctx.create_funded_account(100 * 1_000_000_000).unwrap();
    let market_id: u64 = 1;
    let question: &str = "Question";
    let alive_time: i64 = 1_000;
    let outcome: bool = true;

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
    let ix = ctx
        .program()
        .accounts(vault::client::accounts::ResolveMarket {
            creator: user.pubkey(),
            market: market_pda,
        })
        .args(vault::client::args::ResolveMarket { outcome })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&user]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Should not resolve without market creator"
    );
}
