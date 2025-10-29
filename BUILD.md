# Build Instructions for mpl-gov-micro

## 📁 Project Structure Created

```
ballo/
├── README.md                          ✅ Overview and architecture
├── INFO.md                            ✅ Your existing file
├── Cargo.toml                         ✅ Workspace configuration
├── Anchor.toml                        ✅ Anchor configuration
├── package.json                       ✅ TypeScript dependencies
├── tsconfig.json                      ✅ TypeScript configuration
├── .gitignore                         ✅ Git ignore rules
│
├── programs/
│   └── mpl-gov-micro/
│       ├── Cargo.toml                 ✅ Program dependencies
│       └── src/
│           ├── lib.rs                 ✅ Main program entry
│           ├── errors.rs              ✅ Error definitions
│           ├── state/
│           │   ├── mod.rs             ✅ State module exports
│           │   ├── election.rs        ✅ Election account (regular)
│           │   ├── voter.rs           ✅ VoterRegistration (compressed)
│           │   └── vote.rs            ✅ VoteRecord & Nullifiers
│           ├── instructions/
│           │   ├── mod.rs             ✅ Instruction exports
│           │   ├── create_election.rs ✅ Create election logic
│           │   ├── register_voter.rs  ✅ Register with compression
│           │   ├── cast_vote.rs       ✅ Cast vote logic
│           │   ├── cast_batch_votes.rs✅ Batch voting
│           │   └── close_election.rs  ✅ Close election
│           └── utils/
│               ├── mod.rs             ✅ Utility exports
│               ├── merkle.rs          ✅ Merkle proof verification
│               └── compression.rs     ✅ Compression helpers
│
└── tests/
    └── mpl-gov-micro.ts               ✅ Integration tests
```

## 🎯 What We Built

### Account Structures (state/)

**Election** (Regular Account - HOT DATA)
- Stores vote counts and active metadata
- Updated frequently, needs fast access
- ~726 bytes, costs ~$0.005 per election

**VoterRegistration** (Compressed - COLD DATA)
- Stores voter eligibility
- Accessed once per vote via merkle proof
- ~104 bytes, costs ~$0.00005 per voter (40x cheaper!)

**VoteRecord** (Compressed - ARCHIVE)
- Historical vote records for audit
- Only created post-election
- ~74-138 bytes, keeps history cheap

**NullifierSet** (Regular Account)
- Prevents double voting
- Fast lookup needed
- Grows with each vote

### Instructions (instructions/)

All instruction handlers are structured with placeholders:

1. **create_election** - Initialize new election
2. **register_voter** - Register with zkCompression
3. **cast_vote** - Vote with merkle proof
4. **cast_batch_votes** - Multiple votes in one tx
5. **close_election** - End voting period

### Utilities (utils/)

- **merkle.rs** - Merkle proof verification functions
- **compression.rs** - zkCompression helpers (Light SDK)

### Errors (errors.rs)

Comprehensive error types for:
- Election lifecycle
- Voter registration
- Vote validation
- Attestation verification
- Merkle proofs

## 🚀 Building the Program

### Prerequisites

```bash
# Check installations
rustc --version    # Should be 1.70+
solana --version   # Should be 1.18+
anchor --version   # Should be 0.30+
```

### Step 1: Install Dependencies

```bash
cd ballo

# Install Rust dependencies (automatic with anchor build)
# Install Node dependencies
npm install
# or
yarn install
```

### Step 2: Build the Program

```bash
# Build the Anchor program
anchor build

# This will:
# - Compile the Rust program
# - Generate the IDL
# - Create program keypair
```

### Step 3: Get Program ID

```bash
# Get the program ID from the build
anchor keys list

# Copy the program ID and update in:
# 1. programs/mpl-gov-micro/src/lib.rs (declare_id! macro)
# 2. Anchor.toml (under [programs.devnet])
```

### Step 4: Build Again (with correct ID)

```bash
# Rebuild with correct program ID
anchor build
```

## 🧪 Testing

```bash
# Run tests (will start local validator)
anchor test

# Run tests without rebuilding
anchor test --skip-build

# Run specific test
anchor test --skip-build -- --grep "Creates an election"
```

## 🚢 Deployment

### Deploy to Devnet

```bash
# Set Solana to devnet
solana config set --url devnet

# Airdrop some SOL for deployment (if needed)
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet

# Your program is now live on devnet!
```

### Deploy to Mainnet

```bash
# Switch to mainnet
solana config set --url mainnet-beta

# IMPORTANT: Make sure you have enough SOL for deployment (~5 SOL)

# Deploy
anchor deploy --provider.cluster mainnet-beta
```

## 📊 What's Implemented vs TODO

### ✅ Implemented (Structure Ready)

- [x] Project structure
- [x] Account definitions
- [x] Error types
- [x] Instruction signatures
- [x] Merkle proof utilities
- [x] Basic configuration

### 🔲 TODO (Next Prompts)

- [ ] Implement create_election logic
- [ ] Implement register_voter with compression
- [ ] Implement cast_vote with merkle verification
- [ ] Implement batch voting
- [ ] Add Light SDK integration
- [ ] Write comprehensive tests
- [ ] Add attestation verification (ballo-sns)
- [ ] Add privacy layer hooks (ballo-layer)

## 🎨 Development Workflow

### Prompt-by-Prompt Build Plan

**Session 1:** ✅ COMPLETE
- Project structure
- Account definitions
- Instruction placeholders

**Session 2:** NEXT
- Implement create_election
- Add validation logic
- Write tests for election creation

**Session 3:**
- Implement register_voter
- Integrate Light SDK
- Add merkle tree management

**Session 4:**
- Implement cast_vote
- Add nullifier checking
- Merkle proof verification

**Session 5:**
- Implement batch voting
- Optimize proof verification
- Complete testing suite

**Session 6:**
- Integration testing
- Deploy to devnet
- Connect with ballo-bot

## 🔧 Common Commands

```bash
# Build
anchor build

# Test
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Generate TypeScript types
anchor build

# Clean build artifacts
anchor clean

# Run local validator (separate terminal)
solana-test-validator

# Check account info
solana account <PROGRAM_ID>

# Check program logs
solana logs <PROGRAM_ID>
```

## 📝 Notes for Development

### When Adding New Instructions

1. Create instruction file in `src/instructions/`
2. Define `Context` struct with accounts
3. Implement `handler` function
4. Export in `instructions/mod.rs`
5. Add to `#[program]` in `lib.rs`
6. Write tests in `tests/`

### When Adding State

1. Define struct in appropriate file in `state/`
2. Add size constants
3. Export in `state/mod.rs`
4. Use in instruction contexts

### zkCompression Notes

- Compression will be added when implementing `register_voter`
- Uses Light Protocol SDK
- Requires Light System Program in Anchor.toml (already added)
- Photon indexer for querying compressed state

## 🐛 Troubleshooting

**Error: Program not found**
- Run `anchor build` to compile
- Check program ID matches in lib.rs and Anchor.toml

**Error: Account not initialized**
- Make sure you're calling create_election before other operations
- Check account seeds match

**Error: Insufficient funds**
- Airdrop SOL: `solana airdrop 2`
- Check balance: `solana balance`

**Build errors**
- Run `anchor clean` then `anchor build`
- Check Rust version: `rustc --version`

## 🔗 Integration Points

### For ballo-bot

Will expose these functions via TypeScript SDK:
```typescript
- createElection(candidates, startTime, endTime)
- registerVoter(wallet, attestation)
- castVote(choice)
- getResults(election)
```

### For ballo-sns

Expects attestation accounts with:
```rust
- subject: Pubkey
- attestation_type: u8
- expires_at: i64
```

### For ballo-layer (Future)

Will add hooks for:
```rust
- castAnonymousVote(encrypted, proof)
- tallyHomomorphically()
```

## 📚 Next Steps

**Ready to continue building?**

Next prompt: "Let's implement the create_election instruction"

This will:
1. Add validation logic
2. Initialize election account
3. Set up nullifier set
4. Write tests

---

**Project Status:** 🟢 Structure Complete, Ready for Implementation

**Next Session:** Implement create_election instruction
