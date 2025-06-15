#[cfg(test)]
mod basic_tests {
    use anchor_lang::prelude::*;
    use crate::constants::*;
    use crate::error::GameError;
    use crate::state::*;

    #[test]
    fn test_constants() {
        // Verify our updated constants
        assert_eq!(DEFAULT_HALVING_INTERVAL, 604800); // 7 days in seconds
        assert_eq!(TOTAL_WEED_SUPPLY, 240_000_000_000_000u64); // 240M WEED (with 6 decimals)
        assert_eq!(FARM_SPACE_COST_SOL, 500_000_000); // 0.5 SOL in lamports
        assert_eq!(SEED_PACK_COST, 300_000_000); // 300 WEED
    }

    #[test]
    fn test_seed_types() {
        // Test seed type grow power values (updated for 6-seed system with constants)
        use crate::constants::SEED_GROW_POWERS;
        assert_eq!(SeedType::Seed1.get_grow_power(), SEED_GROW_POWERS[0]); // 100
        assert_eq!(SeedType::Seed2.get_grow_power(), SEED_GROW_POWERS[1]); // 180
        assert_eq!(SeedType::Seed3.get_grow_power(), SEED_GROW_POWERS[2]); // 420
        assert_eq!(SeedType::Seed4.get_grow_power(), SEED_GROW_POWERS[3]); // 720
        assert_eq!(SeedType::Seed5.get_grow_power(), SEED_GROW_POWERS[4]); // 1000
        assert_eq!(SeedType::Seed6.get_grow_power(), SEED_GROW_POWERS[5]); // 5000
        
        // Test secret seeds (9-16) have default grow power 0 (7-8 are still known)
        assert_eq!(SeedType::Seed9.get_grow_power(), 0);
        assert_eq!(SeedType::Seed10.get_grow_power(), 0);
        assert_eq!(SeedType::Seed11.get_grow_power(), 0);
    }

    #[test]
    fn test_seed_probabilities() {
        // Test that seed types can be created from index
        assert!(SeedType::from_index(0).is_ok());
        assert!(SeedType::from_index(15).is_ok());
        assert!(SeedType::from_index(16).is_err());
        
        // Test default grow powers for known seeds (first 8 are known, rest are secret)
        assert_eq!(SeedType::Seed1.get_default_grow_power(), 100);
        assert_eq!(SeedType::Seed6.get_default_grow_power(), 5000); // Last known seed
        assert_eq!(SeedType::Seed7.get_default_grow_power(), 15000); // Still known
        assert_eq!(SeedType::Seed8.get_default_grow_power(), 30000); // Still known
        assert_eq!(SeedType::Seed9.get_default_grow_power(), 0); // Secret seed
    }

    #[test]
    fn test_farm_space_levels() {
        // Test farm space capacity by level (updated to use constants)
        use crate::constants::FARM_CAPACITIES;
        assert_eq!(FarmSpace::get_capacity_for_level(1), FARM_CAPACITIES[0]); // 4
        assert_eq!(FarmSpace::get_capacity_for_level(2), FARM_CAPACITIES[1]); // 6
        assert_eq!(FarmSpace::get_capacity_for_level(3), FARM_CAPACITIES[2]); // 10
        assert_eq!(FarmSpace::get_capacity_for_level(4), FARM_CAPACITIES[3]); // 16
        assert_eq!(FarmSpace::get_capacity_for_level(5), FARM_CAPACITIES[4]); // 25
    }

    #[test]
    fn test_pack_upgrade_thresholds() {
        // Test auto-upgrade thresholds based on pack purchases
        use crate::constants::FARM_UPGRADE_THRESHOLDS;
        assert_eq!(FARM_UPGRADE_THRESHOLDS[0], 0);   // Level 1: 0 packs
        assert_eq!(FARM_UPGRADE_THRESHOLDS[1], 30);  // Level 2: 30 packs
        assert_eq!(FARM_UPGRADE_THRESHOLDS[2], 100); // Level 3: 100 packs
        assert_eq!(FARM_UPGRADE_THRESHOLDS[3], 300); // Level 4: 300 packs
        assert_eq!(FARM_UPGRADE_THRESHOLDS[4], 500); // Level 5: 500 packs
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

