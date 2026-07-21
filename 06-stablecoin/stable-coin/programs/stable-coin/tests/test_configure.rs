mod utils;

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::Keypair;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::utils::configure;
use crate::utils::create_ctx;
use crate::utils::get_config_pda;
use crate::utils::get_minter_config_pda;
use crate::utils::initialize;

declare_program!(stable_coin);

#[test]
fn test_configure_minter() {
    let mut ctx = create_ctx();
    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize(&mut ctx, &admin);

    let minter = Keypair::new();
    let allowance: u64 = 1_000_000_000;

    configure(&mut ctx, &admin, &minter, allowance);

    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    // Verify accounts were created
    assert!(
        ctx.account_exists(&minter_config_pda),
        "Minter config account should exist"
    );
}

#[test]
fn test_configure_minter_unauthorized() {
    let mut ctx = create_ctx();
    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let unauthorized = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize(&mut ctx, &admin);

    let minter = Keypair::new();
    let config_pda = get_config_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let allowance: u64 = 1_000_000_000;

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::ConfigureMinter {
            admin: unauthorized.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::ConfigureMinter { allowance })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&unauthorized]);

    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Unauthorized configure_minter should fail"
    );
}

#[test]
fn test_update_minter_allowance() {
    let mut ctx = create_ctx();
    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize(&mut ctx, &admin);

    let minter = Keypair::new();
    let allowance1: u64 = 1_000_000_000;

    configure(&mut ctx, &admin, &minter, allowance1);

    let allowance2: u64 = 2_000_000_000;

    configure(&mut ctx, &admin, &minter, allowance2);
}
