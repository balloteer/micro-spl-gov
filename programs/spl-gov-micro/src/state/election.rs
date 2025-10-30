use anchor_lang::prelude::*;

/// Election account - Regular Solana account (not compressed)
/// This is HOT DATA that needs fast, frequent access
#[account]
#[derive(Debug)]
pub struct Election {
    /// Authority that can manage the election
    pub authority: Pubkey,

    /// List of candidates/options (max 10 candidates, 50 chars each)
    pub candidates: Vec<String>,

    /// Vote counts for each candidate (parallel to candidates array)
    pub vote_counts: Vec<u64>,

    /// Total number of votes cast
    pub total_votes: u64,

    /// Merkle root of registered voters (compressed tree)
    pub voter_merkle_root: [u8; 32],

    /// Unix timestamp when voting starts
    pub start_time: i64,

    /// Unix timestamp when voting ends
    pub end_time: i64,

    /// Current status of the election
    pub status: ElectionStatus,

    /// Whether voter registrations use zkCompression
    pub use_compression: bool,

    /// Merkle tree account for compressed voter registrations (if compression enabled)
    /// Set to default Pubkey if compression is disabled
    pub merkle_tree: Pubkey,

    /// Total number of registered voters
    pub total_registered: u64,

    /// Bump seed for PDA
    pub bump: u8,
}

impl Election {
    /// Calculate space needed for Election account
    /// 8 (discriminator)
    /// + 32 (authority)
    /// + 4 (vec len) + (10 * (4 + 50)) (candidates: max 10 @ 50 chars)
    /// + 4 (vec len) + (10 * 8) (vote_counts: max 10 u64s)
    /// + 8 (total_votes)
    /// + 32 (voter_merkle_root)
    /// + 8 (start_time)
    /// + 8 (end_time)
    /// + 1 (status)
    /// + 1 (use_compression)
    /// + 32 (merkle_tree)
    /// + 8 (total_registered)
    /// + 1 (bump)
    /// = 8 + 32 + 544 + 84 + 8 + 32 + 8 + 8 + 1 + 1 + 32 + 8 + 1 = 767 bytes
    pub const MAX_SIZE: usize = 767;

    /// Maximum number of candidates allowed
    pub const MAX_CANDIDATES: usize = 10;

    /// Maximum length of candidate name
    pub const MAX_CANDIDATE_NAME_LEN: usize = 50;
}

/// Status of an election
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElectionStatus {
    /// Election has been created but not yet started
    Pending,
    /// Election is currently active and accepting votes
    Active,
    /// Election has ended, no more votes accepted
    Ended,
    /// Election has been cancelled by authority
    Cancelled,
}

impl Default for ElectionStatus {
    fn default() -> Self {
        ElectionStatus::Pending
    }
}
