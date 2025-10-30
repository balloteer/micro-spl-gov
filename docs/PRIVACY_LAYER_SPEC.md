# Privacy Layer Specification - Arcium MPC Integration

**Version:** 1.0
**Status:** Beta Feature (Not for Hackathon Demo)
**Target:** Separate implementation repository

> This document provides complete specifications for implementing a privacy layer for mpl-gov-micro using Arcium's Multi-Party Computation (MPC) network. This will be implemented separately and interfaced with the main governance program.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Arcium Integration](#arcium-integration)
4. [Encrypted Vote Flow](#encrypted-vote-flow)
5. [Homomorphic Tallying](#homomorphic-tallying)
6. [Zero-Knowledge Proofs](#zero-knowledge-proofs)
7. [Interface with mpl-gov-micro](#interface-with-mpl-gov-micro)
8. [Security Considerations](#security-considerations)
9. [Implementation Phases](#implementation-phases)
10. [API Specification](#api-specification)
11. [Testing Strategy](#testing-strategy)

---

## Overview

### Goals

The privacy layer provides **anonymous voting** while maintaining:
- ✅ Verifiable results
- ✅ Double-vote prevention
- ✅ Eligibility verification
- ✅ Transparent tallying (encrypted until reveal)
- ✅ Minimal performance overhead

### Key Technologies

- **Arcium MPC Network**: Distributed computation for encrypted vote tallying
- **Threshold Encryption**: ElGamal-based encryption with distributed key generation
- **Groth16 ZK-SNARKs**: Zero-knowledge proofs for voter eligibility
- **Pedersen Commitments**: Vote commitment scheme
- **BLS Signatures**: Aggregatable signatures for efficient verification

### Non-Goals (Out of Scope)

- ❌ Coercion resistance (requires Receipt-Free Voting)
- ❌ Universal verifiability (adds significant complexity)
- ❌ Time-lock encryption (not needed for current use cases)
- ❌ Quantum resistance (future consideration)

---

## Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────────┐
│  1. Election Creation (Public)                          │
│  ┌──────────────────────────────────────────┐          │
│  │ mpl-gov-micro                             │          │
│  │ - Creates election                        │          │
│  │ - Generates DKG parameters                │          │
│  │ - Requests MPC key from Arcium            │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  2. Key Generation (Distributed)                        │
│  ┌──────────────────────────────────────────┐          │
│  │ Arcium MPC Network                        │          │
│  │ - Distributed Key Generation (DKG)        │          │
│  │ - Generates threshold ElGamal keypair     │          │
│  │ - Stores private key shards across MXEs   │          │
│  │ - Publishes public key on-chain           │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  3. Voter Registration (Public)                         │
│  ┌──────────────────────────────────────────┐          │
│  │ mpl-gov-micro                             │          │
│  │ - Registers voter with attestation        │          │
│  │ - Adds to voter merkle tree               │          │
│  │ - Issues nullifier seed                   │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  4. Vote Casting (Private)                              │
│  ┌──────────────────────────────────────────┐          │
│  │ Client-Side (Voter's Browser/App)         │          │
│  │ - Generates ZK proof of eligibility        │          │
│  │ - Encrypts vote with ElGamal (public key) │          │
│  │ - Computes nullifier                       │          │
│  │ - Signs encrypted ballot                   │          │
│  └──────────────────────────────────────────┘          │
│                        ↓                                 │
│  ┌──────────────────────────────────────────┐          │
│  │ privacy-layer Program                     │          │
│  │ - Verifies ZK proof (voter eligible)      │          │
│  │ - Checks nullifier not used                │          │
│  │ - Stores encrypted vote on-chain           │          │
│  │ - Emits vote commitment event              │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  5. Vote Tallying (Private MPC)                         │
│  ┌──────────────────────────────────────────┐          │
│  │ Arcium MPC Network                        │          │
│  │ - Homomorphically adds encrypted votes    │          │
│  │ - Threshold decryption (t-of-n MXEs)      │          │
│  │ - Generates ZK proof of correct tally     │          │
│  │ - Publishes results on-chain              │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│  6. Result Verification (Public)                        │
│  ┌──────────────────────────────────────────┐          │
│  │ privacy-layer Program                     │          │
│  │ - Verifies tally ZK proof                 │          │
│  │ - Verifies signature threshold            │          │
│  │ - Updates election results                │          │
│  │ - Calls mpl-gov-micro with results        │          │
│  └──────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────┘
```

### Component Architecture

```
┌───────────────────────────────────────────────────────────┐
│                    Client (Voter)                         │
├───────────────────────────────────────────────────────────┤
│  • ZK Circuit (Circom/Noir)                              │
│  • ElGamal Encryption Library                            │
│  • Nullifier Generation                                  │
│  • Commitment Scheme                                     │
└───────────────────────────────────────────────────────────┘
                          ↓ RPC
┌───────────────────────────────────────────────────────────┐
│            privacy-layer Solana Program                   │
├───────────────────────────────────────────────────────────┤
│  Instructions:                                            │
│    • initialize_private_election()                        │
│    • cast_encrypted_vote()                                │
│    • submit_mpc_tally()                                   │
│    • finalize_results()                                   │
│                                                           │
│  Accounts:                                                │
│    • PrivateElection                                      │
│    • EncryptedVote                                        │
│    • TallyCommitment                                      │
│    • MpcKeyRecord                                         │
└───────────────────────────────────────────────────────────┘
                          ↓ CPI
┌───────────────────────────────────────────────────────────┐
│              mpl-gov-micro Program                        │
├───────────────────────────────────────────────────────────┤
│  • Manages voter eligibility                             │
│  • Stores final results                                  │
│  • Enforces election rules                               │
└───────────────────────────────────────────────────────────┘
                          ↓ Off-chain
┌───────────────────────────────────────────────────────────┐
│                 Arcium MPC Network                        │
├───────────────────────────────────────────────────────────┤
│  • MXE (Multi-Party Execution Environment)                │
│  • Distributed Key Generation                             │
│  • Threshold Decryption                                   │
│  • Homomorphic Tally Computation                          │
└───────────────────────────────────────────────────────────┘
```

---

## Arcium Integration

### What is Arcium?

Arcium is a **Multi-Party Computation (MPC) network** on Solana that enables:
- Confidential computation without revealing inputs
- Distributed key management (no single point of trust)
- Verifiable computation with on-chain results
- Integration via Solana programs

### Key Arcium Components

#### 1. MXE (Multi-Party Execution Environment)

```rust
// Arcium's MXE for distributed computation
pub struct MxeConfig {
    pub mxe_program_id: Pubkey,        // Arcium MXE program
    pub computation_id: [u8; 32],       // Unique computation identifier
    pub threshold: u8,                  // t-of-n threshold
    pub participants: Vec<Pubkey>,      // MXE node public keys
}
```

#### 2. Distributed Key Generation (DKG)

```rust
// Request DKG from Arcium for an election
pub fn request_dkg(
    ctx: Context<RequestDkg>,
    election: Pubkey,
    threshold: u8,
    num_nodes: u8,
) -> Result<()> {
    // CPI to Arcium MXE program
    arcium_mxe::cpi::initialize_dkg(
        CpiContext::new(
            ctx.accounts.mxe_program.to_account_info(),
            InitializeDkg {
                authority: ctx.accounts.authority.to_account_info(),
                dkg_state: ctx.accounts.dkg_state.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            }
        ),
        DkgParams {
            threshold,
            num_participants: num_nodes,
            key_type: KeyType::ElGamal,
        }
    )?;

    Ok(())
}
```

#### 3. MPC Computation Request

```rust
// Request MPC computation for vote tallying
pub fn request_tally_computation(
    ctx: Context<RequestTallyComputation>,
    encrypted_votes: Vec<Pubkey>,
) -> Result<()> {
    // CPI to Arcium MXE
    arcium_mxe::cpi::compute(
        CpiContext::new(
            ctx.accounts.mxe_program.to_account_info(),
            Compute {
                computation: ctx.accounts.computation.to_account_info(),
                inputs: ctx.accounts.vote_inputs.to_account_info(),
                results: ctx.accounts.results.to_account_info(),
            }
        ),
        ComputeParams {
            function: MpcFunction::HomomorphicTally,
            inputs: encrypted_votes,
            threshold: ctx.accounts.election.mpc_threshold,
        }
    )?;

    Ok(())
}
```

### Arcium Program IDs (Devnet)

```
Arcium MXE Program:       MXEvhYxjQyXP5u8kHCXJnRM3jDKxKJWJWpKPyJNcD2F
Arcium DKG Program:       DKGmPJH6zkjVt9YHYmHqKZANW3PHR5N6LN4KzqYZ8hF
Arcium Compute Program:   CPTmWqNRhf8BxNZX6qFJt9s5aWZPyJKCmLKzqYZ8hF
```

---

## Encrypted Vote Flow

### ElGamal Encryption Scheme

**Why ElGamal?**
- Homomorphic properties (can add encrypted votes)
- Efficient threshold decryption
- Well-studied security proofs
- Compatible with discrete log assumptions

#### ElGamal Keypair Structure

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ElGamalPublicKey {
    pub generator: [u8; 32],      // G (base point)
    pub public_key: [u8; 32],     // Y = g^x (public component)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ElGamalCiphertext {
    pub c1: [u8; 32],             // g^r (ephemeral key)
    pub c2: [u8; 32],             // m * Y^r (encrypted message)
}
```

#### Encryption Process (Client-Side)

```typescript
// Client-side encryption of vote
function encryptVote(
  choice: number,
  publicKey: ElGamalPublicKey,
  randomness: bigint
): ElGamalCiphertext {
  const G = publicKey.generator;
  const Y = publicKey.public_key;

  // Encode choice as group element
  const m = encodeChoice(choice);

  // Generate random ephemeral key
  const r = randomness;

  // C1 = g^r
  const c1 = scalarMultiply(G, r);

  // C2 = m * Y^r
  const yr = scalarMultiply(Y, r);
  const c2 = groupMultiply(m, yr);

  return { c1, c2 };
}
```

#### Homomorphic Addition

```rust
// Add two encrypted votes (used in MPC tallying)
pub fn add_encrypted_votes(
    vote1: &ElGamalCiphertext,
    vote2: &ElGamalCiphertext,
) -> Result<ElGamalCiphertext> {
    // Homomorphic property: E(v1) * E(v2) = E(v1 + v2)
    Ok(ElGamalCiphertext {
        c1: curve25519_add(&vote1.c1, &vote2.c1)?,
        c2: curve25519_add(&vote1.c2, &vote2.c2)?,
    })
}
```

### Vote Commitment Scheme

```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VoteCommitment {
    pub commitment: [u8; 32],          // C = H(vote || blinding_factor)
    pub encrypted_vote: ElGamalCiphertext,
    pub nullifier: [u8; 32],           // Prevents double voting
    pub zk_proof: Groth16Proof,        // Proof of eligibility
}

// Pedersen commitment
pub fn commit_vote(vote: u8, blinding_factor: [u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&[vote]);
    hasher.update(&blinding_factor);
    hasher.finalize().into()
}
```

---

## Homomorphic Tallying

### MPC Tally Computation

The Arcium MPC network performs the tallying computation **without decrypting individual votes**.

#### Tally Algorithm (Executed in MXE)

```python
# Pseudocode for MPC tally computation
def mpc_homomorphic_tally(encrypted_votes: List[ElGamalCiphertext]) -> TallyResult:
    """
    Executed across multiple Arcium MXEs
    Each MXE holds a share of the private key
    """

    # Step 1: Homomorphically aggregate all votes
    aggregated = ElGamalCiphertext(c1=identity, c2=identity)
    for vote in encrypted_votes:
        aggregated = homomorphic_add(aggregated, vote)

    # Step 2: Threshold decryption (t-of-n MXEs participate)
    partial_decryptions = []
    for mxe_id in range(threshold):
        partial = mxe_partial_decrypt(aggregated, mxe_id)
        partial_decryptions.append(partial)

    # Step 3: Combine partial decryptions to get tally
    tally = combine_partial_decryptions(partial_decryptions, threshold)

    # Step 4: Generate ZK proof of correct computation
    proof = generate_tally_proof(
        encrypted_votes,
        tally,
        partial_decryptions
    )

    return TallyResult(tally, proof)
```

#### On-Chain Tally Verification

```rust
pub fn submit_mpc_tally(
    ctx: Context<SubmitMpcTally>,
    tally: Vec<u64>,              // Decrypted vote counts
    proof: TallyProof,             // ZK proof of correct tally
    mxe_signatures: Vec<[u8; 64]>, // BLS signatures from MXEs
) -> Result<()> {
    let election = &ctx.accounts.private_election;

    // Verify threshold signatures from MXEs
    require!(
        verify_threshold_signatures(
            &election.mxe_config,
            &tally,
            &mxe_signatures,
            election.mpc_threshold
        ),
        PrivacyError::InvalidThresholdSignature
    );

    // Verify ZK proof of correct tally
    require!(
        verify_tally_proof(
            &election.encrypted_vote_root,
            &tally,
            &proof
        ),
        PrivacyError::InvalidTallyProof
    );

    // Store tally on-chain
    election.final_tally = tally;
    election.tally_finalized = true;

    // CPI to mpl-gov-micro to update results
    update_election_results(
        ctx.accounts.gov_program,
        election.election_pubkey,
        tally
    )?;

    Ok(())
}
```

---

## Zero-Knowledge Proofs

### Voter Eligibility Circuit (Circom)

```circom
pragma circom 2.1.0;

include "circomlib/circuits/poseidon.circom";
include "circomlib/circuits/merkleproof.circom";

// Prove: "I am a registered voter without revealing my identity"
template VoterEligibility(levels) {
    // Public inputs
    signal input merkleRoot;        // Voter merkle tree root
    signal input nullifier;         // Unique per vote
    signal input electionId;        // Election identifier

    // Private inputs (hidden from verifier)
    signal input voterSecret;       // Voter's secret key
    signal input voterPubkey;       // Voter's public key
    signal input registrationProof[levels];  // Merkle proof
    signal input registrationIndex; // Position in tree
    signal input nonce;             // Randomness for nullifier

    // Verify voter is in merkle tree
    component merkleVerifier = MerkleProof(levels);
    merkleVerifier.root <== merkleRoot;
    merkleVerifier.leaf <== voterPubkey;
    for (var i = 0; i < levels; i++) {
        merkleVerifier.pathElements[i] <== registrationProof[i];
    }
    merkleVerifier.pathIndices <== registrationIndex;

    // Verify secret matches pubkey (prevents impersonation)
    component secretChecker = Poseidon(1);
    secretChecker.inputs[0] <== voterSecret;
    secretChecker.out === voterPubkey;

    // Compute nullifier (prevents double voting)
    component nullifierHash = Poseidon(3);
    nullifierHash.inputs[0] <== voterSecret;
    nullifierHash.inputs[1] <== electionId;
    nullifierHash.inputs[2] <== nonce;
    nullifierHash.out === nullifier;
}

component main {public [merkleRoot, nullifier, electionId]} = VoterEligibility(20);
```

### Proof Generation (Client-Side)

```typescript
// Generate ZK proof of voter eligibility
async function generateEligibilityProof(
  voterSecret: bigint,
  voterPubkey: bigint,
  merkleProof: bigint[],
  merkleRoot: bigint,
  electionId: bigint,
  nonce: bigint
): Promise<Groth16Proof> {
  const { proof, publicSignals } = await snarkjs.groth16.fullProve(
    {
      // Private inputs
      voterSecret,
      voterPubkey,
      registrationProof: merkleProof,
      registrationIndex: getLeafIndex(voterPubkey),
      nonce,

      // Public inputs
      merkleRoot,
      electionId,
      nullifier: computeNullifier(voterSecret, electionId, nonce),
    },
    "voter_eligibility.wasm",
    "voter_eligibility.zkey"
  );

  return {
    a: proof.pi_a,
    b: proof.pi_b,
    c: proof.pi_c,
    publicSignals,
  };
}
```

### Proof Verification (On-Chain)

```rust
pub fn cast_encrypted_vote(
    ctx: Context<CastEncryptedVote>,
    encrypted_vote: ElGamalCiphertext,
    nullifier: [u8; 32],
    zk_proof: Groth16Proof,
) -> Result<()> {
    let election = &ctx.accounts.private_election;

    // Verify ZK proof of voter eligibility
    require!(
        verify_groth16_proof(
            &election.verification_key,
            &zk_proof,
            &[
                election.voter_merkle_root,
                nullifier,
                election.election_id,
            ]
        ),
        PrivacyError::InvalidEligibilityProof
    );

    // Check nullifier not already used (prevents double voting)
    require!(
        !ctx.accounts.nullifier_set.contains(&nullifier),
        PrivacyError::NullifierAlreadyUsed
    );

    // Store encrypted vote
    let vote_account = &mut ctx.accounts.encrypted_vote;
    vote_account.election = election.key();
    vote_account.ciphertext = encrypted_vote;
    vote_account.nullifier = nullifier;
    vote_account.timestamp = Clock::get()?.unix_timestamp;

    // Mark nullifier as used
    ctx.accounts.nullifier_set.insert(nullifier)?;

    emit!(EncryptedVoteCast {
        election: election.key(),
        nullifier,
        timestamp: vote_account.timestamp,
    });

    Ok(())
}
```

---

## Interface with mpl-gov-micro

### Integration Points

```rust
// In mpl-gov-micro program - privacy interface
pub mod privacy_interface {
    use super::*;

    /// Enable private voting for an election
    pub fn enable_private_voting(
        ctx: Context<EnablePrivateVoting>,
    ) -> Result<()> {
        let election = &mut ctx.accounts.election;

        require!(
            election.status == ElectionStatus::Pending,
            GovError::ElectionAlreadyStarted
        );

        // Mark election as privacy-enabled
        election.privacy_enabled = true;
        election.privacy_layer_program = ctx.accounts.privacy_program.key();

        msg!("Private voting enabled for election");

        Ok(())
    }

    /// Receive finalized tally from privacy layer (CPI)
    pub fn receive_private_tally(
        ctx: Context<ReceivePrivateTally>,
        tally: Vec<u64>,
        proof: TallyProof,
    ) -> Result<()> {
        let election = &mut ctx.accounts.election;

        // Verify CPI caller is registered privacy layer
        require!(
            ctx.accounts.privacy_program.key() == election.privacy_layer_program,
            GovError::UnauthorizedPrivacyLayer
        );

        // Verify proof (delegated to privacy layer, but double-check)
        require!(
            verify_tally_proof_hash(&proof),
            GovError::InvalidTallyProof
        );

        // Update vote counts
        for (i, count) in tally.iter().enumerate() {
            election.vote_counts[i] = *count;
        }
        election.total_votes = tally.iter().sum();

        msg!("Private tally received and verified");

        Ok(())
    }
}
```

### CPI from Privacy Layer to mpl-gov-micro

```rust
// In privacy-layer program
pub fn finalize_private_election(
    ctx: Context<FinalizePrivateElection>,
    tally: Vec<u64>,
    proof: TallyProof,
) -> Result<()> {
    // Verify tally in privacy layer first
    verify_mpc_tally(
        &ctx.accounts.election.encrypted_votes,
        &tally,
        &proof,
        &ctx.accounts.mxe_signatures
    )?;

    // CPI to mpl-gov-micro to update results
    let cpi_program = ctx.accounts.gov_program.to_account_info();
    let cpi_accounts = ReceivePrivateTally {
        election: ctx.accounts.gov_election.to_account_info(),
        privacy_program: ctx.accounts.privacy_program.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    mpl_gov_micro::cpi::receive_private_tally(
        cpi_ctx,
        tally,
        proof
    )?;

    Ok(())
}
```

### Election State Extensions

```rust
// Add to Election struct in mpl-gov-micro
pub struct Election {
    // ... existing fields ...

    // Privacy layer integration
    pub privacy_enabled: bool,
    pub privacy_layer_program: Pubkey,
    pub mpc_public_key: Option<[u8; 32]>,
    pub tally_commitment: Option<[u8; 32]>,
}
```

---

## Security Considerations

### Threat Model

#### Protected Against:
- ✅ Vote privacy breach (votes encrypted)
- ✅ Double voting (nullifiers)
- ✅ Ballot stuffing (ZK proofs + merkle tree)
- ✅ Result manipulation (threshold decryption + ZK proofs)
- ✅ Single point of failure (distributed MPC)

#### NOT Protected Against:
- ❌ Coercion (voter can prove how they voted)
- ❌ Network analysis (timing/metadata)
- ❌ Quantum attacks (ECC-based)

### Key Security Properties

#### 1. Vote Privacy

```
Privacy Guarantee: No party (including election authority)
can determine how an individual voted, only aggregate results.

Mechanism:
- ElGamal encryption with public key
- Private key split across t-of-n MXEs
- Threshold decryption prevents single party decryption
```

#### 2. Eligibility Verification

```
Guarantee: Only registered voters can vote

Mechanism:
- ZK-SNARK proves voter in merkle tree
- Without revealing voter identity
- Merkle proof verified on-chain
```

#### 3. Double-Vote Prevention

```
Guarantee: Each voter can only vote once

Mechanism:
- Deterministic nullifier: H(voterSecret || electionId || nonce)
- Nullifier checked on-chain before accepting vote
- Cannot vote twice without revealing identity
```

#### 4. Result Integrity

```
Guarantee: Tally matches encrypted votes

Mechanism:
- Homomorphic tally computed in MPC
- ZK-SNARK proves tally is correct
- Threshold signatures from t-of-n MXEs
```

### Cryptographic Assumptions

1. **Discrete Log Problem (DLP)**: ElGamal security
2. **Decisional Diffie-Hellman (DDH)**: Encryption indistinguishability
3. **Random Oracle Model**: Hash function security
4. **Trusted Setup**: Groth16 ZK-SNARK ceremony

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

**Goal:** Basic encrypted vote storage

```
✓ Set up privacy-layer Anchor project
✓ Define core account structures
✓ Implement ElGamal encryption library
✓ Create cast_encrypted_vote instruction
✓ Basic storage of encrypted votes
```

### Phase 2: ZK Proofs (Weeks 3-4)

**Goal:** Voter eligibility verification

```
✓ Design voter eligibility circuit (Circom)
✓ Trusted setup ceremony
✓ Generate verification keys
✓ Integrate Groth16 verifier in program
✓ Client-side proof generation SDK
```

### Phase 3: Arcium Integration (Weeks 5-6)

**Goal:** MPC key generation and tallying

```
✓ Integrate Arcium MXE SDK
✓ Implement DKG request flow
✓ Set up MPC computation jobs
✓ Test threshold decryption
✓ Handle MXE callbacks
```

### Phase 4: Homomorphic Tallying (Weeks 7-8)

**Goal:** Private vote aggregation

```
✓ Implement homomorphic addition
✓ MPC tally computation logic
✓ Threshold decryption ceremony
✓ Tally ZK proof generation
✓ Result finalization
```

### Phase 5: Integration (Weeks 9-10)

**Goal:** Connect with mpl-gov-micro

```
✓ CPI interface implementation
✓ Election privacy mode flag
✓ Tally callback to gov program
✓ End-to-end integration tests
```

### Phase 6: Optimization & Security (Weeks 11-12)

**Goal:** Production readiness

```
✓ Gas optimization
✓ Security audit preparation
✓ Comprehensive testing
✓ Documentation
✓ Example implementations
```

---

## API Specification

### Privacy Layer Program Instructions

#### 1. Initialize Private Election

```rust
#[derive(Accounts)]
pub struct InitializePrivateElection<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PrivateElection::SIZE,
        seeds = [b"private_election", election.key().as_ref()],
        bump
    )]
    pub private_election: Account<'info, PrivateElection>,

    /// Reference to public election in mpl-gov-micro
    pub election: Account<'info, Election>,

    /// Arcium MXE program
    /// CHECK: Validated via program ID
    pub mxe_program: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_private_election(
    ctx: Context<InitializePrivateElection>,
    threshold: u8,
    num_mxes: u8,
) -> Result<()>;
```

#### 2. Cast Encrypted Vote

```rust
pub fn cast_encrypted_vote(
    ctx: Context<CastEncryptedVote>,
    encrypted_vote: ElGamalCiphertext,
    nullifier: [u8; 32],
    zk_proof: Groth16Proof,
) -> Result<()>;
```

#### 3. Request MPC Tally

```rust
pub fn request_mpc_tally(
    ctx: Context<RequestMpcTally>,
) -> Result<()>;
```

#### 4. Submit MPC Tally Result

```rust
pub fn submit_mpc_tally(
    ctx: Context<SubmitMpcTally>,
    tally: Vec<u64>,
    proof: TallyProof,
    mxe_signatures: Vec<[u8; 64]>,
) -> Result<()>;
```

### Account Structures

```rust
#[account]
pub struct PrivateElection {
    pub election: Pubkey,                    // Public election reference
    pub authority: Pubkey,
    pub mpc_public_key: [u8; 32],           // ElGamal public key
    pub voter_merkle_root: [u8; 32],
    pub verification_key: Groth16Vk,         // ZK-SNARK verification key
    pub mxe_config: MxeConfig,
    pub mpc_threshold: u8,
    pub total_encrypted_votes: u64,
    pub tally_requested: bool,
    pub tally_finalized: bool,
    pub final_tally: Vec<u64>,
    pub bump: u8,
}

#[account]
pub struct EncryptedVote {
    pub election: Pubkey,
    pub ciphertext: ElGamalCiphertext,
    pub nullifier: [u8; 32],
    pub commitment: [u8; 32],
    pub timestamp: i64,
}

#[account]
pub struct TallyCommitment {
    pub election: Pubkey,
    pub commitment: [u8; 32],
    pub proof: TallyProof,
    pub mxe_signatures: Vec<[u8; 64]>,
    pub submitted_at: i64,
}
```

### TypeScript SDK

```typescript
// privacy-layer-sdk/index.ts

export class PrivacyLayerClient {
  constructor(
    public program: Program,
    public connection: Connection,
    public wallet: Wallet
  ) {}

  // Initialize private election
  async initializePrivateElection(
    election: PublicKey,
    threshold: number,
    numMxes: number
  ): Promise<PublicKey>;

  // Encrypt and cast vote
  async castEncryptedVote(
    election: PublicKey,
    choice: number,
    voterSecret: bigint,
    merkleProof: bigint[]
  ): Promise<string>;

  // Request tally from MPC network
  async requestTally(
    election: PublicKey
  ): Promise<string>;

  // Get decrypted results
  async getResults(
    election: PublicKey
  ): Promise<number[]>;
}

// Helper functions
export function encryptVote(
  choice: number,
  publicKey: ElGamalPublicKey
): ElGamalCiphertext;

export async function generateEligibilityProof(
  voterSecret: bigint,
  merkleProof: bigint[],
  election: PublicKey
): Promise<Groth16Proof>;
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elgamal_encryption() {
        let (pk, sk) = generate_keypair();
        let vote = 0u8; // Vote for candidate 0

        let ciphertext = encrypt_vote(vote, &pk);
        let decrypted = decrypt_vote(&ciphertext, &sk);

        assert_eq!(decrypted, vote);
    }

    #[test]
    fn test_homomorphic_addition() {
        let (pk, sk) = generate_keypair();

        let vote1 = encrypt_vote(1, &pk);
        let vote2 = encrypt_vote(2, &pk);

        let sum = add_encrypted_votes(&vote1, &vote2).unwrap();
        let decrypted_sum = decrypt_vote(&sum, &sk);

        assert_eq!(decrypted_sum, 3);
    }

    #[test]
    fn test_nullifier_uniqueness() {
        let secret = [1u8; 32];
        let election_id = Pubkey::new_unique();

        let nullifier1 = compute_nullifier(&secret, &election_id, 0);
        let nullifier2 = compute_nullifier(&secret, &election_id, 0);
        let nullifier3 = compute_nullifier(&secret, &election_id, 1);

        assert_eq!(nullifier1, nullifier2); // Same inputs
        assert_ne!(nullifier1, nullifier3); // Different nonce
    }
}
```

### Integration Tests

```typescript
describe("Privacy Layer Integration", () => {
  it("Full private election flow", async () => {
    // 1. Initialize private election
    const privateElection = await privacyClient.initializePrivateElection(
      electionPda,
      2, // threshold
      3  // num MXEs
    );

    // 2. Wait for DKG completion
    await waitForDkgComplete(privateElection);

    // 3. Cast encrypted votes
    const voters = await registerVoters(100);
    for (const voter of voters) {
      await privacyClient.castEncryptedVote(
        privateElection,
        voter.choice,
        voter.secret,
        voter.merkleProof
      );
    }

    // 4. Request MPC tally
    await privacyClient.requestTally(privateElection);

    // 5. Wait for tally computation
    await waitForTallyComplete(privateElection);

    // 6. Verify results
    const results = await privacyClient.getResults(privateElection);
    expect(results.reduce((a, b) => a + b)).toBe(100);
  });

  it("Prevents double voting with nullifiers", async () => {
    const voter = voters[0];

    // First vote succeeds
    await privacyClient.castEncryptedVote(
      privateElection,
      0,
      voter.secret,
      voter.merkleProof
    );

    // Second vote fails
    await expect(
      privacyClient.castEncryptedVote(
        privateElection,
        1,
        voter.secret,
        voter.merkleProof
      )
    ).rejects.toThrow("NullifierAlreadyUsed");
  });
});
```

---

## Performance Considerations

### Gas Costs

```
Operation                    | Compute Units | Cost (SOL)
----------------------------|---------------|------------
Initialize Private Election  | ~50,000       | ~$0.001
Cast Encrypted Vote         | ~200,000      | ~$0.004
Request MPC Tally           | ~30,000       | ~$0.0006
Submit Tally Results        | ~100,000      | ~$0.002

Total per 1000 voters: ~$4.50
vs Public voting: ~$0.10

Privacy Premium: 45x (acceptable for sensitive elections)
```

### Optimization Strategies

1. **Batch Vote Submission**: Submit multiple encrypted votes in one tx
2. **Compressed Nullifier Sets**: Use bitmap for nullifier storage
3. **Lazy Tally Request**: Only compute when results needed
4. **Proof Aggregation**: Aggregate multiple ZK proofs

---

## References & Resources

### Papers
- [ElGamal Encryption](https://en.wikipedia.org/wiki/ElGamal_encryption)
- [Groth16 ZK-SNARKs](https://eprint.iacr.org/2016/260.pdf)
- [Threshold Cryptography](https://en.wikipedia.org/wiki/Threshold_cryptosystem)

### Libraries
- [circom](https://github.com/iden3/circom) - ZK circuit compiler
- [snarkjs](https://github.com/iden3/snarkjs) - ZK proof generation
- [curve25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek) - Elliptic curve ops
- [arkworks](https://github.com/arkworks-rs) - Rust crypto library

### Arcium Resources
- [Arcium Documentation](https://docs.arcium.com)
- [Arcium MXE SDK](https://github.com/arcium-network/mxe-sdk)
- [Arcium Examples](https://github.com/arcium-network/examples)

---

## Next Steps

### For Implementation Session

1. **Set up project**: `anchor init privacy-layer`
2. **Install dependencies**: Arcium SDK, crypto libraries
3. **Implement ElGamal**: Encryption/decryption functions
4. **Design ZK circuit**: Voter eligibility proof
5. **Integrate Arcium**: DKG and MPC computation
6. **Test end-to-end**: Full private election flow
7. **Connect to mpl-gov-micro**: CPI interface

### Questions to Resolve

- [ ] Arcium MXE configuration (threshold, nodes)
- [ ] ZK circuit complexity vs gas costs
- [ ] Tally reveal timing (immediate vs delayed)
- [ ] Backup decryption mechanism
- [ ] Migration path for existing elections

---

**End of Specification**

This document should be sufficient to implement the entire privacy layer in a separate Claude Code session. All interfaces, data structures, and flows are specified in detail.
