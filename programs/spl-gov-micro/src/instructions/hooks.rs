/// Execution Hooks (BETA)
///
/// This module provides execution hooks - arbitrary code that can be executed
/// after an election ends based on the outcome (success/failure).
///
/// Use cases:
/// - Execute treasury transfers based on proposal outcome
/// - Trigger on-chain actions (update parameters, mint tokens, etc.)
/// - Call other programs with election results
/// - Automate DAO governance actions

use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct SetSuccessHook<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    /// Program to call when election succeeds
    /// CHECK: Validated by checking it's executable
    pub hook_program: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Set a hook to execute when election succeeds (passes threshold)
///
/// The hook program will be called via CPI when the election is finalized
/// and the winning proposal meets the success threshold.
///
/// Can only be set before election starts.
pub fn set_success_hook(
    ctx: Context<SetSuccessHook>,
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

    require!(
        ctx.accounts.hook_program.executable,
        GovError::InvalidHookProgram
    );

    election.on_success_hook = Some(ctx.accounts.hook_program.key());

    msg!("Success hook set: {}", ctx.accounts.hook_program.key());

    Ok(())
}

#[derive(Accounts)]
pub struct SetFailureHook<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    /// Program to call when election fails
    /// CHECK: Validated by checking it's executable
    pub hook_program: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Set a hook to execute when election fails (doesn't pass threshold)
pub fn set_failure_hook(
    ctx: Context<SetFailureHook>,
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

    require!(
        ctx.accounts.hook_program.executable,
        GovError::InvalidHookProgram
    );

    election.on_failure_hook = Some(ctx.accounts.hook_program.key());

    msg!("Failure hook set: {}", ctx.accounts.hook_program.key());

    Ok(())
}

#[derive(Accounts)]
pub struct ClearHooks<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Clear all hooks (before election starts)
pub fn clear_hooks(
    ctx: Context<ClearHooks>,
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

    election.on_success_hook = None;
    election.on_failure_hook = None;

    msg!("Hooks cleared");

    Ok(())
}

/// Execution hook context passed to hook programs
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct HookContext {
    pub election: Pubkey,
    pub winning_choice: u8,
    pub winning_votes: u64,
    pub total_votes: u64,
    pub passed_threshold: bool,
    pub timestamp: i64,
}

/// Execute hooks after election ends
///
/// This is called internally by close_election() - not a direct instruction.
/// The appropriate hook is called based on whether the proposal passed.
///
/// Hook programs receive HookContext as instruction data.
pub fn execute_hooks(
    election: &Election,
    election_pubkey: Pubkey,
    hook_program: &AccountInfo,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // Determine if election passed (simple majority for MVP)
    let total_votes = election.total_votes;
    let max_votes = election.vote_counts.iter().max().copied().unwrap_or(0);

    let passed_threshold = if total_votes > 0 {
        // Simple majority: more than 50% of votes
        max_votes > (total_votes / 2)
    } else {
        false
    };

    // Find winning choice
    let winning_choice = election
        .vote_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &votes)| votes)
        .map(|(idx, _)| idx as u8)
        .unwrap_or(0);

    // Prepare hook context
    let hook_ctx = HookContext {
        election: election_pubkey,
        winning_choice,
        winning_votes: max_votes,
        total_votes,
        passed_threshold,
        timestamp: Clock::get()?.unix_timestamp,
    };

    // Serialize context for CPI
    let mut data = Vec::new();
    data.extend_from_slice(&hook_ctx.try_to_vec()?);

    msg!("Executing hook: {}", hook_program.key());
    msg!("Passed: {}, Winner: {}, Votes: {}/{}",
        passed_threshold, winning_choice, max_votes, total_votes);

    // CPI to hook program
    // Note: Hook program must have an instruction that accepts HookContext
    solana_program::program::invoke(
        &solana_program::instruction::Instruction {
            program_id: hook_program.key(),
            accounts: remaining_accounts
                .iter()
                .map(|acc| solana_program::instruction::AccountMeta {
                    pubkey: acc.key(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            data,
        },
        remaining_accounts,
    )?;

    msg!("Hook executed successfully");

    Ok(())
}

/// Helper to determine which hook to execute
pub fn get_active_hook(election: &Election) -> Option<Pubkey> {
    // Check if election passed threshold
    let total_votes = election.total_votes;
    if total_votes == 0 {
        return election.on_failure_hook;
    }

    let max_votes = election.vote_counts.iter().max().copied().unwrap_or(0);
    let passed = max_votes > (total_votes / 2);

    if passed {
        election.on_success_hook
    } else {
        election.on_failure_hook
    }
}

// Example hook program instruction layout:
//
// ```rust
// #[derive(Accounts)]
// pub struct ExecuteProposal<'info> {
//     // ... your accounts ...
// }
//
// pub fn execute_proposal(
//     ctx: Context<ExecuteProposal>,
//     hook_ctx: HookContext,
// ) -> Result<()> {
//     // Hook context contains election results
//     require!(hook_ctx.passed_threshold, MyError::ProposalFailed);
//
//     // Execute your logic based on results
//     match hook_ctx.winning_choice {
//         0 => execute_option_a(ctx)?,
//         1 => execute_option_b(ctx)?,
//         _ => return Err(MyError::InvalidChoice.into()),
//     }
//
//     Ok(())
// }
// ```
