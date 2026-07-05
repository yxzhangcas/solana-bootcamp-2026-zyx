use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::system_program;
use anchor_lang::{prelude::*, solana_program::program::invoke};

use crate::event::WithdrawEvent;
use crate::{
    error::PrivateTransfersError,
    state::{NullifierSet, Pool},
    SUNSPOT_VERIFIER_ID,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [b"nullifiers", pool.key().as_ref()], bump)]
    pub nullifier_set: Account<'info, NullifierSet>,

    #[account(mut, seeds = [b"vault", pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    /// CHECK: Validated in instruction logic
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Validated by constraint
    #[account(constraint = verifier_program.key() == SUNSPOT_VERIFIER_ID @ PrivateTransfersError::InvalidVerifier)]
    pub verifier_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

fn withdraw(
    ctx: Context<Withdraw>,
    proof: Vec<u8>,
    nullifier_hash: [u8; 32],
    root: [u8; 32],
    recipient: Pubkey,
    amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let nullifier_set = &mut ctx.accounts.nullifier_set;

    require!(
        !nullifier_set.is_nullifier_used(&nullifier_hash),
        PrivateTransfersError::NullifierUsed
    );

    require!(
        pool.is_known_root(&root),
        PrivateTransfersError::InvalidRoot
    );

    // Prevents front-running by binding proof to recipient
    require!(
        ctx.accounts.recipient.key() == recipient,
        PrivateTransfersError::RecipientMismatch
    );

    require!(
        ctx.accounts.pool_vault.lamports() >= amount,
        PrivateTransfersError::InsufficientVaultBalance
    );

    // Verify ZK proof via CPI to Sunspot
    let public_inputs = encode_public_inputs(&root, &nullifier_hash, &recipient, amount);
    let instruction_data = [proof.as_slice(), public_inputs.as_slice()].concat();

    invoke(
        &Instruction {
            program_id: ctx.accounts.verifier_program.key(),
            accounts: vec![],
            data: instruction_data,
        },
        &[ctx.accounts.verifier_program.to_account_info()],
    )?;

    nullifier_set.mark_nullifier_used(nullifier_hash)?;

    let pool_key = pool.key();
    let seeds = &[
        b"vault".as_ref(),
        pool_key.as_ref(),
        &[ctx.bumps.pool_vault],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
        },
        signer_seeds,
    );
    system_program::transfer(cpi_context, amount)?;

    emit!(WithdrawEvent {
        nullifier_hash,
        recipient: ctx.accounts.recipient.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Withdrawal: {} lamports to {}", amount, recipient);
    Ok(())
}

/// Gnark witness format: 12-byte header + 4x32-byte public inputs
fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
) -> Vec<u8> {
    const NR_PUBLIC_INPUTS: u32 = 4;
    let mut inputs = Vec::with_capacity(12 + 128);

    // Header: num_public (4) | num_private (4) | vector_len (4)
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());
    inputs.extend_from_slice(&0u32.to_be_bytes());
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());

    inputs.extend_from_slice(root);
    inputs.extend_from_slice(nullifier_hash);
    inputs.extend_from_slice(recipient.as_ref());

    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    inputs.extend_from_slice(&amount_bytes);

    inputs
}

pub fn handle_withdraw(
    ctx: Context<Withdraw>,
    proof: Vec<u8>,
    nullifier_hash: [u8; 32],
    root: [u8; 32],
    recipient: Pubkey,
    amount: u64,
) -> Result<()> {
    withdraw(ctx, proof, nullifier_hash, root, recipient, amount)
}
