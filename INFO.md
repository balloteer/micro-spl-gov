# mpl-gov-micro Project Structure

```
mpl-gov-micro/
├── README.md                    ✅ Complete
├── Anchor.toml                  🔲 Next: Initialize
├── Cargo.toml                   🔲 Next: Initialize
│
├── programs/
│   └── mpl-gov-micro/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs           🔲 Main program logic
│           ├── state/
│           │   ├── mod.rs
│           │   ├── election.rs      🔲 Election account
│           │   ├── voter.rs         🔲 VoterRegistration (compressed)
│           │   └── vote.rs          🔲 VoteRecord (compressed)
│           ├── instructions/
│           │   ├── mod.rs
│           │   ├── create_election.rs
│           │   ├── register_voter.rs
│           │   ├── cast_vote.rs
│           │   └── cast_batch_votes.rs
│           ├── errors.rs
│           └── utils/
│               ├── mod.rs
│               ├── merkle.rs        🔲 Merkle proof verification
│               └── compression.rs   🔲 Compression helpers
│
├── tests/
│   └── mpl-gov-micro.ts         🔲 Integration tests
│
└── client/
    └── sdk/
        ├── index.ts             🔲 TypeScript SDK
        └── types.ts             🔲 Type definitions
```

## Build Order

### Session 1: Project Setup & Core State
1. ✅ README (done)
2. Initialize Anchor project
3. Define Election account (regular)
4. Define VoterRegistration account (compressed)
5. Define basic error types

### Session 2: Election Management
1. Implement create_election instruction
2. Add election validation logic
3. Add election status management
4. Write tests for election creation

### Session 3: Voter Registration
1. Implement register_voter with compression
2. Add merkle tree integration
3. Add attestation verification
4. Write tests for registration

### Session 4: Voting Logic
1. Implement cast_vote instruction
2. Add merkle proof verification
3. Add nullifier checking
4. Write tests for voting

### Session 5: Batch Voting
1. Implement cast_batch_votes
2. Optimize proof verification
3. Add batch validation
4. Write tests for batching

### Session 6: Integration & Testing
1. Complete integration tests
2. Add TypeScript SDK
3. Deploy to devnet
4. Integration with ballo-bot

## Dependencies

```toml
[dependencies]
anchor-lang = "0.30.1"
anchor-spl = "0.30.1"
light-sdk = "0.9.0"
light-hasher = "0.9.0"
light-verifier = "0.9.0"
borsh = "0.10.3"
```