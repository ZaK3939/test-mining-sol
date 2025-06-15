#!/usr/bin/env bun
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { SimpleClient } from './src/simple-client';
import fs from 'fs';

async function testProgram() {
  console.log('🧪 Testing Deployed Program');
  
  // Setup connection to local validator
  const connection = new Connection('http://localhost:8899', 'confirmed');
  
  // Load wallet
  const keypairPath = `${process.env.HOME}/.config/solana/id.json`;
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
  const payer = Keypair.fromSecretKey(new Uint8Array(keypairData));
  
  console.log('👤 Wallet:', payer.publicKey.toString());
  
  // Check balance
  const balance = await connection.getBalance(payer.publicKey);
  console.log('💰 Balance:', balance / 1e9, 'SOL');
  
  if (balance === 0) {
    console.log('❌ No SOL balance - need airdrop');
    return;
  }
  
  // Setup provider and client
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });
  const client = new SimpleClient(provider);
  
  console.log('✅ Client initialized');
  console.log('📱 Program ID:', client.program.programId.toString());
  
  // Check if program is deployed
  const programAccount = await connection.getAccountInfo(client.program.programId);
  if (!programAccount) {
    console.log('❌ Program not deployed');
    return;
  }
  
  console.log('✅ Program found on chain');
  console.log('📊 Program data length:', programAccount.data.length);
  
  try {
    // Try to fetch config to see if it's initialized
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      client.program.programId
    );
    
    console.log('🔍 Config PDA:', configPDA.toString());
    
    const configAccount = await connection.getAccountInfo(configPDA);
    if (configAccount) {
      console.log('✅ Config account exists');
      console.log('📄 Config data length:', configAccount.data.length);
    } else {
      console.log('⚠️ Config not initialized yet');
    }
    
  } catch (error) {
    console.error('❌ Error checking config:', error);
  }
}

testProgram()
  .then(() => console.log('🎉 Test completed'))
  .catch(console.error);