use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::merkle::create_voter_leaf;
use crate::utils::compression::{CompressedVoterData, append_voter_to_tree};

#[derive(Accounts)]
pub struct RegisterVoter<'info> {
    #[account(mut)]
    pub election: Account<'info, Election>,

    /// Voter registration account (only created in legacy mode)
    /// In compression mode, this account is optional and not initialized
    #[account(
        init_if_needed,
        payer = voter,
        space = 8 + VoterRegistration::SIZE,
        seeds = [
            b"voter_registration",
            election.key().as_ref(),
            voter.key().as_ref()
        ],
        bump
    )]
    pub voter_registration: Option<Account<'info, VoterRegistration>>,

    /// Merkle tree for compressed voter registrations (optional, only if compression enabled)
    /// CHECK: Merkle tree account is validated when compression is enabled
    #[account(mut)]
    pub merkle_tree: Option<AccountInfo<'info>>,

    #[account(mut)]
    pub voter: Signer<'info>,

    /// CHECK: Attestation account from ballo-sns (validation deferred to ballo-sns integration)
    pub attestation: UncheckedAccount<'info>,

    /// CHECK: Compression program is validated when used
    pub compression_program: Option<AccountInfo<'info>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterVoter>,
) -> Result<()> {
    let election = &mut ctx.accounts.election;
    let clock = Clock::get()?;

    // TODO: Validate attestation from ballo-sns when integrated
    // For MVP, we accept any attestation account
    // In production:
    // - Verify attestation.subject == voter.key()
    // - Verify attestation.expires_at > clock.unix_timestamp
    // - Verify attestation type is valid for this election

    let voter_key = ctx.accounts.voter.key();
    let attestation_key = ctx.accounts.attestation.key();
    let election_key = election.key();

    if election.use_compression {
        // ===== COMPRESSION MODE =====
        msg!("Registering voter in compression mode");

        // Merkle tree is optional in MVP mode
        // In production, merkle tree would be required for proper compression

        // Create compressed voter data
        let compressed_data = CompressedVoterData::new(
            voter_key,
            election_key,
            attestation_key,
            clock.unix_timestamp,
        );

        // Generate leaf hash
        let leaf_hash = compressed_data.to_leaf_hash()?;

        // In production, the client would append this to the merkle tree
        // via SPL Account Compression program
        // For now, we track the leaf hash
        #[cfg(feature = "compression")]
        {
            if let (Some(merkle_tree), Some(compression_program)) =
                (ctx.accounts.merkle_tree.as_ref(), ctx.accounts.compression_program.as_ref())
            {
                let election_info = election.to_account_info();
                // Append to merkle tree (placeholder - client handles this)
                let _ = append_voter_to_tree(
                    merkle_tree,
                    &election_info,
                    compression_program,
                    &compressed_data,
                )?;
            }
        }

        // Update election's merkle root with the leaf hash
        // In production, this would be updated with the actual tree root after append
        election.voter_merkle_root = leaf_hash;
        election.total_registered = election.total_registered
            .checked_add(1)
            .ok_or(crate::errors::GovError::ArithmeticOverflow)?;

        msg!("Voter registered (compressed): {}", voter_key);
        msg!("Leaf hash: {:?}", leaf_hash);
        msg!("Total registered: {}", election.total_registered);

    } else {
        // ===== LEGACY MODE (Regular Accounts) =====
        msg!("Registering voter in legacy mode");

        // Voter registration account must be provided in legacy mode
        require!(
            ctx.accounts.voter_registration.is_some(),
            crate::errors::GovError::InvalidAttestation
        );

        let voter_registration = ctx.accounts.voter_registration.as_mut().unwrap();

        // Initialize voter registration
        voter_registration.wallet = voter_key;
        voter_registration.attestation = attestation_key;
        voter_registration.election = election_key;
        voter_registration.registered_at = clock.unix_timestamp;

        // Create leaf hash for merkle tree (legacy approach)
        let leaf = create_voter_leaf(
            &voter_registration.wallet,
            &voter_registration.election,
            &voter_registration.attestation,
        );

        // Track the last leaf hash as the root (MVP approach)
        election.voter_merkle_root = leaf;
        election.total_registered = election.total_registered
            .checked_add(1)
            .ok_or(crate::errors::GovError::ArithmeticOverflow)?;

        msg!("Voter registered (legacy): {}", voter_registration.wallet);
        msg!("Attestation: {}", voter_registration.attestation);
        msg!("Total registered: {}", election.total_registered);
    }

    Ok(())
}
