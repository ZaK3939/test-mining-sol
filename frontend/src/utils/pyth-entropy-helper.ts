/**
 * Pyth Entropy Integration Helper
 * Provides utilities for integrating Pyth Entropy in the frontend
 */

import { PublicKey, Connection } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// Pyth Entropy Program ID (replace with actual mainnet ID when available)
export const PYTH_ENTROPY_PROGRAM_ID = new PublicKey('4HBnqo3Q7zNtLQd5QHGX1ZMp8R4jKPH6sWGKyU5A7zFT');

// Pyth Entropy Provider addresses (these would be provided by Pyth)
export const PYTH_ENTROPY_PROVIDER = new PublicKey('EntropyProvider1111111111111111111111111111'); // Placeholder

/**
 * Generate cryptographically secure user entropy seed
 * This provides additional randomness that combines with Pyth's oracle entropy
 */
export function generateUserEntropySeed(): BN {
  // Use Web Crypto API for cryptographically secure random generation
  const buffer = new Uint8Array(8);
  crypto.getRandomValues(buffer);
  
  // Convert to BN for Anchor compatibility
  const hex = Array.from(buffer)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  return new BN(hex, 16);
}

/**
 * Derive entropy request PDA
 * This calculates the expected address for the entropy request account
 */
export function deriveEntropyRequestPDA(
  user: PublicKey,
  sequence: BN | number
): [PublicKey, number] {
  const sequenceBuffer = typeof sequence === 'number' 
    ? Buffer.from(new BN(sequence).toArray('le', 8))
    : Buffer.from(sequence.toArray('le', 8));
    
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from('entropy_request'),
      user.toBuffer(),
      sequenceBuffer,
    ],
    PYTH_ENTROPY_PROGRAM_ID
  );
}

/**
 * Check if entropy result is ready
 * Polls the entropy request account to see if Pyth has provided the result
 */
export async function checkEntropyResult(
  connection: Connection,
  entropyRequestAccount: PublicKey
): Promise<{
  isReady: boolean;
  sequence?: number;
  randomValue?: BN;
  error?: string;
}> {
  try {
    const accountInfo = await connection.getAccountInfo(entropyRequestAccount);
    
    if (!accountInfo) {
      return { isReady: false, error: 'Entropy request account not found' };
    }
    
    if (accountInfo.data.length < 16) {
      return { isReady: false, error: 'Entropy result not ready yet' };
    }
    
    // Parse the entropy result data structure
    // This is a simplified parsing - actual implementation would follow Pyth format
    const sequence = accountInfo.data.readBigUInt64LE(0);
    const randomValue = accountInfo.data.readBigUInt64LE(8);
    
    // Check if entropy is ready (non-zero random value)
    const isReady = randomValue !== 0n;
    
    return {
      isReady,
      sequence: Number(sequence),
      randomValue: isReady ? new BN(randomValue.toString()) : undefined,
    };
  } catch (error) {
    return { isReady: false, error: `Failed to check entropy: ${error}` };
  }
}

/**
 * Wait for entropy result with polling
 * Continuously checks until the entropy result is available
 */
export async function waitForEntropyResult(
  connection: Connection,
  entropyRequestAccount: PublicKey,
  options: {
    maxAttempts?: number;
    pollIntervalMs?: number;
    timeoutMs?: number;
  } = {}
): Promise<{
  sequence: number;
  randomValue: BN;
}> {
  const {
    maxAttempts = 60,
    pollIntervalMs = 1000,
    timeoutMs = 60000,
  } = options;
  
  const startTime = Date.now();
  
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    // Check timeout
    if (Date.now() - startTime > timeoutMs) {
      throw new Error(`Entropy result timeout after ${timeoutMs}ms`);
    }
    
    const result = await checkEntropyResult(connection, entropyRequestAccount);
    
    if (result.isReady && result.randomValue && result.sequence !== undefined) {
      return {
        sequence: result.sequence,
        randomValue: result.randomValue,
      };
    }
    
    if (result.error && !result.error.includes('not ready')) {
      throw new Error(`Entropy error: ${result.error}`);
    }
    
    // Wait before next attempt
    await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
  }
  
  throw new Error(`Entropy result not ready after ${maxAttempts} attempts`);
}

/**
 * Calculate estimated wait time for entropy result
 * Provides user feedback on expected wait time
 */
export function getEstimatedEntropyWaitTime(): {
  minSeconds: number;
  maxSeconds: number;
  averageSeconds: number;
} {
  // Pyth Entropy typically provides results within seconds
  // These are conservative estimates
  return {
    minSeconds: 5,
    maxSeconds: 30,
    averageSeconds: 10,
  };
}

/**
 * Validate entropy request account
 * Ensures the account is properly formatted and owned by Pyth
 */
export async function validateEntropyRequestAccount(
  connection: Connection,
  entropyRequestAccount: PublicKey,
  expectedUser: PublicKey,
  expectedSequence: BN | number
): Promise<boolean> {
  try {
    const accountInfo = await connection.getAccountInfo(entropyRequestAccount);
    
    if (!accountInfo) {
      return false;
    }
    
    // Check owner is Pyth Entropy program
    if (!accountInfo.owner.equals(PYTH_ENTROPY_PROGRAM_ID)) {
      return false;
    }
    
    // Verify the PDA derivation is correct
    const [expectedPDA] = deriveEntropyRequestPDA(expectedUser, expectedSequence);
    
    return entropyRequestAccount.equals(expectedPDA);
  } catch (error) {
    console.error('Error validating entropy request account:', error);
    return false;
  }
}

/**
 * Create instruction data for purchase seed pack with entropy
 */
export interface PurchaseSeedPackWithEntropyParams {
  quantity: number;
  userEntropySeed: BN;
}

/**
 * Helper to prepare accounts for seed pack purchase with Pyth Entropy
 */
export function prepareSeedPackPurchaseAccounts(
  user: PublicKey,
  seedPackPDA: PublicKey,
  entropySequence: BN
): {
  entropyProvider: PublicKey;
  entropyRequest: PublicKey;
  pythEntropyProgram: PublicKey;
} {
  const [entropyRequest] = deriveEntropyRequestPDA(user, entropySequence);
  
  return {
    entropyProvider: PYTH_ENTROPY_PROVIDER,
    entropyRequest,
    pythEntropyProgram: PYTH_ENTROPY_PROGRAM_ID,
  };
}

/**
 * Format entropy result for display
 */
export function formatEntropyValue(value: BN): string {
  return `0x${value.toString(16).padStart(16, '0')}`;
}

/**
 * Estimate gas/compute units for entropy operations
 */
export function getEntropyComputeUnits(): {
  purchase: number;
  open: number;
} {
  return {
    purchase: 200000, // Purchase + entropy request
    open: 150000,     // Open + entropy validation
  };
}