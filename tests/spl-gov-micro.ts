import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MplGovMicro } from "../target/types/mpl_gov_micro";
import { expect } from "chai";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";

describe("mpl-gov-micro", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MplGovMicro as Program<MplGovMicro>;

  // Test accounts
  const authority = provider.wallet;
  let electionPda: PublicKey;
  let electionBump: number;
  let voter1: Keypair;
  let voter2: Keypair;
  let voter3: Keypair;
  let attestation: Keypair;

  // Helper to get current timestamp
  const getCurrentTimestamp = () => Math.floor(Date.now() / 1000);

  // Helper to derive PDAs
  const deriveElectionPda = async (authority: PublicKey) => {
    return await PublicKey.findProgramAddress(
      [Buffer.from("election"), authority.toBuffer()],
      program.programId
    );
  };

  const deriveVoterRegistrationPda = async (
    election: PublicKey,
    voter: PublicKey
  ) => {
    return await PublicKey.findProgramAddress(
      [
        Buffer.from("voter_registration"),
        election.toBuffer(),
        voter.toBuffer(),
      ],
      program.programId
    );
  };

  const deriveNullifierSetPda = async (election: PublicKey) => {
    return await PublicKey.findProgramAddress(
      [Buffer.from("nullifiers"), election.toBuffer()],
      program.programId
    );
  };

  before(async () => {
    // Create test keypairs
    voter1 = Keypair.generate();
    voter2 = Keypair.generate();
    voter3 = Keypair.generate();
    attestation = Keypair.generate();

    // Airdrop SOL to test voters
    await provider.connection.requestAirdrop(
      voter1.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      voter2.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      voter3.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );

    // Wait for airdrops
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Derive election PDA
    [electionPda, electionBump] = await deriveElectionPda(
      authority.publicKey
    );
  });

  describe("Election Creation", () => {
    it("Creates an election successfully", async () => {
      const candidates = ["Alice", "Bob", "Charlie"];
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 86400); // +24 hours

      await program.methods
        .createElection(candidates, startTime, endTime, false, 1000) // use_compression=false, max_voters=1000
        .accounts({
          election: electionPda,
          authority: authority.publicKey,
          merkleTree: null,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Fetch and verify election data
      const election = await program.account.election.fetch(electionPda);

      expect(election.authority.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(election.candidates).to.deep.equal(candidates);
      expect(election.voteCounts.map((v) => v.toNumber())).to.deep.equal([
        0, 0, 0,
      ]);
      expect(election.totalVotes.toNumber()).to.equal(0);
      expect(election.startTime.toNumber()).to.equal(startTime.toNumber());
      expect(election.endTime.toNumber()).to.equal(endTime.toNumber());
    });

    it("Fails with too many candidates", async () => {
      // Use a different authority to avoid account collision
      const testAuthority = Keypair.generate();
      await provider.connection.requestAirdrop(
        testAuthority.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [testElectionPda] = await deriveElectionPda(testAuthority.publicKey);

      const tooManyCandidates = Array(11)
        .fill(null)
        .map((_, i) => `Candidate${i}`);
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 86400);

      try {
        await program.methods
          .createElection(tooManyCandidates, startTime, endTime, false, 1000)
          .accounts({
            election: testElectionPda,
            authority: testAuthority.publicKey,
            merkleTree: null,
            compressionProgram: null,
            systemProgram: SystemProgram.programId,
          })
          .signers([testAuthority])
          .rpc();
        expect.fail("Should have failed with too many candidates");
      } catch (error: any) {
        expect(
          error.error?.errorCode?.code ||
          error.message ||
          error.toString()
        ).to.include("TooManyCandidates");
      }
    });

    it("Fails with invalid time range", async () => {
      // Use a different authority to avoid account collision
      const testAuthority = Keypair.generate();
      await provider.connection.requestAirdrop(
        testAuthority.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [testElectionPda] = await deriveElectionPda(testAuthority.publicKey);

      const candidates = ["Alice", "Bob"];
      const startTime = new anchor.BN(getCurrentTimestamp() + 86400);
      const endTime = new anchor.BN(getCurrentTimestamp()); // End before start

      try {
        await program.methods
          .createElection(candidates, startTime, endTime, false, 1000)
          .accounts({
            election: testElectionPda,
            authority: testAuthority.publicKey,
            merkleTree: null,
            compressionProgram: null,
            systemProgram: SystemProgram.programId,
          })
          .signers([testAuthority])
          .rpc();
        expect.fail("Should have failed with invalid time range");
      } catch (error: any) {
        expect(
          error.error?.errorCode?.code ||
          error.message ||
          error.toString()
        ).to.include("InvalidTimeRange");
      }
    });
  });

  describe("Voter Registration", () => {
    it("Registers a voter successfully", async () => {
      const [voterRegPda] = await deriveVoterRegistrationPda(
        electionPda,
        voter1.publicKey
      );

      await program.methods
        .registerVoter()
        .accounts({
          election: electionPda,
          voterRegistration: voterRegPda,
          voter: voter1.publicKey,
          attestation: attestation.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      // Verify registration
      const voterReg = await program.account.voterRegistration.fetch(
        voterRegPda
      );

      expect(voterReg.wallet.toString()).to.equal(voter1.publicKey.toString());
      expect(voterReg.election.toString()).to.equal(electionPda.toString());
      expect(voterReg.attestation.toString()).to.equal(
        attestation.publicKey.toString()
      );
    });

    it("Registers multiple voters", async () => {
      const [voterReg2Pda] = await deriveVoterRegistrationPda(
        electionPda,
        voter2.publicKey
      );

      await program.methods
        .registerVoter()
        .accounts({
          election: electionPda,
          voterRegistration: voterReg2Pda,
          voter: voter2.publicKey,
          attestation: attestation.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter2])
        .rpc();

      const [voterReg3Pda] = await deriveVoterRegistrationPda(
        electionPda,
        voter3.publicKey
      );

      await program.methods
        .registerVoter()
        .accounts({
          election: electionPda,
          voterRegistration: voterReg3Pda,
          voter: voter3.publicKey,
          attestation: attestation.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter3])
        .rpc();

      // Verify both registrations
      const voterReg2 = await program.account.voterRegistration.fetch(
        voterReg2Pda
      );
      const voterReg3 = await program.account.voterRegistration.fetch(
        voterReg3Pda
      );

      expect(voterReg2.wallet.toString()).to.equal(
        voter2.publicKey.toString()
      );
      expect(voterReg3.wallet.toString()).to.equal(
        voter3.publicKey.toString()
      );
    });
  });

  describe("Voting", () => {
    it("Casts a vote successfully", async () => {
      const [voterRegPda] = await deriveVoterRegistrationPda(
        electionPda,
        voter1.publicKey
      );
      const [nullifierSetPda] = await deriveNullifierSetPda(electionPda);

      const choice = 0; // Vote for Alice
      const merkleProof = []; // Empty for MVP

      await program.methods
        .castVote(choice, merkleProof)
        .accounts({
          election: electionPda,
          voterRegistration: voterRegPda,
          nullifierSet: nullifierSetPda,
          voter: voter1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      // Verify vote was recorded
      const election = await program.account.election.fetch(electionPda);

      expect(election.voteCounts[0].toNumber()).to.equal(1);
      expect(election.totalVotes.toNumber()).to.equal(1);
    });

    it("Records multiple votes for different candidates", async () => {
      const [voterReg2Pda] = await deriveVoterRegistrationPda(
        electionPda,
        voter2.publicKey
      );
      const [nullifierSetPda] = await deriveNullifierSetPda(electionPda);

      // Voter 2 votes for Bob (choice 1)
      await program.methods
        .castVote(1, [])
        .accounts({
          election: electionPda,
          voterRegistration: voterReg2Pda,
          nullifierSet: nullifierSetPda,
          voter: voter2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter2])
        .rpc();

      const [voterReg3Pda] = await deriveVoterRegistrationPda(
        electionPda,
        voter3.publicKey
      );

      // Voter 3 votes for Charlie (choice 2)
      await program.methods
        .castVote(2, [])
        .accounts({
          election: electionPda,
          voterRegistration: voterReg3Pda,
          nullifierSet: nullifierSetPda,
          voter: voter3.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter3])
        .rpc();

      // Verify vote counts
      const election = await program.account.election.fetch(electionPda);

      expect(election.voteCounts.map((v) => v.toNumber())).to.deep.equal([
        1, 1, 1,
      ]); // Alice: 1, Bob: 1, Charlie: 1
      expect(election.totalVotes.toNumber()).to.equal(3);
    });

    it("Prevents double voting", async () => {
      const [voterRegPda] = await deriveVoterRegistrationPda(
        electionPda,
        voter1.publicKey
      );
      const [nullifierSetPda] = await deriveNullifierSetPda(electionPda);

      try {
        // Try to vote again
        await program.methods
          .castVote(1, [])
          .accounts({
            election: electionPda,
            voterRegistration: voterRegPda,
            nullifierSet: nullifierSetPda,
            voter: voter1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([voter1])
          .rpc();

        expect.fail("Should have failed with AlreadyVoted");
      } catch (error: any) {
        expect(
          error.error?.errorCode?.code ||
          error.message ||
          error.toString()
        ).to.include("AlreadyVoted");
      }
    });

    it("Fails with invalid choice", async () => {
      const voter4 = Keypair.generate();
      await provider.connection.requestAirdrop(
        voter4.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [voterReg4Pda] = await deriveVoterRegistrationPda(
        electionPda,
        voter4.publicKey
      );

      // Register voter4
      await program.methods
        .registerVoter()
        .accounts({
          election: electionPda,
          voterRegistration: voterReg4Pda,
          voter: voter4.publicKey,
          attestation: attestation.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter4])
        .rpc();

      const [nullifierSetPda] = await deriveNullifierSetPda(electionPda);

      try {
        // Try to vote for invalid candidate (index 3, but only 0-2 exist)
        await program.methods
          .castVote(3, [])
          .accounts({
            election: electionPda,
            voterRegistration: voterReg4Pda,
            nullifierSet: nullifierSetPda,
            voter: voter4.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([voter4])
          .rpc();

        expect.fail("Should have failed with InvalidChoice");
      } catch (error: any) {
        expect(
          error.error?.errorCode?.code ||
          error.message ||
          error.toString()
        ).to.include("InvalidChoice");
      }
    });
  });

  describe("Election Closing", () => {
    it("Closes an election successfully", async () => {
      await program.methods
        .closeElection()
        .accounts({
          election: electionPda,
          authority: authority.publicKey,
        })
        .rpc();

      // Verify election is closed
      const election = await program.account.election.fetch(electionPda);

      // Status enum: Pending=0, Active=1, Ended=2, Cancelled=3
      expect(election.status).to.have.property("ended");
    });

    it("Prevents voting after election is closed", async () => {
      const voter5 = Keypair.generate();
      await provider.connection.requestAirdrop(
        voter5.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [voterReg5Pda] = await deriveVoterRegistrationPda(
        electionPda,
        voter5.publicKey
      );

      // Register voter5
      await program.methods
        .registerVoter()
        .accounts({
          election: electionPda,
          voterRegistration: voterReg5Pda,
          voter: voter5.publicKey,
          attestation: attestation.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter5])
        .rpc();

      const [nullifierSetPda] = await deriveNullifierSetPda(electionPda);

      try {
        await program.methods
          .castVote(0, [])
          .accounts({
            election: electionPda,
            voterRegistration: voterReg5Pda,
            nullifierSet: nullifierSetPda,
            voter: voter5.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([voter5])
          .rpc();

        expect.fail("Should have failed - election is ended");
      } catch (error: any) {
        expect(
          error.error?.errorCode?.code ||
          error.message ||
          error.toString()
        ).to.include("ElectionEnded");
      }
    });
  });

  describe("Complete Election Flow", () => {
    it("Runs a complete election from creation to close", async () => {
      // Create new election
      const authority2 = Keypair.generate();
      await provider.connection.requestAirdrop(
        authority2.publicKey,
        5 * anchor.web3.LAMPORTS_PER_SOL
      );
      await new Promise((resolve) => setTimeout(resolve, 1000));

      const [election2Pda] = await deriveElectionPda(authority2.publicKey);

      const candidates = ["Option A", "Option B"];
      const startTime = new anchor.BN(getCurrentTimestamp());
      const endTime = new anchor.BN(getCurrentTimestamp() + 3600);

      await program.methods
        .createElection(candidates, startTime, endTime, false, 1000)
        .accounts({
          election: election2Pda,
          authority: authority2.publicKey,
          merkleTree: null,
          compressionProgram: null,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority2])
        .rpc();

      // Create voters
      const voters = [Keypair.generate(), Keypair.generate(), Keypair.generate()];
      for (const voter of voters) {
        await provider.connection.requestAirdrop(
          voter.publicKey,
          2 * anchor.web3.LAMPORTS_PER_SOL
        );
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Register voters
      for (const voter of voters) {
        const [voterRegPda] = await deriveVoterRegistrationPda(
          election2Pda,
          voter.publicKey
        );

        await program.methods
          .registerVoter()
          .accounts({
            election: election2Pda,
            voterRegistration: voterRegPda,
            voter: voter.publicKey,
            attestation: attestation.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([voter])
          .rpc();
      }

      // Cast votes (2 for Option A, 1 for Option B)
      const [nullifierSet2Pda] = await deriveNullifierSetPda(election2Pda);

      for (let i = 0; i < voters.length; i++) {
        const [voterRegPda] = await deriveVoterRegistrationPda(
          election2Pda,
          voters[i].publicKey
        );

        const choice = i < 2 ? 0 : 1; // First 2 vote for Option A, last votes for Option B

        await program.methods
          .castVote(choice, [])
          .accounts({
            election: election2Pda,
            voterRegistration: voterRegPda,
            nullifierSet: nullifierSet2Pda,
            voter: voters[i].publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([voters[i]])
          .rpc();
      }

      // Verify results before closing
      let election = await program.account.election.fetch(election2Pda);
      expect(election.voteCounts.map((v) => v.toNumber())).to.deep.equal([2, 1]);
      expect(election.totalVotes.toNumber()).to.equal(3);

      // Close election
      await program.methods
        .closeElection()
        .accounts({
          election: election2Pda,
          authority: authority2.publicKey,
        })
        .signers([authority2])
        .rpc();

      // Verify final state
      election = await program.account.election.fetch(election2Pda);
      expect(election.status).to.have.property("ended");
      expect(election.voteCounts.map((v) => v.toNumber())).to.deep.equal([2, 1]);
      expect(election.totalVotes.toNumber()).to.equal(3);
    });
  });
});
