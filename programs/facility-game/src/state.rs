use anchor_lang::prelude::*;
use crate::error::*;

// VRF専用にするため、RandomnessMethodは削除
// 全てSwitchboard VRFで統一

/// Global system configuration
/// Stores all game parameters and admin settings
#[account]
pub struct Config {
    /// Base reward rate in tokens per second (default: 100 WEED/sec)
    pub base_rate: u64,
    /// Halving mechanism interval in seconds (default: 200 seconds)
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
    /// Operator address with unlimited invite privileges
    pub operator: Pubkey,
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
    /// Total number of seed packs purchased by this user (for farm auto-upgrade)
    pub total_packs_purchased: u32,
    /// Reserved bytes for future features (reduced from 32 to 28 to accommodate total_packs_purchased)
    pub reserve: [u8; 28],
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
    /// Reserved bytes for future expansion
    pub reserve: [u8; 32],
}

/// Dynamic probability table for seed generation
/// Allows admin to update probabilities without code changes
#[account]
pub struct ProbabilityTable {
    /// Table version number for tracking updates
    pub version: u32,
    /// Number of seed types in this table (6 for Table 1, 9 for Table 2)
    pub seed_count: u8,
    /// Grow power values for each seed type (max 9 types)
    pub grow_powers: [u64; 9],
    /// Probability thresholds for cumulative distribution (max 9 types)
    pub probability_thresholds: [u16; 9],
    /// Human-readable probability percentages (max 9 types)
    pub probability_percentages: [f32; 9],
    /// Expected value of a pack with this table
    pub expected_value: u64,
    /// Table name/description
    pub name: [u8; 32],
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

impl ProbabilityTable {
    pub const LEN: usize = 8 + // discriminator
        4 + // version
        1 + // seed_count
        72 + // grow_powers (9 * 8)
        18 + // probability_thresholds (9 * 2)
        36 + // probability_percentages (9 * 4)
        8 + // expected_value
        32 + // name
        8 + // created_at
        8 + // updated_at
        32; // reserve

    /// Initialize with Table 1 settings (6 seeds)
    pub fn init_table_1() -> Self {
        let mut table = Self {
            version: 1,
            seed_count: 6,
            grow_powers: [0; 9],
            probability_thresholds: [0; 9],
            probability_percentages: [0.0; 9],
            expected_value: 0,
            name: [0; 32],
            created_at: 0,
            updated_at: 0,
            reserve: [0; 32],
        };
        
        // Table 1 settings
        table.grow_powers[0..6].copy_from_slice(&[100, 180, 420, 720, 1000, 5000]);
        table.probability_thresholds[0..6].copy_from_slice(&[4300, 6800, 8200, 9100, 9700, 10000]);
        table.probability_percentages[0..6].copy_from_slice(&[43.0, 25.0, 14.0, 9.0, 6.0, 3.0]);
        table.expected_value = 421; // Calculated expected value
        
        // Set name "Table1"
        let name_bytes = b"Table1";
        table.name[0..name_bytes.len()].copy_from_slice(name_bytes);
        
        table
    }
    
    /// Get active grow powers (only up to seed_count)
    pub fn get_active_grow_powers(&self) -> &[u64] {
        &self.grow_powers[0..self.seed_count as usize]
    }
    
    /// Get active probability thresholds (only up to seed_count)
    pub fn get_active_thresholds(&self) -> &[u16] {
        &self.probability_thresholds[0..self.seed_count as usize]
    }
    
    /// Get active probability percentages (only up to seed_count)
    pub fn get_active_percentages(&self) -> &[f32] {
        &self.probability_percentages[0..self.seed_count as usize]
    }
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
        32 + // operator
        2; // reserve (kept for backward compatibility)
        
    /// Default farm space cost (0.5 SOL in lamports)
    pub const DEFAULT_FARM_SPACE_COST: u64 = 500_000_000; // 0.5 SOL
    
    /// Default base rate (100 WEED per second)
    pub const DEFAULT_BASE_RATE: u64 = 100;
    
    /// Default halving interval (200 seconds)
    pub const DEFAULT_HALVING_INTERVAL: i64 = 200;
}

impl UserState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        8 + // total_grow_power
        8 + // last_harvest_time
        1 + // has_farm_space
        1 + 32 + // referrer (Option<Pubkey>: 1 byte discriminator + 32 bytes for Some(Pubkey))
        8 + // pending_referral_rewards
        4 + // total_packs_purchased
        28; // reserve (reduced from 32)
        
    /// Increment pack purchase count and return if farm upgrade is needed
    pub fn increment_pack_purchases(&mut self, quantity: u32) -> bool {
        let old_total = self.total_packs_purchased;
        self.total_packs_purchased = self.total_packs_purchased.saturating_add(quantity);
        
        // Check if the pack purchase crossed an upgrade threshold
        let old_level = FarmSpace::calculate_level_from_packs(old_total);
        let new_level = FarmSpace::calculate_level_from_packs(self.total_packs_purchased);
        
        new_level > old_level
    }
}

impl FarmSpace {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 + // level
        1 + // capacity
        1 + // seed_count
        8 + // total_grow_power
        32; // reserve
    
        
    /// Get capacity for a given level using constants
    pub fn get_capacity_for_level(level: u8) -> u8 {
        use crate::constants::FARM_CAPACITIES;
        if level == 0 || level > FARM_CAPACITIES.len() as u8 {
            return FARM_CAPACITIES[FARM_CAPACITIES.len() - 1]; // Max capacity
        }
        FARM_CAPACITIES[(level - 1) as usize]
    }
    
    /// Get legacy upgrade cost for next level (deprecated - now uses auto-upgrade)
    pub fn get_legacy_upgrade_cost(current_level: u8) -> Option<u64> {
        use crate::constants::LEGACY_UPGRADE_COSTS;
        if current_level == 0 || current_level > LEGACY_UPGRADE_COSTS.len() as u8 {
            return None;
        }
        Some(LEGACY_UPGRADE_COSTS[(current_level - 1) as usize])
    }
    
    /// Calculate required farm level based on cumulative pack purchases
    pub fn calculate_level_from_packs(total_packs: u32) -> u8 {
        use crate::constants::FARM_UPGRADE_THRESHOLDS;
        
        for (i, &threshold) in FARM_UPGRADE_THRESHOLDS.iter().enumerate().rev() {
            if total_packs >= threshold {
                return (i + 1) as u8;
            }
        }
        1 // Default to level 1
    }
    
    /// Update farm space level and capacity based on pack purchases
    pub fn auto_upgrade(&mut self, total_packs: u32) -> bool {
        let new_level = Self::calculate_level_from_packs(total_packs);
        if new_level > self.level {
            self.level = new_level;
            self.capacity = Self::get_capacity_for_level(new_level);
            return true; // Upgrade occurred
        }
        false // No upgrade needed
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
    
    /// Create SeedType from index (for dynamic probability tables)
    pub fn from_index(index: u8) -> Result<SeedType> {
        match index {
            0 => Ok(SeedType::Seed1),
            1 => Ok(SeedType::Seed2),
            2 => Ok(SeedType::Seed3),
            3 => Ok(SeedType::Seed4),
            4 => Ok(SeedType::Seed5),
            5 => Ok(SeedType::Seed6),
            6 => Ok(SeedType::Seed7),
            7 => Ok(SeedType::Seed8),
            8 => Ok(SeedType::Seed9),
            _ => Err(crate::error::GameError::InvalidConfig.into()),
        }
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
    /// Pack cost paid in WEED tokens
    pub cost_paid: u64,
    /// VRF fee paid in SOL lamports (0 for Solana native)
    pub vrf_fee_paid: u64,
    /// Whether pack has been opened
    pub is_opened: bool,
    /// VRF sequence number (for tracking the VRF request)
    pub vrf_sequence: u64,
    /// User-provided entropy seed (for additional randomness)
    pub user_entropy_seed: u64,
    /// Final random value (set after opening)
    pub final_random_value: u64,
    /// Pack ID
    pub pack_id: u64,
    /// Switchboard VRF account
    pub vrf_account: Pubkey,
    /// Reserved for future expansion
    pub reserve: [u8; 8],
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
        8 + // vrf_fee_paid
        1 + // is_opened
        8 + // vrf_sequence
        8 + // user_entropy_seed
        8 + // final_random_value
        8 + // pack_id
        32 + // vrf_account (Pubkey)
        8; // reserve
}

/// Invite code management PDA (Hash-based for privacy)
#[account]
pub struct InviteCode {
    /// Invite code creator (user who can invite others)
    pub inviter: Pubkey,
    /// Current number of people invited
    pub invites_used: u16,
    /// Maximum invite limit for this user
    pub invite_limit: u16,
    /// Hash of the invite code (SHA256(code + salt))
    pub code_hash: [u8; 32],
    /// Random salt for hash security
    pub salt: [u8; 16],
    /// Index in the inviter's code list
    pub code_index: u16,
    /// Creation timestamp
    pub created_at: i64,
    /// Whether this code is active
    pub is_active: bool,
    /// Whether this code was created by an operator (for privilege validation)
    pub created_as_operator: bool,
    /// Reserved for future expansion
    pub reserve: [u8; 14],
}

impl InviteCode {
    pub const LEN: usize = 8 + 32 + 2 + 2 + 32 + 16 + 2 + 8 + 1 + 1 + 14;
}

/// Single-use secret invite code PDA (Operator-only, hash-based)
#[account]
pub struct SingleUseSecretInvite {
    /// Creator (operator only)
    pub creator: Pubkey,
    /// Hash of the invite code (SHA256(code + salt))
    pub code_hash: [u8; 32],
    /// Random salt for hash security
    pub salt: [u8; 16],
    /// Whether this code has been used
    pub is_used: bool,
    /// User who used this code
    pub used_by: Option<Pubkey>,
    /// Creation timestamp
    pub created_at: i64,
    /// Usage timestamp
    pub used_at: Option<i64>,
    /// Campaign ID for tracking
    pub campaign_id: [u8; 8],
    /// Reserved for future expansion
    pub reserve: [u8; 16],
}

impl SingleUseSecretInvite {
    pub const LEN: usize = 8 + 32 + 32 + 16 + 1 + 1 + 32 + 8 + 1 + 8 + 8 + 16;
}

/// Inviter code registry PDA (manages multiple codes per inviter)
#[account]
pub struct InviterCodeRegistry {
    /// Inviter pubkey
    pub inviter: Pubkey,
    /// Total number of codes created by this inviter
    pub total_codes_created: u16,
    /// Number of currently active codes
    pub active_codes_count: u16,
    /// Total invites used across all codes
    pub total_invites_used: u32,
    /// Last code creation timestamp
    pub last_code_created_at: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

impl InviterCodeRegistry {
    pub const LEN: usize = 8 + 32 + 2 + 2 + 4 + 8 + 32;
}

/// Global statistics PDA
#[account]
pub struct GlobalStats {
    /// Total grow power across all users
    pub total_grow_power: u64,
    /// Total number of active farm spaces
    pub total_farm_spaces: u64,
    /// Total $WEED supply (240 million)
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
/// Tracks all seeds owned by a user with type-based limits
#[account]
pub struct SeedStorage {
    /// Storage owner's public key
    pub owner: Pubkey,
    /// Dynamic array of seed IDs (max 2000 total)
    pub seed_ids: Vec<u64>,
    /// Current seed count for quick access
    pub total_seeds: u32,
    /// Count of each seed type (9 types, max 100 each)
    pub seed_type_counts: [u16; 9],
    /// Reserved bytes for future features
    pub reserve: [u8; 16], // Reduced due to seed_type_counts addition
}

impl SeedStorage {
    /// Maximum total seeds per user (2000)
    /// Supports up to 400 mystery packs worth of seeds
    /// Rent cost: ~0.12 SOL for the entire account
    pub const MAX_TOTAL_SEEDS: usize = 2_000;
    
    /// Maximum seeds per type (100 each)
    /// Prevents hoarding of specific seed types
    pub const MAX_SEEDS_PER_TYPE: u16 = 100;
    
    /// Check if storage has capacity for more seeds (total limit)
    pub fn can_add_seed(&self) -> bool {
        self.seed_ids.len() < Self::MAX_TOTAL_SEEDS
    }
    
    /// Check if specific seed type has capacity
    pub fn can_add_seed_type(&self, seed_type: &SeedType) -> bool {
        let type_index = *seed_type as usize;
        self.seed_type_counts[type_index] < Self::MAX_SEEDS_PER_TYPE
    }
    
    /// Check if we can add a seed (both total and type limits)
    pub fn can_add_seed_with_type(&self, seed_type: &SeedType) -> bool {
        self.can_add_seed() && self.can_add_seed_type(seed_type)
    }
    
    /// Add a new seed ID to storage with type tracking
    pub fn add_seed(&mut self, seed_id: u64, seed_type: &SeedType) -> Result<()> {
        require!(self.can_add_seed(), crate::error::GameError::StorageFull);
        require!(self.can_add_seed_type(seed_type), crate::error::GameError::SeedTypeLimitReached);
        
        self.seed_ids.push(seed_id);
        self.total_seeds = self.seed_ids.len() as u32;
        
        // Update type count
        let type_index = *seed_type as usize;
        self.seed_type_counts[type_index] += 1;
        
        Ok(())
    }
    
    /// Remove seed ID from storage with type tracking
    pub fn remove_seed(&mut self, seed_id: u64, seed_type: &SeedType) -> bool {
        if let Some(pos) = self.seed_ids.iter().position(|&x| x == seed_id) {
            self.seed_ids.remove(pos);
            self.total_seeds = self.seed_ids.len() as u32;
            
            // Update type count
            let type_index = *seed_type as usize;
            if self.seed_type_counts[type_index] > 0 {
                self.seed_type_counts[type_index] -= 1;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Auto-discard lowest value seeds when type limit is reached
    /// Returns the number of seeds discarded
    pub fn auto_discard_excess(&mut self, new_seed_type: &SeedType) -> Result<u16> {
        let type_index = *new_seed_type as usize;
        
        // If we're not at the limit, no need to discard
        if self.seed_type_counts[type_index] < Self::MAX_SEEDS_PER_TYPE {
            return Ok(0);
        }
        
        // Find the lowest value seed of this type to discard
        // For now, we'll discard one seed to make room
        // In a full implementation, you'd want to track individual seed values
        if self.seed_type_counts[type_index] > 0 {
            self.seed_type_counts[type_index] -= 1;
            // Note: In practice, you'd also remove the specific seed_id
            // and properly handle the seed account closure
            return Ok(1);
        }
        
        Ok(0)
    }
    
    /// Get count of specific seed type
    pub fn get_seed_type_count(&self, seed_type: &SeedType) -> u16 {
        let type_index = *seed_type as usize;
        self.seed_type_counts[type_index]
    }
    
    /// Get remaining capacity for specific seed type
    pub fn get_remaining_capacity(&self, seed_type: &SeedType) -> u16 {
        let type_index = *seed_type as usize;
        Self::MAX_SEEDS_PER_TYPE.saturating_sub(self.seed_type_counts[type_index])
    }
    
    /// Initialize seed type counts (for existing accounts)
    pub fn initialize_type_counts(&mut self) {
        self.seed_type_counts = [0; 9];
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


impl GlobalStats {
    pub const LEN: usize = 8 + // discriminator
        8 + // total_grow_power
        8 + // total_farm_spaces
        8 + // total_supply
        8 + // current_rewards_per_second
        8 + // last_update_time
        32; // reserve
        
    /// Initial total supply (240 million WEED)
    pub const INITIAL_TOTAL_SUPPLY: u64 = 240_000_000 * 1_000_000; // 240M WEED with 6 decimals
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
        4 + (8 * 2_000) + // seed_ids (Vec<u64> with max 2,000 seeds)
        4 + // total_seeds (u32 for 2,000+ seeds)
        (2 * 9) + // seed_type_counts (9 x u16)
        16; // reserve
        // Total: 16,076 bytes (~16KB) - Affordable, rent ~0.12 SOL
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

/// Dynamic farm level configuration
/// Allows flexible addition of new farm levels
#[account]
pub struct FarmLevelConfig {
    /// Maximum supported level (1-20)
    pub max_level: u8,
    /// Capacity for each level (dynamic array)
    pub capacities: Vec<u8>,
    /// Pack purchase thresholds for each level (dynamic array)
    pub upgrade_thresholds: Vec<u32>,
    /// Optional names for each level (dynamic array)
    pub level_names: Vec<String>,
    /// Configuration creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

impl FarmLevelConfig {
    // Variable size account - calculate based on actual content
    pub const BASE_LEN: usize = 8 + // discriminator
        1 + // max_level
        4 + // capacities vec header
        4 + // upgrade_thresholds vec header  
        4 + // level_names vec header
        8 + // created_at
        8 + // updated_at
        32; // reserve
        
    /// Calculate space needed for a specific configuration
    pub fn calculate_space(max_level: u8, level_names: &[String]) -> usize {
        let capacities_space = max_level as usize; // Vec<u8>
        let thresholds_space = max_level as usize * 4; // Vec<u32>
        let names_space = level_names.iter().map(|s| 4 + s.len()).sum::<usize>(); // Vec<String>
        
        Self::BASE_LEN + capacities_space + thresholds_space + names_space
    }
    
    /// Get default 5-level configuration space
    pub const DEFAULT_SPACE: usize = Self::BASE_LEN + 
        5 + // 5 levels (u8)
        20 + // 5 thresholds (u32)
        (4 + 12) * 5; // 5 names (~12 chars each with length prefix)
}

impl FarmSpace {
    /// Get capacity for level using dynamic configuration
    pub fn get_capacity_for_level_with_config(level: u8, config: &FarmLevelConfig) -> Result<u8> {
        require!(level >= 1 && level <= config.max_level, GameError::InvalidFarmLevel);
        Ok(config.capacities[(level - 1) as usize])
    }
    
    /// Calculate level from pack purchases using dynamic configuration
    pub fn calculate_level_from_packs_with_config(total_packs: u32, config: &FarmLevelConfig) -> u8 {
        for (i, &threshold) in config.upgrade_thresholds.iter().enumerate().rev() {
            if total_packs >= threshold {
                return (i + 1) as u8;
            }
        }
        1
    }
    
    /// Auto-upgrade using dynamic configuration
    pub fn auto_upgrade_with_config(&mut self, total_packs: u32, config: &FarmLevelConfig) -> Result<bool> {
        let new_level = Self::calculate_level_from_packs_with_config(total_packs, config);
        if new_level > self.level && new_level <= config.max_level {
            self.level = new_level;
            self.capacity = Self::get_capacity_for_level_with_config(new_level, config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}