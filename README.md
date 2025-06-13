# Solana Facility Game

An agricultural simulation game built on Solana with time-based rewards, facility management, and SPL token economics.

## 📚 Documentation

- **[System Overview](./docs/overview.md)** - Complete technical specification
- **[Probability Management](./docs/PROBABILITY_TABLE_MANAGEMENT.md)** - Dynamic probability table updates
- **[Farm Level Design](./docs/FARM_LEVEL_EXTENSION_DESIGN.md)** - Farm level extension design
- **[Invite System](./docs/SIMPLIFIED_INVITE_SYSTEM.md)** - Simplified invite system
- **[All Documentation](./docs/README.md)** - Full documentation index

## Core Features

1. **Facility Purchase** - One facility per user with automatic machine placement
2. **Time-based Rewards** - Lazy calculation system based on elapsed time and grow power
3. **SPL Token Claims** - Mint and claim rewards as SPL tokens
4. **Halving Mechanism** - Periodic reward rate reduction
5. **Extensible Architecture** - PDA design with reserved space for future features

## Architecture

### Account Structure

- **Config**: Global settings (base rate, halving settings)
- **UserState**: User state (PDA: `["user", user_pubkey]`)
- **Facility**: Facility information (PDA: `["facility", user_pubkey]`)
- **RewardMint**: Reward token mint (PDA: `["reward_mint"]`)

### Instructions

1. `initialize_config` - Initialize system settings
2. `create_reward_mint` - Create reward token mint
3. `init_user` - Initialize user account
4. `buy_facility` - Purchase facility + initial machine placement
5. `claim_reward` - Claim rewards (time calculation + token mint)

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) and Yarn

### Development Setup

```bash
# Install dependencies
yarn install

# Build program
anchor build

# Start local validator (separate terminal)
solana-test-validator

# Run tests
anchor test --skip-local-validator
```

### Deploy to Devnet

```bash
# Configure for devnet
solana config set --url devnet
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet
```

## Technical Details

### Reward Formula
```
Reward = (Elapsed Time × Grow Power × Base Rate) / 1000
```

### PDA Seeds
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- Facility: `["facility", user_pubkey]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`

### Key Constants
- Initial grow power: 100
- Default halving interval: 1 year
- Program ID: `EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89`

## Development

### Common Commands
```bash
# Build program
anchor build

# Run all tests
anchor test

# Run tests without validator restart
anchor test --skip-local-validator

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Check program logs
solana logs
```

### Testing
Tests follow the complete user journey from system initialization through reward claims. See `/tests/` for comprehensive test examples.

## License

MIT License
