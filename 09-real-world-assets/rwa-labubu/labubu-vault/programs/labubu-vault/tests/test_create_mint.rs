mod utils;

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::TestHelpers;
use anchor_spl::token_interface::Mint;

use crate::utils::create_mint;
use crate::utils::initialize_collection;

declare_program!(labubu_vault);

#[test]
pub fn test_create_mint() {
    let program_bytes = include_bytes!("../../../target/deploy/labubu_vault.so");
    let mut ctx = AnchorLiteSVM::build_with_program(labubu_vault::ID, program_bytes);
    let authority = ctx.create_funded_account(10_000_000_000).unwrap();

    initialize_collection(&mut ctx, &authority);

    for labubu_id in 1..11 as u8 {
        create_mint(&mut ctx, &authority, labubu_id);
    }

    // 检查结果
    let collection_pda = ctx.svm.get_pda(&[b"collection"], &labubu_vault::ID);
    for labubu_id in 1..11 as u8 {
        let mint_pda = ctx
            .svm
            .get_pda(&[b"labubu_mint".as_ref(), &[labubu_id]], &labubu_vault::ID);
        ctx.svm.assert_account_exists(&mint_pda);
        let mint_info = ctx.get_account::<Mint>(&mint_pda).unwrap();

        assert!(mint_info.decimals == 0);
        assert!(mint_info.freeze_authority.is_none());
        assert!(mint_info.mint_authority.unwrap() == collection_pda);
        assert!(mint_info.supply == 0);
    }
}
