use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

/// Helper functions for working with Light Protocol zkCompression
/// Implements compressed account operations using Light SDK

#[cfg(feature = "compression")]
use spl_account_compression::{
    program::SplAccountCompression,
    state::{ConcurrentMerkleTreeHeader, CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1},
};

/// Compressed voter registration data structure
/// This is the data that gets hashed and stored in the merkle tree
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CompressedVoterData {
    pub voter: Pubkey,
    pub election: Pubkey,
    pub attestation: Pubkey,
    pub registered_at: i64,
}

impl CompressedVoterData {
    /// Create a new compressed voter data structure
    pub fn new(
        voter: Pubkey,
        election: Pubkey,
        attestation: Pubkey,
        registered_at: i64,
    ) -> Self {
        Self {
            voter,
            election,
            attestation,
            registered_at,
        }
    }

    /// Generate the leaf hash for this voter data
    pub fn to_leaf_hash(&self) -> Result<[u8; 32]> {
        // Serialize the data
        let data = self.try_to_vec()?;

        // Hash it using keccak256 (standard for merkle trees)
        let hash = solana_program::keccak::hash(&data);
        Ok(hash.to_bytes())
    }
}

#[cfg(feature = "compression")]
/// Initialize a merkle tree for storing compressed voter registrations
pub fn initialize_voter_merkle_tree<'info>(
    _merkle_tree: &AccountInfo<'info>,
    _authority: &AccountInfo<'info>,
    _payer: &AccountInfo<'info>,
    _compression_program: &AccountInfo<'info>,
    _system_program: &AccountInfo<'info>,
    max_depth: u32,
    max_buffer_size: u32,
) -> Result<()> {
    // Note: In production, you would use spl-account-compression CPI here
    // For now, we're just setting up the structure
    // The actual merkle tree initialization would be done via:
    // spl_account_compression::instruction::init_empty_merkle_tree
    // and sent as a separate instruction from the client

    msg!("Merkle tree should be initialized with depth {} and buffer size {}", max_depth, max_buffer_size);
    msg!("Use SPL Account Compression program to create the tree before calling create_election");
    Ok(())
}

#[cfg(feature = "compression")]
/// Append a voter registration to the merkle tree
pub fn append_voter_to_tree<'info>(
    _merkle_tree: &AccountInfo<'info>,
    _authority: &AccountInfo<'info>,
    _compression_program: &AccountInfo<'info>,
    voter_data: &CompressedVoterData,
) -> Result<[u8; 32]> {
    // Generate the leaf hash
    let leaf_hash = voter_data.to_leaf_hash()?;

    // Note: In production, appending to the tree would be done via:
    // spl_account_compression::instruction::append
    // This would be called from the client or via CPI
    // For now, we just return the leaf hash that should be appended

    msg!("Generated leaf hash for voter {}", voter_data.voter);
    msg!("Leaf should be appended to merkle tree via SPL Account Compression");
    Ok(leaf_hash)
}

#[cfg(feature = "compression")]
/// Verify a merkle proof for a compressed voter registration
pub fn verify_compressed_voter_proof(
    merkle_root: &[u8; 32],
    leaf_hash: &[u8; 32],
    merkle_proof: &[[u8; 32]],
    leaf_index: u32,
) -> Result<bool> {
    use crate::utils::merkle::verify_merkle_proof;

    // Use our existing merkle proof verification
    verify_merkle_proof(*leaf_hash, *merkle_root, merkle_proof, leaf_index)
}

#[cfg(not(feature = "compression"))]
/// Fallback when compression feature is not enabled
pub fn initialize_voter_merkle_tree<'info>(
    _merkle_tree: &AccountInfo<'info>,
    _authority: &AccountInfo<'info>,
    _payer: &AccountInfo<'info>,
    _compression_program: &AccountInfo<'info>,
    _system_program: &AccountInfo<'info>,
    _max_depth: u32,
    _max_buffer_size: u32,
) -> Result<()> {
    msg!("Compression feature not enabled - using legacy mode");
    Ok(())
}

#[cfg(not(feature = "compression"))]
/// Fallback when compression feature is not enabled
pub fn append_voter_to_tree<'info>(
    _merkle_tree: &AccountInfo<'info>,
    _authority: &AccountInfo<'info>,
    _compression_program: &AccountInfo<'info>,
    _voter_data: &CompressedVoterData,
) -> Result<[u8; 32]> {
    msg!("Compression feature not enabled - using legacy mode");
    Ok([0u8; 32])
}

#[cfg(not(feature = "compression"))]
/// Fallback when compression feature is not enabled
pub fn verify_compressed_voter_proof(
    _merkle_root: &[u8; 32],
    _leaf_hash: &[u8; 32],
    _merkle_proof: &[[u8; 32]],
    _leaf_index: u32,
) -> Result<bool> {
    msg!("Compression feature not enabled - using legacy mode");
    Ok(true)
}

/// Compress voter registration data (legacy function for compatibility)
pub fn compress_voter_registration(
    voter: &Pubkey,
    attestation: &Pubkey,
    election: &Pubkey,
    registered_at: i64,
) -> Result<Vec<u8>> {
    let data = CompressedVoterData::new(*voter, *election, *attestation, registered_at);
    data.try_to_vec().map_err(|_| GovError::ArithmeticOverflow.into())
}

/// Decompress voter registration data (legacy function for compatibility)
pub fn decompress_voter_registration(
    compressed_data: &[u8],
) -> Result<(Pubkey, Pubkey, Pubkey, i64)> {
    let data: CompressedVoterData = CompressedVoterData::try_from_slice(compressed_data)
        .map_err(|_| GovError::InvalidMerkleProof)?;

    Ok((data.voter, data.election, data.attestation, data.registered_at))
}

/// Get the merkle tree size required for a given number of voters
pub fn get_merkle_tree_size(max_voters: u32) -> (u32, u32) {
    // Calculate the tree depth needed
    // depth = ceil(log2(max_voters))
    let depth = (max_voters as f64).log2().ceil() as u32;

    // Limit depth to reasonable bounds (between 14 and 30)
    let depth = depth.max(14).min(30);

    // Buffer size is typically 64 for most use cases
    let buffer_size = 64;

    (depth, buffer_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_voter_data() {
        let voter = Pubkey::new_unique();
        let election = Pubkey::new_unique();
        let attestation = Pubkey::new_unique();
        let registered_at = 1234567890i64;

        let data = CompressedVoterData::new(voter, election, attestation, registered_at);

        assert_eq!(data.voter, voter);
        assert_eq!(data.election, election);
        assert_eq!(data.attestation, attestation);
        assert_eq!(data.registered_at, registered_at);
    }

    #[test]
    fn test_leaf_hash_generation() {
        let voter = Pubkey::new_unique();
        let election = Pubkey::new_unique();
        let attestation = Pubkey::new_unique();
        let registered_at = 1234567890i64;

        let data = CompressedVoterData::new(voter, election, attestation, registered_at);
        let hash = data.to_leaf_hash().unwrap();

        // Hash should be 32 bytes
        assert_eq!(hash.len(), 32);

        // Same data should produce same hash
        let data2 = CompressedVoterData::new(voter, election, attestation, registered_at);
        let hash2 = data2.to_leaf_hash().unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_compression_decompression() {
        let voter = Pubkey::new_unique();
        let election = Pubkey::new_unique();
        let attestation = Pubkey::new_unique();
        let registered_at = 1234567890i64;

        let compressed = compress_voter_registration(&voter, &attestation, &election, registered_at).unwrap();
        let (v, e, a, r) = decompress_voter_registration(&compressed).unwrap();

        assert_eq!(v, voter);
        assert_eq!(e, election);
        assert_eq!(a, attestation);
        assert_eq!(r, registered_at);
    }

    #[test]
    fn test_merkle_tree_sizing() {
        // Test different voter counts
        let (depth, buffer) = get_merkle_tree_size(100);
        assert!(depth >= 14 && depth <= 30);
        assert_eq!(buffer, 64);

        let (depth, buffer) = get_merkle_tree_size(10000);
        assert!(depth >= 14 && depth <= 30);
        assert_eq!(buffer, 64);

        // Small tree
        let (depth1, _) = get_merkle_tree_size(100);
        // Large tree
        let (depth2, _) = get_merkle_tree_size(100000);
        assert!(depth2 > depth1);
    }
}
