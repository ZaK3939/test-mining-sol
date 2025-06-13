/**
 * Test Data Factory
 * Provides realistic test data generation for all game entities
 */

import { Keypair, PublicKey } from '@solana/web3.js';
// import type { SeedType } from '../../types/program-types';

// Temporary seed type definition
enum SeedType {
  Common = 1,
  Uncommon = 2,
  Rare = 3,
  Epic = 4,
  Legendary = 5,
}

// =============== USER DATA FACTORIES ===============

export interface TestUser {
  keypair: Keypair;
  publicKey: PublicKey;
  name: string;
  index: number;
  referrer?: PublicKey;
}

export interface TestUserState {
  owner: PublicKey;
  totalGrowPower: number;
  lastHarvestTime: number;
  hasFarmSpace: boolean;
  referrer?: PublicKey;
  pendingReferralRewards: number;
}

export interface TestFarmSpace {
  owner: PublicKey;
  level: number;
  capacity: number;
  seedCount: number;
  totalGrowPower: number;
  upgradeStartTime: number;
  upgradeTargetLevel: number;
}

export interface TestSeed {
  id: number;
  owner: PublicKey;
  seedType: number;
  growPower: number;
  isPlanted: boolean;
  plantedFarmSpace?: PublicKey;
  createdAt: number;
}

export interface TestSeedPack {
  id: number;
  purchaser: PublicKey;
  purchasedAt: number;
  costPaid: number;
  isOpened: boolean;
  randomSeed: number;
}

export interface TestInviteCode {
  code: string;
  inviter: PublicKey;
  invitesUsed: number;
  inviteLimit: number;
  createdAt: number;
}

export class TestDataFactory {
  private static userCounter = 0;
  private static seedCounter = 0;
  private static packCounter = 0;

  // =============== USER FACTORIES ===============

  static createTestUser(name?: string): TestUser {
    const index = ++this.userCounter;
    const keypair = Keypair.generate();
    
    return {
      keypair,
      publicKey: keypair.publicKey,
      name: name || `TestUser${index}`,
      index,
    };
  }

  static createMultipleTestUsers(count: number, namePrefix = 'User'): TestUser[] {
    return Array.from({ length: count }, (_, i) => 
      this.createTestUser(`${namePrefix}${i + 1}`)
    );
  }

  static createUserState(overrides: Partial<TestUserState> = {}): TestUserState {
    const defaultUser = this.createTestUser();
    
    return {
      owner: defaultUser.publicKey,
      totalGrowPower: 100,
      lastHarvestTime: Date.now() - 3600000, // 1 hour ago
      hasFarmSpace: true,
      pendingReferralRewards: 0,
      ...overrides,
    };
  }

  // =============== FARM SPACE FACTORIES ===============

  static createFarmSpace(level: number = 1, overrides: Partial<TestFarmSpace> = {}): TestFarmSpace {
    const owner = overrides.owner || this.createTestUser().publicKey;
    const capacity = this.getFarmCapacityForLevel(level);
    const seedCount = Math.min(level, capacity); // Start with some seeds
    
    return {
      owner,
      level,
      capacity,
      seedCount,
      totalGrowPower: seedCount * 100, // Basic grow power per seed
      upgradeStartTime: 0,
      upgradeTargetLevel: 0,
      ...overrides,
    };
  }

  static createUpgradingFarmSpace(currentLevel: number, targetLevel: number): TestFarmSpace {
    return this.createFarmSpace(currentLevel, {
      upgradeStartTime: Date.now() - 12 * 60 * 60 * 1000, // 12 hours ago
      upgradeTargetLevel: targetLevel,
    });
  }

  static createMaxLevelFarmSpace(): TestFarmSpace {
    return this.createFarmSpace(5, {
      seedCount: 20, // Max capacity
      totalGrowPower: 20 * 5000, // High-level seeds
    });
  }

  // =============== SEED FACTORIES ===============

  static createSeed(seedType: number = 1, overrides: Partial<TestSeed> = {}): TestSeed {
    const id = ++this.seedCounter;
    const growPower = this.getGrowPowerForSeedType(seedType);
    
    return {
      id,
      owner: this.createTestUser().publicKey,
      seedType,
      growPower,
      isPlanted: false,
      createdAt: Date.now(),
      ...overrides,
    };
  }

  static createSeedCollection(owner: PublicKey, count: number = 5): TestSeed[] {
    return Array.from({ length: count }, () => {
      // Create a mix of seed types with weighted distribution
      const seedType = this.generateRandomSeedType();
      return this.createSeed(seedType, { owner });
    });
  }

  static createPlantedSeed(farmSpace: PublicKey, seedType: number = 1): TestSeed {
    return this.createSeed(seedType, {
      isPlanted: true,
      plantedFarmSpace: farmSpace,
    });
  }

  static createHighValueSeedCollection(owner: PublicKey): TestSeed[] {
    return [
      this.createSeed(6, { owner }), // Seed6 - 5000 grow power
      this.createSeed(7, { owner }), // Seed7 - 15000 grow power
      this.createSeed(8, { owner }), // Seed8 - 30000 grow power
      this.createSeed(9, { owner }), // Seed9 - 60000 grow power
    ];
  }

  // =============== SEED PACK FACTORIES ===============

  static createSeedPack(overrides: Partial<TestSeedPack> = {}): TestSeedPack {
    const id = ++this.packCounter;
    
    return {
      id,
      purchaser: this.createTestUser().publicKey,
      purchasedAt: Date.now(),
      costPaid: 300 * 1_000_000, // 300 WEED with 6 decimals
      isOpened: false,
      randomSeed: Math.floor(Math.random() * 10000),
      ...overrides,
    };
  }

  static createMultipleSeedPacks(count: number, purchaser: PublicKey): TestSeedPack[] {
    return Array.from({ length: count }, () => 
      this.createSeedPack({ purchaser })
    );
  }

  static createOpenedSeedPack(): TestSeedPack {
    return this.createSeedPack({
      isOpened: true,
      purchasedAt: Date.now() - 60000, // 1 minute ago
    });
  }

  // =============== INVITE CODE FACTORIES ===============

  static createInviteCode(inviter: PublicKey, overrides: Partial<TestInviteCode> = {}): TestInviteCode {
    return {
      code: this.generateRandomInviteCode(),
      inviter,
      invitesUsed: 0,
      inviteLimit: 5,
      createdAt: Date.now(),
      ...overrides,
    };
  }

  static createUsedInviteCode(inviter: PublicKey, usedCount: number = 3): TestInviteCode {
    return this.createInviteCode(inviter, {
      invitesUsed: usedCount,
    });
  }

  static createFullInviteCode(inviter: PublicKey): TestInviteCode {
    return this.createInviteCode(inviter, {
      invitesUsed: 5,
      inviteLimit: 5,
    });
  }

  // =============== REFERRAL CHAIN FACTORIES ===============

  static createReferralChain(depth: number = 3): TestUser[] {
    const users = this.createMultipleTestUsers(depth);
    
    // Create referral relationships
    for (let i = 1; i < users.length; i++) {
      // Each user refers the next one
      users[i].referrer = users[i - 1].publicKey;
    }
    
    return users;
  }

  static createNetworkBuilderScenario(): {
    networkBuilder: TestUser;
    directReferrals: TestUser[];
    indirectReferrals: TestUser[];
  } {
    const networkBuilder = this.createTestUser('NetworkBuilder');
    const directReferrals = this.createMultipleTestUsers(5, 'Direct');
    const indirectReferrals = this.createMultipleTestUsers(15, 'Indirect');

    return {
      networkBuilder,
      directReferrals,
      indirectReferrals,
    };
  }

  // =============== GAME STATE SCENARIOS ===============

  static createNewPlayerScenario(): {
    user: TestUser;
    userState: TestUserState;
  } {
    const user = this.createTestUser('NewPlayer');
    const userState = this.createUserState({
      owner: user.publicKey,
      totalGrowPower: 0,
      hasFarmSpace: false,
      lastHarvestTime: 0,
    });

    return { user, userState };
  }

  static createActivePlayerScenario(): {
    user: TestUser;
    userState: TestUserState;
    farmSpace: TestFarmSpace;
    seeds: TestSeed[];
  } {
    const user = this.createTestUser('ActivePlayer');
    const farmSpace = this.createFarmSpace(2, { owner: user.publicKey });
    const seeds = this.createSeedCollection(user.publicKey, 6);
    
    // Plant some seeds
    seeds.slice(0, farmSpace.seedCount).forEach(seed => {
      seed.isPlanted = true;
      seed.plantedFarmSpace = user.publicKey; // Using user's public key as farm space identifier
    });

    const userState = this.createUserState({
      owner: user.publicKey,
      totalGrowPower: farmSpace.totalGrowPower,
      hasFarmSpace: true,
      lastHarvestTime: Date.now() - 2 * 60 * 60 * 1000, // 2 hours ago
    });

    return { user, userState, farmSpace, seeds };
  }

  static createWhalePlayerScenario(): {
    user: TestUser;
    userState: TestUserState;
    farmSpace: TestFarmSpace;
    seeds: TestSeed[];
    seedPacks: TestSeedPack[];
  } {
    const user = this.createTestUser('WhalePlayer');
    const farmSpace = this.createMaxLevelFarmSpace();
    farmSpace.owner = user.publicKey;

    const seeds = [
      ...this.createHighValueSeedCollection(user.publicKey),
      ...this.createSeedCollection(user.publicKey, 16), // Fill the rest
    ];

    const seedPacks = this.createMultipleSeedPacks(10, user.publicKey);

    const userState = this.createUserState({
      owner: user.publicKey,
      totalGrowPower: farmSpace.totalGrowPower,
      hasFarmSpace: true,
      lastHarvestTime: Date.now() - 30 * 60 * 1000, // 30 minutes ago
      pendingReferralRewards: 1500, // Substantial referral rewards
    });

    return { user, userState, farmSpace, seeds, seedPacks };
  }

  // =============== UTILITY METHODS ===============

  private static getFarmCapacityForLevel(level: number): number {
    const capacities = [0, 4, 8, 12, 16, 20]; // Index 0 unused, levels 1-5
    return capacities[Math.min(level, 5)] || 20;
  }

  private static getGrowPowerForSeedType(seedType: number): number {
    const growPowers = [0, 100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000];
    return growPowers[Math.min(seedType, 9)] || 100;
  }

  private static generateRandomSeedType(): number {
    // Weighted distribution matching the game probabilities
    const random = Math.random() * 10000;
    if (random < 4222) return 1; // 42.22%
    if (random < 6666) return 2; // 24.44%
    if (random < 7999) return 3; // 13.33%
    if (random < 8832) return 4; // 8.33%
    if (random < 9388) return 5; // 5.56%
    if (random < 9721) return 6; // 3.33%
    if (random < 9854) return 7; // 1.33%
    if (random < 9943) return 8; // 0.89%
    return 9; // 0.57%
  }

  private static generateRandomInviteCode(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    return Array.from({ length: 8 }, () => 
      chars.charAt(Math.floor(Math.random() * chars.length))
    ).join('');
  }

  // =============== RESET METHODS ===============

  static resetCounters() {
    this.userCounter = 0;
    this.seedCounter = 0;
    this.packCounter = 0;
  }

  static createCleanState() {
    this.resetCounters();
    return {
      users: [],
      farmSpaces: [],
      seeds: [],
      seedPacks: [],
      inviteCodes: [],
    };
  }
}