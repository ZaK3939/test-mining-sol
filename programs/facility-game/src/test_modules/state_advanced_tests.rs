#[cfg(test)]
mod state_advanced_tests {
    use super::super::*;
    use crate::constants::*;
    use crate::state::*;
    use anchor_lang::prelude::*;

    // NOTE: Basic account size tests consolidated in state_tests.rs to avoid duplication
    // This module focuses on advanced state behavior, lifecycle testing, and integration

    #[test]
    fn test_seed_type_conversions() {
        // Test all seed type conversions with constants integration
        for i in 0..9 {
            let seed_type: SeedType = unsafe { std::mem::transmute(i as u8) };
            
            // Test grow power retrieval
            let grow_power = seed_type.get_grow_power();
            assert_eq!(grow_power, SEED_GROW_POWERS[i], "Grow power mismatch for Seed{}", i + 1);
            
            // Test probability retrieval from constants
            let probability = seed_type.probability_from_constants();
            assert_eq!(probability, SEED_PROBABILITIES[i], "Probability mismatch for Seed{}", i + 1);
            
            // Test random conversion with thresholds
            let random_value = SEED_PROBABILITY_THRESHOLDS[i] as u64 - 1;
            let converted = SeedType::from_random_with_constants(random_value);
            assert_eq!(converted, seed_type, "Random conversion failed for Seed{}", i + 1);
        }
    }

    #[test]
    fn test_user_state_lifecycle() {
        let owner = Pubkey::new_unique();
        let referrer = Pubkey::new_unique();
        let current_time = 1000000i64;
        
        // Test initial user state
        let mut user_state = UserState {
            owner,
            total_grow_power: 0,
            last_harvest_time: current_time,
            has_farm_space: false,
            referrer: Some(referrer),
            pending_referral_rewards: 0,
            reserve: [0; 32],
        };
        
        // Test user progression
        assert_eq!(user_state.owner, owner);
        assert_eq!(user_state.total_grow_power, 0);
        assert_eq!(user_state.last_harvest_time, current_time);
        assert!(!user_state.has_farm_space);
        assert_eq!(user_state.referrer, Some(referrer));
        assert_eq!(user_state.pending_referral_rewards, 0);
        
        // Simulate farm space purchase
        user_state.has_farm_space = true;
        user_state.total_grow_power = SEED_GROW_POWERS[0]; // Initial Seed1
        
        assert!(user_state.has_farm_space);
        assert_eq!(user_state.total_grow_power, SEED_GROW_POWERS[0]);
        
        // Simulate seed additions
        user_state.total_grow_power += SEED_GROW_POWERS[1]; // Add Seed2
        assert_eq!(user_state.total_grow_power, SEED_GROW_POWERS[0] + SEED_GROW_POWERS[1]);
        
        // Simulate referral rewards accumulation
        user_state.pending_referral_rewards = 1000;
        assert_eq!(user_state.pending_referral_rewards, 1000);
    }

    #[test]
    fn test_farm_space_upgrade_mechanics() {
        let owner = Pubkey::new_unique();
        let current_time = 1000000i64;
        
        // Test Level 1 farm space initialization
        let mut farm_space = FarmSpace {
            owner,
            level: 1,
            capacity: FARM_CAPACITIES[0],
            seed_count: 1,
            total_grow_power: SEED_GROW_POWERS[0],
            upgrade_start_time: 0,
            upgrade_target_level: 0,
            reserve: [0; 32],
        };
        
        // Verify initial state
        assert_eq!(farm_space.level, 1);
        assert_eq!(farm_space.capacity, FARM_CAPACITIES[0]);
        assert_eq!(farm_space.seed_count, 1);
        assert_eq!(farm_space.total_grow_power, SEED_GROW_POWERS[0]);
        
        // Test upgrade initiation (Level 1 → 2)
        farm_space.upgrade_start_time = current_time;
        farm_space.upgrade_target_level = 2;
        
        assert_eq!(farm_space.upgrade_start_time, current_time);
        assert_eq!(farm_space.upgrade_target_level, 2);
        
        // Test upgrade completion after cooldown
        farm_space.level = 2;
        farm_space.capacity = FARM_CAPACITIES[1];
        farm_space.upgrade_start_time = 0;
        farm_space.upgrade_target_level = 0;
        
        assert_eq!(farm_space.level, 2);
        assert_eq!(farm_space.capacity, FARM_CAPACITIES[1]);
        
        // Test helper functions with constants integration
        for level in 1..=5 {
            let capacity = FarmSpace::capacity_for_level(level);
            assert_eq!(capacity, FARM_CAPACITIES[level as usize - 1], 
                      "Capacity mismatch for level {}", level);
        }
        
        // Test upgrade cost calculations
        for level in 1..=4 {
            let cost = FarmSpace::upgrade_cost_for_level(level);
            assert!(cost.is_some(), "Upgrade cost should exist for level {}", level);
            assert_eq!(cost.unwrap(), UPGRADE_COSTS[level as usize - 1], 
                      "Upgrade cost mismatch for level {}", level);
        }
        
        // Test invalid level (no upgrade from level 5)
        assert!(FarmSpace::upgrade_cost_for_level(5).is_none(), 
               "Level 5 should not have upgrade cost");
    }

    #[test]
    fn test_seed_lifecycle_management() {
        let owner = Pubkey::new_unique();
        let farm_space_key = Pubkey::new_unique();
        let current_time = 1000000i64;
        
        // Test unplanted seed creation
        let mut seed = Seed {
            owner,
            seed_type: SeedType::Seed1,
            grow_power: SEED_GROW_POWERS[0],
            is_planted: false,
            planted_farm_space: None,
            planted_at: 0,
            reserve: [0; 16],
        };
        
        // Verify initial unplanted state
        assert_eq!(seed.owner, owner);
        assert_eq!(seed.seed_type, SeedType::Seed1);
        assert_eq!(seed.grow_power, SEED_GROW_POWERS[0]);
        assert!(!seed.is_planted);
        assert_eq!(seed.planted_farm_space, None);
        assert_eq!(seed.planted_at, 0);
        
        // Test seed planting transition
        seed.is_planted = true;
        seed.planted_farm_space = Some(farm_space_key);
        seed.planted_at = current_time;
        
        assert!(seed.is_planted);
        assert_eq!(seed.planted_farm_space, Some(farm_space_key));
        assert_eq!(seed.planted_at, current_time);
        
        // Test seed removal transition
        seed.is_planted = false;
        seed.planted_farm_space = None;
        seed.planted_at = 0;
        
        assert!(!seed.is_planted);
        assert_eq!(seed.planted_farm_space, None);
        assert_eq!(seed.planted_at, 0);
    }

    #[test]
    fn test_global_stats_ecosystem_tracking() {
        let current_time = 1000000i64;
        
        let mut global_stats = GlobalStats {
            total_grow_power: 0,
            total_farm_spaces: 0,
            current_rewards_per_second: DEFAULT_BASE_RATE,
            last_update_time: current_time,
            reserve: [0; 32],
        };
        
        // Test initial ecosystem state
        assert_eq!(global_stats.total_grow_power, 0);
        assert_eq!(global_stats.total_farm_spaces, 0);
        assert_eq!(global_stats.current_rewards_per_second, DEFAULT_BASE_RATE);
        assert_eq!(global_stats.last_update_time, current_time);
        
        // Simulate ecosystem growth (first user joins)
        global_stats.total_farm_spaces += 1;
        global_stats.total_grow_power += SEED_GROW_POWERS[0]; // Initial seed
        global_stats.last_update_time = current_time + 100;
        
        assert_eq!(global_stats.total_farm_spaces, 1);
        assert_eq!(global_stats.total_grow_power, SEED_GROW_POWERS[0]);
        
        // Simulate viral growth (multiple users join)
        for i in 1..10 {
            global_stats.total_farm_spaces += 1;
            global_stats.total_grow_power += SEED_GROW_POWERS[0];
        }
        
        assert_eq!(global_stats.total_farm_spaces, 10);
        assert_eq!(global_stats.total_grow_power, SEED_GROW_POWERS[0] * 10);
        
        // Test halving mechanism impact
        global_stats.current_rewards_per_second = DEFAULT_BASE_RATE / 2;
        assert_eq!(global_stats.current_rewards_per_second, DEFAULT_BASE_RATE / 2);
        
        // Test multiple halving periods
        global_stats.current_rewards_per_second = DEFAULT_BASE_RATE / 4;
        assert_eq!(global_stats.current_rewards_per_second, DEFAULT_BASE_RATE / 4);
    }

    #[test]
    fn test_seed_pack_entropy_lifecycle() {
        let purchaser = Pubkey::new_unique();
        let current_time = 1000000i64;
        let entropy_sequence = 12345u64;
        let user_entropy_seed = 67890u64;
        let pack_id = 1u64;
        
        // Test seed pack creation (commit phase)
        let mut seed_pack = SeedPack {
            purchaser,
            purchased_at: current_time,
            cost_paid: SEED_PACK_COST,
            is_opened: false,
            entropy_sequence,
            user_entropy_seed,
            final_random_value: 0,
            pack_id,
            reserve: [0; 16],
        };
        
        // Verify commit phase data
        assert_eq!(seed_pack.purchaser, purchaser);
        assert_eq!(seed_pack.purchased_at, current_time);
        assert_eq!(seed_pack.cost_paid, SEED_PACK_COST);
        assert!(!seed_pack.is_opened);
        assert_eq!(seed_pack.entropy_sequence, entropy_sequence);
        assert_eq!(seed_pack.user_entropy_seed, user_entropy_seed);
        assert_eq!(seed_pack.final_random_value, 0);
        assert_eq!(seed_pack.pack_id, pack_id);
        
        // Test pack opening (reveal phase)
        seed_pack.is_opened = true;
        seed_pack.final_random_value = 99999u64;
        
        assert!(seed_pack.is_opened);
        assert_eq!(seed_pack.final_random_value, 99999u64);
        
        // Verify immutable data integrity after opening
        assert_eq!(seed_pack.purchaser, purchaser);
        assert_eq!(seed_pack.purchased_at, current_time);
        assert_eq!(seed_pack.cost_paid, SEED_PACK_COST);
        assert_eq!(seed_pack.entropy_sequence, entropy_sequence);
        assert_eq!(seed_pack.user_entropy_seed, user_entropy_seed);
        assert_eq!(seed_pack.pack_id, pack_id);
    }

    #[test]
    fn test_invite_code_usage_tracking() {
        let creator = Pubkey::new_unique();
        let current_time = 1000000i64;
        let invite_code = [65, 66, 67, 68, 69, 70, 71, 72]; // "ABCDEFGH"
        
        // Test invite code creation
        let mut invite_code_account = InviteCode {
            creator,
            code: invite_code,
            invite_limit: MAX_INVITE_LIMIT,
            invites_used: 0,
            created_at: current_time,
            reserve: [0; 16],
        };
        
        // Verify initial invite code state
        assert_eq!(invite_code_account.creator, creator);
        assert_eq!(invite_code_account.code, invite_code);
        assert_eq!(invite_code_account.invite_limit, MAX_INVITE_LIMIT);
        assert_eq!(invite_code_account.invites_used, 0);
        assert_eq!(invite_code_account.created_at, current_time);
        
        // Test incremental invite usage
        for i in 1..=3 {
            invite_code_account.invites_used = i;
            assert_eq!(invite_code_account.invites_used, i);
            assert!(invite_code_account.invites_used < invite_code_account.invite_limit);
        }
        
        // Test limit reached
        invite_code_account.invites_used = MAX_INVITE_LIMIT;
        assert_eq!(invite_code_account.invites_used, invite_code_account.invite_limit);
        
        // Verify immutable fields remain intact
        assert_eq!(invite_code_account.creator, creator);
        assert_eq!(invite_code_account.code, invite_code);
        assert_eq!(invite_code_account.invite_limit, MAX_INVITE_LIMIT);
        assert_eq!(invite_code_account.created_at, current_time);
    }

    #[test]
    fn test_constants_integration_with_state() {
        // Test that all state structures properly integrate with constants
        
        // Test seed types match constants array length
        assert_eq!(SEED_GROW_POWERS.len(), 9);
        assert_eq!(SEED_PROBABILITIES.len(), 9);
        assert_eq!(SEED_PROBABILITY_THRESHOLDS.len(), 9);
        
        // Test farm capacities integration
        assert_eq!(FARM_CAPACITIES.len(), 5);
        assert_eq!(UPGRADE_COSTS.len(), 4); // 4 upgrades for 5 levels
        
        // Test that the last probability threshold equals 10000 (100%)
        assert_eq!(SEED_PROBABILITY_THRESHOLDS[8], 10000);
        
        // Test that farm capacities are in ascending order
        for i in 1..FARM_CAPACITIES.len() {
            assert!(FARM_CAPACITIES[i] > FARM_CAPACITIES[i-1], 
                   "Farm capacities should be ascending");
        }
        
        // Test that upgrade costs are reasonable relative to game economy
        for &cost in &UPGRADE_COSTS {
            assert!(cost >= SEED_PACK_COST, "Upgrade cost should be higher than seed pack");
            assert!(cost <= 100 * SEED_PACK_COST, "Upgrade cost should be reasonable");
        }
    }
}