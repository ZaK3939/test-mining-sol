use anchor_lang::prelude::*;

/// Global system configuration
/// Stores all game parameters and admin settings
#[account]
pub struct Config {
    /// Base reward rate in tokens per second (default: 100 WEED/sec)
    pub base_rate: u64,
    /// Halving mechanism interval in seconds (default: 6 days)
    pub halving_interval: i64,
    /// Next scheduled halving timestamp
    pub next_halving_time: i64,
    /// System administrator public key
    pub admin: Pubkey,
    /// Treasury wallet for fee collection and SOL payments
    pub treasury: Pubkey,
    /// Mystery seed pack cost in base token units (300 WEED)
    pub seed_pack_cost: u64,
    /// Global seed counter for unique seed IDs
    pub seed_counter: u64,
    /// Global seed pack counter for unique pack IDs
    pub seed_pack_counter: u64,
    /// Farm space purchase cost in lamports (0.5 SOL)
    pub farm_space_cost_sol: u64,
    /// Maximum allowed invites per user (default: 5)
    pub max_invite_limit: u8,
    /// Trading fee as percentage (default: 2%)
    pub trading_fee_percentage: u8,
    /// Protocol address that doesn't receive referral rewards
    pub protocol_referral_address: Pubkey,
    /// Total amount of WEED tokens minted so far
    pub total_supply_minted: u64,
    /// Reserved bytes for future upgrades (reduced from 2 to accommodate total_supply_minted)
    pub reserve: [u8; 2],
}

/// Individual user account state
/// Tracks user progress and referral relationships
#[account]
pub struct UserState {
    /// User's wallet public key
    pub owner: Pubkey,
    /// Sum of all grow power from user's planted seeds
    pub total_grow_power: u64,
    /// Timestamp of last reward claim
    pub last_harvest_time: i64,
    /// Farm space ownership flag
    pub has_farm_space: bool,
    /// Optional referrer for multi-level referral system
    pub referrer: Option<Pubkey>,
    /// Unclaimed referral commission tokens
    pub pending_referral_rewards: u64,
    /// Reserved bytes for future features
    pub reserve: [u8; 32],
}

/// Farm space account for seed cultivation
/// Manages capacity, upgrades, and seed placement
#[account]
pub struct FarmSpace {
    /// Farm space owner's public key
    pub owner: Pubkey,
    /// Current level (1-5, affects capacity)
    pub level: u8,
    /// Maximum seeds that can be planted at current level
    pub capacity: u8,
    /// Number of seeds currently planted
    pub seed_count: u8,
    /// Combined grow power of all planted seeds
    pub total_grow_power: u64,
    /// Upgrade initiation timestamp (0 = not upgrading)
    pub upgrade_start_time: i64,
    /// Level to reach after 24h cooldown
    pub upgrade_target_level: u8,
    /// Reserved bytes for future expansion
    pub reserve: [u8; 32],
}

impl Config {
    pub const LEN: usize = 8 + // discriminator
        8 + // base_rate
        8 + // halving_interval
        8 + // next_halving_time
        32 + // admin
        32 + // treasury
        8 + // seed_pack_cost
        8 + // seed_counter
        8 + // seed_pack_counter
        8 + // farm_space_cost_sol
        1 + // max_invite_limit
        1 + // trading_fee_percentage
        32 + // protocol_referral_address
        8 + // total_supply_minted
        2; // reserve (kept for backward compatibility)
        
    /// Default farm space cost (0.5 SOL in lamports)
    pub const DEFAULT_FARM_SPACE_COST: u64 = 500_000_000; // 0.5 SOL
    
    /// Default base rate (100 WEED per second)
    pub const DEFAULT_BASE_RATE: u64 = 100;
    
    /// Default halving interval (6 days in seconds)
    pub const DEFAULT_HALVING_INTERVAL: i64 = 6 * 24 * 60 * 60;
}

impl UserState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        8 + // total_grow_power
        8 + // last_harvest_time
        1 + // has_farm_space
        1 + 32 + // referrer (Option<Pubkey>: 1 byte discriminator + 32 bytes for Some(Pubkey))
        8 + // pending_referral_rewards
        32; // reserve
}

impl FarmSpace {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 + // level
        1 + // capacity
        1 + // seed_count
        8 + // total_grow_power
        8 + // upgrade_start_time
        1 + // upgrade_target_level
        32; // reserve
    
    /// Upgrade cooldown duration (24 hours in seconds)
    pub const UPGRADE_COOLDOWN: i64 = 24 * 60 * 60;
        
    /// Get capacity for a given level
    pub fn get_capacity_for_level(level: u8) -> u8 {
        match level {
            1 => 4,
            2 => 8,
            3 => 12,
            4 => 16,
            5 => 20,
            _ => 20, // Maximum capacity
        }
    }
    
    /// Get upgrade cost for next level
    pub fn get_upgrade_cost(current_level: u8) -> Option<u64> {
        match current_level {
            1 => Some(3500),   // Level 1→2: 3500 $WEED
            2 => Some(18000),  // Level 2→3: 18000 $WEED
            3 => Some(20000),  // Level 3→4: 20000 $WEED
            4 => Some(25000),  // Level 4→5: 25000 $WEED
            _ => None,         // No upgrade available
        }
    }
    
    /// Check if upgrade is complete
    pub fn is_upgrade_complete(&self, current_time: i64) -> bool {
        if self.upgrade_start_time == 0 {
            return false;
        }
        current_time >= self.upgrade_start_time + Self::UPGRADE_COOLDOWN
    }
}

/// Seed types with fixed grow power and probabilities
#[derive(Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum SeedType {
    Seed1,      // Grow Power: 100 - 42.23%
    Seed2,      // Grow Power: 180 - 24.44%
    Seed3,      // Grow Power: 420 - 13.33%
    Seed4,      // Grow Power: 720 - 8.33%
    Seed5,      // Grow Power: 1000 - 5.56%
    Seed6,      // Grow Power: 5000 - 3.33%
    Seed7,      // Grow Power: 15000 - 1.33%
    Seed8,      // Grow Power: 30000 - 0.89%
    Seed9,      // Grow Power: 60000 - 0.56%
}

impl SeedType {
    /// Optimized lookup table for grow power values
    const GROW_POWERS: [u64; 9] = [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000];
    
    /// Cumulative probability thresholds for weighted random selection
    /// Values out of 10000 for precise percentage control
    const PROBABILITY_THRESHOLDS: [u16; 9] = [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000];
    
    /// Get grow power efficiently using array lookup
    pub fn get_grow_power(&self) -> u64 {
        Self::GROW_POWERS[*self as usize]
    }
    
    /// Convert random value to seed type using weighted probabilities
    /// Uses linear search for small array - more efficient than binary search for 9 elements
    pub fn from_random(random: u64) -> Self {
        let value = (random % 10000) as u16;
        
        for (i, &threshold) in Self::PROBABILITY_THRESHOLDS.iter().enumerate() {
            if value < threshold {
                return unsafe { std::mem::transmute(i as u8) };
            }
        }
        
        // Fallback to highest rarity (should never happen with proper thresholds)
        SeedType::Seed9
    }
    
    /// Iterator-friendly array of all seed types
    pub const fn all_types() -> [SeedType; 9] {
        [
            SeedType::Seed1, SeedType::Seed2, SeedType::Seed3,
            SeedType::Seed4, SeedType::Seed5, SeedType::Seed6,
            SeedType::Seed7, SeedType::Seed8, SeedType::Seed9,
        ]
    }
    
    /// Get display probability as percentage (for UI/documentation)
    pub fn get_probability_percent(&self) -> f32 {
        const PROBABILITIES: [f32; 9] = [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56];
        PROBABILITIES[*self as usize]
    }
    
    /// Validate seed type enum value
    pub fn is_valid(value: u8) -> bool {
        value < 9
    }
}

/// Seed account (stored in user's storage)
#[account]
pub struct Seed {
    /// Seed owner
    pub owner: Pubkey,
    /// Seed type (determines grow power)
    pub seed_type: SeedType,
    /// Grow power of this seed
    pub grow_power: u64,
    /// Whether this seed is currently planted in farm space
    pub is_planted: bool,
    /// Farm space where this seed is planted (if any)
    pub planted_farm_space: Option<Pubkey>,
    /// Creation timestamp
    pub created_at: i64,
    /// Unique seed ID
    pub seed_id: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

/// Seed Pack purchase tracker
#[account]
pub struct SeedPack {
    /// Purchaser
    pub purchaser: Pubkey,
    /// Purchase timestamp
    pub purchased_at: i64,
    /// Pack cost paid
    pub cost_paid: u64,
    /// Whether pack has been opened
    pub is_opened: bool,
    /// Pyth Entropy sequence number (for tracking the random request)
    pub entropy_sequence: u64,
    /// User-provided entropy seed (for additional randomness)
    pub user_entropy_seed: u64,
    /// Final random value (set after opening)
    pub final_random_value: u64,
    /// Pack ID
    pub pack_id: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 16],
}

impl Seed {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 + // seed_type (enum)
        8 + // grow_power
        1 + // is_planted
        1 + 32 + // planted_farm_space (Option<Pubkey>)
        8 + // created_at
        8 + // seed_id
        32; // reserve
}

impl SeedPack {
    pub const LEN: usize = 8 + // discriminator
        32 + // purchaser
        8 + // purchased_at
        8 + // cost_paid
        1 + // is_opened
        8 + // entropy_sequence
        8 + // user_entropy_seed
        8 + // final_random_value
        8 + // pack_id
        16; // reserve
}

/// Invite code management PDA
#[account]
pub struct InviteCode {
    /// Invite code creator (user who can invite others)
    pub inviter: Pubkey,
    /// Current number of people invited
    pub invites_used: u8,
    /// Maximum invite limit for this user
    pub invite_limit: u8,
    /// Invite code string (8 characters)
    pub code: [u8; 8],
    /// Creation timestamp
    pub created_at: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

/// Global statistics PDA
#[account]
pub struct GlobalStats {
    /// Total grow power across all users
    pub total_grow_power: u64,
    /// Total number of active farm spaces
    pub total_farm_spaces: u64,
    /// Total $WEED supply (1 billion)
    pub total_supply: u64,
    /// Current rewards per second (decreases with halving)
    pub current_rewards_per_second: u64,
    /// Last update timestamp
    pub last_update_time: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

/// Fee collection pool PDA
#[account]
pub struct FeePool {
    /// Accumulated trading fees in $WEED
    pub accumulated_fees: u64,
    /// Treasury multisig/address for SOL conversion
    pub treasury_address: Pubkey,
    /// Last fee collection timestamp
    pub last_collection_time: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 48],
}

/// User's seed inventory management
/// Tracks all seeds owned by a user
#[account]
pub struct SeedStorage {
    /// Storage owner's public key
    pub owner: Pubkey,
    /// Dynamic array of seed IDs (max 100 for rent optimization)
    pub seed_ids: Vec<u64>,
    /// Current seed count for quick access
    pub total_seeds: u16,
    /// Reserved bytes for future features
    pub reserve: [u8; 32],
}

impl SeedStorage {
    /// Maximum seeds per user to prevent excessive rent costs
    pub const MAX_SEEDS: usize = 100;
    
    /// Check if storage has capacity for more seeds
    pub fn can_add_seed(&self) -> bool {
        self.seed_ids.len() < Self::MAX_SEEDS
    }
    
    /// Add a new seed ID to storage
    pub fn add_seed(&mut self, seed_id: u64) -> Result<()> {
        require!(self.can_add_seed(), crate::error::GameError::StorageFull);
        self.seed_ids.push(seed_id);
        self.total_seeds = self.seed_ids.len() as u16;
        Ok(())
    }
    
    /// Remove seed ID from storage
    pub fn remove_seed(&mut self, seed_id: u64) -> bool {
        if let Some(pos) = self.seed_ids.iter().position(|&x| x == seed_id) {
            self.seed_ids.remove(pos);
            self.total_seeds = self.seed_ids.len() as u16;
            true
        } else {
            false
        }
    }
}

/// Individual reward accumulation PDA per user
#[account]
pub struct RewardAccount {
    /// User who can claim these rewards
    pub user: Pubkey,
    /// Accumulated claimable rewards
    pub claimable_amount: u64,
    /// Last harvest timestamp
    pub last_harvest_time: i64,
    /// Referral rewards (level 1: 10%, level 2: 5%)
    pub referral_rewards_l1: u64,
    pub referral_rewards_l2: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

impl InviteCode {
    pub const LEN: usize = 8 + // discriminator
        32 + // inviter
        1 + // invites_used
        1 + // invite_limit
        8 + // code
        8 + // created_at
        32; // reserve
}

impl GlobalStats {
    pub const LEN: usize = 8 + // discriminator
        8 + // total_grow_power
        8 + // total_farm_spaces
        8 + // total_supply
        8 + // current_rewards_per_second
        8 + // last_update_time
        32; // reserve
        
    /// Initial total supply (1 billion WEED)
    pub const INITIAL_TOTAL_SUPPLY: u64 = 1_000_000_000 * 1_000_000; // 1B WEED with 6 decimals
}

impl FeePool {
    pub const LEN: usize = 8 + // discriminator
        8 + // accumulated_fees
        32 + // treasury_address
        8 + // last_collection_time
        48; // reserve
}

impl SeedStorage {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + (8 * 100) + // seed_ids (Vec<u64> with max 100 seeds)
        2 + // total_seeds
        32; // reserve
}

impl RewardAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // claimable_amount
        8 + // last_harvest_time
        8 + // referral_rewards_l1
        8 + // referral_rewards_l2
        32; // reserve
}