mod utils; // 加载工具模块

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;
use anchor_spl::token_2022;

use crate::utils::burn;
use crate::utils::configure;
use crate::utils::create_ctx;
use crate::utils::get_ata;
use crate::utils::get_config_pda;
use crate::utils::get_mint_pda;
use crate::utils::get_token_balance;
use crate::utils::initialize;
use crate::utils::mint;

declare_program!(stable_coin);

#[test]
fn test_burn_tokens() {
    let mut ctx = create_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let allowance: u64 = 1_000_000_000;
    let mint_amount: u64 = 200_000_000;
    let burn_amount: u64 = 50_000_000;

    initialize(&mut ctx, &admin);
    configure(&mut ctx, &admin, &minter, allowance);
    mint(&mut ctx, &minter, &recipient.pubkey(), mint_amount);
    burn(&mut ctx, &recipient, burn_amount);

    let mint_pda = get_mint_pda();
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);
    // Verify remaining balance
    assert_eq!(
        get_token_balance(&mut ctx, &destination_ata),
        mint_amount - burn_amount,
        "User token balance after burn mismatch"
    );
}

#[test]
fn test_more_than_balance() {
    let mut ctx = create_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let allowance: u64 = 1_000_000_000;
    let mint_amount: u64 = 100_000_000;
    let burn_amount: u64 = 500_000_000;

    initialize(&mut ctx, &admin);
    configure(&mut ctx, &admin, &minter, allowance);
    mint(&mut ctx, &minter, &recipient.pubkey(), mint_amount);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::BurnTokens {
            owner: recipient.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_account: destination_ata,
            token_program: token_2022::ID,
        })
        .args(stable_coin::client::args::BurnTokens {
            amount: burn_amount,
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&recipient]);

    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Burn more than balance should fail"
    );
}
