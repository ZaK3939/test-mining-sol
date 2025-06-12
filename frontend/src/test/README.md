# Modern Frontend Testing Framework

A comprehensive, modern testing framework for the WEED Farm Game frontend with proper mocking, test data factories, and assertion utilities.

## 🚀 Quick Start

```bash
# Run all tests
bun run test

# Run tests with UI
bun run test:ui

# Run tests with coverage
bun run test:coverage

# Run tests in debug mode
bun run test:debug

# Run integration tests only
bun run test:integration
```

## 📁 Framework Structure

```
src/test/
├── setup.ts                    # Global test setup and mocking
├── vitest.config.ts            # Vitest configuration
├── mocks/
│   └── service-mocks.ts         # Comprehensive service mocks
├── factories/
│   └── test-data-factory.ts     # Test data generation
├── utils/
│   ├── test-scenarios.ts        # Pre-built test scenarios
│   ├── test-assertions.ts       # Assertion utilities
│   └── test-helpers.ts          # Legacy helpers (being phased out)
└── examples/
    └── modern-testing-example.test.ts  # Framework usage examples
```

## 🎯 Key Features

### 1. **Comprehensive Service Mocking**
- Mock wallet services (Phantom, connection states)
- Mock Anchor client with realistic game state simulation
- Mock Solana RPC interactions
- Configurable error simulation

### 2. **Test Data Factories**
- Generate realistic test users, farm spaces, seeds
- Create complex scenarios (whale players, referral chains)
- Configurable probability distributions
- Reset utilities for clean test isolation

### 3. **Scenario-Based Testing**
- Pre-built scenarios for common test cases
- Error scenarios (network, funds, capacity)
- Performance and stress testing scenarios
- Edge case handling

### 4. **Advanced Assertions**
- Callback verification (success, error, loading states)
- Service interaction validation
- Game state assertions
- Transaction result verification
- Timing and performance validation

## 🧪 Writing Tests

### Basic Test Structure

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { TestScenarios } from '../utils/test-scenarios';
import { CallbackAssertions, ServiceAssertions } from '../utils/test-assertions';

describe('My Component', () => {
  let context: ScenarioContext;

  beforeEach(() => {
    context = TestScenarios.createActivePlayerScenario();
  });

  it('should handle user action', async () => {
    const { services, callbacks } = context;
    
    await services.gameService.claimRewards(callbacks);
    
    ServiceAssertions.expectTransaction(services, 'claim_reward_with_referral_rewards');
    CallbackAssertions.expectSuccessCallback(callbacks);
  });
});
```

### Using Test Scenarios

```typescript
// New user onboarding
const context = TestScenarios.createNewUserScenario();

// Active player with existing farm
const context = TestScenarios.createActivePlayerScenario();

// Whale player with max resources
const context = TestScenarios.createWhalePlayerScenario();

// Error scenarios
const context = TestScenarios.createNetworkErrorScenario();
const context = TestScenarios.createInsufficientFundsScenario();

// Special game states
const context = TestScenarios.createUpgradeInProgressScenario();
const context = TestScenarios.createReferralNetworkScenario();
```

### Creating Test Data

```typescript
// Generate test users
const users = TestDataFactory.createMultipleTestUsers(5);

// Create farm spaces at different levels
const level3Farm = TestDataFactory.createFarmSpace(3);
const maxLevelFarm = TestDataFactory.createMaxLevelFarmSpace();

// Generate seed collections
const seeds = TestDataFactory.createSeedCollection(userPublicKey, 10);
const rareSeeds = TestDataFactory.createHighValueSeedCollection(userPublicKey);

// Create referral chains
const referralChain = TestDataFactory.createReferralChain(5);
```

### Advanced Assertions

```typescript
// Callback assertions
CallbackAssertions.expectSuccessCallback(callbacks, /successfully/);
CallbackAssertions.expectErrorCallback(callbacks, /failed/);
CallbackAssertions.expectLoadingStates(callbacks);

// Service assertions
ServiceAssertions.expectWalletConnection(services);
ServiceAssertions.expectTransaction(services, 'claim_reward_with_referral_rewards');
ServiceAssertions.expectMethodCalledWith(services, 'anchorClient', 'claimReward');

// Game state assertions
GameStateAssertions.expectUserState(userState, {
  hasFarmSpace: true,
  totalGrowPower: 1000
});

// Transaction assertions
await TransactionAssertions.expectTransactionFailure(
  services.gameService.purchaseMysteryPack(callbacks),
  /Insufficient funds/
);
```

## 🔧 Configuration Options

### Environment Variables

```bash
# Enable verbose test output
VERBOSE_TESTS=true bun run test

# Enable strict error testing
STRICT_ERROR_TESTING=true bun run test

# Use fake timers for time-dependent tests
USE_FAKE_TIMERS=true bun run test
```

### Mock Configuration

```typescript
// Simulate wallet connection
services.walletService.simulateConnection(publicKey, 2.5);

// Simulate specific errors
services.anchorClient.simulateError('claimReward', new Error('Network error'));

// Set custom game state
services.anchorClient.setMockGameState({
  balance: 1000,
  farmLevel: 3,
  claimableRewards: 500
});
```

## 📊 Testing Patterns

### 1. **User Journey Testing**
Test complete user flows from wallet connection to advanced operations:

```typescript
it('should handle complete user onboarding', async () => {
  // Connect wallet
  await services.solanaService.connect();
  
  // Initialize user
  await services.gameService.initializeUser(callbacks);
  
  // Purchase farm
  await services.gameService.purchaseFarmSpace(callbacks);
  
  // Verify complete state
  const userState = await services.anchorClient.getUserState();
  GameStateAssertions.expectUserState(userState, {
    hasUserState: true,
    hasFarmSpace: true
  });
});
```

### 2. **Error Handling Testing**
Verify graceful error handling across all scenarios:

```typescript
it('should handle network errors gracefully', async () => {
  const context = TestScenarios.createNetworkErrorScenario();
  
  await expect(
    context.services.gameService.claimRewards(context.callbacks)
  ).rejects.toThrow();
  
  ErrorAssertions.expectProperErrorHandling(context.callbacks, error);
});
```

### 3. **Performance Testing**
Validate performance under various conditions:

```typescript
it('should complete operations within time limits', async () => {
  await TimingAssertions.expectWithinTimeLimit(
    () => services.gameService.claimRewards(callbacks),
    1000 // 1 second max
  );
});
```

### 4. **Integration Testing**
Test complete workflows with multiple components:

```typescript
it('should handle referral reward distribution', async () => {
  const context = TestScenarios.createReferralNetworkScenario();
  
  // Referred user claims rewards
  await context.services.gameService.claimRewards(context.callbacks);
  
  // Referrer should receive commission
  const referrerState = await context.services.anchorClient.getUserState();
  expect(referrerState.pendingReferralRewards).toBeGreaterThan(0);
});
```

## 🛠 Utilities & Helpers

### Global Test Utilities

```typescript
// Available in all tests via global.testUtils
await testUtils.waitForNextTick();
await testUtils.flushPromises();
testUtils.mockConsole();
testUtils.restoreConsole();
const errors = testUtils.captureErrors();
```

### Test Runner

```typescript
await TestRunner.runComprehensiveTest(
  'Complex Operation',
  async () => {
    // Your test operation
    return await someComplexOperation();
  },
  {
    callbacks,
    services,
    transaction: true,
    timing: { maxMs: 1000 },
    performance: { maxMs: 500 }
  }
);
```

## 📈 Coverage & Quality

### Coverage Targets
- **Branches**: 80%
- **Functions**: 80%
- **Lines**: 80%
- **Statements**: 80%

### Quality Checks
```bash
# Run all quality checks
bun run quality-check

# Individual checks
bun run type-check
bun run lint
bun run format:check
bun run test:coverage
```

## 🚨 Common Pitfalls

1. **Mock Cleanup**: Always reset mocks between tests using `TestScenarios.resetScenario()`
2. **Async Operations**: Use `testUtils.flushPromises()` to wait for async operations
3. **Error Testing**: Use proper error assertion utilities instead of raw expect().rejects
4. **State Isolation**: Each test should start with fresh state using appropriate scenario creators

## 🔍 Debugging

### Debug Mode
```bash
VERBOSE_TESTS=true bun run test:debug
```

### Error Capture
```typescript
const errors = testUtils.captureErrors();
console.log('Captured errors:', errors);
```

### Mock Inspection
```typescript
expect(callbacks.showSuccess).toHaveBeenCalledTimes(1);
console.log('Mock calls:', callbacks.showSuccess.mock.calls);
```

## 📚 Examples

See `src/test/examples/modern-testing-example.test.ts` for comprehensive examples covering:
- User journey testing
- Error handling
- Performance testing
- Integration testing
- Data factory usage
- Advanced assertion patterns

This modern testing framework ensures robust, maintainable tests that accurately reflect real-world usage patterns while providing excellent developer experience and debugging capabilities.