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

interface PerformanceMetrics {
  instruction: string;
  averageTime: number;
  minTime: number;
  maxTime: number;
  samples: number;
  computeUnitsUsed?: number;
}

describe('facility-game performance tests', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.FacilityGame as Program<FacilityGame>;
  const admin = provider.wallet as anchor.Wallet;
  const treasury = Keypair.generate();

  // Performance metrics storage
  const metrics: PerformanceMetrics[] = [];

  // Helper function to measure execution time
  async function measurePerformance<T>(
    instruction: string,
    operation: () => Promise<T>,
    samples: number = 5
  ): Promise<PerformanceMetrics> {
    const times: number[] = [];
    
    for (let i = 0; i < samples; i++) {
      const startTime = process.hrtime.bigint();
      try {
        await operation();
        const endTime = process.hrtime.bigint();
        const duration = Number(endTime - startTime) / 1000000; // Convert to milliseconds
        times.push(duration);
      } catch (error) {
        console.warn(`Sample ${i} failed for ${instruction}:`, error.message);
      }
      
      // Small delay between samples
      await new Promise(resolve => setTimeout(resolve, 200));
    }

    if (times.length === 0) {
      throw new Error(`All samples failed for ${instruction}`);
    }

    const metric: PerformanceMetrics = {
      instruction,
      averageTime: times.reduce((a, b) => a + b, 0) / times.length,
      minTime: Math.min(...times),
      maxTime: Math.max(...times),
      samples: times.length,
    };

    metrics.push(metric);
    return metric;
  }

  // PDAs
  let configPda: PublicKey;
  let rewardMintPda: PublicKey;
  let mintAuthorityPda: PublicKey;

  before(async () => {
    await provider.connection.requestAirdrop(treasury.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);

    [configPda] = PublicKey.findProgramAddressSync([Buffer.from('config')], program.programId);
    [rewardMintPda] = PublicKey.findProgramAddressSync([Buffer.from('reward_mint')], program.programId);
    [mintAuthorityPda] = PublicKey.findProgramAddressSync([Buffer.from('mint_authority')], program.programId);
  });

  describe('Core Instruction Performance', () => {
    it('Should benchmark initialize_config performance', async () => {
      const metric = await measurePerformance(
        'initialize_config',
        async () => {
          await program.methods
            .initializeConfig(new anchor.BN(10), new anchor.BN(86400 * 365), treasury.publicKey)
            .accounts({
              config: configPda,
              admin: admin.publicKey,
              systemProgram: SystemProgram.programId,
            })
            .rpc();
        },
        1 // Only run once since it can only be initialized once
      );

      console.log(`initialize_config: avg ${metric.averageTime.toFixed(2)}ms`);
      expect(metric.averageTime).to.be.lessThan(5000); // Should complete under 5s
    });

    it('Should benchmark create_reward_mint performance', async () => {
      const TOKEN_METADATA_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
      const [metadataAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), TOKEN_METADATA_ID.toBuffer(), rewardMintPda.toBuffer()],
        TOKEN_METADATA_ID
      );

      const metric = await measurePerformance(
        'create_reward_mint',
        async () => {
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
        },
        1 // Only run once
      );

      console.log(`create_reward_mint: avg ${metric.averageTime.toFixed(2)}ms`);
      expect(metric.averageTime).to.be.lessThan(10000); // May take longer due to metadata
    });

    it('Should benchmark user operations at scale', async () => {
      const userCount = 10;
      const users: Keypair[] = [];
      const userMetrics: number[] = [];

      // Generate users
      for (let i = 0; i < userCount; i++) {
        users.push(Keypair.generate());
      }

      // Airdrop to all users in parallel
      await Promise.all(
        users.map(user => 
          provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
        )
      );

      // Benchmark user initialization
      for (let i = 0; i < userCount; i++) {
        const user = users[i];
        const [userStatePda] = PublicKey.findProgramAddressSync(
          [Buffer.from('user'), user.publicKey.toBuffer()],
          program.programId
        );

        const startTime = process.hrtime.bigint();
        
        await program.methods
          .initUser(i > 0 ? users[i - 1].publicKey : null) // Chain referrals
          .accounts({
            userState: userStatePda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        const endTime = process.hrtime.bigint();
        const duration = Number(endTime - startTime) / 1000000;
        userMetrics.push(duration);
      }

      const avgUserInit = userMetrics.reduce((a, b) => a + b, 0) / userMetrics.length;
      console.log(`Average user initialization: ${avgUserInit.toFixed(2)}ms over ${userCount} users`);
      expect(avgUserInit).to.be.lessThan(3000);

      // Benchmark facility purchases
      const facilityMetrics: number[] = [];
      for (let i = 0; i < Math.min(5, userCount); i++) {
        const user = users[i];
        const [userStatePda] = PublicKey.findProgramAddressSync(
          [Buffer.from('user'), user.publicKey.toBuffer()],
          program.programId
        );
        const [facilityPda] = PublicKey.findProgramAddressSync(
          [Buffer.from('facility'), user.publicKey.toBuffer()],
          program.programId
        );

        const startTime = process.hrtime.bigint();
        
        await program.methods
          .buyFacility()
          .accounts({
            userState: userStatePda,
            facility: facilityPda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        const endTime = process.hrtime.bigint();
        const duration = Number(endTime - startTime) / 1000000;
        facilityMetrics.push(duration);
      }

      const avgFacilityPurchase = facilityMetrics.reduce((a, b) => a + b, 0) / facilityMetrics.length;
      console.log(`Average facility purchase: ${avgFacilityPurchase.toFixed(2)}ms over ${facilityMetrics.length} purchases`);
      expect(avgFacilityPurchase).to.be.lessThan(3000);
    });
  });

  describe('Reward System Performance', () => {
    it('Should benchmark reward claiming with various time gaps', async () => {
      const testUser = Keypair.generate();
      await provider.connection.requestAirdrop(testUser.publicKey, 3 * anchor.web3.LAMPORTS_PER_SOL);

      const [userStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), testUser.publicKey.toBuffer()],
        program.programId
      );
      const [facilityPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('facility'), testUser.publicKey.toBuffer()],
        program.programId
      );
      const userTokenAccount = await getAssociatedTokenAddress(rewardMintPda, testUser.publicKey);

      // Setup user
      await program.methods
        .initUser(null)
        .accounts({
          userState: userStatePda,
          user: testUser.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      await program.methods
        .buyFacility()
        .accounts({
          userState: userStatePda,
          facility: facilityPda,
          user: testUser.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([testUser])
        .rpc();

      // Create token account
      const instruction = createAssociatedTokenAccountInstruction(
        testUser.publicKey,
        userTokenAccount,
        testUser.publicKey,
        rewardMintPda,
      );
      const transaction = new anchor.web3.Transaction().add(instruction);
      await provider.sendAndConfirm(transaction, [testUser]);

      // Test reward claiming with different time gaps
      const timeGaps = [1000, 2000, 5000]; // 1s, 2s, 5s
      
      for (const gap of timeGaps) {
        await new Promise(resolve => setTimeout(resolve, gap));
        
        const metric = await measurePerformance(
          `claim_reward_${gap}ms_gap`,
          async () => {
            await program.methods
              .claimReward()
              .accounts({
                userState: userStatePda,
                config: configPda,
                rewardMint: rewardMintPda,
                mintAuthority: mintAuthorityPda,
                userTokenAccount: userTokenAccount,
                user: testUser.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
              })
              .signers([testUser])
              .rpc();
          },
          3
        );

        console.log(`claim_reward (${gap}ms gap): avg ${metric.averageTime.toFixed(2)}ms`);
        expect(metric.averageTime).to.be.lessThan(5000);
      }
    });

    it('Should benchmark transfer with fee calculation', async () => {
      const sender = Keypair.generate();
      const receiver = Keypair.generate();
      
      await provider.connection.requestAirdrop(sender.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
      await provider.connection.requestAirdrop(receiver.publicKey, 1 * anchor.web3.LAMPORTS_PER_SOL);

      // Setup sender
      const [senderStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), sender.publicKey.toBuffer()],
        program.programId
      );
      const [senderFacilityPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('facility'), sender.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .initUser(null)
        .accounts({
          userState: senderStatePda,
          user: sender.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([sender])
        .rpc();

      await program.methods
        .buyFacility()
        .accounts({
          userState: senderStatePda,
          facility: senderFacilityPda,
          user: sender.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([sender])
        .rpc();

      // Create token accounts
      const senderTokenAccount = await getAssociatedTokenAddress(rewardMintPda, sender.publicKey);
      const receiverTokenAccount = await getAssociatedTokenAddress(rewardMintPda, receiver.publicKey);
      const treasuryTokenAccount = await getAssociatedTokenAddress(rewardMintPda, treasury.publicKey);

      for (const user of [sender, receiver, treasury]) {
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

      // Get some tokens first
      await new Promise(resolve => setTimeout(resolve, 2000));
      await program.methods
        .claimReward()
        .accounts({
          userState: senderStatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: senderTokenAccount,
          user: sender.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([sender])
        .rpc();

      // Benchmark transfers with different amounts
      const transferAmounts = [10, 100, 1000];
      
      for (const amount of transferAmounts) {
        const senderBalance = await getAccount(provider.connection, senderTokenAccount);
        if (Number(senderBalance.amount) < amount) {
          console.log(`Skipping transfer test for ${amount} tokens - insufficient balance`);
          continue;
        }

        const metric = await measurePerformance(
          `transfer_with_fee_${amount}`,
          async () => {
            await program.methods
              .transferWithFee(new anchor.BN(amount))
              .accounts({
                fromTokenAccount: senderTokenAccount,
                toTokenAccount: receiverTokenAccount,
                config: configPda,
                treasuryTokenAccount: treasuryTokenAccount,
                rewardMint: rewardMintPda,
                from: sender.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
              })
              .signers([sender])
              .rpc();
          },
          2
        );

        console.log(`transfer_with_fee (${amount} tokens): avg ${metric.averageTime.toFixed(2)}ms`);
        expect(metric.averageTime).to.be.lessThan(4000);
      }
    });
  });

  describe('Mystery Box Performance', () => {
    it('Should benchmark mystery box operations', async () => {
      const user = Keypair.generate();
      await provider.connection.requestAirdrop(user.publicKey, 3 * anchor.web3.LAMPORTS_PER_SOL);

      // Setup user with facility to earn tokens
      const [userStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), user.publicKey.toBuffer()],
        program.programId
      );
      const [facilityPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('facility'), user.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .initUser(null)
        .accounts({
          userState: userStatePda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      await program.methods
        .buyFacility()
        .accounts({
          userState: userStatePda,
          facility: facilityPda,
          user: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      const userTokenAccount = await getAssociatedTokenAddress(rewardMintPda, user.publicKey);
      const treasuryTokenAccount = await getAssociatedTokenAddress(rewardMintPda, treasury.publicKey);

      // Create token accounts
      for (const keypair of [user, treasury]) {
        const tokenAccount = await getAssociatedTokenAddress(rewardMintPda, keypair.publicKey);
        const instruction = createAssociatedTokenAccountInstruction(
          keypair.publicKey,
          tokenAccount,
          keypair.publicKey,
          rewardMintPda,
        );
        const transaction = new anchor.web3.Transaction().add(instruction);
        await provider.sendAndConfirm(transaction, [keypair]);
      }

      // Earn tokens
      await new Promise(resolve => setTimeout(resolve, 3000));
      await program.methods
        .claimReward()
        .accounts({
          userState: userStatePda,
          config: configPda,
          rewardMint: rewardMintPda,
          mintAuthority: mintAuthorityPda,
          userTokenAccount: userTokenAccount,
          user: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();

      const balance = await getAccount(provider.connection, userTokenAccount);
      if (Number(balance.amount) < 1000) {
        console.log('Insufficient balance for mystery box test, skipping');
        return;
      }

      // Benchmark mystery box purchase
      const boxId = new anchor.BN(1);
      const [mysteryBoxPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('mystery_box'), boxId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );

      const purchaseMetric = await measurePerformance(
        'purchase_mystery_box',
        async () => {
          await program.methods
            .purchaseMysteryBox(boxId)
            .accounts({
              config: configPda,
              mysteryBox: mysteryBoxPda,
              rewardMint: rewardMintPda,
              purchaserTokenAccount: userTokenAccount,
              treasuryTokenAccount: treasuryTokenAccount,
              purchaser: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
        },
        1
      );

      console.log(`purchase_mystery_box: avg ${purchaseMetric.averageTime.toFixed(2)}ms`);

      // Benchmark mystery box opening
      const seedId = new anchor.BN(1);
      const [seedPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('seed'), seedId.toArrayLike(Buffer, 'le', 8)],
        program.programId
      );

      const openMetric = await measurePerformance(
        'open_mystery_box',
        async () => {
          await program.methods
            .openMysteryBox(boxId, seedId)
            .accounts({
              config: configPda,
              mysteryBox: mysteryBoxPda,
              seed: seedPda,
              opener: user.publicKey,
              systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();
        },
        1
      );

      console.log(`open_mystery_box: avg ${openMetric.averageTime.toFixed(2)}ms`);
      
      expect(purchaseMetric.averageTime).to.be.lessThan(5000);
      expect(openMetric.averageTime).to.be.lessThan(3000);
    });
  });

  describe('Stress Testing', () => {
    it('Should handle concurrent users claiming rewards', async () => {
      const concurrentUsers = 5;
      const users: Keypair[] = [];
      
      // Setup multiple users
      for (let i = 0; i < concurrentUsers; i++) {
        const user = Keypair.generate();
        users.push(user);
        await provider.connection.requestAirdrop(user.publicKey, 3 * anchor.web3.LAMPORTS_PER_SOL);
      }

      // Initialize all users and their facilities
      for (const user of users) {
        const [userStatePda] = PublicKey.findProgramAddressSync(
          [Buffer.from('user'), user.publicKey.toBuffer()],
          program.programId
        );
        const [facilityPda] = PublicKey.findProgramAddressSync(
          [Buffer.from('facility'), user.publicKey.toBuffer()],
          program.programId
        );

        await program.methods
          .initUser(null)
          .accounts({
            userState: userStatePda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        await program.methods
          .buyFacility()
          .accounts({
            userState: userStatePda,
            facility: facilityPda,
            user: user.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        // Create token account
        const userTokenAccount = await getAssociatedTokenAddress(rewardMintPda, user.publicKey);
        const instruction = createAssociatedTokenAccountInstruction(
          user.publicKey,
          userTokenAccount,
          user.publicKey,
          rewardMintPda,
        );
        const transaction = new anchor.web3.Transaction().add(instruction);
        await provider.sendAndConfirm(transaction, [user]);
      }

      // Wait for rewards to accumulate
      await new Promise(resolve => setTimeout(resolve, 3000));

      // Concurrent reward claiming
      const startTime = process.hrtime.bigint();
      
      const claimPromises = users.map(async (user, index) => {
        const [userStatePda] = PublicKey.findProgramAddressSync(
          [Buffer.from('user'), user.publicKey.toBuffer()],
          program.programId
        );
        const userTokenAccount = await getAssociatedTokenAddress(rewardMintPda, user.publicKey);

        try {
          await program.methods
            .claimReward()
            .accounts({
              userState: userStatePda,
              config: configPda,
              rewardMint: rewardMintPda,
              mintAuthority: mintAuthorityPda,
              userTokenAccount: userTokenAccount,
              user: user.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .signers([user])
            .rpc();
          return { success: true, user: index };
        } catch (error) {
          return { success: false, user: index, error: error.message };
        }
      });

      const results = await Promise.all(claimPromises);
      const endTime = process.hrtime.bigint();
      const totalTime = Number(endTime - startTime) / 1000000;

      const successes = results.filter(r => r.success).length;
      const failures = results.filter(r => !r.success).length;

      console.log(`Concurrent claims: ${successes} succeeded, ${failures} failed in ${totalTime.toFixed(2)}ms`);
      
      expect(successes).to.be.at.least(concurrentUsers * 0.8); // At least 80% should succeed
      expect(totalTime).to.be.lessThan(15000); // Should complete within 15 seconds
    });
  });

  after(() => {
    // Print performance summary
    console.log('\n=== PERFORMANCE SUMMARY ===');
    console.log('Instruction                | Avg Time (ms) | Min (ms) | Max (ms) | Samples');
    console.log('---------------------------|---------------|----------|----------|--------');
    
    metrics.forEach(metric => {
      const instruction = metric.instruction.padEnd(26);
      const avg = metric.averageTime.toFixed(2).padStart(12);
      const min = metric.minTime.toFixed(2).padStart(8);
      const max = metric.maxTime.toFixed(2).padStart(8);
      const samples = metric.samples.toString().padStart(7);
      
      console.log(`${instruction} | ${avg} | ${min} | ${max} | ${samples}`);
    });
    
    console.log('============================\n');
  });
});