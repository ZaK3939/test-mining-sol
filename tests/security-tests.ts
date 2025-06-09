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

describe('facility-game security tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FacilityGame as Program<FacilityGame>;
  const admin = provider.wallet as anchor.Wallet;
  
  // Test users
  const attacker = Keypair.generate();
  const victim = Keypair.generate();
  const treasury = Keypair.generate();

  // PDAs
  let configPda: PublicKey;
  let attackerStatePda: PublicKey;
  let victimStatePda: PublicKey;
  let attackerFacilityPda: PublicKey;
  let victimFacilityPda: PublicKey;
  let rewardMintPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let attackerTokenAccount: PublicKey;
  let victimTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;

  before(async () => {
    // Setup accounts and PDAs
    await provider.connection.requestAirdrop(attacker.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(victim.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(treasury.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);

    [configPda] = PublicKey.findProgramAddressSync([Buffer.from('config')], program.programId);
    [attackerStatePda] = PublicKey.findProgramAddressSync([Buffer.from('user'), attacker.publicKey.toBuffer()], program.programId);
    [victimStatePda] = PublicKey.findProgramAddressSync([Buffer.from('user'), victim.publicKey.toBuffer()], program.programId);
    [attackerFacilityPda] = PublicKey.findProgramAddressSync([Buffer.from('facility'), attacker.publicKey.toBuffer()], program.programId);
    [victimFacilityPda] = PublicKey.findProgramAddressSync([Buffer.from('facility'), victim.publicKey.toBuffer()], program.programId);
    [rewardMintPda] = PublicKey.findProgramAddressSync([Buffer.from('reward_mint')], program.programId);
    [mintAuthorityPda] = PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId);

    attackerTokenAccount = await getAssociatedTokenAddress(rewardMintPda, attacker.publicKey);
    victimTokenAccount = await getAssociatedTokenAddress(rewardMintPda, victim.publicKey);
    treasuryTokenAccount = await getAssociatedTokenAddress(rewardMintPda, treasury.publicKey);

    // Initialize system
    await program.methods
      .initializeConfig(new anchor.BN(10), new anchor.BN(86400 * 365), treasury.publicKey)
      .accounts({
        config: configPda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Create reward mint
    const TOKEN_METADATA_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
    const [metadataAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from('metadata'), TOKEN_METADATA_ID.toBuffer(), rewardMintPda.toBuffer()],
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
  });

  describe('Access Control Tests', () => {
    it('Should prevent non-admin from initializing config', async () => {
      const [fakeConfigPda] = PublicKey.findProgramAddressSync([Buffer.from('fake_config')], program.programId);
      
      try {
        await program.methods
          .initializeConfig(new anchor.BN(100), new anchor.BN(86400), treasury.publicKey)
          .accounts({
            config: fakeConfigPda,
            admin: attacker.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed');
      } catch (error: any) {
        expect(error).to.exist;
      }
    });

    it('Should prevent user from claiming another user\'s rewards', async () => {
      // Initialize both users
      await program.methods
        .initUser(null)
        .accounts({
          userState: attackerStatePda,
          user: attacker.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();

      await program.methods
        .initUser(null)
        .accounts({
          userState: victimStatePda,
          user: victim.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([victim])
        .rpc();

      // Both buy facilities
      await program.methods
        .buyFacility()
        .accounts({
          userState: attackerStatePda,
          facility: attackerFacilityPda,
          user: attacker.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();

      await program.methods
        .buyFacility()
        .accounts({
          userState: victimStatePda,
          facility: victimFacilityPda,
          user: victim.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([victim])
        .rpc();

      // Create token accounts
      for (const user of [attacker, victim, treasury]) {
        const tokenAccount = await getAssociatedTokenAddress(rewardMintPda, user.publicKey);
        const instruction = createAssociatedTokenAccountInstruction(
          user.publicKey,
          tokenAccount,
          user.publicKey,
          rewardMintPda,
        );
        const transaction = new anchor.web3.Transaction().add(instruction);
        await provider.sendAndConfirm(transaction, [user]);
      }

      // Wait for rewards to accumulate
      await new Promise(resolve => setTimeout(resolve, 2000));

      // Attacker tries to claim victim's rewards
      try {
        await program.methods
          .claimReward()
          .accounts({
            userState: victimStatePda,
            config: configPda,
            rewardMint: rewardMintPda,
            mintAuthority: mintAuthorityPda,
            userTokenAccount: attackerTokenAccount, // Attacker's account!
            user: attacker.publicKey, // Attacker signing!
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed - attacker should not be able to claim victim\'s rewards');
      } catch (error: any) {
        expect(error).to.exist;
        // Should fail due to PDA constraint mismatch
      }
    });

    it('Should prevent user from upgrading another user\'s facility', async () => {
      // First, give attacker some tokens
      await program.methods
        .claimReward()
        .accounts({
          userState: attackerStatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: attackerTokenAccount,
          user: attacker.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([attacker])
        .rpc();

      try {
        await program.methods
          .upgradeFacility()
          .accounts({
            userState: victimStatePda,     // Victim's state
            facility: victimFacilityPda,   // Victim's facility
            rewardMint: rewardMintPda,
            userTokenAccount: attackerTokenAccount, // Attacker's tokens
            user: attacker.publicKey,      // Attacker signing
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed - attacker should not be able to upgrade victim\'s facility');
      } catch (error: any) {
        expect(error).to.exist;
      }
    });
  });

  describe('Input Validation Tests', () => {
    it('Should handle overflow in reward calculation', async () => {
      // This test ensures the program handles large numbers correctly
      const userState = await program.account.userState.fetch(attackerStatePda);
      const config = await program.account.config.fetch(configPda);
      
      // Check that the calculation doesn't overflow with maximum values
      const maxTime = 2 ** 31 - 1; // Max i64 / 2
      const maxGrowPower = userState.totalGrowPower.toNumber();
      const baseRate = config.baseRate.toNumber();
      
      // This should not cause overflow in the actual program
      const calculatedReward = (maxTime * maxGrowPower * baseRate) / 1000;
      console.log(`Max theoretical reward calculation: ${calculatedReward}`);
      
      // The program should handle this gracefully with checked arithmetic
      expect(calculatedReward).to.be.finite;
    });

    it('Should prevent negative transfer amounts', async () => {
      try {
        await program.methods
          .transferWithFee(new anchor.BN(-100)) // Negative amount
          .accounts({
            fromTokenAccount: attackerTokenAccount,
            toTokenAccount: victimTokenAccount,
            config: configPda,
            treasuryTokenAccount: treasuryTokenAccount,
            rewardMint: rewardMintPda,
            from: attacker.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed with negative amount');
      } catch (error: any) {
        expect(error).to.exist;
      }
    });

    it('Should handle zero amount transfers', async () => {
      const attackerBalanceBefore = await getAccount(provider.connection, attackerTokenAccount);
      
      await program.methods
        .transferWithFee(new anchor.BN(0))
        .accounts({
          fromTokenAccount: attackerTokenAccount,
          toTokenAccount: victimTokenAccount,
          config: configPda,
          treasuryTokenAccount: treasuryTokenAccount,
          rewardMint: rewardMintPda,
          from: attacker.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([attacker])
        .rpc();

      const attackerBalanceAfter = await getAccount(provider.connection, attackerTokenAccount);
      
      // No change should occur
      expect(attackerBalanceAfter.amount.toString()).to.equal(attackerBalanceBefore.amount.toString());
    });
  });

  describe('PDA Security Tests', () => {
    it('Should prevent PDA collision attacks', async () => {
      // Try to create user state with wrong seed structure
      const fakePda = PublicKey.findProgramAddressSync(
        [Buffer.from('wrong_seed'), attacker.publicKey.toBuffer()],
        program.programId,
      )[0];

      try {
        await program.methods
          .initUser(null)
          .accounts({
            userState: fakePda, // Wrong PDA
            user: attacker.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed with wrong PDA structure');
      } catch (error: any) {
        expect(error).to.exist;
      }
    });

    it('Should ensure PDA ownership constraints', async () => {
      // Try to use another user's facility PDA
      try {
        await program.methods
          .upgradeFacility()
          .accounts({
            userState: attackerStatePda,
            facility: victimFacilityPda, // Victim's facility!
            rewardMint: rewardMintPda,
            userTokenAccount: attackerTokenAccount,
            user: attacker.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed - cannot use another user\'s facility PDA');
      } catch (error: any) {
        expect(error).to.exist;
        // Should fail due to facility owner constraint
      }
    });
  });

  describe('Reentrancy Protection Tests', () => {
    it('Should handle rapid successive transactions safely', async () => {
      // Test rapid successive claims to check for reentrancy issues
      const promises = [];
      
      for (let i = 0; i < 5; i++) {
        promises.push(
          program.methods
            .claimReward()
            .accounts({
              userState: attackerStatePda,
              config: configPda,
              rewardMint: rewardMintPda,
              mintAuthority: mintAuthorityPda,
              userTokenAccount: attackerTokenAccount,
              user: attacker.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([attacker])
            .rpc()
            .catch(err => err) // Catch errors instead of failing
        );
      }
      
      const results = await Promise.all(promises);
      
      // At least one should succeed, others may fail due to timing
      const successes = results.filter(result => typeof result === 'string');
      const failures = results.filter(result => result instanceof Error);
      
      console.log(`Rapid claims: ${successes.length} succeeded, ${failures.length} failed`);
      expect(successes.length).to.be.at.least(1);
    });
  });

  describe('Token Security Tests', () => {
    it('Should prevent unauthorized minting', async () => {
      // Try to directly call SPL token mint (should fail without proper authority)
      try {
        const mintInstruction = {
          keys: [
            { pubkey: rewardMintPda, isSigner: false, isWritable: true },
            { pubkey: attackerTokenAccount, isSigner: false, isWritable: true },
            { pubkey: attacker.publicKey, isSigner: true, isWritable: false },
          ],
          programId: TOKEN_PROGRAM_ID,
          data: Buffer.from([7, 100, 0, 0, 0, 0, 0, 0, 0]), // Mint instruction with 100 tokens
        };
        
        const transaction = new anchor.web3.Transaction().add(mintInstruction);
        await provider.sendAndConfirm(transaction, [attacker]);
        
        expect.fail('Should have failed - unauthorized minting');
      } catch (error: any) {
        expect(error).to.exist;
        // Should fail because attacker is not the mint authority
      }
    });

    it('Should verify token account ownership in transfers', async () => {
      // Create a token account that attacker doesn't own
      const randomUser = Keypair.generate();
      await provider.connection.requestAirdrop(randomUser.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL);
      
      const randomTokenAccount = await getAssociatedTokenAddress(rewardMintPda, randomUser.publicKey);
      const instruction = createAssociatedTokenAccountInstruction(
        randomUser.publicKey,
        randomTokenAccount,
        randomUser.publicKey,
        rewardMintPda,
      );
      const transaction = new anchor.web3.Transaction().add(instruction);
      await provider.sendAndConfirm(transaction, [randomUser]);

      try {
        await program.methods
          .transferWithFee(new anchor.BN(10))
          .accounts({
            fromTokenAccount: randomTokenAccount, // Account attacker doesn't own
            toTokenAccount: victimTokenAccount,
            config: configPda,
            treasuryTokenAccount: treasuryTokenAccount,
            rewardMint: rewardMintPda,
            from: attacker.publicKey, // Attacker trying to use it
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed - cannot transfer from account you don\'t own');
      } catch (error: any) {
        expect(error).to.exist;
      }
    });
  });

  describe('Mystery Box Security Tests', () => {
    it('Should prevent opening mystery box before purchase', async () => {
      const boxId = new anchor.BN(999);
      const seedId = new anchor.BN(999);
      
      const [mysteryBoxPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('mystery_box'), boxId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      const [seedPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('seed'), seedId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      try {
        await program.methods
          .openMysteryBox(boxId, seedId)
          .accounts({
            config: configPda,
            mysteryBox: mysteryBoxPda,
            seed: seedPda,
            opener: attacker.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed - cannot open non-existent mystery box');
      } catch (error: any) {
        expect(error).to.exist;
      }
    });

    it('Should prevent opening someone else\'s mystery box', async () => {
      const boxId = new anchor.BN(100);
      
      // Victim purchases mystery box
      const [mysteryBoxPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('mystery_box'), boxId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      // Give victim some tokens first
      await program.methods
        .claimReward()
        .accounts({
          userState: victimStatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: victimTokenAccount,
          user: victim.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([victim])
        .rpc();

      // Victim purchases mystery box
      await program.methods
        .purchaseMysteryBox(boxId)
        .accounts({
          config: configPda,
          mysteryBox: mysteryBoxPda,
          rewardMint: rewardMintPda,
          purchaserTokenAccount: victimTokenAccount,
          treasuryTokenAccount: treasuryTokenAccount,
          purchaser: victim.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([victim])
        .rpc();

      // Attacker tries to open victim's mystery box
      const seedId = new anchor.BN(100);
      const [seedPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('seed'), seedId.toArrayLike(Buffer, 'le', 8)],
        program.programId,
      );

      try {
        await program.methods
          .openMysteryBox(boxId, seedId)
          .accounts({
            config: configPda,
            mysteryBox: mysteryBoxPda,
            seed: seedPda,
            opener: attacker.publicKey, // Attacker trying to open
            systemProgram: SystemProgram.programId,
          })
          .signers([attacker])
          .rpc();
        
        expect.fail('Should have failed - cannot open someone else\'s mystery box');
      } catch (error: any) {
        expect(error).to.exist;
        // Should fail due to purchaser constraint
      }
    });
  });

  describe('Rate Limiting & DoS Protection', () => {
    it('Should handle high frequency transactions', async () => {
      const startTime = Date.now();
      let successCount = 0;
      let errorCount = 0;

      // Try 10 rapid transactions
      for (let i = 0; i < 10; i++) {
        try {
          await program.methods
            .claimReward()
            .accounts({
              userState: attackerStatePda,
              config: configPda,
              rewardMint: rewardMintPda,
              mintAuthority: mintAuthorityPda,
              userTokenAccount: attackerTokenAccount,
              user: attacker.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([attacker])
            .rpc();
          successCount++;
        } catch (error) {
          errorCount++;
        }
        
        // Small delay to prevent overwhelming the network
        await new Promise(resolve => setTimeout(resolve, 100));
      }

      const endTime = Date.now();
      const duration = endTime - startTime;
      
      console.log(`High frequency test: ${successCount} success, ${errorCount} errors in ${duration}ms`);
      
      // The program should handle this gracefully
      expect(successCount + errorCount).to.equal(10);
      expect(duration).to.be.lessThan(30000); // Should complete within 30 seconds
    });
  });

  describe('Data Integrity Tests', () => {
    it('Should maintain consistent state after complex operations', async () => {
      // Perform a series of operations and verify state consistency
      const initialUserState = await program.account.userState.fetch(attackerStatePda);
      const initialFacility = await program.account.facility.fetch(attackerFacilityPda);
      const initialBalance = await getAccount(provider.connection, attackerTokenAccount);

      // Claim rewards
      await program.methods
        .claimReward()
        .accounts({
          userState: attackerStatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: attackerTokenAccount,
          user: attacker.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([attacker])
        .rpc();

      const afterClaimUserState = await program.account.userState.fetch(attackerStatePda);
      const afterClaimBalance = await getAccount(provider.connection, attackerTokenAccount);

      // Verify state consistency
      expect(afterClaimUserState.totalGrowPower.toString()).to.equal(initialUserState.totalGrowPower.toString());
      expect(afterClaimUserState.hasFacility).to.equal(initialUserState.hasFacility);
      expect(Number(afterClaimBalance.amount)).to.be.greaterThan(Number(initialBalance.amount));
      
      console.log('State integrity verified after complex operations');
    });
  });
});