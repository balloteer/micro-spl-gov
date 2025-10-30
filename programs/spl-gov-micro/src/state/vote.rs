use anchor_lang::prelude::*;

/// Vote Record - Compressed account for historical archive
/// Created after election ends for audit trail
/// Using compression keeps historical data cheap
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VoteRecord {
    /// The election this vote was cast in
    pub election: Pubkey,
    
    /// Hash of the voter (anonymous for privacy)
    /// This is derived from voter pubkey + salt
    pub voter_hash: [u8; 32],
    
    /// The choice that was voted for (index into candidates array)
    pub choice: u8,
    
    /// Unix timestamp when vote was cast
    pub timestamp: i64,
    
    /// Optional: Transaction signature for verification
    pub signature: Option<[u8; 64]>,
}

impl VoteRecord {
    /// Size of a vote record
    /// 32 (election) + 32 (voter_hash) + 1 (choice) + 8 (timestamp) + 1 (option tag) + 64 (signature)
    /// = 138 bytes
    pub const SIZE: usize = 138;
    
    /// Size without signature
    /// 32 (election) + 32 (voter_hash) + 1 (choice) + 8 (timestamp) + 1 (option tag)
    /// = 74 bytes
    pub const SIZE_NO_SIG: usize = 74;
}

/// Nullifier to prevent double voting
/// Stored in a regular account for fast lookup
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct VoteNullifier {
    /// Hash that uniquely identifies this vote
    /// Derived from: hash(voter_pubkey, election_pubkey, nonce)
    pub nullifier_hash: [u8; 32],
}

impl VoteNullifier {
    pub const SIZE: usize = 32;
    
    /// Create nullifier from components
    pub fn new(voter: &Pubkey, election: &Pubkey, nonce: u64) -> Self {
        use anchor_lang::solana_program::hash::hash;
        
        let mut data = Vec::new();
        data.extend_from_slice(voter.as_ref());
        data.extend_from_slice(election.as_ref());
        data.extend_from_slice(&nonce.to_le_bytes());
        
        let hash = hash(&data);
        
        Self {
            nullifier_hash: hash.to_bytes(),
        }
    }
}

/// Set of used nullifiers (prevents double voting)
/// This is a regular account for fast checking
#[account]
pub struct NullifierSet {
    /// The election these nullifiers belong to
    pub election: Pubkey,
    
    /// List of used nullifier hashes
    /// For MVP, we use a Vec. For production, consider a Bloom filter
    pub used_nullifiers: Vec<[u8; 32]>,
    
    /// Bump seed for PDA
    pub bump: u8,
}

impl NullifierSet {
    /// Initial size allocation
    /// Will need to be resized as nullifiers are added
    pub const INIT_SIZE: usize = 8 + 32 + 4 + 1;
    
    /// Maximum nullifiers in one account
    /// Limited by account size (10MB)
    pub const MAX_NULLIFIERS: usize = 300_000; // ~10MB / 32 bytes
}
