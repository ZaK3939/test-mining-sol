# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Development
```bash
# Build the Solana program
anchor build

# Run tests with local validator
anchor test

# Run tests without starting local validator (faster if validator already running)
anchor test --skip-local-validator

# Deploy to configured cluster
anchor deploy

# Deploy to specific cluster
anchor deploy --provider.cluster devnet

# Lint TypeScript/JavaScript files
yarn lint

# Auto-fix linting issues
yarn lint:fix

# Install dependencies
yarn install
```

### Testing Single Tests
```bash
# Run specific test by name
anchor test --skip-deploy -- --grep "test_name"

# Run tests with Mocha directly
yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts
```

### Local Development Setup
```bash
# Start local validator (separate terminal)
solana-test-validator

# Configure for local development
solana config set --url localhost

# Airdrop SOL (devnet)
solana airdrop 2
```

## Architecture Overview

### Program Structure
The Solana program follows Anchor framework patterns with clear separation:

1. **lib.rs** - Main program entry with 5 core instructions:
   - `initialize_config` - System setup by admin
   - `create_reward_mint` - SPL token mint creation
   - `init_user` - User account initialization
   - `buy_facility` - Facility purchase with initial machine
   - `claim_reward` - Time-based reward calculation and minting

2. **state.rs** - Account structures with PDA design:
   - Config (global settings)
   - UserState (user data and facility ownership)
   - Facility (facility details and grow power)
   - All accounts include 64-byte reserve for future expansion

3. **instructions/mod.rs** - Instruction contexts with account validation
4. **error.rs** - Custom error definitions

### Key Architectural Decisions

**PDA Seeds Pattern**:
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- Facility: `["facility", user_pubkey]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`

**Reward System**:
- Lazy evaluation: rewards calculated only on claim
- Formula: `(elapsed_time × grow_power × base_rate) / 1000`
- Automatic halving mechanism reduces base_rate periodically
- SPL token minting with PDA mint authority

**Security Patterns**:
- All user actions require signature
- PDAs ensure deterministic account addresses
- Overflow protection with checked arithmetic
- Constraint validation in account contexts

### Testing Approach
Tests follow user journey:
1. Initialize system configuration
2. Create reward mint
3. Initialize user
4. Buy facility
5. Create token account
6. Claim rewards after time passes
7. Test error cases (e.g., duplicate facility purchase)

### Important Constants
- Initial grow power: 100
- Default halving interval: 1 year (31536000 seconds)
- Program ID: `EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89`