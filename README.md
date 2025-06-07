# Solana Facility Game MVP

An MVP (Minimum Viable Product) of a facility management game running on the Solana blockchain. Implemented using the Anchor framework.

## Main Features

### Phase 1: MVP Features
1. **Facility Purchase System** - One facility per user, automatic initial machine placement
2. **Grow Power & Lazy Calculation** - Reward accumulation system based on time elapsed
3. **Claim Rewards** - Receive rewards as SPL tokens
4. **Halving System** - Periodic reward rate reduction mechanism
5. **PDA Design** - Extensible account structure for future expansion

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

## Setup Instructions

### 1. Install Required Tools

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Node.js dependencies
npm install -g yarn
```

### 2. Initialize Project

```bash
# Create a new Anchor project
anchor init facility-game --no-git
cd facility-game

# Place the provided files in their corresponding locations
```

### 3. File Structure

```
facility-game/
├── programs/facility-game/src/
│   ├── lib.rs              # Main program
│   ├── state.rs            # Account structures
│   ├── instructions/
│   │   └── mod.rs          # Instruction contexts
│   └── error.rs            # Error definitions
├── tests/
│   └── facility-game.ts    # Test file
├── Anchor.toml             # Anchor configuration
├── Cargo.toml              # Rust dependencies
├── package.json            # Node.js configuration
└── tsconfig.json           # TypeScript configuration
```

### 4. Build and Test

```bash
# Install dependencies
yarn install

# Build program
anchor build

# Start local validator (in separate terminal)
solana-test-validator

# Run tests
anchor test --skip-local-validator
```

### 5. Deploy

```bash
# Switch to Devnet
solana config set --url devnet

# Airdrop (Devnet)
solana airdrop 2

# Deploy to Devnet
anchor deploy --provider.cluster devnet
```

## Usage Examples

### 1. System Initialization (Admin)

```bash
# Initialize Config (base rate: 10, halving period: 1 year)
anchor run initialize-config
```

### 2. User Flow

```typescript
// 1. Initialize user
await program.methods.initUser().accounts({...}).rpc();

// 2. Purchase facility
await program.methods.buyFacility().accounts({...}).rpc();

// 3. After time elapsed, claim rewards
await program.methods.claimReward().accounts({...}).rpc();
```

## Technical Specifications

### Reward Calculation Formula
```
Reward = (Elapsed Time[seconds] × Grow Power × base_rate) / 1000
```

### Halving
- `base_rate` halves at each configured interval
- Default: 1 year interval

### PDA Seeds
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- Facility: `["facility", user_pubkey]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`

## Future Expansion Plans

### Phase 2 Planned Features
- Multiple machine types
- Machine upgrade system
- Referral reward system
- Multiple facility ownership

### Extensible Design
- 64-byte `reserve` field in each account
- Secure account management through PDA structure
- Modular instruction design

## Troubleshooting

### Common Issues

1. **Build Error**: Check Anchor version
2. **Test Failure**: Ensure Solana Test Validator is running
3. **Deploy Error**: Check sufficient SOL balance

### Debugging Methods

```bash
# Check logs
solana logs

# Check account state
solana account <PDA_ADDRESS>

# Program logs
anchor test --skip-deploy -- --grep "test_name"
```

## License

MIT License# test-mining-sol
