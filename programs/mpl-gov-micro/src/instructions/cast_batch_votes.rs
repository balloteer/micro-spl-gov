use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::VoteInput;

#[derive(Accounts)]
pub struct CastBatchVotes<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,

    // NOTE: Remaining accounts will contain pairs of:
    // - Election account (mutable)
    // - VoterRegistration account
    // - NullifierSet account (mutable)
    // Pattern repeats for each vote
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, CastBatchVotes<'info>>,
    votes: Vec<VoteInput>,
) -> Result<()> {
    require!(
        votes.len() > 0 && votes.len() <= 50,
        GovError::InvalidBatchVote
    );

    let clock = Clock::get()?;
    let remaining_accounts = &mut ctx.remaining_accounts.iter();

    msg!("Processing batch of {} votes", votes.len());

    // Process each vote
    for (idx, vote_input) in votes.iter().enumerate() {
        // Get the three accounts for this vote
        let election_info = remaining_accounts
            .next()
            .ok_or(GovError::InvalidBatchVote)?;

        let voter_registration_info = remaining_accounts
            .next()
            .ok_or(GovError::InvalidBatchVote)?;

        let nullifier_set_info = remaining_accounts
            .next()
            .ok_or(GovError::InvalidBatchVote)?;

        // Deserialize accounts
        let mut election = Account::<Election>::try_from(election_info)?;
        let voter_registration = Account::<VoterRegistration>::try_from(voter_registration_info)?;
        let mut nullifier_set = Account::<NullifierSet>::try_from(nullifier_set_info)?;

        // Verify accounts are mutable
        require!(election_info.is_writable, GovError::InvalidBatchVote);
        require!(nullifier_set_info.is_writable, GovError::InvalidBatchVote);

        // Verify election matches
        require!(
            election.key() == vote_input.election,
            GovError::InvalidBatchVote
        );

        // Verify voter registration matches
        require!(
            voter_registration.wallet == ctx.accounts.voter.key(),
            GovError::NotRegistered
        );

        require!(
            voter_registration.election == vote_input.election,
            GovError::NotRegistered
        );

        // Verify election is active
        require!(
            clock.unix_timestamp >= election.start_time,
            GovError::ElectionNotStarted
        );

        require!(
            clock.unix_timestamp <= election.end_time,
            GovError::ElectionEnded
        );

        // Verify choice is valid
        require!(
            (vote_input.choice as usize) < election.candidates.len(),
            GovError::InvalidChoice
        );

        // Check nullifier not used
        require!(
            !nullifier_set.used_nullifiers.contains(&vote_input.nullifier),
            GovError::AlreadyVoted
        );

        // Update election status if needed
        if election.status == ElectionStatus::Pending && clock.unix_timestamp >= election.start_time {
            election.status = ElectionStatus::Active;
        }

        // Record the vote
        election.vote_counts[vote_input.choice as usize] = election.vote_counts[vote_input.choice as usize]
            .checked_add(1)
            .ok_or(GovError::ArithmeticOverflow)?;

        election.total_votes = election.total_votes
            .checked_add(1)
            .ok_or(GovError::ArithmeticOverflow)?;

        // Mark nullifier as used
        nullifier_set.used_nullifiers.push(vote_input.nullifier);

        // Save changes
        election.exit(&crate::ID)?;
        nullifier_set.exit(&crate::ID)?;

        msg!("Batch vote {}/{} processed for election {}", idx + 1, votes.len(), vote_input.election);
    }

    msg!("Batch voting complete: {} votes processed", votes.len());

    Ok(())
}
