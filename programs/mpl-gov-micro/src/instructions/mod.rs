pub mod create_election;
pub mod register_voter;
pub mod cast_vote;
pub mod cast_batch_votes;
pub mod close_election;

// Re-export all generated types from Anchor macros
pub use create_election::*;
pub use register_voter::*;
pub use cast_vote::*;
pub use cast_batch_votes::*;
pub use close_election::*;
