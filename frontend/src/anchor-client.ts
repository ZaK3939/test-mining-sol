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
  type FacilityAccount,
  type ConfigAccount,
  type MysteryBoxAccount,
  type SeedAccount,
  type TransactionResult,
  type ProgramError,
  isUserStateAccount,
  isFacilityAccount,
  isConfigAccount,
  safeBNToNumber
} from './types/program-types';
import idl from './idl/facility_game.json';

// Type for the Facility Game IDL
type FacilityGameIDL = Idl;

// Program interface for type-safe account access
interface ProgramAccountNamespace {
  userState: {
    fetchNullable(address: PublicKey): Promise<UserStateAccount | null>;
  };
  facility: {
    fetchNullable(address: PublicKey): Promise<FacilityAccount | null>;
  };
  config: {
    fetchNullable(address: PublicKey): Promise<ConfigAccount | null>;
  };
  mysteryBox: {
    fetchNullable(address: PublicKey): Promise<MysteryBoxAccount | null>;
  };
  seed: {
    fetchNullable(address: PublicKey): Promise<SeedAccount | null>;
  };
}

// Program interface for type-safe methods
interface ProgramMethodNamespace {
  initUser(referrer?: PublicKey | null): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  buyFacility(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  claimReward(): {
    accounts(accounts: Record<string, PublicKey>): {
      preInstructions?(instructions: any[]): {
        rpc(): Promise<string>;
      };
      rpc(): Promise<string>;
    };
  };
  distributeReferralReward(amount: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  claimReferralRewards(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  upgradeFacility(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  addMachine(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  transferWithFee(amount: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  purchaseMysteryBox(): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
  openMysteryBox(mysteryBoxId: BN): {
    accounts(accounts: Record<string, PublicKey>): {
      rpc(): Promise<string>;
    };
  };
}

// Enhanced program interface
interface TypedProgram {
  programId: PublicKey;
  coder: any;
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
    const anchorProgram = new Program(idl as FacilityGameIDL, this.provider);
    this.program = {
      programId: anchorProgram.programId,
      coder: anchorProgram.coder,
      account: anchorProgram.account as any,
      methods: anchorProgram.methods as any,
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
      logger.error(
        `ユーザー初期化エラー: ${programError.message}`
      );
      throw programError;
    }
  }

  // Buy facility
  async buyFacility(): Promise<TransactionResult> {
    try {
      logger.info('🏭 施設を購入中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user is initialized with type validation
      const userStateAccount = await this.program.account.userState.fetchNullable(pdas.userState);
      if (!userStateAccount || !isUserStateAccount(userStateAccount)) {
        logger.error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
        throw new Error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
      }

      // Check if user already has facility
      if (userStateAccount.hasFacility) {
        logger.warn(ERROR_MESSAGES.FACILITY_ALREADY_OWNED);
        return 'already_owned';
      }

      // Send buyFacility transaction
      const tx = await this.program.methods
        .buyFacility()
        .accounts({
          userState: pdas.userState,
          facility: pdas.facility,
          user: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`施設購入成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`施設購入エラー: ${programError.message}`);
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
      const userStateAccount = await (this.program.account as any).userState.fetchNullable(pdas.userState);
      if (!userStateAccount) {
        logger.error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
        throw new Error(ERROR_MESSAGES.USER_NOT_INITIALIZED);
      }

      // Check if user has facility
      if (!userStateAccount.hasFacility) {
        logger.error(ERROR_MESSAGES.FACILITY_NOT_OWNED);
        throw new Error(ERROR_MESSAGES.FACILITY_NOT_OWNED);
      }

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
      logger.error(
        `ユーザー状態取得エラー: ${programError.message}`
      );
      return null;
    }
  }

  // Fetch facility with caching and type validation
  async fetchFacility(userPublicKey: PublicKey): Promise<FacilityAccount | null> {
    try {
      // Check cache first
      const cached = cacheManager.getCachedFacility(userPublicKey);
      if (cached) {
        return cached;
      }
      
      const pdas = await this.calculatePDAs(userPublicKey);
      const facility = await this.program.account.facility.fetchNullable(pdas.facility);
      
      if (facility && !isFacilityAccount(facility)) {
        logger.warn('取得した施設状態の型が不正です');
        return null;
      }
      
      // Cache the result
      if (facility) {
        cacheManager.cacheFacility(userPublicKey, facility);
      }
      
      return facility;
    } catch (error) {
      const programError = this.handleProgramError(error);
      logger.error(`施設状態取得エラー: ${programError.message}`);
      return null;
    }
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
      const userStateAccount = await (this.program.account as any).userState.fetchNullable(pdas.userState);
      if (userStateAccount) {
        logger.warn(ERROR_MESSAGES.USER_ALREADY_INITIALIZED);
        return 'already_initialized';
      }

      const accounts: any = {
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

  // Distribute referral rewards
  async distributeReferralReward(amount: number): Promise<string> {
    try {
      logger.info('💰 紹介報酬を配布中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      const tx = await this.program.methods
        .distributeReferralReward(new BN(amount))
        .accounts({
          userState: pdas.userState,
          user: userPublicKey,
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
          userState: pdas.userState,
          config: pdas.config,
          rewardMint: pdas.rewardMint,
          mintAuthority: pdas.mintAuthority,
          userTokenAccount: userTokenAccount,
          user: userPublicKey,
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

  // Upgrade facility
  async upgradeFacility(): Promise<string> {
    try {
      logger.info('🔧 施設をアップグレード中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      const tx = await this.program.methods
        .upgradeFacility()
        .accounts({
          facility: pdas.facility,
          userTokenAccount: userTokenAccount,
          rewardMint: pdas.rewardMint,
          user: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`施設アップグレード成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`施設アップグレードエラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Add machine to facility
  async addMachine(): Promise<string> {
    try {
      logger.info('⚙️ マシンを追加中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      const tx = await this.program.methods
        .addMachine()
        .accounts({
          userState: pdas.userState,
          facility: pdas.facility,
          userTokenAccount: userTokenAccount,
          rewardMint: pdas.rewardMint,
          user: userPublicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`マシン追加成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`マシン追加エラー: ${error instanceof Error ? error.message : String(error)}`);
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
      const recipientTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, recipientPublicKey);
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
      logger.error(`手数料付き転送エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Purchase mystery box
  async purchaseMysteryBox(): Promise<string> {
    try {
      logger.info('📦 ミステリーボックスを購入中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);
      
      // Generate mystery box PDA
      const config = await this.fetchConfig();
      if (!config) throw new Error('Config not found');
      
      const mysteryBoxId = config.mysteryBoxCounter;
      const [mysteryBoxPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('mystery_box'), userPublicKey.toBuffer(), mysteryBoxId.toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );

      const tx = await this.program.methods
        .purchaseMysteryBox()
        .accounts({
          config: pdas.config,
          mysteryBox: mysteryBoxPDA,
          userTokenAccount: userTokenAccount,
          rewardMint: pdas.rewardMint,
          user: userPublicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      logger.success(`ミステリーボックス購入成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`ミステリーボックス購入エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Open mystery box
  async openMysteryBox(mysteryBoxId: number): Promise<string> {
    try {
      logger.info('📦 ミステリーボックスを開封中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);
      
      const [mysteryBoxPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('mystery_box'), userPublicKey.toBuffer(), new BN(mysteryBoxId).toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );
      
      const config = await this.fetchConfig();
      if (!config) throw new Error('Config not found');
      
      const seedId = config.seedCounter;
      const [seedPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('seed'), userPublicKey.toBuffer(), seedId.toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );

      const tx = await this.program.methods
        .openMysteryBox(new BN(mysteryBoxId))
        .accounts({
          config: pdas.config,
          mysteryBox: mysteryBoxPDA,
          seed: seedPDA,
          user: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      logger.success(`ミステリーボックス開封成功! トランザクション: ${tx}`);
      return tx;
    } catch (error) {
      logger.error(`ミステリーボックス開封エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  // Fetch mystery box
  async fetchMysteryBox(userPublicKey: PublicKey, mysteryBoxId: number): Promise<MysteryBoxAccount | null> {
    try {
      const [mysteryBoxPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('mystery_box'), userPublicKey.toBuffer(), new BN(mysteryBoxId).toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );
      const mysteryBox = await (this.program.account as any).mysteryBox.fetchNullable(mysteryBoxPDA);
      return mysteryBox;
    } catch (error) {
      logger.error(`ミステリーボックス取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
  }

  // Fetch seed
  async fetchSeed(userPublicKey: PublicKey, seedId: number): Promise<SeedAccount | null> {
    try {
      const [seedPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('seed'), userPublicKey.toBuffer(), new BN(seedId).toArrayLike(Buffer, 'le', 8)],
        this.program.programId
      );
      const seed = await (this.program.account as any).seed.fetchNullable(seedPDA);
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

      const tokenAccountInfo =
        await this.provider.connection.getTokenAccountBalance(userTokenAccount);
      
      const amount = new BN(tokenAccountInfo.value.amount);
      const decimals = tokenAccountInfo.value.decimals;
      
      // Use safe conversion to avoid overflow
      return safeBNToNumber(amount) / Math.pow(10, decimals);
    } catch (error) {
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
      logger.error(`バッチゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      
      // Fallback to individual fetching
      const userState = await this.fetchUserState(userPublicKey);
      const facility = await this.fetchFacility(userPublicKey);
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
      this.fetchFacility(userPublicKey),
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
