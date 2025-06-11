// Simple script to initialize system on devnet
import * as anchor from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';

const PROGRAM_ID = 'FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B';
const RPC_URL = 'https://api.devnet.solana.com';

async function main() {
  // Load keypair
  const keypairPath = path.join(process.env.HOME!, '.config/solana/id.json');
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
  const payer = Keypair.fromSecretKey(new Uint8Array(keypairData));

  // Setup provider
  const connection = new Connection(RPC_URL, 'confirmed');
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  // Load program
  const idl = JSON.parse(fs.readFileSync('./target/idl/farm_game.json', 'utf8'));
  const program = new anchor.Program(idl, PROGRAM_ID);

  console.log('🚀 Initializing system on devnet...');
  console.log(`Program: ${program.programId.toString()}`);
  console.log(`Admin: ${payer.publicKey.toString()}`);

  // Calculate PDAs
  const [configPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    program.programId
  );

  console.log(`Config PDA: ${configPDA.toString()}`);

  try {
    // Check if already initialized
    const existingConfig = await connection.getAccountInfo(configPDA);
    if (existingConfig) {
      console.log('✅ System already initialized!');
      return;
    }

    // Initialize system
    console.log('📦 Initializing config...');
    const tx = await program.rpc.initializeConfig({
      accounts: {
        config: configPDA,
        admin: payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });

    console.log(`✅ System initialized! Tx: ${tx}`);

    // Create reward mint
    console.log('🪙 Creating reward mint...');
    const [rewardMintPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('reward_mint')],
      program.programId
    );

    const [mintAuthorityPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('mint_authority')],
      program.programId
    );

    const mintTx = await program.rpc.createRewardMint({
      accounts: {
        config: configPDA,
        rewardMint: rewardMintPDA,
        mintAuthority: mintAuthorityPDA,
        admin: payer.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });

    console.log(`✅ Reward mint created! Tx: ${mintTx}`);

  } catch (error) {
    console.error('❌ Error:', error);
    if (error instanceof Error && error.message.includes('already in use')) {
      console.log('ℹ️ Account may already exist');
    }
  }
}

main().catch(console.error);