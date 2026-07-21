use anchor_lang::declare_program;
use anchor_lang::prelude::*;
use anchor_litesvm::AnchorLiteSVM;
use anchor_litesvm::Signer;
use anchor_litesvm::{AnchorContext, Keypair};
use anchor_spl::{associated_token, token_2022};

declare_program!(stable_coin);

pub fn create_ctx() -> AnchorContext {
    AnchorLiteSVM::build_with_program(
        stable_coin::ID,
        include_bytes!("../../../target/deploy/stable_coin.so"),
    )
}

/* PDAs */
pub fn get_config_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"config"], &stable_coin::ID).0
}
pub fn get_mint_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"mint"], &stable_coin::ID).0
}
pub fn get_minter_config_pda(minter: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"minter", minter.as_ref()], &stable_coin::ID).0
}
pub fn get_ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[wallet.as_ref(), token_2022::ID.as_ref(), mint.as_ref()],
        &associated_token::ID,
    )
    .0
}

pub fn get_token_balance(ctx: &mut AnchorContext, token_account: &Pubkey) -> u64 {
    let account = ctx
        .svm
        .get_account(token_account)
        .expect("Token account should exist");
    let data = &account.data;
    u64::from_le_bytes(data[64..72].try_into().unwrap()) // 直接根据offset取出对应字节，并未完整解析
}

pub fn initialize(ctx: &mut AnchorContext, admin: &Keypair) {
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
}

pub fn configure(ctx: &mut AnchorContext, admin: &Keypair, minter: &Keypair, allowance: u64) {
    let config_pda = get_config_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::ConfigureMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::ConfigureMinter { allowance })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();
}

pub fn remove(ctx: &mut AnchorContext, admin: &Keypair, minter: &Keypair) {
    let config_pda = get_config_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::RemoveMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
        })
        .args(stable_coin::client::args::RemoveMinter {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();
}

pub fn mint(ctx: &mut AnchorContext, minter: &Keypair, recipient: &Pubkey, amount: u64) {
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(&recipient, &mint_pda);

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: *recipient,
            token_program: token_2022::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        })
        .args(stable_coin::client::args::MintTokens { amount: amount })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&minter])
        .unwrap()
        .assert_success();
}

pub fn burn(ctx: &mut AnchorContext, recipient: &Keypair, amount: u64) {
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::BurnTokens {
            owner: recipient.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_account: destination_ata,
            token_program: token_2022::ID,
        })
        .args(stable_coin::client::args::BurnTokens { amount: amount })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&recipient])
        .unwrap()
        .assert_success();
}

pub fn pause(ctx: &mut AnchorContext, admin: &Keypair) {
    let config_pda = get_config_pda();
    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::Pause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(stable_coin::client::args::Pause {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();
}

pub fn unpause(ctx: &mut AnchorContext, admin: &Keypair) {
    let config_pda = get_config_pda();
    let ix = ctx
        .program()
        .accounts(stable_coin::client::accounts::Unpause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(stable_coin::client::args::Unpause {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();
}