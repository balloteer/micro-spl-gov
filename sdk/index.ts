/**
 * micro-spl-gov TypeScript SDK
 *
 * This module provides utilities for interacting with the micro-spl-gov program,
 * including zkCompression features for cost-efficient voter management.
 */

// Export all compression utilities
export * from "./compression";

// Re-export common Anchor/Solana types for convenience
export { PublicKey, Keypair, Connection, Transaction } from "@solana/web3.js";
export { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
