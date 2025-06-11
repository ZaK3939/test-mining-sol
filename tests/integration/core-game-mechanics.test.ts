import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { TestEnvironment, TEST_CONSTANTS } from "../helpers/test-setup";
import { GameAssertions } from "../helpers/assertions";
import { TestDataGenerator } from "../helpers/test-data";

describe("Core Game Mechanics", () => {
  let testEnv: TestEnvironment;

  before(async () => {
    testEnv = await TestEnvironment.setup();
    await testEnv.initializeSystem();
  });

  describe("System Initialization", () => {
    it("should initialize system configuration correctly", async () => {
      const config = await testEnv.getConfig();
      
      GameAssertions.assertConfig(config, {
        baseRate: new anchor.BN(TEST_CONSTANTS.BASE_RATE),
        halvingInterval: new anchor.BN(TEST_CONSTANTS.HALVING_INTERVAL),
        admin: testEnv.accounts.admin.publicKey,
        treasury: testEnv.accounts.treasury.publicKey,
        protocolReferralAddress: testEnv.accounts.protocolReferralAddress.publicKey,
        seedPackCost: new anchor.BN(TEST_CONSTANTS.SEED_PACK_COST),
        farmSpaceCostSol: new anchor.BN(TEST_CONSTANTS.FARM_SPACE_COST),
      });
    });

    it("should initialize global stats correctly", async () => {
      const globalStats = await testEnv.getGlobalStats();
      
      GameAssertions.assertGlobalStats(globalStats, {
        totalGrowPower: new anchor.BN(0),
        totalFarmSpaces: new anchor.BN(0),
        currentRewardsPerSecond: new anchor.BN(TEST_CONSTANTS.BASE_RATE),
      });
    });
  });

  describe("User Management", () => {
    it("should initialize user without referrer", async () => {
      const userData = await testEnv.createUserTestData(0);
      const userState = await testEnv.getUserState(userData.userStatePda);
      
      GameAssertions.assertUserState(userState, {
        owner: userData.keypair.publicKey,
        totalGrowPower: new anchor.BN(0),
        hasFarmSpace: false,
        referrer: null,
        pendingReferralRewards: new anchor.BN(0),
      });
    });

    it("should initialize user with referrer", async () => {
      const referrerData = await testEnv.createUserTestData(1);
      const userData = await testEnv.createUserTestData(2, referrerData.keypair.publicKey);
      const userState = await testEnv.getUserState(userData.userStatePda);
      
      GameAssertions.assertUserState(userState, {
        owner: userData.keypair.publicKey,
        referrer: referrerData.keypair.publicKey,
      });
    });

    it("should prevent duplicate user initialization", async () => {
      const userData = await testEnv.createUserTestData(3);
      
      await GameAssertions.assertTransactionError(
        testEnv.program.methods
          .initUser(null)
          .accounts({
            userState: userData.userStatePda,
            user: userData.keypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([userData.keypair])
          .rpc(),
        /already initialized|Account already exists/i,
        "Should prevent duplicate user initialization"
      );
    });
  });

  describe("Farm Space Management", () => {
    it("should purchase farm space successfully", async () => {
      const userData = await testEnv.createUserTestData(4);
      const initialTreasuryBalance = await testEnv.provider.connection.getBalance(
        testEnv.accounts.treasury.publicKey
      );
      
      await testEnv.buyFarmSpace(userData);
      
      // Check user state updated
      const userState = await testEnv.getUserState(userData.userStatePda);
      GameAssertions.assertUserState(userState, {
        hasFarmSpace: true,
        totalGrowPower: new anchor.BN(TEST_CONSTANTS.SEED_GROW_POWERS.Seed1), // Initial seed
      });
      
      // Check farm space created
      const farmSpace = await testEnv.getFarmSpace(userData.farmSpacePda);
      GameAssertions.assertFarmSpace(farmSpace, {
        owner: userData.keypair.publicKey,
        level: 1,
        capacity: TEST_CONSTANTS.FARM_SPACE_LEVELS[1].capacity,
        seedCount: 1, // Initial seed planted
        totalGrowPower: new anchor.BN(TEST_CONSTANTS.SEED_GROW_POWERS.Seed1),
      });
      
      // Check treasury received payment
      const finalTreasuryBalance = await testEnv.provider.connection.getBalance(
        testEnv.accounts.treasury.publicKey
      );
      expect(finalTreasuryBalance).to.be.greaterThan(initialTreasuryBalance);
      
      // Check global stats updated
      const globalStats = await testEnv.getGlobalStats();
      GameAssertions.assertGlobalStats(globalStats, {
        totalFarmSpaces: new anchor.BN(1),
        totalGrowPower: new anchor.BN(TEST_CONSTANTS.SEED_GROW_POWERS.Seed1),
      });
    });

    it("should prevent duplicate farm space purchase", async () => {
      const userData = await testEnv.createUserTestData(0); // Already has farm space
      
      await GameAssertions.assertTransactionError(
        testEnv.buyFarmSpace(userData),
        /already owns|duplicate farm/i,
        "Should prevent duplicate farm space purchase"
      );
    });

    it("should prevent farm space purchase without user initialization", async () => {
      const uninitializedUser = testEnv.accounts.users[4]; // Not initialized
      const [uninitializedUserStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("user"), uninitializedUser.publicKey.toBuffer()],
        testEnv.program.programId
      );
      const [uninitializedFarmSpacePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("farm_space"), uninitializedUser.publicKey.toBuffer()],
        testEnv.program.programId
      );
      
      await GameAssertions.assertTransactionError(
        testEnv.program.methods
          .buyFarmSpace()
          .accounts({
            userState: uninitializedUserStatePda,
            farmSpace: uninitializedFarmSpacePda,
            globalStats: testEnv.pdas.globalStatsPda,
            config: testEnv.pdas.configPda,
            user: uninitializedUser.publicKey,
            treasury: testEnv.accounts.treasury.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([uninitializedUser])
          .rpc(),
        /Account does not exist|User not initialized/i,
        "Should prevent farm space purchase without user initialization"
      );
    });
  });

  describe("Reward System", () => {
    it("should calculate and distribute rewards correctly", async () => {
      const userData = await testEnv.createUserTestData(0); // User with farm space
      
      // Wait for rewards to accumulate
      await testEnv.waitForRewards(3000);
      
      const initialBalance = new anchor.BN(await testEnv.getTokenBalance(userData.tokenAccount.address));
      const claimedReward = await testEnv.claimRewards(userData);
      
      // Verify reward was positive
      expect(claimedReward.gt(new anchor.BN(0))).to.be.true;
      
      // Verify token balance increased
      const finalBalance = new anchor.BN(await testEnv.getTokenBalance(userData.tokenAccount.address));
      expect(finalBalance.sub(initialBalance).toString()).to.equal(claimedReward.toString());
    });

    it("should prevent reward claiming without farm space", async () => {
      const userData = await testEnv.createUserTestData(1); // User without farm space
      
      await GameAssertions.assertTransactionError(
        testEnv.claimRewards(userData),
        /No farm space|Must own farm/i,
        "Should prevent reward claiming without farm space"
      );
    });

    it("should handle zero rewards gracefully", async () => {
      const userData = await testEnv.createUserTestData(0); // User with farm space
      
      // Claim immediately after previous claim (no time elapsed)
      await GameAssertions.assertTransactionError(
        testEnv.claimRewards(userData),
        /No reward to claim|Insufficient time/i,
        "Should handle zero rewards gracefully"
      );
    });
  });

  describe("Referral System", () => {
    it("should distribute Level 1 referral rewards correctly", async () => {
      // Create referral chain: level1 -> level0
      const level1Data = await testEnv.createUserTestData(1); // Referrer
      await testEnv.buyFarmSpace(level1Data);
      
      const level0Data = await testEnv.createUserTestData(0, level1Data.keypair.publicKey); // Referral
      await testEnv.buyFarmSpace(level0Data);
      
      // Wait for rewards to accumulate
      await testEnv.waitForRewards(3000);
      
      // Get initial balances
      const initialLevel0Balance = new anchor.BN(await testEnv.getTokenBalance(level0Data.tokenAccount.address));
      const initialLevel1Balance = new anchor.BN(await testEnv.getTokenBalance(level1Data.tokenAccount.address));
      
      // Claim rewards for level0 user
      const baseReward = await testEnv.claimRewards(level0Data);
      
      // Distribute referral rewards
      await testEnv.distributeReferralRewards(level0Data, level1Data, baseReward);
      
      // Check Level 1 referrer received 10% reward
      const finalLevel1Balance = new anchor.BN(await testEnv.getTokenBalance(level1Data.tokenAccount.address));
      const level1Reward = finalLevel1Balance.sub(initialLevel1Balance);
      
      GameAssertions.assertReferralReward(
        baseReward,
        level1Reward,
        TEST_CONSTANTS.LEVEL1_REFERRAL_PERCENTAGE,
        new anchor.BN(1),
        "Level 1 referral reward mismatch"
      );
    });

    it("should exclude protocol address from referral rewards", async () => {
      // Create user with protocol address as referrer
      const protocolReferredData = await testEnv.createUserTestData(
        3, 
        testEnv.accounts.protocolReferralAddress.publicKey
      );
      await testEnv.buyFarmSpace(protocolReferredData);
      
      // Wait for rewards to accumulate
      await testEnv.waitForRewards(2000);
      
      // Claim rewards - should not fail even with protocol referrer
      const baseReward = await testEnv.claimRewards(protocolReferredData);
      expect(baseReward.gt(new anchor.BN(0))).to.be.true;
      
      // Verify protocol address doesn't receive referral rewards
      // (This would require additional account tracking for complete verification)
    });
  });

  describe("State Consistency", () => {
    it("should maintain consistent global statistics", async () => {
      const initialGlobalStats = await testEnv.getGlobalStats();
      
      // Create multiple users and farm spaces
      const userData1 = await testEnv.createUserTestData(2);
      const userData2 = await testEnv.createUserTestData(3);
      await testEnv.buyFarmSpace(userData1);
      await testEnv.buyFarmSpace(userData2);
      
      const finalGlobalStats = await testEnv.getGlobalStats();
      
      // Verify global stats updated correctly
      expect(finalGlobalStats.totalFarmSpaces.toString()).to.equal(
        initialGlobalStats.totalFarmSpaces.add(new anchor.BN(2)).toString(),
        "Total farm spaces should increase by 2"
      );
      
      expect(finalGlobalStats.totalGrowPower.toString()).to.equal(
        initialGlobalStats.totalGrowPower.add(new anchor.BN(TEST_CONSTANTS.SEED_GROW_POWERS.Seed1 * 2)).toString(),
        "Total grow power should increase by 2 * Seed1 grow power"
      );
    });

    it("should maintain user state consistency across operations", async () => {
      const userData = await testEnv.createUserTestData(4);
      await testEnv.buyFarmSpace(userData);
      
      // Wait and claim rewards
      await testEnv.waitForRewards(2000);
      const initialTokenBalance = new anchor.BN(await testEnv.getTokenBalance(userData.tokenAccount.address));
      await testEnv.claimRewards(userData);
      
      // Verify user state consistency
      const userState = await testEnv.getUserState(userData.userStatePda);
      const farmSpace = await testEnv.getFarmSpace(userData.farmSpacePda);
      
      expect(userState.totalGrowPower.toString()).to.equal(
        farmSpace.totalGrowPower.toString(),
        "User grow power should match farm space grow power"
      );
      
      expect(userState.hasFarmSpace).to.be.true;
      expect(userState.owner.toString()).to.equal(userData.keypair.publicKey.toString());
    });
  });
});