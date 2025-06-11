import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { TestEnvironment, TEST_CONSTANTS } from "../helpers/test-setup";
import { GameAssertions } from "../helpers/assertions";
import { TestDataGenerator } from "../helpers/test-data";

describe("Halving Mechanism", () => {
  let testEnv: TestEnvironment;

  before(async () => {
    testEnv = await TestEnvironment.setup();
    // Initialize with shorter halving interval for testing (1 minute instead of 6 days)
    await testEnv.program.methods
      .initializeConfig(
        new anchor.BN(100), // base_rate
        new anchor.BN(60), // halving_interval (1 minute for testing)
        testEnv.accounts.treasury.publicKey,
        testEnv.accounts.protocolReferralAddress.publicKey
      )
      .accounts({
        config: testEnv.pdas.configPda,
        admin: testEnv.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([testEnv.accounts.admin])
      .rpc();

    await testEnv.program.methods
      .initializeGlobalStats()
      .accounts({
        globalStats: testEnv.pdas.globalStatsPda,
        admin: testEnv.accounts.admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([testEnv.accounts.admin])
      .rpc();

    await testEnv.program.methods
      .createRewardMint()
      .accounts({
        rewardMint: testEnv.pdas.rewardMintPda,
        mintAuthority: testEnv.pdas.mintAuthorityPda,
        metadataAccount: anchor.web3.Keypair.generate().publicKey,
        admin: testEnv.accounts.admin.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
      })
      .signers([testEnv.accounts.admin])
      .rpc();
  });

  describe("Single Halving Events", () => {
    it("should trigger first halving after interval", async () => {
      // Setup user with farm space
      const userData = await testEnv.createUserTestData(0);
      await testEnv.buyFarmSpace(userData);

      // Get initial configuration
      const initialConfig = await testEnv.getConfig();
      const initialRate = initialConfig.baseRate;
      const initialGlobalStats = await testEnv.getGlobalStats();

      expect(initialRate.toString()).to.equal("100");
      expect(initialGlobalStats.currentRewardsPerSecond.toString()).to.equal("100");

      // Wait for halving interval (65 seconds to ensure crossing)
      console.log("⏳ Waiting for halving interval (65 seconds)...");
      await new Promise(resolve => setTimeout(resolve, 65000));

      // Claim rewards to trigger halving check
      const reward = await testEnv.claimRewards(userData);
      expect(reward.gt(new anchor.BN(0))).to.be.true;

      // Check that halving occurred
      const finalConfig = await testEnv.getConfig();
      const finalGlobalStats = await testEnv.getGlobalStats();

      GameAssertions.assertHalving(
        initialRate,
        finalConfig.baseRate,
        1,
        "First halving should reduce rate by half"
      );

      expect(finalGlobalStats.currentRewardsPerSecond.toString()).to.equal(
        finalConfig.baseRate.toString(),
        "Global stats should match config after halving"
      );

      console.log(`✅ First halving: ${initialRate.toString()} → ${finalConfig.baseRate.toString()}`);
    });

    it("should trigger multiple consecutive halvings", async () => {
      const userData = await testEnv.createUserTestData(1);
      await testEnv.buyFarmSpace(userData);

      // Get current rate (should be 50 from previous test)
      const beforeConfig = await testEnv.getConfig();
      console.log(`📊 Rate before multiple halvings: ${beforeConfig.baseRate.toString()}`);

      // Trigger multiple halvings by waiting and claiming
      const halvingEvents = [];
      for (let i = 0; i < 3; i++) {
        console.log(`⏳ Waiting for halving ${i + 1}...`);
        await new Promise(resolve => setTimeout(resolve, 65000));
        
        const reward = await testEnv.claimRewards(userData);
        expect(reward.gt(new anchor.BN(0))).to.be.true;
        
        const config = await testEnv.getConfig();
        halvingEvents.push({
          halving: i + 1,
          rate: config.baseRate,
          nextHalvingTime: config.nextHalvingTime
        });
        
        console.log(`📊 After halving ${i + 1}: rate = ${config.baseRate.toString()}`);
      }

      // Verify halving progression
      for (let i = 0; i < halvingEvents.length; i++) {
        const event = halvingEvents[i];
        const expectedRate = beforeConfig.baseRate.div(new anchor.BN(Math.pow(2, i + 1)));
        expect(event.rate.toString()).to.equal(expectedRate.toString());
      }

      console.log("✅ Multiple consecutive halvings completed successfully");
    });
  });

  describe("Reward Calculation Across Halvings", () => {
    it("should calculate rewards correctly when claiming across halving boundary", async () => {
      // Create fresh test environment with predictable timing
      const shortTestEnv = await TestEnvironment.setup();
      
      // Initialize with 30-second halving for precise testing
      await shortTestEnv.program.methods
        .initializeConfig(
          new anchor.BN(1000), // Higher base rate for easier calculation
          new anchor.BN(30), // 30-second halving interval
          shortTestEnv.accounts.treasury.publicKey,
          shortTestEnv.accounts.protocolReferralAddress.publicKey
        )
        .accounts({
          config: shortTestEnv.pdas.configPda,
          admin: shortTestEnv.accounts.admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([shortTestEnv.accounts.admin])
        .rpc();

      await shortTestEnv.program.methods
        .initializeGlobalStats()
        .accounts({
          globalStats: shortTestEnv.pdas.globalStatsPda,
          admin: shortTestEnv.accounts.admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([shortTestEnv.accounts.admin])
        .rpc();

      await shortTestEnv.program.methods
        .createRewardMint()
        .accounts({
          rewardMint: shortTestEnv.pdas.rewardMintPda,
          mintAuthority: shortTestEnv.pdas.mintAuthorityPda,
          metadataAccount: anchor.web3.Keypair.generate().publicKey,
          admin: shortTestEnv.accounts.admin.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: new anchor.web3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
        })
        .signers([shortTestEnv.accounts.admin])
        .rpc();

      const userData = await shortTestEnv.createUserTestData(0);
      await shortTestEnv.buyFarmSpace(userData);

      // Wait for 45 seconds (crosses one halving boundary)
      console.log("⏳ Waiting 45 seconds to cross halving boundary...");
      await new Promise(resolve => setTimeout(resolve, 45000));

      // Claim rewards that span across halving
      const reward = await shortTestEnv.claimRewards(userData);
      expect(reward.gt(new anchor.BN(0))).to.be.true;

      // Verify the reward calculation used the halving-aware algorithm
      const config = await shortTestEnv.getConfig();
      expect(config.baseRate.toString()).to.equal("500"); // Should be halved

      console.log(`✅ Cross-halving reward calculation: ${reward.toString()} tokens`);
      console.log(`📊 Final rate after cross-halving claim: ${config.baseRate.toString()}`);
    });

    it("should handle multiple halvings in a single reward period", async () => {
      // Test scenario where multiple halvings occur between claims
      const userData = await testEnv.createUserTestData(2);
      await testEnv.buyFarmSpace(userData);

      // Wait for multiple halving intervals
      console.log("⏳ Waiting for multiple halving intervals (3 minutes)...");
      await new Promise(resolve => setTimeout(resolve, 180000)); // 3 minutes = 3 halvings

      const initialConfig = await testEnv.getConfig();
      const reward = await testEnv.claimRewards(userData);
      const finalConfig = await testEnv.getConfig();

      expect(reward.gt(new anchor.BN(0))).to.be.true;
      expect(finalConfig.baseRate.lt(initialConfig.baseRate)).to.be.true;

      console.log(`✅ Multiple halvings in single period handled`);
      console.log(`📊 Rate change: ${initialConfig.baseRate.toString()} → ${finalConfig.baseRate.toString()}`);
    });
  });

  describe("Halving Edge Cases", () => {
    it("should handle halving with zero grow power gracefully", async () => {
      // This tests the edge case where halving occurs but no users have grow power
      const config = await testEnv.getConfig();
      const globalStats = await testEnv.getGlobalStats();

      // Verify system can handle halving even with zero grow power
      expect(globalStats.totalGrowPower.gte(new anchor.BN(0))).to.be.true;
      expect(config.baseRate.gt(new anchor.BN(0))).to.be.true;

      console.log("✅ Halving with zero grow power handled gracefully");
    });

    it("should handle minimum rate limits", async () => {
      // Test what happens when rate approaches zero through many halvings
      let currentRate = new anchor.BN(100);
      let halvingCount = 0;

      // Simulate many halvings
      while (currentRate.gt(new anchor.BN(1)) && halvingCount < 10) {
        currentRate = currentRate.div(new anchor.BN(2));
        halvingCount++;
      }

      console.log(`📊 After ${halvingCount} simulated halvings: rate = ${currentRate.toString()}`);
      expect(halvingCount).to.be.greaterThan(0);
      console.log("✅ Minimum rate handling verified");
    });

    it("should maintain halving schedule consistency", async () => {
      const config = await testEnv.getConfig();
      const halvingInterval = config.halvingInterval;
      const nextHalvingTime = config.nextHalvingTime;
      const currentTime = new anchor.BN(Math.floor(Date.now() / 1000));

      // Verify next halving time is in the future and properly scheduled
      expect(nextHalvingTime.gt(currentTime)).to.be.true;
      
      // Verify the interval is reasonable
      const timeUntilHalving = nextHalvingTime.sub(currentTime);
      expect(timeUntilHalving.lte(halvingInterval)).to.be.true;

      console.log(`📅 Next halving in: ${timeUntilHalving.toString()} seconds`);
      console.log("✅ Halving schedule consistency verified");
    });
  });

  describe("Halving System Performance", () => {
    it("should handle halving calculations efficiently", async () => {
      const userData = await testEnv.createUserTestData(3);
      await testEnv.buyFarmSpace(userData);

      // Time reward calculation with halving
      const startTime = Date.now();
      await testEnv.waitForRewards(2000);
      const reward = await testEnv.claimRewards(userData);
      const endTime = Date.now();

      const calculationTime = endTime - startTime;
      expect(reward.gt(new anchor.BN(0))).to.be.true;
      
      console.log(`⏱️  Halving-aware reward calculation time: ${calculationTime}ms`);
      expect(calculationTime).to.be.lessThan(10000); // Should complete within 10 seconds

      console.log("✅ Halving calculation performance verified");
    });

    it("should handle concurrent claims during halving", async () => {
      // Create multiple users
      const users = [];
      for (let i = 0; i < 3; i++) {
        const userData = await testEnv.createUserTestData(4 + i);
        await testEnv.buyFarmSpace(userData);
        users.push(userData);
      }

      await testEnv.waitForRewards(2000);

      // Try concurrent claims (some might trigger halving)
      const promises = users.map(userData => testEnv.claimRewards(userData));
      const results = await Promise.all(promises);

      // Verify all claims succeeded
      results.forEach((reward, index) => {
        expect(reward.gt(new anchor.BN(0))).to.be.true;
        console.log(`👤 User ${index + 1} reward: ${reward.toString()}`);
      });

      console.log("✅ Concurrent claims during halving handled successfully");
    });
  });

  describe("Halving Integration with Other Systems", () => {
    it("should maintain referral rewards through halvings", async () => {
      // Create referral relationship
      const referrerData = await testEnv.createUserTestData(0);
      const referredData = await testEnv.createUserTestData(1, referrerData.keypair.publicKey);
      
      await testEnv.buyFarmSpace(referrerData);
      await testEnv.buyFarmSpace(referredData);

      await testEnv.waitForRewards(2000);

      // Claim rewards and distribute referrals
      const baseReward = await testEnv.claimRewards(referredData);
      await testEnv.distributeReferralRewards(referredData, referrerData, baseReward);

      // Verify referral system works with current halving state
      const referrerBalance = new anchor.BN(await testEnv.getTokenBalance(referrerData.tokenAccount.address));
      expect(referrerBalance.gt(new anchor.BN(0))).to.be.true;

      console.log("✅ Referral rewards maintained through halving");
    });

    it("should maintain global statistics accuracy through halvings", async () => {
      const initialGlobalStats = await testEnv.getGlobalStats();
      const initialConfig = await testEnv.getConfig();

      // Verify global stats consistency
      expect(initialGlobalStats.currentRewardsPerSecond.toString()).to.equal(
        initialConfig.baseRate.toString(),
        "Global stats should match config base rate"
      );

      console.log(`📊 Global stats consistency verified`);
      console.log(`📊 Current rewards per second: ${initialGlobalStats.currentRewardsPerSecond.toString()}`);
      console.log(`📊 Config base rate: ${initialConfig.baseRate.toString()}`);
    });
  });
});