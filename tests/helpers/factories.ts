import * as anchor from "@coral-xyz/anchor";
import { TestEnvironment, TestUser, createUser } from "./setup";

// Test data factories for consistent test object creation

/**
 * Factory for creating test scenarios with multiple users
 */
export class TestScenarioFactory {
  constructor(private testEnv: TestEnvironment) {}

  /**
   * Create a basic user from existing keypair (for invite system tests)
   */
  async createBasicUserFromKeypair(keypair: anchor.web3.Keypair, userName: string): Promise<TestUser> {
    const user = await createUser(this.testEnv, userName, keypair);
    
    // Buy farm space (user should already be initialized via invite system)
    await this.testEnv.program.methods
      .buyFarmSpace()
      .accountsPartial({
        userState: user.userStatePda,
        config: this.testEnv.pdas.configPda,
        farmSpace: user.farmSpacePda,
        initialSeed: user.initialSeedPda,
        globalStats: this.testEnv.pdas.globalStatsPda,
        treasury: this.testEnv.pdas.treasuryPda,
        user: user.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user.keypair])
      .rpc();

    return user;
  }

  /**
   * Create a basic user scenario with initialized user and farm
   */
  async createBasicUserScenario(userName: string): Promise<TestUser> {
    const user = await createUser(this.testEnv, userName);
    
    // Initialize user
    await this.testEnv.program.methods
      .initUser(null)
      .accountsPartial({
        userState: user.userStatePda,
        user: user.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user.keypair])
      .rpc();

    // Buy farm space
    await this.testEnv.program.methods
      .buyFarmSpace()
      .accountsPartial({
        userState: user.userStatePda,
        config: this.testEnv.pdas.configPda,
        farmSpace: user.farmSpacePda,
        user: user.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user.keypair])
      .rpc();

    return user;
  }

  /**
   * Create a referral scenario with referred user
   */
  async createReferralScenario(
    referrerName: string, 
    referredName: string
  ): Promise<{ referrer: TestUser; referred: TestUser }> {
    // Create and setup referrer
    const referrer = await this.createBasicUserScenario(referrerName);
    
    // Create invite code (must be exactly 8 bytes)
    const inviteCodeString = `${referrerName.toUpperCase()}001`;
    await this.testEnv.program.methods
      .createInviteCode(createInviteCodeArray(inviteCodeString))
      .accountsPartial({
        inviteCodeAccount: referrer.inviteCodePda,
        config: this.testEnv.pdas.configPda,
        inviter: referrer.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([referrer.keypair])
      .rpc();

    // Create referred user
    const referred = await createUser(this.testEnv, referredName);
    
    // Initialize referred user with referral
    await this.testEnv.program.methods
      .initUser(referrer.keypair.publicKey)
      .accountsPartial({
        userState: referred.userStatePda,
        user: referred.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([referred.keypair])
      .rpc();

    // Buy farm space for referred user
    await this.testEnv.program.methods
      .buyFarmSpace()
      .accountsPartial({
        userState: referred.userStatePda,
        config: this.testEnv.pdas.configPda,
        farmSpace: referred.farmSpacePda,
        user: referred.keypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([referred.keypair])
      .rpc();

    return { referrer, referred };
  }

  /**
   * Create multiple users for stress testing
   */
  async createMultiUserScenario(count: number, prefix: string = "User"): Promise<TestUser[]> {
    const users: TestUser[] = [];
    
    for (let i = 0; i < count; i++) {
      const user = await this.createBasicUserScenario(`${prefix}${i + 1}`);
      users.push(user);
    }
    
    return users;
  }

  /**
   * Create a complex referral chain
   */
  async createReferralChain(depth: number): Promise<TestUser[]> {
    const chain: TestUser[] = [];
    
    // Create founder
    const founder = await this.createBasicUserScenario("Founder");
    chain.push(founder);
    
    let currentReferrer = founder;
    
    for (let i = 1; i < depth; i++) {
      const newUserName = `ChainUser${i}`;
      const inviteCode = `CHAIN${i.toString().padStart(3, '0')}`;
      
      // Create invite code (must be exactly 8 bytes)
      await this.testEnv.program.methods
        .createInviteCode(createInviteCodeArray(inviteCode))
        .accountsPartial({
          inviteCodeAccount: currentReferrer.inviteCodePda,
          config: this.testEnv.pdas.configPda,
          inviter: currentReferrer.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([currentReferrer.keypair])
        .rpc();
      
      // Create new user
      const newUser = await createUser(this.testEnv, newUserName);
      
      // Initialize with referral
      await this.testEnv.program.methods
        .initUser(currentReferrer.keypair.publicKey)
        .accountsPartial({
          userState: newUser.userStatePda,
          user: newUser.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([newUser.keypair])
        .rpc();

      // Buy farm space
      await this.testEnv.program.methods
        .buyFarmSpace()
        .accountsPartial({
          userState: newUser.userStatePda,
          config: this.testEnv.pdas.configPda,
          farmSpace: newUser.farmSpacePda,
          user: newUser.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([newUser.keypair])
        .rpc();
      
      chain.push(newUser);
      currentReferrer = newUser;
    }
    
    return chain;
  }

  /**
   * Wait for time to pass (for reward accumulation)
   */
  async waitForRewards(milliseconds: number = 2000): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, milliseconds));
  }

  /**
   * Claim rewards for a user
   */
  async claimUserRewards(user: TestUser): Promise<string> {
    return await this.testEnv.program.methods
      .claimReward()
      .accountsPartial({
        userState: user.userStatePda,
        config: this.testEnv.pdas.configPda,
        globalStats: this.testEnv.pdas.globalStatsPda,
        rewardMint: this.testEnv.pdas.rewardMintPda,
        mintAuthority: this.testEnv.pdas.mintAuthorityPda,
        userTokenAccount: user.tokenAccount,
        user: user.keypair.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([user.keypair])
      .rpc();
  }

  /**
   * Buy mystery pack for user
   * TODO: Implement when entropy provider PDAs are properly set up
   */
  async buyMysteryPack(_user: TestUser, _quantity: number = 1): Promise<string> {
    throw new Error("buyMysteryPack not yet implemented - requires entropy provider setup");
    // return await this.testEnv.program.methods
    //   .purchaseSeedPack(quantity, new anchor.BN(Date.now()))
    //   .accountsPartial({
    //     userState: user.userStatePda,
    //     config: this.testEnv.pdas.configPda,
    //     seedPack: user.seedPackPda,
    //     rewardMint: this.testEnv.pdas.rewardMintPda,
    //     userTokenAccount: user.tokenAccount,
    //     entropyProvider: this.testEnv.pdas.entropyProvider,
    //     entropyRequest: user.entropyRequest,
    //     user: user.keypair.publicKey,
    //     pythEntropyProgram: this.testEnv.pdas.pythEntropyProgram,
    //     tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    //     systemProgram: anchor.web3.SystemProgram.programId,
    //   })
    //   .signers([user.keypair])
    //   .rpc();
  }
}

/**
 * Test constants for consistent testing
 */
export const TEST_CONSTANTS = {
  BASE_RATE: 100,
  HALVING_INTERVAL: 6 * 24 * 60 * 60, // 6 days
  SEED_PACK_COST: 300_000_000, // 300 WEED with 6 decimals
  FIXED_SUPPLY_CAP: 120_000_000_000_000, // 120M WEED with 6 decimals
  DEFAULT_GROW_POWER: 100,
  FARM_SPACE_LEVELS: {
    1: { capacity: 4, upgradeCost: 0 },
    2: { capacity: 8, upgradeCost: 3500_000_000 },
    3: { capacity: 12, upgradeCost: 18000_000_000 },
  },
  REFERRAL_RATES: {
    LEVEL1: 10, // 10%
    LEVEL2: 5,  // 5%
  }
};

/**
 * Format token amount for display
 */
export function formatTokenAmount(amount: string | number): string {
  const num = typeof amount === 'string' ? parseInt(amount) : amount;
  return (num / 1_000_000).toLocaleString('en-US', { 
    minimumFractionDigits: 0, 
    maximumFractionDigits: 6 
  });
}

/**
 * Sleep utility for test timing
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Convert string to invite code array (exactly 8 bytes)
 */
export function createInviteCodeArray(inviteCodeString: string): number[] {
  const code = inviteCodeString.slice(0, 8).padEnd(8, '0');
  return Array.from(Buffer.from(code, 'utf8'));
}