import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { setupTestEnvironment, createUser } from "../../helpers/setup";
import { TestScenarioFactory } from "../../helpers/factories";
import { 
  assertUserGrowPower,
  assertGlobalStats,
  assertReferralRelationship,
  assertAccountExists 
} from "../../helpers/assertions";

describe("User Management Instructions", () => {
  let testEnv: any;
  let factory: TestScenarioFactory;
  
  before(async () => {
    testEnv = await setupTestEnvironment();
    factory = new TestScenarioFactory(testEnv);
  });

  describe("init_user", () => {
    it("Should initialize user without referrer", async () => {
      const user = await createUser(testEnv, "TestUser1");
      
      const signature = await testEnv.program.methods
        .initUser(null)
        .accountsPartial({
          userState: user.userStatePda,
          user: user.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user.keypair])
        .rpc();

      // Verify user state was created
      await assertAccountExists(testEnv, user.userStatePda, "UserState");
      
      const userState = await testEnv.program.account.userState.fetch(user.userStatePda);
      expect(userState.owner.toString()).to.equal(user.keypair.publicKey.toString());
      expect(userState.referrer).to.be.null;
      expect(userState.totalGrowPower.toNumber()).to.equal(0);
      expect(userState.lastClaimTime.toNumber()).to.be.greaterThan(0);
    });

    it("Should initialize user with referrer", async () => {
      const { referrer, referred } = await factory.createReferralScenario("Referrer1", "Referred1");
      
      // Verify referral relationship
      await assertReferralRelationship(testEnv, referred.userStatePda, referrer.keypair.publicKey);
      
      const referredState = await testEnv.program.account.userState.fetch(referred.userStatePda);
      expect(referredState.referrer?.toString()).to.equal(referrer.keypair.publicKey.toString());
    });

    it("Should reject duplicate user initialization", async () => {
      const user = await createUser(testEnv, "DuplicateUser");
      
      // Initialize user first time
      await testEnv.program.methods
        .initUser(null)
        .accountsPartial({
          userState: user.userStatePda,
          user: user.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user.keypair])
        .rpc();

      // Try to initialize again (should fail)
      try {
        await testEnv.program.methods
          .initUser(null)
          .accountsPartial({
            userState: user.userStatePda,
            user: user.keypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user.keypair])
          .rpc();
        
        expect.fail("Should have failed on duplicate initialization");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Should reject initialization with invalid referrer", async () => {
      const user = await createUser(testEnv, "InvalidReferrerUser");
      const fakeReferrer = anchor.web3.Keypair.generate();
      
      try {
        await testEnv.program.methods
          .initUser(fakeReferrer.publicKey)
          .accountsPartial({
            userState: user.userStatePda,
            user: user.keypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user.keypair])
          .rpc();
        
        expect.fail("Should have failed with invalid referrer");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Should update global stats on user creation", async () => {
      const initialStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
      const initialUserCount = initialStats.totalUsers.toNumber();
      
      const user = await createUser(testEnv, "StatsTestUser");
      await testEnv.program.methods
        .initUser(null)
        .accountsPartial({
          userState: user.userStatePda,
          user: user.keypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([user.keypair])
        .rpc();

      const finalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
      expect(finalStats.totalUsers.toNumber()).to.equal(initialUserCount + 1);
    });
  });

  describe("buy_farm_space", () => {
    it("Should allow user to buy farm space", async () => {
      const user = await factory.createBasicUserScenario("FarmBuyer");
      
      // Verify farm space was created
      await assertAccountExists(testEnv, user.farmSpacePda, "FarmSpace");
      
      const farmSpace = await testEnv.program.account.farmSpace.fetch(user.farmSpacePda);
      expect(farmSpace.owner.toString()).to.equal(user.keypair.publicKey.toString());
      expect(farmSpace.level).to.equal(1);
      expect(farmSpace.capacity).to.equal(4);
      expect(farmSpace.seedCount).to.equal(1); // Comes with one seed
      expect(farmSpace.totalGrowPower.toNumber()).to.equal(100); // Default grow power
    });

    it("Should update user grow power after farm purchase", async () => {
      const user = await factory.createBasicUserScenario("GrowPowerUser");
      
      await assertUserGrowPower(testEnv, user.userStatePda, 100);
    });

    it("Should update global stats after farm purchase", async () => {
      const initialStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
      const initialGrowPower = initialStats.totalGrowPower.toNumber();
      
      const user = await factory.createBasicUserScenario("GlobalStatsUser");
      
      const finalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
      expect(finalStats.totalGrowPower.toNumber()).to.equal(initialGrowPower + 100);
    });

    it("Should reject duplicate farm space purchase", async () => {
      const user = await factory.createBasicUserScenario("DuplicateFarmUser");
      
      // Try to buy farm space again (should fail)
      try {
        await testEnv.program.methods
          .buyFarmSpace()
          .accountsPartial({
            userState: user.userStatePda,
            config: testEnv.pdas.configPda,
            farmSpace: user.farmSpacePda,
            user: user.keypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user.keypair])
          .rpc();
        
        expect.fail("Should have failed on duplicate farm space purchase");
      } catch (error) {
        expect(error).to.exist;
      }
    });

    it("Should reject farm space purchase without user initialization", async () => {
      const user = await createUser(testEnv, "UninitializedUser");
      
      try {
        await testEnv.program.methods
          .buyFarmSpace()
          .accountsPartial({
            userState: user.userStatePda,
            config: testEnv.pdas.configPda,
            farmSpace: user.farmSpacePda,
            user: user.keypair.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([user.keypair])
          .rpc();
        
        expect.fail("Should have failed without user initialization");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("User state validation", () => {
    it("Should maintain consistent user state across operations", async () => {
      const user = await factory.createBasicUserScenario("ConsistentUser");
      
      // Verify user state consistency
      const userState = await testEnv.program.account.userState.fetch(user.userStatePda);
      const farmSpace = await testEnv.program.account.farmSpace.fetch(user.farmSpacePda);
      
      expect(userState.totalGrowPower.toNumber()).to.equal(farmSpace.totalGrowPower.toNumber());
      expect(userState.owner.toString()).to.equal(farmSpace.owner.toString());
    });

    it("Should handle multiple users correctly", async () => {
      const users = await factory.createMultiUserScenario(3, "MultiUser");
      
      // Verify each user has correct state
      for (let i = 0; i < users.length; i++) {
        const user = users[i];
        const userState = await testEnv.program.account.userState.fetch(user.userStatePda);
        const farmSpace = await testEnv.program.account.farmSpace.fetch(user.farmSpacePda);
        
        expect(userState.owner.toString()).to.equal(user.keypair.publicKey.toString());
        expect(farmSpace.owner.toString()).to.equal(user.keypair.publicKey.toString());
        expect(userState.totalGrowPower.toNumber()).to.equal(100);
      }
    });
  });
});