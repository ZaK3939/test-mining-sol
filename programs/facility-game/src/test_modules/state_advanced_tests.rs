#[cfg(test)]
mod state_advanced_tests {
    use super::super::*;
    use crate::constants::*;
    use crate::state::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_account_size_calculations() {
        // Verify account sizes match actual memory layout
        
        // Config account
        let config_size = 8 + // discriminator
            32 + // admin
            32 + // treasury
            32 + // protocol_referral_address
            8 + // base_rate
            8 + // halving_interval
            8 + // next_halving_time
            8 + // seed_pack_cost
            8 + // farm_space_cost_sol
            8 + // seed_pack_counter
            8 + // seed_counter
            8 + // invite_code_counter
            1 + // max_invite_limit
            48; // reserve
        
        assert_eq!(Config::LEN, config_size, "Config size mismatch");

        // UserState account
        let user_state_size = 8 + // discriminator
            32 + // owner
            8 + // total_grow_power
            8 + // last_harvest_time
            1 + // has_farm_space
            1 + 32 + // Option<Pubkey> referrer (1 + 32)
            8 + // pending_referral_rewards
            32; // reserve
        
        assert_eq!(UserState::LEN, user_state_size, "UserState size mismatch");

        // FarmSpace account
        let farm_space_size = 8 + // discriminator
            32 + // owner
            1 + // level
            1 + // capacity
            1 + // seed_count
            8 + // total_grow_power
            8 + // upgrade_start_time
            1 + // upgrade_target_level
            32; // reserve
        
        assert_eq!(FarmSpace::LEN, farm_space_size, "FarmSpace size mismatch");

        // Seed account  
        let seed_size = 8 + // discriminator
            32 + // owner
            1 + // seed_type
            8 + // grow_power
            1 + // is_planted
            1 + 32 + // Option<Pubkey> planted_farm_space (1 + 32)
            8 + // planted_at
            16; // reserve
        
        assert_eq!(Seed::LEN, seed_size, "Seed size mismatch");
    }

    #[test]
    fn test_seed_type_conversions() {
        // Test all seed type conversions
        for i in 0..9 {
            let seed_type: SeedType = unsafe { std::mem::transmute(i as u8) };
            
            // Test grow power retrieval
            let grow_power = seed_type.get_grow_power();
            assert_eq!(grow_power, SEED_GROW_POWERS[i], "Grow power mismatch for Seed{}", i + 1);
            
            // Test probability retrieval
            let probability = seed_type.probability_from_constants();
            assert_eq!(probability, SEED_PROBABILITIES[i], "Probability mismatch for Seed{}", i + 1);
            
            // Test random conversion
            let random_value = SEED_PROBABILITY_THRESHOLDS[i] as u64 - 1;
            let converted = SeedType::from_random_with_constants(random_value);
            assert_eq!(converted, seed_type, "Random conversion failed for Seed{}", i + 1);
        }
    }

    #[test]
    fn test_config_initialization() {
        let admin = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let protocol_address = Pubkey::new_unique();
        
        let mut config = Config {
            admin,
            treasury,
            protocol_referral_address: protocol_address,
            base_rate: DEFAULT_BASE_RATE,
            halving_interval: DEFAULT_HALVING_INTERVAL,
            next_halving_time: 0,
            seed_pack_cost: SEED_PACK_COST,
            farm_space_cost_sol: FARM_SPACE_COST_SOL,
            seed_pack_counter: 0,
            seed_counter: 0,
            invite_code_counter: 0,
            max_invite_limit: MAX_INVITE_LIMIT,
            reserve: [0; 48],
        };
        
        // Test config initialization
        let current_time = 1000000i64;
        config.next_halving_time = current_time + DEFAULT_HALVING_INTERVAL;
        
        assert_eq!(config.admin, admin);
        assert_eq!(config.treasury, treasury);
        assert_eq!(config.protocol_referral_address, protocol_address);
        assert_eq!(config.base_rate, DEFAULT_BASE_RATE);
        assert_eq!(config.halving_interval, DEFAULT_HALVING_INTERVAL);
        assert_eq!(config.next_halving_time, current_time + DEFAULT_HALVING_INTERVAL);
        assert_eq!(config.seed_pack_cost, SEED_PACK_COST);
        assert_eq!(config.farm_space_cost_sol, FARM_SPACE_COST_SOL);
        assert_eq!(config.max_invite_limit, MAX_INVITE_LIMIT);
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
        
        // Simulate referral rewards
        user_state.pending_referral_rewards = 1000;
        assert_eq!(user_state.pending_referral_rewards, 1000);
    }

    #[test]
    fn test_farm_space_upgrades() {
        let owner = Pubkey::new_unique();
        let current_time = 1000000i64;
        
        // Test Level 1 farm space
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
        
        assert_eq!(farm_space.level, 1);
        assert_eq!(farm_space.capacity, FARM_CAPACITIES[0]);
        assert_eq!(farm_space.seed_count, 1);
        assert_eq!(farm_space.total_grow_power, SEED_GROW_POWERS[0]);
        assert_eq!(farm_space.upgrade_start_time, 0);
        assert_eq!(farm_space.upgrade_target_level, 0);
        
        // Test upgrade initiation
        farm_space.upgrade_start_time = current_time;
        farm_space.upgrade_target_level = 2;
        
        assert_eq!(farm_space.upgrade_start_time, current_time);
        assert_eq!(farm_space.upgrade_target_level, 2);
        
        // Test upgrade completion
        farm_space.level = 2;
        farm_space.capacity = FARM_CAPACITIES[1];
        farm_space.upgrade_start_time = 0;
        farm_space.upgrade_target_level = 0;
        
        assert_eq!(farm_space.level, 2);
        assert_eq!(farm_space.capacity, FARM_CAPACITIES[1]);
        assert_eq!(farm_space.upgrade_start_time, 0);
        assert_eq!(farm_space.upgrade_target_level, 0);
        
        // Test maximum capacity
        for level in 1..=5 {
            assert_eq!(
                FarmSpace::get_capacity_for_level(level),
                FARM_CAPACITIES[level as usize - 1],
                "Capacity mismatch for level {}", level
            );
        }
        
        // Test upgrade costs
        for level in 1..=4 {
            let cost = FarmSpace::upgrade_cost_for_level(level);
            assert!(cost.is_some(), "Upgrade cost should exist for level {}", level);
            assert_eq!(
                cost.unwrap(),
                UPGRADE_COSTS[level as usize - 1],
                "Upgrade cost mismatch for level {}", level
            );
        }
        
        // Test invalid level
        assert!(FarmSpace::upgrade_cost_for_level(5).is_none(), "Level 5 should not have upgrade cost");
    }

    #[test]
    fn test_seed_lifecycle() {
        let owner = Pubkey::new_unique();
        let farm_space_key = Pubkey::new_unique();
        let current_time = 1000000i64;
        
        // Test unplanted seed
        let mut seed = Seed {
            owner,
            seed_type: SeedType::Seed1,
            grow_power: SEED_GROW_POWERS[0],
            is_planted: false,
            planted_farm_space: None,
            planted_at: 0,
            reserve: [0; 16],
        };
        
        assert_eq!(seed.owner, owner);
        assert_eq!(seed.seed_type, SeedType::Seed1);
        assert_eq!(seed.grow_power, SEED_GROW_POWERS[0]);
        assert!(!seed.is_planted);
        assert_eq!(seed.planted_farm_space, None);
        assert_eq!(seed.planted_at, 0);
        
        // Test seed planting
        seed.is_planted = true;
        seed.planted_farm_space = Some(farm_space_key);
        seed.planted_at = current_time;
        
        assert!(seed.is_planted);
        assert_eq!(seed.planted_farm_space, Some(farm_space_key));
        assert_eq!(seed.planted_at, current_time);
        
        // Test seed removal
        seed.is_planted = false;
        seed.planted_farm_space = None;
        seed.planted_at = 0;
        
        assert!(!seed.is_planted);
        assert_eq!(seed.planted_farm_space, None);
        assert_eq!(seed.planted_at, 0);
    }

    #[test]
    fn test_global_stats_tracking() {
        let current_time = 1000000i64;
        
        let mut global_stats = GlobalStats {
            total_grow_power: 0,
            total_farm_spaces: 0,
            current_rewards_per_second: DEFAULT_BASE_RATE,
            last_update_time: current_time,
            reserve: [0; 32],
        };
        
        // Test initial state
        assert_eq!(global_stats.total_grow_power, 0);
        assert_eq!(global_stats.total_farm_spaces, 0);
        assert_eq!(global_stats.current_rewards_per_second, DEFAULT_BASE_RATE);
        assert_eq!(global_stats.last_update_time, current_time);
        
        // Test farm space addition
        global_stats.total_farm_spaces += 1;
        global_stats.total_grow_power += SEED_GROW_POWERS[0]; // Initial seed
        global_stats.last_update_time = current_time + 100;
        
        assert_eq!(global_stats.total_farm_spaces, 1);
        assert_eq!(global_stats.total_grow_power, SEED_GROW_POWERS[0]);
        assert_eq!(global_stats.last_update_time, current_time + 100);
        
        // Test multiple users
        for i in 1..10 {
            global_stats.total_farm_spaces += 1;
            global_stats.total_grow_power += SEED_GROW_POWERS[0];
        }
        
        assert_eq!(global_stats.total_farm_spaces, 10);
        assert_eq!(global_stats.total_grow_power, SEED_GROW_POWERS[0] * 10);
        
        // Test halving effect on rewards per second
        global_stats.current_rewards_per_second = DEFAULT_BASE_RATE / 2;
        assert_eq!(global_stats.current_rewards_per_second, DEFAULT_BASE_RATE / 2);
    }

    #[test]
    fn test_seed_pack_data_integrity() {
        let purchaser = Pubkey::new_unique();
        let current_time = 1000000i64;
        let entropy_sequence = 12345u64;
        let user_entropy_seed = 67890u64;
        let pack_id = 1u64;
        let cost_paid = SEED_PACK_COST;
        
        let seed_pack = SeedPack {
            purchaser,
            purchased_at: current_time,
            cost_paid,
            is_opened: false,
            entropy_sequence,
            user_entropy_seed,
            final_random_value: 0,
            pack_id,
            reserve: [0; 16],
        };
        
        // Test initial seed pack state
        assert_eq!(seed_pack.purchaser, purchaser);
        assert_eq!(seed_pack.purchased_at, current_time);
        assert_eq!(seed_pack.cost_paid, SEED_PACK_COST);
        assert!(!seed_pack.is_opened);
        assert_eq!(seed_pack.entropy_sequence, entropy_sequence);
        assert_eq!(seed_pack.user_entropy_seed, user_entropy_seed);
        assert_eq!(seed_pack.final_random_value, 0);
        assert_eq!(seed_pack.pack_id, pack_id);
        
        // Test pack opening simulation
        let mut opened_pack = seed_pack;
        opened_pack.is_opened = true;
        opened_pack.final_random_value = 99999u64;
        
        assert!(opened_pack.is_opened);
        assert_eq!(opened_pack.final_random_value, 99999u64);
        
        // Verify immutable fields remain unchanged
        assert_eq!(opened_pack.purchaser, purchaser);
        assert_eq!(opened_pack.purchased_at, current_time);
        assert_eq!(opened_pack.cost_paid, SEED_PACK_COST);
        assert_eq!(opened_pack.entropy_sequence, entropy_sequence);
        assert_eq!(opened_pack.user_entropy_seed, user_entropy_seed);
        assert_eq!(opened_pack.pack_id, pack_id);
    }

    #[test]
    fn test_invite_code_data_integrity() {
        let creator = Pubkey::new_unique();
        let current_time = 1000000i64;
        let invite_code = [65, 66, 67, 68, 69, 70, 71, 72]; // "ABCDEFGH"
        
        let invite_code_account = InviteCode {
            creator,
            code: invite_code,
            invite_limit: MAX_INVITE_LIMIT,
            invites_used: 0,
            created_at: current_time,
            reserve: [0; 16],
        };
        
        // Test initial invite code state
        assert_eq!(invite_code_account.creator, creator);
        assert_eq!(invite_code_account.code, invite_code);
        assert_eq!(invite_code_account.invite_limit, MAX_INVITE_LIMIT);
        assert_eq!(invite_code_account.invites_used, 0);
        assert_eq!(invite_code_account.created_at, current_time);
        
        // Test invite usage simulation
        let mut used_invite = invite_code_account;
        used_invite.invites_used = 3;
        
        assert_eq!(used_invite.invites_used, 3);
        assert!(used_invite.invites_used < used_invite.invite_limit);
        
        // Test limit reached
        used_invite.invites_used = MAX_INVITE_LIMIT;
        assert_eq!(used_invite.invites_used, used_invite.invite_limit);
        
        // Verify immutable fields remain unchanged
        assert_eq!(used_invite.creator, creator);
        assert_eq!(used_invite.code, invite_code);
        assert_eq!(used_invite.invite_limit, MAX_INVITE_LIMIT);
        assert_eq!(used_invite.created_at, current_time);
    }
}