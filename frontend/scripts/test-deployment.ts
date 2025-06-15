#!/usr/bin/env bun

/**
 * Test script for deployed Facility Game program
 * This script tests all major functionality with a real deployed program
 * 
 * Prerequisites:
 * 1. Program must be deployed to devnet/mainnet
 * 2. Admin wallet must have SOL for transactions
 * 3. Environment variables must be set:
 *    - ADMIN_PRIVATE_KEY: Base58 encoded private key of admin wallet
 *    - RPC_URL: RPC endpoint URL (optional, defaults to devnet)
 * 
 * Usage:
 * ADMIN_PRIVATE_KEY="your-private-key" bun run scripts/test-deployment.ts
 */

import { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { AnchorClient } from '../src/anchor-client';
import { logger } from '../src/logger';
import bs58 from 'bs58';

// Test configuration
const RPC_URL = process.env.RPC_URL || 'https://api.devnet.solana.com';
const ADMIN_PRIVATE_KEY = process.env.ADMIN_PRIVATE_KEY;

// Test parameters
const TEST_BASE_RATE = 100; // 100 WEED per second (total)
const TEST_HALVING_INTERVAL = 604800; // 7 days
const TEST_INVITE_CODE = 'TEST1234';

async function delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function testDeployedProgram() {
    logger.info('🚀 Starting Facility Game deployment test...');
    
    // Validate environment
    if (!ADMIN_PRIVATE_KEY) {
        logger.error('❌ ADMIN_PRIVATE_KEY environment variable not set');
        logger.error('Usage: ADMIN_PRIVATE_KEY="your-private-key" bun run scripts/test-deployment.ts');
        process.exit(1);
    }
    
    try {
        // Setup connection and wallet
        logger.info(`📡 Connecting to ${RPC_URL}...`);
        const connection = new Connection(RPC_URL, 'confirmed');
        
        // Decode admin keypair
        const adminKeypair = Keypair.fromSecretKey(bs58.decode(ADMIN_PRIVATE_KEY));
        const adminWallet = new Wallet(adminKeypair);
        logger.success(`✅ Admin wallet: ${adminKeypair.publicKey.toString()}`);
        
        // Check admin balance
        const balance = await connection.getBalance(adminKeypair.publicKey);
        logger.info(`💰 Admin balance: ${balance / LAMPORTS_PER_SOL} SOL`);
        
        if (balance < 0.1 * LAMPORTS_PER_SOL) {
            logger.error('❌ Insufficient balance. Need at least 0.1 SOL for tests');
            process.exit(1);
        }
        
        // Create anchor provider and client
        const provider = new AnchorProvider(connection, adminWallet, {
            preflightCommitment: 'confirmed',
            commitment: 'confirmed',
        });
        
        const client = new AnchorClient(provider);
        logger.success(`✅ Connected to program: ${client.getProgramId().toString()}`);
        
        // Test 1: Initialize Config
        logger.info('📋 Test 1: Initialize Config...');
        try {
            const configTx = await client.initializeConfig(
                TEST_BASE_RATE,
                TEST_HALVING_INTERVAL,
                adminKeypair.publicKey
            );
            logger.success(`✅ Config initialized: ${configTx}`);
        } catch (error: any) {
            if (error.message?.includes('already in use')) {
                logger.warn('⚠️ Config already initialized (expected for existing deployment)');
            } else {
                throw error;
            }
        }
        
        // Test 2: Fetch Config
        logger.info('📋 Test 2: Fetch Config...');
        const config = await client.fetchConfig();
        if (config) {
            logger.success('✅ Config fetched successfully:');
            logger.info(`  - Admin: ${config.admin.toString()}`);
            logger.info(`  - Treasury: ${config.treasury.toString()}`);
            logger.info(`  - Base Rate: ${config.baseRate.toString()}`);
            logger.info(`  - Halving Interval: ${config.halvingInterval.toString()}`);
        } else {
            logger.error('❌ Failed to fetch config');
            process.exit(1);
        }
        
        // Test 3: Create Reward Mint
        logger.info('🪙 Test 3: Create Reward Mint...');
        try {
            const mintTx = await client.createRewardMint();
            logger.success(`✅ Reward mint created: ${mintTx}`);
        } catch (error: any) {
            if (error.message?.includes('already in use')) {
                logger.warn('⚠️ Reward mint already created (expected for existing deployment)');
            } else {
                throw error;
            }
        }
        
        // Test 4: Initialize Global Stats
        logger.info('📊 Test 4: Initialize Global Stats...');
        try {
            const statsTx = await client.initializeGlobalStats();
            logger.success(`✅ Global stats initialized: ${statsTx}`);
        } catch (error: any) {
            if (error.message?.includes('already in use')) {
                logger.warn('⚠️ Global stats already initialized (expected for existing deployment)');
            } else {
                throw error;
            }
        }
        
        // Test 5: Create test user wallet
        logger.info('👤 Test 5: Create test user...');
        const testUser = Keypair.generate();
        logger.info(`  - Test user: ${testUser.publicKey.toString()}`);
        
        // Airdrop SOL to test user
        logger.info('💸 Airdropping SOL to test user...');
        const airdropSig = await connection.requestAirdrop(
            testUser.publicKey,
            1 * LAMPORTS_PER_SOL
        );
        await connection.confirmTransaction(airdropSig);
        logger.success('✅ Airdrop confirmed');
        
        // Create client for test user
        const testUserWallet = new Wallet(testUser);
        const testUserProvider = new AnchorProvider(connection, testUserWallet, {
            preflightCommitment: 'confirmed',
            commitment: 'confirmed',
        });
        const testUserClient = new AnchorClient(testUserProvider);
        
        // Test 6: Create invite code
        logger.info('🎟️ Test 6: Create invite code...');
        const inviteCodeTx = await client.createInviteCode(TEST_INVITE_CODE);
        logger.success(`✅ Invite code created: ${inviteCodeTx}`);
        
        // Test 7: Use invite code to initialize user
        logger.info('👤 Test 7: Initialize user with invite code...');
        const initUserTx = await testUserClient.useInviteCode(
            TEST_INVITE_CODE,
            adminKeypair.publicKey
        );
        logger.success(`✅ User initialized with invite code: ${initUserTx}`);
        
        // Test 8: Fetch user state
        logger.info('👤 Test 8: Fetch user state...');
        const userState = await testUserClient.fetchUserState(testUser.publicKey);
        if (userState) {
            logger.success('✅ User state fetched:');
            logger.info(`  - Owner: ${userState.owner.toString()}`);
            logger.info(`  - Has Farm Space: ${userState.hasFarmSpace}`);
            logger.info(`  - Referrer: ${userState.referrer?.toString() || 'None'}`);
        } else {
            logger.error('❌ Failed to fetch user state');
            process.exit(1);
        }
        
        // Test 9: Buy farm space
        logger.info('🌾 Test 9: Buy farm space...');
        const buyFarmTx = await testUserClient.buyFarmSpace();
        logger.success(`✅ Farm space purchased: ${buyFarmTx}`);
        
        // Test 10: Fetch farm space
        logger.info('🌾 Test 10: Fetch farm space...');
        const farmSpace = await testUserClient.fetchFarmSpace(testUser.publicKey);
        if (farmSpace) {
            logger.success('✅ Farm space fetched:');
            logger.info(`  - Level: ${farmSpace.level}`);
            logger.info(`  - Capacity: ${farmSpace.capacity}`);
            logger.info(`  - Total Grow Power: ${farmSpace.totalGrowPower.toString()}`);
        } else {
            logger.error('❌ Failed to fetch farm space');
            process.exit(1);
        }
        
        // Test 11: Wait and claim rewards
        logger.info('⏳ Test 11: Waiting 5 seconds before claiming rewards...');
        await delay(5000);
        
        logger.info('💰 Claiming rewards...');
        const claimTx = await testUserClient.claimRewards();
        logger.success(`✅ Rewards claimed: ${claimTx}`);
        
        // Test 12: Check token balance
        logger.info('💰 Test 12: Check token balance...');
        const tokenBalance = await testUserClient.getTokenBalance(testUser.publicKey);
        logger.success(`✅ Token balance: ${tokenBalance} WEED`);
        
        // Test 13: Fetch global stats
        logger.info('📊 Test 13: Fetch global stats...');
        const globalStats = await client.fetchGlobalStats();
        if (globalStats) {
            logger.success('✅ Global stats fetched:');
            logger.info(`  - Total Grow Power: ${globalStats.totalGrowPower}`);
            logger.info(`  - Total Farm Spaces: ${globalStats.totalFarmSpaces}`);
            logger.info(`  - Total Supply: ${globalStats.totalSupply}`);
        }
        
        // Summary
        logger.success('🎉 All tests completed successfully!');
        logger.info('📋 Test Summary:');
        logger.info('  ✅ Config initialization');
        logger.info('  ✅ Reward mint creation');
        logger.info('  ✅ Global stats initialization');
        logger.info('  ✅ Invite code creation');
        logger.info('  ✅ User initialization with invite code');
        logger.info('  ✅ Farm space purchase');
        logger.info('  ✅ Reward claiming');
        logger.info('  ✅ Token balance check');
        logger.info('  ✅ Global stats fetch');
        
        logger.info(`\n🔗 Program ID: ${client.getProgramId().toString()}`);
        logger.info(`🌐 Network: ${RPC_URL}`);
        
    } catch (error) {
        logger.error('❌ Test failed:', error);
        process.exit(1);
    }
}

// Run the test
testDeployedProgram().then(() => {
    logger.success('✅ Deployment test completed successfully');
    process.exit(0);
}).catch((error) => {
    logger.error('💥 Deployment test failed:', error);
    process.exit(1);
});