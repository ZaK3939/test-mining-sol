# Facility Game Test Suite

This directory contains a comprehensive test suite for the Facility Game smart contract, covering functionality, security, and performance aspects.

## Test Files

### 1. `facility-game.ts`
Basic integration tests covering the core functionality:
- System initialization (config, mint creation)
- User lifecycle (initialization, facility purchase)
- Reward claiming mechanism
- Basic error scenarios

### 2. `comprehensive-tests.ts`
Complete end-to-end tests for all 11 instructions:
- User management & referral system
- Facility management (purchase, upgrade, machine addition)
- Reward system with referral distribution
- Transfer system with 2% fees
- Mystery box purchase and opening
- Error scenarios and edge cases

### 3. `security-tests.ts`
Security-focused tests including:
- Access control verification
- Input validation
- PDA security and ownership constraints
- Reentrancy protection
- Token security measures
- Mystery box security

### 4. `performance-tests.ts`
Performance benchmarking and stress tests:
- Core instruction performance measurement
- Reward system performance under various conditions
- Transfer system benchmarks
- Mystery box operation timing
- Concurrent user stress testing
- Large-scale operation handling

### 5. `test-config.ts`
Shared test utilities and configuration:
- Test constants and thresholds
- Utility functions for common operations
- Performance tracking utilities
- Security testing helpers
- Error matchers for common scenarios

## Running Tests

### Prerequisites
1. Start local Solana validator:
   ```bash
   solana-test-validator
   ```

2. Install dependencies:
   ```bash
   yarn install
   ```

### Test Commands

#### Basic Tests
```bash
# Run core functionality tests
yarn test:basic

# Run comprehensive integration tests
yarn test:comprehensive
```

#### Advanced Tests
```bash
# Run security tests
yarn test:security

# Run performance benchmarks
yarn test:performance
```

#### Test Suites
```bash
# Run all unit/integration tests
yarn test:unit

# Run all advanced tests (security + performance)
yarn test:advanced

# Run complete test suite
yarn test:all

# Use Anchor's built-in test runner
anchor test
```

## Test Coverage

### Instructions Tested
- ✅ `initialize_config` - System initialization
- ✅ `create_reward_mint` - $WEED token creation with metadata
- ✅ `init_user` - User account initialization with referrals
- ✅ `buy_facility` - Facility purchase and setup
- ✅ `claim_reward` - Time-based reward claiming
- ✅ `distribute_referral_reward` - Referral reward distribution
- ✅ `claim_referral_rewards` - Referral reward claiming
- ✅ `upgrade_facility` - Facility size upgrades
- ✅ `add_machine` - Machine addition to facilities
- ✅ `transfer_with_fee` - 2% fee token transfers
- ✅ `purchase_mystery_box` - Mystery box purchasing
- ✅ `open_mystery_box` - Mystery box opening and seed generation

### Security Aspects Tested
- ✅ Access control enforcement
- ✅ PDA ownership verification
- ✅ Input validation and overflow protection
- ✅ Token account ownership verification
- ✅ Unauthorized operation prevention
- ✅ Reentrancy protection
- ✅ Mystery box ownership constraints

### Performance Metrics
- ✅ Individual instruction timing
- ✅ Concurrent operation handling
- ✅ Large-scale stress testing
- ✅ Memory and compute efficiency
- ✅ Network throughput testing

## Expected Performance Thresholds

| Operation | Expected Time | Max Acceptable |
|-----------|---------------|----------------|
| User Init | < 1s | 3s |
| Facility Purchase | < 1s | 3s |
| Reward Claim | < 2s | 5s |
| Transfer w/ Fee | < 1s | 4s |
| Mystery Box Ops | < 3s | 5s |
| Concurrent Claims (5 users) | < 10s | 15s |

## Security Test Scenarios

### Access Control
- Non-admin config initialization attempts
- Cross-user reward claiming attempts
- Unauthorized facility upgrades
- Invalid PDA usage

### Input Validation
- Overflow/underflow protection
- Negative value handling
- Zero amount operations
- Large number processing

### Token Security
- Unauthorized minting attempts
- Token account ownership verification
- Transfer constraint enforcement

## Troubleshooting

### Common Issues

**Connection Refused Error**
```
Error: Connection refused (os error 61)
```
Solution: Start the local validator with `solana-test-validator`

**Insufficient SOL Error**
Solution: Ensure test accounts have sufficient SOL for transactions

**Timeout Errors**
Solution: Increase timeout values in test configuration or check network connectivity

**Metadata Program Errors**
Solution: These are often non-fatal in test environments and can be ignored

### Test Environment Setup

1. **Local Validator Configuration**:
   ```bash
   solana config set --url localhost
   solana-test-validator --reset
   ```

2. **Account Funding**:
   ```bash
   solana airdrop 5 <WALLET_ADDRESS>
   ```

3. **Program Deployment**:
   ```bash
   anchor build
   anchor deploy
   ```

## Contributing to Tests

When adding new features to the smart contract:

1. Add basic functionality tests to `comprehensive-tests.ts`
2. Add security tests to `security-tests.ts` 
3. Add performance benchmarks to `performance-tests.ts`
4. Update this README with new test coverage
5. Ensure all tests pass before submitting

### Test Writing Guidelines

- Use descriptive test names that explain the scenario
- Include both positive and negative test cases
- Add performance measurements for new operations
- Test edge cases and error conditions
- Use the utility functions from `test-config.ts`
- Follow the established test structure and naming conventions

## Performance Monitoring

The test suite automatically tracks and reports performance metrics:

- Average execution time per instruction
- Min/max execution times
- 95th and 99th percentile measurements
- Concurrent operation benchmarks
- Memory usage patterns

Results are displayed in a summary table after test completion.