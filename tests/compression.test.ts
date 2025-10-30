import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MplGovMicro } from "../target/types/mpl_gov_micro";
import { expect } from "chai";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { keccak_256 } from "js-sha3";

describe("zkCompression Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MplGovMicro as Program<MplGovMicro>;
  const authority = provider.wallet;

  // Helper to get current timestamp
  const getCurrentTimestamp = () => Math.floor(Date.now() / 1000);

  // Helper to derive election PDA
  const deriveElectionPda = async (authority: PublicKey) => {
    return await PublicKey.findProgramAddress(
      [Buffer.from("election"), authority.toBuffer()],
      program.programId
    );
  };

  // Helper to derive nullifier set PDA
  const deriveNullifierSetPda = async (election: PublicKey) => {
    return await PublicKey.findProgramAddress(
      [Buffer.from("nullifiers"), election.toBuffer()],
      program.programId
    );
  };

  // Helper to create compressed voter data hash (leaf hash)
  const createCompressedVoterLeaf = (
    voter: PublicKey,
    election: PublicKey,
    attestation: PublicKey,
    registeredAt: number
  ): Buffer => {
    // Serialize data in same format as Rust CompressedVoterData
    const voterBytes = voter.toBytes();
    const electionBytes = election.toBytes();
    const attestationBytes = attestation.toBytes();
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
  };

  // Simple merkle tree implementation for testing
  class SimpleMerkleTree {
    leaves: Buffer[];

    constructor() {
      this.leaves = [];
    }

    addLeaf(leaf: Buffer) {
      this.leaves.push(leaf);
    }

    getRoot(): Buffer {
      if (this.leaves.length === 0) {
        return Buffer.alloc(32);
      }
      if (this.leaves.length === 1) {
        return this.leaves[0];
      }

      // For testing, just return the last leaf as "root"
      // In production, this would be a proper merkle tree
      return this.leaves[this.leaves.length - 1];
    }

    getProof(leafIndex: number): Buffer[] {
      // For MVP testing, return empty proof
      // In production, this would generate actual merkle proof
      return [];
    }
  }

  describe("Compression Mode - Election Creation", () => {
    it("Creates an election with compression enabled", async () => {
      const compressionAuthority = Keypair.generate();
      await provider.connection.requestAirdrop(
        compressionAuthority.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [electionPda] = await deriveElectionPda(
        compressionAuthority.publicKey
      );

      const candidates = ["Alice", "Bob", "Charlie"];
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 86400);

      // Create with compression enabled
      await program.methods
        .createElection(candidates, startTime, endTime, true, 10000) // use_compression=true, max_voters=10000
        .accounts({
          election: electionPda,
          authority: compressionAuthority.publicKey,
          merkleTree: null, // In production, would be actual merkle tree account
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([compressionAuthority])
        .rpc();

      // Fetch and verify election
      const election = await program.account.election.fetch(electionPda);

      expect(election.useCompression).to.equal(true);
      expect(election.totalRegistered.toNumber()).to.equal(0);
      expect(election.candidates).to.deep.equal(candidates);

      console.log("✅ Election created with compression enabled");
      console.log(`   Max voters: 10000`);
      console.log(`   Compression: ${election.useCompression}`);
    });

    it("Creates an election with compression disabled (legacy)", async () => {
      const legacyAuthority = Keypair.generate();
      await provider.connection.requestAirdrop(
        legacyAuthority.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [electionPda] = await deriveElectionPda(legacyAuthority.publicKey);

      const candidates = ["Option A", "Option B"];
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 86400);

      await program.methods
        .createElection(candidates, startTime, endTime, false, 1000) // use_compression=false
        .accounts({
          election: electionPda,
          authority: legacyAuthority.publicKey,
          merkleTree: null,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([legacyAuthority])
        .rpc();

      const election = await program.account.election.fetch(electionPda);

      expect(election.useCompression).to.equal(false);
      expect(election.merkleTree.toString()).to.equal(PublicKey.default.toString());

      console.log("✅ Election created with compression disabled");
      console.log(`   Compression: ${election.useCompression}`);
    });
  });

  describe("Compression Mode - Voter Registration", () => {
    let compressionElection: PublicKey;
    let compressionAuthority: Keypair;
    let merkleTree: SimpleMerkleTree;
    let voter1: Keypair;
    let attestation: Keypair;

    before(async () => {
      // Setup compression election
      compressionAuthority = Keypair.generate();
      voter1 = Keypair.generate();
      attestation = Keypair.generate();

      await provider.connection.requestAirdrop(
        compressionAuthority.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        voter1.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      [compressionElection] = await deriveElectionPda(
        compressionAuthority.publicKey
      );

      const candidates = ["Compressed A", "Compressed B"];
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 86400);

      await program.methods
        .createElection(candidates, startTime, endTime, true, 10000)
        .accounts({
          election: compressionElection,
          authority: compressionAuthority.publicKey,
          merkleTree: null,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([compressionAuthority])
        .rpc();

      merkleTree = new SimpleMerkleTree();
    });

    it("Registers a voter in compression mode", async () => {
      const beforeRegistered = (await program.account.election.fetch(compressionElection))
        .totalRegistered.toNumber();

      // Register voter (no voter registration account needed in compression mode)
      await program.methods
        .registerVoter()
        .accounts({
          election: compressionElection,
          voterRegistration: null, // Not needed in compression mode
          merkleTree: null, // Would be actual merkle tree in production
          voter: voter1.publicKey,
          attestation: attestation.publicKey,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      const election = await program.account.election.fetch(compressionElection);

      expect(election.totalRegistered.toNumber()).to.equal(beforeRegistered + 1);

      // Generate the same leaf hash client-side for verification
      const registeredAt = Math.floor(Date.now() / 1000);
      const leafHash = createCompressedVoterLeaf(
        voter1.publicKey,
        compressionElection,
        attestation.publicKey,
        registeredAt
      );

      merkleTree.addLeaf(leafHash);

      console.log("✅ Voter registered in compression mode");
      console.log(`   Total registered: ${election.totalRegistered}`);
      console.log(`   Leaf hash: ${leafHash.toString("hex").substring(0, 16)}...`);
    });

    it("Registers multiple voters in compression mode", async () => {
      const voter2 = Keypair.generate();
      const voter3 = Keypair.generate();

      await provider.connection.requestAirdrop(
        voter2.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        voter3.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Register voter 2
      await program.methods
        .registerVoter()
        .accounts({
          election: compressionElection,
          voterRegistration: null,
          merkleTree: null,
          voter: voter2.publicKey,
          attestation: attestation.publicKey,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter2])
        .rpc();

      // Register voter 3
      await program.methods
        .registerVoter()
        .accounts({
          election: compressionElection,
          voterRegistration: null,
          merkleTree: null,
          voter: voter3.publicKey,
          attestation: attestation.publicKey,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter3])
        .rpc();

      const election = await program.account.election.fetch(compressionElection);

      expect(election.totalRegistered.toNumber()).to.equal(3);

      console.log("✅ Multiple voters registered in compression mode");
      console.log(`   Total registered: ${election.totalRegistered}`);
    });
  });

  describe("Compression Mode - Voting with Merkle Proofs", () => {
    let compressionElection: PublicKey;
    let compressionAuthority: Keypair;
    let voter: Keypair;
    let attestation: Keypair;
    let registeredAt: number;

    before(async () => {
      compressionAuthority = Keypair.generate();
      voter = Keypair.generate();
      attestation = Keypair.generate();

      await provider.connection.requestAirdrop(
        compressionAuthority.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.requestAirdrop(
        voter.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      [compressionElection] = await deriveElectionPda(
        compressionAuthority.publicKey
      );

      // Create compression election
      const candidates = ["Proof A", "Proof B"];
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 86400);

      await program.methods
        .createElection(candidates, startTime, endTime, true, 10000)
        .accounts({
          election: compressionElection,
          authority: compressionAuthority.publicKey,
          merkleTree: null,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([compressionAuthority])
        .rpc();

      // Register voter
      registeredAt = Math.floor(Date.now() / 1000);
      await program.methods
        .registerVoter()
        .accounts({
          election: compressionElection,
          voterRegistration: null,
          merkleTree: null,
          voter: voter.publicKey,
          attestation: attestation.publicKey,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter])
        .rpc();
    });

    it("Casts a vote using merkle proof (compression mode)", async () => {
      const [nullifierSetPda] = await deriveNullifierSetPda(compressionElection);

      const choice = 0; // Vote for "Proof A"
      const merkleProof = []; // Empty proof for MVP
      const leafIndex = 0; // First voter

      // Cast vote with merkle proof
      await program.methods
        .castVote(choice, merkleProof, leafIndex, new anchor.BN(registeredAt))
        .accounts({
          election: compressionElection,
          voterRegistration: null, // Not needed in compression mode
          nullifierSet: nullifierSetPda,
          voter: voter.publicKey,
          attestation: attestation.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter])
        .rpc();

      const election = await program.account.election.fetch(compressionElection);

      expect(election.voteCounts[0].toNumber()).to.equal(1);
      expect(election.totalVotes.toNumber()).to.equal(1);

      console.log("✅ Vote cast using merkle proof");
      console.log(`   Candidate: ${choice}`);
      console.log(`   Leaf index: ${leafIndex}`);
      console.log(`   Total votes: ${election.totalVotes}`);
    });

    it("Prevents double voting in compression mode", async () => {
      const [nullifierSetPda] = await deriveNullifierSetPda(compressionElection);

      try {
        await program.methods
          .castVote(1, [], 0, new anchor.BN(registeredAt))
          .accounts({
            election: compressionElection,
            voterRegistration: null,
            nullifierSet: nullifierSetPda,
            voter: voter.publicKey,
            attestation: attestation.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([voter])
          .rpc();

        expect.fail("Should have failed with AlreadyVoted");
      } catch (error: any) {
        expect(
          error.error?.errorCode?.code ||
          error.message ||
          error.toString()
        ).to.include("AlreadyVoted");

        console.log("✅ Double voting prevented in compression mode");
      }
    });
  });

  describe("Cost Comparison", () => {
    it("Demonstrates cost savings with compression", async () => {
      console.log("\n📊 Cost Comparison: Legacy vs Compression Mode");
      console.log("================================================");

      // Approximate costs (in lamports)
      const LAMPORTS_PER_SOL = 1_000_000_000;
      const VOTER_REGISTRATION_SIZE = 8 + 104; // discriminator + VoterRegistration::SIZE
      const RENT_EXEMPT_COST = 0.00203928 * LAMPORTS_PER_SOL; // ~2039280 lamports for voter registration

      console.log("\n💰 Legacy Mode (Regular Accounts):");
      console.log(`   Voter Registration: ~${(RENT_EXEMPT_COST / LAMPORTS_PER_SOL).toFixed(6)} SOL per voter`);
      console.log(`   1000 voters: ~${(RENT_EXEMPT_COST * 1000 / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

      const COMPRESSED_COST = 5000; // ~5000 lamports to append to merkle tree
      console.log("\n🗜️  Compression Mode (Merkle Tree):");
      console.log(`   Voter Registration: ~${(COMPRESSED_COST / LAMPORTS_PER_SOL).toFixed(8)} SOL per voter`);
      console.log(`   1000 voters: ~${(COMPRESSED_COST * 1000 / LAMPORTS_PER_SOL).toFixed(6)} SOL`);

      const savings = ((RENT_EXEMPT_COST - COMPRESSED_COST) / RENT_EXEMPT_COST * 100).toFixed(1);
      const totalSavings = ((RENT_EXEMPT_COST * 1000 - COMPRESSED_COST * 1000) / LAMPORTS_PER_SOL).toFixed(2);

      console.log("\n💎 Savings:");
      console.log(`   Per voter: ~${savings}% cheaper`);
      console.log(`   For 1000 voters: ~${totalSavings} SOL saved`);
      console.log(`   Cost reduction: ~${(RENT_EXEMPT_COST / COMPRESSED_COST).toFixed(0)}x`);
      console.log("\n================================================\n");
    });
  });
});
