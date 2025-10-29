use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct CloseElection<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"election", authority.key().as_ref()],
        bump = election.bump
    )]
    pub election: Account<'info, Election>,
    
    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<CloseElection>,
) -> Result<()> {
    let election = &mut ctx.accounts.election;

    // Verify election can be closed
    require!(
        election.status != ElectionStatus::Ended,
        GovError::ElectionEnded
    );

    require!(
        election.status != ElectionStatus::Cancelled,
        GovError::ElectionNotActive
    );

    // Close the election
    election.status = ElectionStatus::Ended;

    msg!("Election closed by authority");
    msg!("Total votes: {}", election.total_votes);
    msg!("Results:");
    for (idx, count) in election.vote_counts.iter().enumerate() {
        msg!("  {}: {} votes", election.candidates[idx], count);
    }

    Ok(())
}
