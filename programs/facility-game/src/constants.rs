/// Game constants and configuration values
/// Centralized location for all game balance and system parameters

// ===== ECONOMIC CONSTANTS =====

/// Default base reward rate (WEED tokens per second)
pub const DEFAULT_BASE_RATE: u64 = 100;

/// Default halving interval (6 days in seconds)
pub const DEFAULT_HALVING_INTERVAL: i64 = 6 * 24 * 60 * 60;

/// Farm space purchase cost (0.5 SOL in lamports)
pub const FARM_SPACE_COST_SOL: u64 = 500_000_000;

/// Mystery seed pack cost (300 WEED with 6 decimals)
pub const SEED_PACK_COST: u64 = 300 * 1_000_000;

/// Trading fee percentage (2%)
pub const TRADING_FEE_PERCENTAGE: u8 = 2;

/// Maximum invite limit per user
pub const MAX_INVITE_LIMIT: u8 = 5;

/// Total WEED supply (120 million tokens with 6 decimals)
/// Fixed supply cap that cannot be exceeded
pub const TOTAL_WEED_SUPPLY: u64 = 120_000_000 * 1_000_000;

// ===== FARM SPACE CONSTANTS =====

/// Farm space upgrade cooldown (24 hours in seconds)
pub const UPGRADE_COOLDOWN: i64 = 24 * 60 * 60;

/// Farm space level capacities
pub const FARM_CAPACITIES: [u8; 5] = [4, 8, 12, 16, 20];

/// Farm space upgrade costs (in WEED tokens with 6 decimals)
pub const UPGRADE_COSTS: [u64; 4] = [
    3_500 * 1_000_000,   // Level 1→2: 3,500 WEED
    18_000 * 1_000_000,  // Level 2→3: 18,000 WEED
    20_000 * 1_000_000,  // Level 3→4: 20,000 WEED
    25_000 * 1_000_000,  // Level 4→5: 25,000 WEED
];

// ===== SEED SYSTEM CONSTANTS =====

/// Maximum seeds per user storage
pub const MAX_SEEDS_PER_USER: usize = 100;

/// Maximum seed pack purchase quantity
pub const MAX_SEED_PACK_QUANTITY: u8 = 100;

/// Seed grow power values
pub const SEED_GROW_POWERS: [u64; 9] = [
    100,    // Seed1
    180,    // Seed2
    420,    // Seed3
    720,    // Seed4
    1000,   // Seed5
    5000,   // Seed6
    15000,  // Seed7
    30000,  // Seed8
    60000,  // Seed9
];

/// Seed probability thresholds (out of 10000 for precise percentage control)
pub const SEED_PROBABILITY_THRESHOLDS: [u16; 9] = [
    4222,  // Seed1: 42.23%
    6666,  // Seed2: 24.44%
    7999,  // Seed3: 13.33%
    8832,  // Seed4: 8.33%
    9388,  // Seed5: 5.56%
    9721,  // Seed6: 3.33%
    9854,  // Seed7: 1.33%
    9943,  // Seed8: 0.89%
    10000, // Seed9: 0.56%
];

/// Seed probability percentages (for display/documentation)
pub const SEED_PROBABILITIES: [f32; 9] = [
    42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56
];

// ===== REFERRAL SYSTEM CONSTANTS =====

/// Level 1 referral reward percentage (10%)
pub const LEVEL1_REFERRAL_PERCENTAGE: u8 = 10;

/// Level 2 referral reward percentage (5%)
pub const LEVEL2_REFERRAL_PERCENTAGE: u8 = 5;

/// Maximum referral chain depth
pub const MAX_REFERRAL_DEPTH: u8 = 2;

// ===== VALIDATION CONSTANTS =====

/// Minimum quantity for operations
pub const MIN_QUANTITY: u8 = 1;

/// Maximum invite code length
pub const INVITE_CODE_LENGTH: usize = 8;

/// Minimum time interval for reward claims (prevent spam)
pub const MIN_CLAIM_INTERVAL: i64 = 1; // 1 second

// ===== PDA SEEDS =====

/// PDA seeds for deterministic address generation
pub mod seeds {
    /// Config PDA seed
    pub const CONFIG: &[u8] = b"config";
    
    /// User state PDA seed prefix
    pub const USER: &[u8] = b"user";
    
    /// Farm space PDA seed prefix
    pub const FARM_SPACE: &[u8] = b"farm_space";
    
    /// Reward mint PDA seed
    pub const REWARD_MINT: &[u8] = b"reward_mint";
    
    /// Mint authority PDA seed
    pub const MINT_AUTHORITY: &[u8] = b"mint_authority";
    
    /// Global stats PDA seed
    pub const GLOBAL_STATS: &[u8] = b"global_stats";
    
    /// Fee pool PDA seed
    pub const FEE_POOL: &[u8] = b"fee_pool";
    
    /// Seed storage PDA seed prefix
    pub const SEED_STORAGE: &[u8] = b"seed_storage";
    
    /// Seed PDA seed prefix
    pub const SEED: &[u8] = b"seed";
    
    /// Seed pack PDA seed prefix
    pub const SEED_PACK: &[u8] = b"seed_pack";
    
    /// Invite code PDA seed prefix
    pub const INVITE_CODE: &[u8] = b"invite_code";
    
    /// Reward account PDA seed prefix
    pub const REWARD_ACCOUNT: &[u8] = b"reward_account";
    
    /// Meteora config PDA seed
    pub const METEORA_CONFIG: &[u8] = b"meteora_config";
    
    /// Pyth entropy request PDA seed prefix
    pub const ENTROPY_REQUEST: &[u8] = b"entropy_request";
}

// ===== TOKEN CONSTANTS =====

/// WEED token decimals
pub const WEED_DECIMALS: u8 = 6;

/// WEED token symbol
pub const WEED_SYMBOL: &str = "WEED";

/// WEED token name
pub const WEED_NAME: &str = "Weed Token";

// ===== TIME CONSTANTS =====

/// Seconds per hour
pub const SECONDS_PER_HOUR: i64 = 60 * 60;

/// Seconds per day
pub const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

/// Seconds per week
pub const SECONDS_PER_WEEK: i64 = 7 * SECONDS_PER_DAY;

// ===== CALCULATION HELPERS =====

/// Helper functions for common calculations
impl crate::state::FarmSpace {
    /// Get capacity for a given level using constants
    pub fn capacity_for_level(level: u8) -> u8 {
        if level == 0 || level > 5 {
            return FARM_CAPACITIES[4]; // Max capacity
        }
        FARM_CAPACITIES[(level - 1) as usize]
    }
    
    /// Get upgrade cost for a given level using constants
    pub fn upgrade_cost_for_level(level: u8) -> Option<u64> {
        if level == 0 || level > 4 {
            return None;
        }
        Some(UPGRADE_COSTS[(level - 1) as usize])
    }
}

/// Helper functions for seed type calculations
impl crate::state::SeedType {
    /// Get grow power using constants
    pub fn grow_power_from_constants(&self) -> u64 {
        SEED_GROW_POWERS[*self as usize]
    }
    
    /// Get probability percentage using constants
    pub fn probability_from_constants(&self) -> f32 {
        SEED_PROBABILITIES[*self as usize]
    }
    
    /// Convert random value to seed type using constants
    pub fn from_random_with_constants(random: u64) -> Self {
        let value = (random % 10000) as u16;
        
        for (i, &threshold) in SEED_PROBABILITY_THRESHOLDS.iter().enumerate() {
            if value < threshold {
                return unsafe { std::mem::transmute(i as u8) };
            }
        }
        
        // Fallback to highest rarity
        Self::Seed9
    }
}

// ===== VALIDATION HELPERS =====

/// Validate quantity is within acceptable range
pub fn validate_quantity(quantity: u8) -> bool {
    quantity >= MIN_QUANTITY && quantity <= MAX_SEED_PACK_QUANTITY
}

/// Validate farm level
pub fn validate_farm_level(level: u8) -> bool {
    level >= 1 && level <= 5
}

/// Validate invite code format
pub fn validate_invite_code(code: &[u8; 8]) -> bool {
    code.iter().all(|&b| {
        (b >= b'A' && b <= b'Z') || 
        (b >= b'a' && b <= b'z') || 
        (b >= b'0' && b <= b'9')
    })
}

/// Calculate referral rewards using constants
pub fn calculate_referral_rewards_with_constants(base_amount: u64) -> (u64, u64) {
    let level1 = base_amount * LEVEL1_REFERRAL_PERCENTAGE as u64 / 100;
    let level2 = base_amount * LEVEL2_REFERRAL_PERCENTAGE as u64 / 100;
    (level1, level2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_consistency() {
        // Verify array lengths match
        assert_eq!(SEED_GROW_POWERS.len(), 9);
        assert_eq!(SEED_PROBABILITY_THRESHOLDS.len(), 9);
        assert_eq!(SEED_PROBABILITIES.len(), 9);
        assert_eq!(FARM_CAPACITIES.len(), 5);
        assert_eq!(UPGRADE_COSTS.len(), 4);
        
        // Verify probability thresholds are in ascending order
        for i in 1..SEED_PROBABILITY_THRESHOLDS.len() {
            assert!(SEED_PROBABILITY_THRESHOLDS[i] > SEED_PROBABILITY_THRESHOLDS[i-1]);
        }
        
        // Verify last threshold is 10000 (100%)
        assert_eq!(SEED_PROBABILITY_THRESHOLDS[8], 10000);
        
        // Verify farm capacities are in ascending order
        for i in 1..FARM_CAPACITIES.len() {
            assert!(FARM_CAPACITIES[i] > FARM_CAPACITIES[i-1]);
        }
    }
    
    #[test]
    fn test_validation_helpers() {
        // Test quantity validation
        assert!(!validate_quantity(0));
        assert!(validate_quantity(1));
        assert!(validate_quantity(50));
        assert!(validate_quantity(100));
        assert!(!validate_quantity(101));
        
        // Test farm level validation
        assert!(!validate_farm_level(0));
        assert!(validate_farm_level(1));
        assert!(validate_farm_level(5));
        assert!(!validate_farm_level(6));
        
        // Test invite code validation
        assert!(validate_invite_code(b"ABCD1234"));
        assert!(validate_invite_code(b"abcd1234"));
        assert!(validate_invite_code(b"AbCd1234"));
        assert!(!validate_invite_code(b"ABC@1234")); // Contains invalid character
        assert!(!validate_invite_code(b"ABC 1234")); // Contains space
    }
    
    #[test]
    fn test_referral_calculations() {
        let (level1, level2) = calculate_referral_rewards_with_constants(1000);
        assert_eq!(level1, 100); // 10% of 1000
        assert_eq!(level2, 50);  // 5% of 1000
        
        let (level1, level2) = calculate_referral_rewards_with_constants(0);
        assert_eq!(level1, 0);
        assert_eq!(level2, 0);
    }
}