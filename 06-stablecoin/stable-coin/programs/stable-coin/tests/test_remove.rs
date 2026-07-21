mod utils;

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::Keypair;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::utils::configure;
use crate::utils::create_ctx;
use crate::utils::get_minter_config_pda;
use crate::utils::initialize;
use crate::utils::remove;

declare_program!(stable_coin);

#[test]
fn test_remove_minter() {
    let mut ctx = create_ctx();
    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize(&mut ctx, &admin);

    let minter = Keypair::new();
    let allowance1: u64 = 1_000_000_000;

    configure(&mut ctx, &admin, &minter, allowance1);

    let minter_config_pda = get_minter_config_pda(&minter.pubkey());

    // Verify accounts were created
    assert!(
        ctx.account_exists(&minter_config_pda),
        "Minter config account should exist"
    );

    remove(&mut ctx, &admin, &minter);

    // Verify minter config was closed
    ctx.svm.assert_account_closed(&minter_config_pda);
}
