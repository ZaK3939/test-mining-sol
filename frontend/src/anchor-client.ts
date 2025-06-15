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
  type WalletAdapter,
  type UserStateAccount,
  type FarmSpaceAccount,
  type ConfigAccount,
  type SeedPackAccount,
  type SeedAccount,
  type GlobalStatsAccount,
  type TransactionResult,
  type ProgramError,
  isUserStateAccount,
  isFarmSpaceAccount,
  isConfigAccount,
  safeBNToNumber,
} from './types/program-types';
import idl from './idl/facility_game.json';

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
  inviteCode: {
    fetchNullable(address: PublicKey): Promise<any | null>;
  };
  globalStats: {
    fetchNullable(address: PublicKey): Promise<GlobalStatsAccount | null>;
  };
}

// Program interface for type-safe methods
interface ProgramMethodNamespace {
  initializeConfig(
    baseRate: BN | null,
    halvingInterval: BN | null,
    treasury: PublicKey
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
  purchaseSeedPack(quantity: number): {
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

  constructor(connection: Connection, wallet: WalletAdapter);
  constructor(provider: AnchorProvider);
  constructor(connectionOrProvider: Connection | AnchorProvider, wallet?: WalletAdapter) {
    // Handle both constructor signatures
    if (connectionOrProvider instanceof AnchorProvider) {
      this.provider = connectionOrProvider;
    } else {
      // Create provider from connection and wallet
      this.provider = new AnchorProvider(connectionOrProvider, wallet!, { preflightCommitment: 'confirmed' });
    }

    // Initialize program with IDL - casting required due to Anchor type limitations
    const anchorProgram = new Program(idl as FarmGameIDL, this.provider);
    this.program = {
      programId: anchorProgram.programId,
      coder: anchorProgram.coder,
      account: anchorProgram.account as ProgramAccountNamespace,
      methods: anchorProgram.methods as unknown as ProgramMethodNamespace,
    };

    // Initialize batch fetcher for performance optimization
    this.batchFetcher = new BatchFetcher(this.provider.connection);
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
    baseRate: number | null = 100,
    halvingInterval: number | null = 604800, // 7 days
    treasury?: PublicKey,
    protocolReferralAddress?: PublicKey | null
  ): Promise<string> {
    try {
      logger.info('⚙️ 設定を初期化中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Use admin as treasury if not provided
      const treasuryAccount = treasury || userPublicKey;

      // Convert to BN for Anchor serialization
      const baseRateBN = baseRate !== null ? new BN(baseRate) : null;
      const halvingIntervalBN = halvingInterval !== null ? new BN(halvingInterval) : null;

      const tx = await this.program.methods
        .initializeConfig(baseRateBN, halvingIntervalBN, treasuryAccount, protocolReferralAddress || null)
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

      logger.info(`Reward mint: ${pdas.rewardMint.toString()}`);

      const tx = await this.program.methods
        .createRewardMint()
        .accountsPartial({
          rewardMint: pdas.rewardMint,
          mintAuthority: pdas.mintAuthority,
          transferFeeConfigAuthority: pdas.mintAuthority,
          withdrawWithheldAuthority: userPublicKey,
          admin: userPublicKey,
          tokenProgram: new PublicKey('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'),
          systemProgram: SystemProgram.programId,
          rent: new PublicKey('SysvarRent111111111111111111111111111111111'),
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

  // Initialize global stats (admin only)
  async initializeGlobalStats(): Promise<string> {
    try {
      logger.info('📊 グローバル統計を初期化中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      const tx = await this.program.methods
        .initializeGlobalStats()
        .accounts({
          globalStats: pdas.globalStats,
          admin: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`グローバル統計初期化成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`グローバル統計初期化エラー: ${programError.message}`);
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

      // Build transaction accounts - omit optional referrer accounts for now
      const accounts: Record<string, PublicKey> = {
        userState: pdas.userState,
        config: pdas.config,
        globalStats: globalStatsPDA,
        rewardMint: pdas.rewardMint,
        mintAuthority: pdas.mintAuthority,
        userTokenAccount: userTokenAccount,
        user: userPublicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      };
      
      let txBuilder = this.program.methods.claimRewardWithReferralRewards().accountsPartial(accounts);

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

  // Purchase seed pack with Switchboard VRF
  async purchaseSeedPack(quantity: number): Promise<string> {
    try {
      logger.info('📦 Switchboard VRFを使用してシードパックを購入中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

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

      const tx = await this.program.methods
        .purchaseSeedPack(quantity)
        .accounts({
          userState: pdas.userState,
          config: pdas.config,
          seedPack: seedPackPDA,
          rewardMint: pdas.rewardMint,
          userTokenAccount: userTokenAccount,
          user: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`シードパック購入成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(
        `シードパック購入エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Open seed pack with Switchboard VRF
  async openSeedPack(packId: number, quantity: number): Promise<string> {
    try {
      logger.info('📦 Switchboard VRFを使用してシードパックを開封中...');

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

      const tx = await this.program.methods
        .openSeedPack(quantity)
        .accounts({
          seedPack: seedPackPDA,
          config: pdas.config,
          seedStorage: seedStoragePDA,
          user: userPublicKey,
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

  // Fetch seed pack (replaces mystery box)
  async fetchSeedPack(
    userPublicKey: PublicKey,
    seedPackId: number
  ): Promise<SeedPackAccount | null> {
    try {
      const [seedPackPDA] = await PublicKey.findProgramAddress(
        [
          Buffer.from('seed_pack'),
          userPublicKey.toBuffer(),
          new BN(seedPackId).toArrayLike(Buffer, 'le', 8),
        ],
        this.program.programId
      );
      const seedPack = await this.program.account.seedPack.fetchNullable(seedPackPDA);
      return seedPack;
    } catch (error) {
      logger.error(
        `シードパック取得エラー: ${error instanceof Error ? error.message : String(error)}`
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
    farmSpace: FarmSpaceAccount | null;
    config: ConfigAccount | null;
    tokenBalance: number;
    userInitialized: boolean;
    hasFarmSpace: boolean;
    growPower: number;
    pendingReferralRewards: number;
  }> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);

      // Create batch request for all game state accounts
      const batchRequests = BatchFetcher.createGameStateBatch(userPublicKey, {
        userState: pdas.userState,
        farmSpace: pdas.farmSpace,
        config: pdas.config,
        rewardMint: pdas.rewardMint,
      });

      // Fetch all accounts in a single batch
      const results = await this.batchFetcher.fetchMultipleAccounts(batchRequests);

      // Process results with caching
      let userState: UserStateAccount | null = null;
      let farmSpace: FarmSpaceAccount | null = null;
      let config: ConfigAccount | null = null;

      for (const result of results) {
        if (result.account === null) continue;

        switch (result.name) {
          case 'userState':
            try {
              const decoded = this.program.coder.accounts.decode('userState', result.account.data) as unknown;
              if (isUserStateAccount(decoded)) {
                userState = decoded;
                cacheManager.cacheUserState(userPublicKey, decoded);
              }
            } catch (e) {
              logger.warn(`UserState decode error: ${e}`);
            }
            break;

          case 'farmSpace':
            try {
              const decoded = this.program.coder.accounts.decode('farmSpace', result.account.data) as unknown;
              if (isFarmSpaceAccount(decoded)) {
                farmSpace = decoded;
                cacheManager.cacheFarmSpace(userPublicKey, decoded);
              }
            } catch (e) {
              logger.warn(`FarmSpace decode error: ${e}`);
            }
            break;

          case 'config':
            try {
              const decoded = this.program.coder.accounts.decode('config', result.account.data) as unknown;
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
        farmSpace,
        config,
        tokenBalance,
        userInitialized: userState !== null,
        hasFarmSpace: userState?.hasFarmSpace ?? false,
        growPower: safeBNToNumber(farmSpace?.totalGrowPower) || 0,
        pendingReferralRewards: userState ? safeBNToNumber(userState.pendingReferralRewards) : 0,
      };
    } catch (error) {
      logger.error(
        `バッチゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );

      // Fallback to individual fetching
      const userState = await this.fetchUserState(userPublicKey);
      const farmSpace = await this.fetchFarmSpace(userPublicKey);
      const config = await this.fetchConfig();
      const tokenBalance = await this.getTokenBalance(userPublicKey);

      return {
        userState,
        farmSpace,
        config,
        tokenBalance,
        userInitialized: userState !== null,
        hasFarmSpace: userState?.hasFarmSpace ?? false,
        growPower: safeBNToNumber(farmSpace?.totalGrowPower) || 0,
        pendingReferralRewards: userState ? safeBNToNumber(userState.pendingReferralRewards) : 0,
      };
    }
  }

  // Cache invalidation for transactions
  invalidateUserCache(userPublicKey: PublicKey): void {
    cacheManager.invalidateUserCache(userPublicKey);
  }

  // Fetch global statistics for network share calculation
  async fetchGlobalStats(): Promise<{
    totalGrowPower: number;
    totalFarmSpaces: number;
    totalSupply: number;
    currentRewardsPerSecond: number;
    lastUpdateTime: number;
  } | null> {
    try {
      const [globalStatsPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('global_stats')],
        this.program.programId
      );

      const globalStats = await this.program.account.globalStats?.fetchNullable?.(globalStatsPDA);
      if (!globalStats) {
        logger.warn('Global stats account not found - may not be initialized yet');
        return null;
      }

      return {
        totalGrowPower: safeBNToNumber(globalStats.totalGrowPower),
        totalFarmSpaces: safeBNToNumber(globalStats.totalFarmSpaces),
        totalSupply: safeBNToNumber(globalStats.totalSupply),
        currentRewardsPerSecond: safeBNToNumber(globalStats.currentRewardsPerSecond),
        lastUpdateTime: safeBNToNumber(globalStats.lastUpdateTime),
      };
    } catch (error) {
      logger.error(`グローバル統計取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
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

  // Get program ID
  getProgramId(): PublicKey {
    return this.program.programId;
  }

  // ===== ADMIN FUNCTIONS =====

  /**
   * Update seed pack cost (admin only)
   * Allows dynamic adjustment of WEED price for seed packs
   * 
   * @param newSeedPackCost - New cost in WEED tokens (with 6 decimals)
   * @returns Transaction result
   * 
   * @example
   * // Set seed pack cost to 500 WEED
   * await client.updateSeedPackCost(500_000_000);
   */
  async updateSeedPackCost(newSeedPackCost: number): Promise<TransactionResult> {
    try {
      logger.info('Updating seed pack cost', { newSeedPackCost });
      
      // Validation
      if (newSeedPackCost <= 0) {
        throw new Error('Seed pack cost must be greater than 0');
      }
      
      if (newSeedPackCost > 10_000 * 1_000_000) {
        throw new Error('Seed pack cost cannot exceed 10,000 WEED');
      }

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      const tx = await this.program.methods
        .updateSeedPackCost(new BN(newSeedPackCost))
        .accounts({
          config: pdas.config,
          admin: userPublicKey,
        })
        .rpc();

      logger.success('Seed pack cost updated successfully', { 
        transaction: tx,
        newCostWeed: newSeedPackCost / 1_000_000
      });

      return tx;

    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error('Failed to update seed pack cost:', programError.message);
      throw programError;
    }
  }

  /**
   * Reveal a new seed type (admin only)
   * Makes a previously hidden seed type visible to users with its values
   * 
   * @param seedIndex - Index of the seed type to reveal (0-15)
   * @param growPower - Grow power value for this seed type
   * @param probabilityPercentage - Probability percentage (0.0-100.0)
   * @returns Transaction result
   * 
   * @example
   * // Reveal Seed9 (index 8) with 60000 grow power and 1.5% probability
   * await client.revealSeed(8, 60000, 1.5);
   */
  async revealSeed(seedIndex: number, growPower: number, probabilityPercentage: number): Promise<TransactionResult> {
    try {
      logger.info('Revealing seed type', { seedIndex, growPower, probabilityPercentage });
      
      // Validation
      if (seedIndex < 0 || seedIndex >= 16) {
        throw new Error('Seed index must be between 0 and 15');
      }
      
      if (growPower <= 0) {
        throw new Error('Grow power must be greater than 0');
      }
      
      if (probabilityPercentage < 0 || probabilityPercentage > 100) {
        throw new Error('Probability percentage must be between 0 and 100');
      }

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Calculate probability table PDA
      const [probabilityTablePDA] = await PublicKey.findProgramAddress(
        [Buffer.from('probability_table')],
        this.program.programId
      );

      const tx = await this.program.methods
        .revealSeed(seedIndex, new BN(growPower), probabilityPercentage)
        .accounts({
          probabilityTable: probabilityTablePDA,
          config: pdas.config,
          admin: userPublicKey,
        })
        .rpc();

      logger.success('Seed type revealed successfully', { 
        transaction: tx,
        seedType: `Seed${seedIndex + 1}`,
        growPower,
        probabilityPercentage
      });

      return tx;

    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error('Failed to reveal seed type:', programError.message);
      throw programError;
    }
  }

  /**
   * Update values for an already revealed seed type (admin only)
   * Allows changing grow power and probability for existing revealed seeds
   * 
   * @param seedIndex - Index of the seed type to update (0-15)
   * @param growPower - New grow power value for this seed type
   * @param probabilityPercentage - New probability percentage (0.0-100.0)
   * @returns Transaction result
   * 
   * @example
   * // Update Seed1 (index 0) to have 150 grow power and 30% probability
   * await client.updateSeedValues(0, 150, 30.0);
   */
  async updateSeedValues(seedIndex: number, growPower: number, probabilityPercentage: number): Promise<TransactionResult> {
    try {
      logger.info('Updating seed values', { seedIndex, growPower, probabilityPercentage });
      
      // Validation
      if (seedIndex < 0 || seedIndex >= 16) {
        throw new Error('Seed index must be between 0 and 15');
      }
      
      if (growPower <= 0) {
        throw new Error('Grow power must be greater than 0');
      }
      
      if (probabilityPercentage < 0 || probabilityPercentage > 100) {
        throw new Error('Probability percentage must be between 0 and 100');
      }

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Calculate probability table PDA
      const [probabilityTablePDA] = await PublicKey.findProgramAddress(
        [Buffer.from('probability_table')],
        this.program.programId
      );

      const tx = await this.program.methods
        .updateSeedValues(seedIndex, new BN(growPower), probabilityPercentage)
        .accounts({
          probabilityTable: probabilityTablePDA,
          config: pdas.config,
          admin: userPublicKey,
        })
        .rpc();

      logger.success('Seed values updated successfully', { 
        transaction: tx,
        seedType: `Seed${seedIndex + 1}`,
        growPower,
        probabilityPercentage
      });

      return tx;

    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error('Failed to update seed values:', programError.message);
      throw programError;
    }
  }
}
