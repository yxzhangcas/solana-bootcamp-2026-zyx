mod utils; // 加载工具模块

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;
use anchor_spl::token_2022;

use crate::utils::create_ctx;
use crate::utils::get_config_pda;
use crate::utils::get_mint_pda;
use crate::utils::initialize;

declare_program!(stable_coin);

#[test]
fn test_initialize() {
    let mut ctx = create_ctx();
    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::Initialize {
            admin: admin.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_program: token_2022::ID,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::Initialize {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

    // Verify accounts were created
    assert!(
        ctx.account_exists(&config_pda),
        "Config account should exist"
    );
    assert!(ctx.account_exists(&mint_pda), "Mint account should exist");
}

#[test]
fn test_initialize_twice_fails() {
    let mut ctx = create_ctx();
    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize(&mut ctx, &admin);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();

    let ix2 = ctx
        .program()
        .accounts(stable_coin::client::accounts::Initialize {
            admin: admin.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_program: token_2022::ID,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::Initialize {})
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix2, &[&admin]);

    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Second initialize should fail"
    );
}
