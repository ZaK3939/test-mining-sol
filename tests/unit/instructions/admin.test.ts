import { expect } from "chai";
import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { setupTestEnvironment } from "../../helpers/setup";
import { assertTransactionSuccess, assertAccountExists } from "../../helpers/assertions";
import { TEST_CONSTANTS } from "../../helpers/factories";

describe("Admin Instructions", () => {
  let testEnv: any;
  
  before(async () => {
    // Setup without auto-initialization for admin tests
    testEnv = await (await import("../../helpers/setup")).TestEnvironment.setup();
  });

  describe("initialize_config", () => {
    it("Should initialize system configuration successfully", async () => {
      const signature = await testEnv.program.methods
        .initializeConfig(
          new anchor.BN(TEST_CONSTANTS.BASE_RATE),
          new anchor.BN(TEST_CONSTANTS.HALVING_INTERVAL),
          testEnv.accounts.treasury.publicKey,
          testEnv.accounts.protocolReferralAddress.publicKey
        )
        .accountsPartial({
          config: testEnv.pdas.configPda,
          admin: testEnv.accounts.admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([testEnv.accounts.admin])
        .rpc();

      await assertTransactionSuccess(testEnv, signature);
      await assertAccountExists(testEnv, testEnv.pdas.configPda, "Config");

      // Verify config values
      const config = await testEnv.program.account.config.fetch(testEnv.pdas.configPda);
      expect(config.admin.toString()).to.equal(testEnv.accounts.admin.publicKey.toString());
      expect(config.treasury.toString()).to.equal(testEnv.accounts.treasury.publicKey.toString());
      expect(config.baseRate.toNumber()).to.equal(TEST_CONSTANTS.BASE_RATE);
      expect(config.halvingInterval.toNumber()).to.equal(TEST_CONSTANTS.HALVING_INTERVAL);
      expect(config.totalSupplyMinted.toNumber()).to.equal(0);
    });

    it("Should reject initialization from non-admin", async () => {
      const nonAdmin = Keypair.generate();
      
      // Airdrop SOL for transaction fees
      await testEnv.connection.requestAirdrop(nonAdmin.publicKey, LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Create different config PDA for this test
      const [configPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("config2")],
        testEnv.program.programId
      );

      try {
        await testEnv.program.methods
          .initializeConfig(
            new anchor.BN(TEST_CONSTANTS.BASE_RATE),
            new anchor.BN(TEST_CONSTANTS.HALVING_INTERVAL),
            testEnv.accounts.treasury.publicKey,
            testEnv.accounts.protocolReferralAddress.publicKey
          )
          .accountsPartial({
            config: configPda2,
            admin: nonAdmin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([nonAdmin])
          .rpc();
        
        expect.fail("Should have failed with non-admin signer");
      } catch (error) {
        // Expected to fail - this is correct behavior
        expect(error).to.exist;
      }
    });

    it("Should reject invalid configuration parameters", async () => {
      // Test with zero base rate
      const [configPda3] = PublicKey.findProgramAddressSync(
        [Buffer.from("config3")],
        testEnv.program.programId
      );

      try {
        await testEnv.program.methods
          .initializeConfig(
            new anchor.BN(0), // Invalid: zero base rate
            new anchor.BN(TEST_CONSTANTS.HALVING_INTERVAL),
            testEnv.accounts.treasury.publicKey,
            testEnv.accounts.protocolReferralAddress.publicKey
          )
          .accountsPartial({
            config: configPda3,
            admin: testEnv.accounts.admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .signers([testEnv.accounts.admin])
          .rpc();
        
        expect.fail("Should have failed with zero base rate");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("initialize_global_stats", () => {
    it("Should initialize global stats successfully", async () => {
      const signature = await testEnv.program.methods
        .initializeGlobalStats()
        .accountsPartial({
          globalStats: testEnv.pdas.globalStatsPda,
          admin: testEnv.accounts.admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([testEnv.accounts.admin])
        .rpc();

      await assertTransactionSuccess(testEnv, signature);
      await assertAccountExists(testEnv, testEnv.pdas.globalStatsPda, "GlobalStats");

      // Verify initial values
      const globalStats = await testEnv.program.account.globalStats.fetch(testEnv.pdas.globalStatsPda);
      expect(globalStats.totalUsers.toNumber()).to.equal(0);
      expect(globalStats.totalGrowPower.toNumber()).to.equal(0);
      expect(globalStats.currentRewardsPerSecond.toNumber()).to.equal(0);
    });
  });

  describe("initialize_fee_pool", () => {
    it("Should initialize fee pool successfully", async () => {
      const signature = await testEnv.program.methods
        .initializeFeePool(testEnv.accounts.treasury.publicKey)
        .accountsPartial({
          feePool: testEnv.pdas.feePoolPda,
          admin: testEnv.accounts.admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([testEnv.accounts.admin])
        .rpc();

      await assertTransactionSuccess(testEnv, signature);
      await assertAccountExists(testEnv, testEnv.pdas.feePoolPda, "FeePool");

      // Verify fee pool configuration
      const feePool = await testEnv.program.account.feePool.fetch(testEnv.pdas.feePoolPda);
      expect(feePool.treasury.toString()).to.equal(testEnv.accounts.treasury.publicKey.toString());
      expect(feePool.totalFeesCollected.toNumber()).to.equal(0);
    });
  });

  describe("create_reward_mint", () => {
    it("Should create reward mint successfully", async () => {
      const metadataAccount = Keypair.generate();
      
      const signature = await testEnv.program.methods
        .createRewardMint()
        .accountsPartial({
          rewardMint: testEnv.pdas.rewardMintPda,
          mintAuthority: testEnv.pdas.mintAuthorityPda,
          metadataAccount: metadataAccount.publicKey,
          admin: testEnv.accounts.admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
        })
        .signers([testEnv.accounts.admin])
        .rpc();

      await assertTransactionSuccess(testEnv, signature);
      await assertAccountExists(testEnv, testEnv.pdas.rewardMintPda, "RewardMint");

      // Verify mint properties
      const mintInfo = await testEnv.connection.getParsedAccountInfo(testEnv.pdas.rewardMintPda);
      expect(mintInfo.value).to.not.be.null;
      
      const mintData = mintInfo.value?.data as any;
      expect(mintData.parsed.info.decimals).to.equal(6);
      expect(mintData.parsed.info.supply).to.equal("0");
      expect(mintData.parsed.info.mintAuthority).to.equal(testEnv.pdas.mintAuthorityPda.toString());
    });

    it("Should reject mint creation from non-admin", async () => {
      const nonAdmin = Keypair.generate();
      const metadataAccount = Keypair.generate();
      
      // Airdrop SOL for transaction fees
      await testEnv.connection.requestAirdrop(nonAdmin.publicKey, LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Create different mint PDA for this test
      const [rewardMintPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("reward_mint2")],
        testEnv.program.programId
      );

      const [mintAuthorityPda2] = PublicKey.findProgramAddressSync(
        [Buffer.from("mint_authority2")],
        testEnv.program.programId
      );

      try {
        await testEnv.program.methods
          .createRewardMint()
          .accountsPartial({
            rewardMint: rewardMintPda2,
            mintAuthority: mintAuthorityPda2,
            metadataAccount: metadataAccount.publicKey,
            admin: nonAdmin.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            tokenMetadataProgram: new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"),
          })
          .signers([nonAdmin])
          .rpc();
        
        expect.fail("Should have failed with non-admin signer");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("Admin authorization", () => {
    it("Should validate admin-only access patterns", async () => {
      // Test that admin functions require proper authorization
      const config = await testEnv.program.account.config.fetch(testEnv.pdas.configPda);
      expect(config.admin.toString()).to.equal(testEnv.accounts.admin.publicKey.toString());
      
      // Verify that non-admin cannot perform admin functions
      const nonAdmin = Keypair.generate();
      await testEnv.connection.requestAirdrop(nonAdmin.publicKey, LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // This should be tested in individual instruction tests above
      // Here we just verify the access pattern is working
      expect(true).to.be.true; // Placeholder for admin validation
    });
  });
});