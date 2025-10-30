use anchor_lang::prelude::*;

#[error_code]
pub enum GovError {
    #[msg("Election has not started yet")]
    ElectionNotStarted,
    
    #[msg("Election has already ended")]
    ElectionEnded,
    
    #[msg("Election is not active")]
    ElectionNotActive,
    
    #[msg("Invalid candidate choice")]
    InvalidChoice,
    
    #[msg("Voter has already voted in this election")]
    AlreadyVoted,
    
    #[msg("Voter is not registered for this election")]
    NotRegistered,
    
    #[msg("Invalid attestation")]
    InvalidAttestation,
    
    #[msg("Attestation has expired")]
    ExpiredAttestation,
    
    #[msg("Attestation does not match voter")]
    AttestationMismatch,
    
    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,
    
    #[msg("Too many candidates (max 10)")]
    TooManyCandidates,
    
    #[msg("Candidate name too long (max 50 chars)")]
    CandidateNameTooLong,
    
    #[msg("Invalid time range (end must be after start)")]
    InvalidTimeRange,
    
    #[msg("Election start time is in the past")]
    StartTimeInPast,
    
    #[msg("Only authority can perform this action")]
    Unauthorized,
    
    #[msg("Nullifier set is full")]
    NullifierSetFull,
    
    #[msg("Invalid nullifier")]
    InvalidNullifier,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Invalid batch vote operation")]
    InvalidBatchVote,
}
