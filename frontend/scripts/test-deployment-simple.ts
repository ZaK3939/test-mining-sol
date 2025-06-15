#!/usr/bin/env bun

/**
 * Simple test script for deployed Facility Game program
 * This script provides basic instructions for testing with a deployed program
 * 
 * Prerequisites:
 * 1. Program must be deployed
 * 2. You need access to admin wallet
 * 
 * To run actual tests:
 * 1. Use Solana CLI to create test wallets
 * 2. Use the deployment test script with proper environment variables
 */

import { PublicKey } from '@solana/web3.js';
import { logger } from '../src/logger';

const PROGRAM_ID = 'GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc';

async function showTestInstructions() {
    logger.info('📚 Facility Game Deployment Test Instructions');
    logger.info('============================================');
    
    logger.info('\n🎯 Program Information:');
    logger.info(`  Program ID: ${PROGRAM_ID}`);
    logger.info(`  Networks: Devnet, Testnet, or Mainnet`);
    
    logger.info('\n📋 Test Sequence:');
    logger.info('  1. Initialize Config (admin only, one-time)');
    logger.info('  2. Create Reward Mint (admin only, one-time)');
    logger.info('  3. Initialize Global Stats (admin only, one-time)');
    logger.info('  4. Create Invite Code (admin/operator)');
    logger.info('  5. Use Invite Code to Initialize User');
    logger.info('  6. Buy Farm Space (0.5 SOL)');
    logger.info('  7. Wait and Claim Rewards');
    logger.info('  8. Check Token Balance');
    
    logger.info('\n🔧 Using Solana CLI:');
    logger.info('  # Create a test wallet');
    logger.info('  solana-keygen new --outfile test-wallet.json');
    logger.info('  ');
    logger.info('  # Airdrop SOL (devnet only)');
    logger.info('  solana airdrop 2 <wallet-address> --url devnet');
    logger.info('  ');
    logger.info('  # Check program deployment');
    logger.info(`  solana program show ${PROGRAM_ID}`);
    
    logger.info('\n💻 Using the Full Test Script:');
    logger.info('  # Export admin private key (base58 encoded)');
    logger.info('  export ADMIN_PRIVATE_KEY="your-private-key"');
    logger.info('  ');
    logger.info('  # Run the full test');
    logger.info('  bun run scripts/test-deployment.ts');
    
    logger.info('\n🌐 RPC Endpoints:');
    logger.info('  Devnet: https://api.devnet.solana.com');
    logger.info('  Testnet: https://api.testnet.solana.com');
    logger.info('  Mainnet: https://api.mainnet-beta.solana.com');
    
    logger.info('\n📊 Expected Results:');
    logger.info('  ✅ Config initialized with base rate and halving interval');
    logger.info('  ✅ WEED token mint created with Token-2022');
    logger.info('  ✅ Global stats tracking total grow power');
    logger.info('  ✅ Invite codes working with hash-based system');
    logger.info('  ✅ Users can buy farm spaces and claim rewards');
    logger.info('  ✅ Referral system tracks relationships');
    
    logger.info('\n⚠️ Common Issues:');
    logger.info('  - "Account already in use": Admin setup already done');
    logger.info('  - "Insufficient balance": Need more SOL for transactions');
    logger.info('  - "Invalid invite code": Code format must be 8 alphanumeric chars');
    logger.info('  - "User not initialized": Must use invite code first');
    
    logger.info('\n📝 Test Checklist:');
    const checklist = [
        'Program deployed to target network',
        'Admin wallet has sufficient SOL',
        'Config initialized successfully',
        'Reward mint created',
        'Global stats initialized',
        'Invite code created and used',
        'Farm space purchased',
        'Rewards claimed after waiting',
        'Token balance verified',
        'Referral rewards distributed'
    ];
    
    checklist.forEach((item, index) => {
        logger.info(`  ${index + 1}. [ ] ${item}`);
    });
    
    logger.info('\n🔗 Useful Links:');
    logger.info('  - Solana Explorer: https://explorer.solana.com');
    logger.info('  - Solana Docs: https://docs.solana.com');
    logger.info('  - Anchor Docs: https://www.anchor-lang.com');
    
    logger.info('\n✨ Ready to test! Follow the instructions above.');
}

// Show instructions
showTestInstructions().then(() => {
    logger.success('📋 Instructions displayed successfully');
    logger.info('Run the full test script when ready to test with a deployed program.');
}).catch((error) => {
    logger.error('Error:', error);
});