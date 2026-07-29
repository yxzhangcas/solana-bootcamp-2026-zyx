use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::Signer;
use anchor_litesvm::TestHelpers;
use anchor_litesvm::{AnchorContext, Keypair};
use anchor_spl::associated_token;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022;

declare_program!(labubu_vault);

pub fn initialize_collection(ctx: &mut AnchorContext, authority: &Keypair) {
    let collection_pda = ctx.svm.get_pda(&[b"collection"], &labubu_vault::ID);

    let ix = ctx
        .program()
        .accounts(labubu_vault::client::accounts::InitializeCollection {
            authority: authority.pubkey(),
            collection: collection_pda,
            system_program: system_program::ID,
        })
        .args(labubu_vault::client::args::InitializeCollection {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&authority])
        .unwrap()
        .assert_success();
}

pub fn create_mint(ctx: &mut AnchorContext, authority: &Keypair, labubu_id: u8) {
    let collection_pda = ctx.svm.get_pda(&[b"collection"], &labubu_vault::ID);
    let mint_pda = ctx
        .svm
        .get_pda(&[b"labubu_mint".as_ref(), &[labubu_id]], &labubu_vault::ID);

    let ix = ctx
        .program()
        .accounts(labubu_vault::client::accounts::CreateMint {
            authority: authority.pubkey(),
            collection: collection_pda,
            labubu_mint: mint_pda,
            token_program: token_2022::ID,
            system_program: system_program::ID,
            rent: rent::ID,
        })
        .args(labubu_vault::client::args::CreateMint { labubu_id })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&authority])
        .unwrap()
        .assert_success();
}

pub fn mint_random(ctx: &mut AnchorContext, user: &Keypair, labubu_id: u8) {
    let collection_pda = ctx.svm.get_pda(&[b"collection"], &labubu_vault::ID);
    let mint_pda = ctx
        .svm
        .get_pda(&[b"labubu_mint".as_ref(), &[labubu_id]], &labubu_vault::ID);
    let user_token_account =
        get_associated_token_address_with_program_id(&user.pubkey(), &mint_pda, &token_2022::ID);

    let ix = ctx
        .program()
        .accounts(labubu_vault::client::accounts::MintRandom {
            user: user.pubkey(),
            collection: collection_pda,
            labubu_mint: mint_pda,
            user_token_account: user_token_account,
            token_program: token_2022::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        })
        .args(labubu_vault::client::args::MintRandom { labubu_id })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&user])
        .unwrap()
        .assert_success();
}

// Token2022貌似没有更好的解析办法，此处直接读取对应位置的字节，解析出balance信息
pub fn get_token_balance(ctx: &mut AnchorContext, token_account: &Pubkey) -> u64 {
  let account = ctx
      .svm
      .get_account(token_account)
      .expect("Token account should exist");
  let data = &account.data;
  u64::from_le_bytes(data[64..72].try_into().unwrap()) // 直接根据offset取出对应字节，并未完整解析
}