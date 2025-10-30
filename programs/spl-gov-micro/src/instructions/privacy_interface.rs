/// Privacy Layer Interface (BETA)
///
/// This module provides CPI interfaces for the privacy layer program.
/// See docs/PRIVACY_LAYER_SPEC.md for full implementation details.
///
/// NOTE: This is a stub implementation - actual privacy layer is in separate repo.

use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct EnablePrivateVoting<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    /// Privacy layer program that will handle encrypted votes
    /// CHECK: Validated by checking it's a valid program
    pub privacy_layer_program: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Enable private voting for an election
///
/// This instruction connects the election to a privacy layer program
/// that will handle encrypted votes, ZK proofs, and MPC tallying.
///
/// Can only be called before election starts.
pub fn enable_private_voting(
    ctx: Context<EnablePrivateVoting>,
) -> Result<()> {
    let election = &mut ctx.accounts.election;

    // Only authority can enable privacy
    require!(
        election.authority == ctx.accounts.authority.key(),
        GovError::Unauthorized
    );

    // Can only enable before election starts
    require!(
        election.status == ElectionStatus::Pending,
        GovError::ElectionAlreadyStarted
    );

    // Verify privacy program is executable
    require!(
        ctx.accounts.privacy_layer_program.executable,
        GovError::InvalidPrivacyProgram
    );

    // Enable privacy
    election.privacy_enabled = true;
    election.privacy_layer_program = ctx.accounts.privacy_layer_program.key();

    msg!("Private voting enabled for election");
    msg!("Privacy program: {}", election.privacy_layer_program);

    Ok(())
}

#[derive(Accounts)]
pub struct ReceivePrivateTally<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    /// Privacy layer program making the CPI call
    /// CHECK: Must match election.privacy_layer_program
    pub privacy_layer_program: Signer<'info>,

    pub authority: Signer<'info>,
}

/// Receive finalized tally from privacy layer (via CPI)
///
/// The privacy layer program calls this after MPC computation
/// to update the election with the final decrypted results.
///
/// This is the ONLY way to update results for a private election.
pub fn receive_private_tally(
    ctx: Context<ReceivePrivateTally>,
    tally: Vec<u64>,
    _proof: Vec<u8>, // TallyProof (not validated in MVP)
) -> Result<()> {
    let election = &mut ctx.accounts.election;

    // Verify election has privacy enabled
    require!(
        election.privacy_enabled,
        GovError::PrivacyNotEnabled
    );

    // Verify CPI caller is registered privacy layer
    require!(
        ctx.accounts.privacy_layer_program.key() == election.privacy_layer_program,
        GovError::UnauthorizedPrivacyLayer
    );

    // Verify tally length matches candidates
    require!(
        tally.len() == election.candidates.len(),
        GovError::InvalidTally
    );

    // In production: verify ZK proof of correct tally
    // For MVP: trust the privacy layer

    // Update vote counts
    election.vote_counts = tally.clone();
    election.total_votes = tally.iter().sum();

    msg!("Private tally received and updated");
    msg!("Total votes: {}", election.total_votes);

    Ok(())
}

#[derive(Accounts)]
pub struct DisablePrivateVoting<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Disable private voting (before election starts)
///
/// Allows reverting to public voting if privacy was enabled by mistake.
pub fn disable_private_voting(
    ctx: Context<DisablePrivateVoting>,
) -> Result<()> {
    let election = &mut ctx.accounts.election;

    require!(
        election.authority == ctx.accounts.authority.key(),
        GovError::Unauthorized
    );

    require!(
        election.status == ElectionStatus::Pending,
        GovError::ElectionAlreadyStarted
    );

    election.privacy_enabled = false;
    election.privacy_layer_program = Pubkey::default();

    msg!("Private voting disabled");

    Ok(())
}
