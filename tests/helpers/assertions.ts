import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { TestEnvironment } from "./setup";

// Custom assertion functions for common test patterns

/**
 * Assert that a token account has the expected balance
 */
export async function assertTokenBalance(
  testEnv: TestEnvironment,
  tokenAccount: PublicKey,
  expectedAmount: number | string,
  tolerance: number = 0
): Promise<void> {
  const balance = await testEnv.connection.getTokenAccountBalance(tokenAccount);
  const actual = parseInt(balance.value.amount);
  const expected = typeof expectedAmount === 'string' ? parseInt(expectedAmount) : expectedAmount;
  
  if (tolerance === 0) {
    expect(actual).to.equal(expected, `Expected balance ${expected}, got ${actual}`);
  } else {
    expect(Math.abs(actual - expected)).to.be.lessThanOrEqual(
      tolerance, 
      `Expected balance ${expected} ± ${tolerance}, got ${actual}`
    );
  }
}

/**
 * Assert that a user has the expected grow power
 */
export async function assertUserGrowPower(
  testEnv: TestEnvironment,
  userStatePda: PublicKey,
  expectedGrowPower: number
): Promise<void> {
  const userState = await testEnv.program.account.userState.fetch(userStatePda);
  expect(userState.totalGrowPower.toNumber()).to.equal(
    expectedGrowPower,
    `Expected grow power ${expectedGrowPower}, got ${userState.totalGrowPower.toNumber()}`
  );
}

/**
 * Assert that a farm space has the expected properties
 */
export async function assertFarmSpace(
  testEnv: TestEnvironment,
  farmSpacePda: PublicKey,
  expectedLevel: number,
  expectedCapacity: number
): Promise<void> {
  const farmSpace = await testEnv.program.account.farmSpace.fetch(farmSpacePda);
  expect(farmSpace.level).to.equal(expectedLevel, `Expected level ${expectedLevel}, got ${farmSpace.level}`);
  expect(farmSpace.capacity).to.equal(expectedCapacity, `Expected capacity ${expectedCapacity}, got ${farmSpace.capacity}`);
}

/**
 * Assert that the global stats match expected values
 */
export async function assertGlobalStats(
  testEnv: TestEnvironment,
  expectedTotalGrowPower?: number
): Promise<void> {
  const globalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
  
  if (expectedTotalGrowPower !== undefined) {
    expect(globalStats.totalGrowPower.toNumber()).to.equal(
      expectedTotalGrowPower,
      `Expected total grow power ${expectedTotalGrowPower}, got ${globalStats.totalGrowPower.toNumber()}`
    );
  }
  
  // Note: totalUsers is not tracked in GlobalStats
  // User count tracking would need to be implemented separately if needed
}

/**
 * Assert that a transaction was successful
 */
export async function assertTransactionSuccess(
  testEnv: TestEnvironment,
  signature: string
): Promise<void> {
  const tx = await testEnv.connection.getTransaction(signature, {
    commitment: "confirmed"
  });
  expect(tx).to.not.be.null;
  expect(tx?.meta?.err).to.be.null;
}

/**
 * Assert that supply cap is not exceeded
 */
export async function assertSupplyCapCompliance(
  testEnv: TestEnvironment,
  maxSupply: number = 120_000_000_000_000
): Promise<void> {
  const config = await testEnv.program.account.config.fetch(testEnv.pdas.configPda);
  const currentSupply = config.totalSupplyMinted.toNumber();
  
  expect(currentSupply).to.be.lessThanOrEqual(
    maxSupply,
    `Supply ${currentSupply} exceeds cap ${maxSupply}`
  );
}

/**
 * Assert that referral relationships are correctly established
 */
export async function assertReferralRelationship(
  testEnv: TestEnvironment,
  referredUserPda: PublicKey,
  expectedReferrer: PublicKey
): Promise<void> {
  const userState = await testEnv.program.account.userState.fetch(referredUserPda);
  expect(userState.referrer?.toString()).to.equal(
    expectedReferrer.toString(),
    `Expected referrer ${expectedReferrer.toString()}, got ${userState.referrer?.toString()}`
  );
}

/**
 * Assert that account exists
 */
export async function assertAccountExists(
  testEnv: TestEnvironment,
  accountPda: PublicKey,
  accountType: string
): Promise<void> {
  try {
    const accountInfo = await testEnv.connection.getAccountInfo(accountPda);
    expect(accountInfo, `${accountType} account should exist at ${accountPda.toString()}`).to.not.be.null;
  } catch (error) {
    throw new Error(`Failed to fetch ${accountType} account: ${error}`);
  }
}

/**
 * Assert that a value is within a percentage range
 */
export function assertWithinPercentage(
  actual: number,
  expected: number,
  percentageTolerance: number,
  message?: string
): void {
  const tolerance = expected * (percentageTolerance / 100);
  const diff = Math.abs(actual - expected);
  expect(diff).to.be.lessThanOrEqual(
    tolerance,
    message || `Expected ${actual} to be within ${percentageTolerance}% of ${expected}`
  );
}

/**
 * Assert that economic calculations are consistent
 */
export async function assertEconomicConsistency(
  testEnv: TestEnvironment
): Promise<void> {
  const config = await testEnv.program.account.config.fetch(testEnv.pdas.configPda);
  const globalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
  
  // Verify that supply tracking is consistent
  const mintInfo = await testEnv.connection.getTokenSupply(testEnv.pdas.rewardMintPda);
  expect(config.totalSupplyMinted.toString()).to.equal(
    mintInfo.value.amount,
    "Config supply tracking should match actual mint supply"
  );
}