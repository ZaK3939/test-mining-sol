#[cfg(test)]
mod utils_tests {
    use anchor_lang::prelude::*;
    use anchor_spl::token::{Mint, TokenAccount};
    use crate::utils::*;
    use crate::state::*;
    use crate::error::GameError;

    // ===== VALIDATION HELPER TESTS =====

    #[test]
    fn test_validate_user_ownership() {
        let owner_key = Pubkey::new_unique();
        let other_key = Pubkey::new_unique();
        
        let user_state = UserState {
            owner: owner_key,
            total_grow_power: 100,
            last_harvest_time: 0,
            has_farm_space: true,
            referrer: None,
            pending_referral_rewards: 0,
            reserve: [0; 32],
        };

        // Valid ownership
        assert!(validate_user_ownership(&user_state, owner_key).is_ok());
        
        // Invalid ownership
        assert_eq!(
            validate_user_ownership(&user_state, other_key).unwrap_err(),
            GameError::Unauthorized.into()
        );
    }

    #[test]
    fn test_validate_token_balance() {
        let mint_key = Pubkey::new_unique();
        let owner_key = Pubkey::new_unique();
        
        let token_account = TokenAccount {
            mint: mint_key,
            owner: owner_key,
            amount: 1000,
            delegate: None,
            state: anchor_spl::token::state::AccountState::Initialized,
            is_native: None,
            delegated_amount: 0,
            close_authority: None,
        };

        // Sufficient balance
        assert!(validate_token_balance(&token_account, 500).is_ok());
        assert!(validate_token_balance(&token_account, 1000).is_ok());
        
        // Insufficient balance
        assert_eq!(
            validate_token_balance(&token_account, 1001).unwrap_err(),
            GameError::InsufficientFunds.into()
        );
    }

    #[test]
    fn test_validate_farm_space_capacity() {
        let mut farm_space = FarmSpace {
            owner: Pubkey::new_unique(),
            level: 2,
            capacity: 8,
            seed_count: 5,
            total_grow_power: 500,
            upgrade_start_time: 0,
            upgrade_target_level: 0,
            reserve: [0; 32],
        };

        // Has capacity
        assert!(validate_farm_space_capacity(&farm_space).is_ok());
        
        // At capacity
        farm_space.seed_count = 8;
        assert_eq!(
            validate_farm_space_capacity(&farm_space).unwrap_err(),
            GameError::FarmSpaceCapacityExceeded.into()
        );
    }

    #[test]
    fn test_validate_has_farm_space() {
        let mut user_state = UserState {
            owner: Pubkey::new_unique(),
            total_grow_power: 0,
            last_harvest_time: 0,
            has_farm_space: true,
            referrer: None,
            pending_referral_rewards: 0,
            reserve: [0; 32],
        };

        // Has farm space
        assert!(validate_has_farm_space(&user_state).is_ok());
        
        // No farm space
        user_state.has_farm_space = false;
        assert_eq!(
            validate_has_farm_space(&user_state).unwrap_err(),
            GameError::NoFarmSpace.into()
        );
    }

    #[test]
    fn test_validate_has_grow_power() {
        let mut user_state = UserState {
            owner: Pubkey::new_unique(),
            total_grow_power: 100,
            last_harvest_time: 0,
            has_farm_space: true,
            referrer: None,
            pending_referral_rewards: 0,
            reserve: [0; 32],
        };

        // Has grow power
        assert!(validate_has_grow_power(&user_state).is_ok());
        
        // No grow power
        user_state.total_grow_power = 0;
        assert_eq!(
            validate_has_grow_power(&user_state).unwrap_err(),
            GameError::NoGrowPower.into()
        );
    }

    // ===== CALCULATION HELPER TESTS =====

    #[test]
    fn test_calculate_reward() {
        // Basic calculation
        let reward = calculate_reward(3600, 100, 10).unwrap(); // 1 hour, 100 GP, 10 rate
        assert_eq!(reward, 3600000); // 3600 * 100 * 10 / 1000
        
        // Zero values
        assert_eq!(calculate_reward(0, 100, 10).unwrap(), 0);
        assert_eq!(calculate_reward(3600, 0, 10).unwrap(), 0);
        assert_eq!(calculate_reward(3600, 100, 0).unwrap(), 0);
        
        // Large values (check overflow handling)
        let large_time = u64::MAX / 1000000;
        let result = calculate_reward(large_time, 100, 100);
        assert!(result.is_err() || result.unwrap() > 0);
    }

    #[test]
    fn test_calculate_referral_rewards() {
        // Standard rewards
        let (level1, level2) = calculate_referral_rewards(1000).unwrap();
        assert_eq!(level1, 100); // 10%
        assert_eq!(level2, 50);  // 5%
        
        // Zero base reward
        let (level1, level2) = calculate_referral_rewards(0).unwrap();
        assert_eq!(level1, 0);
        assert_eq!(level2, 0);
        
        // Large values
        let (level1, level2) = calculate_referral_rewards(1_000_000).unwrap();
        assert_eq!(level1, 100_000); // 10%
        assert_eq!(level2, 50_000);  // 5%
    }

    #[test]
    fn test_calculate_user_share_of_global_rewards() {
        // Standard case
        let reward = calculate_user_share_of_global_rewards(100, 1000, 10, 3600).unwrap();
        assert_eq!(reward, 3600); // (100/1000) * 10 * 3600
        
        // Edge cases
        assert_eq!(calculate_user_share_of_global_rewards(0, 1000, 10, 3600).unwrap(), 0);
        assert_eq!(calculate_user_share_of_global_rewards(100, 0, 10, 3600).unwrap(), 0);
        assert_eq!(calculate_user_share_of_global_rewards(100, 1000, 0, 3600).unwrap(), 0);
        
        // User has all grow power
        let reward = calculate_user_share_of_global_rewards(1000, 1000, 10, 3600).unwrap();
        assert_eq!(reward, 36000); // 100% of rewards
    }

    #[test]
    fn test_calculate_user_rewards_across_halving() {
        let user_grow_power = 100;
        let global_grow_power = 1000;
        let base_rate = 100;
        let halving_interval = 86400; // 1 day
        
        // Test within single period (no halving)
        let last_harvest = 0;
        let current_time = 43200; // 12 hours
        let next_halving = 86400; // 1 day
        
        let reward = calculate_user_rewards_across_halving(
            user_grow_power,
            global_grow_power,
            base_rate,
            last_harvest,
            current_time,
            next_halving,
            halving_interval,
        ).unwrap();
        
        // Should be (100/1000) * 100 * 43200 = 432000
        assert_eq!(reward, 432000);
        
        // Test across one halving
        let last_harvest = 0;
        let current_time = 129600; // 1.5 days
        let next_halving = 86400; // 1 day
        
        let reward = calculate_user_rewards_across_halving(
            user_grow_power,
            global_grow_power,
            base_rate,
            last_harvest,
            current_time,
            next_halving,
            halving_interval,
        ).unwrap();
        
        // First day: (100/1000) * 100 * 86400 = 864000
        // Half day after halving: (100/1000) * 50 * 43200 = 216000
        // Total: 1080000
        assert_eq!(reward, 1080000);
    }

    #[test]
    fn test_get_upgrade_cost_for_level() {
        assert_eq!(get_upgrade_cost_for_level(1).unwrap(), 3_500_000_000);
        assert_eq!(get_upgrade_cost_for_level(2).unwrap(), 18_000_000_000);
        assert_eq!(get_upgrade_cost_for_level(3).unwrap(), 20_000_000_000);
        assert_eq!(get_upgrade_cost_for_level(4).unwrap(), 25_000_000_000);
        assert!(get_upgrade_cost_for_level(5).is_err());
        assert!(get_upgrade_cost_for_level(6).is_err());
    }

    #[test]
    fn test_check_and_apply_halving() {
        let current_rate = 100;
        let halving_interval = 86400;
        
        // Before halving
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            50000,
            86400,
            current_rate,
            halving_interval,
        );
        assert!(!should_halve);
        assert_eq!(new_rate, current_rate);
        assert_eq!(new_time, 86400);
        
        // At halving time
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            86400,
            86400,
            current_rate,
            halving_interval,
        );
        assert!(should_halve);
        assert_eq!(new_rate, 50); // Halved
        assert_eq!(new_time, 172800); // Next halving
        
        // After halving
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            100000,
            86400,
            current_rate,
            halving_interval,
        );
        assert!(should_halve);
        assert_eq!(new_rate, 50);
        assert_eq!(new_time, 172800);
    }

    #[test]
    fn test_calculate_transfer_fee() {
        // Standard 2% fee
        let (fee, transfer_amount) = calculate_transfer_fee(1000).unwrap();
        assert_eq!(fee, 20); // 2%
        assert_eq!(transfer_amount, 980);
        
        // Minimum amounts
        let (fee, transfer_amount) = calculate_transfer_fee(50).unwrap();
        assert_eq!(fee, 1); // 2% of 50
        assert_eq!(transfer_amount, 49);
        
        // Zero amount
        let (fee, transfer_amount) = calculate_transfer_fee(0).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(transfer_amount, 0);
    }

    // ===== FARM SPACE HELPER TESTS =====

    #[test]
    fn test_initialize_farm_space_level_1() {
        let owner = Pubkey::new_unique();
        let mut farm_space = FarmSpace {
            owner: Pubkey::default(),
            level: 0,
            capacity: 0,
            seed_count: 0,
            total_grow_power: 0,
            upgrade_start_time: 0,
            upgrade_target_level: 0,
            reserve: [0; 32],
        };

        initialize_farm_space_level_1(&mut farm_space, owner).unwrap();

        assert_eq!(farm_space.owner, owner);
        assert_eq!(farm_space.level, 1);
        assert_eq!(farm_space.capacity, 4);
        assert_eq!(farm_space.seed_count, 1);
        assert_eq!(farm_space.total_grow_power, 100); // Seed1 power
        assert_eq!(farm_space.upgrade_start_time, 0);
        assert_eq!(farm_space.upgrade_target_level, 0);
    }

    #[test]
    fn test_update_global_grow_power() {
        let mut global_stats = GlobalStats {
            total_grow_power: 1000,
            total_farm_spaces: 10,
            total_supply: 1_000_000_000_000_000,
            current_rewards_per_second: 100,
            last_update_time: 0,
            reserve: [0; 32],
        };

        // Positive change
        update_global_grow_power(&mut global_stats, 500, 1000).unwrap();
        assert_eq!(global_stats.total_grow_power, 1500);
        assert_eq!(global_stats.last_update_time, 1000);

        // Negative change
        update_global_grow_power(&mut global_stats, -300, 2000).unwrap();
        assert_eq!(global_stats.total_grow_power, 1200);
        assert_eq!(global_stats.last_update_time, 2000);

        // Prevent underflow
        update_global_grow_power(&mut global_stats, -2000, 3000).unwrap();
        assert_eq!(global_stats.total_grow_power, 0);
        assert_eq!(global_stats.last_update_time, 3000);
    }

    // ===== SEED MANAGEMENT HELPER TESTS =====

    #[test]
    fn test_initialize_seed_storage() {
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner: Pubkey::default(),
            seed_ids: vec![],
            total_seeds: 0,
            reserve: [0; 32],
        };

        initialize_seed_storage(&mut seed_storage, owner);

        assert_eq!(seed_storage.owner, owner);
        assert_eq!(seed_storage.seed_ids.len(), 0);
        assert_eq!(seed_storage.total_seeds, 0);
    }

    #[test]
    fn test_add_seed_to_storage() {
        let mut seed_storage = SeedStorage {
            owner: Pubkey::new_unique(),
            seed_ids: vec![],
            total_seeds: 0,
            reserve: [0; 32],
        };

        // Add seeds
        for i in 1..=10 {
            add_seed_to_storage(&mut seed_storage, i).unwrap();
            assert_eq!(seed_storage.seed_ids.len(), i as usize);
            assert_eq!(seed_storage.total_seeds, i as u16);
        }

        // Test capacity limit
        seed_storage.seed_ids = vec![0; 100]; // Fill to max
        seed_storage.total_seeds = 100;
        
        assert_eq!(
            add_seed_to_storage(&mut seed_storage, 101).unwrap_err(),
            GameError::InvalidQuantity.into()
        );
    }

    #[test]
    fn test_derive_seed_randomness() {
        let base_entropy = 0x123456789ABCDEF0u64;
        
        // Different indices should produce different values
        let random1 = derive_seed_randomness(base_entropy, 0);
        let random2 = derive_seed_randomness(base_entropy, 1);
        let random3 = derive_seed_randomness(base_entropy, 2);
        
        assert_ne!(random1, random2);
        assert_ne!(random2, random3);
        assert_ne!(random1, random3);
        
        // Same inputs should produce same output (deterministic)
        let random1_again = derive_seed_randomness(base_entropy, 0);
        assert_eq!(random1, random1_again);
    }

    // ===== PYTH ENTROPY HELPER TESTS =====

    #[test]
    fn test_derive_entropy_request_key() {
        let user = Pubkey::new_unique();
        let sequence = 12345u64;
        let pyth_program = Pubkey::new_unique();
        
        let (pda1, bump1) = derive_entropy_request_key(user, sequence, pyth_program);
        
        // Verify deterministic
        let (pda2, bump2) = derive_entropy_request_key(user, sequence, pyth_program);
        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
        
        // Different inputs produce different PDAs
        let (pda3, _) = derive_entropy_request_key(user, sequence + 1, pyth_program);
        assert_ne!(pda1, pda3);
    }

    #[test]
    fn test_validate_sufficient_balance() {
        // Sufficient balance
        assert!(validate_sufficient_balance(1000, 500).is_ok());
        assert!(validate_sufficient_balance(1000, 1000).is_ok());
        
        // Insufficient balance
        assert_eq!(
            validate_sufficient_balance(1000, 1001).unwrap_err(),
            GameError::InsufficientFunds.into()
        );
    }
}