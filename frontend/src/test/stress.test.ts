// Stress tests for production readiness
// 本番環境での負荷に対応できるかをテストします

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { Connection, Keypair } from '@solana/web3.js';
import { AnchorClient } from '../anchor-client';
import { config } from '../config';
import { logger } from '../logger';
import { TestHelpers } from './utils/test-helpers';
import { TEST_CONSTANTS } from './utils/test-constants';

describe('Stress Tests - Production Readiness', () => {
  let connection: Connection;
  let testUsers: Array<{
    keypair: Keypair;
    client: AnchorClient;
    name: string;
  }>;

  beforeAll(async () => {
    connection = new Connection(config.rpcUrl, 'confirmed');
    
    // Create test users for stress testing
    testUsers = await TestHelpers.createMultipleTestUsers(
      connection, 
      TEST_CONSTANTS.OPERATION_COUNTS.MEDIUM, 
      'StressUser'
    );
    
    logger.info('💪 Stress test environment initialized');
  }, TEST_CONSTANTS.TIMEOUTS.STRESS);

  afterAll(() => {
    // Clean up cache for all test users
    testUsers.forEach(user => {
      user.client.invalidateUserCache(user.keypair.publicKey);
    });
    logger.info('🧹 Stress test cleanup completed');
  });

  describe('Concurrent User Operations', () => {
    it('should handle concurrent user initializations', async () => {
      logger.info('🚀 Testing concurrent user initializations');
      
      const initPromises = testUsers.map(async (user, index) => {
        try {
          const result = await user.client.initUser();
          return { index, success: true, result };
        } catch (error) {
          return { index, success: false, error: error instanceof Error ? error : new Error(String(error)) };
        }
      });
      
      const { result: results, duration } = await TestHelpers.measurePerformance(
        () => Promise.all(initPromises),
        'Concurrent user initializations'
      );
      
      const analysis = TestHelpers.analyzeResults(results);
      
      TestHelpers.validateSuccessRate(
        analysis.successRate, 
        TEST_CONSTANTS.SUCCESS_RATES.MEDIUM, 
        'Concurrent user initializations'
      );
      
      expect(duration).toBeLessThan(TEST_CONSTANTS.TIMEOUTS.STRESS);
    }, TEST_CONSTANTS.TIMEOUTS.STRESS);

    it('should handle concurrent facility purchases', async () => {
      logger.info('🏭 Testing concurrent facility purchases');
      
      const facilityPromises = testUsers.map(async (user, index) => {
        try {
          const result = await user.client.buyFacility();
          return { index, success: true, result };
        } catch (error) {
          return { index, success: false, error: error instanceof Error ? error : new Error(String(error)) };
        }
      });
      
      const { result: results, duration } = await TestHelpers.measurePerformance(
        () => Promise.all(facilityPromises),
        'Concurrent facility purchases'
      );
      
      const analysis = TestHelpers.analyzeResults(results);
      
      // Facility purchases might have lower success rate due to dependencies
      TestHelpers.validateSuccessRate(
        analysis.successRate, 
        TEST_CONSTANTS.SUCCESS_RATES.LOW, 
        'Concurrent facility purchases'
      );
      
      expect(duration).toBeLessThan(TEST_CONSTANTS.TIMEOUTS.STRESS);
    }, TEST_CONSTANTS.TIMEOUTS.STRESS);

    it('should handle rapid sequential API calls', async () => {
      logger.info('⚡ Testing rapid sequential API calls');
      
      const user = testUsers[0];
      const callCount = TEST_CONSTANTS.OPERATION_COUNTS.LARGE;
      
      const operations = Array.from({ length: callCount }, () => 
        user.client.fetchUserState(user.keypair.publicKey)
      );
      
      const { result: results, duration } = await TestHelpers.measurePerformance(
        () => Promise.all(operations),
        `${callCount} rapid API calls`
      );
      
      const successCount = results.filter(r => r !== null).length;
      const averageTime = duration / callCount;
      
      expect(successCount).toBeGreaterThan(callCount * TEST_CONSTANTS.SUCCESS_RATES.HIGH);
      expect(averageTime).toBeLessThan(TEST_CONSTANTS.PERFORMANCE.MAX_OPERATION_TIME / 10);
      
      logger.info(`✅ Rapid API calls: ${successCount}/${callCount} succeeded, avg ${averageTime.toFixed(2)}ms per call`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);
  });

  describe('Memory and Performance Tests', () => {
    it('should maintain performance under memory pressure', async () => {
      logger.info('🧠 Testing performance under memory pressure');
      
      const user = testUsers[0];
      const iterations = TEST_CONSTANTS.OPERATION_COUNTS.LARGE;
      
      // Create memory pressure
      await TestHelpers.simulateMemoryPressure(TEST_CONSTANTS.MEMORY.PRESSURE_SIZE_MB);
      
      // Perform operations under memory pressure
      const operations = Array.from({ length: iterations }, () =>
        user.client.fetchUserState(user.keypair.publicKey)
      );
      
      const { result: results, duration } = await TestHelpers.measurePerformance(
        () => Promise.all(operations),
        `${iterations} operations under memory pressure`
      );
      
      const successCount = results.filter(r => r !== null).length;
      const averageTime = duration / iterations;
      
      TestHelpers.validateSuccessRate(
        successCount / iterations,
        TEST_CONSTANTS.SUCCESS_RATES.HIGH,
        'Operations under memory pressure'
      );
      
      expect(averageTime).toBeLessThan(TEST_CONSTANTS.PERFORMANCE.MAX_OPERATION_TIME / 20);
      
      logger.info(`✅ Memory pressure test: ${successCount}/${iterations} succeeded, avg ${averageTime.toFixed(2)}ms`);
    }, TEST_CONSTANTS.TIMEOUTS.LONG);

    it('should demonstrate cache efficiency under load', async () => {
      logger.info('📦 Testing cache efficiency under load');
      
      const user = testUsers[0];
      const operationsPerRound = TEST_CONSTANTS.OPERATION_COUNTS.MEDIUM;
      
      // Clear cache
      user.client.invalidateUserCache(user.keypair.publicKey);
      
      // First round (cold cache)
      const { duration: coldTime } = await TestHelpers.measurePerformance(
        () => Promise.all(Array.from({ length: operationsPerRound }, () =>
          user.client.fetchUserState(user.keypair.publicKey)
        )),
        'Cold cache operations'
      );
      
      // Second round (warm cache)
      const { duration: warmTime } = await TestHelpers.measurePerformance(
        () => Promise.all(Array.from({ length: operationsPerRound }, () =>
          user.client.fetchUserState(user.keypair.publicKey)
        )),
        'Warm cache operations'
      );
      
      const improvementRatio = (coldTime - warmTime) / coldTime;
      
      expect(improvementRatio).toBeGreaterThan(TEST_CONSTANTS.PERFORMANCE.BATCH_IMPROVEMENT_MIN);
      
      logger.info(`✅ Cache efficiency: ${(improvementRatio * 100).toFixed(1)}% improvement with cache`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);

    it('should handle network instability', async () => {
      logger.info('🌐 Testing network instability resilience');
      
      const user = testUsers[0];
      const retryOperations = TEST_CONSTANTS.OPERATION_COUNTS.SMALL;
      
      const operations = Array.from({ length: retryOperations }, async (_, index) => {
        let attempts = 0;
        const maxAttempts = TEST_CONSTANTS.NETWORK.RETRY_ATTEMPTS;
        
        while (attempts < maxAttempts) {
          try {
            const result = await user.client.fetchUserState(user.keypair.publicKey);
            return { index, success: true, attempts: attempts + 1, result };
          } catch (error) {
            attempts++;
            if (attempts >= maxAttempts) {
              return { index, success: false, attempts, error: error instanceof Error ? error : new Error(String(error)) };
            }
            await new Promise(resolve => setTimeout(resolve, TEST_CONSTANTS.NETWORK.RETRY_DELAY));
          }
        }
        // This should never be reached, but TypeScript requires a return
        return { index, success: false, attempts: maxAttempts, error: new Error('Max attempts reached') };
      });
      
      const { result: results } = await TestHelpers.measurePerformance(
        () => Promise.all(operations),
        'Network retry operations'
      );
      
      const analysis = TestHelpers.analyzeResults(results);
      
      TestHelpers.validateSuccessRate(
        analysis.successRate,
        TEST_CONSTANTS.SUCCESS_RATES.HIGH,
        'Network retry operations'
      );
      
      const avgAttempts = results.reduce((sum, r) => sum + (r?.attempts || 1), 0) / results.length;
      
      logger.info(`✅ Network resilience: ${(analysis.successRate * 100).toFixed(1)}% success, avg ${avgAttempts.toFixed(1)} attempts`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);
  });

  describe('Resource Management', () => {
    it('should efficiently manage batch operations', async () => {
      logger.info('📊 Testing batch operation efficiency');
      
      const user = testUsers[0];
      
      // Test individual vs batch fetching
      const { duration: individualTime } = await TestHelpers.measurePerformance(
        async () => {
          await user.client.fetchUserState(user.keypair.publicKey);
          await user.client.fetchFacility(user.keypair.publicKey);
          await user.client.fetchConfig();
          await user.client.getTokenBalance(user.keypair.publicKey);
        },
        'Individual fetch operations'
      );
      
      const { duration: batchTime } = await TestHelpers.measurePerformance(
        () => user.client.fetchCompleteGameState(user.keypair.publicKey),
        'Batch fetch operation'
      );
      
      const improvement = (individualTime - batchTime) / individualTime;
      
      expect(improvement).toBeGreaterThan(TEST_CONSTANTS.PERFORMANCE.BATCH_IMPROVEMENT_MIN);
      
      logger.info(`✅ Batch efficiency: ${(improvement * 100).toFixed(1)}% faster than individual calls`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);

    it('should handle memory cleanup effectively', async () => {
      logger.info('🗑️ Testing memory cleanup effectiveness');
      
      const user = testUsers[0];
      const largeOperationCount = TEST_CONSTANTS.OPERATION_COUNTS.STRESS;
      
      // Perform large number of operations
      const operations = Array.from({ length: largeOperationCount }, () =>
        user.client.fetchUserState(user.keypair.publicKey)
      );
      
      const { result: results } = await TestHelpers.measurePerformance(
        () => Promise.all(operations),
        `${largeOperationCount} memory stress operations`
      );
      
      // Force cleanup
      user.client.invalidateUserCache(user.keypair.publicKey);
      
      // Perform garbage collection if available
      if (global.gc) {
        global.gc();
      }
      
      // Check that operations still work after cleanup
      const postCleanupResult = await user.client.fetchUserState(user.keypair.publicKey);
      
      expect(results.length).toBe(largeOperationCount);
      expect(postCleanupResult).toBeTruthy();
      
      logger.info(`✅ Memory cleanup: ${largeOperationCount} operations completed, cleanup successful`);
    }, TEST_CONSTANTS.TIMEOUTS.LONG);
  });
});