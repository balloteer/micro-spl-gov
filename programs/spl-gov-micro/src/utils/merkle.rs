use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

/// Verify a merkle proof
pub fn verify_merkle_proof(
    leaf: [u8; 32],
    merkle_root: [u8; 32],
    proof: &[[u8; 32]],
    leaf_index: u32,
) -> Result<bool> {
    let mut computed_hash = leaf;
    let mut index = leaf_index;
    
    for proof_element in proof.iter() {
        if index % 2 == 0 {
            // If index is even, current hash goes on left
            computed_hash = hash_pair(&computed_hash, proof_element);
        } else {
            // If index is odd, current hash goes on right
            computed_hash = hash_pair(proof_element, &computed_hash);
        }
        index /= 2;
    }
    
    Ok(computed_hash == merkle_root)
}

/// Hash two nodes together (left and right)
fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut data = Vec::with_capacity(64);
    data.extend_from_slice(left);
    data.extend_from_slice(right);
    
    let hash_result = hash(&data);
    hash_result.to_bytes()
}

/// Create a leaf hash from voter data
pub fn create_voter_leaf(
    voter: &Pubkey,
    election: &Pubkey,
    attestation: &Pubkey,
) -> [u8; 32] {
    let mut data = Vec::with_capacity(96);
    data.extend_from_slice(voter.as_ref());
    data.extend_from_slice(election.as_ref());
    data.extend_from_slice(attestation.as_ref());
    
    let hash_result = hash(&data);
    hash_result.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle_proof() {
        // TODO: Add merkle proof tests
    }
}
