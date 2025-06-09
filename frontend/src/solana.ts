// Solana接続とプログラム相互作用

import {
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  Transaction,
  VersionedTransaction,
} from '@solana/web3.js';
import {
  getAssociatedTokenAddress,
  getAccount,
  TokenAccountNotFoundError,
  TokenInvalidAccountOwnerError,
} from '@solana/spl-token';
import { AnchorProvider, setProvider } from '@coral-xyz/anchor';
import { Buffer } from 'buffer';
import { config } from './config';
import { logger } from './logger';
import { AnchorClient } from './anchor-client';
import { PDAHelper } from './utils/pda-helper';
import { GAME_CONSTANTS, NETWORK_CONSTANTS } from './utils/constants';
import type { WalletState, GameState } from './types';

// Bufferをグローバルに設定
if (typeof window !== 'undefined') {
  (window as any).Buffer = Buffer;
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
      signTransaction: (transaction: any) => Promise<any>;
      signAllTransactions: (transactions: any[]) => Promise<any[]>;
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

  // ゲーム状態取得
  async getGameState(): Promise<GameState> {
    if (!this.wallet.publicKey) {
      throw new Error('ウォレットが接続されていません');
    }

    try {
      logger.info('🎮 ゲーム状態を取得中...');

      const pdas = await this.calculatePDAs(this.wallet.publicKey);

      // アカウント情報取得
      const [userStateAccount, facilityAccount] = await Promise.all([
        this.connection.getAccountInfo(pdas.userState),
        this.connection.getAccountInfo(pdas.facility),
      ]);

      const gameState: GameState = {
        userInitialized: !!userStateAccount,
        hasFacility: !!facilityAccount,
        growPower: 0,
        tokenBalance: 0,
        lastHarvestTime: 0,
      };

      if (userStateAccount) {
        logger.success(`ユーザー状態確認: ${pdas.userState.toString()}`);
      } else {
        logger.info('ユーザー未初期化');
      }

      if (facilityAccount) {
        logger.success(`施設確認: ${pdas.facility.toString()}`);
        // 簡易的にGrow Powerを初期値に設定（実際はアカウントデータを解析）
        gameState.growPower = GAME_CONSTANTS.INITIAL_GROW_POWER;
      } else {
        logger.info('施設未所有');
      }

      // トークン残高確認
      try {
        const userTokenAccount = await getAssociatedTokenAddress(
          pdas.rewardMint,
          this.wallet.publicKey
        );

        try {
          const tokenAccount = await getAccount(this.connection, userTokenAccount);
          const balance = Number(tokenAccount.amount);
          gameState.tokenBalance = balance;
          logger.info(`トークンアカウント確認: ${userTokenAccount.toString()}, 残高: ${balance}`);
        } catch (error) {
          if (error instanceof TokenAccountNotFoundError) {
            logger.info('トークンアカウントが見つかりません（報酬請求時に自動作成されます）');
            gameState.tokenBalance = 0;
          } else if (error instanceof TokenInvalidAccountOwnerError) {
            logger.warn('トークンアカウントの所有者が無効です');
            gameState.tokenBalance = 0;
          } else {
            throw error;
          }
        }
      } catch (error) {
        logger.warn(
          `トークン残高確認エラー: ${error instanceof Error ? error.message : String(error)}`
        );
        gameState.tokenBalance = 0;
      }

      logger.success('ゲーム状態取得完了');
      return gameState;
    } catch (error) {
      logger.error(
        `ゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      throw error;
    }
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
  getNetworkInfo() {
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

  // 開発用SOLエアドロップ（ローカル環境のみ）
  async airdropSol(amount: number = NETWORK_CONSTANTS.DEFAULT_AIRDROP_AMOUNT): Promise<void> {
    if (!this.wallet.publicKey) {
      throw new Error('ウォレットが接続されていません');
    }

    // ローカル環境でのみ実行
    if (!config.rpcUrl.includes('localhost') && !config.rpcUrl.includes('127.0.0.1')) {
      throw new Error('エアドロップはローカル環境でのみ利用可能です');
    }

    try {
      logger.info(`💰 ${amount} SOL をエアドロップ中...`);

      const signature = await this.connection.requestAirdrop(
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
