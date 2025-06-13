#[cfg(test)]
mod state_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;
    use crate::error::GameError;

    // ===== ACCOUNT SIZE TESTS =====
    // Consolidated account size testing to prevent duplication
    // Advanced behavior and lifecycle tests are in state_advanced_tests.rs

    #[test]
    fn test_config_account_size() {
        // Test Config account size calculation
        let expected_len = 8 + // discriminator
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

        assert_eq!(Config::LEN, expected_len, "Config account size mismatch");
    }

    #[test]
    fn test_user_state_account_size() {
        // Test UserState account size calculation
        let expected_len = 8 + // discriminator
            32 + // owner
            8 + // total_grow_power
            8 + // last_harvest_time
            1 + // has_farm_space
            1 + 32 + // referrer (Option<Pubkey>)
            8 + // pending_referral_rewards
            32; // reserve

        assert_eq!(UserState::LEN, expected_len, "UserState account size mismatch");
    }

    #[test]
    fn test_farm_space_account_size() {
        // Test FarmSpace account size calculation
        let expected_len = 8 + // discriminator
            32 + // owner
            1 + // level
            1 + // capacity
            1 + // seed_count
            8 + // total_grow_power
            8 + // upgrade_start_time
            1 + // upgrade_target_level
            32; // reserve

        assert_eq!(FarmSpace::LEN, expected_len, "FarmSpace account size mismatch");
    }

    #[test]
    fn test_seed_account_size() {
        // Test Seed account size calculation
        let expected_len = 8 + // discriminator
            32 + // owner
            1 + // seed_type
            8 + // grow_power
            1 + // is_planted
            1 + 32 + // planted_farm_space (Option<Pubkey>)
            8 + // planted_at
            16; // reserve

        assert_eq!(Seed::LEN, expected_len, "Seed account size mismatch");
    }

    #[test]
    fn test_seed_pack_account_size_vrf_only() {
        // Test VRF-only SeedPack account size calculation
        let expected_len = 8 + // discriminator
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

        assert_eq!(SeedPack::LEN, expected_len, "VRF SeedPack account size mismatch");
    }

    #[test]
    fn test_invite_code_account_size() {
        // Test InviteCode account size calculation
        let expected_len = 8 + // discriminator
            32 + // creator
            8 + // code ([u8; 8])
            1 + // invite_limit
            1 + // invites_used
            8 + // created_at
            16; // reserve

        assert_eq!(InviteCode::LEN, expected_len, "InviteCode account size mismatch");
    }

    #[test]
    fn test_global_stats_account_size() {
        // Test GlobalStats account size calculation
        let expected_len = 8 + // discriminator
            8 + // total_grow_power
            8 + // total_farm_spaces
            8 + // current_rewards_per_second
            8 + // last_update_time
            32; // reserve

        assert_eq!(GlobalStats::LEN, expected_len, "GlobalStats account size mismatch");
    }

    #[test]
    fn test_fee_pool_account_size() {
        // Test FeePool account size calculation
        let expected_len = 8 + // discriminator
            8 + // accumulated_fees
            32 + // treasury_address
            8 + // last_collection_time
            48; // reserve

        assert_eq!(FeePool::LEN, expected_len, "FeePool account size mismatch");
    }

    // ===== BASIC CONSTANT VERIFICATION =====
    // Basic constant validation - complex integration tests in state_advanced_tests.rs

    #[test]
    fn test_seed_type_basic_properties() {
        // Test that SeedType enum has correct number of variants
        assert_eq!(SEED_GROW_POWERS.len(), 9, "Should have 9 seed types");
        assert_eq!(SEED_PROBABILITIES.len(), 9, "Should have 9 probability values");
        assert_eq!(SEED_PROBABILITY_THRESHOLDS.len(), 9, "Should have 9 threshold values");
        
        // Test basic grow power values
        assert_eq!(SeedType::Seed1.get_grow_power(), 100);
        assert_eq!(SeedType::Seed9.get_grow_power(), 60000);
    }

    #[test]
    fn test_farm_space_basic_constants() {
        // Test farm level capacity constants
        assert_eq!(FARM_CAPACITIES.len(), 5, "Should have 5 farm levels");
        assert_eq!(UPGRADE_COSTS.len(), 4, "Should have 4 upgrade paths");
        
        // Test capacity helper functions
        assert_eq!(FarmSpace::capacity_for_level(1), FARM_CAPACITIES[0]);
        assert_eq!(FarmSpace::capacity_for_level(5), FARM_CAPACITIES[4]);
        
        // Test upgrade cost helper functions
        assert_eq!(FarmSpace::upgrade_cost_for_level(1), Some(UPGRADE_COSTS[0]));
        assert!(FarmSpace::upgrade_cost_for_level(5).is_none());
    }

    #[test] 
    fn test_basic_game_constants() {
        // Test core economic constants
        assert_eq!(DEFAULT_BASE_RATE, 100);
        assert_eq!(DEFAULT_HALVING_INTERVAL, 7 * 24 * 60 * 60); // 7 days
        assert_eq!(SEED_PACK_COST, 300 * 1_000_000);
        assert_eq!(FARM_SPACE_COST_SOL, 500_000_000);
        assert_eq!(TRADING_FEE_PERCENTAGE, 2);
        assert_eq!(MAX_INVITE_LIMIT, 5);
    }

    // ===== BASIC VALIDATION TESTS =====
    // Simple validation tests - complex scenarios in other test modules

    #[test]
    fn test_invite_code_validation() {
        // Test valid invite code formats
        assert!(crate::constants::validate_invite_code(b"ABCD1234"));
        assert!(crate::constants::validate_invite_code(b"abcd1234"));
        assert!(crate::constants::validate_invite_code(b"A1B2C3D4"));
        
        // Test invalid formats
        assert!(!crate::constants::validate_invite_code(b"ABC@1234"));
        assert!(!crate::constants::validate_invite_code(b"ABC 1234"));
    }

    #[test]
    fn test_quantity_validation() {
        // Test seed pack quantity validation
        assert!(!crate::constants::validate_quantity(0));
        assert!(crate::constants::validate_quantity(1));
        assert!(crate::constants::validate_quantity(50));
        assert!(crate::constants::validate_quantity(100));
        assert!(!crate::constants::validate_quantity(101));
    }

    #[test]
    fn test_farm_level_validation() {
        // Test farm level validation
        assert!(!crate::constants::validate_farm_level(0));
        assert!(crate::constants::validate_farm_level(1));
        assert!(crate::constants::validate_farm_level(5));
        assert!(!crate::constants::validate_farm_level(6));
    }
}