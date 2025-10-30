/**
 * Example: Creating and running a compressed election
 *
 * This example demonstrates how to use the SDK helpers to:
 * 1. Create an election with compression enabled
 * 2. Register voters using compressed data
 * 3. Cast votes with merkle proofs
 * 4. Calculate cost savings
 */

import * as anchor from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import {
  createCompressedElection,
  registerCompressedVoter,
  castCompressedVote,
  createCompressedVoterLeaf,
  SimpleMerkleTree,
  calculateCompressionSavings,
  deriveElectionPda,
  formatLeafHash,
} from "../compression";

// This example assumes you have:
// - A running local validator or connection to devnet
// - An initialized Anchor program instance
// - A wallet with SOL for transactions

async function runCompressedElectionExample(
  program: anchor.Program,
  provider: anchor.AnchorProvider
) {
  console.log("🗳️  Compressed Election Example\n");

  // 1. Create an election authority
  const authority = Keypair.generate();
  console.log("Creating election authority...");

  // Airdrop SOL to authority (for local testing)
  const airdropSig = await provider.connection.requestAirdrop(
    authority.publicKey,
    5 * anchor.web3.LAMPORTS_PER_SOL
  );
  await provider.connection.confirmTransaction(airdropSig);
  console.log("✅ Authority funded\n");

  // 2. Create a compressed election
  const candidates = ["Alice", "Bob", "Charlie"];
  const startTime = new anchor.BN(Math.floor(Date.now() / 1000));
  const endTime = new anchor.BN(Math.floor(Date.now() / 1000) + 86400); // 24 hours
  const maxVoters = 10000;

  console.log("Creating compressed election...");
  console.log(`  Candidates: ${candidates.join(", ")}`);
  console.log(`  Max voters: ${maxVoters}`);

  const createTx = await createCompressedElection(
    program,
    authority,
    candidates,
    startTime,
    endTime,
    maxVoters
  );

  console.log(`✅ Election created: ${createTx}\n`);

  // Get election PDA
  const [electionPda] = await deriveElectionPda(
    authority.publicKey,
    program.programId
  );

  // Fetch and display election details
  const election = await program.account.election.fetch(electionPda);
  console.log("Election Details:");
  console.log(`  Address: ${electionPda.toString()}`);
  console.log(`  Compression enabled: ${election.useCompression}`);
  console.log(`  Candidates: ${election.candidates.length}`);
  console.log(`  Status: ${Object.keys(election.status)[0]}\n`);

  // 3. Register voters
  console.log("Registering voters...");
  const voters: Keypair[] = [];
  const attestations: Keypair[] = [];
  const merkleTree = new SimpleMerkleTree();

  for (let i = 0; i < 3; i++) {
    const voter = Keypair.generate();
    const attestation = Keypair.generate();

    // Airdrop SOL to voter
    const voterAirdrop = await provider.connection.requestAirdrop(
      voter.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(voterAirdrop);

    // Register voter
    const registeredAt = Math.floor(Date.now() / 1000);
    const registerTx = await registerCompressedVoter(
      program,
      electionPda,
      voter,
      attestation.publicKey
    );

    // Generate leaf hash for merkle tree
    const leafHash = createCompressedVoterLeaf(
      voter.publicKey,
      electionPda,
      attestation.publicKey,
      registeredAt
    );
    merkleTree.addLeaf(leafHash);

    voters.push(voter);
    attestations.push(attestation);

    console.log(`  ✅ Voter ${i + 1} registered`);
    console.log(`     Leaf hash: ${formatLeafHash(leafHash)}`);
  }

  // Fetch updated election
  const updatedElection = await program.account.election.fetch(electionPda);
  console.log(`\n✅ Total registered: ${updatedElection.totalRegistered}\n`);

  // 4. Cast votes
  console.log("Casting votes...");

  for (let i = 0; i < voters.length; i++) {
    const voter = voters[i];
    const attestation = attestations[i];
    const choice = i % candidates.length; // Vote for different candidates
    const registeredAt = Math.floor(Date.now() / 1000);

    // Get merkle proof (empty for MVP)
    const merkleProof = merkleTree.getProof(i);

    const voteTx = await castCompressedVote(
      program,
      electionPda,
      voter,
      attestation.publicKey,
      choice,
      i, // leaf index
      registeredAt,
      merkleProof
    );

    console.log(`  ✅ Voter ${i + 1} voted for "${candidates[choice]}"`);
  }

  // 5. Display results
  const finalElection = await program.account.election.fetch(electionPda);
  console.log("\n📊 Election Results:");

  for (let i = 0; i < candidates.length; i++) {
    const votes = finalElection.voteCounts[i].toNumber();
    console.log(`  ${candidates[i]}: ${votes} vote(s)`);
  }

  console.log(`  Total votes: ${finalElection.totalVotes}\n`);

  // 6. Calculate and display cost savings
  console.log("💰 Cost Analysis:");
  const savings = calculateCompressionSavings(10000);

  console.log(`  For ${savings.numVoters} voters:`);
  console.log(`    Legacy mode: ${savings.legacy.total.toFixed(4)} SOL`);
  console.log(`    Compressed: ${savings.compressed.total.toFixed(6)} SOL`);
  console.log(`    Savings: ${savings.savings.total.toFixed(4)} SOL (${savings.savings.percentage.toFixed(1)}%)`);
  console.log(`    Cost reduction: ${savings.savings.costReduction.toFixed(0)}x cheaper\n`);

  console.log("✅ Example completed successfully!");
}

// Example usage (uncomment to run):
/*
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { MplGovMicro } from "../../target/types/mpl_gov_micro";

async function main() {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MplGovMicro as Program<MplGovMicro>;

  await runCompressedElectionExample(program, provider);
}

main().catch(console.error);
*/

export { runCompressedElectionExample };
