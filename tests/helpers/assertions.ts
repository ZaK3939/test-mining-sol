import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";

export class GameAssertions {
  /**
   * Assert that a token balance matches expected amount within tolerance
   */
  static async assertTokenBalance(
    connection: anchor.web3.Connection,
    tokenAccount: PublicKey,
    expectedAmount: anchor.BN,
    tolerance: anchor.BN = new anchor.BN(0),
    message?: string
  ): Promise<void> {
    const balance = await connection.getTokenAccountBalance(tokenAccount);
    const actualAmount = new anchor.BN(balance.value.amount);
    const diff = actualAmount.sub(expectedAmount).abs();
    
    expect(diff.lte(tolerance), 
      `${message || 'Token balance mismatch'}: Expected ${expectedAmount.toString()}, ` +
      `got ${actualAmount.toString()}, difference ${diff.toString()}`
    ).to.be.true;
  }

  /**
   * Assert that a token balance increased by expected amount
   */
  static async assertTokenBalanceIncrease(
    connection: anchor.web3.Connection,
    tokenAccount: PublicKey,
    initialBalance: anchor.BN,
    expectedIncrease: anchor.BN,
    tolerance: anchor.BN = new anchor.BN(0),
    message?: string
  ): Promise<void> {
    const currentBalance = await connection.getTokenAccountBalance(tokenAccount);
    const actualBalance = new anchor.BN(currentBalance.value.amount);
    const actualIncrease = actualBalance.sub(initialBalance);
    const diff = actualIncrease.sub(expectedIncrease).abs();
    
    expect(diff.lte(tolerance),
      `${message || 'Token balance increase mismatch'}: Expected increase ${expectedIncrease.toString()}, ` +
      `got ${actualIncrease.toString()}, difference ${diff.toString()}`
    ).to.be.true;
  }

  /**
   * Assert referral reward percentages
   */
  static assertReferralReward(
    baseReward: anchor.BN,
    actualReward: anchor.BN,
    percentage: number,
    tolerance: anchor.BN = new anchor.BN(1),
    message?: string
  ): void {
    const expectedReward = baseReward.muln(percentage).divn(100);
    const diff = actualReward.sub(expectedReward).abs();
    
    expect(diff.lte(tolerance),
      `${message || `${percentage}% referral reward mismatch`}: Expected ${expectedReward.toString()}, ` +
      `got ${actualReward.toString()}, difference ${diff.toString()}`
    ).to.be.true;
  }

  /**
   * Assert user state properties
   */
  static assertUserState(
    userState: any,
    expected: {
      owner?: PublicKey;
      totalGrowPower?: anchor.BN;
      hasFarmSpace?: boolean;
      referrer?: PublicKey | null;
      pendingReferralRewards?: anchor.BN;
    }
  ): void {
    if (expected.owner) {
      expect(userState.owner.toString()).to.equal(expected.owner.toString(), "User state owner mismatch");
    }
    
    if (expected.totalGrowPower !== undefined) {
      expect(userState.totalGrowPower.toString()).to.equal(expected.totalGrowPower.toString(), "User grow power mismatch");
    }
    
    if (expected.hasFarmSpace !== undefined) {
      expect(userState.hasFarmSpace).to.equal(expected.hasFarmSpace, "User farm space ownership mismatch");
    }
    
    if (expected.referrer !== undefined) {
      if (expected.referrer === null) {
        expect(userState.referrer).to.be.null;
      } else {
        expect(userState.referrer.toString()).to.equal(expected.referrer.toString(), "User referrer mismatch");
      }
    }
    
    if (expected.pendingReferralRewards !== undefined) {
      expect(userState.pendingReferralRewards.toString()).to.equal(
        expected.pendingReferralRewards.toString(), 
        "User pending referral rewards mismatch"
      );
    }
  }

  /**
   * Assert farm space properties
   */
  static assertFarmSpace(
    farmSpace: any,
    expected: {
      owner?: PublicKey;
      level?: number;
      capacity?: number;
      seedCount?: number;
      totalGrowPower?: anchor.BN;
      upgradeStartTime?: anchor.BN;
      upgradeTargetLevel?: number;
    }
  ): void {
    if (expected.owner) {
      expect(farmSpace.owner.toString()).to.equal(expected.owner.toString(), "Farm space owner mismatch");
    }
    
    if (expected.level !== undefined) {
      expect(farmSpace.level).to.equal(expected.level, "Farm space level mismatch");
    }
    
    if (expected.capacity !== undefined) {
      expect(farmSpace.capacity).to.equal(expected.capacity, "Farm space capacity mismatch");
    }
    
    if (expected.seedCount !== undefined) {
      expect(farmSpace.seedCount).to.equal(expected.seedCount, "Farm space seed count mismatch");
    }
    
    if (expected.totalGrowPower !== undefined) {
      expect(farmSpace.totalGrowPower.toString()).to.equal(
        expected.totalGrowPower.toString(), 
        "Farm space grow power mismatch"
      );
    }
    
    if (expected.upgradeStartTime !== undefined) {
      expect(farmSpace.upgradeStartTime.toString()).to.equal(
        expected.upgradeStartTime.toString(), 
        "Farm space upgrade start time mismatch"
      );
    }
    
    if (expected.upgradeTargetLevel !== undefined) {
      expect(farmSpace.upgradeTargetLevel).to.equal(
        expected.upgradeTargetLevel, 
        "Farm space upgrade target level mismatch"
      );
    }
  }

  /**
   * Assert global stats properties
   */
  static assertGlobalStats(
    globalStats: any,
    expected: {
      totalGrowPower?: anchor.BN;
      totalFarmSpaces?: anchor.BN;
      totalSupply?: anchor.BN;
      currentRewardsPerSecond?: anchor.BN;
    }
  ): void {
    if (expected.totalGrowPower !== undefined) {
      expect(globalStats.totalGrowPower.toString()).to.equal(
        expected.totalGrowPower.toString(), 
        "Global total grow power mismatch"
      );
    }
    
    if (expected.totalFarmSpaces !== undefined) {
      expect(globalStats.totalFarmSpaces.toString()).to.equal(
        expected.totalFarmSpaces.toString(), 
        "Global total farm spaces mismatch"
      );
    }
    
    if (expected.totalSupply !== undefined) {
      expect(globalStats.totalSupply.toString()).to.equal(
        expected.totalSupply.toString(), 
        "Global total supply mismatch"
      );
    }
    
    if (expected.currentRewardsPerSecond !== undefined) {
      expect(globalStats.currentRewardsPerSecond.toString()).to.equal(
        expected.currentRewardsPerSecond.toString(), 
        "Global current rewards per second mismatch"
      );
    }
  }

  /**
   * Assert config properties
   */
  static assertConfig(
    config: any,
    expected: {
      baseRate?: anchor.BN;
      halvingInterval?: anchor.BN;
      admin?: PublicKey;
      treasury?: PublicKey;
      protocolReferralAddress?: PublicKey;
      seedPackCost?: anchor.BN;
      farmSpaceCostSol?: anchor.BN;
    }
  ): void {
    if (expected.baseRate !== undefined) {
      expect(config.baseRate.toString()).to.equal(expected.baseRate.toString(), "Config base rate mismatch");
    }
    
    if (expected.halvingInterval !== undefined) {
      expect(config.halvingInterval.toString()).to.equal(expected.halvingInterval.toString(), "Config halving interval mismatch");
    }
    
    if (expected.admin) {
      expect(config.admin.toString()).to.equal(expected.admin.toString(), "Config admin mismatch");
    }
    
    if (expected.treasury) {
      expect(config.treasury.toString()).to.equal(expected.treasury.toString(), "Config treasury mismatch");
    }
    
    if (expected.protocolReferralAddress) {
      expect(config.protocolReferralAddress.toString()).to.equal(
        expected.protocolReferralAddress.toString(), 
        "Config protocol referral address mismatch"
      );
    }
    
    if (expected.seedPackCost !== undefined) {
      expect(config.seedPackCost.toString()).to.equal(expected.seedPackCost.toString(), "Config seed pack cost mismatch");
    }
    
    if (expected.farmSpaceCostSol !== undefined) {
      expect(config.farmSpaceCostSol.toString()).to.equal(
        expected.farmSpaceCostSol.toString(), 
        "Config farm space cost mismatch"
      );
    }
  }

  /**
   * Assert that transaction failed with specific error
   */
  static async assertTransactionError(
    transactionPromise: Promise<any>,
    expectedError: string | RegExp,
    message?: string
  ): Promise<void> {
    try {
      await transactionPromise;
      throw new Error(`${message || 'Expected transaction to fail'}, but it succeeded`);
    } catch (error: any) {
      if (typeof expectedError === 'string') {
        expect(error.message).to.include(expectedError, message || `Expected error containing "${expectedError}"`);
      } else {
        expect(error.message).to.match(expectedError, message || `Expected error matching ${expectedError}`);
      }
    }
  }

  /**
   * Assert halving mechanism
   */
  static assertHalving(
    initialRate: anchor.BN,
    finalRate: anchor.BN,
    halvingCount: number = 1,
    message?: string
  ): void {
    let expectedRate = initialRate;
    for (let i = 0; i < halvingCount; i++) {
      expectedRate = expectedRate.divn(2);
    }
    
    expect(finalRate.toString()).to.equal(
      expectedRate.toString(),
      `${message || 'Halving calculation mismatch'}: After ${halvingCount} halving(s), ` +
      `expected rate ${expectedRate.toString()}, got ${finalRate.toString()}`
    );
  }

  /**
   * Assert reward calculation
   */
  static assertRewardCalculation(
    userGrowPower: anchor.BN,
    globalGrowPower: anchor.BN,
    baseRate: anchor.BN,
    timeElapsed: anchor.BN,
    actualReward: anchor.BN,
    tolerance: anchor.BN = new anchor.BN(1),
    message?: string
  ): void {
    // Expected reward = (userGrowPower / globalGrowPower) * baseRate * timeElapsed
    const expectedReward = userGrowPower
      .mul(baseRate)
      .mul(timeElapsed)
      .div(globalGrowPower);
    
    const diff = actualReward.sub(expectedReward).abs();
    
    expect(diff.lte(tolerance),
      `${message || 'Reward calculation mismatch'}: Expected ${expectedReward.toString()}, ` +
      `got ${actualReward.toString()}, difference ${diff.toString()}`
    ).to.be.true;
  }
}

// Custom Chai matchers for better test readability
export function addCustomMatchers(): void {
  // Add custom matchers if needed in the future
}