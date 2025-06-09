// Batch fetcher for optimized multiple account requests

import { Connection, PublicKey, AccountInfo } from '@solana/web3.js';
import { logger } from '../logger';

interface BatchRequest {
  address: PublicKey;
  name: string;
}

interface BatchResult {
  address: PublicKey;
  name: string;
  account: AccountInfo<Buffer> | null;
}

export class BatchFetcher {
  private connection: Connection;
  private readonly MAX_BATCH_SIZE = 100; // Solana RPC batch limit

  constructor(connection: Connection) {
    this.connection = connection;
  }

  /**
   * Fetch multiple accounts in batches for better performance
   * @param requests Array of batch requests
   * @returns Promise resolving to batch results
   */
  async fetchMultipleAccounts(requests: BatchRequest[]): Promise<BatchResult[]> {
    if (requests.length === 0) {
      return [];
    }

    try {
      const results: BatchResult[] = [];
      
      // Split requests into batches
      for (let i = 0; i < requests.length; i += this.MAX_BATCH_SIZE) {
        const batch = requests.slice(i, i + this.MAX_BATCH_SIZE);
        const batchResults = await this.processBatch(batch);
        results.push(...batchResults);
      }

      logger.info(`✅ バッチフェッチ完了: ${requests.length}件のアカウント`);
      return results;
    } catch (error) {
      logger.error(`❌ バッチフェッチエラー: ${error instanceof Error ? error.message : String(error)}`);
      throw error;
    }
  }

  /**
   * Process a single batch of requests
   */
  private async processBatch(batch: BatchRequest[]): Promise<BatchResult[]> {
    const addresses = batch.map(req => req.address);
    
    try {
      const accounts = await this.connection.getMultipleAccountsInfo(addresses);
      
      return batch.map((request, index) => ({
        address: request.address,
        name: request.name,
        account: accounts[index],
      }));
    } catch (error) {
      logger.warn(`⚠️ バッチ処理エラー、個別取得にフォールバック: ${error instanceof Error ? error.message : String(error)}`);
      
      // Fallback to individual requests if batch fails
      return await this.fallbackToIndividualRequests(batch);
    }
  }

  /**
   * Fallback to individual requests if batch fails
   */
  private async fallbackToIndividualRequests(batch: BatchRequest[]): Promise<BatchResult[]> {
    const results: BatchResult[] = [];
    
    for (const request of batch) {
      try {
        const account = await this.connection.getAccountInfo(request.address);
        results.push({
          address: request.address,
          name: request.name,
          account,
        });
      } catch (error) {
        logger.warn(`⚠️ 個別アカウント取得失敗 ${request.name}: ${error instanceof Error ? error.message : String(error)}`);
        results.push({
          address: request.address,
          name: request.name,
          account: null,
        });
      }
    }
    
    return results;
  }

  /**
   * Utility method to create batch requests for common use cases
   */
  static createGameStateBatch(_userPublicKey: PublicKey, pdas: {
    userState: PublicKey;
    facility: PublicKey;
    config: PublicKey;
    rewardMint: PublicKey;
  }): BatchRequest[] {
    return [
      { address: pdas.userState, name: 'userState' },
      { address: pdas.facility, name: 'facility' },
      { address: pdas.config, name: 'config' },
      { address: pdas.rewardMint, name: 'rewardMint' },
    ];
  }

  /**
   * Utility method to create batch requests for multiple users
   */
  static createMultiUserBatch(users: Array<{
    publicKey: PublicKey;
    userStatePDA: PublicKey;
    facilityPDA: PublicKey;
  }>): BatchRequest[] {
    const requests: BatchRequest[] = [];
    
    users.forEach((user, index) => {
      requests.push(
        { address: user.userStatePDA, name: `userState_${index}` },
        { address: user.facilityPDA, name: `facility_${index}` }
      );
    });
    
    return requests;
  }

  /**
   * Enhanced method with retry logic
   */
  async fetchWithRetry(
    requests: BatchRequest[],
    maxRetries: number = 3,
    retryDelay: number = 1000
  ): Promise<BatchResult[]> {
    let lastError: Error | null = null;
    
    for (let attempt = 0; attempt < maxRetries; attempt++) {
      try {
        return await this.fetchMultipleAccounts(requests);
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        
        if (attempt < maxRetries - 1) {
          logger.warn(`🔄 バッチフェッチリトライ ${attempt + 1}/${maxRetries}: ${lastError.message}`);
          await new Promise(resolve => setTimeout(resolve, retryDelay * (attempt + 1)));
        }
      }
    }
    
    throw lastError || new Error('バッチフェッチが最大リトライ回数に達しました');
  }

  /**
   * Get performance metrics for monitoring
   */
  async measureBatchPerformance(requests: BatchRequest[]): Promise<{
    duration: number;
    requestCount: number;
    successCount: number;
    failureCount: number;
    averageTimePerRequest: number;
  }> {
    const startTime = performance.now();
    
    try {
      const results = await this.fetchMultipleAccounts(requests);
      const endTime = performance.now();
      const duration = endTime - startTime;
      
      const successCount = results.filter(r => r.account !== null).length;
      const failureCount = results.length - successCount;
      
      return {
        duration,
        requestCount: requests.length,
        successCount,
        failureCount,
        averageTimePerRequest: duration / requests.length,
      };
    } catch (error) {
      const endTime = performance.now();
      throw {
        error,
        duration: endTime - startTime,
        requestCount: requests.length,
        successCount: 0,
        failureCount: requests.length,
      };
    }
  }
}