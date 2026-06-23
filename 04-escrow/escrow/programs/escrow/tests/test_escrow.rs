use anchor_lang::prelude::*;
use anchor_litesvm::{AnchorLiteSVM, AssertionHelpers, Signer, TestHelpers};
use anchor_spl::associated_token::{get_associated_token_address, spl_associated_token_account};
use anchor_spl::token::spl_token;

declare_program!(escrow);

use self::escrow::accounts::Escrow;
use self::escrow::client::{accounts, args};
use self::escrow::ID;

#[test]
fn test_escrow_make() {
    let program_id = ID;
    let mut ctx = AnchorLiteSVM::build_with_program(
        program_id,
        include_bytes!("../../../target/deploy/escrow.so"),
    );
    let maker = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let mint_a = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let mint_b = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let maker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &maker)
        .unwrap();
    ctx.svm
        .mint_to(&mint_a.pubkey(), &maker_ata_a, &maker, 1_000_000_000)
        .unwrap();
    let seed: u64 = 42;
    let escrow_pda = ctx.svm.get_pda(
        &[b"escrow", maker.pubkey().as_ref(), &seed.to_le_bytes()],
        &program_id,
    );
    let vault = get_associated_token_address(&escrow_pda, &mint_a.pubkey());

    let make_ix = ctx
        .program()
        .accounts({
            accounts::ExecMake {
                maker: maker.pubkey(),
                escrow: escrow_pda,
                mint_a: mint_a.pubkey(),
                mint_b: mint_b.pubkey(),
                maker_ata_a,
                vault,
                associated_token_program: spl_associated_token_account::program::ID,
                token_program: spl_token::id(),
                system_program: system_program::ID,
            }
        })
        .args(args::ExecMake {
            seed,
            receive: 500_000_000,
            amount: 1_000_000_000,
        })
        .instruction()
        .unwrap();
      ctx.execute_instruction(make_ix, &[&maker]).unwrap().assert_success();

      assert!(ctx.account_exists(&escrow_pda), "Escrow account should exist");
      ctx.svm.assert_token_balance(&maker_ata_a, 0);
      ctx.svm.assert_token_balance(&vault, 1_000_000_000);
}
