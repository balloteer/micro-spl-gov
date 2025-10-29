# mpl-gov-micro Operations Guide

Complete guide for building, testing, deploying, and operating the mpl-gov-micro governance program.

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Initial Setup](#initial-setup)
3. [Building](#building)
4. [Testing](#testing)
5. [Deployment](#deployment)
6. [Program Operations](#program-operations)
7. [Monitoring & Debugging](#monitoring--debugging)
8. [Troubleshooting](#troubleshooting)
9. [Maintenance](#maintenance)

---

## Prerequisites

### Required Tools

```bash
# Check installations
rustc --version    # Should be 1.70+
cargo --version
solana --version   # Should be 1.18+
anchor --version   # Should be 0.30+
node --version     # Should be 18+
```

### Install Rust (if needed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update
```

### Install Solana (if needed)

```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version
```

### Install Anchor (if needed)

```bash
# Install avm (Anchor Version Manager)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# Install latest Anchor
avm install latest
avm use latest

# Verify
anchor --version
```

### Install Node Dependencies

```bash
cd ballo
npm install
# or
yarn install
```

---

## Initial Setup

### 1. Configure Solana CLI

```bash
# Set to devnet for development
solana config set --url devnet

# Or use localnet for testing
solana config set --url localhost

# Create a wallet (if you don't have one)
solana-keygen new --outfile ~/.config/solana/id.json

# Check your configuration
solana config get
```

### 2. Get Devnet SOL

```bash
# Airdrop SOL for testing
solana airdrop 2

# Check balance
solana balance
```

### 3. Clone and Navigate

```bash
cd ballo
ls -la
```

---

## Building

### Quick Build

```bash
# Clean build from scratch
anchor clean
anchor build

# This will:
# - Compile the Rust program
# - Generate the IDL (Interface Definition Language)
# - Create program keypair
# - Generate TypeScript types
```

### Update Program ID

After the first build, you need to update the program ID:

```bash
# 1. Get the program ID
anchor keys list

# Output will show:
# mpl_gov_micro: <PROGRAM_ID>

# 2. Update in lib.rs
# Edit programs/mpl-gov-micro/src/lib.rs
# Replace the ID in declare_id!("...") with your program ID

# 3. Update in Anchor.toml
# Replace all instances of the old program ID with the new one

# 4. Rebuild
anchor build
```

### Build Verification

```bash
# Check if build artifacts exist
ls -la target/deploy/
# Should see: mpl_gov_micro.so and mpl_gov_micro-keypair.json

# Check IDL
ls -la target/idl/
# Should see: mpl_gov_micro.json

# Check TypeScript types
ls -la target/types/
# Should see: mpl_gov_micro.ts
```

---

## Testing

### Local Validator Testing

#### Option 1: Anchor Test (Recommended)

```bash
# Run all tests (starts validator automatically)
anchor test

# Run tests without rebuilding
anchor test --skip-build

# Run specific test
anchor test --skip-build -- --grep "Creates an election"

# Run with detailed logs
anchor test -- --verbose
```

#### Option 2: Manual Validator + Tests

```bash
# Terminal 1: Start local validator
solana-test-validator

# Terminal 2: Run tests
anchor test --skip-local-validator
```

### Test Output

Successful test output should show:

```
  mpl-gov-micro
    Election Creation
      ✔ Creates an election successfully (1234ms)
      ✔ Fails with too many candidates (567ms)
      ✔ Fails with invalid time range (432ms)
    Voter Registration
      ✔ Registers a voter successfully (890ms)
      ✔ Registers multiple voters (1120ms)
    Voting
      ✔ Casts a vote successfully (678ms)
      ✔ Records multiple votes for different candidates (1456ms)
      ✔ Prevents double voting (543ms)
      ✔ Fails with invalid choice (876ms)
    Election Closing
      ✔ Closes an election successfully (432ms)
      ✔ Prevents voting after election is closed (765ms)
    Complete Election Flow
      ✔ Runs a complete election from creation to close (3210ms)

  13 passing (12s)
```

### Test Individual Instructions

```bash
# Test only election creation
anchor test --skip-build -- --grep "Election Creation"

# Test only voting
anchor test --skip-build -- --grep "Voting"

# Test complete flow
anchor test --skip-build -- --grep "Complete Election Flow"
```

### Run TypeScript Tests Only

```bash
# After building and deploying locally
npm run test
# or
yarn test
```

---

## Deployment

### Deploy to Devnet

```bash
# 1. Set cluster to devnet
solana config set --url devnet

# 2. Ensure you have SOL
solana balance
# If low: solana airdrop 2

# 3. Build
anchor build

# 4. Deploy
anchor deploy --provider.cluster devnet

# 5. Verify deployment
solana program show <PROGRAM_ID>
```

### Deploy to Mainnet

⚠️ **WARNING: Mainnet deployment costs real SOL**

```bash
# 1. Switch to mainnet
solana config set --url mainnet-beta

# 2. Ensure you have enough SOL (at least 5 SOL)
solana balance

# 3. Build with mainnet config
anchor build

# 4. Update Anchor.toml to use mainnet
# Edit [provider] section:
# cluster = "Mainnet"

# 5. Deploy (CONFIRM YOU'RE READY!)
anchor deploy --provider.cluster mainnet-beta

# 6. Verify
solana program show <PROGRAM_ID>

# 7. IMPORTANT: Back up keypairs!
cp target/deploy/mpl_gov_micro-keypair.json ~/backups/
```

### Upgrade Program

```bash
# 1. Make code changes
# 2. Build
anchor build

# 3. Upgrade (uses same program ID)
anchor upgrade target/deploy/mpl_gov_micro.so --program-id <PROGRAM_ID>

# 4. Verify new version
solana program show <PROGRAM_ID>
```

---

## Program Operations

### Create an Election

```bash
# Using Anchor client
anchor run create-election
```

TypeScript example:

```typescript
const candidates = ["Alice", "Bob", "Charlie"];
const startTime = new anchor.BN(Math.floor(Date.now() / 1000));
const endTime = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

const [electionPda] = await PublicKey.findProgramAddress(
  [Buffer.from("election"), authority.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .createElection(candidates, startTime, endTime)
  .accounts({
    election: electionPda,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Register a Voter

```typescript
const [voterRegPda] = await PublicKey.findProgramAddress(
  [
    Buffer.from("voter_registration"),
    election.toBuffer(),
    voter.publicKey.toBuffer(),
  ],
  program.programId
);

await program.methods
  .registerVoter()
  .accounts({
    election: electionPda,
    voterRegistration: voterRegPda,
    voter: voter.publicKey,
    attestation: attestationPubkey,
    systemProgram: SystemProgram.programId,
  })
  .signers([voter])
  .rpc();
```

### Cast a Vote

```typescript
const [voterRegPda] = await PublicKey.findProgramAddress(
  [
    Buffer.from("voter_registration"),
    election.toBuffer(),
    voter.publicKey.toBuffer(),
  ],
  program.programId
);

const [nullifierSetPda] = await PublicKey.findProgramAddress(
  [Buffer.from("nullifiers"), election.toBuffer()],
  program.programId
);

await program.methods
  .castVote(0, []) // Vote for candidate 0, empty merkle proof for MVP
  .accounts({
    election: electionPda,
    voterRegistration: voterRegPda,
    nullifierSet: nullifierSetPda,
    voter: voter.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([voter])
  .rpc();
```

### Close an Election

```typescript
await program.methods
  .closeElection()
  .accounts({
    election: electionPda,
    authority: authority.publicKey,
  })
  .rpc();
```

### Query Election Results

```typescript
const election = await program.account.election.fetch(electionPda);

console.log("Election Results:");
console.log("Total Votes:", election.totalVotes.toNumber());
console.log("\nCandidates:");
election.candidates.forEach((candidate, idx) => {
  console.log(
    `  ${candidate}: ${election.voteCounts[idx].toNumber()} votes`
  );
});
```

---

## Monitoring & Debugging

### View Program Logs

```bash
# Stream logs for your program
solana logs <PROGRAM_ID>

# View logs with grep filter
solana logs <PROGRAM_ID> | grep "Vote cast"

# View transaction logs
solana confirm -v <TRANSACTION_SIGNATURE>
```

### Check Account Data

```bash
# View election account
solana account <ELECTION_PDA>

# View with deserialization (using anchor)
anchor account election <ELECTION_PDA>

# View voter registration
anchor account voterRegistration <VOTER_REG_PDA>

# View nullifier set
anchor account nullifierSet <NULLIFIER_SET_PDA>
```

### Get Program Info

```bash
# Program details
solana program show <PROGRAM_ID>

# Program account size
solana account <PROGRAM_ID>

# Check upgrade authority
solana program show <PROGRAM_ID> | grep "Upgrade Authority"
```

### Monitor Transaction Status

```bash
# Check recent transactions
solana transaction-history <WALLET_ADDRESS>

# Check specific transaction
solana confirm <SIGNATURE>

# Check with verbose output
solana confirm -v <SIGNATURE>
```

---

## Troubleshooting

### Build Errors

**Error: `command not found: anchor`**

```bash
# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

**Error: `failed to compile`**

```bash
# Clean and rebuild
anchor clean
rm -rf target/
anchor build
```

**Error: `program id mismatch`**

```bash
# Update program ID
anchor keys list
# Update lib.rs declare_id!() and Anchor.toml
anchor build
```

### Test Errors

**Error: `Airdrop request failed`**

```bash
# Use devnet instead
solana config set --url devnet
# Or start local validator
solana-test-validator
```

**Error: `Transaction simulation failed`**

```bash
# Check program logs
anchor test -- --verbose

# Run specific failing test
anchor test --skip-build -- --grep "test name"
```

**Error: `Account does not exist`**

```bash
# Ensure accounts are initialized in correct order:
# 1. Create election first
# 2. Register voters
# 3. Cast votes
```

### Deployment Errors

**Error: `Insufficient funds`**

```bash
# Check balance
solana balance

# Airdrop (devnet only)
solana airdrop 5

# For mainnet, transfer SOL from another wallet
```

**Error: `Program already deployed`**

```bash
# Use upgrade instead
anchor upgrade target/deploy/mpl_gov_micro.so --program-id <PROGRAM_ID>
```

**Error: `Invalid program executable`**

```bash
# Rebuild from scratch
anchor clean
anchor build
anchor deploy
```

### Runtime Errors

**Error: `AlreadyVoted`**

- Voter has already cast a vote in this election
- Check nullifier set for voter's nullifier

**Error: `InvalidChoice`**

- Choice index exceeds number of candidates
- Ensure choice is 0-indexed and < candidates.length

**Error: `ElectionNotStarted`**

- Current time < election.start_time
- Wait until start time or adjust start time

**Error: `ElectionEnded`**

- Current time > election.end_time
- Election is closed, no more votes accepted

---

## Maintenance

### Regular Tasks

#### Daily

```bash
# Check program health (if deployed)
solana program show <PROGRAM_ID>

# Monitor logs for errors
solana logs <PROGRAM_ID> | grep "Error"
```

#### Weekly

```bash
# Review active elections
# Query all election PDAs
# Check for any stuck states

# Update dependencies
npm update
cargo update
```

#### Monthly

```bash
# Security audit
cargo audit

# Update Anchor version
avm update
avm install latest

# Review and archive old elections
```

### Backup Procedures

```bash
# Backup program keypair
cp target/deploy/mpl_gov_micro-keypair.json ~/backups/keypair-$(date +%Y%m%d).json

# Backup IDL
cp target/idl/mpl_gov_micro.json ~/backups/idl-$(date +%Y%m%d).json

# Backup election data (programmatically)
# Create script to fetch and store all election PDAs
```

### Performance Optimization

```bash
# Optimize build
anchor build --release

# Check program size
ls -lh target/deploy/mpl_gov_micro.so

# Analyze compute units
# Add to test: transaction.computeUnits
```

### Security Checklist

- [ ] Program upgrade authority secured
- [ ] Keypair backups stored safely
- [ ] Access controls tested (authority checks)
- [ ] Double-vote prevention verified
- [ ] Integer overflow protection in place
- [ ] Input validation on all parameters
- [ ] PDA derivation security reviewed

---

## Quick Reference

### Common Commands

```bash
# Clean build
anchor clean && anchor build

# Test specific suite
anchor test --skip-build -- --grep "Voting"

# Deploy to devnet
solana config set --url devnet && anchor deploy

# View logs
solana logs <PROGRAM_ID>

# Check account
anchor account election <ELECTION_PDA>

# Airdrop SOL
solana airdrop 2
```

### Important PDAs

```typescript
// Election PDA
[b"election", authority.publicKey]

// Voter Registration PDA
[b"voter_registration", election.publicKey, voter.publicKey]

// Nullifier Set PDA
[b"nullifiers", election.publicKey]
```

### Program Accounts

| Account | Type | Size | Purpose |
|---------|------|------|---------|
| Election | Regular | ~726 bytes | Hot data, vote counts |
| VoterRegistration | Regular* | ~112 bytes | Voter eligibility |
| NullifierSet | Regular | Dynamic | Double-vote prevention |
| VoteRecord | Compressed* | ~138 bytes | Historical archive |

*Will be compressed in production

---

## Environment Variables

Create a `.env` file for configuration:

```bash
# Solana Configuration
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json

# Program IDs
PROGRAM_ID=<your_program_id>

# Testing
TEST_VALIDATOR_URL=http://localhost:8899
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Solana
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH

      - name: Install Anchor
        run: |
          cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
          avm install latest
          avm use latest

      - name: Build
        run: anchor build

      - name: Test
        run: anchor test
```

---

## Support & Resources

- **Anchor Documentation:** https://www.anchor-lang.com
- **Solana Documentation:** https://docs.solana.com
- **Light Protocol (zkCompression):** https://www.zkcompression.com
- **Project README:** [README.md](./README.md)
- **Build Guide:** [BUILD.md](./BUILD.md)

---

**Last Updated:** 2025-10-28
**Program Version:** MVP v0.1.0
**Anchor Version:** 0.30.1
