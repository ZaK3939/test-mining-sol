use anchor_lang::prelude::*;

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

impl Config {
    /// Account data size calculation
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
        2; // reserve

    /// Default base rate for reward calculations
    pub const DEFAULT_BASE_RATE: u64 = crate::constants::DEFAULT_BASE_RATE;
    
    /// Default halving interval
    pub const DEFAULT_HALVING_INTERVAL: i64 = crate::constants::DEFAULT_HALVING_INTERVAL;
    
    /// Default farm space cost in lamports (0.5 SOL)
    pub const DEFAULT_FARM_SPACE_COST: u64 = crate::constants::FARM_SPACE_COST_SOL;
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
    /// Current level (1-5 currently, up to 10 in future, affects capacity)
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

/// Seed types with dynamic grow power and probabilities
/// Initially 8 types are known, additional types are secret until revealed
#[derive(Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum SeedType {
    Seed1,      // Known: Basic seed type
    Seed2,      // Known: Basic seed type
    Seed3,      // Known: Basic seed type
    Seed4,      // Known: Basic seed type
    Seed5,      // Known: Basic seed type
    Seed6,      // Known: Basic seed type
    Seed7,      // Known: Basic seed type
    Seed8,      // Known: Basic seed type
    Seed9,      // Secret: Values hidden until revealed
    Seed10,     // Secret: Values hidden until revealed
    Seed11,     // Secret: Values hidden until revealed
    Seed12,     // Secret: Values hidden until revealed
    Seed13,     // Secret: Values hidden until revealed
    Seed14,     // Secret: Values hidden until revealed
    Seed15,     // Secret: Values hidden until revealed
    Seed16,     // Secret: Values hidden until revealed
}

/// Dynamic probability table for seed generation
/// Supports up to 16 seed types for extensive variety and frequent updates
/// Includes secret seed management for hidden seed types
#[account]
pub struct ProbabilityTable {
    /// Table version number for tracking frequent updates
    pub version: u32,
    /// Number of seed types in this table (1-16 for maximum flexibility)
    pub seed_count: u8,
    /// Number of revealed seed types (initially 8, can be increased)
    pub revealed_seed_count: u8,
    /// Grow power values for each seed type (max 16 types)
    /// Hidden values for secret seeds are encrypted or set to 0
    pub grow_powers: [u64; 16],
    /// Probability thresholds for cumulative distribution (max 16 types)
    pub probability_thresholds: [u16; 16],
    /// Human-readable probability percentages (max 16 types)
    /// Hidden values for secret seeds are set to 0
    pub probability_percentages: [f32; 16],
    /// Expected value of a pack with this table
    pub expected_value: u64,
    /// Table name/description (e.g., "SpringEvent2024", "RarityFest")
    pub name: [u8; 32],
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Table category for organization (e.g., "seasonal", "premium", "event")
    pub category: [u8; 16],
    /// Bitfield indicating which seed types are revealed (bit 0 = Seed1, bit 1 = Seed2, etc.)
    pub revealed_seeds_mask: u16,
    /// Reserved for future expansion
    pub reserve: [u8; 14],
}

impl ProbabilityTable {
    pub const LEN: usize = 8 + // discriminator
        4 + // version
        1 + // seed_count
        1 + // revealed_seed_count
        128 + // grow_powers (16 * 8)
        32 + // probability_thresholds (16 * 2)
        64 + // probability_percentages (16 * 4)
        8 + // expected_value
        32 + // name
        8 + // created_at
        8 + // updated_at
        16 + // category
        2 + // revealed_seeds_mask
        14; // reserve

    /// Initialize with standard 8-seed table (initially revealed)
    pub fn init_standard_table() -> Self {
        let mut table = Self {
            version: 1,
            seed_count: 8,              // Start with 8 seeds
            revealed_seed_count: 8,     // All 8 are initially revealed
            grow_powers: [0; 16],
            probability_thresholds: [0; 16],
            probability_percentages: [0.0; 16],
            expected_value: 0,
            name: [0; 32],
            created_at: 0,
            updated_at: 0,
            category: [0; 16],
            revealed_seeds_mask: 0xFF,  // First 8 bits set (seeds 1-8 revealed)
            reserve: [0; 14],
        };
        
        // Standard 8-seed settings (known values)
        table.grow_powers[0..8].copy_from_slice(&[100, 180, 420, 720, 1000, 5000, 15000, 30000]);
        table.probability_thresholds[0..8].copy_from_slice(&[3000, 5500, 7200, 8300, 9000, 9400, 9700, 10000]);
        table.probability_percentages[0..8].copy_from_slice(&[30.0, 25.0, 17.0, 11.0, 7.0, 4.0, 3.0, 3.0]);
        table.expected_value = 1590; // Calculated expected value for 8 seeds
        
        // Secret seeds (9-16) remain hidden with 0 values
        // They will be revealed later through admin updates
        
        // Set name "StandardTable8"
        let name_bytes = b"StandardTable8";
        table.name[0..name_bytes.len()].copy_from_slice(name_bytes);
        
        // Set category "standard"
        let category_bytes = b"standard";
        table.category[0..category_bytes.len()].copy_from_slice(category_bytes);
        
        table
    }
    
    /// Initialize with enhanced table (admin-only, reveals all secret seeds)
    pub fn init_enhanced_table() -> Self {
        let mut table = Self {
            version: 1,
            seed_count: 16,
            revealed_seed_count: 16,    // All seeds revealed for admin table
            grow_powers: [
                100, 180, 420, 720, 1000, 5000, 15000, 30000,     // Original 8 types
                60000, 120000, 250000, 500000, 1000000, 2000000, 4000000, 10000000 // Extended 8 types
            ],
            probability_thresholds: [
                2500, 4500, 6200, 7500, 8400, 9000, 9350, 9550,  // First 8
                9700, 9800, 9870, 9920, 9955, 9975, 9990, 10000  // Last 8
            ],
            probability_percentages: [
                25.0, 20.0, 17.0, 13.0, 9.0, 6.0, 3.5, 2.0,      // First 8
                1.5, 1.0, 0.7, 0.5, 0.35, 0.2, 0.15, 0.1         // Last 8
            ],
            expected_value: 425,
            name: [0; 32],
            created_at: 0,
            updated_at: 0,
            category: [0; 16],
            revealed_seeds_mask: 0xFFFF, // All 16 bits set (all seeds revealed)
            reserve: [0; 14],
        };
        
        // Set name "EnhancedTable"
        let name_bytes = b"EnhancedTable";
        table.name[0..name_bytes.len()].copy_from_slice(name_bytes);
        
        // Set category "premium"
        let category_bytes = b"premium";
        table.category[0..category_bytes.len()].copy_from_slice(category_bytes);
        
        table
    }
    
    /// Legacy compatibility method
    pub fn init_table_1() -> Self {
        Self::init_standard_table()
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
    
    /// Check if a seed type is revealed (visible to users)
    pub fn is_seed_revealed(&self, seed_index: u8) -> bool {
        if seed_index >= 16 {
            return false;
        }
        (self.revealed_seeds_mask & (1 << seed_index)) != 0
    }
    
    /// Reveal a new seed type (admin only)
    pub fn reveal_seed(&mut self, seed_index: u8, grow_power: u64, probability_percentage: f32) -> bool {
        if seed_index >= 16 || self.is_seed_revealed(seed_index) {
            return false;
        }
        
        // Set the grow power and probability (threshold will be recalculated)
        self.grow_powers[seed_index as usize] = grow_power;
        self.probability_percentages[seed_index as usize] = probability_percentage;
        
        // Mark as revealed
        self.revealed_seeds_mask |= 1 << seed_index;
        self.revealed_seed_count += 1;
        
        true
    }
    
    /// Update values for an already revealed seed type (admin only)
    pub fn update_seed_values(&mut self, seed_index: u8, grow_power: u64, probability_percentage: f32) -> bool {
        if seed_index >= 16 || !self.is_seed_revealed(seed_index) {
            return false;
        }
        
        // Update the grow power and probability
        self.grow_powers[seed_index as usize] = grow_power;
        self.probability_percentages[seed_index as usize] = probability_percentage;
        
        true
    }
    
    /// Get grow power only if seed is revealed, otherwise return 0
    pub fn get_revealed_grow_power(&self, seed_index: u8) -> u64 {
        if self.is_seed_revealed(seed_index) && seed_index < 16 {
            self.grow_powers[seed_index as usize]
        } else {
            0 // Hidden seed value
        }
    }
    
    /// Get probability only if seed is revealed, otherwise return 0
    pub fn get_revealed_probability(&self, seed_index: u8) -> f32 {
        if self.is_seed_revealed(seed_index) && seed_index < 16 {
            self.probability_percentages[seed_index as usize]
        } else {
            0.0 // Hidden seed value
        }
    }
    
    /// Get active grow powers (only revealed seeds)
    pub fn get_revealed_grow_powers(&self) -> Vec<u64> {
        (0..self.seed_count)
            .map(|i| self.get_revealed_grow_power(i))
            .collect()
    }
    
    /// Get active probability percentages (only revealed seeds)
    pub fn get_revealed_percentages(&self) -> Vec<f32> {
        (0..self.seed_count)
            .map(|i| self.get_revealed_probability(i))
            .collect()
    }
    
    /// Validate table configuration
    pub fn validate(&self) -> bool {
        // Check that seed_count is within limits
        if self.seed_count == 0 || self.seed_count > 16 {
            return false;
        }
        
        // Check that revealed_seed_count is within limits
        if self.revealed_seed_count > self.seed_count {
            return false;
        }
        
        // Check that last threshold is 10000 (100%)
        let last_index = (self.seed_count - 1) as usize;
        if self.probability_thresholds[last_index] != 10000 {
            return false;
        }
        
        // Check that thresholds are in ascending order
        for i in 1..self.seed_count as usize {
            if self.probability_thresholds[i] <= self.probability_thresholds[i - 1] {
                return false;
            }
        }
        
        true
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

/// User's seed inventory management
/// Tracks all seeds owned by a user with type-based limits (16 seed types)
#[account]
pub struct SeedStorage {
    /// Storage owner's public key
    pub owner: Pubkey,
    /// Dynamic array of seed IDs (max 2000 total)
    pub seed_ids: Vec<u64>,
    /// Corresponding seed types for each ID (same indexing as seed_ids)
    pub seed_types: Vec<SeedType>,
    /// Current seed count for quick access
    pub total_seeds: u32,
    /// Count of each seed type (16 types, max 100 each)
    pub seed_type_counts: [u16; 16],
    /// Reserved bytes for future features
    pub reserve: [u8; 8],
}

impl FarmSpace {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 + // level
        1 + // capacity
        1 + // seed_count
        8 + // total_grow_power
        32; // reserve
        
    /// Get capacity for a given level
    pub fn get_capacity_for_level(level: u8) -> u8 {
        use crate::constants::FARM_CAPACITIES;
        if level == 0 || level > FARM_CAPACITIES.len() as u8 {
            return FARM_CAPACITIES[FARM_CAPACITIES.len() - 1]; // Max capacity
        }
        FARM_CAPACITIES[(level - 1) as usize]
    }
}

impl SeedStorage {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + (8 * 2_000) + // seed_ids (Vec<u64> with max 2,000 seeds)
        4 + (1 * 2_000) + // seed_types (Vec<SeedType> with max 2,000 entries)
        4 + // total_seeds (u32 for 2,000+ seeds)
        (2 * 16) + // seed_type_counts (16 x u16)
        8; // reserve
        // Total: 18,094 bytes (~18KB) - Still affordable, rent ~0.13 SOL
        
    /// Maximum total seeds per user (2000)
    pub const MAX_TOTAL_SEEDS: usize = 2_000;
    
    /// Maximum seeds per type (100 each)
    pub const MAX_SEEDS_PER_TYPE: u16 = 100;
    
    /// Check if storage has capacity for more seeds (total limit)
    pub fn can_add_seed(&self) -> bool {
        self.seed_ids.len() < Self::MAX_TOTAL_SEEDS
    }
    
    /// Check if specific seed type has capacity
    pub fn can_add_seed_type(&self, seed_type: &SeedType) -> bool {
        let type_index = *seed_type as usize;
        if type_index >= 16 {
            return false;
        }
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
        
        // Add seed ID and corresponding type (maintaining same indexing)
        self.seed_ids.push(seed_id);
        self.seed_types.push(*seed_type);
        self.total_seeds = self.seed_ids.len() as u32;
        
        // Update type count
        let type_index = *seed_type as usize;
        if type_index < 16 {
            self.seed_type_counts[type_index] += 1;
        }
        
        Ok(())
    }
    
    /// Remove seed ID from storage with type tracking
    pub fn remove_seed(&mut self, seed_id: u64, seed_type: &SeedType) -> bool {
        if let Some(pos) = self.seed_ids.iter().position(|&x| x == seed_id) {
            // Remove both the seed ID and its corresponding type
            self.seed_ids.remove(pos);
            self.seed_types.remove(pos);
            self.total_seeds = self.seed_ids.len() as u32;
            
            // Update type count
            let type_index = *seed_type as usize;
            if type_index < 16 && self.seed_type_counts[type_index] > 0 {
                self.seed_type_counts[type_index] -= 1;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Get count of specific seed type
    pub fn get_seed_type_count(&self, seed_type: &SeedType) -> u16 {
        let type_index = *seed_type as usize;
        if type_index < 16 {
            self.seed_type_counts[type_index]
        } else {
            0
        }
    }
    
    /// Get remaining capacity for specific seed type
    pub fn get_remaining_capacity(&self, seed_type: &SeedType) -> u16 {
        let type_index = *seed_type as usize;
        if type_index < 16 {
            Self::MAX_SEEDS_PER_TYPE.saturating_sub(self.seed_type_counts[type_index])
        } else {
            0
        }
    }
    
    /// Initialize seed type counts and ensure vectors are empty
    pub fn initialize_type_counts(&mut self) {
        self.seed_type_counts = [0; 16];
        self.seed_ids.clear();
        self.seed_types.clear();
        self.total_seeds = 0;
    }
    
    /// Auto-discard excess seeds if over limit
    /// Strategy: Remove oldest seeds first (FIFO - First In, First Out)
    pub fn auto_discard_excess(&mut self, seed_type: &SeedType) -> Result<()> {
        let type_index = *seed_type as usize;
        if type_index >= 16 {
            return Ok(()); // Invalid seed type, skip
        }
        
        // Check if we need to discard due to type limit
        if self.seed_type_counts[type_index] >= Self::MAX_SEEDS_PER_TYPE {
            // Find and remove the oldest seed of this specific type
            if let Some(oldest_seed_id) = self.find_oldest_seed_of_type(seed_type) {
                self.remove_seed(oldest_seed_id, seed_type);
                msg!("Auto-discarded oldest seed ID {} of type {:?} due to type limit", 
                     oldest_seed_id, seed_type);
            }
        }
        
        // Check if we need to discard due to total limit
        while self.seed_ids.len() >= Self::MAX_TOTAL_SEEDS {
            if !self.seed_ids.is_empty() && !self.seed_types.is_empty() {
                // Get the oldest seed (first in vector) and its type
                let oldest_seed_id = self.seed_ids[0];
                let oldest_seed_type = self.seed_types[0];
                
                // Remove using proper method to maintain consistency
                self.remove_seed(oldest_seed_id, &oldest_seed_type);
                
                msg!("Auto-discarded oldest seed ID {} of type {:?} due to total storage limit", 
                     oldest_seed_id, oldest_seed_type);
            } else {
                break; // No more seeds to remove
            }
        }
        
        Ok(())
    }
    
    /// Find the oldest seed ID of a specific type
    /// Uses parallel indexing of seed_ids and seed_types to find the first occurrence
    /// FIFO: First seed of the specified type in the vector is the oldest
    fn find_oldest_seed_of_type(&self, target_type: &SeedType) -> Option<u64> {
        let target_index = *target_type as usize;
        if target_index >= 16 || self.seed_type_counts[target_index] == 0 {
            return None;
        }
        
        // Find the first (oldest) seed of the specified type
        // Since we maintain parallel vectors, we can iterate through both simultaneously
        for (i, &seed_type) in self.seed_types.iter().enumerate() {
            if seed_type == *target_type {
                // Found the oldest seed of this type
                return Some(self.seed_ids[i]);
            }
        }
        
        None // No seeds of this type found (shouldn't happen if count > 0)
    }
}

impl UserState {
    /// Auto-upgrade farm if eligible based on pack purchases
    /// This method checks if the user is eligible for a farm upgrade based on their total pack purchases
    /// Note: This method only determines eligibility - the actual upgrade should be done via FarmSpace::auto_upgrade
    pub fn auto_upgrade_if_eligible(&mut self, total_packs_purchased: u32) -> Result<bool> {
        use crate::constants::FARM_UPGRADE_THRESHOLDS;
        
        // Update our internal pack count
        self.total_packs_purchased = total_packs_purchased;
        
        // Check if user has a farm space
        if !self.has_farm_space {
            return Ok(false); // Cannot upgrade without a farm space
        }
        
        // Check if we've crossed any upgrade threshold
        for &threshold in FARM_UPGRADE_THRESHOLDS.iter().skip(1) { // Skip level 1 (threshold 0)
            if total_packs_purchased >= threshold {
                // User is eligible for upgrade to this level
                // The actual upgrade logic should be handled by FarmSpace::auto_upgrade
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Increment pack purchases and return if upgrade is needed
    pub fn increment_pack_purchases(&mut self, quantity: u32) -> bool {
        let old_total = self.total_packs_purchased;
        self.total_packs_purchased += quantity;
        
        // Check if we crossed any upgrade threshold
        use crate::constants::FARM_UPGRADE_THRESHOLDS;
        
        // Find the highest threshold we crossed
        for &threshold in FARM_UPGRADE_THRESHOLDS.iter().rev() {
            if old_total < threshold && self.total_packs_purchased >= threshold {
                return true;
            }
        }
        
        false
    }
}

impl FarmSpace {
    /// Auto-upgrade farm if eligible based on pack purchases
    pub fn auto_upgrade(&mut self, total_packs_purchased: u32) -> Result<bool> {
        use crate::constants::FARM_UPGRADE_THRESHOLDS;
        
        // Check if already at max level (currently level 5, future expansion to 10)
        if self.level >= 5 {
            return Ok(false);
        }
        
        // Get the next level's threshold
        let next_threshold = FARM_UPGRADE_THRESHOLDS[self.level as usize];
        
        // Check if we've reached the threshold for upgrade
        if total_packs_purchased >= next_threshold {
            // Upgrade to next level
            self.level += 1;
            self.capacity = Self::get_capacity_for_level(self.level);
            
            msg!("Farm auto-upgraded to level {} (capacity: {}) after {} total packs purchased", 
                 self.level, self.capacity, total_packs_purchased);
            
            return Ok(true);
        }
        
        Ok(false)
    }
}

/// Global statistics tracking for the entire game ecosystem
#[account]
pub struct GlobalStats {
    /// Total grow power across all farms
    pub total_grow_power: u64,
    /// Total number of farm spaces created
    pub total_farm_spaces: u64,
    /// Total WEED tokens minted
    pub total_supply: u64,
    /// Current effective rewards per second (after halving)
    pub current_rewards_per_second: u64,
    /// Last time statistics were updated
    pub last_update_time: i64,
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
        
    /// Initial total supply (placeholder value)
    pub const INITIAL_TOTAL_SUPPLY: u64 = 0;
}

/// Fee pool for collecting and managing trading fees
#[account]
pub struct FeePool {
    /// Accumulated fees in lamports
    pub accumulated_fees: u64,
    /// Treasury address for fee withdrawal
    pub treasury_address: Pubkey,
    /// Last time fees were collected
    pub last_collection_time: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 48],
}

impl FeePool {
    pub const LEN: usize = 8 + // discriminator
        8 + // accumulated_fees
        32 + // treasury_address
        8 + // last_collection_time
        48; // reserve
}

/// Individual seed account for planted seeds
#[account]
pub struct Seed {
    /// Seed unique ID
    pub seed_id: u64,
    /// Seed type (determines grow power)
    pub seed_type: SeedType,
    /// Owner of this seed
    pub owner: Pubkey,
    /// Grow power value for this seed
    pub grow_power: u64,
    /// When this seed was planted
    pub planted_at: i64,
    /// Whether this seed is currently planted in a farm
    pub is_planted: bool,
    /// Which farm space this seed is planted in (if any)
    pub planted_farm_space: Option<Pubkey>,
    /// When this seed was created
    pub created_at: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 16],
}

impl Seed {
    pub const LEN: usize = 8 + // discriminator
        8 + // seed_id
        1 + // seed_type (enum as u8)
        32 + // owner
        8 + // grow_power
        8 + // planted_at
        1 + // is_planted
        (1 + 32) + // planted_farm_space (Option<Pubkey>)
        8 + // created_at
        16; // reserve
}

/// Invite code account for referral system
#[account]
pub struct InviteCode {
    /// 12-byte invite code
    pub code: [u8; 12],
    /// Address of the inviter who created this code
    pub inviter: Pubkey,
    /// Number of times this code has been used (alias for uses)
    pub uses: u32,
    /// Maximum number of uses allowed
    pub max_uses: u32,
    /// When this code was created
    pub created_at: i64,
    /// Whether this code is still active
    pub is_active: bool,
    /// Whether this code was created as operator
    pub created_as_operator: bool,
    /// Salt for hash generation
    pub salt: [u8; 16],
    /// Hash of the code for verification
    pub code_hash: [u8; 32],
    /// Invite limit for this code (alias for max_uses)
    pub invite_limit: u32,
    /// Reserved for future expansion
    pub reserve: [u8; 10],
}

impl InviteCode {
    pub const LEN: usize = 8 + // discriminator
        12 + // code (increased to 12)
        32 + // inviter
        4 + // uses
        4 + // max_uses
        8 + // created_at
        1 + // is_active
        1 + // created_as_operator
        16 + // salt
        32 + // code_hash
        4 + // invite_limit
        10; // reserve (decreased to maintain total size)
        
}

/// Farm level configuration for dynamic level management
#[account]
pub struct FarmLevelConfig {
    /// Maximum level available
    pub max_level: u8,
    /// Capacity for each level (array of 20 levels max)
    pub capacities: [u8; 20],
    /// Pack purchase thresholds for each level
    pub upgrade_thresholds: [u32; 20],
    /// Level names (optional, 32 bytes each)
    pub level_names: [[u8; 32]; 20],
    /// When this config was created
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

impl FarmLevelConfig {
    pub const LEN: usize = 8 + // discriminator
        1 + // max_level
        20 + // capacities (20 * 1)
        80 + // upgrade_thresholds (20 * 4)
        640 + // level_names (20 * 32)
        8 + // created_at
        8 + // updated_at
        32; // reserve
        
    pub const DEFAULT_SPACE: usize = Self::LEN;
}

/// Seed pack account for mystery box functionality
#[account]
pub struct SeedPack {
    /// Pack unique ID
    pub pack_id: u64,
    /// Owner of this pack (replacing purchaser field)
    pub owner: Pubkey,
    /// Number of seeds in this pack
    pub quantity: u8,
    /// Whether this pack has been opened
    pub is_opened: bool,
    /// Probability table version used for this pack
    pub table_version: u32,
    /// Switchboard VRF request for randomness
    pub vrf_request: Option<Pubkey>,
    /// Random seed for pack opening
    pub random_seed: Option<u64>,
    /// When this pack was purchased
    pub purchased_at: i64,
    /// When this pack was opened (if opened)
    pub opened_at: Option<i64>,
    /// Cost paid for this pack in WEED tokens
    pub cost_paid: u64,
    /// VRF fee paid for randomness
    pub vrf_fee_paid: u64,
    /// VRF sequence number (legacy field)
    pub vrf_sequence: Option<u64>,
    /// User entropy seed for additional randomness
    pub user_entropy_seed: Option<u64>,
    /// Final random value generated
    pub final_random_value: Option<u64>,
    /// VRF account used for randomness
    pub vrf_account: Option<Pubkey>,
    /// Reserved for future expansion
    pub reserve: [u8; 8],
}

impl SeedPack {
    pub const LEN: usize = 8 + // discriminator
        8 + // pack_id
        32 + // owner
        1 + // quantity
        1 + // is_opened
        4 + // table_version
        (1 + 32) + // vrf_request (Option<Pubkey>)
        (1 + 8) + // random_seed (Option<u64>)
        8 + // purchased_at
        (1 + 8) + // opened_at (Option<i64>)
        8 + // cost_paid
        8 + // vrf_fee_paid
        (1 + 8) + // vrf_sequence (Option<u64>)
        (1 + 8) + // user_entropy_seed (Option<u64>)
        (1 + 8) + // final_random_value (Option<u64>)
        (1 + 32) + // vrf_account (Option<Pubkey>)
        8; // reserve
}

impl UserState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        8 + // total_grow_power
        8 + // last_harvest_time
        1 + // has_farm_space
        (1 + 32) + // referrer (Option<Pubkey>)
        8 + // pending_referral_rewards
        4 + // total_packs_purchased
        28; // reserve
}

impl SeedType {
    /// Default grow power values for known seed types (first 8)
    /// Secret seed values are not exposed here
    const KNOWN_GROW_POWERS: [u64; 8] = [
        100, 180, 420, 720, 1000, 5000, 15000, 30000
    ];
    
    /// Get grow power (alias for get_default_grow_power for backward compatibility)
    pub fn get_grow_power(&self) -> u64 {
        self.get_default_grow_power()
    }
    
    /// Get grow power from probability table (respects reveal status)
    pub fn get_grow_power_from_table(&self, table: &ProbabilityTable) -> u64 {
        let index = *self as usize;
        if index < table.seed_count as usize && index < 16 {
            // Only return actual value if seed is revealed
            if table.is_seed_revealed(index as u8) {
                table.grow_powers[index]
            } else {
                0 // Hidden seed - return 0 to external queries
            }
        } else {
            self.get_default_grow_power()
        }
    }
    
    /// Get grow power from probability table (internal use - bypasses reveal check)
    pub fn get_actual_grow_power_from_table(&self, table: &ProbabilityTable) -> u64 {
        let index = *self as usize;
        if index < table.seed_count as usize && index < 16 {
            table.grow_powers[index] // Return actual value regardless of reveal status
        } else {
            self.get_default_grow_power()
        }
    }
    
    /// Get default grow power (only for known seed types)
    pub fn get_default_grow_power(&self) -> u64 {
        let index = *self as usize;
        if index < 8 {
            Self::KNOWN_GROW_POWERS[index]
        } else {
            0 // Secret seeds have no default value
        }
    }
    
    /// Check if this seed type is initially known (not secret)
    pub fn is_initially_known(&self) -> bool {
        (*self as u8) < 8
    }
    
    /// Convert random value to seed type using probability table
    pub fn from_random_with_table(random: u64, table: &ProbabilityTable) -> Self {
        let value = (random % 10000) as u16;
        
        for i in 0..(table.seed_count as usize).min(16) {
            if value < table.probability_thresholds[i] {
                return Self::from_index(i as u8).unwrap_or(SeedType::Seed1);
            }
        }
        
        // Fallback to last seed type in table
        let last_index = ((table.seed_count - 1) as usize).min(15);
        Self::from_index(last_index as u8).unwrap_or(SeedType::Seed1)
    }
    
    /// Legacy method for backward compatibility
    pub fn from_random(random: u64) -> Self {
        let table = ProbabilityTable::init_standard_table();
        Self::from_random_with_table(random, &table)
    }
    
    /// Iterator-friendly array of all seed types (16 types)
    pub const fn all_types() -> [SeedType; 16] {
        [
            SeedType::Seed1, SeedType::Seed2, SeedType::Seed3, SeedType::Seed4,
            SeedType::Seed5, SeedType::Seed6, SeedType::Seed7, SeedType::Seed8,
            SeedType::Seed9, SeedType::Seed10, SeedType::Seed11, SeedType::Seed12,
            SeedType::Seed13, SeedType::Seed14, SeedType::Seed15, SeedType::Seed16,
        ]
    }
    
    /// Get probability from table
    pub fn get_probability_from_table(&self, table: &ProbabilityTable) -> f32 {
        let index = *self as usize;
        if index < table.seed_count as usize && index < 16 {
            table.probability_percentages[index]
        } else {
            0.0
        }
    }
    
    /// Validate seed type enum value (supports 16 types)
    pub fn is_valid(value: u8) -> bool {
        value < 16
    }
    
    /// Create SeedType from index (supports 16 types)
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
            9 => Ok(SeedType::Seed10),
            10 => Ok(SeedType::Seed11),
            11 => Ok(SeedType::Seed12),
            12 => Ok(SeedType::Seed13),
            13 => Ok(SeedType::Seed14),
            14 => Ok(SeedType::Seed15),
            15 => Ok(SeedType::Seed16),
            _ => Err(crate::error::GameError::InvalidConfig.into()),
        }
    }
}
