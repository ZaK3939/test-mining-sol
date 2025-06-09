import { expect } from 'chai';

// Global test configuration
export const TEST_CONFIG = {
  // Timeouts
  DEFAULT_TIMEOUT: 30000,
  LONG_TIMEOUT: 60000,
  
  // Performance thresholds
  MAX_INSTRUCTION_TIME: 5000,
  MAX_COMPLEX_OPERATION_TIME: 10000,
  
  // Concurrency limits
  MAX_CONCURRENT_USERS: 10,
  
  // Token amounts for testing
  INITIAL_AIRDROP: 5,
  MYSTERY_BOX_COST: 1000,
  MACHINE_COST: 500,
  
  // Retry configuration
  MAX_RETRIES: 3,
  RETRY_DELAY: 1000,
};

// Test utilities
export class TestUtils {
  static async delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  static async retryAsync<T>(
    operation: () => Promise<T>,
    maxRetries: number = TEST_CONFIG.MAX_RETRIES,
    delayMs: number = TEST_CONFIG.RETRY_DELAY
  ): Promise<T> {
    let lastError: any;
    
    for (let i = 0; i <= maxRetries; i++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error;
        if (i < maxRetries) {
          console.warn(`Attempt ${i + 1} failed, retrying in ${delayMs}ms...`);
          await this.delay(delayMs);
        }
      }
    }
    
    throw lastError;
  }

  static expectTransaction(promise: Promise<any>) {
    return expect(promise).to.eventually.be.fulfilled;
  }

  static expectTransactionError(promise: Promise<any>, errorMessage?: string) {
    const assertion = expect(promise).to.eventually.be.rejected;
    return errorMessage ? assertion.and.have.property('message').that.includes(errorMessage) : assertion;
  }

  static formatBalance(amount: bigint | number): string {
    const num = typeof amount === 'bigint' ? Number(amount) : amount;
    return (num / 1_000_000).toFixed(6); // Assuming 6 decimals
  }

  static randomString(length: number = 8): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }
}

// Performance measurement utilities
export class PerformanceTracker {
  private static measurements: Map<string, number[]> = new Map();

  static startMeasurement(name: string): () => number {
    const startTime = process.hrtime.bigint();
    
    return () => {
      const endTime = process.hrtime.bigint();
      const duration = Number(endTime - startTime) / 1000000; // Convert to milliseconds
      
      if (!this.measurements.has(name)) {
        this.measurements.set(name, []);
      }
      this.measurements.get(name)!.push(duration);
      
      return duration;
    };
  }

  static getStats(name: string) {
    const measurements = this.measurements.get(name) || [];
    if (measurements.length === 0) {
      return null;
    }

    const sorted = [...measurements].sort((a, b) => a - b);
    const sum = measurements.reduce((a, b) => a + b, 0);

    return {
      count: measurements.length,
      min: Math.min(...measurements),
      max: Math.max(...measurements),
      average: sum / measurements.length,
      median: sorted[Math.floor(sorted.length / 2)],
      p95: sorted[Math.floor(sorted.length * 0.95)],
      p99: sorted[Math.floor(sorted.length * 0.99)],
    };
  }

  static printSummary() {
    console.log('\n=== PERFORMANCE SUMMARY ===');
    console.log('Operation                  | Count | Avg (ms) | Min (ms) | Max (ms) | P95 (ms) | P99 (ms)');
    console.log('---------------------------|-------|----------|----------|----------|----------|----------');

    for (const [name, _] of this.measurements) {
      const stats = this.getStats(name);
      if (!stats) continue;

      const operation = name.padEnd(26);
      const count = stats.count.toString().padStart(5);
      const avg = stats.average.toFixed(2).padStart(8);
      const min = stats.min.toFixed(2).padStart(8);
      const max = stats.max.toFixed(2).padStart(8);
      const p95 = stats.p95.toFixed(2).padStart(8);
      const p99 = stats.p99.toFixed(2).padStart(8);

      console.log(`${operation} | ${count} | ${avg} | ${min} | ${max} | ${p95} | ${p99}`);
    }
    console.log('============================\n');
  }

  static clear() {
    this.measurements.clear();
  }
}

// Security test utilities
export class SecurityTestUtils {
  static generateMaliciousInputs() {
    return {
      largeNumbers: [
        2n ** 64n - 1n, // Max u64
        2n ** 63n - 1n, // Max i64
      ],
      negativeNumbers: [-1, -100, -1000],
      zeroValues: [0],
      specialStrings: [
        '', // Empty string
        'x'.repeat(1000), // Very long string
        '\x00\x01\x02', // Binary data
      ],
    };
  }

  static async testAccessControl<T>(
    validOperation: () => Promise<T>,
    invalidOperations: Array<{
      name: string;
      operation: () => Promise<any>;
    }>
  ) {
    // First ensure the valid operation works
    await expect(validOperation()).to.eventually.be.fulfilled;

    // Then test that invalid operations fail
    for (const { name, operation } of invalidOperations) {
      try {
        await operation();
        throw new Error(`Security test failed: ${name} should have been rejected`);
      } catch (error) {
        // Expected to fail
        console.log(`✓ Security test passed: ${name} properly rejected`);
      }
    }
  }
}

// Error matchers for common Solana/Anchor errors
export const ErrorMatchers = {
  ALREADY_HAS_FACILITY: /AlreadyHasFacility/,
  NO_FACILITY: /NoFacility/,
  NO_REWARD_TO_CLAIM: /NoRewardToClaim/,
  INSUFFICIENT_FUNDS: /InsufficientFunds/,
  INVALID_REFERRER: /InvalidReferrer/,
  CALCULATION_OVERFLOW: /CalculationOverflow/,
  FACILITY_AT_MAX_CAPACITY: /FacilityAtMaxCapacity/,
  UNAUTHORIZED: /Unauthorized/,
  CONSTRAINT_VIOLATION: /ConstraintHasOneGroup|ConstraintRaw|ConstraintOwner|ConstraintMint/,
  ACCOUNT_NOT_FOUND: /AccountNotFound/,
  PROGRAM_ERROR: /ProgramError/,
};