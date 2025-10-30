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
declare_id!("G3oRp71dn6S5TRmhXWXaURzGTtk485zSdZ6Xy46JkRDR");

#[program]
pub mod mpl_gov_micro {
    use super::*;

    /// Create a new election
    pub fn create_election(
        ctx: Context<CreateElection>,
        candidates: Vec<String>,
        start_time: i64,
        end_time: i64,
        use_compression: bool,
        max_voters: u32,
    ) -> Result<()> {
        instructions::create_election::handler(ctx, candidates, start_time, end_time, use_compression, max_voters)
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
        leaf_index: Option<u32>,
        registered_at: Option<i64>,
    ) -> Result<()> {
        instructions::cast_vote::handler(ctx, choice, merkle_proof, leaf_index, registered_at)
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

    // ===== BETA FEATURES (Not for Hackathon Demo) =====

    /// Enable private voting for an election (privacy layer integration)
    pub fn enable_private_voting(
        ctx: Context<EnablePrivateVoting>,
    ) -> Result<()> {
        instructions::privacy_interface::enable_private_voting(ctx)
    }

    /// Receive finalized tally from privacy layer (CPI only)
    pub fn receive_private_tally(
        ctx: Context<ReceivePrivateTally>,
        tally: Vec<u64>,
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::privacy_interface::receive_private_tally(ctx, tally, proof)
    }

    /// Disable private voting (before election starts)
    pub fn disable_private_voting(
        ctx: Context<DisablePrivateVoting>,
    ) -> Result<()> {
        instructions::privacy_interface::disable_private_voting(ctx)
    }

    /// Set a hook to execute when election succeeds
    pub fn set_success_hook(
        ctx: Context<SetSuccessHook>,
    ) -> Result<()> {
        instructions::hooks::set_success_hook(ctx)
    }

    /// Set a hook to execute when election fails
    pub fn set_failure_hook(
        ctx: Context<SetFailureHook>,
    ) -> Result<()> {
        instructions::hooks::set_failure_hook(ctx)
    }

    /// Clear all hooks
    pub fn clear_hooks(
        ctx: Context<ClearHooks>,
    ) -> Result<()> {
        instructions::hooks::clear_hooks(ctx)
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
