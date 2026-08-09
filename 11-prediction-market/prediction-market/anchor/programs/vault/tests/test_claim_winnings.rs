mod utils;

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::vault::accounts::UserPosition;
use crate::utils::create_market;
use crate::utils::place_bet;
use crate::utils::resolve_market;

declare_program!(vault);

#[test]
pub fn test_claim_winnings() {
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
    let outcome: bool = true;

    create_market(&mut ctx, &creator, market_id, question, alive_time);
    place_bet(&mut ctx, &creator, &user, market_id, amount, bet_yes);

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
    let user_pda = ctx.svm.get_pda(
        &[b"position", market_pda.as_ref(), user.pubkey().as_ref()],
        &vault::ID,
    );

    let ix = ctx
        .program()
        .accounts(vault::client::accounts::ClaimWinnings {
            user: user.pubkey(),
            market: market_pda,
            user_position: user_pda,
            system_program: system_program::ID,
        })
        .args(vault::client::args::ClaimWinnings {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&user])
        .unwrap()
        .assert_success();

    ctx.svm.assert_account_exists(&market_pda);
    let market_balance = ctx.svm.get_balance(&market_pda).unwrap();
    assert!(market_balance < 1_000_000_000); // 还有分配空间需要的lamports需要考虑

    ctx.svm.assert_account_exists(&user_pda);
    let user_position = ctx.get_account::<UserPosition>(&user_pda).unwrap();
    assert!(user_position.claimed == true);

    let user_balance = ctx.svm.get_balance(&user.pubkey()).unwrap();
    assert!(user_balance > 99 * 1_000_000_000);
}
