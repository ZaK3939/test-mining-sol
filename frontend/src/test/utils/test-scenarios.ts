/**
 * Test Scenarios
 * Comprehensive scenarios for testing different game states and user journeys
 */

import { vi } from 'vitest';
import { PublicKey } from '@solana/web3.js';
import { MockServiceFactory, type MockServices } from '../mocks/service-mocks';
import { TestDataFactory, type TestUser } from '../factories/test-data-factory';
import type { UICallbacks } from '../../types';

export interface ScenarioContext {
  services: MockServices;
  users: TestUser[];
  currentUser: TestUser;
  callbacks: UICallbacks;
}

export class TestScenarios {
  private static mockCallbacks: UICallbacks = {
    showSuccess: vi.fn(),
    showError: vi.fn(),
    showInfo: vi.fn(),
    showLoading: vi.fn(),
    hideLoading: vi.fn(),
    updateGameState: vi.fn(),
  };

  // =============== BASIC SCENARIOS ===============

  /**
   * New user scenario - no game state, wallet not connected
   */
  static createNewUserScenario(): ScenarioContext {
    const services = MockServiceFactory.createFullMockSuite();
    const users = [TestDataFactory.createTestUser('NewUser')];
    
    // Ensure clean state
    services.walletService.simulateDisconnection();
    services.gameService.resetMockState();
    
    return {
      services,
      users,
      currentUser: users[0],
      callbacks: { ...this.mockCallbacks },
    };
  }

  /**
   * Connected but uninitialized user
   */
  static createConnectedUninitializedScenario(): ScenarioContext {
    const context = this.createNewUserScenario();
    
    // Connect wallet but don't initialize game state
    context.services.walletService.simulateConnection(
      context.currentUser.publicKey,
      2.5 // 2.5 SOL balance
    );
    
    return context;
  }

  /**
   * Fully initialized active player
   */
  static createActivePlayerScenario(): ScenarioContext {
    const context = this.createConnectedUninitializedScenario();
    
    // Set up active game state
    context.services.anchorClient.setMockGameState({
      hasUserState: true,
      hasFarmSpace: true,
      farmLevel: 2,
      farmCapacity: 8,
      seedCount: 5,
      totalGrowPower: 2100, // Mix of seed types
      lastHarvestTime: Date.now() - 2 * 60 * 60 * 1000, // 2 hours ago
      claimableRewards: 420.5,
      balance: 1500,
    });
    
    return context;
  }

  /**
   * Whale player with max farm and resources
   */
  static createWhalePlayerScenario(): ScenarioContext {
    const context = this.createConnectedUninitializedScenario();
    
    // Set up whale game state
    context.services.anchorClient.setMockGameState({
      hasUserState: true,
      hasFarmSpace: true,
      farmLevel: 5,
      farmCapacity: 20,
      seedCount: 20,
      totalGrowPower: 150000, // High-value seeds
      lastHarvestTime: Date.now() - 30 * 60 * 1000, // 30 minutes ago
      claimableRewards: 2500,
      balance: 50000,
    });
    
    return context;
  }

  // =============== ERROR SCENARIOS ===============

  /**
   * Network error scenario
   */
  static createNetworkErrorScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Simulate network errors
    context.services.anchorClient.simulateError(
      'claimReward',
      new Error('Network error: RPC endpoint unavailable')
    );
    
    context.services.anchorClient.simulateError(
      'getUserState', 
      new Error('Connection timeout')
    );
    
    return context;
  }

  /**
   * Insufficient funds scenario
   */
  static createInsufficientFundsScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Set low balance
    context.services.anchorClient.setMockGameState({
      balance: 50, // Not enough for seed pack (300 WEED)
    });
    
    // Simulate insufficient funds errors
    context.services.anchorClient.simulateError(
      'purchaseSeedPack',
      new Error('Insufficient funds')
    );
    
    context.services.anchorClient.simulateError(
      'upgradeFarmSpace',
      new Error('Insufficient funds')
    );
    
    return context;
  }

  /**
   * Transaction failure scenario
   */
  static createTransactionFailureScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Simulate various transaction failures
    context.services.anchorClient.simulateError(
      'claimReward',
      new Error('Transaction failed: blockhash not found')
    );
    
    context.services.anchorClient.simulateError(
      'plantSeed',
      new Error('Transaction failed: simulation failed')
    );
    
    return context;
  }

  // =============== SPECIAL GAME STATE SCENARIOS ===============

  /**
   * Farm at capacity scenario
   */
  static createFarmAtCapacityScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Set farm to full capacity
    context.services.anchorClient.setMockGameState({
      seedCount: 8, // Same as capacity
      farmCapacity: 8,
    });
    
    // Simulate capacity errors
    context.services.anchorClient.simulateError(
      'plantSeed',
      new Error('Farm space capacity exceeded')
    );
    
    return context;
  }

  /**
   * Upgrade in progress scenario
   */
  static createUpgradeInProgressScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Mock upgrade in progress
    const upgradeStartTime = Date.now() - 12 * 60 * 60 * 1000; // 12 hours ago
    
    context.services.anchorClient.getFarmSpace = vi.fn().mockResolvedValue({
      owner: context.currentUser.publicKey,
      level: 2,
      capacity: 8,
      seedCount: 5,
      totalGrowPower: 2100,
      upgradeStartTime,
      upgradeTargetLevel: 3,
    });
    
    return context;
  }

  /**
   * Ready to complete upgrade scenario
   */
  static createUpgradeReadyScenario(): ScenarioContext {
    const context = this.createUpgradeInProgressScenario();
    
    // Mock upgrade ready to complete (24+ hours passed)
    const upgradeStartTime = Date.now() - 25 * 60 * 60 * 1000; // 25 hours ago
    
    context.services.anchorClient.getFarmSpace = vi.fn().mockResolvedValue({
      owner: context.currentUser.publicKey,
      level: 2,
      capacity: 8,
      seedCount: 5,
      totalGrowPower: 2100,
      upgradeStartTime,
      upgradeTargetLevel: 3,
    });
    
    return context;
  }

  // =============== REFERRAL SCENARIOS ===============

  /**
   * Referral network scenario
   */
  static createReferralNetworkScenario(): ScenarioContext {
    const services = MockServiceFactory.createFullMockSuite();
    const referralChain = TestDataFactory.createReferralChain(5);
    
    // Set up network builder as current user
    const currentUser = referralChain[0];
    services.walletService.simulateConnection(currentUser.publicKey, 3.0);
    
    // Mock referral rewards
    services.anchorClient.setMockGameState({
      hasUserState: true,
      hasFarmSpace: true,
      farmLevel: 3,
      farmCapacity: 12,
      seedCount: 8,
      totalGrowPower: 5000,
      lastHarvestTime: Date.now() - 60 * 60 * 1000,
      claimableRewards: 300,
      balance: 2000,
    });
    
    // Mock pending referral rewards
    services.anchorClient.getUserState = vi.fn().mockResolvedValue({
      owner: currentUser.publicKey,
      totalGrowPower: 5000,
      lastHarvestTime: Date.now() - 60 * 60 * 1000,
      hasFarmSpace: true,
      referrer: null,
      pendingReferralRewards: 850, // Substantial referral income
    });
    
    return {
      services,
      users: referralChain,
      currentUser,
      callbacks: { ...this.mockCallbacks },
    };
  }

  /**
   * Referred user scenario
   */
  static createReferredUserScenario(): ScenarioContext {
    const context = this.createNewUserScenario();
    const referrer = TestDataFactory.createTestUser('Referrer');
    
    // Set current user as referred
    context.users.push(referrer);
    
    // Mock user state with referrer
    context.services.anchorClient.getUserState = vi.fn().mockResolvedValue({
      owner: context.currentUser.publicKey,
      totalGrowPower: 100,
      lastHarvestTime: Date.now(),
      hasFarmSpace: true,
      referrer: referrer.publicKey,
      pendingReferralRewards: 0,
    });
    
    return context;
  }

  // =============== MYSTERY PACK SCENARIOS ===============

  /**
   * Lucky mystery pack scenario (high value seeds)
   */
  static createLuckyMysteryPackScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Mock high-value seed results
    context.services.anchorClient.openSeedPack = vi.fn().mockResolvedValue({
      seedType: 8, // Seed8 - very rare
      signature: 'lucky_pack_signature',
    });
    
    return context;
  }

  /**
   * Unlucky mystery pack scenario (low value seeds)
   */
  static createUnluckyMysteryPackScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Mock low-value seed results
    context.services.anchorClient.openSeedPack = vi.fn().mockResolvedValue({
      seedType: 1, // Seed1 - common
      signature: 'unlucky_pack_signature',
    });
    
    return context;
  }

  // =============== SPECIAL RETURN SCENARIOS ===============

  /**
   * Already owned scenarios (for duplicate actions)
   */
  static createAlreadyOwnedScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Mock "already owned" special returns
    context.services.anchorClient.simulateSpecialReturn(
      'buyFarmSpace',
      'already_owned'
    );
    
    context.services.anchorClient.simulateSpecialReturn(
      'initializeUser',
      'already_initialized'
    );
    
    return context;
  }

  // =============== PERFORMANCE SCENARIOS ===============

  /**
   * High load scenario for stress testing
   */
  static createHighLoadScenario(): ScenarioContext {
    const context = this.createActivePlayerScenario();
    
    // Add artificial delays to simulate high load
    const originalMethods = [
      'claimReward',
      'purchaseSeedPack',
      'plantSeed',
      'upgradeFarmSpace',
    ] as const;
    
    originalMethods.forEach(method => {
      const originalMethod = context.services.anchorClient[method];
      context.services.anchorClient[method] = vi.fn().mockImplementation(async (...args) => {
        // Add 2-5 second delay to simulate high load
        await new Promise(resolve => 
          setTimeout(resolve, 2000 + Math.random() * 3000)
        );
        return originalMethod.apply(context.services.anchorClient, args);
      });
    });
    
    return context;
  }

  // =============== UTILITY METHODS ===============

  /**
   * Apply multiple error conditions to a scenario
   */
  static applyErrorConditions(
    context: ScenarioContext,
    errorTypes: Array<'network' | 'funds' | 'transaction' | 'capacity'>
  ): ScenarioContext {
    errorTypes.forEach(errorType => {
      switch (errorType) {
        case 'network':
          context.services.anchorClient.simulateError(
            'getUserState',
            new Error('Network error')
          );
          break;
        case 'funds':
          context.services.anchorClient.setMockGameState({ balance: 10 });
          break;
        case 'transaction':
          context.services.anchorClient.simulateError(
            'claimReward',
            new Error('Transaction failed')
          );
          break;
        case 'capacity':
          context.services.anchorClient.setMockGameState({
            seedCount: 8,
            farmCapacity: 8,
          });
          break;
      }
    });
    
    return context;
  }

  /**
   * Reset all mocks in a scenario
   */
  static resetScenario(context: ScenarioContext): void {
    MockServiceFactory.resetAllMocks(context.services);
    TestDataFactory.resetCounters();
    
    // Reset callbacks
    Object.values(context.callbacks).forEach(callback => {
      if (vi.isMockFunction(callback)) {
        callback.mockClear();
      }
    });
  }

  /**
   * Get fresh callbacks for testing
   */
  static getFreshCallbacks(): UICallbacks {
    return {
      showSuccess: vi.fn(),
      showError: vi.fn(),
      showInfo: vi.fn(),
      showLoading: vi.fn(),
      hideLoading: vi.fn(),
      updateGameState: vi.fn(),
    };
  }
}