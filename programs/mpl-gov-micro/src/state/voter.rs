use anchor_lang::prelude::*;

/// Voter Registration - For MVP using regular account
/// In production this will be a compressed account (COLD DATA)
/// This is rarely accessed (only once per vote via merkle proof)
/// Using compression saves 40x cost per registration
#[account]
#[derive(Debug)]
pub struct VoterRegistration {
    /// The wallet address of the registered voter
    pub wallet: Pubkey,

    /// Reference to the attestation from ballo-sns
    /// This proves the voter's eligibility (human, KYC, etc.)
    pub attestation: Pubkey,

    /// The election this voter is registered for
    pub election: Pubkey,

    /// Unix timestamp when voter registered
    pub registered_at: i64,
}

impl VoterRegistration {
    /// Size of a voter registration record
    /// 32 (wallet) + 32 (attestation) + 32 (election) + 8 (registered_at)
    /// = 104 bytes
    pub const SIZE: usize = 104;
}

/// Helper struct for merkle tree operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VoterMerkleProof {
    /// Merkle proof path (up to 20 levels for 1M voters)
    pub proof: Vec<[u8; 32]>,
    
    /// Leaf index in the tree
    pub leaf_index: u32,
}

impl VoterMerkleProof {
    /// Maximum depth of merkle tree
    /// 20 levels = 2^20 = 1,048,576 voters
    pub const MAX_DEPTH: usize = 20;
}
