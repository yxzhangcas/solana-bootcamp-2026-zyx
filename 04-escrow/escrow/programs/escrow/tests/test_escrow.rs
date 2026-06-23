use anchor_lang::prelude::*;
use anchor_litesvm::{AnchorLiteSVM, AssertionHelpers, Signer, TestHelpers};
use anchor_spl::associated_token::{get_associated_token_address, spl_associated_token_account};
use anchor_spl::token::spl_token;

declare_program!(escrow);

use self::escrow::accounts::Escrow;
use self::escrow::client::{accounts, args};
use self::escrow::ID;

#[test]
fn test_escrow_make_and_take() {
    let program_id = ID;
    let mut ctx = AnchorLiteSVM::build_with_program(
        program_id,
        include_bytes!("../../../target/deploy/escrow.so"),
    );
    // 充值SOL
    let maker = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let taker = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let mint_a = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let mint_b = ctx.svm.create_token_mint(&maker, 9).unwrap();
    let maker_ata_a = ctx
        .svm
        .create_associated_token_account(&mint_a.pubkey(), &maker)
        .unwrap();
    // 充值MINT
    ctx.svm
        .mint_to(&mint_a.pubkey(), &maker_ata_a, &maker, 1_000_000_000)
        .unwrap();
    let taker_ata_b = ctx
        .svm
        .create_associated_token_account(&mint_b.pubkey(), &taker)
        .unwrap();
    // 充值MINT
    ctx.svm
        .mint_to(&mint_b.pubkey(), &taker_ata_b, &maker, 500_000_000)
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
    ctx.execute_instruction(make_ix, &[&maker])
        .unwrap()
        .assert_success();

    assert!(
        ctx.account_exists(&escrow_pda),
        "Escrow account should exist"
    );
    ctx.svm.assert_token_balance(&maker_ata_a, 0);
    ctx.svm.assert_token_balance(&vault, 1_000_000_000);
    ctx.svm.assert_token_balance(&taker_ata_b, 500_000_000);

    let escrow_account: Escrow = ctx.get_account(&escrow_pda).unwrap();
    assert_eq!(escrow_account.receive, 500_000_000);

    let taker_ata_a = get_associated_token_address(&taker.pubkey(), &mint_a.pubkey());
    let maker_ata_b = get_associated_token_address(&maker.pubkey(), &mint_b.pubkey());

    let take_ix = ctx
        .program()
        .accounts(accounts::ExecTake {
            taker: taker.pubkey(),
            maker: maker.pubkey(),
            escrow: escrow_pda,
            mint_a: mint_a.pubkey(),
            mint_b: mint_b.pubkey(),
            vault,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            associated_token_program: spl_associated_token_account::program::ID,
            token_program: spl_token::ID,
            system_program: system_program::ID,
        })
        .args(args::ExecTake {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(take_ix, &[&taker])
        .unwrap()
        .assert_success();

    ctx.svm.assert_account_closed(&escrow_pda);
    ctx.svm.assert_account_closed(&vault);

    ctx.svm.assert_token_balance(&taker_ata_a, 1_000_000_000);
    ctx.svm.assert_token_balance(&taker_ata_b, 0);
    ctx.svm.assert_token_balance(&maker_ata_b, 500_000_000);
}
