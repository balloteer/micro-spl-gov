# micro-spl-gov TypeScript SDK

TypeScript SDK for interacting with the micro-spl-gov program, featuring zkCompression utilities for cost-efficient governance.

## Features

- 🗜️ **zkCompression Support** - Full support for compressed voter registrations using merkle trees
- 💰 **Cost Optimization** - Up to 400x cost reduction compared to regular account storage
- 🔐 **Merkle Proof Verification** - Client-side proof generation and verification
- 🛠️ **Helper Functions** - Easy-to-use wrappers for common operations
- 📊 **Cost Analysis** - Built-in tools to calculate compression savings

## Installation

```bash
# Install the SDK (once published)
npm install @your-org/micro-spl-gov-sdk

# Or use locally
npm install ../sdk
```

## Quick Start

### Basic Usage

```typescript
import * as anchor from "@coral-xyz/anchor";
import {
  createCompressedElection,
  registerCompressedVoter,
  castCompressedVote,
  calculateCompressionSavings
} from "@your-org/micro-spl-gov-sdk";

// Initialize Anchor
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.MplGovMicro as Program<MplGovMicro>;

// Create a compressed election
const authority = Keypair.generate();
const candidates = ["Alice", "Bob", "Charlie"];
const startTime = new anchor.BN(Math.floor(Date.now() / 1000));
const endTime = new anchor.BN(startTime.toNumber() + 86400);

const txSig = await createCompressedElection(
  program,
  authority,
  candidates,
  startTime,
  endTime,
  10000 // max voters
);
```

### Working with Compressed Voters

```typescript
import {
  createCompressedVoterLeaf,
  SimpleMerkleTree,
  registerCompressedVoter
} from "@your-org/micro-spl-gov-sdk";

// Register a voter
const voter = Keypair.generate();
const attestation = Keypair.generate(); // From ballo-sns

await registerCompressedVoter(
  program,
  electionPda,
  voter,
  attestation.publicKey
);

// Generate leaf hash for client-side merkle tree
const registeredAt = Math.floor(Date.now() / 1000);
const leafHash = createCompressedVoterLeaf(
  voter.publicKey,
  electionPda,
  attestation.publicKey,
  registeredAt
);

// Add to merkle tree
const merkleTree = new SimpleMerkleTree();
merkleTree.addLeaf(leafHash);
```

### Casting Votes with Merkle Proofs

```typescript
import { castCompressedVote } from "@your-org/micro-spl-gov-sdk";

// Cast a vote using merkle proof
const choice = 0; // Vote for first candidate
const leafIndex = 0; // Voter's position in merkle tree
const merkleProof = merkleTree.getProof(leafIndex);

await castCompressedVote(
  program,
  electionPda,
  voter,
  attestation.publicKey,
  choice,
  leafIndex,
  registeredAt,
  merkleProof
);
```

## API Reference

### Compression Functions

#### `createCompressedVoterLeaf()`

Creates a leaf hash for a compressed voter registration. Must match the Rust implementation.

```typescript
function createCompressedVoterLeaf(
  voter: PublicKey,
  election: PublicKey,
  attestation: PublicKey,
  registeredAt: number
): Buffer
```

**Parameters:**
- `voter` - Voter's public key
- `election` - Election public key
- `attestation` - Attestation public key (from ballo-sns)
- `registeredAt` - Unix timestamp of registration

**Returns:** 32-byte keccak256 hash

#### `createCompressedElection()`

Helper to create an election with compression enabled.

```typescript
async function createCompressedElection(
  program: Program,
  authority: Keypair,
  candidates: string[],
  startTime: anchor.BN,
  endTime: anchor.BN,
  maxVoters: number
): Promise<string>
```

**Returns:** Transaction signature

#### `registerCompressedVoter()`

Registers a voter in compression mode (no account created).

```typescript
async function registerCompressedVoter(
  program: Program,
  election: PublicKey,
  voter: Keypair,
  attestation: PublicKey
): Promise<string>
```

#### `castCompressedVote()`

Casts a vote using merkle proof verification.

```typescript
async function castCompressedVote(
  program: Program,
  election: PublicKey,
  voter: Keypair,
  attestation: PublicKey,
  choice: number,
  leafIndex: number,
  registeredAt: number,
  merkleProof: Buffer[]
): Promise<string>
```

### PDA Derivation

#### `deriveElectionPda()`

```typescript
async function deriveElectionPda(
  authority: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]>
```

#### `deriveVoterRegistrationPda()`

```typescript
async function deriveVoterRegistrationPda(
  election: PublicKey,
  voter: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]>
```

#### `deriveNullifierSetPda()`

```typescript
async function deriveNullifierSetPda(
  election: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]>
```

### Merkle Tree Utilities

#### `SimpleMerkleTree`

Basic merkle tree implementation for testing and client-side proof generation.

```typescript
class SimpleMerkleTree {
  addLeaf(leaf: Buffer): void
  getLeaves(): Buffer[]
  getRoot(): Buffer
  getProof(leafIndex: number): Buffer[]
  static verifyProof(
    leaf: Buffer,
    proof: Buffer[],
    leafIndex: number,
    root: Buffer
  ): boolean
}
```

**Note:** This is an MVP implementation. For production, use a robust merkle tree library like `@solana/spl-account-compression`.

### Cost Analysis

#### `calculateCompressionSavings()`

Calculates cost savings of using compression vs regular accounts.

```typescript
function calculateCompressionSavings(numVoters: number): {
  numVoters: number;
  legacy: {
    perVoter: number;
    total: number;
  };
  compressed: {
    perVoter: number;
    total: number;
  };
  savings: {
    total: number;
    percentage: number;
    costReduction: number;
  };
}
```

**Example:**

```typescript
const savings = calculateCompressionSavings(10000);
console.log(`Cost reduction: ${savings.savings.costReduction.toFixed(0)}x`);
// Output: Cost reduction: 408x
```

### Utility Functions

#### `getMerkleTreeSize()`

Returns appropriate merkle tree depth and buffer size for max voters.

```typescript
function getMerkleTreeSize(maxVoters: number): [number, number]
```

#### `formatLeafHash()`

Formats a leaf hash for display (first 16 hex chars + ...).

```typescript
function formatLeafHash(leafHash: Buffer): string
```

#### `createNullifier()`

Creates a nullifier hash for double-vote prevention.

```typescript
function createNullifier(
  voter: PublicKey,
  election: PublicKey,
  nonce: number
): Buffer
```

## Cost Comparison

### Regular Accounts (Legacy Mode)

- **Per voter:** ~0.002039 SOL
- **1000 voters:** ~2.04 SOL

### Compressed Mode

- **Per voter:** ~0.000005 SOL
- **1000 voters:** ~0.005 SOL

### Savings

- **Per voter:** 99.8% cheaper
- **Cost reduction:** 408x
- **For 1000 voters:** Save ~2.03 SOL

## Examples

See the [examples](./examples/) directory for complete examples:

- [`compressed-election.ts`](./examples/compressed-election.ts) - Full compressed election workflow

## Architecture

### Dual-Mode System

The program supports both compression and legacy modes:

**Legacy Mode (use_compression = false):**
- Creates VoterRegistration accounts (~112 bytes each)
- Traditional on-chain storage
- Higher costs but simpler implementation

**Compression Mode (use_compression = true):**
- No VoterRegistration accounts created
- Voter data hashed into merkle tree leaves
- Merkle root stored in Election account
- Nullifier-based double-vote prevention
- 400x+ cost reduction

### Merkle Proof Verification Flow

1. **Registration:** Voter data is hashed → leaf added to merkle tree
2. **Voting:** Voter provides merkle proof showing they're in the tree
3. **Verification:** Program verifies proof against stored merkle root
4. **Nullifier:** Unique hash prevents double voting

## Integration with ballo-sns

This SDK is designed to work with the ballo-sns attestation system:

```typescript
// Get attestation from ballo-sns
const attestation = await balloSns.getAttestation(userWallet);

// Register voter with attestation
await registerCompressedVoter(
  program,
  electionPda,
  voter,
  attestation.publicKey
);
```

## TypeScript Types

```typescript
interface CompressedVoterData {
  voter: PublicKey;
  election: PublicKey;
  attestation: PublicKey;
  registeredAt: number;
}

interface MerkleProof {
  proof: Buffer[];
  leafIndex: number;
  root: Buffer;
}
```

## Development

```bash
# Build the SDK
npm run build

# Run tests
npm test

# Generate docs
npm run docs
```

## License

MIT

## Support

For issues, questions, or contributions, please visit the [GitHub repository](https://github.com/your-org/micro-spl-gov).
