#[cfg(test)]
mod basic_tests {
    use anchor_lang::prelude::*;
    use crate::constants::*;
    use crate::error::GameError;
    use crate::state::*;

    #[test]
    fn test_constants() {
        // Verify our updated constants
        assert_eq!(DEFAULT_HALVING_INTERVAL, 200); // 200 seconds
        assert_eq!(TOTAL_WEED_SUPPLY, 240_000_000_000_000u64); // 240M WEED (with 6 decimals)
        assert_eq!(FARM_SPACE_COST_SOL, 500_000_000); // 0.5 SOL in lamports
        assert_eq!(SEED_PACK_COST, 300_000_000); // 300 WEED
    }

    #[test]
    fn test_seed_types() {
        // Test seed type grow power values
        assert_eq!(SeedType::Seed1.get_grow_power(), 100);
        assert_eq!(SeedType::Seed2.get_grow_power(), 180);
        assert_eq!(SeedType::Seed3.get_grow_power(), 420);
        assert_eq!(SeedType::Seed4.get_grow_power(), 720);
        assert_eq!(SeedType::Seed5.get_grow_power(), 1000);
        assert_eq!(SeedType::Seed6.get_grow_power(), 5000);
        assert_eq!(SeedType::Seed7.get_grow_power(), 15000);
        assert_eq!(SeedType::Seed8.get_grow_power(), 30000);
        assert_eq!(SeedType::Seed9.get_grow_power(), 60000);
    }

    #[test]
    fn test_seed_probabilities() {
        // Test that individual probabilities are correct
        assert_eq!(SeedType::Seed1.get_probability_percent(), 42.23);
        assert_eq!(SeedType::Seed9.get_probability_percent(), 0.56);
        
        // Test sum of all probabilities is approximately 100%
        let all_types = SeedType::all_types();
        let sum: f32 = all_types.iter().map(|t| t.get_probability_percent()).sum();
        assert!((sum - 100.0).abs() < 0.01); // Within 0.01% tolerance
    }

    #[test]
    fn test_farm_space_levels() {
        // Test farm space capacity by level (updated)
        assert_eq!(FarmSpace::get_capacity_for_level(1), 4);
        assert_eq!(FarmSpace::get_capacity_for_level(2), 6);
        assert_eq!(FarmSpace::get_capacity_for_level(3), 8);
        assert_eq!(FarmSpace::get_capacity_for_level(4), 10);
        assert_eq!(FarmSpace::get_capacity_for_level(5), 12);
    }

    #[test]
    fn test_pack_thresholds() {
        // Test auto-upgrade thresholds based on pack purchases
        assert_eq!(FarmSpace::calculate_level_from_packs(0), 1);   // 0 packs = Level 1
        assert_eq!(FarmSpace::calculate_level_from_packs(29), 1);  // 29 packs = Level 1
        assert_eq!(FarmSpace::calculate_level_from_packs(30), 2);  // 30 packs = Level 2
        assert_eq!(FarmSpace::calculate_level_from_packs(99), 2);  // 99 packs = Level 2
        assert_eq!(FarmSpace::calculate_level_from_packs(100), 3); // 100 packs = Level 3
        assert_eq!(FarmSpace::calculate_level_from_packs(299), 3); // 299 packs = Level 3
        assert_eq!(FarmSpace::calculate_level_from_packs(300), 4); // 300 packs = Level 4
        assert_eq!(FarmSpace::calculate_level_from_packs(499), 4); // 499 packs = Level 4
        assert_eq!(FarmSpace::calculate_level_from_packs(500), 5); // 500 packs = Level 5
        assert_eq!(FarmSpace::calculate_level_from_packs(1000), 5); // 1000 packs = Level 5 (max)
    }
    
    #[test]
    fn test_legacy_upgrade_costs() {
        // Test legacy upgrade costs (now deprecated)
        assert_eq!(FarmSpace::get_legacy_upgrade_cost(1), Some(3_500_000_000)); // Level 1→2
        assert_eq!(FarmSpace::get_legacy_upgrade_cost(2), Some(18_000_000_000)); // Level 2→3
        assert_eq!(FarmSpace::get_legacy_upgrade_cost(3), Some(20_000_000_000)); // Level 3→4
        assert_eq!(FarmSpace::get_legacy_upgrade_cost(4), Some(25_000_000_000)); // Level 4→5
        assert_eq!(FarmSpace::get_legacy_upgrade_cost(5), None); // No upgrade available
    }

    #[test]
    fn test_error_codes() {
        // Test that error types are properly defined
        let error = GameError::AlreadyHasFarmSpace;
        assert_eq!(error, GameError::AlreadyHasFarmSpace);
        
        let error2 = GameError::InsufficientFunds;
        assert_ne!(error, error2);
    }
}

// Helper function for farm space capacity
fn get_farm_space_capacity(level: u8) -> u8 {
    match level {
        1 => 4,
        2 => 8,
        3 => 12,
        4 => 16,
        5 => 20,
        _ => 0,
    }
}