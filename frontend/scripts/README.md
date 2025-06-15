# Facility Game Test Scripts

This directory contains test scripts for the Facility Game Solana program.

## Available Scripts

### 1. `test-deployment.ts` - Full Deployment Test
Complete end-to-end test script for deployed programs. Tests all major functionality including:
- Config initialization
- Reward mint creation
- User initialization with invite codes
- Farm space purchase
- Reward claiming
- Token transfers

**Usage:**
```bash
# Set admin private key (base58 encoded)
export ADMIN_PRIVATE_KEY="your-private-key"

# Optional: Set custom RPC endpoint (defaults to devnet)
export RPC_URL="https://api.devnet.solana.com"

# Run the test
bun run scripts/test-deployment.ts
```

### 2. `test-deployment-simple.ts` - Test Instructions
Displays comprehensive testing instructions and checklist for manual testing.

**Usage:**
```bash
bun run scripts/test-deployment-simple.ts
```

### 3. `example.ts` - Local Development Examples
Example interactions with the program for local development.

**Usage:**
```bash
bun run scripts/example.ts [journey|farming|referral]
```

### 4. `initialize.ts` & `initialize-config.ts`
Helper scripts for program initialization during development.

## Testing Checklist

- [ ] **Program Deployment**
  - [ ] Program deployed to target network
  - [ ] Program ID matches configuration
  - [ ] Admin wallet has authority

- [ ] **Initial Setup (Admin Only)**
  - [ ] Config initialized with parameters
  - [ ] Reward mint created (Token-2022)
  - [ ] Global stats initialized
  - [ ] Probability table initialized

- [ ] **User Flow**
  - [ ] Invite code created successfully
  - [ ] User initialized with invite code
  - [ ] Referral relationship established
  - [ ] Farm space purchased (0.5 SOL)

- [ ] **Game Mechanics**
  - [ ] Seeds planted in farm space
  - [ ] Grow power calculated correctly
  - [ ] Rewards accumulate over time
  - [ ] Rewards claimed successfully

- [ ] **Token System**
  - [ ] WEED tokens minted correctly
  - [ ] Transfer fees applied (2%)
  - [ ] Token balances accurate
  - [ ] Associated token accounts created

- [ ] **Referral System**
  - [ ] Level 1 referrals (10%) distributed
  - [ ] Level 2 referrals (5%) distributed
  - [ ] Pending rewards tracked
  - [ ] Referral claims work

## Common Issues & Solutions

### "Account already in use"
Admin setup has already been completed. This is expected for existing deployments.

### "Insufficient balance"
Ensure wallets have enough SOL:
- Admin: ~0.5 SOL for setup
- Users: ~1 SOL for farm purchase + fees

### "Invalid invite code"
Invite codes must be exactly 8 alphanumeric characters (A-Z, a-z, 0-9).

### "User not initialized"
Users must be initialized with an invite code before they can interact with the game.

## Environment Variables

- `ADMIN_PRIVATE_KEY`: Base58 encoded private key of admin wallet
- `RPC_URL`: Solana RPC endpoint (defaults to devnet)

## Network Endpoints

- **Devnet**: https://api.devnet.solana.com
- **Testnet**: https://api.testnet.solana.com
- **Mainnet**: https://api.mainnet-beta.solana.com

## Program Details

- **Program ID**: GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc
- **Framework**: Anchor 0.31.1
- **Token Standard**: SPL Token-2022 with Transfer Fee Extension
- **Language**: Rust (on-chain), TypeScript (client)