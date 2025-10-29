use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    #[account(
        seeds = [
            b"voter_registration",
            election.key().as_ref(),
            voter.key().as_ref()
        ],
        bump
    )]
    pub voter_registration: Account<'info, VoterRegistration>,

    #[account(
        init_if_needed,
        payer = voter,
        space = NullifierSet::INIT_SIZE + (32 * 100), // Initial space for 100 nullifiers
        seeds = [b"nullifiers", election.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSet>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CastVote>,
    choice: u8,
    _merkle_proof: Vec<[u8; 32]>,
) -> Result<()> {
    let election = &mut ctx.accounts.election;
    let nullifier_set = &mut ctx.accounts.nullifier_set;
    let voter_registration = &ctx.accounts.voter_registration;
    let clock = Clock::get()?;

    // Verify election is active
    require!(
        clock.unix_timestamp >= election.start_time,
        GovError::ElectionNotStarted
    );

    require!(
        clock.unix_timestamp <= election.end_time,
        GovError::ElectionEnded
    );

    require!(
        election.status == ElectionStatus::Active ||
        (election.status == ElectionStatus::Pending && clock.unix_timestamp >= election.start_time),
        GovError::ElectionNotActive
    );

    // Verify choice is valid
    require!(
        (choice as usize) < election.candidates.len(),
        GovError::InvalidChoice
    );

    // Create nullifier for this vote
    let nullifier = VoteNullifier::new(
        &voter_registration.wallet,
        &election.key(),
        0 // nonce - for MVP we use 0, in production this could be dynamic
    );

    // Initialize nullifier set if needed
    if nullifier_set.election == Pubkey::default() {
        nullifier_set.election = election.key();
        nullifier_set.used_nullifiers = Vec::new();
        nullifier_set.bump = ctx.bumps.nullifier_set;
    }

    // Check if voter has already voted
    require!(
        !nullifier_set.used_nullifiers.contains(&nullifier.nullifier_hash),
        GovError::AlreadyVoted
    );

    // Update election status if needed
    if election.status == ElectionStatus::Pending && clock.unix_timestamp >= election.start_time {
        election.status = ElectionStatus::Active;
    }

    // Record the vote
    election.vote_counts[choice as usize] = election.vote_counts[choice as usize]
        .checked_add(1)
        .ok_or(GovError::ArithmeticOverflow)?;

    election.total_votes = election.total_votes
        .checked_add(1)
        .ok_or(GovError::ArithmeticOverflow)?;

    // Mark nullifier as used
    nullifier_set.used_nullifiers.push(nullifier.nullifier_hash);

    msg!("Vote cast for candidate {} by voter {}", choice, voter_registration.wallet);
    msg!("Total votes: {}", election.total_votes);

    Ok(())
}
