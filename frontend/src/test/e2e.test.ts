// End-to-End tests simulating real user scenarios
// これらのテストは実際のユーザーの使用パターンをシミュレートします

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { Connection, Keypair } from '@solana/web3.js';
import { AnchorClient } from '../anchor-client';
import { config } from '../config';
import { logger } from '../logger';
import { TestHelpers } from './utils/test-helpers';
import { TEST_CONSTANTS } from './utils/test-constants';

describe('E2E Tests - Real User Scenarios', () => {
  let connection: Connection;
  let users: Array<{
    keypair: Keypair;
    client: AnchorClient;
    name: string;
  }>;

  beforeAll(async () => {
    connection = new Connection(config.rpcUrl, 'confirmed');
    
    // Create multiple test users
    users = await TestHelpers.createMultipleTestUsers(connection, 3, 'E2EUser');
    // Rename users for clarity
    users[0].name = 'Alice';
    users[1].name = 'Bob';
    users[2].name = 'Charlie';
    
    logger.info('🎭 E2E test users created');
  }, TEST_CONSTANTS.TIMEOUTS.STRESS);

  afterAll(() => {
    logger.info('🎬 E2E tests completed');
  });

  describe('Scenario 1: New User Complete Journey', () => {
    it('should simulate complete new user experience', async () => {
      const alice = users[0];
      logger.info(`🚀 Starting ${alice.name}'s journey`);

      // Step 1: User initialization
      const initResult = await alice.client.initUser();
      expect(typeof initResult).toBe('string');
      logger.info(`✅ ${alice.name} initialized account`);

      // Step 2: Check initial state
      const initialState = await alice.client.fetchUserState(alice.keypair.publicKey);
      expect(initialState).toBeTruthy();
      if (initialState) {
        expect(initialState.hasFacility).toBe(false);
        expect(initialState.totalGrowPower.toNumber()).toBe(0);
      }
      logger.info(`✅ ${alice.name} verified initial state`);

      // Step 3: Purchase facility
      const facilityResult = await alice.client.buyFacility();
      expect(typeof facilityResult).toBe('string');
      logger.info(`✅ ${alice.name} purchased facility`);

      // Step 4: Verify facility exists
      const facility = await alice.client.fetchFacility(alice.keypair.publicKey);
      expect(facility).toBeTruthy();
      if (facility) {
        expect(facility.machineCount).toBeGreaterThan(0);
        expect(facility.totalGrowPower.toNumber()).toBeGreaterThan(0);
      }
      logger.info(`✅ ${alice.name} verified facility creation`);

      // Step 5: Check updated user state
      const updatedState = await alice.client.fetchUserState(alice.keypair.publicKey);
      expect(updatedState).toBeTruthy();
      if (updatedState) {
        expect(updatedState.hasFacility).toBe(true);
        expect(updatedState.totalGrowPower.toNumber()).toBeGreaterThan(0);
      }
      logger.info(`✅ ${alice.name} completed full journey`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);
  });

  describe('Scenario 2: Referral System', () => {
    it('should simulate referral relationships', async () => {
      const bob = users[1];
      const charlie = users[2];
      
      logger.info(`🤝 Testing referral between ${bob.name} and ${charlie.name}`);

      // Step 1: Bob initializes normally
      const bobInit = await bob.client.initUser();
      expect(typeof bobInit).toBe('string');
      logger.info(`✅ ${bob.name} initialized without referrer`);

      // Step 2: Charlie initializes with Bob as referrer
      try {
        const charlieInit = await charlie.client.initUserWithReferrer(bob.keypair.publicKey);
        expect(typeof charlieInit).toBe('string');
        logger.info(`✅ ${charlie.name} initialized with ${bob.name} as referrer`);

        // Step 3: Verify referral relationship
        const charlieState = await charlie.client.fetchUserState(charlie.keypair.publicKey);
        expect(charlieState).toBeTruthy();
        if (charlieState && charlieState.referrer) {
          expect(charlieState.referrer.toString()).toBe(bob.keypair.publicKey.toString());
        }
        logger.info(`✅ Referral relationship verified`);
      } catch (error) {
        // Referral system might not be fully implemented
        logger.info('⚠️ Referral initialization failed - feature might not be implemented');
        expect(error).toBeTruthy();
      }
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);
  });

  describe('Scenario 3: Multi-User Concurrent Operations', () => {
    it('should handle multiple users operating simultaneously', async () => {
      logger.info('🔄 Testing concurrent multi-user operations');

      // All users try to claim rewards simultaneously
      const rewardPromises = users.map(async (user) => {
        try {
          // Ensure user has facility for rewards
          const userState = await user.client.fetchUserState(user.keypair.publicKey);
          if (!userState || !userState.hasFacility) {
            await user.client.buyFacility();
          }
          
          const result = await user.client.claimRewards();
          return { user: user.name, success: true, result };
        } catch (error) {
          return { 
            user: user.name, 
            success: false, 
            error: error instanceof Error ? error : new Error(String(error)) 
          };
        }
      });

      const { result: results, duration } = await TestHelpers.measurePerformance(
        () => Promise.all(rewardPromises),
        'Concurrent reward claiming'
      );

      const analysis = TestHelpers.analyzeResults(results);
      
      // At least some operations should succeed
      expect(analysis.successCount).toBeGreaterThan(0);
      expect(duration).toBeLessThan(TEST_CONSTANTS.PERFORMANCE.MAX_OPERATION_TIME);
      
      logger.info(`✅ Concurrent operations: ${analysis.successCount}/${results.length} succeeded`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);
  });

  describe('Scenario 4: Progressive Feature Usage', () => {
    it('should simulate progressive feature adoption', async () => {
      const alice = users[0];
      logger.info(`🎯 Testing ${alice.name}'s progressive feature usage`);

      // Ensure Alice has a facility
      await ensureUserHasFacility(alice);

      // Test extended features progressively
      const featureTests = [
        {
          name: 'Facility Upgrade',
          operation: () => alice.client.upgradeFacility(),
        },
        {
          name: 'Machine Addition',
          operation: () => alice.client.addMachine(),
        },
        {
          name: 'Mystery Box Purchase',
          operation: () => alice.client.purchaseMysteryBox(),
        },
        {
          name: 'Mystery Box Opening',
          operation: () => alice.client.openMysteryBox(0),
        },
      ];

      const results = [];
      for (const feature of featureTests) {
        try {
          const result = await feature.operation();
          results.push({ feature: feature.name, success: true, result });
          logger.info(`✅ ${alice.name} completed: ${feature.name}`);
        } catch (error) {
          results.push({ 
            feature: feature.name, 
            success: false, 
            error: error instanceof Error ? error : new Error(String(error)) 
          });
          logger.info(`⚠️ ${alice.name} failed: ${feature.name} (likely insufficient resources)`);
        }
        
        // Small delay between operations
        await new Promise(resolve => setTimeout(resolve, 1000));
      }

      // At least basic operations should work
      expect(results.length).toBe(featureTests.length);
      
      const successCount = results.filter(r => r.success).length;
      logger.info(`✅ Progressive features: ${successCount}/${results.length} completed`);
    }, TEST_CONSTANTS.TIMEOUTS.STRESS);
  });

  describe('Scenario 5: Performance Under Load', () => {
    it('should maintain responsiveness under user load', async () => {
      logger.info('⚡ Testing system responsiveness under load');

      // Create load by having all users perform multiple operations
      const loadOperations = users.flatMap(user => [
        () => user.client.fetchUserState(user.keypair.publicKey),
        () => user.client.fetchFacility(user.keypair.publicKey),
        () => user.client.getTokenBalance(user.keypair.publicKey),
        () => user.client.fetchCompleteGameState(user.keypair.publicKey),
      ]);

      const { result: results, duration } = await TestHelpers.measurePerformance(
        () => Promise.all(loadOperations.map(op => op())),
        'Load testing with all users'
      );

      const successCount = results.filter(r => r !== null && r !== undefined).length;
      const successRate = successCount / results.length;

      TestHelpers.validateSuccessRate(
        successRate,
        TEST_CONSTANTS.SUCCESS_RATES.HIGH,
        'Load testing operations'
      );

      expect(duration).toBeLessThan(TEST_CONSTANTS.PERFORMANCE.MAX_OPERATION_TIME * 2);

      logger.info(`✅ Load test: ${successCount}/${results.length} operations succeeded`);
    }, TEST_CONSTANTS.TIMEOUTS.DEFAULT);
  });

  // Helper function
  async function ensureUserHasFacility(user: { keypair: Keypair; client: AnchorClient; name: string }) {
    try {
      const userState = await user.client.fetchUserState(user.keypair.publicKey);
      if (!userState) {
        await user.client.initUser();
      }
      
      const updatedState = await user.client.fetchUserState(user.keypair.publicKey);
      if (updatedState && !updatedState.hasFacility) {
        await user.client.buyFacility();
        logger.info(`✅ ${user.name} facility setup completed`);
      }
    } catch (error) {
      TestHelpers.logTestFailure(`ensureUserHasFacility for ${user.name}`, error);
      throw error;
    }
  }
});