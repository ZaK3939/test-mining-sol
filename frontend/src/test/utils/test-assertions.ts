/**
 * Test Assertions
 * Comprehensive assertion utilities for validating game state and behavior
 */

import { expect } from 'vitest';
import type { MockedFunction } from 'vitest';
import type { PublicKey } from '@solana/web3.js';
import type { UICallbacks } from '../../types';
import type { MockServices } from '../mocks/service-mocks';

// =============== CALLBACK ASSERTIONS ===============

export class CallbackAssertions {
  /**
   * Assert that success callback was called with expected message
   */
  static expectSuccessCallback(
    callbacks: UICallbacks,
    expectedMessage?: string | RegExp
  ): void {
    const mockFn = callbacks.showSuccess as MockedFunction<any>;
    expect(mockFn).toHaveBeenCalled();
    
    if (expectedMessage) {
      if (typeof expectedMessage === 'string') {
        expect(mockFn).toHaveBeenCalledWith(expectedMessage);
      } else {
        expect(mockFn).toHaveBeenCalledWith(expect.stringMatching(expectedMessage));
      }
    }
  }

  /**
   * Assert that error callback was called with expected message
   */
  static expectErrorCallback(
    callbacks: UICallbacks,
    expectedMessage?: string | RegExp
  ): void {
    const mockFn = callbacks.showError as MockedFunction<any>;
    expect(mockFn).toHaveBeenCalled();
    
    if (expectedMessage) {
      if (typeof expectedMessage === 'string') {
        expect(mockFn).toHaveBeenCalledWith(expectedMessage);
      } else {
        expect(mockFn).toHaveBeenCalledWith(expect.stringMatching(expectedMessage));
      }
    }
  }

  /**
   * Assert that info callback was called
   */
  static expectInfoCallback(callbacks: UICallbacks, expectedMessage?: string): void {
    const mockFn = callbacks.showInfo as MockedFunction<any>;
    expect(mockFn).toHaveBeenCalled();
    
    if (expectedMessage) {
      expect(mockFn).toHaveBeenCalledWith(expectedMessage);
    }
  }

  /**
   * Assert that loading states were properly managed
   */
  static expectLoadingStates(callbacks: UICallbacks): void {
    const showLoadingFn = callbacks.showLoading as MockedFunction<any>;
    const hideLoadingFn = callbacks.hideLoading as MockedFunction<any>;
    
    expect(showLoadingFn).toHaveBeenCalled();
    expect(hideLoadingFn).toHaveBeenCalled();
    
    // Verify loading was shown before hidden
    const showCalls = showLoadingFn.mock.invocationCallOrder;
    const hideCalls = hideLoadingFn.mock.invocationCallOrder;
    
    expect(showCalls[0]).toBeLessThan(hideCalls[0]);
  }

  /**
   * Assert that game state update was triggered
   */
  static expectGameStateUpdate(callbacks: UICallbacks): void {
    const mockFn = callbacks.updateGameState as MockedFunction<any>;
    expect(mockFn).toHaveBeenCalled();
  }

  /**
   * Assert no error callbacks were called
   */
  static expectNoErrors(callbacks: UICallbacks): void {
    const mockFn = callbacks.showError as MockedFunction<any>;
    expect(mockFn).not.toHaveBeenCalled();
  }
}

// =============== SERVICE ASSERTIONS ===============

export class ServiceAssertions {
  /**
   * Assert that specific service methods were called
   */
  static expectMethodCalled(
    services: MockServices,
    serviceName: keyof MockServices,
    methodName: string,
    callCount = 1
  ): void {
    const service = services[serviceName] as any;
    const method = service[methodName] as MockedFunction<any>;
    
    expect(method).toHaveBeenCalledTimes(callCount);
  }

  /**
   * Assert that method was called with specific arguments
   */
  static expectMethodCalledWith(
    services: MockServices,
    serviceName: keyof MockServices,
    methodName: string,
    ...expectedArgs: any[]
  ): void {
    const service = services[serviceName] as any;
    const method = service[methodName] as MockedFunction<any>;
    
    expect(method).toHaveBeenCalledWith(...expectedArgs);
  }

  /**
   * Assert that wallet connection was attempted
   */
  static expectWalletConnection(services: MockServices): void {
    expect(services.walletService.connect).toHaveBeenCalled();
  }

  /**
   * Assert that wallet disconnection was attempted
   */
  static expectWalletDisconnection(services: MockServices): void {
    expect(services.walletService.disconnect).toHaveBeenCalled();
  }

  /**
   * Assert specific transaction was executed
   */
  static expectTransaction(
    services: MockServices,
    transactionType: 'user_init' | 'farm_purchase' | 'claim_reward_with_referral_rewards' | 'seed_pack' | 'upgrade'
  ): void {
    switch (transactionType) {
      case 'user_init':
        expect(services.anchorClient.initializeUser).toHaveBeenCalled();
        break;
      case 'farm_purchase':
        expect(services.anchorClient.buyFarmSpace).toHaveBeenCalled();
        break;
      case 'claim_reward_with_referral_rewards':
        expect(services.anchorClient.claimRewardWithReferralRewards).toHaveBeenCalled();
        break;
      case 'seed_pack':
        expect(services.anchorClient.purchaseSeedPack).toHaveBeenCalled();
        break;
      case 'upgrade':
        expect(services.anchorClient.upgradeFarmSpace).toHaveBeenCalled();
        break;
    }
  }
}

// =============== GAME STATE ASSERTIONS ===============

export class GameStateAssertions {
  /**
   * Assert user state matches expected values
   */
  static expectUserState(
    actualState: any,
    expectedState: {
      hasUserState?: boolean;
      hasFarmSpace?: boolean;
      totalGrowPower?: number;
      balance?: number;
    }
  ): void {
    if (expectedState.hasUserState !== undefined) {
      expect(actualState.hasUserState).toBe(expectedState.hasUserState);
    }
    
    if (expectedState.hasFarmSpace !== undefined) {
      expect(actualState.hasFarmSpace).toBe(expectedState.hasFarmSpace);
    }
    
    if (expectedState.totalGrowPower !== undefined) {
      expect(actualState.totalGrowPower).toBe(expectedState.totalGrowPower);
    }
    
    if (expectedState.balance !== undefined) {
      expect(actualState.balance).toBe(expectedState.balance);
    }
  }

  /**
   * Assert farm space state
   */
  static expectFarmSpaceState(
    actualState: any,
    expectedState: {
      level?: number;
      capacity?: number;
      seedCount?: number;
      isUpgrading?: boolean;
    }
  ): void {
    if (expectedState.level !== undefined) {
      expect(actualState.level).toBe(expectedState.level);
    }
    
    if (expectedState.capacity !== undefined) {
      expect(actualState.capacity).toBe(expectedState.capacity);
    }
    
    if (expectedState.seedCount !== undefined) {
      expect(actualState.seedCount).toBe(expectedState.seedCount);
    }
    
    if (expectedState.isUpgrading !== undefined) {
      const isUpgrading = actualState.upgradeStartTime > 0;
      expect(isUpgrading).toBe(expectedState.isUpgrading);
    }
  }

  /**
   * Assert balance changes
   */
  static expectBalanceChange(
    initialBalance: number,
    finalBalance: number,
    expectedChange: number,
    tolerance = 0.01
  ): void {
    const actualChange = finalBalance - initialBalance;
    expect(Math.abs(actualChange - expectedChange)).toBeLessThan(tolerance);
  }

  /**
   * Assert rewards calculation
   */
  static expectRewardsInRange(
    actualRewards: number,
    minExpected: number,
    maxExpected: number
  ): void {
    expect(actualRewards).toBeGreaterThanOrEqual(minExpected);
    expect(actualRewards).toBeLessThanOrEqual(maxExpected);
  }
}

// =============== TRANSACTION ASSERTIONS ===============

export class TransactionAssertions {
  /**
   * Assert transaction completed successfully
   */
  static expectTransactionSuccess(signature: string): void {
    expect(signature).toBeDefined();
    expect(signature).toMatch(/^[a-zA-Z0-9_]+$/);
    expect(signature.length).toBeGreaterThan(10);
  }

  /**
   * Assert transaction failed with expected error
   */
  static async expectTransactionFailure(
    transactionPromise: Promise<any>,
    expectedErrorMessage?: string | RegExp
  ): Promise<void> {
    await expect(transactionPromise).rejects.toThrow();
    
    if (expectedErrorMessage) {
      await expect(transactionPromise).rejects.toThrow(expectedErrorMessage);
    }
  }

  /**
   * Assert special return case
   */
  static async expectSpecialReturn(
    transactionPromise: Promise<any>,
    expectedReturn: string
  ): Promise<void> {
    const result = await expect(transactionPromise).resolves.toBe(expectedReturn);
  }
}

// =============== TIMING ASSERTIONS ===============

export class TimingAssertions {
  /**
   * Assert operation completed within time limit
   */
  static async expectWithinTimeLimit<T>(
    operation: () => Promise<T>,
    maxTimeMs: number
  ): Promise<T> {
    const startTime = Date.now();
    const result = await operation();
    const duration = Date.now() - startTime;
    
    expect(duration).toBeLessThan(maxTimeMs);
    return result;
  }

  /**
   * Assert cooldown period is respected
   */
  static expectCooldownRespected(
    lastActionTime: number,
    currentTime: number,
    cooldownMs: number
  ): void {
    const timeSinceLastAction = currentTime - lastActionTime;
    expect(timeSinceLastAction).toBeGreaterThanOrEqual(cooldownMs);
  }

  /**
   * Assert upgrade timing
   */
  static expectUpgradeTimingValid(
    upgradeStartTime: number,
    currentTime: number,
    isComplete: boolean
  ): void {
    const UPGRADE_DURATION = 24 * 60 * 60 * 1000; // 24 hours
    const timeElapsed = currentTime - upgradeStartTime;
    
    if (isComplete) {
      expect(timeElapsed).toBeGreaterThanOrEqual(UPGRADE_DURATION);
    } else {
      expect(timeElapsed).toBeLessThan(UPGRADE_DURATION);
    }
  }
}

// =============== ERROR ASSERTIONS ===============

export class ErrorAssertions {
  /**
   * Assert specific error type was thrown
   */
  static expectErrorType(
    error: any,
    expectedType: 'network' | 'funds' | 'transaction' | 'validation' | 'authorization'
  ): void {
    expect(error).toBeInstanceOf(Error);
    
    const errorMessage = error.message.toLowerCase();
    
    switch (expectedType) {
      case 'network':
        expect(errorMessage).toMatch(/network|connection|timeout|rpc/);
        break;
      case 'funds':
        expect(errorMessage).toMatch(/insufficient|balance|funds/);
        break;
      case 'transaction':
        expect(errorMessage).toMatch(/transaction|simulation|blockhash/);
        break;
      case 'validation':
        expect(errorMessage).toMatch(/invalid|validation|capacity|limit/);
        break;
      case 'authorization':
        expect(errorMessage).toMatch(/unauthorized|permission|signature/);
        break;
    }
  }

  /**
   * Assert error handling was properly done
   */
  static expectProperErrorHandling(
    callbacks: UICallbacks,
    error: Error
  ): void {
    // Error should be shown to user
    CallbackAssertions.expectErrorCallback(callbacks);
    
    // Loading should be stopped
    const hideLoadingFn = callbacks.hideLoading as MockedFunction<any>;
    expect(hideLoadingFn).toHaveBeenCalled();
    
    // No success message should be shown
    const showSuccessFn = callbacks.showSuccess as MockedFunction<any>;
    expect(showSuccessFn).not.toHaveBeenCalled();
  }
}

// =============== PERFORMANCE ASSERTIONS ===============

export class PerformanceAssertions {
  /**
   * Assert operation performance metrics
   */
  static expectPerformanceMetrics(
    duration: number,
    expectedMaxDuration: number,
    operationName: string
  ): void {
    expect(duration).toBeLessThan(expectedMaxDuration);
    console.log(`✅ ${operationName} completed in ${duration.toFixed(2)}ms`);
  }

  /**
   * Assert memory usage is reasonable
   */
  static expectReasonableMemoryUsage(
    beforeMemory: number,
    afterMemory: number,
    maxIncreaseBytes: number
  ): void {
    const memoryIncrease = afterMemory - beforeMemory;
    expect(memoryIncrease).toBeLessThan(maxIncreaseBytes);
  }

  /**
   * Assert success rate meets threshold
   */
  static expectSuccessRate(
    successCount: number,
    totalCount: number,
    minSuccessRate: number
  ): void {
    const actualSuccessRate = successCount / totalCount;
    expect(actualSuccessRate).toBeGreaterThanOrEqual(minSuccessRate);
    console.log(`✅ Success rate: ${(actualSuccessRate * 100).toFixed(1)}%`);
  }
}

// =============== COMPREHENSIVE TEST RUNNER ===============

export class TestRunner {
  /**
   * Run a comprehensive test with all assertion categories
   */
  static async runComprehensiveTest<T>(
    testName: string,
    operation: () => Promise<T>,
    assertions: {
      callbacks?: UICallbacks;
      services?: MockServices;
      gameState?: any;
      transaction?: boolean;
      timing?: { maxMs: number };
      performance?: { maxMs: number };
    }
  ): Promise<T> {
    console.log(`🧪 Running comprehensive test: ${testName}`);
    
    const startTime = Date.now();
    let result: T;
    let error: Error | null = null;
    
    try {
      result = await operation();
      
      // Transaction assertions
      if (assertions.transaction) {
        TransactionAssertions.expectTransactionSuccess(result as string);
      }
      
      // Callback assertions
      if (assertions.callbacks) {
        CallbackAssertions.expectNoErrors(assertions.callbacks);
        CallbackAssertions.expectGameStateUpdate(assertions.callbacks);
      }
      
      // Service assertions
      if (assertions.services) {
        // Add specific service assertions as needed
      }
      
      console.log(`✅ Test passed: ${testName}`);
      
    } catch (e) {
      error = e as Error;
      
      // Error handling assertions
      if (assertions.callbacks) {
        ErrorAssertions.expectProperErrorHandling(assertions.callbacks, error);
      }
      
      console.log(`❌ Test failed: ${testName} - ${error.message}`);
      throw error;
      
    } finally {
      const duration = Date.now() - startTime;
      
      // Timing assertions
      if (assertions.timing) {
        TimingAssertions.expectWithinTimeLimit(
          () => Promise.resolve(result!),
          assertions.timing.maxMs
        );
      }
      
      // Performance assertions
      if (assertions.performance) {
        PerformanceAssertions.expectPerformanceMetrics(
          duration,
          assertions.performance.maxMs,
          testName
        );
      }
    }
    
    return result!;
  }
}