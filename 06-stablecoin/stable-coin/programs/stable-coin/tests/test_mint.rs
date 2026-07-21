mod utils; // 加载工具模块

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::Keypair;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;
use anchor_spl::associated_token;
use anchor_spl::token_2022;

use crate::utils::configure;
use crate::utils::create_ctx;
use crate::utils::get_ata;
use crate::utils::get_config_pda;
use crate::utils::get_mint_pda;
use crate::utils::get_minter_config_pda;
use crate::utils::get_token_balance;
use crate::utils::initialize;
use crate::utils::mint;

declare_program!(stable_coin);

#[test]
fn test_mint_tokens() {
    let mut ctx = create_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();
    let allowance: u64 = 1_000_000_000;
    let mint_amount: u64 = 100_000_000;

    initialize(&mut ctx, &admin);

    configure(&mut ctx, &admin, &minter, allowance);

    mint(&mut ctx, &minter, &recipient.pubkey(), mint_amount);

    let mint_pda = get_mint_pda();
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    // Verify destination token account was created and has tokens
    assert!(
        ctx.account_exists(&destination_ata),
        "Destination token account should exist"
    );
    assert_eq!(
        get_token_balance(&mut ctx, &destination_ata),
        mint_amount,
        "Destination token balance mismatch"
    );
}

#[test]
fn test_mint_exceed_allowance() {
    let mut ctx = create_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();
    let allowance: u64 = 100_000_000;
    let mint_amount: u64 = 200_000_000;

    initialize(&mut ctx, &admin);

    configure(&mut ctx, &admin, &minter, allowance);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: token_2022::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&minter]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Mint exceeding allowance should fail"
    );
}

#[test]
fn test_mint_unauthorized() {
    let mut ctx = create_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let unauthorized = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();
    let mint_amount: u64 = 100_000_000;

    initialize(&mut ctx, &admin);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&unauthorized.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::MintTokens {
            minter: unauthorized.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: token_2022::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[&unauthorized]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Unauthorized mint should fail"
    );
}
