/**
 * Example TypeScript Code for Executing Rust Program Instructions
 * 
 * このファイルは、Solana上のRustプログラムの各命令を実行する
 * 簡単なTypeScriptスニペットの例を示します。
 * 
 * 使用可能な命令:
 * - Admin: initialize_config, create_reward_mint, update_config
 * - User: init_user
 * - Farm: buy_farm_space, upgrade_farm_space
 * - Rewards: claim_reward_with_referral_rewards
 * - Invites: create_invite_code, use_invite_code
 * - Seeds: purchase_seed_pack, open_seed_pack, plant_seed, remove_seed
 */

import { 
  Connection, 
  Keypair, 
  PublicKey, 
  LAMPORTS_PER_SOL,
  SystemProgram
} from '@solana/web3.js';
import { AnchorProvider, Wallet, BN } from '@coral-xyz/anchor';
import { getAssociatedTokenAddress } from '@solana/spl-token';

// プログラム定数
const PROGRAM_ID = new PublicKey('7r3R1S43BS9fQbh1eBhM63u8XZJd7bYRtgMrAQRNrfcB');
const RPC_URL = 'https://api.devnet.solana.com';

// ログ用のヘルパー関数
const log = {
  info: (message: string, data?: Record<string, unknown>) => {
    // eslint-disable-next-line no-console
    console.log(`ℹ️  ${message}`, data || '');
  },
  success: (message: string) => {
    // eslint-disable-next-line no-console
    console.log(`✅ ${message}`);
  },
  error: (message: string, error?: unknown) => {
    // eslint-disable-next-line no-console
    console.error(`❌ ${message}`, error || '');
  },
  header: (message: string) => {
    // eslint-disable-next-line no-console
    console.log(`\n🎮 === ${message} ===\n`);
  }
};

// 簡略化されたユーザー状態の型
interface UserState {
  owner: PublicKey;
  totalGrowPower?: number;
  hasFarmSpace?: boolean;
  lastHarvestTime?: number;
}

// 簡略化された農場スペースの型
interface FarmSpace {
  owner: PublicKey;
  level?: number;
  capacity?: number;
  seedCount?: number;
}

class FarmGameClient {
  private connection: Connection;

  constructor(wallet: Wallet, connection?: Connection) {
    this.connection = connection || new Connection(RPC_URL, 'confirmed');
    
    // Note: IDLが必要ですが、この例では概念的な実装です
    // 実際の実装では anchor build で生成されたIDLを使用してください
    new AnchorProvider(this.connection, wallet, {});
  }

  // =============== PDA計算ヘルパー ===============

  /**
   * 各種PDAアドレスを計算
   */
  private async calculatePDAs(userPubkey: PublicKey) {
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      PROGRAM_ID
    );

    const [userStatePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), userPubkey.toBuffer()],
      PROGRAM_ID
    );

    const [farmSpacePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('farm_space'), userPubkey.toBuffer()],
      PROGRAM_ID
    );

    const [rewardMintPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('reward_mint')],
      PROGRAM_ID
    );

    const [mintAuthorityPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('mint_authority')],
      PROGRAM_ID
    );

    const [seedStoragePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('seed_storage'), userPubkey.toBuffer()],
      PROGRAM_ID
    );

    return {
      config: configPDA,
      userState: userStatePDA,
      farmSpace: farmSpacePDA,
      rewardMint: rewardMintPDA,
      mintAuthority: mintAuthorityPDA,
      seedStorage: seedStoragePDA
    };
  }

  // =============== ADMIN INSTRUCTIONS ===============

  /**
   * システム設定を初期化（管理者のみ）
   */
  async initializeConfig(
    admin: Keypair,
    treasuryAddress: PublicKey,
    baseRate: number = 200,
    halvingInterval: number = 200 // 200秒
  ): Promise<string> {
    const pdas = await this.calculatePDAs(admin.publicKey);

    // 実際の実装では、ここでトランザクションを構築・送信
    // Buffer.from([
    //   0, // instruction discriminator for initialize_config
    //   ...new BN(baseRate).toArray('le', 8),
    //   ...new BN(halvingInterval).toArray('le', 8),
    //   ...treasuryAddress.toBuffer(),
    // ])

    log.info('🔧 Initialize Config:', {
      admin: admin.publicKey.toString(),
      treasury: treasuryAddress.toString(),
      baseRate,
      halvingInterval,
      configPDA: pdas.config.toString()
    });

    return 'initialize_config_signature_example';
  }

  /**
   * WEEDトークンミントを作成（管理者のみ）
   */
  async createRewardMint(admin: Keypair): Promise<string> {
    const pdas = await this.calculatePDAs(admin.publicKey);

    log.info('💰 Create Reward Mint:', {
      admin: admin.publicKey.toString(),
      rewardMint: pdas.rewardMint.toString(),
      mintAuthority: pdas.mintAuthority.toString()
    });

    return 'create_reward_mint_signature_example';
  }

  // =============== USER INSTRUCTIONS ===============

  /**
   * ユーザーアカウントを初期化
   */
  async initUser(user: Keypair, referrer?: PublicKey): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);

    log.info('👤 Initialize User:', {
      user: user.publicKey.toString(),
      referrer: referrer?.toString() || 'None',
      userStatePDA: pdas.userState.toString()
    });

    return 'init_user_signature_example';
  }

  // =============== FARM INSTRUCTIONS ===============

  /**
   * 農場スペースを購入（0.5 SOL）
   */
  async buyFarmSpace(user: Keypair, treasuryAddress: PublicKey): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);
    // Farm space costs 0.5 SOL

    log.info('🚜 Buy Farm Space:', {
      user: user.publicKey.toString(),
      cost: '0.5 SOL',
      treasury: treasuryAddress.toString(),
      farmSpacePDA: pdas.farmSpace.toString()
    });

    return 'buy_farm_space_signature_example';
  }

  /**
   * 農場スペースをアップグレード
   */
  async upgradeFarmSpace(user: Keypair): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);

    // アップグレードコスト（WEED）
    const upgradeCosts = [3500, 18000, 20000, 25000]; // Level 1→2, 2→3, 3→4, 4→5

    log.info('⬆️ Upgrade Farm Space:', {
      user: user.publicKey.toString(),
      farmSpacePDA: pdas.farmSpace.toString(),
      possibleCosts: upgradeCosts
    });

    return 'upgrade_farm_space_signature_example';
  }

  // =============== REWARD INSTRUCTIONS ===============

  /**
   * 報酬を請求（農場報酬 + 紹介報酬）
   */
  async claimRewards(user: Keypair): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);
    
    // ユーザーのトークンアカウントを計算
    const userTokenAccount = await getAssociatedTokenAddress(
      pdas.rewardMint,
      user.publicKey
    );

    log.info('💎 Claim Rewards:', {
      user: user.publicKey.toString(),
      farmSpace: pdas.farmSpace.toString(),
      userToken: userTokenAccount.toString(),
      rewardMint: pdas.rewardMint.toString()
    });

    return 'claim_rewards_signature_example';
  }

  // =============== INVITE INSTRUCTIONS ===============

  /**
   * 招待コードを作成
   */
  async createInviteCode(user: Keypair, inviteCode: string): Promise<string> {
    if (inviteCode.length !== 8) {
      throw new Error('Invite code must be exactly 8 characters');
    }

    const codeBytes = Buffer.from(inviteCode, 'utf8');
    const [inviteCodePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('invite_code'), codeBytes],
      PROGRAM_ID
    );

    log.info('📨 Create Invite Code:', {
      user: user.publicKey.toString(),
      inviteCode,
      inviteCodePDA: inviteCodePDA.toString()
    });

    return 'create_invite_code_signature_example';
  }

  /**
   * 招待コードを使用
   */
  async useInviteCode(
    newUser: Keypair, 
    inviteCode: string, 
    inviterPubkey: PublicKey
  ): Promise<string> {
    const codeBytes = Buffer.from(inviteCode, 'utf8');
    const [inviteCodePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('invite_code'), codeBytes],
      PROGRAM_ID
    );

    log.info('🎫 Use Invite Code:', {
      newUser: newUser.publicKey.toString(),
      inviter: inviterPubkey.toString(),
      inviteCode,
      inviteCodePDA: inviteCodePDA.toString()
    });

    return 'use_invite_code_signature_example';
  }

  // =============== SEED INSTRUCTIONS ===============

  /**
   * シードパックを購入（300 WEED）
   */
  async purchaseSeedPack(
    user: Keypair, 
    quantity: number = 1,
    maxVrfFee: number = 0.002 * LAMPORTS_PER_SOL
  ): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);
    // Generate entropy seed for VRF
    Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
    
    log.info('📦 Purchase Seed Pack:', {
      user: user.publicKey.toString(),
      quantity,
      cost: `${300 * quantity} WEED`,
      vrfFee: `${maxVrfFee / LAMPORTS_PER_SOL} SOL`,
      seedStorage: pdas.seedStorage.toString()
    });

    return 'purchase_seed_pack_signature_example';
  }

  /**
   * シードパックを開封
   */
  async openSeedPack(user: Keypair, quantity: number = 1): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);

    log.info('🎁 Open Seed Pack:', {
      user: user.publicKey.toString(),
      quantity,
      seedStorage: pdas.seedStorage.toString()
    });

    return 'open_seed_pack_signature_example';
  }

  /**
   * 種を植える
   */
  async plantSeed(user: Keypair, seedId: number): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);

    log.info('🌱 Plant Seed:', {
      user: user.publicKey.toString(),
      seedId,
      farmSpace: pdas.farmSpace.toString()
    });

    return 'plant_seed_signature_example';
  }

  /**
   * 種を除去
   */
  async removeSeed(user: Keypair, seedId: number): Promise<string> {
    const pdas = await this.calculatePDAs(user.publicKey);

    log.info('🗑️ Remove Seed:', {
      user: user.publicKey.toString(),
      seedId,
      farmSpace: pdas.farmSpace.toString()
    });

    return 'remove_seed_signature_example';
  }

  // =============== クエリ関数 ===============

  /**
   * ユーザー状態を取得
   */
  async getUserState(userPubkey: PublicKey): Promise<UserState | null> {
    const pdas = await this.calculatePDAs(userPubkey);
    
    try {
      const accountInfo = await this.connection.getAccountInfo(pdas.userState);
      if (!accountInfo) {
        return null;
      }
      
      log.info('📊 User State:', {
        user: userPubkey.toString(),
        exists: true,
        dataLength: accountInfo.data.length
      });
      
      return {
        owner: userPubkey,
        // 実際の実装では、アカウントデータをデシリアライズ
        // totalGrowPower: ...,
        // hasFarmSpace: ...,
        // lastHarvestTime: ...
      };
    } catch {
      log.info('❌ User State not found:', { user: userPubkey.toString() });
      return null;
    }
  }

  /**
   * 農場スペース情報を取得
   */
  async getFarmSpace(userPubkey: PublicKey): Promise<FarmSpace | null> {
    const pdas = await this.calculatePDAs(userPubkey);
    
    try {
      const accountInfo = await this.connection.getAccountInfo(pdas.farmSpace);
      if (!accountInfo) {
        return null;
      }
      
      log.info('🚜 Farm Space:', {
        user: userPubkey.toString(),
        exists: true,
        dataLength: accountInfo.data.length
      });
      
      return {
        owner: userPubkey,
        // level: ...,
        // capacity: ...,
        // seedCount: ...
      };
    } catch {
      log.info('❌ Farm Space not found:', { user: userPubkey.toString() });
      return null;
    }
  }
}

// =============== 使用例 ===============

/**
 * 新規ユーザーの完全なジャーニー例
 */
async function completeUserJourney() {
  log.header('COMPLETE USER JOURNEY EXAMPLE');

  // セットアップ
  const admin = Keypair.generate();
  const treasury = Keypair.generate();
  const user = Keypair.generate();
  const connection = new Connection(RPC_URL, 'confirmed');
  const wallet = new Wallet(user);
  const client = new FarmGameClient(wallet, connection);

  log.info('👥 Generated Accounts:', {
    admin: admin.publicKey.toString(),
    treasury: treasury.publicKey.toString(),
    user: user.publicKey.toString()
  });

  try {
    // 1. 管理者がシステムを初期化
    log.info('🔧 Step 1: Initialize System');
    await client.initializeConfig(admin, treasury.publicKey, 200, 200);
    await client.createRewardMint(admin);
    log.success('System initialized');

    // 2. ユーザーアカウント作成
    log.info('👤 Step 2: Create User Account');
    await client.initUser(user);
    log.success('User account created');

    // 3. 農場スペース購入
    log.info('🚜 Step 3: Purchase Farm Space');
    await client.buyFarmSpace(user, treasury.publicKey);
    log.success('Farm space purchased');

    // 4. シードパック購入・開封
    log.info('📦 Step 4: Purchase & Open Seed Packs');
    await client.purchaseSeedPack(user, 3);
    await client.openSeedPack(user, 3);
    log.success('Seed packs opened');

    // 5. 種を植える
    log.info('🌱 Step 5: Plant Seeds');
    await client.plantSeed(user, 1);
    await client.plantSeed(user, 2);
    log.success('Seeds planted');

    // 6. 報酬請求（時間経過後）
    log.info('💎 Step 6: Claim Rewards');
    await client.claimRewards(user);
    log.success('Rewards claimed');

    // 7. 農場アップグレード
    log.info('⬆️ Step 7: Upgrade Farm');
    await client.upgradeFarmSpace(user);
    log.success('Farm upgraded');

    // 8. 招待システム
    log.info('📨 Step 8: Invite System');
    await client.createInviteCode(user, 'WELCOME1');
    
    const newUser = Keypair.generate();
    await client.useInviteCode(newUser, 'WELCOME1', user.publicKey);
    log.success('Invite system used');

    log.success('🎉 Complete user journey finished successfully!');

  } catch (error) {
    log.error('❌ Error in user journey:', error);
  }
}

/**
 * 農場運営シミュレーション例
 */
async function farmingSimulation() {
  log.header('FARMING SIMULATION EXAMPLE');

  const user = Keypair.generate();
  const wallet = new Wallet(user);
  const client = new FarmGameClient(wallet);

  // 農場の状態確認
  const userState = await client.getUserState(user.publicKey);
  const farmSpace = await client.getFarmSpace(user.publicKey);

  log.info('Current farm status:', { userState, farmSpace });

  // 複数のシードパック購入
  for (let i = 0; i < 5; i++) {
    log.info(`📦 Purchasing seed pack ${i + 1}/5`);
    await client.purchaseSeedPack(user, 1);
    await client.openSeedPack(user, 1);
  }

  // 種植えシミュレーション
  for (let seedId = 1; seedId <= 10; seedId++) {
    log.info(`🌱 Planting seed ${seedId}`);
    await client.plantSeed(user, seedId);
  }

  // 定期的な報酬請求シミュレーション
  for (let day = 1; day <= 7; day++) {
    log.info(`💎 Day ${day}: Claiming rewards`);
    await client.claimRewards(user);
  }

  log.success('🎯 Farming simulation completed!');
}

/**
 * 招待ネットワーク構築例
 */
async function referralNetworkExample() {
  log.header('REFERRAL NETWORK EXAMPLE');

  const wallet = new Wallet(Keypair.generate());
  const client = new FarmGameClient(wallet);

  // 親ユーザー
  const parent = Keypair.generate();
  await client.initUser(parent);
  await client.createInviteCode(parent, 'PARENT01');

  // 子ユーザー達
  const children = [];
  for (let i = 0; i < 3; i++) {
    const child = Keypair.generate();
    await client.useInviteCode(child, 'PARENT01', parent.publicKey);
    await client.createInviteCode(child, `CHILD00${i+1}`);
    children.push(child);
  }

  // 孫ユーザー達
  for (let i = 0; i < children.length; i++) {
    const grandchild = Keypair.generate();
    await client.useInviteCode(grandchild, `CHILD00${i+1}`, children[i].publicKey);
  }

  log.success(`👨‍👩‍👧‍👦 Created referral network: 1 parent → ${children.length} children → ${children.length} grandchildren`);
}

// =============== エクスポート ===============

export {
  FarmGameClient,
  completeUserJourney,
  farmingSimulation,
  referralNetworkExample,
  PROGRAM_ID,
  RPC_URL
};

// =============== 実行例 ===============

// メイン実行関数
async function runExamples(mode = 'all') {
  log.header(`Running Farm Game Examples - ${mode.toUpperCase()}`);
  
  try {
    switch (mode) {
      case 'journey':
        await completeUserJourney();
        break;
      case 'farming':
        await farmingSimulation();
        break;
      case 'referral':
        await referralNetworkExample();
        break;
      case 'all':
      default:
        await completeUserJourney();
        await farmingSimulation();
        await referralNetworkExample();
        break;
    }
    log.success('\n✨ All examples completed successfully!');
  } catch (error) {
    log.error('\n❌ Example execution failed:', error);
    process.exit(1);
  }
}

// コマンドライン引数の処理
function parseArgs(): string {
  const args = process.argv.slice(2);
  return args[0] || 'all';
}

// スクリプトとして直接実行された場合
if (import.meta.main) {
  const mode = parseArgs();
  runExamples(mode);
}