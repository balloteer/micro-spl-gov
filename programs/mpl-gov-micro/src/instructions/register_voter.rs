use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::merkle::create_voter_leaf;

#[derive(Accounts)]
pub struct RegisterVoter<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    #[account(
        init,
        payer = voter,
        space = 8 + VoterRegistration::SIZE,
        seeds = [
            b"voter_registration",
            election.key().as_ref(),
            voter.key().as_ref()
        ],
        bump
    )]
    pub voter_registration: Account<'info, VoterRegistration>,

    #[account(mut)]
    pub voter: Signer<'info>,

    /// CHECK: Attestation account from ballo-sns (validation deferred to ballo-sns integration)
    pub attestation: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterVoter>,
) -> Result<()> {
    let election = &ctx.accounts.election;
    let voter_registration = &mut ctx.accounts.voter_registration;
    let clock = Clock::get()?;

    // TODO: Validate attestation from ballo-sns when integrated
    // For MVP, we accept any attestation account
    // In production:
    // - Verify attestation.subject == voter.key()
    // - Verify attestation.expires_at > clock.unix_timestamp
    // - Verify attestation type is valid for this election

    // Initialize voter registration
    voter_registration.wallet = ctx.accounts.voter.key();
    voter_registration.attestation = ctx.accounts.attestation.key();
    voter_registration.election = election.key();
    voter_registration.registered_at = clock.unix_timestamp;

    // Create leaf hash for merkle tree
    let leaf = create_voter_leaf(
        &voter_registration.wallet,
        &voter_registration.election,
        &voter_registration.attestation,
    );

    // TODO: For MVP, we'll track the last leaf hash as the root
    // In production with zkCompression:
    // - Add leaf to compressed merkle tree
    // - Update election.voter_merkle_root with tree root
    // - Store compressed voter registration
    let election = &mut ctx.accounts.election;
    election.voter_merkle_root = leaf;

    msg!("Voter registered: {}", voter_registration.wallet);
    msg!("Attestation: {}", voter_registration.attestation);

    Ok(())
}
