#!/usr/bin/env bun
// シンプルな初期化スクリプト - Solana Web3.jsのみを使用

import { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createInitializeMintInstruction, getMint, MINT_SIZE, getMinimumBalanceForRentExemptMint } from '@solana/spl-token';
import { logger } from '../src/logger';
import fs from 'fs';
import path from 'path';
import os from 'os';
import * as borsh from 'borsh';

// 設定
const PROGRAM_ID = new PublicKey('EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89');
const RPC_URL = 'http://localhost:8899';

// Metaplex Token Metadata Program ID
const TOKEN_METADATA_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

// 管理者のキーペアを読み込む
function loadAdminKeypair(): Keypair {
  try {
    const keypairPath = path.join(os.homedir(), '.config', 'solana', 'id.json');
    if (fs.existsSync(keypairPath)) {
      const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
      logger.info(`🔑 キーペアを読み込みました: ${keypairPath}`);
      return Keypair.fromSecretKey(new Uint8Array(keypairData));
    }
    
    throw new Error('キーペアが見つかりません');
  } catch (error) {
    logger.error('キーペアの読み込みに失敗しました:', error);
    throw error;
  }
}

// PDAの計算
function calculatePDAs(programId: PublicKey) {
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    programId
  );

  const [rewardMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('reward_mint')],
    programId
  );

  const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('mint_authority')],
    programId
  );

  return {
    config: configPda,
    rewardMint: rewardMintPda,
    mintAuthority: mintAuthorityPda,
  };
}

// 初期化設定の命令を作成
function createInitializeConfigInstruction(
  programId: PublicKey,
  config: PublicKey,
  admin: PublicKey,
  baseRate: bigint,
  halvingInterval: bigint
): TransactionInstruction {
  // Discriminator for initializeConfig: [208, 127, 21, 1, 194, 190, 196, 70]
  const discriminator = Buffer.from([208, 127, 21, 1, 194, 190, 196, 70]);
  
  // Serialize arguments
  const data = Buffer.concat([
    discriminator,
    Buffer.from(new Uint8Array(new BigUint64Array([baseRate]).buffer)),
    Buffer.from(new Uint8Array(new BigInt64Array([halvingInterval]).buffer))
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

// リワードミント作成の命令を作成
function createRewardMintInstruction(
  programId: PublicKey,
  rewardMint: PublicKey,
  mintAuthority: PublicKey,
  metadataAccount: PublicKey,
  admin: PublicKey,
  rent: PublicKey
): TransactionInstruction {
  // Discriminator for createRewardMint: [149, 144, 95, 196, 171, 77, 31, 66]
  const discriminator = Buffer.from([149, 144, 95, 196, 171, 77, 31, 66]);

  return new TransactionInstruction({
    keys: [
      { pubkey: rewardMint, isSigner: false, isWritable: true },
      { pubkey: mintAuthority, isSigner: false, isWritable: false },
      { pubkey: metadataAccount, isSigner: false, isWritable: true },
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: rent, isSigner: false, isWritable: false },
      { pubkey: TOKEN_METADATA_ID, isSigner: false, isWritable: false },
    ],
    programId,
    data: discriminator,
  });
}

async function initialize() {
  try {
    logger.info('🚀 シンプル初期化開始...');
    
    // 管理者キーペアを読み込む
    const adminKeypair = loadAdminKeypair();
    logger.info(`👤 管理者アドレス: ${adminKeypair.publicKey.toBase58()}`);
    
    // 接続
    const connection = new Connection(RPC_URL, 'confirmed');
    
    // PDAの計算
    const pdas = calculatePDAs(PROGRAM_ID);
    
    logger.info('📍 PDAアドレス:');
    logger.info(`  - Config: ${pdas.config.toBase58()}`);
    logger.info(`  - Reward Mint: ${pdas.rewardMint.toBase58()}`);
    logger.info(`  - Mint Authority: ${pdas.mintAuthority.toBase58()}`);
    
    // 1. Config が既に初期化されているか確認
    let configExists = false;
    try {
      const configAccount = await connection.getAccountInfo(pdas.config);
      if (configAccount && configAccount.data.length > 0) {
        configExists = true;
        logger.warn('⚠️  Config は既に初期化されています');
      }
    } catch (e) {
      logger.info('✅ Config は未初期化です。初期化を実行します。');
    }
    
    // 2. Config の初期化（必要な場合）
    if (!configExists) {
      logger.info('📝 Config を初期化中...');
      
      const baseRate = BigInt(10);
      const halvingInterval = BigInt(86400 * 365); // 1年（秒単位）
      
      const initConfigIx = createInitializeConfigInstruction(
        PROGRAM_ID,
        pdas.config,
        adminKeypair.publicKey,
        baseRate,
        halvingInterval
      );
      
      const tx1 = new Transaction().add(initConfigIx);
      const sig1 = await connection.sendTransaction(tx1, [adminKeypair]);
      await connection.confirmTransaction(sig1);
      
      logger.success(`✅ Config 初期化成功! TX: ${sig1}`);
    }
    
    // 3. Reward Mint が既に存在するか確認
    let mintExists = false;
    try {
      const mintAccount = await connection.getAccountInfo(pdas.rewardMint);
      if (mintAccount && mintAccount.data.length > 0) {
        mintExists = true;
        logger.warn('⚠️  Reward Mint は既に作成されています');
      }
    } catch (e) {
      logger.info('✅ Reward Mint は未作成です。作成を実行します。');
    }
    
    // 4. Reward Mint の作成（必要な場合）
    if (!mintExists) {
      logger.info('🪙 Reward Mint を作成中...');
      
      // Derive metadata account PDA
      const [metadataAccount] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          TOKEN_METADATA_ID.toBuffer(),
          pdas.rewardMint.toBuffer()
        ],
        TOKEN_METADATA_ID
      );
      
      const rentPubkey = new PublicKey('SysvarRent111111111111111111111111111111111');
      
      const createMintIx = createRewardMintInstruction(
        PROGRAM_ID,
        pdas.rewardMint,
        pdas.mintAuthority,
        metadataAccount,
        adminKeypair.publicKey,
        rentPubkey
      );
      
      const tx2 = new Transaction().add(createMintIx);
      const sig2 = await connection.sendTransaction(tx2, [adminKeypair]);
      await connection.confirmTransaction(sig2);
      
      logger.success(`✅ Reward Mint ($WEED) 作成成功! TX: ${sig2}`);
      logger.info(`  - Mint Address: ${pdas.rewardMint.toBase58()}`);
      logger.info(`  - Token Name: Weed Token`);
      logger.info(`  - Token Symbol: WEED`);
    }
    
    // 5. 残高確認
    const balance = await connection.getBalance(adminKeypair.publicKey);
    logger.info(`💰 管理者の残高: ${balance / 1e9} SOL`);
    
    logger.success('🎉 初期化が完了しました！');
    logger.info('');
    logger.info('📌 次のステップ:');
    logger.info('  1. フロントエンドを起動: bun run dev');
    logger.info('  2. ウォレットを接続');
    logger.info('  3. ユーザーを初期化');
    logger.info('  4. 施設を購入');
    logger.info('  5. 報酬を請求');
    
  } catch (error) {
    logger.error(`❌ エラーが発生しました: ${error instanceof Error ? error.message : String(error)}`);
    if (error instanceof Error && error.stack) {
      logger.error(error.stack);
    }
    process.exit(1);
  }
}

// メイン実行
initialize().catch(console.error);