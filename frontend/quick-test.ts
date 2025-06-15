#!/usr/bin/env bun
import { Connection, Keypair, PublicKey } from '@solana/web3.js';

async function quickTest() {
  console.log('🔍 Quick Solana Program Test');
  
  // Test connection to local validator
  try {
    const connection = new Connection('http://localhost:8899', 'confirmed');
    const version = await connection.getVersion();
    console.log('✅ Local validator connected:', version);
    
    // Generate test keypair
    const testKeypair = Keypair.generate();
    console.log('🔑 Test keypair:', testKeypair.publicKey.toString());
    
    // Test our program ID
    const programId = new PublicKey('GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc');
    const programAccount = await connection.getAccountInfo(programId);
    
    if (programAccount) {
      console.log('✅ Program deployed and accessible');
      console.log('📊 Program account owner:', programAccount.owner.toString());
      console.log('💾 Program data length:', programAccount.data.length);
    } else {
      console.log('❌ Program not found - need to deploy first');
    }
    
  } catch (error) {
    console.error('❌ Connection failed:', error);
  }
}

quickTest();