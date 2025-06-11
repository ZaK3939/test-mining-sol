# Farm Game Test Suite

This directory contains a comprehensive, refactored test suite for the Farm Game Solana program. The tests are organized by type and purpose to maximize clarity, reduce redundancy, and ensure complete coverage.

## Test Structure

```
tests-new/
├── unit/                    # Unit tests for individual functions
│   ├── instructions/        # Test each instruction type
│   │   ├── admin.test.ts   # Admin-only functions (config, mint creation)
│   │   └── user.test.ts    # User management (init_user, buy_farm_space)
│   └── state/              # State validation tests (planned)
├── integration/            # Integration tests for complete workflows
│   └── user-journey.test.ts # Complete user flows and multi-user scenarios
├── e2e/                    # End-to-end tests
│   └── game-simulation.test.ts # Full game ecosystem simulation
├── security/               # Security-focused tests
│   └── access-control.test.ts # Authorization, PDA security, attack prevention
├── helpers/                # Shared utilities
│   ├── setup.ts           # Test environment setup and user creation
│   ├── assertions.ts      # Custom assertion functions
│   └── factories.ts       # Test data factories and scenarios
└── fixtures/               # Test data fixtures (planned)
```

## Test Categories

### Unit Tests (`unit/`)
- **Purpose**: Test individual functions and instructions in isolation
- **Coverage**: Each program instruction, state validation, edge cases
- **Files**:
  - `instructions/admin.test.ts`: Admin functions (initialize_config, create_reward_mint, etc.)
  - `instructions/user.test.ts`: User functions (init_user, buy_farm_space, etc.)

### Integration Tests (`integration/`)
- **Purpose**: Test complete user workflows and multi-user interactions
- **Coverage**: User journeys, referral chains, economic scenarios
- **Files**:
  - `user-journey.test.ts`: Complete user onboarding → farming → rewards cycle

### End-to-End Tests (`e2e/`)
- **Purpose**: Test the entire game ecosystem under realistic conditions
- **Coverage**: Viral growth simulation, economic sustainability, scalability
- **Files**:
  - `game-simulation.test.ts`: Full ecosystem simulation with 40+ users

### Security Tests (`security/`)
- **Purpose**: Validate security controls and prevent attack vectors
- **Coverage**: Access control, PDA security, unauthorized access attempts
- **Files**:
  - `access-control.test.ts`: Authorization validation and attack prevention

## Helper Utilities

### `helpers/setup.ts`
- `TestEnvironment`: Main test environment class
- `setupTestEnvironment()`: Quick environment setup with initialization
- `createUser()`: Factory for creating test users with all PDAs

### `helpers/assertions.ts`
- `assertTokenBalance()`: Verify token account balances
- `assertUserGrowPower()`: Check user grow power values
- `assertSupplyCapCompliance()`: Validate 120M WEED supply cap
- `assertEconomicConsistency()`: Verify economic state consistency

### `helpers/factories.ts`
- `TestScenarioFactory`: Factory class for creating complex test scenarios
- `createBasicUserScenario()`: User with farm space ready for testing
- `createReferralScenario()`: Referrer + referred user setup
- `createMultiUserScenario()`: Multiple users for stress testing
- `createReferralChain()`: Multi-level referral chains

## Running Tests

### Prerequisites
1. Local Solana validator running
2. Program deployed to local validator
3. Dependencies installed

### Commands
```bash
# Run all tests
anchor test --skip-local-validator

# Run specific test categories
npx ts-mocha tests-new/unit/**/*.test.ts
npx ts-mocha tests-new/integration/**/*.test.ts
npx ts-mocha tests-new/e2e/**/*.test.ts
npx ts-mocha tests-new/security/**/*.test.ts

# Run individual test files
npx ts-mocha tests-new/unit/instructions/admin.test.ts
npx ts-mocha tests-new/integration/user-journey.test.ts
npx ts-mocha tests-new/e2e/game-simulation.test.ts
```

## Test Features

### Comprehensive Coverage
- **Admin Functions**: Config initialization, mint creation, fee pool setup
- **User Management**: User initialization, farm space purchase, referral system
- **Economic System**: Reward claiming, mystery packs, supply cap enforcement
- **Security**: Access control, PDA validation, attack prevention
- **Game Mechanics**: Invite codes, referral chains, viral growth simulation

### Realistic Scenarios
- **Growth Simulation**: 1 founder → 40+ users through invite codes
- **Economic Strategies**: Conservative, aggressive, and balanced player types
- **Viral Mechanics**: Multi-wave user onboarding through referral chains
- **Sustainability**: Long-term economic model validation

### Quality Assurance
- **Supply Cap Compliance**: All tests verify 120M WEED token limit
- **Economic Consistency**: Global state validation across all operations
- **Security Validation**: Unauthorized access prevention
- **Performance Testing**: Multi-user concurrent operations

## Key Test Scenarios

### Basic User Journey
1. User initialization
2. Farm space purchase
3. Reward accumulation and claiming
4. Token balance verification

### Referral Chain Testing
1. Founder creates invite codes
2. Pioneers join via founder codes
3. Pioneers create their own codes
4. Community members join via pioneer codes
5. Referral reward distribution validation

### Security Testing
1. Unauthorized admin function access attempts
2. Cross-user account manipulation prevention
3. PDA spoofing attack prevention
4. Referral system abuse prevention

### Economic Simulation
1. Multi-phase community growth (Bootstrap → Viral → Diversification → Maturation)
2. Different player strategies and their outcomes
3. Mystery pack purchasing and strategy validation
4. Long-term sustainability and scalability analysis

## Migration from Old Tests

This refactored test suite consolidates and improves upon the previous test files:

**Replaced Files**:
- `farm-game.ts` (446 lines) → Distributed across unit and integration tests
- `strategic-user-journey.ts` (625 lines) → Consolidated into user-journey.test.ts
- `economic-simulation.test.ts` (454 lines) → Merged into game-simulation.test.ts
- `game-experience.test.ts` (595 lines) → Incorporated into user-journey.test.ts
- `progressive-invite-game.test.ts` (549 lines) → Enhanced in game-simulation.test.ts

**Improvements**:
- ✅ Eliminated ~1,500 lines of duplicate test code
- ✅ Improved test organization and discoverability
- ✅ Enhanced reusability through factory pattern
- ✅ Better separation of concerns (unit vs integration vs e2e)
- ✅ Comprehensive security testing addition
- ✅ Realistic end-to-end simulation scenarios

## Contributing

When adding new tests:

1. **Choose the right category**: Unit for isolated functions, Integration for workflows, E2E for ecosystem
2. **Use existing helpers**: Leverage setup.ts, assertions.ts, and factories.ts
3. **Follow naming conventions**: `*.test.ts` for all test files
4. **Add comprehensive logging**: Help developers understand test progression
5. **Validate economics**: Always check supply cap and state consistency
6. **Test security**: Consider attack vectors and unauthorized access

## Future Enhancements

- [ ] Add performance benchmarking tests
- [ ] Implement stress testing with 100+ concurrent users
- [ ] Add property-based testing for economic calculations
- [ ] Create visual test reporting and metrics dashboard
- [ ] Add automated test data generation and seeding