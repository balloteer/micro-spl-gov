use anchor_lang::prelude::*;

// Module declarations
pub mod errors;
pub mod state;
pub mod instructions;
pub mod utils;

// Re-exports
pub use errors::*;
pub use state::*;
pub use instructions::*;

// Program ID - This will be updated after deployment
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mpl_gov_micro {
    use super::*;

    /// Create a new election
    pub fn create_election(
        ctx: Context<CreateElection>,
        candidates: Vec<String>,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        instructions::create_election::handler(ctx, candidates, start_time, end_time)
    }

    /// Register a voter for an election (with compression)
    pub fn register_voter(
        ctx: Context<RegisterVoter>,
    ) -> Result<()> {
        instructions::register_voter::handler(ctx)
    }

    /// Cast a vote
    pub fn cast_vote(
        ctx: Context<CastVote>,
        choice: u8,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        instructions::cast_vote::handler(ctx, choice, merkle_proof)
    }

    /// Cast multiple votes in a batch
    pub fn cast_batch_votes<'info>(
        ctx: Context<'_, '_, 'info, 'info, CastBatchVotes<'info>>,
        votes: Vec<VoteInput>,
    ) -> Result<()> {
        instructions::cast_batch_votes::handler(ctx, votes)
    }

    /// Close an election (only authority)
    pub fn close_election(
        ctx: Context<CloseElection>,
    ) -> Result<()> {
        instructions::close_election::handler(ctx)
    }
}

/// Input for a single vote in batch voting
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VoteInput {
    /// The election to vote in
    pub election: Pubkey,
    /// The choice to vote for
    pub choice: u8,
    /// Merkle proof showing voter is registered
    pub merkle_proof: Vec<[u8; 32]>,
    /// Nullifier to prevent double voting
    pub nullifier: [u8; 32],
}
