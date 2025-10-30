use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::compression::get_merkle_tree_size;

#[cfg(feature = "compression")]
use spl_account_compression::program::SplAccountCompression;

#[derive(Accounts)]
#[instruction(candidates: Vec<String>, start_time: i64, end_time: i64, use_compression: bool, max_voters: u32)]
pub struct CreateElection<'info> {
    #[account(
        init,
        payer = authority,
        space = Election::MAX_SIZE,
        seeds = [b"election", authority.key().as_ref()],
        bump
    )]
    pub election: Account<'info, Election>,

    /// Merkle tree for compressed voter registrations (optional, only if use_compression = true)
    /// CHECK: Merkle tree account is validated in handler when compression is enabled
    #[account(mut)]
    pub merkle_tree: Option<AccountInfo<'info>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Compression program is validated when used
    pub compression_program: Option<AccountInfo<'info>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateElection>,
    candidates: Vec<String>,
    start_time: i64,
    end_time: i64,
    use_compression: bool,
    max_voters: u32,
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
    election.total_registered = 0;
    election.voter_merkle_root = [0; 32]; // Will be updated when voters register
    election.start_time = start_time;
    election.end_time = end_time;
    election.use_compression = use_compression;

    // Set status based on start time
    election.status = if start_time <= clock.unix_timestamp {
        ElectionStatus::Active
    } else {
        ElectionStatus::Pending
    };

    // Handle compression setup
    if use_compression {
        // In production, merkle tree and compression program should be provided
        // For MVP, we allow optional merkle tree setup
        if ctx.accounts.merkle_tree.is_some() && ctx.accounts.compression_program.is_some() {
            let merkle_tree = ctx.accounts.merkle_tree.as_ref().unwrap();
            let compression_program = ctx.accounts.compression_program.as_ref().unwrap();

            // Store merkle tree account
            election.merkle_tree = merkle_tree.key();

            msg!("Compression enabled with merkle tree: {}", merkle_tree.key());
        } else {
            // MVP mode: compression enabled but merkle tree setup deferred to client
            msg!("Compression enabled (MVP mode - merkle tree setup deferred to client)");
            election.merkle_tree = Pubkey::default();
        }

        // Initialize merkle tree with appropriate size (only if accounts provided)
        if ctx.accounts.merkle_tree.is_some() && ctx.accounts.compression_program.is_some() {
            let merkle_tree = ctx.accounts.merkle_tree.as_ref().unwrap();
            let compression_program = ctx.accounts.compression_program.as_ref().unwrap();
            let (depth, buffer_size) = get_merkle_tree_size(max_voters);

            #[cfg(feature = "compression")]
            {
                use crate::utils::compression::initialize_voter_merkle_tree;
                initialize_voter_merkle_tree(
                    merkle_tree,
                    &ctx.accounts.authority.to_account_info(),
                    &ctx.accounts.authority.to_account_info(), // payer
                    compression_program,
                    &ctx.accounts.system_program.to_account_info(),
                    depth,
                    buffer_size,
                )?;
            }

            msg!("Compression enabled - initialized merkle tree with depth {}", depth);
        }
    } else {
        // No compression - set merkle_tree to default
        election.merkle_tree = Pubkey::default();
        msg!("Compression disabled - using regular accounts");
    }

    // Beta features - initialize as disabled
    election.privacy_enabled = false;
    election.privacy_layer_program = Pubkey::default();
    election.on_success_hook = None;
    election.on_failure_hook = None;

    election.bump = ctx.bumps.election;

    msg!("Election created with {} candidates", candidates.len());
    msg!("Start: {}, End: {}", start_time, end_time);
    msg!("Compression: {}", use_compression);

    Ok(())
}
