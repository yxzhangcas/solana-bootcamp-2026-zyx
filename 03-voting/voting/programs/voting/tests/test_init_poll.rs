use anchor_litesvm::TransactionResult;
use anchor_litesvm::{
    AnchorContext, AnchorLiteSVM, AssertionHelpers, Keypair, Pubkey, Signer, TestHelpers,
};

use anchor_lang::declare_program;
use anchor_lang::prelude::*;

declare_program!(voting);

use self::voting::accounts::{CandidateState, PollState};
use self::voting::client::{accounts, args};
use self::voting::ID;

const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/voting.so");

pub fn setup() -> AnchorContext {
    use anchor_lang::solana_program::clock::Clock;
    let mut ctx = AnchorLiteSVM::build_with_program(ID, PROGRAM_BYTES);
    let clock = Clock {
        slot: 1000,
        epoch_start_timestamp: 0,
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: 1000,
    };
    ctx.svm.set_sysvar(&clock);
    ctx
}

fn poll_state_address(poll_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"poll", &poll_id.to_le_bytes()], &voting::ID).0
}

fn candidate_state_address(poll_id: u64, candidate: &str) -> Pubkey {
    Pubkey::find_program_address(&[&poll_id.to_le_bytes(), candidate.as_bytes()], &voting::ID).0
}

fn init_poll(
    ctx: &mut AnchorContext,
    signer: &Keypair,
    poll_id: u64,
    name: &str,
    description: &str,
    start_time: u64,
    stop_time: u64,
) {
    let poll_pda = poll_state_address(poll_id);
    let ix = ctx
        .program()
        .accounts(accounts::InitPoll {
            signer: signer.pubkey(),
            poll_state: poll_pda,
            system_program: system_program::ID,
        })
        .args(args::InitPoll {
            poll_id,
            name: name.to_string(),
            description: description.to_string(),
            start_time,
            stop_time,
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
    ctx.svm.assert_account_exists(&poll_pda);
}

#[test]
fn test_init_poll() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(1_000_000_000).unwrap();

    let poll_id: u64 = 1;
    let name: &str = "Test Poll";
    let description: &str = "A test poll for voting";
    let start_time: u64 = 0;
    let stop_time: u64 = u64::MAX / 2;
    init_poll(
        &mut ctx,
        &user,
        poll_id,
        name,
        description,
        start_time,
        stop_time,
    );

    let poll_pda = poll_state_address(poll_id);
    let poll_state: PollState = ctx.get_account(&poll_pda).unwrap();

    assert_eq!(poll_state.name, name);
    assert_eq!(poll_state.description, description);
    assert_eq!(poll_state.voting_start, start_time);
    assert_eq!(poll_state.voting_stop, stop_time);
    assert_eq!(poll_state.option_index, 0);
}

fn init_candidate(ctx: &mut AnchorContext, signer: &Keypair, poll_id: u64, candidate: &str) {
    let poll_pda = poll_state_address(poll_id);
    let candidate_pda = candidate_state_address(poll_id, candidate);
    let ix = ctx
        .program()
        .accounts(accounts::InitCandidate {
            signer: signer.pubkey(),
            poll_state: poll_pda,
            candidate_state: candidate_pda,
            system_program: system_program::ID,
        })
        .args(args::InitCandidate {
            poll_id,
            candidate: candidate.to_string(),
        })
        .instruction()
        .unwrap();
    let result = ctx.execute_instruction(ix, &[signer]).unwrap();
    result.assert_success();
    ctx.svm.assert_account_exists(&candidate_pda);
}

#[test]
fn test_init_candidate() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(1_000_000_000).unwrap();

    let poll_id: u64 = 1;
    let name: &str = "Test Poll";
    let description: &str = "A test poll for voting";
    let start_time: u64 = 0;
    let stop_time: u64 = u64::MAX / 2;
    init_poll(
        &mut ctx,
        &user,
        poll_id,
        name,
        description,
        start_time,
        stop_time,
    );
    init_candidate(&mut ctx, &user, poll_id, "Alice");
    init_candidate(&mut ctx, &user, poll_id, "Bob");

    let poll_pda = poll_state_address(poll_id);
    let poll_state: PollState = ctx.get_account(&poll_pda).unwrap();
    let alice_state: CandidateState = ctx
        .get_account(&candidate_state_address(poll_id, "Alice"))
        .unwrap();
    let bob_state: CandidateState = ctx
        .get_account(&candidate_state_address(poll_id, "Bob"))
        .unwrap();

    assert_eq!(poll_state.option_index, 2);
    assert_eq!(alice_state.name, "Alice");
    assert_eq!(alice_state.votes, 0);
    assert_eq!(bob_state.name, "Bob");
    assert_eq!(bob_state.votes, 0);
}

fn exec_vote(
    ctx: &mut AnchorContext,
    signer: &Keypair,
    poll_id: u64,
    candidate: &str,
) -> TransactionResult {
    let poll_pda = poll_state_address(poll_id);
    let candidate_pda = candidate_state_address(poll_id, candidate);
    let ix = ctx
        .program()
        .accounts(accounts::Vote {
            signer: signer.pubkey(),
            poll_state: poll_pda,
            candidate_state: candidate_pda,
        })
        .args(args::Vote {
            poll_id,
            candidate: candidate.to_string(),
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[signer]).unwrap()
}

#[test]
fn test_vote() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id: u64 = 1;
    let name: &str = "Test Poll";
    let description: &str = "A test poll for voting";
    let start_time: u64 = 0;
    let stop_time: u64 = u64::MAX / 2;
    init_poll(
        &mut ctx,
        &user,
        poll_id,
        name,
        description,
        start_time,
        stop_time,
    );
    init_candidate(&mut ctx, &user, poll_id, "Alice");

    let result = exec_vote(&mut ctx, &voter, poll_id, "Alice");
    result.assert_success();

    let alice_state: CandidateState = ctx
        .get_account(&candidate_state_address(poll_id, "Alice"))
        .unwrap();
    assert_eq!(alice_state.votes, 1);
}

#[test]
fn test_vote_before_start_fails() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    init_poll(
        &mut ctx,
        &user,
        poll_id,
        "Future Poll",
        "Voting has not started yet",
        1_500,
        2_000,
    );
    init_candidate(&mut ctx, &user, poll_id, "Alice");

    exec_vote(&mut ctx, &voter, poll_id, "Alice")
        .assert_failure()
        .assert_anchor_error("VotingNotStarted");
}

#[test]
fn test_vote_after_stop_fails() {
    let mut ctx = setup();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let voter = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    let poll_id = 1;
    init_poll(
        &mut ctx,
        &user,
        poll_id,
        "Closed Poll",
        "Voting has stopped",
        0,
        500,
    );
    init_candidate(&mut ctx, &user, poll_id, "Alice");

    exec_vote(&mut ctx, &voter, poll_id, "Alice")
        .assert_failure()
        .assert_anchor_error("VotingStopped");
}
