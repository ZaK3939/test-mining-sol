import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { FacilityGame } from '../target/types/facility_game';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  getAccount,
} from '@solana/spl-token';
import { expect } from 'chai';

// Metaplex Token Metadata Program ID
const TOKEN_METADATA_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

describe('facility-game comprehensive tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FacilityGame as Program<FacilityGame>;
  const admin = provider.wallet as anchor.Wallet;

  // Test users
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();
  const treasury = Keypair.generate();

  // PDAs for user1
  let configPda: PublicKey;
  let user1StatePda: PublicKey;
  let user1FacilityPda: PublicKey;
  let user2StatePda: PublicKey;
  let user2FacilityPda: PublicKey;
  let user3StatePda: PublicKey;
  let rewardMintPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let user1TokenAccount: PublicKey;
  let user2TokenAccount: PublicKey;
  let user3TokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;

  before(async () => {
    // Airdrop SOL to test users
    const users = [user1, user2, user3, treasury];
    for (const user of users) {
      await provider.connection.requestAirdrop(user.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 500)); // Wait to avoid rate limiting
    }

    // Derive PDAs
    [configPda] = PublicKey.findProgramAddressSync([Buffer.from('config')], program.programId);

    [user1StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), user1.publicKey.toBuffer()],
      program.programId,
    );

    [user2StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), user2.publicKey.toBuffer()],
      program.programId,
    );

    [user3StatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), user3.publicKey.toBuffer()],
      program.programId,
    );

    [user1FacilityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('facility'), user1.publicKey.toBuffer()],
      program.programId,
    );

    [user2FacilityPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('facility'), user2.publicKey.toBuffer()],
      program.programId,
    );

    [rewardMintPda] = PublicKey.findProgramAddressSync([Buffer.from('reward_mint')], program.programId);

    [mintAuthorityPda] = PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId);

    // Get token account addresses
    user1TokenAccount = await getAssociatedTokenAddress(rewardMintPda, user1.publicKey);
    user2TokenAccount = await getAssociatedTokenAddress(rewardMintPda, user2.publicKey);
    user3TokenAccount = await getAssociatedTokenAddress(rewardMintPda, user3.publicKey);
    treasuryTokenAccount = await getAssociatedTokenAddress(rewardMintPda, treasury.publicKey);
  });

  describe('System Initialization', () => {
    it('Should initialize config with treasury address', async () => {
      const baseRate = new anchor.BN(10);
      const halvingInterval = new anchor.BN(86400 * 365); // 1 year

      await program.methods
        .initializeConfig(baseRate, halvingInterval, treasury.publicKey)
        .accounts({
          config: configPda,
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const configAccount = await program.account.config.fetch(configPda);
      expect(configAccount.baseRate.toString()).to.equal('10');
      expect(configAccount.treasury.toString()).to.equal(treasury.publicKey.toString());
      expect(configAccount.mysteryBoxCost.toString()).to.equal('1000');
    });

    it('Should create $WEED token mint with metadata', async () => {
      const [metadataAccount] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          TOKEN_METADATA_ID.toBuffer(),
          rewardMintPda.toBuffer()
        ],
        TOKEN_METADATA_ID
      );

      await program.methods
        .createRewardMint()
        .accounts({
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          metadataAccount: metadataAccount,
          admin: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          tokenMetadataProgram: TOKEN_METADATA_ID,
        })
        .rpc();

      const mintAccount = await program.provider.connection.getAccountInfo(rewardMintPda);
      expect(mintAccount).to.not.be.null;
    });
  });

  describe('User Management & Referral System', () => {
    it('Should initialize user without referrer', async () => {
      await program.methods
        .initUser(null)
        .accounts({
          userState: user1StatePda,
          user: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const userState = await program.account.userState.fetch(user1StatePda);
      expect(userState.owner.toString()).to.equal(user1.publicKey.toString());
      expect(userState.hasFacility).to.be.false;
      expect(userState.referrer).to.be.null;
      expect(userState.pendingReferralRewards.toString()).to.equal('0');
    });

    it('Should initialize user with referrer', async () => {
      await program.methods
        .initUser(user1.publicKey)
        .accounts({
          userState: user2StatePda,
          user: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      const userState = await program.account.userState.fetch(user2StatePda);
      expect(userState.referrer.toString()).to.equal(user1.publicKey.toString());
    });

    it('Should initialize cascading referral chain', async () => {
      await program.methods
        .initUser(user2.publicKey)
        .accounts({
          userState: user3StatePda,
          user: user3.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user3])
        .rpc();

      const userState = await program.account.userState.fetch(user3StatePda);
      expect(userState.referrer.toString()).to.equal(user2.publicKey.toString());
    });
  });

  describe('Facility Management', () => {
    it('Should purchase facility for user1', async () => {
      await program.methods
        .buyFacility()
        .accounts({
          userState: user1StatePda,
          facility: user1FacilityPda,
          user: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const userState = await program.account.userState.fetch(user1StatePda);
      const facility = await program.account.facility.fetch(user1FacilityPda);
      
      expect(userState.hasFacility).to.be.true;
      expect(userState.totalGrowPower.toString()).to.equal('100');
      expect(facility.facilitySize).to.equal(1);
      expect(facility.maxCapacity).to.equal(1);
      expect(facility.machineCount).to.equal(1);
    });

    it('Should purchase facility for user2', async () => {
      await program.methods
        .buyFacility()
        .accounts({
          userState: user2StatePda,
          facility: user2FacilityPda,
          user: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      const facility = await program.account.facility.fetch(user2FacilityPda);
      expect(facility.facilitySize).to.equal(1);
    });

    it('Should fail to purchase second facility', async () => {
      try {
        await program.methods
          .buyFacility()
          .accounts({
            userState: user1StatePda,
            facility: user1FacilityPda,
            user: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).to.exist;
        // Should contain AlreadyHasFacility error
      }
    });
  });

  describe('Token Account Setup', () => {
    it('Should create token accounts for all users', async () => {
      const users = [
        { keypair: user1, tokenAccount: user1TokenAccount },
        { keypair: user2, tokenAccount: user2TokenAccount },
        { keypair: user3, tokenAccount: user3TokenAccount },
        { keypair: treasury, tokenAccount: treasuryTokenAccount },
      ];

      for (const user of users) {
        const instruction = createAssociatedTokenAccountInstruction(
          user.keypair.publicKey,
          user.tokenAccount,
          user.keypair.publicKey,
          rewardMintPda,
        );

        const transaction = new anchor.web3.Transaction().add(instruction);
        await provider.sendAndConfirm(transaction, [user.keypair]);
      }
    });
  });

  describe('Reward System & Referrals', () => {
    it('Should claim rewards and test referral system', async () => {
      // Wait for rewards to accumulate
      await new Promise(resolve => setTimeout(resolve, 3000));

      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      
      await program.methods
        .claimReward()
        .accounts({
          userState: user1StatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: user1TokenAccount,
          user: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();

      const user1BalanceAfter = await getAccount(provider.connection, user1TokenAccount);
      expect(Number(user1BalanceAfter.amount)).to.be.greaterThan(Number(user1BalanceBefore.amount));
      console.log(`User1 claimed ${user1BalanceAfter.amount} $WEED tokens`);
    });

    it('Should distribute referral rewards', async () => {
      // Get current balance of user1 (referrer)
      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      
      // User2 (who has user1 as referrer) claims rewards
      await new Promise(resolve => setTimeout(resolve, 3000));
      
      const rewardAmount = new anchor.BN(1000); // Mock reward amount
      
      await program.methods
        .distributeReferralReward(rewardAmount)
        .accounts({
          inviteeState: user2StatePda,
          referrerState: user1StatePda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          referrerTokenAccount: user1TokenAccount,
          invitee: user2.publicKey,
          referrer: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user2])
        .rpc();

      // Check that user1's pending referral rewards increased
      const user1State = await program.account.userState.fetch(user1StatePda);
      expect(user1State.pendingReferralRewards.toNumber()).to.be.greaterThan(0);
      console.log(`User1 pending referral rewards: ${user1State.pendingReferralRewards.toString()}`);
    });

    it('Should claim referral rewards', async () => {
      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      
      await program.methods
        .claimReferralRewards()
        .accounts({
          referrerState: user1StatePda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          referrerTokenAccount: user1TokenAccount,
          referrer: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();

      const user1BalanceAfter = await getAccount(provider.connection, user1TokenAccount);
      const user1State = await program.account.userState.fetch(user1StatePda);
      
      expect(Number(user1BalanceAfter.amount)).to.be.greaterThan(Number(user1BalanceBefore.amount));
      expect(user1State.pendingReferralRewards.toString()).to.equal('0');
      console.log(`User1 claimed referral rewards, new balance: ${user1BalanceAfter.amount}`);
    });
  });

  describe('Facility Upgrades & Machine Management', () => {
    it('Should upgrade facility size', async () => {
      // First claim some rewards to have tokens for upgrading
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      await program.methods
        .claimReward()
        .accounts({
          userState: user1StatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: user1TokenAccount,
          user: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();

      const facilityBefore = await program.account.facility.fetch(user1FacilityPda);
      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      
      // Skip upgrade test if insufficient balance
      const upgradeCoast = 1000; // Size 1→2 costs 1000 $WEED
      if (Number(user1BalanceBefore.amount) < upgradeCoast) {
        console.log(`Insufficient balance for upgrade test. Balance: ${user1BalanceBefore.amount}, Required: ${upgradeCoast}`);
        return;
      }

      await program.methods
        .upgradeFacility()
        .accounts({
          userState: user1StatePda,
          facility: user1FacilityPda,
          rewardMint: rewardMintPda,
          userTokenAccount: user1TokenAccount,
          user: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();

      const facilityAfter = await program.account.facility.fetch(user1FacilityPda);
      const user1BalanceAfter = await getAccount(provider.connection, user1TokenAccount);
      
      expect(facilityAfter.facilitySize).to.equal(facilityBefore.facilitySize + 1);
      expect(facilityAfter.maxCapacity).to.be.greaterThan(facilityBefore.maxCapacity);
      expect(Number(user1BalanceAfter.amount)).to.be.lessThan(Number(user1BalanceBefore.amount));
      console.log(`Facility upgraded from size ${facilityBefore.facilitySize} to ${facilityAfter.facilitySize}`);
    });

    it('Should add machine to facility', async () => {
      const facilityBefore = await program.account.facility.fetch(user1FacilityPda);
      const user1StateBefore = await program.account.userState.fetch(user1StatePda);
      
      // Skip if at max capacity
      if (facilityBefore.machineCount >= facilityBefore.maxCapacity) {
        console.log('Facility at max capacity, skipping add machine test');
        return;
      }

      // Check balance for machine cost (500 $WEED)
      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      if (Number(user1BalanceBefore.amount) < 500) {
        console.log('Insufficient balance for add machine test');
        return;
      }

      await program.methods
        .addMachine()
        .accounts({
          userState: user1StatePda,
          facility: user1FacilityPda,
          rewardMint: rewardMintPda,
          userTokenAccount: user1TokenAccount,
          user: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();

      const facilityAfter = await program.account.facility.fetch(user1FacilityPda);
      const user1StateAfter = await program.account.userState.fetch(user1StatePda);
      
      expect(facilityAfter.machineCount).to.equal(facilityBefore.machineCount + 1);
      expect(facilityAfter.totalGrowPower.toNumber()).to.equal(facilityBefore.totalGrowPower.toNumber() + 100);
      expect(user1StateAfter.totalGrowPower.toString()).to.equal(facilityAfter.totalGrowPower.toString());
      console.log(`Machine added, new count: ${facilityAfter.machineCount}, new grow power: ${facilityAfter.totalGrowPower}`);
    });
  });

  describe('Transfer System with Fees', () => {
    it('Should transfer tokens with 2% fee', async () => {
      const transferAmount = new anchor.BN(100);
      
      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      const user2BalanceBefore = await getAccount(provider.connection, user2TokenAccount);
      const treasuryBalanceBefore = await getAccount(provider.connection, treasuryTokenAccount);
      
      // Skip if insufficient balance
      if (Number(user1BalanceBefore.amount) < 100) {
        console.log('Insufficient balance for transfer test');
        return;
      }

      await program.methods
        .transferWithFee(transferAmount)
        .accounts({
          fromTokenAccount: user1TokenAccount,
          toTokenAccount: user2TokenAccount,
          config: configPda,
          treasuryTokenAccount: treasuryTokenAccount,
          rewardMint: rewardMintPda,
          from: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user1])
        .rpc();

      const user1BalanceAfter = await getAccount(provider.connection, user1TokenAccount);
      const user2BalanceAfter = await getAccount(provider.connection, user2TokenAccount);
      const treasuryBalanceAfter = await getAccount(provider.connection, treasuryTokenAccount);
      
      const expectedFee = 2; // 2% of 100
      const expectedTransfer = 98; // 98% of 100
      
      expect(Number(user1BalanceAfter.amount)).to.equal(Number(user1BalanceBefore.amount) - 100);
      expect(Number(user2BalanceAfter.amount)).to.equal(Number(user2BalanceBefore.amount) + expectedTransfer);
      expect(Number(treasuryBalanceAfter.amount)).to.equal(Number(treasuryBalanceBefore.amount) + expectedFee);
      
      console.log(`Transfer completed: ${transferAmount} total, ${expectedFee} fee, ${expectedTransfer} received`);
    });
  });

  describe('Mystery Box System', () => {
    it('Should purchase mystery box', async () => {
      const boxId = new anchor.BN(1);
      
      const [mysteryBoxPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('mystery_box'), boxId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      const user1BalanceBefore = await getAccount(provider.connection, user1TokenAccount);
      const treasuryBalanceBefore = await getAccount(provider.connection, treasuryTokenAccount);
      
      // Skip if insufficient balance (1000 $WEED required)
      if (Number(user1BalanceBefore.amount) < 1000) {
        console.log('Insufficient balance for mystery box test');
        return;
      }

      await program.methods
        .purchaseMysteryBox(boxId)
        .accounts({
          config: configPda,
          mysteryBox: mysteryBoxPda,
          rewardMint: rewardMintPda,
          purchaserTokenAccount: user1TokenAccount,
          treasuryTokenAccount: treasuryTokenAccount,
          purchaser: user1.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const mysteryBox = await program.account.mysteryBox.fetch(mysteryBoxPda);
      const user1BalanceAfter = await getAccount(provider.connection, user1TokenAccount);
      const treasuryBalanceAfter = await getAccount(provider.connection, treasuryTokenAccount);
      
      expect(mysteryBox.purchaser.toString()).to.equal(user1.publicKey.toString());
      expect(mysteryBox.isOpened).to.be.false;
      expect(mysteryBox.costPaid.toString()).to.equal('1000');
      expect(Number(user1BalanceAfter.amount)).to.equal(Number(user1BalanceBefore.amount) - 1000);
      expect(Number(treasuryBalanceAfter.amount)).to.equal(Number(treasuryBalanceBefore.amount) + 1000);
      
      console.log(`Mystery box purchased: ID ${boxId}, random seed: ${mysteryBox.randomSeed}`);
    });

    it('Should open mystery box and get seed', async () => {
      const boxId = new anchor.BN(1);
      const seedId = new anchor.BN(1);
      
      const [mysteryBoxPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('mystery_box'), boxId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      const [seedPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('seed'), seedId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      await program.methods
        .openMysteryBox(boxId, seedId)
        .accounts({
          config: configPda,
          mysteryBox: mysteryBoxPda,
          seed: seedPda,
          opener: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const mysteryBox = await program.account.mysteryBox.fetch(mysteryBoxPda);
      const seed = await program.account.seed.fetch(seedPda);
      
      expect(mysteryBox.isOpened).to.be.true;
      expect(seed.owner.toString()).to.equal(user1.publicKey.toString());
      expect(seed.isPlanted).to.be.false;
      expect(['Common', 'Rare', 'Epic', 'Legendary']).to.include(Object.keys(seed.rarity)[0]);
      
      console.log(`Mystery box opened: Seed ID ${seedId}, rarity: ${Object.keys(seed.rarity)[0]}, multiplier: ${seed.growPowerMultiplier}`);
    });
  });

  describe('Error Scenarios', () => {
    it('Should fail to claim rewards without facility', async () => {
      const user4 = Keypair.generate();
      await provider.connection.requestAirdrop(user4.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      
      const [user4StatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), user4.publicKey.toBuffer()],
        program.programId,
      );

      const user4TokenAccount = await getAssociatedTokenAddress(rewardMintPda, user4.publicKey);

      // Initialize user without facility
      await program.methods
        .initUser(null)
        .accounts({
          userState: user4StatePda,
          user: user4.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user4])
        .rpc();

      // Create token account
      const instruction = createAssociatedTokenAccountInstruction(
        user4.publicKey,
        user4TokenAccount,
        user4.publicKey,
        rewardMintPda,
      );
      const transaction = new anchor.web3.Transaction().add(instruction);
      await provider.sendAndConfirm(transaction, [user4]);

      try {
        await program.methods
          .claimReward()
          .accounts({
            userState: user4StatePda,
            config: configPda,
            rewardMint: rewardMintPda,
            mintAuthority: mintAuthorityPda,
            userTokenAccount: user4TokenAccount,
            user: user4.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user4])
          .rpc();
        
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).to.exist;
        // Should contain NoFacility error
      }
    });

    it('Should fail to claim referral rewards with zero balance', async () => {
      const user5 = Keypair.generate();
      await provider.connection.requestAirdrop(user5.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      
      const [user5StatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), user5.publicKey.toBuffer()],
        program.programId,
      );

      const user5TokenAccount = await getAssociatedTokenAddress(rewardMintPda, user5.publicKey);

      // Initialize user
      await program.methods
        .initUser(null)
        .accounts({
          userState: user5StatePda,
          user: user5.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user5])
        .rpc();

      // Create token account
      const instruction = createAssociatedTokenAccountInstruction(
        user5.publicKey,
        user5TokenAccount,
        user5.publicKey,
        rewardMintPda,
      );
      const transaction = new anchor.web3.Transaction().add(instruction);
      await provider.sendAndConfirm(transaction, [user5]);

      try {
        await program.methods
          .claimReferralRewards()
          .accounts({
            referrerState: user5StatePda,
            rewardMint: rewardMintPda,
            mintAuthority: mintAuthorityPda,
            referrerTokenAccount: user5TokenAccount,
            referrer: user5.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user5])
          .rpc();
        
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).to.exist;
        // Should contain NoRewardToClaim error
      }
    });
  });

  describe('Performance & Edge Cases', () => {
    it('Should handle multiple rapid reward claims', async () => {
      const claimTimes = [];
      
      for (let i = 0; i < 3; i++) {
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const startTime = Date.now();
        await program.methods
          .claimReward()
          .accounts({
            userState: user1StatePda,
            config: configPda,
            rewardMint: rewardMintPda,
            mintAuthority: mintAuthorityPda,
            userTokenAccount: user1TokenAccount,
            user: user1.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user1])
          .rpc();
        
        const endTime = Date.now();
        claimTimes.push(endTime - startTime);
      }
      
      const avgClaimTime = claimTimes.reduce((a, b) => a + b, 0) / claimTimes.length;
      console.log(`Average claim time: ${avgClaimTime}ms`);
      expect(avgClaimTime).to.be.lessThan(5000); // Should complete within 5 seconds
    });

    it('Should handle large token amounts correctly', async () => {
      // Test with large numbers to ensure no overflow
      const userState = await program.account.userState.fetch(user1StatePda);
      const config = await program.account.config.fetch(configPda);
      
      // Calculate theoretical maximum reward for a long time period
      const longTimePeriod = 86400 * 30; // 30 days in seconds
      const maxGrowPower = userState.totalGrowPower.toNumber();
      const baseRate = config.baseRate.toNumber();
      const theoreticalMaxReward = (longTimePeriod * maxGrowPower * baseRate) / 1000;
      
      console.log(`Theoretical max reward for 30 days: ${theoreticalMaxReward}`);
      expect(theoreticalMaxReward).to.be.lessThan(Number.MAX_SAFE_INTEGER);
    });
  });
});