import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";

export interface TestScenario {
  name: string;
  description: string;
  setup: () => Promise<void>;
  execute: () => Promise<any>;
  verify: (result: any) => Promise<void>;
}

export class TestDataGenerator {
  /**
   * Generate test invite codes
   */
  static generateInviteCode(): [number] {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    let result = '';
    for (let i = 0; i < 8; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return Buffer.from(result, 'utf8') as any;
  }

  /**
   * Generate referral chain test data
   */
  static generateReferralChain(depth: number): {
    users: Keypair[];
    referralRelationships: { user: number; referrer: number | null }[];
  } {
    const users: Keypair[] = [];
    const referralRelationships: { user: number; referrer: number | null }[] = [];

    for (let i = 0; i < depth; i++) {
      users.push(Keypair.generate());
      referralRelationships.push({
        user: i,
        referrer: i === 0 ? null : i - 1, // Each user refers the next one
      });
    }

    return { users, referralRelationships };
  }

  /**
   * Generate farm upgrade scenarios
   */
  static generateFarmUpgradeScenarios(): Array<{
    fromLevel: number;
    toLevel: number;
    cost: anchor.BN;
    newCapacity: number;
    cooldownHours: number;
  }> {
    return [
      {
        fromLevel: 1,
        toLevel: 2,
        cost: new anchor.BN(3500 * 1_000_000),
        newCapacity: 8,
        cooldownHours: 24,
      },
      {
        fromLevel: 2,
        toLevel: 3,
        cost: new anchor.BN(18000 * 1_000_000),
        newCapacity: 12,
        cooldownHours: 24,
      },
      {
        fromLevel: 3,
        toLevel: 4,
        cost: new anchor.BN(20000 * 1_000_000),
        newCapacity: 16,
        cooldownHours: 24,
      },
      {
        fromLevel: 4,
        toLevel: 5,
        cost: new anchor.BN(25000 * 1_000_000),
        newCapacity: 20,
        cooldownHours: 24,
      },
    ];
  }

  /**
   * Generate seed types and their properties
   */
  static getSeedTestData(): Array<{
    name: string;
    type: number;
    growPower: anchor.BN;
    probability: number;
  }> {
    return [
      { name: "Seed1", type: 0, growPower: new anchor.BN(100), probability: 42.23 },
      { name: "Seed2", type: 1, growPower: new anchor.BN(180), probability: 24.44 },
      { name: "Seed3", type: 2, growPower: new anchor.BN(420), probability: 13.33 },
      { name: "Seed4", type: 3, growPower: new anchor.BN(720), probability: 8.33 },
      { name: "Seed5", type: 4, growPower: new anchor.BN(1000), probability: 5.56 },
      { name: "Seed6", type: 5, growPower: new anchor.BN(5000), probability: 3.33 },
      { name: "Seed7", type: 6, growPower: new anchor.BN(15000), probability: 1.33 },
      { name: "Seed8", type: 7, growPower: new anchor.BN(30000), probability: 0.89 },
      { name: "Seed9", type: 8, growPower: new anchor.BN(60000), probability: 0.56 },
    ];
  }

  /**
   * Generate halving test scenarios
   */
  static generateHalvingScenarios(): Array<{
    name: string;
    initialRate: anchor.BN;
    halvingInterval: anchor.BN;
    timeElapsed: anchor.BN;
    expectedHalvings: number;
    expectedFinalRate: anchor.BN;
  }> {
    const baseRate = new anchor.BN(100);
    const sixDays = new anchor.BN(6 * 24 * 60 * 60);

    return [
      {
        name: "No halving within interval",
        initialRate: baseRate,
        halvingInterval: sixDays,
        timeElapsed: new anchor.BN(3 * 24 * 60 * 60), // 3 days
        expectedHalvings: 0,
        expectedFinalRate: baseRate,
      },
      {
        name: "One halving after 6 days",
        initialRate: baseRate,
        halvingInterval: sixDays,
        timeElapsed: new anchor.BN(7 * 24 * 60 * 60), // 7 days
        expectedHalvings: 1,
        expectedFinalRate: baseRate.divn(2),
      },
      {
        name: "Two halvings after 12 days",
        initialRate: baseRate,
        halvingInterval: sixDays,
        timeElapsed: new anchor.BN(13 * 24 * 60 * 60), // 13 days
        expectedHalvings: 2,
        expectedFinalRate: baseRate.divn(4),
      },
      {
        name: "Three halvings after 18 days",
        initialRate: baseRate,
        halvingInterval: sixDays,
        timeElapsed: new anchor.BN(19 * 24 * 60 * 60), // 19 days
        expectedHalvings: 3,
        expectedFinalRate: baseRate.divn(8),
      },
    ];
  }

  /**
   * Generate error test scenarios
   */
  static generateErrorScenarios(): Array<{
    name: string;
    description: string;
    expectedError: string | RegExp;
    setup: () => Promise<any>;
  }> {
    return [
      {
        name: "Unauthorized admin action",
        description: "Non-admin user tries to perform admin-only action",
        expectedError: /Unauthorized|Access denied/i,
        setup: async () => {
          // Setup scenario where non-admin tries admin action
          return { scenario: "unauthorized_admin" };
        },
      },
      {
        name: "Insufficient funds for farm purchase",
        description: "User with insufficient SOL tries to buy farm space",
        expectedError: /Insufficient funds|Not enough SOL/i,
        setup: async () => {
          return { scenario: "insufficient_funds" };
        },
      },
      {
        name: "Double farm space purchase",
        description: "User tries to buy farm space twice",
        expectedError: /Already owns farm space|Duplicate farm/i,
        setup: async () => {
          return { scenario: "double_farm_purchase" };
        },
      },
      {
        name: "Claim rewards without farm space",
        description: "User tries to claim rewards without owning a farm space",
        expectedError: /No farm space|Must own farm/i,
        setup: async () => {
          return { scenario: "no_farm_space" };
        },
      },
      {
        name: "Upgrade farm space without funds",
        description: "User tries to upgrade farm space without sufficient WEED tokens",
        expectedError: /Insufficient WEED|Not enough tokens/i,
        setup: async () => {
          return { scenario: "insufficient_weed" };
        },
      },
    ];
  }

  /**
   * Generate performance test scenarios
   */
  static generatePerformanceScenarios(): Array<{
    name: string;
    description: string;
    userCount: number;
    operationsPerUser: number;
    expectedMaxTime: number; // milliseconds
    operation: string;
  }> {
    return [
      {
        name: "Mass user initialization",
        description: "Initialize many users simultaneously",
        userCount: 50,
        operationsPerUser: 1,
        expectedMaxTime: 30000, // 30 seconds
        operation: "init_user",
      },
      {
        name: "Concurrent farm space purchases",
        description: "Multiple users buy farm spaces at the same time",
        userCount: 20,
        operationsPerUser: 1,
        expectedMaxTime: 15000, // 15 seconds
        operation: "buy_farm_space",
      },
      {
        name: "Mass reward claiming",
        description: "Many users claim rewards simultaneously",
        userCount: 30,
        operationsPerUser: 1,
        expectedMaxTime: 20000, // 20 seconds
        operation: "claim_reward",
      },
      {
        name: "Seed pack purchasing spree",
        description: "Multiple users purchase seed packs rapidly",
        userCount: 15,
        operationsPerUser: 5,
        expectedMaxTime: 25000, // 25 seconds
        operation: "purchase_seed_pack",
      },
    ];
  }

  /**
   * Generate stress test data
   */
  static generateStressTestData(): {
    maxUsers: number;
    maxFarmSpaces: number;
    maxSeedsPerUser: number;
    maxReferralDepth: number;
    maxConcurrentOperations: number;
  } {
    return {
      maxUsers: 100,
      maxFarmSpaces: 100,
      maxSeedsPerUser: 50,
      maxReferralDepth: 10,
      maxConcurrentOperations: 20,
    };
  }

  /**
   * Generate test token amounts
   */
  static generateTokenAmounts(): {
    small: anchor.BN;
    medium: anchor.BN;
    large: anchor.BN;
    max: anchor.BN;
  } {
    return {
      small: new anchor.BN(100 * 1_000_000), // 100 WEED
      medium: new anchor.BN(10_000 * 1_000_000), // 10,000 WEED
      large: new anchor.BN(100_000 * 1_000_000), // 100,000 WEED
      max: new anchor.BN(1_000_000 * 1_000_000), // 1,000,000 WEED
    };
  }
}

export class MockDataProvider {
  /**
   * Create mock user with specific properties
   */
  static createMockUser(overrides: {
    totalGrowPower?: anchor.BN;
    hasFarmSpace?: boolean;
    referrer?: PublicKey | null;
    pendingReferralRewards?: anchor.BN;
  } = {}): any {
    return {
      owner: Keypair.generate().publicKey,
      totalGrowPower: overrides.totalGrowPower || new anchor.BN(0),
      lastHarvestTime: new anchor.BN(Date.now() / 1000),
      hasFarmSpace: overrides.hasFarmSpace !== undefined ? overrides.hasFarmSpace : false,
      referrer: overrides.referrer !== undefined ? overrides.referrer : null,
      pendingReferralRewards: overrides.pendingReferralRewards || new anchor.BN(0),
      reserve: new Array(32).fill(0),
    };
  }

  /**
   * Create mock farm space with specific properties
   */
  static createMockFarmSpace(overrides: {
    level?: number;
    capacity?: number;
    seedCount?: number;
    totalGrowPower?: anchor.BN;
    upgradeStartTime?: anchor.BN;
    upgradeTargetLevel?: number;
  } = {}): any {
    const level = overrides.level || 1;
    return {
      owner: Keypair.generate().publicKey,
      level,
      capacity: overrides.capacity || (level * 4),
      seedCount: overrides.seedCount || 0,
      totalGrowPower: overrides.totalGrowPower || new anchor.BN(0),
      upgradeStartTime: overrides.upgradeStartTime || new anchor.BN(0),
      upgradeTargetLevel: overrides.upgradeTargetLevel || 0,
      reserve: new Array(32).fill(0),
    };
  }

  /**
   * Create mock global stats
   */
  static createMockGlobalStats(overrides: {
    totalGrowPower?: anchor.BN;
    totalFarmSpaces?: anchor.BN;
    currentRewardsPerSecond?: anchor.BN;
  } = {}): any {
    return {
      totalGrowPower: overrides.totalGrowPower || new anchor.BN(0),
      totalFarmSpaces: overrides.totalFarmSpaces || new anchor.BN(0),
      totalSupply: new anchor.BN(1_000_000_000 * 1_000_000), // 1B WEED
      currentRewardsPerSecond: overrides.currentRewardsPerSecond || new anchor.BN(100),
      lastUpdateTime: new anchor.BN(Date.now() / 1000),
      reserve: new Array(32).fill(0),
    };
  }
}