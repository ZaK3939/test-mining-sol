# Farm Game Test Suite

This directory contains comprehensive tests for the Farm Game Solana program.

## Test Structure

### Organization
```
tests/
├── helpers/                    # Test utilities and shared code
│   ├── test-setup.ts          # TestEnvironment class and setup utilities
│   ├── assertions.ts          # Custom assertion functions
│   └── test-data.ts           # Test data generators and mocks
├── integration/               # Integration tests
│   ├── core-game-mechanics.test.ts    # Core game functionality
│   ├── referral-system.test.ts        # Referral reward system
│   └── halving-mechanism.test.ts      # Halving mechanism tests
└── farm-game.test.ts          # Main test entry point
```

### Test Categories

1. **Core Game Mechanics** (`core-game-mechanics.test.ts`)
   - System initialization
   - User management
   - Farm space management
   - Basic reward system
   - State consistency

2. **Referral System** (`referral-system.test.ts`)
   - Multi-level referral chains
   - Referral reward distribution
   - Protocol address exclusion
   - Edge cases and performance

3. **Halving Mechanism** (`halving-mechanism.test.ts`)
   - Single and multiple halving events
   - Cross-halving reward calculations
   - Edge cases and system integration

## Running Tests

### Prerequisites
1. Start local Solana test validator:
   ```bash
   solana-test-validator --reset
   ```

2. Configure Solana CLI for localhost:
   ```bash
   solana config set --url localhost
   ```

### Test Commands

#### Run All Tests
```bash
anchor test --skip-local-validator
```

#### Run Specific Test Suite
```bash
# Core game mechanics only
anchor test --skip-local-validator tests/integration/core-game-mechanics.test.ts

# Referral system only
anchor test --skip-local-validator tests/integration/referral-system.test.ts

# Halving mechanism only
anchor test --skip-local-validator tests/integration/halving-mechanism.test.ts
```

#### Run with Verbose Output
```bash
anchor test --skip-local-validator -- --reporter spec
```

## Test Utilities

### TestEnvironment Class
The `TestEnvironment` class provides a high-level interface for test setup and execution:

```typescript
import { TestEnvironment } from './helpers/test-setup';

// Setup test environment
const testEnv = await TestEnvironment.setup();
await testEnv.initializeSystem();

// Create test user
const userData = await testEnv.createUserTestData(0);
await testEnv.buyFarmSpace(userData);

// Test operations
const reward = await testEnv.claimRewards(userData);
```

### Assertions
Custom assertion functions for game-specific validations:

```typescript
import { GameAssertions } from './helpers/assertions';

// Assert token balance
await GameAssertions.assertTokenBalance(
  connection, 
  tokenAccount, 
  expectedAmount, 
  tolerance
);

// Assert referral reward percentages
GameAssertions.assertReferralReward(
  baseReward, 
  actualReward, 
  percentage
);

// Assert game state
GameAssertions.assertUserState(userState, expected);
GameAssertions.assertFarmSpace(farmSpace, expected);
```

### Test Data Generators
Utilities for generating test scenarios:

```typescript
import { TestDataGenerator } from './helpers/test-data';

// Generate referral chain
const chain = TestDataGenerator.generateReferralChain(5);

// Generate upgrade scenarios
const scenarios = TestDataGenerator.generateFarmUpgradeScenarios();

// Generate halving scenarios
const halvingTests = TestDataGenerator.generateHalvingScenarios();
```

## Test Configuration

### Constants
Test constants are defined in `helpers/test-setup.ts`:

```typescript
export const TEST_CONSTANTS = {
  BASE_RATE: 100,
  HALVING_INTERVAL: 6 * 24 * 60 * 60, // 6 days
  LEVEL1_REFERRAL_PERCENTAGE: 10,
  LEVEL2_REFERRAL_PERCENTAGE: 5,
  // ... more constants
};
```

### Timeouts
Tests involving time-based mechanics (halving, rewards) have extended timeouts:
- Default: 30 seconds
- Halving tests: 2-5 minutes
- Performance tests: 30 seconds - 2 minutes

## Test Data and Scenarios

### Referral Chains
Tests support various referral relationship patterns:
- Linear chains (A → B → C → D)
- Star patterns (A ← B, C, D, E)
- Complex networks with multiple levels

### Halving Scenarios
- Short intervals for testing (30-60 seconds)
- Multiple consecutive halvings
- Cross-halving reward calculations
- Zero grow power edge cases

### Performance Tests
- Concurrent operations (5-50 users)
- Large referral networks (10-100 users)
- Stress testing with maximum limits

## Best Practices

### Writing New Tests
1. Use `TestEnvironment` for setup
2. Use `GameAssertions` for validations
3. Use `TestDataGenerator` for test data
4. Follow the existing test structure
5. Add appropriate timeouts for time-based tests

### Test Organization
- Group related tests in `describe` blocks
- Use descriptive test names
- Include setup and teardown as needed
- Document complex test scenarios

### Performance Considerations
- Use parallel execution where possible
- Clean up resources between tests
- Use reasonable timeouts
- Consider validator limitations

## Troubleshooting

### Common Issues

1. **Validator Connection Issues**
   - Ensure `solana-test-validator` is running
   - Check Solana CLI configuration
   - Verify localhost is accessible

2. **Timeout Errors**
   - Increase test timeouts for time-based tests
   - Check validator performance
   - Reduce test complexity if needed

3. **Account Initialization Errors**
   - Ensure proper account setup order
   - Check for duplicate initializations
   - Verify PDA derivations

4. **Token Account Issues**
   - Ensure token accounts are created before use
   - Check mint authority configurations
   - Verify sufficient balances

### Debug Tips
- Use `console.log` for intermediate values
- Check on-chain state between operations
- Verify transaction signatures and logs
- Use Solana Explorer for transaction details

## Contributing

When adding new tests:
1. Follow the existing structure and patterns
2. Add appropriate documentation
3. Ensure tests are deterministic
4. Include both positive and negative test cases
5. Update this README if adding new test categories

## Legacy Test Files

The following legacy test files are maintained for compatibility but should not be used for new tests:
- `basic-farm-game.ts` - Superseded by core-game-mechanics.test.ts
- `comprehensive-facility-game.ts` - Old naming, functionality moved to new structure
- `facility-game.ts` - Basic tests, superseded by integration tests
- `referral-rewards.ts` - Superseded by referral-system.test.ts

## Performance Benchmarks

Expected performance thresholds:

| Operation | Expected Time | Max Acceptable |
|-----------|---------------|----------------|
| User Init | < 1s | 3s |
| Farm Purchase | < 1s | 3s |
| Reward Claim | < 2s | 5s |
| Referral Distribution | < 1s | 4s |
| Halving Calculation | < 3s | 5s |
| Concurrent Claims (5 users) | < 10s | 15s |

## Security Coverage

The test suite covers:
- ✅ Access control enforcement
- ✅ PDA ownership verification
- ✅ Input validation and overflow protection
- ✅ Token account security
- ✅ Protocol address exclusion
- ✅ Referral system integrity
- ✅ Halving mechanism accuracy