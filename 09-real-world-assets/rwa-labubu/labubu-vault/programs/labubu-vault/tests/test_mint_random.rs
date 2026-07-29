mod utils;

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022;
use anchor_spl::token_interface::Mint;

use crate::utils::create_mint;
use crate::utils::get_token_balance;
use crate::utils::initialize_collection;
use crate::utils::mint_random;

declare_program!(labubu_vault);

#[test]
pub fn test_mint_random() {
    let program_bytes = include_bytes!("../../../target/deploy/labubu_vault.so");
    let mut ctx = AnchorLiteSVM::build_with_program(labubu_vault::ID, program_bytes);
    let authority = ctx.create_funded_account(10_000_000_000).unwrap();
    let user = ctx.create_funded_account(10_000_000_000).unwrap();

    initialize_collection(&mut ctx, &authority);

    for labubu_id in 1..11 as u8 {
        create_mint(&mut ctx, &authority, labubu_id);
        mint_random(&mut ctx, &user, labubu_id);
    }

    // 检查结果
    for labubu_id in 1..11 as u8 {
        let mint_pda = ctx
            .svm
            .get_pda(&[b"labubu_mint".as_ref(), &[labubu_id]], &labubu_vault::ID);
        let user_token_account = get_associated_token_address_with_program_id(
            &user.pubkey(),
            &mint_pda,
            &token_2022::ID,
        );

        ctx.svm.assert_account_exists(&user_token_account);

        let mint_info = ctx.get_account::<Mint>(&mint_pda).unwrap();
        assert!(mint_info.supply == 1);

        assert!(get_token_balance(&mut ctx, &user_token_account) == 1);
    }
}
