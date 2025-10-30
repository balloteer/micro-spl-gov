use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::compression::{CompressedVoterData, verify_compressed_voter_proof};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    /// Voter registration account (only required for legacy mode)
    /// In compression mode, voter eligibility is verified via merkle proof
    #[account(
        seeds = [
            b"voter_registration",
            election.key().as_ref(),
            voter.key().as_ref()
        ],
        bump
    )]
    pub voter_registration: Option<Account<'info, VoterRegistration>>,

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

    /// CHECK: Attestation account (optional, only for compression mode proof verification)
    pub attestation: Option<UncheckedAccount<'info>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CastVote>,
    choice: u8,
    merkle_proof: Vec<[u8; 32]>,
    leaf_index: Option<u32>,
    registered_at: Option<i64>,
) -> Result<()> {
    let election = &mut ctx.accounts.election;
    let nullifier_set = &mut ctx.accounts.nullifier_set;
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

    // Check if election has been manually ended or cancelled
    require!(
        election.status != ElectionStatus::Ended,
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

    let voter_key = ctx.accounts.voter.key();
    let election_key = election.key();

    // Verify voter eligibility based on compression mode
    if election.use_compression {
        // ===== COMPRESSION MODE: Verify via merkle proof =====
        msg!("Verifying voter via merkle proof (compression mode)");

        // Validate required parameters for compression mode
        require!(
            leaf_index.is_some() && registered_at.is_some() && ctx.accounts.attestation.is_some(),
            GovError::InvalidMerkleProof
        );

        let attestation_key = ctx.accounts.attestation.as_ref().unwrap().key();

        // Reconstruct the voter data to generate leaf hash
        let compressed_data = CompressedVoterData::new(
            voter_key,
            election_key,
            attestation_key,
            registered_at.unwrap(),
        );

        let leaf_hash = compressed_data.to_leaf_hash()?;

        // Verify the merkle proof
        let is_valid = verify_compressed_voter_proof(
            &election.voter_merkle_root,
            &leaf_hash,
            &merkle_proof,
            leaf_index.unwrap(),
        )?;

        require!(is_valid, GovError::InvalidMerkleProof);

        msg!("Merkle proof verified for voter: {}", voter_key);

    } else {
        // ===== LEGACY MODE: Verify via voter registration account =====
        msg!("Verifying voter via registration account (legacy mode)");

        require!(
            ctx.accounts.voter_registration.is_some(),
            GovError::NotRegistered
        );

        let voter_registration = ctx.accounts.voter_registration.as_ref().unwrap();

        // Verify voter registration matches
        require!(
            voter_registration.wallet == voter_key,
            GovError::NotRegistered
        );

        msg!("Voter registration verified: {}", voter_key);
    }

    // Create nullifier for this vote (same for both modes)
    let nullifier = VoteNullifier::new(
        &voter_key,
        &election_key,
        0 // nonce - for MVP we use 0, in production this could be dynamic
    );

    // Initialize nullifier set if needed
    if nullifier_set.election == Pubkey::default() {
        nullifier_set.election = election_key;
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

    msg!("Vote cast for candidate {} by voter {}", choice, voter_key);
    msg!("Total votes: {}", election.total_votes);
    msg!("Compression mode: {}", election.use_compression);

    Ok(())
}
