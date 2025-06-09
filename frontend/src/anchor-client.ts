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
import { ERROR_MESSAGES } from './utils/constants';
import idl from './idl/facility_game.json';

// Type for the Facility Game IDL
type FacilityGameIDL = Idl;

// Account types from IDL
export interface UserState {
  owner: PublicKey;
  totalGrowPower: BN;
  lastHarvestTime: BN;
  hasFacility: boolean;
  reserve: number[];
}

export interface Facility {
  owner: PublicKey;
  machineCount: number;
  totalGrowPower: BN;
  reserve: number[];
}

export interface Config {
  baseRate: BN;
  halvingInterval: BN;
  nextHalvingTime: BN;
  admin: PublicKey;
  reserve: number[];
}

export class AnchorClient {
  private program: Program<FacilityGameIDL>;
  private provider: AnchorProvider;

  constructor(connection: Connection, wallet: any) {
    // Create provider
    this.provider = new AnchorProvider(connection, wallet, { preflightCommitment: 'confirmed' });

    // Initialize program with IDL
    this.program = new Program(idl as any, this.provider);
  }

  // Calculate PDAs using shared helper
  async calculatePDAs(userPublicKey: PublicKey): Promise<PDAs> {
    return await PDAHelper.calculatePDAs(userPublicKey, this.program.programId);
  }

  // Initialize user account
  async initUser(): Promise<string> {
    try {
      logger.info('👤 ユーザーアカウントを初期化中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user already initialized
      const userStateAccount = await (this.program.account as any).userState.fetchNullable(pdas.userState);
      if (userStateAccount) {
        logger.warn(ERROR_MESSAGES.USER_ALREADY_INITIALIZED);
        return 'already_initialized';
      }

      // Send initUser transaction
      const tx = await this.program.methods
        .initUser()
        .accounts({
          userState: pdas.userState,
          user: userPublicKey,
          systemProgram: SystemProgram.programId,
        })
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

  // Buy facility
  async buyFacility(): Promise<string> {
    try {
      logger.info('🏭 施設を購入中...');

      const userPublicKey = this.provider.wallet.publicKey;
      const pdas = await this.calculatePDAs(userPublicKey);

      // Check if user is initialized
      const userStateAccount = await (this.program.account as any).userState.fetchNullable(pdas.userState);
      if (!userStateAccount) {
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
      logger.error(`施設購入エラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
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
      if (createATAInstruction) {
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

  // Fetch user state
  async fetchUserState(userPublicKey: PublicKey): Promise<UserState | null> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const userState = await (this.program.account as any).userState.fetchNullable(pdas.userState);
      return userState;
    } catch (error) {
      logger.error(
        `ユーザー状態取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      return null;
    }
  }

  // Fetch facility
  async fetchFacility(userPublicKey: PublicKey): Promise<Facility | null> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const facility = await (this.program.account as any).facility.fetchNullable(pdas.facility);
      return facility;
    } catch (error) {
      logger.error(`施設状態取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
  }

  // Fetch config
  async fetchConfig(): Promise<Config | null> {
    try {
      const [configPDA] = await PublicKey.findProgramAddress(
        [Buffer.from('config')],
        this.program.programId
      );
      const config = await (this.program.account as any).config.fetchNullable(configPDA);
      return config;
    } catch (error) {
      logger.error(`設定取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      return null;
    }
  }

  // Get token balance
  async getTokenBalance(userPublicKey: PublicKey): Promise<number> {
    try {
      const pdas = await this.calculatePDAs(userPublicKey);
      const userTokenAccount = await getAssociatedTokenAddress(pdas.rewardMint, userPublicKey);

      const tokenAccountInfo =
        await this.provider.connection.getTokenAccountBalance(userTokenAccount);
      return (
        parseFloat(tokenAccountInfo.value.amount) / Math.pow(10, tokenAccountInfo.value.decimals)
      );
    } catch (error) {
      // Token account might not exist yet
      return 0;
    }
  }
}
