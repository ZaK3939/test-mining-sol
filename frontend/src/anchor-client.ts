// Anchor client for Facility Game program

import { AnchorProvider, Program, Idl, BN } from '@coral-xyz/anchor';
import { Connection, PublicKey, SystemProgram } from '@solana/web3.js';
import {
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';
import { logger } from './logger';
import { PDAHelper, type PDAs } from './utils/pda-helper';
import { cacheManager } from './utils/cache-manager';
import { BatchFetcher } from './utils/batch-fetcher';
import { ERROR_MESSAGES } from './utils/constants';
import { 
  generateUserEntropySeed, 
  prepareSeedPackPurchaseAccounts,
  waitForEntropyResult,
  deriveEntropyRequestPDA 
} from './utils/pyth-entropy-helper';
import {
  type WalletAdapter,
  type UserStateAccount,
  type FarmSpaceAccount,
  type ConfigAccount,
  type SeedPackAccount,
  type SeedAccount,
  type TransactionResult,
  type ProgramError,
  isUserStateAccount,
  isFarmSpaceAccount,
  isConfigAccount,
  safeBNToNumber,
} from './types/program-types';
import idl from './idl/farm_game.json';

// Type for the Farm Game IDL
type FarmGameIDL = Idl;

// Program interface for type-safe account access
interface ProgramAccountNamespace {
  userState: {
    fetchNullable(address: PublicKey): Promise<UserStateAccount | null>;
  };
  farmSpace: {
    fetchNullable(address: PublicKey): Promise<FarmSpaceAccount | null>;
  };
  config: {
    fetchNullable(address: PublicKey): Promise<ConfigAccount | null>;
  };
  seedPack: {
    fetchNullable(address: PublicKey): Promise<SeedPackAccount | null>;
  };
  seed: {
    fetchNullable(address: PublicKey): Promise<SeedAccount | null>;
  };
}

// Program interface for type-safe methods
interface ProgramMethodNamespace {
  initializeConfig(
    baseRate: BN | null,
    halvingInterval: BN | null,
    treasury: PublicKey,
    protocolReferralAddress: PublicKey | null
  ): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  createRewardMint(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  initUser(referrer?: PublicKey | null): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  buyFarmSpace(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  claimReward(): {
    accounts(accounts: Record<string, PublicKey>): {
      preInstructions?(instructions: unknown[]): {
        rpc(): Promise<string>;
      };
      rpc(): Promise<string>;
    };
  };
  plantSeed(seedId: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  removeSeed(seedId: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  upgradeFarmSpace(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  completeFarmSpaceUpgrade(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  purchaseSeedPack(quantity: number, userEntropySeed: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  openSeedPack(quantity: number): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  createInviteCode(inviteCode: number[]): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  useInviteCode(inviteCode: number[]): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  distributeReferralOnClaim(baseReward: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  claimReferralRewards(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  transferWithFee(amount: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
}

// Enhanced program interface
interface TypedProgram {
  programId: PublicKey;
  coder: unknown;
  account: ProgramAccountNamespace;
  methods: ProgramMethodNamespace;
}

export class AnchorClient {
  private program: TypedProgram;
  private provider: AnchorProvider;
  private batchFetcher: BatchFetcher;

  constructor(connection: Connection, wallet: WalletAdapter) {
    // Create provider
    this.provider = new AnchorProvider(connection, wallet, { preflightCommitment: 'confirmed' });

    // Initialize program with IDL - casting required due to Anchor type limitations
    const anchorProgram = new Program(idl as FarmGameIDL, this.provider);
    this.program = {
      programId: anchorProgram.programId,
      coder: anchorProgram.coder,
      account: anchorProgram.account as ProgramAccountNamespace,
      methods: anchorProgram.methods as ProgramMethodNamespace,
    };

    // Initialize batch fetcher for performance optimization
    this.batchFetcher = new BatchFetcher(connection);
  }

  // Calculate PDAs using shared helper with caching
  async calculatePDAs(userPublicKey: PublicKey): Promise<PDAs> {
    // Check cache first
    const cached = cacheManager.getCachedPDAs(userPublicKey);
    if (cached) {
      return cached;
    }

    // Calculate and cache PDAs
    const pdas = await PDAHelper.calculatePDAs(userPublicKey, this.program.programId);
    cacheManager.cachePDAs(userPublicKey, pdas);

    return pdas;
  }

  // Initialize config (admin only)
  async initializeConfig(
    baseRate: BN | null = new BN(100),
    halvingInterval: BN | null = new BN(518400), // 6 days
    treasury?: PublicKey,
    protocolReferralAddress?: PublicKey
  ): Promise<string> {
    try {
      logger.info('⚙️ 設定を初期化中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Use admin as treasury if not provided
      const treasuryAccount = treasury || userPublicKey;

      const tx = await this.program.methods
        .initializeConfig(baseRate, halvingInterval, treasuryAccount, protocolReferralAddress || null)
        .accounts({
          config: pdas.config,
          admin: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`設定初期化成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`設定初期化エラー: ${programError.message}`);
      throw programError;
    }
  }

  // Create reward mint (admin only)
  async createRewardMint(): Promise<string> {
    try {
      logger.info('🪙 報酬ミントを作成中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Create the correct metadata account PDA as required by Token Metadata Program
      const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
        'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
      );
      const [metadataAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), pdas.rewardMint.toBuffer()],
        TOKEN_METADATA_PROGRAM_ID
      );

      logger.info(`Metadata account: ${metadataAccount.toString()}`);
      logger.info(`Reward mint: ${pdas.rewardMint.toString()}`);

      const tx = await this.program.methods
        .createRewardMint()
        .accounts({
          rewardMint: pdas.rewardMint,
          mintAuthority: pdas.mintAuthority,
          metadataAccount: metadataAccount,
          admin: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: new PublicKey('SysvarRent111111111111111111111111111111111'),
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .rpc();

      logger.success(`報酬ミント作成成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`報酬ミント作成エラー: ${programError.message}`);
      throw programError;
    }
  }

  // Initialize user account
  async initUser(): Promise<TransactionResult> {
    try {
      logger.info('👤 ユーザーアカウントを初期化中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user already initialized with type validation
      const userStateAccount = await this.program.account.userState.fetchNullable(pdas.userState);
      if (userStateAccount && isUserStateAccount(userStateAccount)) {
        logger.warn(ERROR_MESSAGES.USER_ALREADY_INITIALIZED);
        return 'already_initialized';
      }

      // Send initUser transaction
      const tx = await this.program.methods
        .initUser(null)
        .accounts({
          userState: pdas.userState,
          user: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`ユーザー初期化成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`ユーザー初期化エラー: ${programError.message}`);
      throw programError;
    }
  }

  // Buy farm space
  async buyFarmSpace(): Promise<TransactionResult> {
    try {
      logger.info('🌾 農場スペースを購入中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user is initialized with type validation
      const userStateAccount = await this.program.account.userState.fetchNullable(pdas.userState);
      if (!userStateAccount || !isUserStateAccount(userStateAccount)) {
        logger.error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
        throw new Error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
      }

      // Check if user already has farm space
      if (userStateAccount.hasFarmSpace) {
        logger.warn('農場スペースは既に所有済みです');
        return 'already_owned';
      }

      // Get config for treasury address
      const config = await this.fetchConfig();
      if (!config) {
        throw new Error('Config not found');
      }

      // Calculate PDAs for farm space, global stats and initial seed
      const [globalStatsPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('global_stats')],
        this.program.programId
      );

      const [initialSeedPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('seed'), userPublicKey.toBuffer(), new BN(0).toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );

      // Send buyFarmSpace transaction
      const tx = await this.program.methods
        .buyFarmSpace()
        .accounts({
          userState: pdas.userState,
          farmSpace: pdas.farmSpace,
          initialSeed: initialSeedPDA,
          config: pdas.config,
          globalStats: globalStatsPDA,
          treasury: config.treasury,
          user: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`農場スペース購入成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`農場スペース購入エラー: ${programError.message}`);
      throw programError;
    }
  }

  // Claim rewards
  async claimRewards(): Promise<string> {
    try {
      logger.info('💰 報酬を請求中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user is initialized
      const userStateAccount = await this.program.account.userState.fetchNullable(pdas.userState);
      if (!userStateAccount) {
        logger.error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
        throw new Error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
      }

      // Check if user has farm space
      if (!userStateAccount.hasFarmSpace) {
        logger.error('農場スペースを所有していません');
        throw new Error('農場スペースを所有していません');
      }

      // Get global stats PDA
      const [globalStatsPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('global_stats')],
        this.program.programId
      );

      // Get or create user token account
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      // Check if token account exists
      const tokenAccountInfo = await this.provider.connection.getAccountInfo(userTokenAccount);

      // If token account doesn't exist, we need to create it
      let createATAInstruction = null;
      if (!tokenAccountInfo) {
        logger.info('トークンアカウントを作成中...');
        createATAInstruction = createAssociatedTokenAccountInstruction(
          userPublicKey, // payer
          userTokenAccount, // associated token account
          userPublicKey, // owner
          pdas.rewardMint // mint
        );
      }

      // Build transaction
      let txBuilder = this.program.methods.claimReward().accounts({
        userState: pdas.userState,
        config: pdas.config,
        globalStats: globalStatsPDA,
        rewardMint: pdas.rewardMint,
        mintAuthority: pdas.mintAuthority,
        userTokenAccount: userTokenAccount,
        user: userPublicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      });

      // Add ATA creation instruction if needed
      if (createATAInstruction && txBuilder.preInstructions) {
        txBuilder = txBuilder.preInstructions([createATAInstruction]);
      }

      // Send transaction
      const tx = await txBuilder.rpc();

      logger.success(`報酬請求成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`報酬請求エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Fetch user state with caching and type validation
  async fetchUserState(userPublicKey: PublicKey): Promise<UserStateAccount | null> {
    try {
      // Check cache first
      const cached = cacheManager.getCachedUserState(userPublicKey);
      if (cached) {
        return cached;
      }

      const pdas = await this.calculatePDAs(userPublicKey);
      const userState = await this.program.account.userState.fetchNullable(pdas.userState);

      if (userState && !isUserStateAccount(userState)) {
        logger.warn('取得したユーザー状態の型が不正です');
        return null;
      }

      // Cache the result
      if (userState) {
        cacheManager.cacheUserState(userPublicKey, userState);
      }

      return userState;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`ユーザー状態取得エラー: ${programError.message}`);
      return null;
    }
  }

  // Fetch farm space with caching and type validation
  async fetchFarmSpace(userPublicKey: PublicKey): Promise<FarmSpaceAccount | null> {
    try {
      // Check cache first
      const cached = cacheManager.getCachedFarmSpace(userPublicKey);
      if (cached) {
        return cached;
      }

      const pdas = await this.calculatePDAs(userPublicKey);
      const farmSpace = await this.program.account.farmSpace.fetchNullable(pdas.farmSpace);

      if (farmSpace && !isFarmSpaceAccount(farmSpace)) {
        logger.warn('取得した農場スペース状態の型が不正です');
        return null;
      }

      // Cache the result
      if (farmSpace) {
        cacheManager.cacheFarmSpace(userPublicKey, farmSpace);
      }

      return farmSpace;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`農場スペース状態取得エラー: ${programError.message}`);
      return null;
    }
  }

  // Fetch facility (alias for fetchFarmSpace for backward compatibility)
  async fetchFacility(userPublicKey: PublicKey): Promise<FarmSpaceAccount | null> {
    return this.fetchFarmSpace(userPublicKey);
  }

  // Fetch config with caching and type validation
  async fetchConfig(): Promise<ConfigAccount | null> {
    try {
      // Check cache first
      const cached = cacheManager.getCachedConfig(this.program.programId);
      if (cached) {
        return cached;
      }

      const [configPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('config')],
        this.program.programId
      );
      const config = await this.program.account.config.fetchNullable(configPDA);

      if (config && !isConfigAccount(config)) {
        logger.warn('取得した設定の型が不正です');
        return null;
      }

      // Cache the result
      if (config) {
        cacheManager.cacheConfig(this.program.programId, config);
      }

      return config;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`設定取得エラー: ${programError.message}`);
      return null;
    }
  }

  // Initialize user with referrer
  async initUserWithReferrer(referrerPublicKey?: PublicKey): Promise<string> {
    try {
      logger.info('👤 紹介者付きでユーザーアカウントを初期化中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user already initialized
      const userStateAccount = await this.program.account.userState.fetchNullable(pdas.userState);
      if (userStateAccount) {
        logger.warn(ERROR_MESSAGES.USER_ALREADY_INITIALIZED);
        return 'already_initialized';
      }

      const accounts: Record<string, PublicKey> = {
        userState: pdas.userState,
        user: userPublicKey,
        systemProgram: SystemProgram.programId,
      };

      // Add referrer if provided
      if (referrerPublicKey) {
        const referrerPDAs = await this.calculatePDAs(referrerPublicKey);
        accounts.referrerState = referrerPDAs.userState;
      }

      // Send initUser transaction
      const tx = await this.program.methods
        .initUser(referrerPublicKey || null)
        .accounts(accounts)
        .rpc();

      logger.success(`ユーザー初期化成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `ユーザー初期化エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Distribute referral rewards during claim process
  async distributeReferralRewards(baseReward: number): Promise<string> {
    try {
      logger.info('💰 紹介報酬を配布中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Get user state to find referrer
      const userState = await this.fetchUserState(userPublicKey);
      if (!userState?.referrer) {
        throw new Error('No referrer found');
      }

      // Calculate referrer PDAs
      const level1ReferrerPDAs = await this.calculatePDAs(userState.referrer);
      const level1TokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userState.referrer);

      const tx = await this.program.methods
        .distributeReferralOnClaim(new BN(baseReward))
        .accounts({
          claimantState: pdas.userState,
          level1ReferrerState: level1ReferrerPDAs.userState,
          level1TokenAccount: level1TokenAccount,
          rewardMint: pdas.rewardMint,
          mintAuthority: pdas.mintAuthority,
          config: pdas.config,
          claimant: userPublicKey,
          level1Referrer: userState.referrer,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`紹介報酬配布成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`紹介報酬配布エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Claim referral rewards
  async claimReferralRewards(): Promise<string> {
    try {
      logger.info('💰 紹介報酬を請求中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      const tx = await this.program.methods
        .claimReferralRewards()
        .accounts({
          referrerState: pdas.userState,
          rewardMint: pdas.rewardMint,
          mintAuthority: pdas.mintAuthority,
          referrerTokenAccount: userTokenAccount,
          referrer: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`紹介報酬請求成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`紹介報酬請求エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Upgrade farm space (instant upgrade)
  async upgradeFarmSpace(): Promise<string> {
    try {
      logger.info('🔧 農場スペースをアップグレード中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      const tx = await this.program.methods
        .upgradeFarmSpace()
        .accounts({
          userState: pdas.userState,
          farmSpace: pdas.farmSpace,
          rewardMint: pdas.rewardMint,
          userTokenAccount: userTokenAccount,
          user: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`農場スペースアップグレード成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `農場スペースアップグレードエラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }


  // Plant seed in farm space
  async plantSeed(seedId: number): Promise<string> {
    try {
      logger.info('🌱 シードを植え付け中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Calculate seed PDA
      const [seedPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('seed'), userPublicKey.toBuffer(), new BN(seedId).toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );

      // Get global stats PDA
      const [globalStatsPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('global_stats')],
        this.program.programId
      );

      const tx = await this.program.methods
        .plantSeed(new BN(seedId))
        .accounts({
          userState: pdas.userState,
          farmSpace: pdas.farmSpace,
          seed: seedPDA,
          globalStats: globalStatsPDA,
          user: userPublicKey,
        })
        .rpc();

      logger.success(`シード植え付け成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`シード植え付けエラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Remove seed from farm space
  async removeSeed(seedId: number): Promise<string> {
    try {
      logger.info('🌾 シードを除去中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Calculate seed PDA
      const [seedPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('seed'), userPublicKey.toBuffer(), new BN(seedId).toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );

      // Get global stats PDA
      const [globalStatsPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('global_stats')],
        this.program.programId
      );

      const tx = await this.program.methods
        .removeSeed(new BN(seedId))
        .accounts({
          userState: pdas.userState,
          farmSpace: pdas.farmSpace,
          seed: seedPDA,
          globalStats: globalStatsPDA,
          user: userPublicKey,
        })
        .rpc();

      logger.success(`シード除去成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`シード除去エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Transfer with fee
  async transferWithFee(recipientPublicKey: PublicKey, amount: number): Promise<string> {
    try {
      logger.info('💸 手数料付き転送を実行中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const senderTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);
      const recipientTokenAccount = await getAssociatedTokenAddress(
        pdas.rewardMint,
        recipientPublicKey
      );
      const treasuryTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, pdas.treasury);

      const tx = await this.program.methods
        .transferWithFee(new BN(amount))
        .accounts({
          config: pdas.config,
          senderTokenAccount: senderTokenAccount,
          recipientTokenAccount: recipientTokenAccount,
          treasuryTokenAccount: treasuryTokenAccount,
          sender: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`手数料付き転送成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `手数料付き転送エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Purchase seed pack with Pyth Entropy
  async purchaseSeedPack(quantity: number): Promise<{
    transactionSignature: string;
    entropySequence: BN;
    entropyRequestAccount: PublicKey;
  }> {
    try {
      logger.info('📦 Pyth Entropyを使用してシードパックを購入中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      // Generate cryptographically secure user entropy seed
      const userEntropySeed = generateUserEntropySeed();
      logger.info(`🎲 ユーザーエントロピーシード生成: ${userEntropySeed.toString()}`);

      // Generate seed pack PDA
      const config = await this.fetchConfig();
      if (!config) throw new Error('Config not found');

      const seedPackId = config.seedPackCounter;
      const [seedPackPDA] = await PublicKey.findProgramAddress(
        [
          Buffer.from('seed_pack'),
          userPublicKey.toBuffer(),
          seedPackId.toArrayLike(Buffer, 'le', 8),
        ],
        this.program.programId
      );

      // Prepare Pyth Entropy accounts
      // Note: In a real implementation, you would get the actual entropy sequence from Pyth
      const entropySequence = new BN(Date.now() + Math.floor(Math.random() * 1000000));
      const entropyAccounts = prepareSeedPackPurchaseAccounts(userPublicKey, seedPackPDA, entropySequence);

      const tx = await this.program.methods
        .purchaseSeedPack(quantity, userEntropySeed)
        .accounts({
          userState: pdas.userState,
          config: pdas.config,
          seedPack: seedPackPDA,
          rewardMint: pdas.rewardMint,
          userTokenAccount: userTokenAccount,
          entropyProvider: entropyAccounts.entropyProvider,
          entropyRequest: entropyAccounts.entropyRequest,
          user: userPublicKey,
          pythEntropyProgram: entropyAccounts.pythEntropyProgram,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`シードパック購入成功! トランザクション: ${tx}`);
      logger.info(`🔗 エントロピー要求アカウント: ${entropyAccounts.entropyRequest.toString()}`);
      
      return {
        transactionSignature: tx,
        entropySequence,
        entropyRequestAccount: entropyAccounts.entropyRequest,
      };
    } catch (error) {
      logger.error(
        `シードパック購入エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Open seed pack with Pyth Entropy validation
  async openSeedPack(
    packId: number, 
    quantity: number,
    entropyRequestAccount?: PublicKey
  ): Promise<string> {
    try {
      logger.info('📦 Pyth Entropyを使用してシードパックを開封中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      const [seedPackPDA] = await PublicKey.findProgramAddress(
        [
          Buffer.from('seed_pack'),
          userPublicKey.toBuffer(),
          new BN(packId).toArrayLike(Buffer, 'le', 8),
        ],
        this.program.programId
      );

      // Get seed storage PDA
      const [seedStoragePDA] = await PublicKey.findProgramAddress(
        [Buffer.from('seed_storage'), userPublicKey.toBuffer()],
        this.program.programId
      );

      // If entropy request account is provided, wait for entropy result
      if (entropyRequestAccount) {
        logger.info('⏳ Pyth Entropyの結果を待機中...');
        try {
          const entropyResult = await waitForEntropyResult(
            this.provider.connection,
            entropyRequestAccount,
            {
              maxAttempts: 30,
              pollIntervalMs: 2000,
              timeoutMs: 60000,
            }
          );
          logger.success(`✅ エントロピー結果取得: ${entropyResult.randomValue.toString()}`);
        } catch (error) {
          logger.warn(`⚠️ エントロピー結果待機タイムアウト。既存の乱数を使用: ${error}`);
          // Continue with fallback - in a real implementation, you might want to retry or error
        }
      }

      // Prepare Pyth Entropy program account
      const pythEntropyAccounts = prepareSeedPackPurchaseAccounts(userPublicKey, seedPackPDA, new BN(0));

      const tx = await this.program.methods
        .openSeedPack(quantity)
        .accounts({
          seedPack: seedPackPDA,
          config: pdas.config,
          seedStorage: seedStoragePDA,
          entropyRequest: entropyRequestAccount || pythEntropyAccounts.entropyRequest,
          user: userPublicKey,
          pythEntropyProgram: pythEntropyAccounts.pythEntropyProgram,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`シードパック開封成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `シードパック開封エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Create invite code
  async createInviteCode(inviteCode: string): Promise<string> {
    try {
      logger.info('🎟️ 招待コードを作成中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Convert invite code to byte array
      const inviteCodeBytes = Array.from(Buffer.from(inviteCode.padEnd(8, '\0')).slice(0, 8));

      const [inviteCodePDA] = await PublicKey.findProgramAddress(
        [Buffer.from('invite_code'), Buffer.from(inviteCodeBytes)],
        this.program.programId
      );

      const tx = await this.program.methods
        .createInviteCode(inviteCodeBytes)
        .accounts({
          inviteCodeAccount: inviteCodePDA,
          config: pdas.config,
          inviter: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`招待コード作成成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `招待コード作成エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Use invite code
  async useInviteCode(inviteCode: string): Promise<string> {
    try {
      logger.info('🎟️ 招待コードを使用中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Convert invite code to byte array
      const inviteCodeBytes = Array.from(Buffer.from(inviteCode.padEnd(8, '\0')).slice(0, 8));

      const [inviteCodePDA] = await PublicKey.findProgramAddress(
        [Buffer.from('invite_code'), Buffer.from(inviteCodeBytes)],
        this.program.programId
      );

      // Get the invite code account to find the inviter
      const inviteCodeAccount = await this.program.account.inviteCode?.fetchNullable?.(inviteCodePDA);
      if (!inviteCodeAccount) {
        throw new Error('Invite code not found');
      }

      const tx = await this.program.methods
        .useInviteCode(inviteCodeBytes)
        .accounts({
          inviteCodeAccount: inviteCodePDA,
          userState: pdas.userState,
          config: pdas.config,
          invitee: userPublicKey,
          inviter: inviteCodeAccount.inviter,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`招待コード使用成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `招待コード使用エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Fetch mystery box
  async fetchMysteryBox(
    userPublicKey: PublicKey,
    mysteryBoxId: number
  ): Promise<MysteryBoxAccount | null> {
    try {
      const [mysteryBoxPDA] = await PublicKey.findProgramAddress(
        [
          Buffer.from('mystery_box'),
          userPublicKey.toBuffer(),
          new BN(mysteryBoxId).toArrayLike(Buffer, 'le', 8),
        ],
        this.program.programId
      );
      const mysteryBox = await this.program.account.mysteryBox.fetchNullable(mysteryBoxPDA);
      return mysteryBox;
    } catch (error) {
      logger.error(
        `ミステリーボックス取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      return null;
    }
  }

  // Fetch seed
  async fetchSeed(userPublicKey: PublicKey, seedId: number): Promise<SeedAccount | null> {
    try {
      const [seedPDA] = await PublicKey.findProgramAddress(
        [
          Buffer.from('seed'),
          userPublicKey.toBuffer(),
          new BN(seedId).toArrayLike(Buffer, 'le', 8),
        ],
        this.program.programId
      );
      const seed = await this.program.account.seed.fetchNullable(seedPDA);
      return seed;
    } catch (error) {
      logger.error(`シード取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
  }

  // Get token balance with safe conversion
  async getTokenBalance(userPublicKey: PublicKey): Promise<number> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      const tokenAccountInfo = await this.provider.connection.getTokenAccountBalance(
        userTokenAccount
      );

      const amount = new BN(tokenAccountInfo.value.amount);
      const decimals = tokenAccountInfo.value.decimals;

      // Use safe conversion to avoid overflow
      return safeBNToNumber(amount) / Math.pow(10, decimals);
    } catch {
      // Token account might not exist yet
      return 0;
    }
  }

  // Get connection instance
  getConnection(): Connection {
    return this.provider.connection;
  }

  // Get user's token account address
  async getTokenAccount(userPublicKey: PublicKey): Promise<PublicKey> {
    const pdas = await this.calculatePDAs(userPublicKey);
    return await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);
  }

  // Optimized batch fetching for complete game state
  async fetchCompleteGameState(userPublicKey: PublicKey): Promise<{
    userState: UserStateAccount | null;
    facility: FacilityAccount | null;
    config: ConfigAccount | null;
    tokenBalance: number;
    userInitialized: boolean;
    hasFacility: boolean;
    growPower: number;
    pendingReferralRewards: number;
  }> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);

      // Create batch request for all game state accounts
      const batchRequests = BatchFetcher.createGameStateBatch(userPublicKey, {
        userState: pdas.userState,
        facility: pdas.facility,
        config: pdas.config,
        rewardMint: pdas.rewardMint,
      });

      // Fetch all accounts in a single batch
      const results = await this.batchFetcher.fetchMultipleAccounts(batchRequests);

      // Process results with caching
      let userState: UserStateAccount | null = null;
      let facility: FacilityAccount | null = null;
      let config: ConfigAccount | null = null;

      for (const result of results) {
        if (result.account === null) continue;

        switch (result.name) {
          case 'userState':
            try {
              const decoded = this.program.coder.accounts.decode('userState', result.account.data);
              if (isUserStateAccount(decoded)) {
                userState = decoded;
                cacheManager.cacheUserState(userPublicKey, decoded);
              }
            } catch (e) {
              logger.warn(`UserState decode error: ${e}`);
            }
            break;

          case 'facility':
            try {
              const decoded = this.program.coder.accounts.decode('facility', result.account.data);
              if (isFacilityAccount(decoded)) {
                facility = decoded;
                cacheManager.cacheFacility(userPublicKey, decoded);
              }
            } catch (e) {
              logger.warn(`Facility decode error: ${e}`);
            }
            break;

          case 'config':
            try {
              const decoded = this.program.coder.accounts.decode('config', result.account.data);
              if (isConfigAccount(decoded)) {
                config = decoded;
                cacheManager.cacheConfig(this.program.programId, decoded);
              }
            } catch (e) {
              logger.warn(`Config decode error: ${e}`);
            }
            break;
        }
      }

      // Get token balance separately (more complex due to ATA)
      const tokenBalance = await this.getTokenBalance(userPublicKey);

      return {
        userState,
        facility,
        config,
        tokenBalance,
        userInitialized: userState !== null,
        hasFacility: userState?.hasFacility ?? false,
        growPower: safeBNToNumber(facility?.totalGrowPower) || 0,
        pendingReferralRewards: userState ? safeBNToNumber(userState.pendingReferralRewards) : 0,
      };
    } catch (error) {
      logger.error(
        `バッチゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );

      // Fallback to individual fetching
      const userState = await this.fetchUserState(userPublicKey);
      const facility = await this.fetchFarmSpace(userPublicKey);
      const config = await this.fetchConfig();
      const tokenBalance = await this.getTokenBalance(userPublicKey);

      return {
        userState,
        facility,
        config,
        tokenBalance,
        userInitialized: userState !== null,
        hasFacility: userState?.hasFacility ?? false,
        growPower: safeBNToNumber(facility?.totalGrowPower) || 0,
        pendingReferralRewards: userState ? safeBNToNumber(userState.pendingReferralRewards) : 0,
      };
    }
  }

  // Cache invalidation for transactions
  invalidateUserCache(userPublicKey: PublicKey): void {
    cacheManager.invalidateUserCache(userPublicKey);
  }

  // Performance monitoring
  async measureFetchPerformance(userPublicKey: PublicKey): Promise<{
    batchTime: number;
    individualTime: number;
    improvement: number;
  }> {
    // Measure batch performance
    const batchStart = performance.now();
    await this.fetchCompleteGameState(userPublicKey);
    const batchTime = performance.now() - batchStart;

    // Clear cache for fair comparison
    cacheManager.invalidateUserCache(userPublicKey);

    // Measure individual performance
    const individualStart = performance.now();
    await Promise.all([
      this.fetchUserState(userPublicKey),
      this.fetchFarmSpace(userPublicKey),
      this.fetchConfig(),
      this.getTokenBalance(userPublicKey),
    ]);
    const individualTime = performance.now() - individualStart;

    const improvement = ((individualTime - batchTime) / individualTime) * 100;

    return {
      batchTime,
      individualTime,
      improvement,
    };
  }

  // Enhanced error handling
  private handleProgramError(error: unknown): ProgramError {
    if (error instanceof Error) {
      // Handle common Anchor/Solana errors
      if (error.message.includes('StructError') || error.message.includes('union of')) {
        return {
          ...error,
          name: 'ProgramError',
          message: 'Account deserialization failed - program may not be initialized or data structure mismatch',
        };
      }
      
      if (error.message.includes('AccountNotFound')) {
        return {
          ...error,
          name: 'ProgramError',
          message: 'Account not found - program may not be deployed or account not initialized',
        };
      }

      return {
        ...error,
        name: 'ProgramError',
        message: error.message,
      };
    }

    if (typeof error === 'object' && error !== null && 'logs' in error) {
      return {
        name: 'ProgramError',
        message: 'Program execution failed',
        logs: (error as { logs: string[] }).logs,
      };
    }

    return {
      name: 'ProgramError',
      message: String(error),
    };
  }
}
