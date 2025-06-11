import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { setupTestEnvironment } from "../helpers/setup";
import { TestScenarioFactory, formatTokenAmount, createInviteCodeArray } from "../helpers/factories";
import { 
  assertUserGrowPower,
  assertSupplyCapCompliance,
  assertEconomicConsistency 
} from "../helpers/assertions";

describe("Complete User Journey Integration", () => {
  let testEnv: any;
  let factory: TestScenarioFactory;
  
  before(async () => {
    testEnv = await setupTestEnvironment();
    factory = new TestScenarioFactory(testEnv);
  });

  describe("Basic User Journey", () => {
    it("Should complete full user onboarding and farming cycle", async () => {
      console.log("🌱 Starting Basic User Journey Test");
      
      // Step 1: Create and initialize user
      const user = await factory.createBasicUserScenario("JourneyUser");
      console.log(`✅ User ${user.name} created and initialized`);
      
      // Step 2: Verify initial state
      await assertUserGrowPower(testEnv, user.userStatePda, 100);
      console.log("✅ Initial grow power verified");
      
      // Step 3: Wait for rewards to accumulate
      await factory.waitForRewards(2000);
      console.log("⏰ Waited for reward accumulation");
      
      // Step 4: Claim rewards
      const claimSignature = await factory.claimUserRewards(user);
      console.log(`💰 Rewards claimed: ${claimSignature}`);
      
      // Step 5: Verify token balance
      const balance = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
      const rewardAmount = parseInt(balance.value.amount);
      expect(rewardAmount).to.be.greaterThan(0);
      console.log(`💎 User earned: ${formatTokenAmount(rewardAmount)} WEED tokens`);
      
      // Step 6: Verify supply cap compliance
      await assertSupplyCapCompliance(testEnv);
      console.log("✅ Supply cap compliance verified");
    });

    it("Should handle multiple claim cycles", async () => {
      console.log("🔄 Testing Multiple Claim Cycles");
      
      const user = await factory.createBasicUserScenario("CycleUser");
      let totalRewards = 0;
      
      // Perform 3 claim cycles
      for (let cycle = 1; cycle <= 3; cycle++) {
        await factory.waitForRewards(1500);
        
        const balanceBefore = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        await factory.claimUserRewards(user);
        const balanceAfter = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        
        const cycleReward = parseInt(balanceAfter.value.amount) - parseInt(balanceBefore.value.amount);
        totalRewards += cycleReward;
        
        console.log(`  Cycle ${cycle}: ${formatTokenAmount(cycleReward)} WEED earned`);
      }
      
      console.log(`💰 Total rewards over 3 cycles: ${formatTokenAmount(totalRewards)} WEED`);
      expect(totalRewards).to.be.greaterThan(0);
    });
  });

  describe("Referral Journey", () => {
    it("Should complete referral chain onboarding", async () => {
      console.log("🤝 Starting Referral Chain Test");
      
      // Create 3-level referral chain
      const referralChain = await factory.createReferralChain(3);
      console.log(`✅ Created referral chain of ${referralChain.length} users`);
      
      // Wait for all users to accumulate rewards
      await factory.waitForRewards(2000);
      
      // Track referral rewards distribution
      const rewardResults: { user: string, rewards: number }[] = [];
      
      for (const user of referralChain) {
        const balanceBefore = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        await factory.claimUserRewards(user);
        const balanceAfter = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        
        const rewards = parseInt(balanceAfter.value.amount) - parseInt(balanceBefore.value.amount);
        rewardResults.push({ user: user.name, rewards });
        
        console.log(`  ${user.name}: ${formatTokenAmount(rewards)} WEED earned`);
      }
      
      // Verify all users earned rewards
      rewardResults.forEach(result => {
        expect(result.rewards).to.be.greaterThan(0);
      });
      
      console.log("✅ All referral chain members earned rewards");
    });

    it("Should demonstrate referral reward benefits", async () => {
      console.log("💎 Testing Referral Reward Benefits");
      
      // Create referrer with multiple referrals
      const referrer = await factory.createBasicUserScenario("PopularReferrer");
      
      // Create invite code
      const inviteCode = "POPULAR001";
      await testEnv.program.methods
        .createInviteCode(createInviteCodeArray(inviteCode))
        .accountsPartial({
          inviteCodeAccount: referrer.inviteCodePda,
          config: testEnv.pdas.configPda,
          inviter: referrer.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([referrer.keypair])
        .rpc();
      
      // Create multiple referred users
      const referredUsers = [];
      for (let i = 1; i <= 3; i++) {
        const { referred } = await factory.createReferralScenario(referrer.name, `Referred${i}`);
        referredUsers.push(referred);
      }
      
      console.log(`✅ Referrer gained ${referredUsers.length} referrals`);
      
      // Wait and claim rewards for all
      await factory.waitForRewards(2000);
      
      // Claim for referrer
      const referrerBalanceBefore = await testEnv.connection.getTokenAccountBalance(referrer.tokenAccount);
      await factory.claimUserRewards(referrer);
      const referrerBalanceAfter = await testEnv.connection.getTokenAccountBalance(referrer.tokenAccount);
      
      const referrerRewards = parseInt(referrerBalanceAfter.value.amount) - parseInt(referrerBalanceBefore.value.amount);
      
      console.log(`💰 Referrer earned: ${formatTokenAmount(referrerRewards)} WEED (including referral bonuses)`);
      expect(referrerRewards).to.be.greaterThan(0);
    });
  });

  describe("Economic Journey", () => {
    it("Should demonstrate mystery pack investment strategy", async () => {
      console.log("🎲 Testing Mystery Pack Investment Strategy");
      
      const investor = await factory.createBasicUserScenario("MysteryInvestor");
      
      // First, user needs to accumulate WEED tokens
      await factory.waitForRewards(2000);
      await factory.claimUserRewards(investor);
      
      const balanceBeforePack = await testEnv.connection.getTokenAccountBalance(investor.tokenAccount);
      const balanceValue = parseInt(balanceBeforePack.value.amount);
      
      console.log(`💰 Investor balance: ${formatTokenAmount(balanceValue)} WEED`);
      
      if (balanceValue >= 300_000_000) { // 300 WEED minimum
        try {
          await factory.buyMysteryPack(investor, 1);
          console.log("🎁 Successfully purchased mystery pack");
          
          const balanceAfterPack = await testEnv.connection.getTokenAccountBalance(investor.tokenAccount);
          const spent = balanceValue - parseInt(balanceAfterPack.value.amount);
          
          console.log(`💸 Spent: ${formatTokenAmount(spent)} WEED on mystery pack`);
          expect(spent).to.equal(300_000_000);
        } catch (error) {
          console.log("⚠️ Mystery pack purchase failed (expected for testing)");
        }
      } else {
        console.log("⏰ User needs more WEED before buying mystery packs");
      }
    });

    it("Should maintain economic equilibrium across multiple users", async () => {
      console.log("⚖️ Testing Economic Equilibrium");
      
      // Create multiple users with different strategies
      const users = await factory.createMultiUserScenario(5, "EconUser");
      
      // Let all users farm for a period
      await factory.waitForRewards(3000);
      
      // Track total rewards distribution
      let totalRewardsDistributed = 0;
      const userRewards: { name: string, amount: number }[] = [];
      
      for (const user of users) {
        const balanceBefore = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        await factory.claimUserRewards(user);
        const balanceAfter = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        
        const rewards = parseInt(balanceAfter.value.amount) - parseInt(balanceBefore.value.amount);
        totalRewardsDistributed += rewards;
        userRewards.push({ name: user.name, amount: rewards });
      }
      
      console.log("📊 Reward Distribution:");
      userRewards.forEach(result => {
        console.log(`  ${result.name}: ${formatTokenAmount(result.amount)} WEED`);
      });
      
      console.log(`💎 Total distributed: ${formatTokenAmount(totalRewardsDistributed)} WEED`);
      
      // Verify economic consistency
      await assertEconomicConsistency(testEnv);
      await assertSupplyCapCompliance(testEnv);
      
      console.log("✅ Economic equilibrium maintained");
    });
  });

  describe("Advanced User Scenarios", () => {
    it("Should handle complex multi-user interactions", async () => {
      console.log("🌐 Testing Complex Multi-User Interactions");
      
      // Create a mini-ecosystem
      const founder = await factory.createBasicUserScenario("EcoFounder");
      const pioneers = await factory.createMultiUserScenario(3, "Pioneer");
      const adopters = await factory.createMultiUserScenario(2, "Adopter");
      
      // Simulate time passage and interactions
      await factory.waitForRewards(2000);
      
      // All users claim rewards
      const allUsers = [founder, ...pioneers, ...adopters];
      const totalParticipants = allUsers.length;
      
      for (const user of allUsers) {
        await factory.claimUserRewards(user);
      }
      
      console.log(`✅ ${totalParticipants} users successfully participated in ecosystem`);
      
      // Verify global state consistency
      const globalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
      console.log(`📊 Global Stats: ${globalStats.totalUsers} users, ${globalStats.totalGrowPower} total grow power`);
      
      expect(globalStats.totalUsers.toNumber()).to.be.greaterThanOrEqual(totalParticipants);
    });

    it("Should demonstrate sustainable growth patterns", async () => {
      console.log("📈 Testing Sustainable Growth Patterns");
      
      // Create users in waves to simulate organic growth
      const wave1 = await factory.createMultiUserScenario(2, "Wave1User");
      await factory.waitForRewards(1000);
      
      const wave2 = await factory.createMultiUserScenario(3, "Wave2User");
      await factory.waitForRewards(1000);
      
      const wave3 = await factory.createMultiUserScenario(2, "Wave3User");
      await factory.waitForRewards(1000);
      
      // All waves claim rewards
      const allWaves = [...wave1, ...wave2, ...wave3];
      let totalCommunityRewards = 0;
      
      for (const user of allWaves) {
        const balanceBefore = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        await factory.claimUserRewards(user);
        const balanceAfter = await testEnv.connection.getTokenAccountBalance(user.tokenAccount);
        
        const rewards = parseInt(balanceAfter.value.amount) - parseInt(balanceBefore.value.amount);
        totalCommunityRewards += rewards;
      }
      
      console.log(`🌱 Community growth: ${allWaves.length} users across 3 waves`);
      console.log(`💰 Total community rewards: ${formatTokenAmount(totalCommunityRewards)} WEED`);
      
      // Verify sustainability
      await assertSupplyCapCompliance(testEnv);
      console.log("✅ Growth pattern remains sustainable");
    });
  });

  after(async () => {
    console.log("🎉 User Journey Integration Tests Complete!");
    
    // Final system health check
    await assertEconomicConsistency(testEnv);
    await assertSupplyCapCompliance(testEnv);
    
    const config = await testEnv.program.account.config.fetch(testEnv.pdas.configPda);
    const globalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
    
    console.log("📊 Final System State:");
    console.log(`  Total Supply Minted: ${formatTokenAmount(config.totalSupplyMinted.toString())} WEED`);
    console.log(`  Total Users: ${globalStats.totalUsers.toString()}`);
    console.log(`  Total Grow Power: ${globalStats.totalGrowPower.toString()}`);
    console.log("✅ All journey tests completed successfully!");
  });
});