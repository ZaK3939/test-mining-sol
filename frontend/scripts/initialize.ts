#!/usr/bin/env bun
// 初期化スクリプト - プログラムの設定とリワードミントを初期化

import * as anchor from '@coral-xyz/anchor';
import { Program, BN } from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { logger } from '../src/logger';
import fs from 'fs';
import path from 'path';
import os from 'os';

// Removed mpl-token-metadata dependency

// 設定
const PROGRAM_ID = new PublicKey('EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89');
const RPC_URL = 'http://localhost:8899';

// IDLを読み込む
function loadIDL() {
  const idlPath = path.join(process.cwd(), 'src/idl/farm_game.json');
  const idlString = fs.readFileSync(idlPath, 'utf8');
  return JSON.parse(idlString);
}

// 管理者のキーペアを読み込む
function loadAdminKeypair(): Keypair {
  try {
    // まず環境変数からチェック
    const privateKeyString = process.env.ADMIN_PRIVATE_KEY;
    if (privateKeyString) {
      const privateKey = JSON.parse(privateKeyString);
      return Keypair.fromSecretKey(new Uint8Array(privateKey));
    }

    // 次にローカルのSolanaキーペアファイルをチェック
    const keypairPath = path.join(os.homedir(), '.config', 'solana', 'id.json');
    if (fs.existsSync(keypairPath)) {
      const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
      logger.info(`🔑 キーペアを読み込みました: ${keypairPath}`);
      return Keypair.fromSecretKey(new Uint8Array(keypairData));
    }

    // どちらも見つからない場合は新しいキーペアを生成
    logger.warn('⚠️  既存のキーペアが見つかりません。新しいキーペアを生成します。');
    const newKeypair = Keypair.generate();
    logger.info(`🔑 新しいキーペア生成: ${newKeypair.publicKey.toBase58()}`);
    return newKeypair;
  } catch (error) {
    logger.error('キーペアの読み込みに失敗しました:', error);
    throw error;
  }
}

// PDAの計算
function calculatePDAs(programId: PublicKey) {
  const [configPda] = PublicKey.findProgramAddressSync([Buffer.from('config')], programId);

  const [rewardMintPda] = PublicKey.findProgramAddressSync([Buffer.from('reward_mint')], programId);

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

async function initialize() {
  try {
    logger.info('🚀 初期化開始...');

    // 管理者キーペアを読み込む
    const adminKeypair = loadAdminKeypair();
    logger.info(`👤 管理者アドレス: ${adminKeypair.publicKey.toBase58()}`);

    // 接続とプロバイダーの設定
    const connection = new Connection(RPC_URL, 'confirmed');
    const wallet = new anchor.Wallet(adminKeypair);
    const provider = new anchor.AnchorProvider(connection, wallet, {
      preflightCommitment: 'confirmed',
    });

    // IDLを読み込んでプログラムを初期化
    const idl = loadIDL();
    const program = new Program(idl, PROGRAM_ID, provider);

    // PDAの計算
    const pdas = calculatePDAs(PROGRAM_ID);

    logger.info('📍 PDAアドレス:');
    logger.info(`  - Config: ${pdas.config.toBase58()}`);
    logger.info(`  - Reward Mint: ${pdas.rewardMint.toBase58()}`);
    logger.info(`  - Mint Authority: ${pdas.mintAuthority.toBase58()}`);

    // 1. Config が既に初期化されているか確認
    let configExists = false;
    try {
      const configAccount = await program.account.config.fetch(pdas.config);
      if (configAccount) {
        configExists = true;
        logger.warn('⚠️  Config は既に初期化されています');
        logger.info(`  - Base Rate: ${configAccount.baseRate.toString()}`);
        logger.info(`  - Admin: ${configAccount.admin.toString()}`);
      }
    } catch (e) {
      logger.info('✅ Config は未初期化です。初期化を実行します。');
    }

    // 2. Config の初期化（必要な場合）
    if (!configExists) {
      logger.info('📝 Config を初期化中...');

      const baseRate = new BN(10);
      const halvingInterval = new BN(200); // 200秒

      const tx1 = await program.methods
        .initializeConfig(baseRate, halvingInterval, wallet.publicKey)
        .accounts({
          config: pdas.config,
          admin: wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`✅ Config 初期化成功! TX: ${tx1}`);

      // 確認
      const configAccount = await program.account.config.fetch(pdas.config);
      logger.info(`  - Base Rate: ${configAccount.baseRate.toString()}`);
      logger.info(`  - Halving Interval: ${configAccount.halvingInterval.toString()}秒`);
      logger.info(
        `  - Next Halving Time: ${new Date(
          configAccount.nextHalvingTime.toNumber() * 1000
        ).toLocaleString()}`
      );
    }

    // 3. Reward Mint が既に存在するか確認
    let mintExists = false;
    try {
      const mintAccount = await connection.getAccountInfo(pdas.rewardMint);
      if (mintAccount) {
        mintExists = true;
        logger.warn('⚠️  Reward Mint は既に作成されています');
      }
    } catch (e) {
      logger.info('✅ Reward Mint は未作成です。作成を実行します。');
    }

    // 4. Reward Mint の作成（必要な場合）
    if (!mintExists) {
      logger.info('🪙 Reward Mint を作成中...');

      const tx2 = await program.methods
        .createRewardMint()
        .accounts({
          rewardMint: pdas.rewardMint,
          mintAuthority: pdas.mintAuthority,
          admin: wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      logger.success(`✅ Reward Mint ($WEED) 作成成功! TX: ${tx2}`);
      logger.info(`  - Mint Address: ${pdas.rewardMint.toBase58()}`);
      logger.info(`  - Token Name: Weed Token`);
      logger.info(`  - Token Symbol: WEED`);
    }

    // 5. 残高確認
    const balance = await connection.getBalance(wallet.publicKey);
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
    logger.error(
      `❌ エラーが発生しました: ${error instanceof Error ? error.message : String(error)}`
    );
    if (error instanceof Error && error.stack) {
      logger.error(error.stack);
    }
    process.exit(1);
  }
}

// メイン実行
initialize().catch(console.error);
