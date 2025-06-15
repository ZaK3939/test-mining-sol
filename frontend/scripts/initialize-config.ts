// Script to manually initialize the config account
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { AnchorProvider, Wallet, Program } from '@coral-xyz/anchor';
import idl from '../src/idl/facility_game.json';
import fs from 'fs';
import path from 'path';

const PROGRAM_ID = 'GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc';
const RPC_URL = 'http://localhost:8899';

async function initializeConfig() {
  console.log('🚀 Starting config initialization...');

  // Load keypair from file system (Solana CLI default)
  const keypairPath = path.join(process.env.HOME!, '.config/solana/id.json');
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
  const payer = Keypair.fromSecretKey(new Uint8Array(keypairData));

  console.log(`👤 Payer: ${payer.publicKey.toString()}`);

  // Create connection and provider
  const connection = new Connection(RPC_URL, 'confirmed');
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' });

  // Load program
  const program = new Program(idl as any, provider);

  console.log(`📡 Program ID: ${program.programId.toString()}`);

  try {
    // Check if config already exists
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      program.programId
    );

    console.log(`🔍 Config PDA: ${configPDA.toString()}`);

    const existingConfig = await connection.getAccountInfo(configPDA);
    if (existingConfig) {
      console.log('✅ Config account already exists!');
      return;
    }

    console.log('📦 Creating config account...');

    // Initialize config
    const tx = await program.methods
      .initializeConfig()
      .accounts({
        config: configPDA,
        admin: payer.publicKey,
        system_program: new PublicKey('11111111111111111111111111111111'),
      })
      .rpc();

    console.log(`✅ Config initialized! Transaction: ${tx}`);

    // Wait for confirmation
    await connection.confirmTransaction(tx, 'confirmed');
    console.log('✅ Transaction confirmed!');

  } catch (error) {
    console.error('❌ Error initializing config:', error);
    
    if (error instanceof Error && error.message.includes('already in use')) {
      console.log('ℹ️ Config account may already exist from previous initialization');
    }
  }
}

// Run the initialization
initializeConfig()
  .then(() => {
    console.log('🎉 Config initialization complete!');
    process.exit(0);
  })
  .catch((error) => {
    console.error('💥 Initialization failed:', error);
    process.exit(1);
  });