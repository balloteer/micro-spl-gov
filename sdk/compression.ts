/**
 * TypeScript SDK helpers for zkCompression features
 *
 * This module provides utility functions for working with compressed voter registrations,
 * merkle proofs, and compression-enabled elections.
 */

import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { keccak_256 } from "js-sha3";
import { Program } from "@coral-xyz/anchor";

/**
 * Compressed voter data structure matching Rust implementation
 */
export interface CompressedVoterData {
  voter: PublicKey;
  election: PublicKey;
  attestation: PublicKey;
  registeredAt: number;
}

/**
 * Merkle proof for compressed voter verification
 */
export interface MerkleProof {
  proof: Buffer[];
  leafIndex: number;
  root: Buffer;
}

/**
 * Creates a leaf hash for a compressed voter registration
 * This must match the Rust implementation in utils/compression.rs
 *
 * @param voter - Voter's public key
 * @param election - Election public key
 * @param attestation - Attestation public key
 * @param registeredAt - Unix timestamp of registration
 * @returns 32-byte leaf hash
 */
export function createCompressedVoterLeaf(
  voter: PublicKey,
  election: PublicKey,
  attestation: PublicKey,
  registeredAt: number
): Buffer {
  // Serialize data in same format as Rust CompressedVoterData
  const voterBytes = voter.toBytes();
  const electionBytes = election.toBytes();
  const attestationBytes = attestation.toBytes();

  // Convert timestamp to 8-byte little-endian
  const registeredAtBytes = Buffer.alloc(8);
  registeredAtBytes.writeBigInt64LE(BigInt(registeredAt));

  // Concatenate all bytes
  const data = Buffer.concat([
    voterBytes,
    electionBytes,
    attestationBytes,
    registeredAtBytes,
  ]);

  // Hash with keccak256
  const hash = keccak_256(data);
  return Buffer.from(hash, "hex");
}

/**
 * Derives the Election PDA address
 *
 * @param authority - Election authority public key
 * @param programId - Program ID
 * @returns [PDA address, bump seed]
 */
export async function deriveElectionPda(
  authority: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(
    [Buffer.from("election"), authority.toBuffer()],
    programId
  );
}

/**
 * Derives the Voter Registration PDA address
 *
 * @param election - Election public key
 * @param voter - Voter public key
 * @param programId - Program ID
 * @returns [PDA address, bump seed]
 */
export async function deriveVoterRegistrationPda(
  election: PublicKey,
  voter: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(
    [Buffer.from("voter_registration"), election.toBuffer(), voter.toBuffer()],
    programId
  );
}

/**
 * Derives the Nullifier Set PDA address
 *
 * @param election - Election public key
 * @param programId - Program ID
 * @returns [PDA address, bump seed]
 */
export async function deriveNullifierSetPda(
  election: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(
    [Buffer.from("nullifiers"), election.toBuffer()],
    programId
  );
}

/**
 * Simple Merkle Tree implementation for testing and client-side proof generation
 */
export class SimpleMerkleTree {
  private leaves: Buffer[];

  constructor() {
    this.leaves = [];
  }

  /**
   * Adds a leaf to the tree
   */
  addLeaf(leaf: Buffer): void {
    this.leaves.push(leaf);
  }

  /**
   * Gets all leaves
   */
  getLeaves(): Buffer[] {
    return [...this.leaves];
  }

  /**
   * Computes the merkle root
   * For MVP, returns the last leaf as root
   * In production, this would compute the actual merkle root
   */
  getRoot(): Buffer {
    if (this.leaves.length === 0) {
      return Buffer.alloc(32);
    }
    if (this.leaves.length === 1) {
      return this.leaves[0];
    }

    // For MVP testing, return the last leaf as "root"
    // In production, implement proper merkle tree root calculation
    return this.leaves[this.leaves.length - 1];
  }

  /**
   * Generates a merkle proof for a leaf at the given index
   * For MVP, returns empty proof
   * In production, this would generate actual merkle siblings
   *
   * @param leafIndex - Index of the leaf to prove
   * @returns Array of sibling hashes forming the proof
   */
  getProof(leafIndex: number): Buffer[] {
    // For MVP testing, return empty proof
    // In production, implement actual merkle proof generation
    return [];
  }

  /**
   * Verifies a merkle proof
   *
   * @param leaf - Leaf hash to verify
   * @param proof - Merkle proof (array of sibling hashes)
   * @param leafIndex - Index of the leaf in the tree
   * @param root - Expected root hash
   * @returns true if proof is valid
   */
  static verifyProof(
    leaf: Buffer,
    proof: Buffer[],
    leafIndex: number,
    root: Buffer
  ): boolean {
    // For MVP with empty proofs, just check if leaf equals root
    if (proof.length === 0) {
      return leaf.equals(root);
    }

    // Production implementation would reconstruct root from proof
    let computedHash = leaf;
    for (let i = 0; i < proof.length; i++) {
      const sibling = proof[i];
      const isLeftNode = (leafIndex & (1 << i)) === 0;

      if (isLeftNode) {
        computedHash = Buffer.from(
          keccak_256(Buffer.concat([computedHash, sibling])),
          "hex"
        );
      } else {
        computedHash = Buffer.from(
          keccak_256(Buffer.concat([sibling, computedHash])),
          "hex"
        );
      }
    }

    return computedHash.equals(root);
  }
}

/**
 * Helper to create an election with compression enabled
 *
 * @param program - Anchor program instance
 * @param authority - Election authority keypair
 * @param candidates - List of candidate names
 * @param startTime - Election start timestamp
 * @param endTime - Election end timestamp
 * @param maxVoters - Maximum number of voters for merkle tree sizing
 * @returns Transaction signature
 */
export async function createCompressedElection(
  program: Program,
  authority: Keypair,
  candidates: string[],
  startTime: anchor.BN,
  endTime: anchor.BN,
  maxVoters: number = 10000
): Promise<string> {
  const [electionPda] = await deriveElectionPda(
    authority.publicKey,
    program.programId
  );

  return await program.methods
    .createElection(candidates, startTime, endTime, true, maxVoters)
    .accounts({
      election: electionPda,
      authority: authority.publicKey,
      merkleTree: null, // MVP mode - merkle tree setup deferred
      compressionProgram: null,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([authority])
    .rpc();
}

/**
 * Helper to register a voter in compression mode
 *
 * @param program - Anchor program instance
 * @param election - Election public key
 * @param voter - Voter keypair
 * @param attestation - Attestation public key
 * @returns Transaction signature
 */
export async function registerCompressedVoter(
  program: Program,
  election: PublicKey,
  voter: Keypair,
  attestation: PublicKey
): Promise<string> {
  return await program.methods
    .registerVoter()
    .accounts({
      election: election,
      voterRegistration: null, // Not needed in compression mode
      merkleTree: null,
      voter: voter.publicKey,
      attestation: attestation,
      compressionProgram: null,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([voter])
    .rpc();
}

/**
 * Helper to cast a vote using merkle proof (compression mode)
 *
 * @param program - Anchor program instance
 * @param election - Election public key
 * @param voter - Voter keypair
 * @param attestation - Attestation public key
 * @param choice - Candidate index to vote for
 * @param leafIndex - Index of voter in merkle tree
 * @param registeredAt - Timestamp when voter registered
 * @param merkleProof - Merkle proof (empty array for MVP)
 * @returns Transaction signature
 */
export async function castCompressedVote(
  program: Program,
  election: PublicKey,
  voter: Keypair,
  attestation: PublicKey,
  choice: number,
  leafIndex: number,
  registeredAt: number,
  merkleProof: Buffer[] = []
): Promise<string> {
  const [nullifierSetPda] = await deriveNullifierSetPda(
    election,
    program.programId
  );

  return await program.methods
    .castVote(choice, merkleProof, leafIndex, new anchor.BN(registeredAt))
    .accounts({
      election: election,
      voterRegistration: null, // Not needed in compression mode
      nullifierSet: nullifierSetPda,
      voter: voter.publicKey,
      attestation: attestation,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([voter])
    .rpc();
}

/**
 * Calculates the cost savings of using compression
 *
 * @param numVoters - Number of voters
 * @returns Cost comparison object
 */
export function calculateCompressionSavings(numVoters: number) {
  const LAMPORTS_PER_SOL = 1_000_000_000;
  const VOTER_REGISTRATION_SIZE = 8 + 104; // discriminator + VoterRegistration::SIZE
  const RENT_EXEMPT_COST = 0.00203928 * LAMPORTS_PER_SOL; // ~2039280 lamports
  const COMPRESSED_COST = 5000; // ~5000 lamports to append to merkle tree

  const legacyTotalCost = (RENT_EXEMPT_COST * numVoters) / LAMPORTS_PER_SOL;
  const compressedTotalCost = (COMPRESSED_COST * numVoters) / LAMPORTS_PER_SOL;
  const savings = legacyTotalCost - compressedTotalCost;
  const savingsPercentage = ((RENT_EXEMPT_COST - COMPRESSED_COST) / RENT_EXEMPT_COST) * 100;
  const costReduction = RENT_EXEMPT_COST / COMPRESSED_COST;

  return {
    numVoters,
    legacy: {
      perVoter: RENT_EXEMPT_COST / LAMPORTS_PER_SOL,
      total: legacyTotalCost,
    },
    compressed: {
      perVoter: COMPRESSED_COST / LAMPORTS_PER_SOL,
      total: compressedTotalCost,
    },
    savings: {
      total: savings,
      percentage: savingsPercentage,
      costReduction: costReduction,
    },
  };
}

/**
 * Gets the merkle tree depth and buffer size for a given number of voters
 * This matches the Rust implementation in create_election.rs
 *
 * @param maxVoters - Maximum number of voters
 * @returns [depth, bufferSize]
 */
export function getMerkleTreeSize(maxVoters: number): [number, number] {
  if (maxVoters <= 1024) {
    return [10, 64]; // 2^10 = 1024 max leaves
  } else if (maxVoters <= 4096) {
    return [12, 64]; // 2^12 = 4096 max leaves
  } else if (maxVoters <= 16384) {
    return [14, 64]; // 2^14 = 16384 max leaves
  } else {
    return [20, 256]; // 2^20 = 1M+ max leaves
  }
}

/**
 * Type guard to check if an election is using compression
 */
export function isCompressionEnabled(election: any): boolean {
  return election.useCompression === true;
}

/**
 * Formats a leaf hash for display (first 16 hex chars + ...)
 */
export function formatLeafHash(leafHash: Buffer): string {
  return leafHash.toString("hex").substring(0, 16) + "...";
}

/**
 * Creates a nullifier hash for double-vote prevention
 * This should match the Rust VoteNullifier implementation
 *
 * @param voter - Voter public key
 * @param election - Election public key
 * @param nonce - Nonce (0 for MVP)
 * @returns 32-byte nullifier hash
 */
export function createNullifier(
  voter: PublicKey,
  election: PublicKey,
  nonce: number = 0
): Buffer {
  const voterBytes = voter.toBytes();
  const electionBytes = election.toBytes();
  const nonceBytes = Buffer.alloc(8);
  nonceBytes.writeBigInt64LE(BigInt(nonce));

  const data = Buffer.concat([voterBytes, electionBytes, nonceBytes]);
  const hash = keccak_256(data);
  return Buffer.from(hash, "hex");
}
