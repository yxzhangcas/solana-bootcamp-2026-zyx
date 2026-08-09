use anchor_lang::prelude::*;
use anchor_litesvm::{AnchorContext, Keypair, Signer, TestHelpers};

declare_program!(vault);

pub fn create_market(
    ctx: &mut AnchorContext,
    creator: &Keypair,
    market_id: u64,
    question: &str,
    alive_time: i64,
) {
    let resolution_time = ctx.svm.get_sysvar::<Clock>().unix_timestamp + alive_time;
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
        .accounts(vault::client::accounts::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        })
        .args(vault::client::args::CreateMarket {
            market_id,
            question: question.to_string(),
            resolution_time,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();
}

pub fn place_bet(
    ctx: &mut AnchorContext,
    creator: &Keypair,
    user: &Keypair,
    market_id: u64,
    amount: u64,
    bet_yes: bool,
) {
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
    ctx.execute_instruction(ix, &[&user])
        .unwrap()
        .assert_success();
}

pub fn resolve_market(ctx: &mut AnchorContext, creator: &Keypair, market_id: u64, outcome: bool) {
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
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();
}
