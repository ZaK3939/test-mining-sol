#[cfg(test)]
mod state_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::error::GameError;

    // ===== CONFIG TESTS =====

    #[test]
    fn test_config_constants() {
        assert_eq!(Config::DEFAULT_FARM_SPACE_COST, 500_000_000); // 0.5 SOL
        assert_eq!(Config::DEFAULT_BASE_RATE, 100);
        assert_eq!(Config::DEFAULT_HALVING_INTERVAL, 6 * 24 * 60 * 60); // 6 days

        // Test account size calculation
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
            2; // reserve

        assert_eq!(Config::LEN, expected_len);
    }

    // ===== USER_STATE TESTS =====

    #[test]
    fn test_user_state_len() {
        let expected_len = 8 + // discriminator
            32 + // owner
            8 + // total_grow_power
            8 + // last_harvest_time
            1 + // has_farm_space
            1 + 32 + // referrer (Option<Pubkey>)
            8 + // pending_referral_rewards
            32; // reserve

        assert_eq!(UserState::LEN, expected_len);
    }

    // ===== FARM_SPACE TESTS =====

    #[test]
    fn test_farm_space_constants() {
        assert_eq!(FarmSpace::UPGRADE_COOLDOWN, 24 * 60 * 60); // 24 hours

        let expected_len = 8 + // discriminator
            32 + // owner
            1 + // level
            1 + // capacity
            1 + // seed_count
            8 + // total_grow_power
            8 + // upgrade_start_time
            1 + // upgrade_target_level
            32; // reserve

        assert_eq!(FarmSpace::LEN, expected_len);
    }

    #[test]
    fn test_farm_space_capacity_for_level() {
        assert_eq!(FarmSpace::get_capacity_for_level(1), 4);
        assert_eq!(FarmSpace::get_capacity_for_level(2), 8);
        assert_eq!(FarmSpace::get_capacity_for_level(3), 12);
        assert_eq!(FarmSpace::get_capacity_for_level(4), 16);
        assert_eq!(FarmSpace::get_capacity_for_level(5), 20);
        assert_eq!(FarmSpace::get_capacity_for_level(6), 20); // Max capacity
        assert_eq!(FarmSpace::get_capacity_for_level(10), 20); // Max capacity
    }

    #[test]
    fn test_farm_space_upgrade_cost() {
        assert_eq!(FarmSpace::get_upgrade_cost(1), Some(3500));
        assert_eq!(FarmSpace::get_upgrade_cost(2), Some(18000));
        assert_eq!(FarmSpace::get_upgrade_cost(3), Some(20000));
        assert_eq!(FarmSpace::get_upgrade_cost(4), Some(25000));
        assert_eq!(FarmSpace::get_upgrade_cost(5), None);
        assert_eq!(FarmSpace::get_upgrade_cost(6), None);
    }

    #[test]
    fn test_farm_space_upgrade_completion() {
        let farm_space = FarmSpace {
            owner: Pubkey::new_unique(),
            level: 1,
            capacity: 4,
            seed_count: 0,
            total_grow_power: 0,
            upgrade_start_time: 1000,
            upgrade_target_level: 2,
            reserve: [0; 32],
        };

        // Not complete before cooldown
        assert!(!farm_space.is_upgrade_complete(1000 + FarmSpace::UPGRADE_COOLDOWN - 1));
        
        // Complete exactly at cooldown time
        assert!(farm_space.is_upgrade_complete(1000 + FarmSpace::UPGRADE_COOLDOWN));
        
        // Complete after cooldown time
        assert!(farm_space.is_upgrade_complete(1000 + FarmSpace::UPGRADE_COOLDOWN + 1000));

        // Test no upgrade in progress
        let no_upgrade_farm = FarmSpace {
            upgrade_start_time: 0,
            ..farm_space
        };
        assert!(!no_upgrade_farm.is_upgrade_complete(10000));
    }

    // ===== SEED_TYPE TESTS =====

    #[test]
    fn test_seed_type_grow_powers() {
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
    fn test_seed_type_probabilities() {
        assert_eq!(SeedType::Seed1.get_probability_percent(), 42.23);
        assert_eq!(SeedType::Seed2.get_probability_percent(), 24.44);
        assert_eq!(SeedType::Seed3.get_probability_percent(), 13.33);
        assert_eq!(SeedType::Seed4.get_probability_percent(), 8.33);
        assert_eq!(SeedType::Seed5.get_probability_percent(), 5.56);
        assert_eq!(SeedType::Seed6.get_probability_percent(), 3.33);
        assert_eq!(SeedType::Seed7.get_probability_percent(), 1.33);
        assert_eq!(SeedType::Seed8.get_probability_percent(), 0.89);
        assert_eq!(SeedType::Seed9.get_probability_percent(), 0.56);
    }

    #[test]
    fn test_seed_type_from_random() {
        // Test threshold boundaries for Seed1 (0-4221)
        assert_eq!(SeedType::from_random(0), SeedType::Seed1);
        assert_eq!(SeedType::from_random(4221), SeedType::Seed1);
        
        // Test threshold boundaries for Seed2 (4222-6665)
        assert_eq!(SeedType::from_random(4222), SeedType::Seed2);
        assert_eq!(SeedType::from_random(6665), SeedType::Seed2);
        
        // Test threshold boundaries for Seed3 (6666-7998)
        assert_eq!(SeedType::from_random(6666), SeedType::Seed3);
        assert_eq!(SeedType::from_random(7998), SeedType::Seed3);
        
        // Test threshold boundaries for Seed9 (9943-9999)
        assert_eq!(SeedType::from_random(9943), SeedType::Seed9);
        assert_eq!(SeedType::from_random(9999), SeedType::Seed9);
        
        // Test modulo behavior with large numbers
        assert_eq!(SeedType::from_random(10000), SeedType::Seed1); // 10000 % 10000 = 0
        assert_eq!(SeedType::from_random(14222), SeedType::Seed2); // 14222 % 10000 = 4222
        assert_eq!(SeedType::from_random(u64::MAX), SeedType::from_random(u64::MAX % 10000));
    }

    #[test]
    fn test_seed_type_all_types() {
        let all_types = SeedType::all_types();
        assert_eq!(all_types.len(), 9);
        assert_eq!(all_types[0], SeedType::Seed1);
        assert_eq!(all_types[8], SeedType::Seed9);
    }

    #[test]
    fn test_seed_type_is_valid() {
        for i in 0..9 {
            assert!(SeedType::is_valid(i));
        }
        assert!(!SeedType::is_valid(9));
        assert!(!SeedType::is_valid(10));
        assert!(!SeedType::is_valid(255));
    }

    #[test]
    fn test_seed_type_probability_distribution() {
        // Test that probabilities add up correctly (within floating point precision)
        let total_probability: f32 = SeedType::all_types()
            .iter()
            .map(|seed| seed.get_probability_percent())
            .sum();
        
        // Should be very close to 100% (allowing for floating point precision)
        assert!((total_probability - 100.0).abs() < 0.01);
    }

    // ===== SEED TESTS =====

    #[test]
    fn test_seed_len() {
        let expected_len = 8 + // discriminator
            32 + // owner
            1 + // seed_type
            8 + // grow_power
            1 + // is_planted
            1 + 32 + // planted_farm_space (Option<Pubkey>)
            8 + // created_at
            8 + // seed_id
            32; // reserve

        assert_eq!(Seed::LEN, expected_len);
    }

    // ===== SEED_PACK TESTS =====

    #[test]
    fn test_seed_pack_len() {
        let expected_len = 8 + // discriminator
            32 + // purchaser
            8 + // purchased_at
            8 + // cost_paid
            1 + // is_opened
            8 + // entropy_sequence
            8 + // user_entropy_seed
            8 + // final_random_value
            8 + // pack_id
            16; // reserve

        assert_eq!(SeedPack::LEN, expected_len);
    }

    // ===== SEED_STORAGE TESTS =====

    #[test]
    fn test_seed_storage_constants() {
        assert_eq!(SeedStorage::MAX_SEEDS, 100);

        let expected_len = 8 + // discriminator
            32 + // owner
            4 + (8 * 100) + // seed_ids (Vec<u64> with max 100 seeds)
            2 + // total_seeds
            32; // reserve

        assert_eq!(SeedStorage::LEN, expected_len);
    }

    #[test]
    fn test_seed_storage_capacity() {
        let mut storage = SeedStorage {
            owner: Pubkey::new_unique(),
            seed_ids: vec![],
            total_seeds: 0,
            reserve: [0; 32],
        };

        // Initially has capacity
        assert!(storage.can_add_seed());

        // Fill to near max
        for i in 0..99 {
            storage.seed_ids.push(i);
        }
        assert!(storage.can_add_seed());

        // Fill to max
        storage.seed_ids.push(99);
        assert!(!storage.can_add_seed());
    }

    #[test]
    fn test_seed_storage_add_seed() {
        let mut storage = SeedStorage {
            owner: Pubkey::new_unique(),
            seed_ids: vec![],
            total_seeds: 0,
            reserve: [0; 32],
        };

        // Add seeds successfully
        assert!(storage.add_seed(1).is_ok());
        assert_eq!(storage.seed_ids.len(), 1);
        assert_eq!(storage.total_seeds, 1);
        assert_eq!(storage.seed_ids[0], 1);

        assert!(storage.add_seed(2).is_ok());
        assert_eq!(storage.seed_ids.len(), 2);
        assert_eq!(storage.total_seeds, 2);

        // Fill to max capacity
        for i in 3..=100 {
            assert!(storage.add_seed(i).is_ok());
        }
        assert_eq!(storage.seed_ids.len(), 100);
        assert_eq!(storage.total_seeds, 100);

        // Try to add beyond capacity - should fail
        assert!(storage.add_seed(101).is_err());
        assert_eq!(storage.seed_ids.len(), 100); // Unchanged
        assert_eq!(storage.total_seeds, 100); // Unchanged
    }

    #[test]
    fn test_seed_storage_remove_seed() {
        let mut storage = SeedStorage {
            owner: Pubkey::new_unique(),
            seed_ids: vec![1, 2, 3, 4, 5],
            total_seeds: 5,
            reserve: [0; 32],
        };

        // Remove existing seed
        assert!(storage.remove_seed(3));
        assert_eq!(storage.seed_ids, vec![1, 2, 4, 5]);
        assert_eq!(storage.total_seeds, 4);

        // Remove first seed
        assert!(storage.remove_seed(1));
        assert_eq!(storage.seed_ids, vec![2, 4, 5]);
        assert_eq!(storage.total_seeds, 3);

        // Remove last seed
        assert!(storage.remove_seed(5));
        assert_eq!(storage.seed_ids, vec![2, 4]);
        assert_eq!(storage.total_seeds, 2);

        // Try to remove non-existent seed
        assert!(!storage.remove_seed(10));
        assert_eq!(storage.seed_ids, vec![2, 4]); // Unchanged
        assert_eq!(storage.total_seeds, 2); // Unchanged

        // Remove remaining seeds
        assert!(storage.remove_seed(2));
        assert!(storage.remove_seed(4));
        assert_eq!(storage.seed_ids.len(), 0);
        assert_eq!(storage.total_seeds, 0);

        // Try to remove from empty storage
        assert!(!storage.remove_seed(1));
    }

    // ===== INVITE_CODE TESTS =====

    #[test]
    fn test_invite_code_len() {
        let expected_len = 8 + // discriminator
            32 + // inviter
            1 + // invites_used
            1 + // invite_limit
            8 + // code
            8 + // created_at
            32; // reserve

        assert_eq!(InviteCode::LEN, expected_len);
    }

    // ===== GLOBAL_STATS TESTS =====

    #[test]
    fn test_global_stats_constants() {
        assert_eq!(GlobalStats::INITIAL_TOTAL_SUPPLY, 1_000_000_000 * 1_000_000); // 1B WEED

        let expected_len = 8 + // discriminator
            8 + // total_grow_power
            8 + // total_farm_spaces
            8 + // total_supply
            8 + // current_rewards_per_second
            8 + // last_update_time
            32; // reserve

        assert_eq!(GlobalStats::LEN, expected_len);
    }

    // ===== FEE_POOL TESTS =====

    #[test]
    fn test_fee_pool_len() {
        let expected_len = 8 + // discriminator
            8 + // accumulated_fees
            32 + // treasury_address
            8 + // last_collection_time
            48; // reserve

        assert_eq!(FeePool::LEN, expected_len);
    }

    // ===== REWARD_ACCOUNT TESTS =====

    #[test]
    fn test_reward_account_len() {
        let expected_len = 8 + // discriminator
            32 + // user
            8 + // claimable_amount
            8 + // last_harvest_time
            8 + // referral_rewards_l1
            8 + // referral_rewards_l2
            32; // reserve

        assert_eq!(RewardAccount::LEN, expected_len);
    }

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_seed_type_random_distribution() {
        // Test that random distribution roughly matches expected probabilities
        let mut counts = [0u32; 9];
        let num_samples = 100000;

        for i in 0..num_samples {
            let seed_type = SeedType::from_random(i);
            counts[seed_type as usize] += 1;
        }

        // Check that Seed1 (most common) has the highest count
        let seed1_count = counts[0];
        assert!(seed1_count > counts[1]); // More than Seed2
        assert!(seed1_count > counts[8]); // Much more than Seed9

        // Check that Seed9 (rarest) has the lowest count
        let seed9_count = counts[8];
        assert!(seed9_count < counts[0]); // Less than Seed1
        assert!(seed9_count < counts[4]); // Less than Seed5

        println!("✅ Seed type distribution test passed");
        for (i, count) in counts.iter().enumerate() {
            let percentage = (*count as f32 / num_samples as f32) * 100.0;
            let expected = SeedType::all_types()[i].get_probability_percent();
            println!("   Seed{}: {:.2}% (expected: {:.2}%)", 
                    i + 1, percentage, expected);
        }
    }

    #[test]
    fn test_farm_space_upgrade_flow() {
        let mut farm_space = FarmSpace {
            owner: Pubkey::new_unique(),
            level: 1,
            capacity: FarmSpace::get_capacity_for_level(1),
            seed_count: 2,
            total_grow_power: 280, // 100 + 180
            upgrade_start_time: 0,
            upgrade_target_level: 0,
            reserve: [0; 32],
        };

        // Start upgrade
        let start_time = 1000;
        farm_space.upgrade_start_time = start_time;
        farm_space.upgrade_target_level = 2;

        // Check upgrade status during cooldown
        assert!(!farm_space.is_upgrade_complete(start_time + 12 * 60 * 60)); // 12 hours
        assert!(!farm_space.is_upgrade_complete(start_time + 23 * 60 * 60)); // 23 hours
        
        // Complete upgrade after 24 hours
        assert!(farm_space.is_upgrade_complete(start_time + 24 * 60 * 60));

        // After upgrade completion
        farm_space.level = farm_space.upgrade_target_level;
        farm_space.capacity = FarmSpace::get_capacity_for_level(farm_space.level);
        farm_space.upgrade_start_time = 0;
        farm_space.upgrade_target_level = 0;

        assert_eq!(farm_space.level, 2);
        assert_eq!(farm_space.capacity, 8);
        assert_eq!(farm_space.seed_count, 2); // Seeds remain planted
        assert_eq!(farm_space.total_grow_power, 280); // Grow power unchanged
    }

    #[test]
    fn test_seed_storage_lifecycle() {
        let owner = Pubkey::new_unique();
        let mut storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            reserve: [0; 32],
        };

        // Add some seeds
        let seed_ids = vec![101, 102, 103, 104, 105];
        for &seed_id in &seed_ids {
            assert!(storage.add_seed(seed_id).is_ok());
        }
        
        assert_eq!(storage.seed_ids, seed_ids);
        assert_eq!(storage.total_seeds, 5);

        // Plant some seeds (remove from storage)
        assert!(storage.remove_seed(102)); // Plant seed 102
        assert!(storage.remove_seed(104)); // Plant seed 104
        
        assert_eq!(storage.seed_ids, vec![101, 103, 105]);
        assert_eq!(storage.total_seeds, 3);

        // Add more seeds
        assert!(storage.add_seed(106).is_ok());
        assert!(storage.add_seed(107).is_ok());
        
        assert_eq!(storage.seed_ids, vec![101, 103, 105, 106, 107]);
        assert_eq!(storage.total_seeds, 5);

        // Harvest seeds (remove from farm, add back to storage)
        assert!(storage.remove_seed(106)); // Remove seed 106
        assert!(storage.add_seed(108).is_ok()); // Add new seed 108
        
        assert_eq!(storage.seed_ids, vec![101, 103, 105, 107, 108]);
        assert_eq!(storage.total_seeds, 5);
    }
}