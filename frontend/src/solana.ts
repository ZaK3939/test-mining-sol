// Solana接続とプログラム相互作用

import {
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  Transaction,
  VersionedTransaction,
} from '@solana/web3.js';
// SPL Token imports removed - using anchor client for token operations
import { AnchorProvider, setProvider } from '@coral-xyz/anchor';
import { Buffer } from 'buffer';
import { config } from './config';
import { logger } from './logger';
import { AnchorClient } from './anchor-client';
import { PDAHelper } from './utils/pda-helper';
import { NETWORK_CONSTANTS } from './utils/constants';
import type { WalletState, GameState, DetailedGameState, NetworkInfo } from './types';

// Bufferをグローバルに設定
if (typeof window !== 'undefined') {
  (window as unknown as { Buffer: typeof Buffer }).Buffer = Buffer;
}

// グローバル変数でPhantomウォレットの型を拡張
declare global {
  interface Window {
    solana?: {
      isPhantom?: boolean;
      connect(): Promise<{ publicKey: PublicKey }>;
      disconnect(): Promise<void>;
      publicKey: PublicKey | null;
      isConnected: boolean;
      signTransaction: (transaction: unknown) => Promise<unknown>;
      signAllTransactions: (transactions: unknown[]) => Promise<unknown[]>;
    };
  }
}

export class SolanaService {
  private connection: Connection;
  private wallet: WalletState;
  private anchorClient: AnchorClient | null = null;

  constructor() {
    this.connection = new Connection(config.rpcUrl, 'confirmed');
    this.wallet = {
      connected: false,
      publicKey: null,
      balance: 0,
    };
  }

  // RPC接続テスト
  async testConnection(): Promise<boolean> {
    try {
      logger.info('🔗 RPC接続テスト開始...');

      const slot = await this.connection.getSlot();
      logger.success(`現在のスロット: ${slot}`);

      const blockTime = await this.connection.getBlockTime(slot);
      if (blockTime) {
        const date = new Date(blockTime * 1000);
        logger.info(`ブロック時刻: ${date.toLocaleString()}`);
      }

      logger.success('RPC接続テスト成功');
      return true;
    } catch (error) {
      logger.error(`RPC接続テスト失敗: ${error instanceof Error ? error.message : String(error)}`);
      return false;
    }
  }

  // ウォレット接続
  async connectWallet(): Promise<WalletState> {
    try {
      logger.info('👛 ウォレット接続を試行中...');

      if (!window.solana || !window.solana.isPhantom) {
        throw new Error(
          'Phantom ウォレットが見つかりません。https://phantom.app からインストールしてください。'
        );
      }

      const response = await window.solana.connect();
      const publicKey = response.publicKey;

      logger.success(`ウォレット接続成功: ${publicKey.toString()}`);

      // 残高取得
      const balance = await this.connection.getBalance(publicKey);
      const solBalance = balance / LAMPORTS_PER_SOL;

      this.wallet = {
        connected: true,
        publicKey,
        balance: solBalance,
      };

      logger.info(`ウォレット残高: ${solBalance.toFixed(4)} SOL`);

      // Anchorクライアントを初期化
      await this.initializeAnchorClient();

      return this.wallet;
    } catch (error) {
      logger.error(
        `ウォレット接続エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // ウォレット切断
  async disconnectWallet(): Promise<void> {
    try {
      if (window.solana) {
        await window.solana.disconnect();
      }

      this.wallet = {
        connected: false,
        publicKey: null,
        balance: 0,
      };

      // Anchorクライアントをクリア
      this.anchorClient = null;

      logger.info('ウォレットを切断しました');
    } catch (error) {
      logger.error(
        `ウォレット切断エラー: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  // PDA計算（共通ヘルパーを使用）
  async calculatePDAs(userPublicKey: PublicKey) {
    const programId = new PublicKey(config.programId);
    return await PDAHelper.calculatePDAs(userPublicKey, programId);
  }

  // ゲーム状態取得（UI表示用に変換）
  async getGameState(): Promise<GameState> {
    if (!this.wallet.publicKey || !this.anchorClient) {
      throw new Error('ウォレットが接続されていません');
    }

    try {
      logger.info('🎮 ゲーム状態を取得中...');
      const detailedState = await this.anchorClient.fetchCompleteGameState(this.wallet.publicKey);
      return this.convertToUIGameState(detailedState);
    } catch (error) {
      logger.error(
        `ゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // 詳細なゲーム状態を取得（内部処理用）
  async getDetailedGameState(): Promise<DetailedGameState> {
    if (!this.wallet.publicKey || !this.anchorClient) {
      return {
        userState: null,
        facility: null,
        config: null,
        tokenBalance: 0,
        userInitialized: false,
        hasFacility: false,
        growPower: 0,
        pendingReferralRewards: 0,
      };
    }

    return await this.anchorClient.fetchCompleteGameState(this.wallet.publicKey);
  }

  // 内部状態をUI表示用に変換
  private convertToUIGameState(detailedState: DetailedGameState): GameState {
    const gameState: GameState = {
      userInitialized: detailedState.userInitialized,
      hasFacility: detailedState.hasFacility,
      growPower: detailedState.growPower,
      tokenBalance: detailedState.tokenBalance,
      lastHarvestTime: detailedState.userState?.lastHarvestTime.toNumber() || 0,
      pendingReferralRewards: detailedState.pendingReferralRewards,
    };

    // 施設情報を追加
    if (detailedState.facility) {
      gameState.facility = {
        facilitySize: detailedState.facility.facilitySize,
        maxCapacity: detailedState.facility.maxCapacity,
        machineCount: detailedState.facility.machineCount,
        totalGrowPower: detailedState.facility.totalGrowPower.toNumber(),
      };
      logger.success(
        `施設確認: サイズ${detailedState.facility.facilitySize}, マシン${detailedState.facility.machineCount}`
      );
    } else {
      logger.info('施設未所有');
    }

    if (detailedState.userState) {
      logger.success(`ユーザー状態確認: Grow Power ${gameState.growPower}`);
      if (detailedState.userState.referrer) {
        logger.info(`紹介者: ${detailedState.userState.referrer.toString()}`);
      }
    } else {
      logger.info('ユーザー未初期化');
    }

    logger.info(`トークン残高: ${gameState.tokenBalance} WEED`);
    logger.success('ゲーム状態取得完了');
    return gameState;
  }

  // 現在のウォレット状態を取得
  getWalletState(): WalletState {
    return { ...this.wallet };
  }

  // 接続状態を取得
  getConnection(): Connection {
    return this.connection;
  }

  // ネットワーク情報を取得
  getNetworkInfo(): NetworkInfo {
    return {
      network: config.network,
      rpcUrl:
        config.rpcUrl.split('?')[0] + (config.rpcUrl.includes('api-key') ? '?api-key=***' : ''),
      programId: config.programId,
    };
  }

  // Anchorクライアントを初期化
  private async initializeAnchorClient() {
    if (!window.solana || !this.wallet.publicKey) {
      throw new Error('ウォレットが接続されていません');
    }

    try {
      // Phantomウォレット用のプロバイダーを作成
      const provider = new AnchorProvider(
        this.connection,
        {
          publicKey: this.wallet.publicKey,
          signTransaction: async <T extends Transaction | VersionedTransaction>(
            tx: T
          ): Promise<T> => {
            if (!window.solana) throw new Error('Wallet not connected');
            return await window.solana.signTransaction(tx);
          },
          signAllTransactions: async <T extends Transaction | VersionedTransaction>(
            txs: T[]
          ): Promise<T[]> => {
            if (!window.solana) throw new Error('Wallet not connected');
            return await window.solana.signAllTransactions(txs);
          },
        },
        { commitment: 'confirmed' }
      );

      setProvider(provider);
      this.anchorClient = new AnchorClient(this.connection, {
        publicKey: this.wallet.publicKey,
        signTransaction: async <T extends Transaction | VersionedTransaction>(
          tx: T
        ): Promise<T> => {
          if (!window.solana) throw new Error('Wallet not connected');
          return await window.solana.signTransaction(tx);
        },
        signAllTransactions: async <T extends Transaction | VersionedTransaction>(
          txs: T[]
        ): Promise<T[]> => {
          if (!window.solana) throw new Error('Wallet not connected');
          return await window.solana.signAllTransactions(txs);
        },
      });
      logger.success('Anchorクライアントを初期化しました');
    } catch (error) {
      logger.error(
        `Anchorクライアント初期化エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
  }

  // Anchorクライアントを取得
  getAnchorClient(): AnchorClient {
    if (!this.anchorClient) {
      throw new Error('Anchorクライアントが初期化されていません');
    }
    return this.anchorClient;
  }

  // 開発用SOLエアドロップ（devnet・ローカル環境）
  async airdropSol(amount: number = NETWORK_CONSTANTS.DEFAULT_AIRDROP_AMOUNT): Promise<void> {
    if (!this.wallet.publicKey) {
      throw new Error('ウォレットが接続されていません');
    }

    // mainnet以外で実行可能
    if (config.network === 'mainnet-beta') {
      throw new Error('エアドロップはmainnetでは利用できません');
    }

    try {
      logger.info(`💰 ${amount} SOL をエアドロップ中...`);

      // エアドロップ用に公式devnet RPCを使用
      const airdropConnection = new Connection('https://api.devnet.solana.com', 'confirmed');

      const signature = await airdropConnection.requestAirdrop(
        this.wallet.publicKey,
        amount * LAMPORTS_PER_SOL
      );

      await this.connection.confirmTransaction(signature, 'confirmed');

      // 残高を更新
      const balance = await this.connection.getBalance(this.wallet.publicKey);
      this.wallet.balance = balance / LAMPORTS_PER_SOL;

      logger.success(`エアドロップ完了: ${amount} SOL`);
      logger.info(`更新された残高: ${this.wallet.balance.toFixed(4)} SOL`);
    } catch (error) {
      logger.error(`エアドロップエラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }
}
