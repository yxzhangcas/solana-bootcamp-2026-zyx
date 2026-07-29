mod utils;

use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::AssertionHelpers;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;

use crate::labubu_vault::accounts::LabubuCollection;
use crate::utils::initialize_collection;

declare_program!(labubu_vault);

#[test]
pub fn test_initialize_collection() {
    let program_bytes = include_bytes!("../../../target/deploy/labubu_vault.so");
    let mut ctx = AnchorLiteSVM::build_with_program(labubu_vault::ID, program_bytes);
    let authority = ctx.create_funded_account(10_000_000_000).unwrap();

    initialize_collection(&mut ctx, &authority);

    // 检查结果
    let collection_pda = ctx.svm.get_pda(&[b"collection"], &labubu_vault::ID);
    ctx.svm.assert_account_exists(&collection_pda);
    let collection = ctx
        .get_account::<LabubuCollection>(&collection_pda)
        .unwrap();
    assert!(collection.authority == authority.pubkey());
    assert!(collection.remaining_supply[0] == labubu_vault::constants::NORMAL_SUPPLY);
    assert!(collection.remaining_supply[10] == labubu_vault::constants::RARE_SUPPLY);
    assert!(collection.total_minted == 0);
}
