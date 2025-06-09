use anchor_lang::prelude::*;

/// Global configuration account
#[account]
pub struct Config {
    /// Base reward rate
    pub base_rate: u64,
    /// Halving interval (seconds)
    pub halving_interval: i64,
    /// Next halving timestamp
    pub next_halving_time: i64,
    /// Admin address
    pub admin: Pubkey,
    /// Treasury account for fee collection
    pub treasury: Pubkey,
    /// Mystery box cost in $WEED tokens
    pub mystery_box_cost: u64,
    /// Global seed counter
    pub seed_counter: u64,
    /// Global mystery box counter
    pub mystery_box_counter: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 8], // Reduced for new fields
}

/// User state account
#[account]
pub struct UserState {
    /// User's public key
    pub owner: Pubkey,
    /// Total Grow Power
    pub total_grow_power: u64,
    /// Last harvest timestamp
    pub last_harvest_time: i64,
    /// Whether the user has a facility
    pub has_facility: bool,
    /// Referrer (inviter) public key - can be None for users without referrer
    pub referrer: Option<Pubkey>,
    /// Accumulated referral rewards (unclaimed)
    pub pending_referral_rewards: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 55], // Reduced for referrer option and pending_referral_rewards
}

/// Facility account
#[account]
pub struct Facility {
    /// Facility owner
    pub owner: Pubkey,
    /// Current facility size/level (determines capacity)
    pub facility_size: u32,
    /// Maximum machine capacity based on facility size
    pub max_capacity: u32,
    /// Machine count (cannot exceed max_capacity)
    pub machine_count: u32,
    /// Total Grow Power
    pub total_grow_power: u64,
    /// Reserved for future expansion (multiple machine types, etc.)
    pub reserve: [u8; 56], // Reduced for new fields
}

impl Config {
    pub const LEN: usize = 8 + // discriminator
        8 + // base_rate
        8 + // halving_interval
        8 + // next_halving_time
        32 + // admin
        32 + // treasury
        8 + // mystery_box_cost
        8 + // seed_counter
        8 + // mystery_box_counter
        8; // reserve
}

impl UserState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        8 + // total_grow_power
        8 + // last_harvest_time
        1 + // has_facility
        1 + 32 + // referrer (Option<Pubkey>: 1 byte discriminator + 32 bytes for Some(Pubkey))
        8 + // pending_referral_rewards
        55; // reserve
}

impl Facility {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + // facility_size
        4 + // max_capacity
        4 + // machine_count
        8 + // total_grow_power
        56; // reserve
        
    /// Calculate maximum capacity based on facility size
    /// 施設サイズに基づいてマシン容量を計算
    /// サイズ1: 1台, サイズ2: 3台, サイズ3: 6台, サイズ4: 10台...
    pub fn calculate_max_capacity(facility_size: u32) -> u32 {
        match facility_size {
            1 => 1,
            2 => 3,
            3 => 6,
            4 => 10,
            5 => 15,
            _ => 15 + (facility_size - 5) * 5, // サイズ6以降は5台ずつ増加
        }
    }
    
    /// Calculate upgrade cost for next facility size level
    /// 次のレベルのアップグレードコストを計算
    pub fn calculate_upgrade_cost(current_size: u32) -> u64 {
        let next_size = current_size + 1;
        match next_size {
            2 => 1000,   // サイズ1→2: 1000 $WEED
            3 => 2500,   // サイズ2→3: 2500 $WEED
            4 => 5000,   // サイズ3→4: 5000 $WEED
            5 => 10000,  // サイズ4→5: 10000 $WEED
            _ => 10000 + (next_size as u64 - 5) * 5000, // サイズ6以降は5000ずつ増加
        }
    }
}

/// Seed rarity levels
#[derive(Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum SeedRarity {
    Common,    // 70% - 1x multiplier
    Rare,      // 20% - 1.5x multiplier  
    Epic,      // 8% - 2x multiplier
    Legendary, // 2% - 3x multiplier
}

impl SeedRarity {
    /// Get grow power multiplier for this rarity
    pub fn get_multiplier(&self) -> u64 {
        match self {
            SeedRarity::Common => 100,    // 1x (100%)
            SeedRarity::Rare => 150,      // 1.5x (150%)
            SeedRarity::Epic => 200,      // 2x (200%) 
            SeedRarity::Legendary => 300, // 3x (300%)
        }
    }
    
    /// Get rarity name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            SeedRarity::Common => "Common",
            SeedRarity::Rare => "Rare",
            SeedRarity::Epic => "Epic",
            SeedRarity::Legendary => "Legendary",
        }
    }
}

/// Seed NFT account
#[account]
pub struct Seed {
    /// Seed owner
    pub owner: Pubkey,
    /// Seed rarity level
    pub rarity: SeedRarity,
    /// Grow power multiplier (based on rarity)
    pub grow_power_multiplier: u64,
    /// Whether this seed is currently planted in a facility
    pub is_planted: bool,
    /// Facility where this seed is planted (if any)
    pub planted_facility: Option<Pubkey>,
    /// Creation timestamp
    pub created_at: i64,
    /// Unique seed ID
    pub seed_id: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

/// Mystery Box purchase tracker
#[account]
pub struct MysteryBox {
    /// Purchaser
    pub purchaser: Pubkey,
    /// Purchase timestamp
    pub purchased_at: i64,
    /// Box cost paid
    pub cost_paid: u64,
    /// Whether box has been opened
    pub is_opened: bool,
    /// Random seed for opening (set when purchased)
    pub random_seed: u64,
    /// Box ID
    pub box_id: u64,
    /// Reserved for future expansion
    pub reserve: [u8; 32],
}

impl Seed {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        1 + // rarity (enum)
        8 + // grow_power_multiplier
        1 + // is_planted
        1 + 32 + // planted_facility (Option<Pubkey>)
        8 + // created_at
        8 + // seed_id
        32; // reserve
}

impl MysteryBox {
    pub const LEN: usize = 8 + // discriminator
        32 + // purchaser
        8 + // purchased_at
        8 + // cost_paid
        1 + // is_opened
        8 + // random_seed
        8 + // box_id
        32; // reserve
}