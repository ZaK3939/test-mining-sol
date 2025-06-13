# Solana Facility Game - Complete Technical Specification

## 🎯 Program Overview

A sophisticated agricultural simulation game built on Solana featuring advanced economic mechanisms with halving reward systems, multi-level referral programs, and comprehensive storage management. This provides a complete agricultural ecosystem where users manage farm spaces, cultivate seeds, and earn WEED tokens through strategic optimization.

**Program ID**: `FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B`

### 🌟 Key Features

- **⚡ Instant Upgrades**: Eliminated 24-hour cooldowns - instant farm level upgrades
- **📦 Advanced Storage**: 2,000 total capacity with per-type limits and auto-discard
- **🔐 Hash-Based Invites**: Privacy-preserving invite system with operator/user tiers
- **💰 SPL Token 2022**: Built-in 2% transfer fees using Transfer Fee Extension
- **🤝 Multi-Level Referrals**: L1 (10%) and L2 (5%) referral reward system
- **🎲 Switchboard VRF**: Verifiable random seed generation with cryptographic proofs
- **🔬 Mathematical Precision**: Overflow protection with 128-bit arithmetic

## 📋 Table of Contents

- [🏗️ Core Architecture](#-core-architecture)
- [📊 Account Structure](#-account-structure)
- [🎮 Game Mechanics](#-game-mechanics)
- [💰 Economic System](#-economic-system)
- [🚀 Advanced Features](#-advanced-features)
- [📊 State Management](#-state-management)
- [🔒 Security & Validation](#-security--validation)
- [📚 Complete Instruction Set](#-complete-instruction-set)
- [🧪 Implementation Details](#-implementation-details)

## 🏗️ Core Architecture

### 🔧 Module Structure

```
programs/facility-game/src/
├── lib.rs                      # Main program entry (17 instructions)
├── state.rs                    # Account structures & data models
├── error.rs                    # Custom error definitions
├── constants.rs                # Centralized game constants
├── economics.rs                # Economic calculations & formulas
├── utils.rs                    # Helper functions & utilities
├── error_handling.rs           # Extended error handling
├── instructions/               # Instruction implementations
│   ├── mod.rs                  # Instruction module integration
│   ├── admin.rs                # Administrator instructions
│   ├── user.rs                 # User management instructions
│   ├── farm.rs                 # Farm management instructions
│   ├── referral.rs             # Referral reward instructions
│   ├── seeds.rs                # Seed system instructions
│   └── invite.rs               # Hash-based invite system
├── validation/                 # Business rule validation
│   ├── mod.rs                  # Validation module integration
│   ├── admin_validation.rs     # Admin permission validation
│   ├── economic_validation.rs  # Economic validation
│   ├── game_validation.rs      # Game logic validation
│   ├── time_validation.rs      # Time-related validation
│   ├── user_validation.rs      # User validation
│   └── common.rs               # Common validation functions
└── test_modules/               # Advanced test modules
    ├── economics_advanced_tests.rs
    ├── error_comprehensive_tests.rs
    └── state_advanced_tests.rs
```

### 🔑 Key Design Decisions

**PDA Seeds Pattern**:
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- FarmSpace: `["farm_space", user_pubkey]`
- SeedStorage: `["seed_storage", user_pubkey]`
- InviteCode: `["invite", inviter_pubkey, sha256(invite_code)]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`
- GlobalStats: `["global_stats"]`
- FeePool: `["fee_pool"]`

**SPL Token 2022 Integration**:
- Transfer Fee Extension with 2% automatic fees
- No custom transfer implementations needed
- Fees automatically collected to treasury
- Maximum fee cap: 1,000 WEED per transfer

**Instant Upgrade System**:
- Complete elimination of 24-hour cooldowns
- WEED token consumption for immediate level completion
- Upgrades: Lv1→2: 3,500 WEED, Lv2→3: 18,000 WEED
- Enhanced user experience and game flow

**Advanced Storage System**:
- Total capacity: 2,000 seeds (2x previous capacity)
- Per-type limits: 100 seeds per type
- Auto-discard functionality for overflow management
- Rent efficiency: maintained cost per seed ratio

**Hash-Based Invite System**:
- SHA256 hashing for privacy protection
- Operator invites: high-volume usage (1024 limit)
- User invites: 5 per user limit
- Privacy protection and spam prevention

## 📊 Account Structure

### 🔧 Config (Global Configuration)
```rust
pub struct Config {
    pub base_rate: u64,                    // Base reward rate (100 WEED/sec)
    pub halving_interval: i64,             // Halving interval (7 days)
    pub next_halving_time: i64,            // Next halving timestamp
    pub admin: Pubkey,                     // System administrator
    pub treasury: Pubkey,                  // Fee collection wallet
    pub seed_pack_cost: u64,               // Seed pack price (300 WEED)
    pub seed_counter: u64,                 // Global seed ID counter
    pub seed_pack_counter: u64,            // Global seed pack ID counter
    pub farm_space_cost_sol: u64,          // Farm purchase cost (0.5 SOL)
    pub max_invite_limit: u8,              // Max invite limit (5)
    pub trading_fee_percentage: u8,        // Trading fee (2%)
    pub protocol_referral_address: Pubkey, // Protocol referral address
    pub total_supply_minted: u64,          // Total WEED minted
    pub operator: Pubkey,                  // Operator address (unlimited invites)
    pub reserve: [u8; 2],                  // Future expansion
}
```

### 👤 UserState (User State)
```rust
pub struct UserState {
    pub owner: Pubkey,                     // User wallet
    pub total_grow_power: u64,             // Total grow power
    pub last_harvest_time: i64,            // Last reward claim time
    pub has_farm_space: bool,              // Farm ownership flag
    pub referrer: Option<Pubkey>,          // Referrer (multi-level rewards)
    pub pending_referral_rewards: u64,     // Unclaimed referral rewards
    pub reserve: [u8; 32],                 // Future expansion
}
```

### 🏭 FarmSpace (Farm Space)
```rust
pub struct FarmSpace {
    pub owner: Pubkey,                     // Farm owner
    pub level: u8,                         // Level (1-5)
    pub capacity: u8,                      // Capacity (4,8,12,16,20)
    pub seed_count: u8,                    // Planted seed count
    pub total_grow_power: u64,             // Farm total grow power
    pub reserve: [u8; 32],                 // Future expansion
}
```

### 📦 SeedStorage (Advanced Storage)
```rust
pub struct SeedStorage {
    pub owner: Pubkey,                     // Storage owner
    pub seed_ids: Vec<u64>,                // Seed ID list (max 2,000)
    pub total_seeds: u32,                  // Total seed count
    pub seed_type_counts: [u16; 9],        // Per-type counts (100 each limit)
    pub reserve: [u8; 16],                 // Future expansion
}

impl SeedStorage {
    pub const MAX_TOTAL_SEEDS: usize = 2_000;     // Total capacity
    pub const MAX_SEEDS_PER_TYPE: u16 = 100;      // Per-type limit
    
    // Capacity check (total limit)
    pub fn can_add_seed(&self) -> bool;
    
    // Capacity check (per-type limit)
    pub fn can_add_seed_type(&self, seed_type: &SeedType) -> bool;
    
    // Auto-discard function (on type limit reached)
    pub fn auto_discard_excess(&mut self, new_seed_type: &SeedType) -> Result<u16>;
}
```

### 🔐 InviteCode (Invite Code)
```rust
pub struct InviteCode {
    pub inviter: Pubkey,                   // Inviter
    pub invite_limit: u16,                 // Invite limit (operator 1024, user 5)
    pub invites_used: u16,                 // Used invites count
    pub creation_time: i64,                // Creation time
    pub reserve: [u8; 32],                 // Future expansion
}
```

## 🎮 Game Mechanics

### ⚡ Instant Farm Upgrade System

#### Upgrade Specifications
- **Instant Execution**: No cooldowns, immediate completion via WEED consumption
- **Level-based Costs**: 
  - Lv1→2: 3,500 WEED (capacity 4→8)
  - Lv2→3: 18,000 WEED (capacity 8→12)
  - Lv3→4: 20,000 WEED (capacity 12→16)
  - Lv4→5: 25,000 WEED (capacity 16→20)

#### Implementation Details
```rust
pub fn upgrade_farm_space(ctx: Context<UpgradeFarmSpace>) -> Result<()> {
    // Instant execution - no cooldown checks
    // WEED token consumption
    // Immediate level & capacity updates
}
```

### 📦 Advanced Storage System

#### Storage Specifications
- **Total Capacity**: 2,000 seeds (2x previous capacity)
- **Per-Type Limits**: 100 seeds per type maximum
- **Account Cost**: ~0.12 SOL (cost efficiency maintained)
- **Rent Recovery**: Individual rent recovery on seed discard

#### Auto-Discard Functionality
```rust
// Auto-discard logic when type limit reached
pub fn auto_discard_excess(&mut self, new_seed_type: &SeedType) -> Result<u16> {
    // Limit check
    // Identify lowest value seeds
    // Execute auto-discard
    // Update counters
}
```

#### Economic Analysis
- **Capacity Efficiency**: Same cost per seed, double capacity
- **Management Improvement**: Per-type limits enable balanced collections
- **Strategic Enhancement**: Rare seed prioritization, common seed auto-cleanup

### 🔐 Hash-Based Invite System

#### Hash-Based Design
```rust
// SHA256(invite_code + salt + inviter_pubkey)
pub fn create_invite_code(
    ctx: Context<CreateInviteCode>, 
    invite_code: [u8; 8]
) -> Result<()>;

pub fn use_invite_code(
    ctx: Context<UseInviteCode>, 
    invite_code: [u8; 8],
    inviter_pubkey: Pubkey
) -> Result<()>;
```

#### Invite Patterns
1. **Operator Invites**:
   - Limit: 1024 uses (high-volume marketing)
   - Purpose: Marketing campaigns, events, initial user acquisition
   - Management: Issued from operator address

2. **User Invites**:
   - Limit: 5 uses
   - Purpose: Friend invitations, community growth
   - Management: Individual user issuance

#### Privacy Protection
- Invite codes are hashed, plain text never stored
- Requires inviter address, prevents brute force attacks
- PDA-based duplicate prevention

## 💰 Economic System

### 💎 Reward Distribution System

#### Proportional Distribution Formula
```
User Reward = (User Grow Power / Total Grow Power) × Base Rate × Elapsed Time
```

#### Halving Mechanism
- **Interval**: Every 7 days (customizable)
- **Effect**: 50% reduction in base rate
- **Purpose**: Inflation control, long-term value preservation

### 🤝 Multi-Level Referral System

#### Referral Reward Rates
- **Level 1 (Direct Invites)**: 10%
- **Level 2 (Indirect Invites)**: 5%
- **Maximum Depth**: 2 levels

#### Implementation Details
```rust
pub fn claim_reward_with_referral_rewards(
    ctx: Context<ClaimRewardWithReferralRewards>
) -> Result<()> {
    // Base reward claim
    // L1: base_reward × 10%
    // L2: base_reward × 5%
    // Protocol address exclusion
    // Integrated processing for all at once
}
```

### 💸 SPL Token 2022 Fee System

#### Transfer Fee Extension
- **Rate**: 2.00% (200 basis points)
- **Maximum Fee**: 1,000 WEED per transfer
- **Collection**: Automatic to treasury via SPL Token 2022
- **Configuration Authority**: mint_authority PDA

#### Fee Flow
1. User transfers → 2% fee automatically collected
2. Treasury accumulation via SPL Token 2022
3. No custom transfer logic needed
4. Fees can be withdrawn by treasury authority

## 🚀 Advanced Features

### 🌱 Seed System

#### Seed Types and Rarity
```rust
pub enum SeedType {
    Seed1 = 0,  // 100GP (42.23%)
    Seed2 = 1,  // 180GP (24.44%)
    Seed3 = 2,  // 420GP (13.33%)
    Seed4 = 3,  // 720GP (8.33%)
    Seed5 = 4,  // 1000GP (5.56%)
    Seed6 = 5,  // 5000GP (3.33%)
    Seed7 = 6,  // 15000GP (1.33%)
    Seed8 = 7,  // 30000GP (0.89%)
    Seed9 = 8,  // 60000GP (0.56%)
}
```

#### Mystery Packs
- **Cost**: 300 WEED + VRF fees (~0.002 SOL)
- **Expected Value**: 1,226.79 grow power
- **Efficiency**: 4.09 GP/WEED
- **Maximum Purchase**: 100 packs per transaction

### 🎲 Switchboard VRF Integration

#### Verifiable Randomness
- **Cryptographic Proofs**: Third-party oracle verification
- **Transparent Probability**: On-chain verification of seed rarities
- **Anti-Manipulation**: Commit-reveal pattern prevents gaming
- **Quality Assurance**: True randomness vs. pseudo-random

#### VRF Cost Structure
- **WEED Burn**: 300 WEED × quantity
- **VRF Fee**: ~0.002 SOL (2,000,000 lamports)
  - Base transaction fees: 5,000 × 15 transactions = 75,000 lamports
  - Storage rent: 2,400 lamports
  - Oracle processing: 2,000,000 lamports
  - Total: ~2,077,400 lamports

## 📚 Complete Instruction Set

### 👨‍💼 Administrator Instructions (5)
1. `initialize_config` - System configuration initialization
2. `create_reward_mint` - WEED token mint creation (SPL Token 2022 + Transfer Fee)
3. `initialize_global_stats` - Global statistics initialization
4. `initialize_fee_pool` - Fee pool initialization
5. `update_config` - System configuration updates

### 👤 User Management Instructions (1)
6. `init_user` - User account initialization

### 🏭 Farm Management Instructions (2)
7. `buy_farm_space` - Farm space purchase (0.5 SOL)
8. `upgrade_farm_space` - **Instant farm upgrades**

### 💰 Reward System Instructions (3)
9. `claim_reward_with_referral_rewards` - **Integrated reward claim** (farm + referral)
10. `accumulate_referral_reward` - Referral reward accumulation (internal)
11. `view_pending_referral_rewards` - Unclaimed referral rewards view

### 🔐 Invite System Instructions (2)
12. `create_invite_code` - **Hash-based invite code creation**
13. `use_invite_code` - **Hash-based invite code usage**

### 🌱 Seed System Instructions (7)
14. `initialize_seed_storage` - **Advanced storage initialization**
15. `purchase_seed_pack` - Mystery pack purchase (Switchboard VRF)
16. `open_seed_pack` - Pack opening with verifiable randomness
17. `plant_seed` - Seed planting
18. `remove_seed` - Seed removal
19. `discard_seed` - Seed discard (rent recovery)
20. `batch_discard_seeds` - Batch seed discard (efficient rent recovery)

## 🔒 Security & Validation

### 🛡️ Security Design Principles

#### PDA-Based Access Control
- All accounts managed via PDAs
- Signer-based ownership verification
- Cross-program call restrictions

#### Numerical Overflow Prevention
```rust
// checked_add, checked_mul usage throughout
// u64/u128 for large capacity calculations
// Comprehensive overflow protection
```

#### Comprehensive Error Handling
- 100+ detailed error definitions
- Business logic validation
- Economic constraint checks

### 🧪 Testing Framework

#### Integration Tests
- Complete user journey testing
- Operator/user invite pattern verification
- Storage system functionality testing
- Error case coverage testing

#### Unit Tests
- Individual module functionality tests
- Economic calculation accuracy verification
- Security boundary testing

## 🎉 Summary

Solana Facility Game is an advanced agricultural simulation game featuring **instant upgrades**, **advanced storage systems**, **hash-based invite systems**, and **SPL Token 2022 integration** as core components.

### Key Technical Achievements
1. **User Experience Enhancement**: Instant upgrades via 24-hour cooldown elimination
2. **Scalability**: 2,000 seed storage capacity with per-type auto-management
3. **Privacy Protection**: Hash-based invite system with cryptographic security
4. **Economic Simplification**: SPL Token 2022 Transfer Fee Extension eliminates custom fee logic
5. **Verifiable Fairness**: Switchboard VRF integration for true randomness
6. **Extensibility**: Modular design supporting future feature additions

This implementation provides users with a comfortable gaming experience while ensuring long-term economic stability and community growth for the protocol.

### Technology Stack
- **Solana**: High-performance blockchain for real-time gaming
- **Anchor Framework**: Type-safe Rust development
- **SPL Token 2022**: Advanced token features with built-in fees
- **Switchboard VRF**: Verifiable random functions
- **SHA256**: Cryptographic hashing for privacy

The simplified architecture eliminates complex custom transfer logic while maintaining all economic incentives through battle-tested SPL Token standards.