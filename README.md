# mpl-gov-micro

**Lightweight Governance Protocol with ZK Compression for Solana**

> Part of the Balloteer voting platform - bringing cost-efficient, scalable governance to Telegram communities, DAOs, and enterprises.

---

## 🎯 What We're Building

**mpl-gov-micro** is a simplified governance program built natively with zkCompression for maximum scalability and minimum costs. Unlike traditional DAO tooling (SPL Governance), we focus solely on voting mechanics with strategic use of Light Protocol's zkCompression to achieve **100x cost reduction** while maintaining real-time performance.

### Core Features

- ✅ **Lightweight Elections**: Create elections in seconds
- ✅ **zkCompression Native**: Strategic use for 100x cost savings
- ✅ **Batch Voting**: Up to 50 votes per transaction
- ✅ **Flexible Voting Types**: Daily polls, governance votes, enterprise decisions
- ✅ **Attestation-Gated**: Integrates with ballo-sns for sybil resistance
- ✅ **Real-time Results**: No indexer latency on vote counts
- ✅ **Audit Trail**: Compressed historical records

### Cost Comparison

| Operation | Standard Solana | mpl-gov-micro | Savings |
|-----------|----------------|---------------|---------|
| 10K voter registration | $50 | $0.50 | **100x** |
| 10K individual votes | $10 | $0.10 | **100x** |
| 10K votes (batched 50x) | $10 | $0.50 | **20x** |
| **Total Election** | **$70** | **$1.10** | **64x** |

Our benchmarking demonstrates significant savings:

💰 Legacy Mode (Regular Accounts):
    Voter Registration: ~0.002039 SOL per voter
    1000 voters: ~2.04 SOL

🗜️  Compression Mode (Merkle Tree):
    Voter Registration: ~0.00000500 SOL per voter
    1000 voters: ~0.005000 SOL

💎 Savings:
    Per voter: ~99.8% cheaper
    For 1000 voters: ~2.03 SOL saved
    Cost reduction: ~408x



## 🏗️ Architecture Overview

### The Ballo Stack

```
┌─────────────────────────────────────────┐
│  ballo-bot (Client)                     │
│  Telegram interface with Privy auth     │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  ballo-sns (Attestation)                │
│   KYC & beyond attestations             │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  mpl-gov-micro (Voting) ← YOU ARE HERE  │
│  zkCompression-native governance        │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  ballo-layer (Privacy)                  │
│  ElGamal + ZK proofs for secret ballot  │
└─────────────────────────────────────────┘
```

### Hybrid Data Strategy

**The Key Innovation:** Not everything should be compressed!

```rust
// HOT DATA: Regular Accounts (Fast, frequently updated)
- Election state (vote counts, status)
- Active election metadata
- Real-time tallies

// COLD DATA: Compressed Accounts (Cheap, infrequently accessed)
- Voter registrations (read once per vote)
- Historical votes (archive after election)
- Audit trail (compliance/verification)
```

**Why This Works:**
- Compression has indexer latency (~100ms)
- Vote tallies need instant updates
- Voter lookups can tolerate latency
- Best of both worlds: Speed + Cost

---

## 📊 Strategic Compression Usage

### ✅ USE Compression For

**1. Voter Registrations**
```rust
#[compressed_account]
pub struct VoterRegistration {
    pub wallet: Pubkey,
    pub attestation: Pubkey,
    pub election: Pubkey,
    pub registered_at: i64,
}
// Cost: $0.00005 vs $0.002 (40x cheaper)
// Access: Once per vote via merkle proof
```

**2. Historical Vote Records**
```rust
#[compressed_account]
pub struct ArchivedVote {
    pub voter_hash: [u8; 32],  // Anonymous
    pub choice: u8,
    pub timestamp: i64,
}
// After election: Compress for audit trail
// Saves rent, maintains verifiability
```

**3. Attestation Records** (in ballo-sns)
```rust
#[compressed_account]
pub struct Attestation {
    pub subject: Pubkey,
    pub attestation_type: u8,
    pub expires_at: i64,
}
// 10,000 attestations: $0.50 vs $50
```

### ❌ DON'T Use Compression For

**1. Active Election State**
```rust
#[account]  // Regular account!
pub struct Election {
    pub vote_counts: Vec<u64>,     // Updated every vote
    pub total_votes: u64,           // Real-time access needed
    pub status: ElectionStatus,     // Frequently checked
}
// Compression would cost MORE due to constant updates
```

**2. Frequently-Read Metadata**
```rust
#[account]  // Regular account!
pub struct Election {
    pub candidates: Vec<String>,
    pub start_time: i64,
    pub end_time: i64,
}
// Users query these constantly - keep fast
```

**3. Active Lookups**
- Current election list
- Vote status checks
- Real-time dashboards

### The Math Behind It

```
Compression Tradeoff:
- Save: $0.002 per account
- Cost: $0.0001 per proof verification

Break-even: 20 reads
If data accessed < 20 times: Compress ✅
If data accessed > 20 times: Regular ✅

Voter registrations: ~1 read per election → Compress
Vote counts: 10,000+ reads per election → Regular
```

---

## 🎰 Batch Voting Architecture

### The Magic: Shared Proof Verification

Instead of:
```
Vote 1: Individual proof ($0.0001) + storage ($0.00001)
Vote 2: Individual proof ($0.0001) + storage ($0.00001)
...
Vote 50: Individual proof ($0.0001) + storage ($0.00001)
Total: $0.005
```

We do:
```
Batch: Single proof ($0.0001) + 50 storages ($0.0005)
Total: $0.0006
Savings: 8.3x
```

### Implementation Strategy

```rust
pub fn cast_batch_votes(
    ctx: Context<CastBatchVotes>,
    votes: Vec<VoteInput>,
) -> Result<()> {
    // Single proof verifies ALL votes
    verify_batch_proof(&ctx.accounts.proof)?;
    
    for vote in votes {
        // Verify voter registered (merkle proof from compressed tree)
        verify_voter_registered(&vote.voter, &vote.merkle_proof)?;
        
        // Check nullifier not used (prevents double voting)
        require!(!is_nullifier_used(&vote.nullifier)?, AlreadyVoted);
        
        // Update counts in regular account (instant)
        election.vote_counts[vote.choice] += 1;
        
        // Mark nullifier used (prevent replay)
        mark_nullifier_used(vote.nullifier)?;
    }
    
    Ok(())
}
```

**Use Cases:**
- Multi-question surveys: Vote on 10 questions at once
- Cross-DAO voting: Vote in 5 DAOs simultaneously  
- Daily batching: Collect votes, submit every hour
- Enterprise: Proxy voting for shareholders

---

## 🗂️ Program Structure

### Account Types

```
Election (Regular)
├── authority: Pubkey
├── candidates: Vec<String>
├── vote_counts: Vec<u64>        ← HOT: Updated every vote
├── total_votes: u64              ← HOT: Real-time access
├── voter_merkle_root: [u8; 32]  ← Just the root (cheap)
├── start_time: i64
├── end_time: i64
└── status: ElectionStatus

VoterRegistration (Compressed)
├── wallet: Pubkey
├── attestation: Pubkey           ← Links to ballo-sns
├── election: Pubkey
└── registered_at: i64

VoteRecord (Compressed - Archive Only)
├── election: Pubkey
├── voter_hash: [u8; 32]         ← Anonymous
├── choice: u8
└── timestamp: i64

NullifierSet (Regular)
├── election: Pubkey
└── used_nullifiers: HashSet<[u8; 32]>  ← Prevents double voting
```

### Instructions

```rust
// Election Management
create_election(candidates, start_time, end_time)
update_election(new_end_time)
close_election()

// Voter Registration
register_voter(attestation)           // Creates compressed record
batch_register_voters(voters)         // Bulk registration

// Voting
cast_vote(choice, merkle_proof)
cast_batch_votes(votes)               // Up to 50 votes
cast_anonymous_vote(encrypted, proof) // Future: ballo-layer

// Queries
get_results(election)
is_voter_registered(wallet, election)
has_voted(wallet, election)

// Archive (Post-Election)
archive_votes()                       // Compress vote history
```

---

## 🔐 Integration with ballo-sns

### Attestation Flow

```rust
// In register_voter instruction
pub fn register_voter(
    ctx: Context<RegisterVoter>,
) -> Result<()> {
    // 1. Verify attestation exists and valid
    let attestation = &ctx.accounts.attestation;
    require!(
        attestation.subject == ctx.accounts.voter.key(),
        InvalidAttestation
    );
    require!(
        attestation.expires_at > Clock::get()?.unix_timestamp,
        ExpiredAttestation
    );
    
    // 2. Create COMPRESSED voter registration
    let voter_reg = VoterRegistration {
        wallet: ctx.accounts.voter.key(),
        attestation: attestation.key(),
        election: ctx.accounts.election.key(),
        registered_at: Clock::get()?.unix_timestamp,
    };
    
    // 3. Add to compressed tree
    ctx.accounts.voter_tree.append(&voter_reg)?;
    
    // 4. Update merkle root (in regular account)
    ctx.accounts.election.voter_merkle_root = 
        ctx.accounts.voter_tree.root();
    
    Ok(())
}
```

### Attestation Types (from ballo-sns)

```rust
pub enum AttestationType {
    Human = 1,      // Basic sybil resistance
    MockKYC = 2,    // For hackathon demo
    RealKYC = 3,    // Future: Real identity verification
    Accredited = 4, // For enterprise/securities voting
}
```

---

## 🚀 Implementation Phases

### Phase 1: Core Voting (Hackathon MVP)

**Goal:** Working voting with compression

```
✅ Election creation
✅ Voter registration with compression
✅ Vote casting with merkle proofs
✅ Real-time tally in regular accounts
✅ Basic batch voting (5-10 votes)
✅ Attestation integration
✅ Results queries
```

**Not Included:**
- Privacy layer (pitch only)
- Advanced batching (50+ votes)
- Historical archival
- Multi-sig authority

### Phase 2: Optimization (Post-Hackathon)

```
⏳ Advanced batch voting (up to 50)
⏳ Optimized merkle proofs
⏳ Historical vote compression
⏳ Multiple election types
⏳ Delegation support
```

### Phase 3: Privacy Layer (Future)

```
⏳ ElGamal encryption integration
⏳ ZK proof circuits
⏳ Anonymous voting
⏳ Homomorphic tallying
⏳ Full ballo-layer integration
```

---

## 🛠️ Technical Stack

### Dependencies

```toml
[dependencies]
anchor-lang = "0.30.1"
anchor-spl = "0.30.1"
light-sdk = "0.9.0"           # zkCompression
light-hasher = "0.9.0"         # Merkle trees
light-verifier = "0.9.0"       # Proof verification
```

### Light Protocol Integration

**Key Components:**
- `light-sdk`: Compressed account macros and helpers
- `light-system-program`: Core compression logic
- `light-compressed-token`: Token compression (future use)
- Photon Indexer: Query compressed state

**Program IDs:**
```
Light System Program: SySTEM1eSU2p4BGQfQpimFEWWSC1XDFeun3Nqzz3rT7
Compressed Token: cTokenmWW8bLPjZEBAUgYy3zKxQZW6VKi7bqNFEVv3m
```

---

## 📐 Data Structure Sizes

### Storage Costs

```rust
// Election (Regular Account)
pub struct Election {
    // 8 (discriminator)
    // 32 (authority)
    // 4 + (10 * 50) = 504 (candidates, max 10 @ 50 chars)
    // 4 + (10 * 8) = 84 (vote_counts, max 10 candidates)
    // 8 (total_votes)
    // 32 (voter_merkle_root)
    // 8 (start_time)
    // 8 (end_time)
    // 1 (status)
    // = ~685 bytes
    // Cost: ~$0.005 (one-time)
}

// VoterRegistration (Compressed Account)
pub struct VoterRegistration {
    // 32 (wallet)
    // 32 (attestation)
    // 32 (election)
    // 8 (registered_at)
    // = 104 bytes
    // Cost: ~$0.00005 with compression
    // vs $0.002 without compression
}

// VoteRecord (Compressed - Archive)
pub struct VoteRecord {
    // 32 (election)
    // 32 (voter_hash)
    // 1 (choice)
    // 8 (timestamp)
    // = 73 bytes
    // Cost: ~$0.00003 with compression
}
```

---

## 🎯 Use Cases

### 1. Daily Polls (Telegram Communities)

```
Scenario: 1,000 member community votes daily
- Registration: $0.05 (one-time)
- Daily vote: $0.001
- Monthly cost: $0.035
vs Standard: $2.50/month
```

### 2. DAO Governance

```
Scenario: 50,000 token holders vote quarterly
- Registration: $2.50
- Quarterly vote: $0.50
- Annual cost: $4.50
vs Standard: $300/year
```

### 3. Enterprise Voting (Shareholder)

```
Scenario: 10,000 shareholders vote on proposals
- Registration: $0.50
- Per proposal: $0.10
- 10 proposals/year: $1.50
vs Standard: $150/year
```

### 4. Multi-Question Surveys

```
Scenario: 5,000 users, 20 questions
- Batch voting: 5,000 batches × $0.0001 = $0.50
- Individual voting: 100,000 votes × $0.001 = $100
Savings: 200x
```

---

## 🔒 Security Considerations

### Double Voting Prevention

```rust
// Nullifier-based approach
pub struct NullifierSet {
    election: Pubkey,
    used_nullifiers: HashSet<[u8; 32]>,
}

// Each vote generates unique nullifier
let nullifier = hash([voter.key(), election.key(), nonce]);

// Check not already used
require!(!nullifier_set.contains(&nullifier), AlreadyVoted);

// Mark as used
nullifier_set.insert(nullifier);
```

### Attestation Verification

```rust
// Always verify attestation on registration
require!(
    verify_attestation(&attestation_account)?,
    InvalidAttestation
);

// Check not expired
require!(
    attestation.expires_at > current_time,
    ExpiredAttestation
);

// Check matches voter
require!(
    attestation.subject == voter.key(),
    AttestationMismatch
);
```

### Merkle Proof Validation

```rust
// Verify voter in compressed tree
pub fn verify_voter_registered(
    voter: Pubkey,
    merkle_root: [u8; 32],
    proof: Vec<[u8; 32]>,
) -> Result<bool> {
    let leaf = hash(voter.as_ref());
    let computed_root = compute_merkle_root(leaf, proof);
    Ok(computed_root == merkle_root)
}
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Election lifecycle
    test_create_election()
    test_invalid_election_params()
    
    // Registration
    test_register_voter()
    test_register_without_attestation()
    test_register_duplicate()
    
    // Voting
    test_cast_vote()
    test_double_vote_prevention()
    test_invalid_choice()
    test_vote_outside_window()
    
    // Batch voting
    test_batch_votes()
    test_batch_with_invalid_vote()
    
    // Compression
    test_compressed_voter_lookup()
    test_merkle_proof_validation()
}
```

### Integration Tests

```typescript
describe("mpl-gov-micro", () => {
  it("Full election flow", async () => {
    // 1. Create election
    const election = await createElection();
    
    // 2. Register 100 voters
    const voters = await registerVoters(100);
    
    // 3. Cast votes
    await castVotes(voters);
    
    // 4. Verify results
    const results = await getResults(election);
    expect(results.total_votes).toBe(100);
  });
  
  it("Batch voting", async () => {
    // Vote in 10 elections at once
    const votes = generateBatchVotes(10);
    await castBatchVotes(votes);
  });
});
```

---

## 📊 Performance Benchmarks

### Expected Performance

| Operation | Throughput | Latency |
|-----------|-----------|---------|
| Register voter | 100/sec | <2s |
| Cast vote | 200/sec | <1s |
| Batch vote (50x) | 10,000 votes/sec | <2s |
| Query results | Instant | <100ms |
| Merkle verification | - | <50ms |

### Scalability Limits

```
Theoretical Maximum:
- Voters per election: Unlimited (compressed)
- Votes per transaction: ~50 (compute limit)
- Concurrent elections: Unlimited
- Historical elections: Unlimited (compressed archive)

Practical Constraints:
- Merkle tree depth: 20 levels = 1M voters
- Batch size: 50 votes (transaction size limit)
- Indexer capacity: ~1000 queries/sec
```

---

## 🔗 Integration Points

### For ballo-bot (Client)

```typescript
import { BalloGovernance } from '@ballo/mpl-gov-micro';

// Initialize
const gov = new BalloGovernance(connection, programId);

// Create election from Telegram
await gov.createElection({
  candidates: ['Alice', 'Bob', 'Charlie'],
  startTime: Date.now(),
  endTime: Date.now() + 86400000, // 24 hours
});

// Register voter (after Privy auth)
await gov.registerVoter(walletPubkey, attestationPubkey);

// User votes through Telegram
await gov.castVote(choice);

// Show results in Telegram
const results = await gov.getResults(electionPubkey);
```

### For ballo-sns (Attestation)

```rust
// ballo-sns provides attestations
// mpl-gov-micro consumes them

pub fn register_voter(
    ctx: Context<RegisterVoter>,
) -> Result<()> {
    // Read attestation from ballo-sns
    let attestation = Account::<Attestation>::try_from(
        &ctx.accounts.attestation
    )?;
    
    // Verify and proceed
    verify_attestation(&attestation)?;
    // ...
}
```

### For ballo-layer (Privacy)

```rust
// Future: Encrypted voting
pub fn cast_anonymous_vote(
    ctx: Context<CastAnonymousVote>,
    encrypted_vote: ElGamalCiphertext,
    zk_proof: Groth16Proof,
) -> Result<()> {
    // Verify ZK proof from ballo-layer
    verify_zkproof(&zk_proof)?;
    
    // Store encrypted vote
    store_encrypted_vote(encrypted_vote)?;
    
    Ok(())
}
```

---

## 🎨 Design Principles

### 1. Compression is Strategic, Not Universal
- Compress cold data (registrations, archives)
- Regular accounts for hot data (tallies, status)
- Measure access patterns before compressing

### 2. Optimize for the Common Case
- Most elections: <10K voters
- Most votes: Single choice
- Most usage: View results frequently

### 3. Pay for What You Use
- No rent for inactive elections (compression)
- Minimal storage for active elections
- Archive old data automatically

### 4. Developer Experience First
- Simple API for common operations
- Advanced features optional
- Clear error messages
- Extensive documentation

### 5. Composability
- Works with any attestation system
- Compatible with any wallet
- Can integrate with existing DAOs
- Privacy layer is additive

---

## 📦 TypeScript SDK

We provide a comprehensive TypeScript SDK for working with compression features. See [sdk/README.md](./sdk/README.md) for full documentation.

### Quick SDK Usage

```typescript
import {
  createCompressedElection,
  registerCompressedVoter,
  castCompressedVote,
  calculateCompressionSavings
} from "./sdk";

// Create compressed election
await createCompressedElection(
  program,
  authority,
  ["Alice", "Bob"],
  startTime,
  endTime,
  10000 // max voters
);

// Register voter (no account created!)
await registerCompressedVoter(
  program,
  electionPda,
  voter,
  attestation.publicKey
);

// Cast vote with merkle proof
await castCompressedVote(
  program,
  electionPda,
  voter,
  attestation.publicKey,
  0, // choice
  0, // leafIndex
  registeredAt,
  [] // merkleProof (empty for MVP)
);

// Calculate savings
const savings = calculateCompressionSavings(10000);
console.log(`Cost reduction: ${savings.savings.costReduction}x`);
// Output: Cost reduction: 408x
```

### SDK Features

- ✅ **Compression helpers** - Create leaf hashes matching Rust implementation
- ✅ **PDA derivation** - Election, VoterRegistration, NullifierSet PDAs
- ✅ **Merkle tree utilities** - SimpleMerkleTree class for testing
- ✅ **Instruction builders** - Easy-to-use wrappers for all instructions
- ✅ **Cost calculator** - Compare compression vs legacy costs
- ✅ **Examples** - Complete compressed election workflow

See the [SDK documentation](./sdk/README.md) and [examples](./sdk/examples/) for more details.

---

## 🚦 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Install Light Protocol CLI
npm install -g @lightprotocol/cli
```

### Build & Deploy

```bash
# Build program
anchor build

# Get program ID
anchor keys list

# Update program ID in lib.rs and Anchor.toml

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run tests
anchor test
```

---

## 📝 Next Steps

### Implementation Status

1. ✅ **Set up project structure**
2. ✅ **Define account structures** (Election, VoterRegistration, NullifierSet)
3. ✅ **Implement election creation** (with compression mode support)
4. ✅ **Implement voter registration with compression** (MVP mode with merkle trees)
5. ✅ **Implement vote casting with merkle proofs** (compression + nullifier verification)
6. ✅ **Add batch voting support** (cast_batch_votes instruction)
7. ✅ **Add attestation verification** (ballo-sns integration hooks)
8. ✅ **Write comprehensive tests** (19 tests passing - legacy + compression modes)
9. ✅ **Create TypeScript SDK** (Helper functions and examples)
10. 🔲 Deploy to devnet
11. 🔲 Integrate with ballo-bot

### Future Enhancements

- Multi-choice voting (rank, approval, quadratic)
- Delegation system
- Time-weighted voting
- Cross-program invocations
- Privacy layer integration
- Governance token staking
- Treasury management (light version)

---

## 📚 Resources

### Documentation
- Light Protocol: https://www.zkcompression.com
- Anchor Framework: https://www.anchor-lang.com
- Solana Docs: https://docs.solana.com

### Code Examples
- Light Protocol Examples: https://github.com/Lightprotocol/light-protocol/tree/main/examples
- Anchor Examples: https://github.com/coral-xyz/anchor/tree/master/examples

### Community
- Light Protocol Discord: https://discord.gg/lightprotocol
- Anchor Discord: https://discord.gg/PDeRXyVURd
- Solana Discord: https://discord.gg/solana

---

## 📄 License

MIT License - Built for the Balloteer platform
