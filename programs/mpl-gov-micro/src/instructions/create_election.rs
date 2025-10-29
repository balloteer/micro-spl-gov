use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct CreateElection<'info> {
    #[account(
        init,
        payer = authority,
        space = Election::MAX_SIZE,
        seeds = [b"election", authority.key().as_ref()],
        bump
    )]
    pub election: Account<'info, Election>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateElection>,
    candidates: Vec<String>,
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    let election = &mut ctx.accounts.election;
    let clock = Clock::get()?;

    // Validation
    require!(
        candidates.len() > 0 && candidates.len() <= Election::MAX_CANDIDATES,
        GovError::TooManyCandidates
    );

    for candidate in &candidates {
        require!(
            candidate.len() <= Election::MAX_CANDIDATE_NAME_LEN,
            GovError::CandidateNameTooLong
        );
    }

    require!(
        end_time > start_time,
        GovError::InvalidTimeRange
    );

    require!(
        start_time >= clock.unix_timestamp,
        GovError::StartTimeInPast
    );

    // Initialize election
    election.authority = ctx.accounts.authority.key();
    election.candidates = candidates.clone();
    election.vote_counts = vec![0; candidates.len()];
    election.total_votes = 0;
    election.voter_merkle_root = [0; 32]; // Will be updated when voters register
    election.start_time = start_time;
    election.end_time = end_time;

    // Set status based on start time
    election.status = if start_time <= clock.unix_timestamp {
        ElectionStatus::Active
    } else {
        ElectionStatus::Pending
    };

    election.bump = ctx.bumps.election;

    msg!("Election created with {} candidates", candidates.len());
    msg!("Start: {}, End: {}", start_time, end_time);

    Ok(())
}
