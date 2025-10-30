pub mod create_election;
pub mod register_voter;
pub mod cast_vote;
pub mod cast_batch_votes;
pub mod close_election;

// Beta features
pub mod privacy_interface;
pub mod hooks;

// Re-export all generated types from Anchor macros
pub use create_election::*;
pub use register_voter::*;
pub use cast_vote::*;
pub use cast_batch_votes::*;
pub use close_election::*;

// Beta feature re-exports
pub use privacy_interface::*;
pub use hooks::*;
