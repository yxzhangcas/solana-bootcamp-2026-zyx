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
use crate::utils::pause;
use crate::utils::remove;
use crate::utils::unpause;

declare_program!(stable_coin);

#[test]
fn test_full_flow() {
    let mut ctx = create_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user1 = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user2 = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let allowance: u64 = 1_000_000_000;
    let user1_amount: u64 = 100_000_000;
    let user2_amount: u64 = 200_000_000;
    let burn_amount: u64 = 50_000_000;

    initialize(&mut ctx, &admin);
    configure(&mut ctx, &admin, &minter, allowance);
    mint(&mut ctx, &minter, &user1.pubkey(), user1_amount);
    mint(&mut ctx, &minter, &user2.pubkey(), user2_amount);
    burn(&mut ctx, &user1, burn_amount);

    let mint_pda = get_mint_pda();
    let user1_ata = get_ata(&user1.pubkey(), &mint_pda);
    // Verify user1 balance after burn
    assert_eq!(
        get_token_balance(&mut ctx, &user1_ata),
        user1_amount - burn_amount,
        "User1 token balance after burn mismatch"
    );

    pause(&mut ctx, &admin);
    unpause(&mut ctx, &admin);
    remove(&mut ctx, &admin, &minter);
}
