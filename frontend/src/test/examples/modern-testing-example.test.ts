/**
 * Modern Testing Framework Example
 * Demonstrates comprehensive testing patterns using the new testing utilities
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TestScenarios, type ScenarioContext } from '../utils/test-scenarios';
import { TestDataFactory } from '../factories/test-data-factory';
import { 
  CallbackAssertions,
  ServiceAssertions,
  GameStateAssertions,
  TransactionAssertions,
  ErrorAssertions,
  TimingAssertions,
  TestRunner
} from '../utils/test-assertions';

describe('Modern Testing Framework Example', () => {
  let context: ScenarioContext;

  beforeEach(() => {
    // Reset all test data between tests
    TestDataFactory.resetCounters();
  });

  describe('User Journey Testing', () => {
    it('should handle complete new user onboarding flow', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createNewUserScenario();
      const { services, currentUser, callbacks } = context;

      // =============== TEST EXECUTION ===============
      await TestRunner.runComprehensiveTest(
        'New User Onboarding',
        async () => {
          // Step 1: Connect wallet
          await services.solanaService.connect();
          services.walletService.simulateConnection(currentUser.publicKey, 2.5);

          // Step 2: Initialize user
          const userInitResult = await services.gameService.initializeUser(callbacks);

          // Step 3: Purchase farm space
          const farmPurchaseResult = await services.gameService.purchaseFarmSpace(callbacks);

          return { userInitResult, farmPurchaseResult };
        },
        {
          callbacks,
          services,
          transaction: true,
          timing: { maxMs: 1000 },
          performance: { maxMs: 500 },
        }
      );

      // =============== DETAILED ASSERTIONS ===============
      
      // Wallet connection assertions
      ServiceAssertions.expectWalletConnection(services);
      expect(services.walletService.getWalletState().isConnected).toBe(true);

      // User initialization assertions
      ServiceAssertions.expectTransaction(services, 'user_init');
      CallbackAssertions.expectSuccessCallback(callbacks, /ユーザーアカウントが初期化されました/);

      // Farm purchase assertions
      ServiceAssertions.expectTransaction(services, 'farm_purchase');
      CallbackAssertions.expectSuccessCallback(callbacks, /農場スペースを購入しました/);

      // Game state updates
      CallbackAssertions.expectGameStateUpdate(callbacks);
      expect(callbacks.updateGameState).toHaveBeenCalledTimes(2); // Once for each transaction
    });

    it('should handle active player reward claiming', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createActivePlayerScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const claimResult = await services.gameService.claimRewards(callbacks);

      // =============== ASSERTIONS ===============
      TransactionAssertions.expectTransactionSuccess(claimResult);
      ServiceAssertions.expectTransaction(services, 'claim_reward_with_referral_rewards');
      CallbackAssertions.expectSuccessCallback(callbacks, /WEEDを獲得しました/);
      
      // Verify balance increased
      const finalState = services.anchorClient.getTokenBalance();
      expect(finalState).resolves.toBeGreaterThan(1500); // Initial balance was 1500
    });

    it('should handle whale player operations', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createWhalePlayerScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const results = await Promise.allSettled([
        services.gameService.claimRewards(callbacks),
        services.gameService.purchaseMysteryPack(callbacks),
        services.anchorClient.claimReferralRewards(),
      ]);

      // =============== ASSERTIONS ===============
      
      // All operations should succeed for whale player
      results.forEach((result, index) => {
        expect(result.status).toBe('fulfilled');
      });

      // Multiple transaction types
      ServiceAssertions.expectTransaction(services, 'claim_reward_with_referral_rewards');
      ServiceAssertions.expectTransaction(services, 'seed_pack');
      expect(services.anchorClient.claimReferralRewards).toHaveBeenCalled();

      // High value rewards
      const userState = await services.anchorClient.getUserState();
      expect(userState.totalGrowPower).toBeGreaterThan(50000); // Whale-level grow power
    });
  });

  describe('Error Handling & Edge Cases', () => {
    it('should gracefully handle network errors', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createNetworkErrorScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      await expect(async () => {
        await services.gameService.claimRewards(callbacks);
      }).rejects.toThrow();

      // =============== ASSERTIONS ===============
      ErrorAssertions.expectProperErrorHandling(callbacks, new Error('Network error'));
      ErrorAssertions.expectErrorType(
        new Error('Network error: RPC endpoint unavailable'),
        'network'
      );
    });

    it('should handle insufficient funds gracefully', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createInsufficientFundsScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      await TransactionAssertions.expectTransactionFailure(
        services.gameService.purchaseMysteryPack(callbacks),
        /Insufficient funds/
      );

      // =============== ASSERTIONS ===============
      CallbackAssertions.expectErrorCallback(callbacks, /購入エラー/);
      ErrorAssertions.expectErrorType(new Error('Insufficient funds'), 'funds');
    });

    it('should handle farm capacity limits', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createFarmAtCapacityScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      await TransactionAssertions.expectTransactionFailure(
        services.anchorClient.plantSeed('test_seed_id'),
        /Farm space capacity exceeded/
      );

      // =============== ASSERTIONS ===============
      const farmState = await services.anchorClient.getFarmSpace();
      GameStateAssertions.expectFarmSpaceState(farmState, {
        seedCount: 8,
        capacity: 8,
      });
    });

    it('should handle special return cases', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createAlreadyOwnedScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const result = await services.gameService.purchaseFarmSpace(callbacks);

      // =============== ASSERTIONS ===============
      expect(result).toBe('農場スペースは既に所有済みです！');
      // Should not show error, just info about already owning
      expect(callbacks.showError).not.toHaveBeenCalled();
    });
  });

  describe('Advanced Game Mechanics', () => {
    it('should handle farm upgrade flow correctly', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createUpgradeInProgressScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      
      // Start upgrade
      const upgradeResult = await services.anchorClient.upgradeFarmSpace();
      TransactionAssertions.expectTransactionSuccess(upgradeResult);

      // Check upgrade state
      const farmState = await services.anchorClient.getFarmSpace();
      GameStateAssertions.expectFarmSpaceState(farmState, {
        isUpgrading: true,
      });

      // Verify cooldown timing
      TimingAssertions.expectUpgradeTimingValid(
        farmState.upgradeStartTime,
        Date.now(),
        false // Not complete yet
      );
    });

    it('should handle upgrade completion', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createUpgradeReadyScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const completionResult = await services.anchorClient.completeFarmSpaceUpgrade();

      // =============== ASSERTIONS ===============
      TransactionAssertions.expectTransactionSuccess(completionResult);
      
      const farmState = await services.anchorClient.getFarmSpace();
      GameStateAssertions.expectFarmSpaceState(farmState, {
        level: 3, // Upgraded from level 2
        capacity: 12, // Level 3 capacity
        isUpgrading: false,
      });
    });

    it('should handle referral network properly', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createReferralNetworkScenario();
      const { services, users, currentUser } = context;

      // =============== TEST EXECUTION ===============
      
      // Create invite code
      const inviteCode = 'TESTCODE';
      await services.anchorClient.createInviteCode(inviteCode);

      // Simulate referred user claiming rewards (triggers referral rewards)
      const referralRewardResult = await services.anchorClient.claimReferralRewards();

      // =============== ASSERTIONS ===============
      TransactionAssertions.expectTransactionSuccess(referralRewardResult);
      
      const userState = await services.anchorClient.getUserState();
      expect(userState.pendingReferralRewards).toBeGreaterThan(0);

      // Verify network structure
      expect(users.length).toBe(5); // Full referral chain
      expect(users[0].name).toContain('TestUser1'); // Network builder at top
    });
  });

  describe('Mystery Pack System', () => {
    it('should handle lucky mystery pack scenario', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createLuckyMysteryPackScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      
      // Purchase pack
      const purchaseResult = await services.gameService.purchaseMysteryPack(callbacks);
      TransactionAssertions.expectTransactionSuccess(purchaseResult);

      // Open pack
      const openResult = await services.anchorClient.openSeedPack('test_pack_id');

      // =============== ASSERTIONS ===============
      expect(openResult.seedType).toBe(8); // High-value seed
      TransactionAssertions.expectTransactionSuccess(openResult.signature);
      
      CallbackAssertions.expectSuccessCallback(callbacks, /ミステリーパックを購入しました/);
    });

    it('should handle unlucky mystery pack scenario', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createUnluckyMysteryPackScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const openResult = await services.anchorClient.openSeedPack('test_pack_id');

      // =============== ASSERTIONS ===============
      expect(openResult.seedType).toBe(1); // Low-value seed
      TransactionAssertions.expectTransactionSuccess(openResult.signature);
      
      // Even unlucky results should be handled gracefully
      expect(callbacks.showError).not.toHaveBeenCalled();
    });
  });

  describe('Performance & Stress Testing', () => {
    it('should handle high load scenarios', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createHighLoadScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const operations = [
        () => services.gameService.claimRewards(callbacks),
        () => services.gameService.purchaseMysteryPack(callbacks),
        () => services.anchorClient.upgradeFarmSpace(),
      ];

      const results = await Promise.allSettled(
        operations.map(async (op, index) => {
          const start = Date.now();
          const result = await op();
          const duration = Date.now() - start;
          return { result, duration, index };
        })
      );

      // =============== ASSERTIONS ===============
      
      // All operations should eventually succeed despite high load
      results.forEach((result) => {
        expect(result.status).toBe('fulfilled');
        
        if (result.status === 'fulfilled') {
          // Should be slower due to simulated high load
          expect(result.value.duration).toBeGreaterThan(2000);
          expect(result.value.duration).toBeLessThan(6000); // But not too slow
        }
      });
    });

    it('should maintain performance under concurrent operations', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createActivePlayerScenario();
      const { services, callbacks } = context;

      // =============== TEST EXECUTION ===============
      const concurrentOps = Array.from({ length: 5 }, () => 
        TimingAssertions.expectWithinTimeLimit(
          () => services.gameService.claimRewards(callbacks),
          500 // Each should complete within 500ms
        )
      );

      const results = await Promise.all(concurrentOps);

      // =============== ASSERTIONS ===============
      results.forEach(result => {
        TransactionAssertions.expectTransactionSuccess(result);
      });
      
      // All operations should have triggered callbacks
      expect(callbacks.updateGameState).toHaveBeenCalledTimes(5);
    });
  });

  describe('Data Factory Testing', () => {
    it('should create realistic test data', () => {
      // =============== USER DATA ===============
      const testUsers = TestDataFactory.createMultipleTestUsers(3, 'Player');
      
      expect(testUsers).toHaveLength(3);
      expect(testUsers[0].name).toBe('Player1');
      expect(testUsers[2].name).toBe('Player3');
      
      testUsers.forEach(user => {
        expect(user.publicKey).toBeDefined();
        expect(user.keypair).toBeDefined();
        expect(user.index).toBeGreaterThan(0);
      });

      // =============== FARM DATA ===============
      const farmSpace = TestDataFactory.createFarmSpace(3);
      
      expect(farmSpace.level).toBe(3);
      expect(farmSpace.capacity).toBe(12); // Level 3 capacity
      expect(farmSpace.seedCount).toBeLessThanOrEqual(farmSpace.capacity);

      // =============== SEED DATA ===============
      const seeds = TestDataFactory.createSeedCollection(testUsers[0].publicKey, 10);
      
      expect(seeds).toHaveLength(10);
      seeds.forEach(seed => {
        expect(seed.owner).toEqual(testUsers[0].publicKey);
        expect(seed.seedType).toBeGreaterThanOrEqual(1);
        expect(seed.seedType).toBeLessThanOrEqual(9);
        expect(seed.growPower).toBeGreaterThan(0);
      });

      // =============== SCENARIO DATA ===============
      const whaleScenario = TestDataFactory.createWhalePlayerScenario();
      
      expect(whaleScenario.farmSpace.level).toBe(5); // Max level
      expect(whaleScenario.seeds.length).toBeGreaterThan(15);
      expect(whaleScenario.userState.totalGrowPower).toBeGreaterThan(50000);
    });

    it('should create referral chains correctly', () => {
      const referralChain = TestDataFactory.createReferralChain(4);
      
      expect(referralChain).toHaveLength(4);
      
      // First user should have no referrer
      expect(referralChain[0].referrer).toBeUndefined();
      
      // Each subsequent user should refer back to the previous one
      for (let i = 1; i < referralChain.length; i++) {
        expect(referralChain[i].referrer).toEqual(referralChain[i - 1].publicKey);
      }
    });
  });

  describe('Integration Testing', () => {
    it('should handle complete user journey end-to-end', async () => {
      // =============== SETUP ===============
      context = TestScenarios.createNewUserScenario();
      const { services, currentUser, callbacks } = context;

      // =============== COMPLETE USER JOURNEY ===============
      
      // Step 1: Wallet connection
      await services.solanaService.connect();
      services.walletService.simulateConnection(currentUser.publicKey, 3.0);
      
      // Step 2: User initialization
      await services.gameService.initializeUser(callbacks);
      
      // Step 3: Farm space purchase
      await services.gameService.purchaseFarmSpace(callbacks);
      
      // Step 4: Wait and claim initial rewards
      await testUtils.flushPromises();
      await services.gameService.claimRewards(callbacks);
      
      // Step 5: Purchase mystery pack
      await services.gameService.purchaseMysteryPack(callbacks);
      
      // Step 6: Create invite code
      await services.anchorClient.createInviteCode('INVITE01');

      // =============== COMPREHENSIVE ASSERTIONS ===============
      
      // Verify all major operations were called
      ServiceAssertions.expectMethodCalled(services, 'walletService', 'connect');
      ServiceAssertions.expectMethodCalled(services, 'gameService', 'initializeUser');
      ServiceAssertions.expectMethodCalled(services, 'gameService', 'purchaseFarmSpace');
      ServiceAssertions.expectMethodCalled(services, 'gameService', 'claimRewards');
      ServiceAssertions.expectMethodCalled(services, 'gameService', 'purchaseMysteryPack');
      
      // Verify final game state
      const finalUserState = await services.anchorClient.getUserState();
      GameStateAssertions.expectUserState(finalUserState, {
        hasUserState: true,
        hasFarmSpace: true,
        totalGrowPower: 100, // Initial seed
      });
      
      // Verify UI feedback was proper throughout
      expect(callbacks.showSuccess).toHaveBeenCalledTimes(4); // Init, purchase, claim, pack
      expect(callbacks.showError).not.toHaveBeenCalled();
      expect(callbacks.updateGameState).toHaveBeenCalledTimes(4);
    });
  });
});