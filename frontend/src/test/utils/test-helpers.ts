// 共通テストヘルパー関数
// テストファイル間でのコード重複を削減

import {
  Connection,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  Transaction,
  VersionedTransaction,
} from '@solana/web3.js';
import { AnchorClient } from '../../anchor-client';
import { logger } from '../../logger';
import type { WalletAdapter } from '../../types/program-types';

export class TestHelpers {
  /**
   * モックウォレットを作成
   */
  static createMockWallet(keypair: Keypair): WalletAdapter {
    return {
      publicKey: keypair.publicKey,
      signTransaction: async <T extends Transaction | VersionedTransaction>(tx: T): Promise<T> => {
        if ('partialSign' in tx) {
          (tx as Transaction).partialSign(keypair);
        }
        return tx;
      },
      signAllTransactions: async <T extends Transaction | VersionedTransaction>(
        txs: T[]
      ): Promise<T[]> => {
        return txs.map((tx) => {
          if ('partialSign' in tx) {
            (tx as Transaction).partialSign(keypair);
          }
          return tx;
        });
      },
    };
  }

  /**
   * アカウントにSOLを送金
   */
  static async fundAccount(
    connection: Connection,
    publicKey: PublicKey,
    solAmount: number
  ): Promise<boolean> {
    try {
      const signature = await connection.requestAirdrop(publicKey, solAmount * LAMPORTS_PER_SOL);
      await connection.confirmTransaction(signature);
      logger.info(`✅ Account funded: ${publicKey.toString()} with ${solAmount} SOL`);
      return true;
    } catch (error) {
      logger.warn(`❌ Failed to fund account ${publicKey.toString()}: ${error}`);
      return false;
    }
  }

  /**
   * テストユーザーを作成（キーペア + AnchorClient）
   */
  static async createTestUser(
    connection: Connection,
    name: string
  ): Promise<{
    keypair: Keypair;
    client: AnchorClient;
    name: string;
  }> {
    const keypair = Keypair.generate();
    const funded = await this.fundAccount(connection, keypair.publicKey, 2);

    if (!funded) {
      throw new Error(`Failed to fund test user: ${name}`);
    }

    const client = new AnchorClient(connection, this.createMockWallet(keypair));

    logger.info(`🧪 Created test user: ${name} - ${keypair.publicKey.toString()}`);
    return { keypair, client, name };
  }

  /**
   * 複数のテストユーザーを作成
   */
  static async createMultipleTestUsers(
    connection: Connection,
    count: number,
    namePrefix: string = 'TestUser'
  ): Promise<
    Array<{
      keypair: Keypair;
      client: AnchorClient;
      name: string;
    }>
  > {
    const users = [];

    for (let i = 0; i < count; i++) {
      const user = await this.createTestUser(connection, `${namePrefix}${i + 1}`);
      users.push(user);
    }

    logger.info(`🧪 Created ${count} test users`);
    return users;
  }

  /**
   * 非同期操作の結果を統計として集計
   */
  static analyzeResults<T>(results: Array<{ success: boolean; result?: T; error?: Error }>): {
    successCount: number;
    failureCount: number;
    successRate: number;
    errors: Error[];
  } {
    const successCount = results.filter((r) => r.success).length;
    const failureCount = results.length - successCount;
    const successRate = successCount / results.length;
    const errors = results.filter((r) => !r.success && r.error).map((r) => r.error as Error);

    return {
      successCount,
      failureCount,
      successRate,
      errors,
    };
  }

  /**
   * パフォーマンス測定のヘルパー
   */
  static async measurePerformance<T>(
    operation: () => Promise<T>,
    name: string
  ): Promise<{ result: T; duration: number }> {
    const startTime = performance.now();
    const result = await operation();
    const duration = performance.now() - startTime;

    logger.info(`⏱️ ${name} completed in ${duration.toFixed(2)}ms`);
    return { result, duration };
  }

  /**
   * テスト失敗時の詳細ログ
   */
  static logTestFailure(testName: string, error: unknown): void {
    logger.error(`❌ Test Failed: ${testName}`);
    if (error instanceof Error) {
      logger.error(`   Error: ${error.message}`);
      if (error.stack) {
        logger.error(`   Stack: ${error.stack.split('\n').slice(0, 3).join('\n')}`);
      }
    } else {
      logger.error(`   Error: ${String(error)}`);
    }
  }

  /**
   * メモリプレッシャーテスト用のヘルパー
   */
  static async simulateMemoryPressure(sizeMB: number): Promise<void> {
    // メモリプレッシャーを作成
    const memoryPressure = new Array((sizeMB * 1024 * 1024) / 8).fill(0);

    // 短時間保持
    await new Promise((resolve) => setTimeout(resolve, 100));

    // メモリを解放
    memoryPressure.length = 0;

    // ガベージコレクションを促す
    if (global.gc) {
      global.gc();
    }
  }

  /**
   * ネットワーク接続テスト
   */
  static async testNetworkConnection(connection: Connection): Promise<boolean> {
    try {
      const slot = await connection.getSlot();
      return slot > 0;
    } catch (error) {
      logger.warn(`Network connection test failed: ${error}`);
      return false;
    }
  }

  /**
   * プログラム ID の検証
   */
  static validateProgramId(expectedId: string, actualId: string): boolean {
    if (expectedId !== actualId) {
      logger.error(`❌ Program ID mismatch: expected ${expectedId}, got ${actualId}`);
      return false;
    }
    return true;
  }

  /**
   * 期待される成功率を検証
   */
  static validateSuccessRate(
    successRate: number,
    expectedMinRate: number,
    operationName: string
  ): void {
    if (successRate < expectedMinRate) {
      throw new Error(
        `${operationName} success rate ${(successRate * 100).toFixed(1)}% ` +
          `is below expected minimum ${(expectedMinRate * 100).toFixed(1)}%`
      );
    }

    logger.success(
      `✅ ${operationName} success rate: ${(successRate * 100).toFixed(1)}% ` +
        `(above ${(expectedMinRate * 100).toFixed(1)}% threshold)`
    );
  }
}
