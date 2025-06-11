#[cfg(test)]
mod economics_advanced_tests {
    use super::super::*;
    use crate::economics::*;
    use crate::constants::*;
    use crate::utils::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_cross_halving_reward_calculation() {
        // Test complex scenario with multiple halvings during reward period
        let user_grow_power = 1000u64;
        let global_grow_power = 10000u64;
        let initial_base_rate = 100u64;
        let halving_interval = 3600i64; // 1 hour for testing
        
        // User last claimed 3.5 hours ago (should cross 3 halving boundaries)
        let current_time = 1000000i64;
        let last_harvest_time = current_time - (3 * 3600 + 1800); // 3.5 hours ago
        let first_halving_time = current_time - (3 * 3600); // 3 hours ago
        
        let reward = calculate_user_rewards_across_halving(
            user_grow_power,
            global_grow_power,
            initial_base_rate,
            last_harvest_time,
            current_time,
            first_halving_time,
            halving_interval,
        ).unwrap();
        
        // Manual calculation:
        // Period 1 (0.5h @ 100 rate): 1800s * (1000/10000) * 100 = 18,000
        // Period 2 (1h @ 50 rate): 3600s * (1000/10000) * 50 = 18,000  
        // Period 3 (1h @ 25 rate): 3600s * (1000/10000) * 25 = 9,000
        // Period 4 (1h @ 12 rate): 3600s * (1000/10000) * 12 = 4,320
        // Total: ~49,320
        
        assert!(reward > 45000, "Reward should be > 45,000, got {}", reward);
        assert!(reward < 55000, "Reward should be < 55,000, got {}", reward);
        
        println!("Cross-halving reward: {}", reward);
    }

    #[test]
    fn test_edge_case_reward_calculations() {
        // Test zero grow power
        let reward = calculate_user_rewards_across_halving(
            0, 1000, 100, 1000, 2000, 3000, 3600
        ).unwrap();
        assert_eq!(reward, 0);
        
        // Test zero global grow power
        let reward = calculate_user_rewards_across_halving(
            1000, 0, 100, 1000, 2000, 3000, 3600
        ).unwrap();
        assert_eq!(reward, 0);
        
        // Test zero base rate
        let reward = calculate_user_rewards_across_halving(
            1000, 1000, 0, 1000, 2000, 3000, 3600
        ).unwrap();
        assert_eq!(reward, 0);
        
        // Test same start and end time
        let reward = calculate_user_rewards_across_halving(
            1000, 1000, 100, 1000, 1000, 3000, 3600
        ).unwrap();
        assert_eq!(reward, 0);
        
        // Test 100% of global grow power
        let reward = calculate_user_rewards_across_halving(
            1000, 1000, 100, 1000, 2000, 3000, 3600
        ).unwrap();
        
        let expected = 100 * 1000; // 100 rate * 1000 seconds
        assert_eq!(reward, expected);
    }

    #[test]
    fn test_multiple_halving_scenarios() {
        let user_grow_power = 500u64;
        let global_grow_power = 5000u64;
        let base_rate = 200u64;
        let halving_interval = 1800i64; // 30 minutes
        
        // Test scenarios with different numbers of halvings
        let scenarios = [
            (0, 1800),    // No halving (exactly 30 min)
            (0, 3600),    // 1 halving (exactly 60 min)
            (0, 5400),    // 2 halvings (exactly 90 min)
            (0, 7200),    // 3 halvings (exactly 120 min)
            (0, 9000),    // 4 halvings (exactly 150 min)
        ];
        
        for (start_offset, end_offset) in scenarios {
            let current_time = 1000000i64;
            let last_harvest = current_time - end_offset;
            let next_halving = current_time + start_offset;
            
            let reward = calculate_user_rewards_across_halving(
                user_grow_power,
                global_grow_power,
                base_rate,
                last_harvest,
                current_time,
                next_halving,
                halving_interval,
            ).unwrap();
            
            println!("Scenario {}s: reward = {}", end_offset, reward);
            assert!(reward > 0, "Reward should be positive for {}s scenario", end_offset);
        }
    }

    #[test]
    fn test_reward_calculation_precision() {
        // Test precision with small values
        let small_grow_power = 1u64;
        let large_global_power = 1000000u64;
        let base_rate = 1u64;
        let time_elapsed = 1i64;
        
        let reward = calculate_user_rewards_across_halving(
            small_grow_power,
            large_global_power,
            base_rate,
            0,
            time_elapsed,
            time_elapsed + 3600,
            3600,
        ).unwrap();
        
        // Should be 1 * 1 * (1/1000000) = 0.000001, rounds to 0
        assert_eq!(reward, 0);
        
        // Test precision with large values
        let large_grow_power = 1000000u64;
        let large_global_power = 1000000u64;
        let large_base_rate = 1000u64;
        let long_time = 3600i64;
        
        let reward = calculate_user_rewards_across_halving(
            large_grow_power,
            large_global_power,
            large_base_rate,
            0,
            long_time,
            long_time + 3600,
            3600,
        ).unwrap();
        
        // Should be 3600 * 1000 * (1000000/1000000) = 3,600,000
        assert_eq!(reward, 3600000);
    }

    #[test]
    fn test_referral_reward_calculations() {
        let base_rewards = [100, 1000, 10000, 100000, 1000000];
        
        for base_reward in base_rewards {
            // Test Level 1 referral (10%)
            let level1_reward = calculate_referral_reward_for_level(base_reward, 1).unwrap();
            assert_eq!(level1_reward, base_reward / 10);
            
            // Test Level 2 referral (5%)
            let level2_reward = calculate_referral_reward_for_level(base_reward, 2).unwrap();
            assert_eq!(level2_reward, base_reward / 20);
            
            // Test invalid level
            let invalid_level_reward = calculate_referral_reward_for_level(base_reward, 3).unwrap();
            assert_eq!(invalid_level_reward, 0);
            
            // Test combined calculation
            let (level1, level2) = calculate_referral_rewards(base_reward).unwrap();
            assert_eq!(level1, base_reward / 10);
            assert_eq!(level2, base_reward / 20);
        }
        
        // Test edge cases
        let (level1, level2) = calculate_referral_rewards(0).unwrap();
        assert_eq!(level1, 0);
        assert_eq!(level2, 0);
        
        let (level1, level2) = calculate_referral_rewards(1).unwrap();
        assert_eq!(level1, 0); // 1/10 = 0 (integer division)
        assert_eq!(level2, 0); // 1/20 = 0
        
        let (level1, level2) = calculate_referral_rewards(19).unwrap();
        assert_eq!(level1, 1); // 19/10 = 1
        assert_eq!(level2, 0); // 19/20 = 0
    }

    #[test]
    fn test_trading_fee_calculations() {
        let test_amounts = [100, 1000, 50000, 1000000, 10000000];
        
        for amount in test_amounts {
            let (fee, transfer_amount) = calculate_trading_fee(amount).unwrap();
            
            // Fee should be 2% of amount
            let expected_fee = amount * TRADING_FEE_PERCENTAGE as u64 / 100;
            assert_eq!(fee, expected_fee);
            
            // Transfer amount should be amount minus fee
            let expected_transfer = amount - expected_fee;
            assert_eq!(transfer_amount, expected_transfer);
            
            // Fee + transfer should equal original amount
            assert_eq!(fee + transfer_amount, amount);
            
            // Fee should be reasonable (2% exactly)
            assert_eq!(fee * 100 / amount, TRADING_FEE_PERCENTAGE as u64);
        }
        
        // Test edge cases
        let (fee, transfer) = calculate_trading_fee(0).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(transfer, 0);
        
        let (fee, transfer) = calculate_trading_fee(1).unwrap();
        assert_eq!(fee, 0); // 1 * 2 / 100 = 0
        assert_eq!(transfer, 1);
        
        let (fee, transfer) = calculate_trading_fee(49).unwrap();
        assert_eq!(fee, 0); // 49 * 2 / 100 = 0
        assert_eq!(transfer, 49);
        
        let (fee, transfer) = calculate_trading_fee(50).unwrap();
        assert_eq!(fee, 1); // 50 * 2 / 100 = 1
        assert_eq!(transfer, 49);
    }

    #[test]
    fn test_upgrade_cost_calculations() {
        // Test all valid upgrade levels
        for level in 1..=4 {
            let cost = get_upgrade_cost_for_level(level);
            assert!(cost.is_ok(), "Should have upgrade cost for level {}", level);
            
            let cost_value = cost.unwrap();
            let expected_cost = UPGRADE_COSTS[level as usize - 1];
            assert_eq!(cost_value, expected_cost, "Upgrade cost mismatch for level {}", level);
            
            // Verify costs are reasonable (between 1,000 and 30,000 WEED)
            assert!(cost_value >= 1_000 * 1_000_000, "Upgrade cost too low for level {}", level);
            assert!(cost_value <= 30_000 * 1_000_000, "Upgrade cost too high for level {}", level);
        }
        
        // Test invalid levels
        assert!(get_upgrade_cost_for_level(0).is_err());
        assert!(get_upgrade_cost_for_level(5).is_err());
        assert!(get_upgrade_cost_for_level(255).is_err());
        
        // Verify upgrade costs are ascending
        for level in 1..=3 {
            let current_cost = get_upgrade_cost_for_level(level).unwrap();
            let next_cost = get_upgrade_cost_for_level(level + 1).unwrap();
            assert!(current_cost < next_cost, "Upgrade costs should be ascending");
        }
    }

    #[test]
    fn test_halving_mechanism_detailed() {
        let current_rate = 1000u64;
        let halving_interval = 3600i64;
        let base_time = 1000000i64;
        
        // Test no halving (before halving time)
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            base_time,
            base_time + halving_interval,
            current_rate,
            halving_interval,
        );
        
        assert!(!should_halve);
        assert_eq!(new_rate, current_rate);
        assert_eq!(new_time, base_time + halving_interval);
        
        // Test halving (at exact halving time)
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            base_time + halving_interval,
            base_time + halving_interval,
            current_rate,
            halving_interval,
        );
        
        assert!(should_halve);
        assert_eq!(new_rate, current_rate / 2);
        assert_eq!(new_time, base_time + halving_interval + halving_interval);
        
        // Test halving (after halving time)
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            base_time + halving_interval + 1000,
            base_time + halving_interval,
            current_rate,
            halving_interval,
        );
        
        assert!(should_halve);
        assert_eq!(new_rate, current_rate / 2);
        assert_eq!(new_time, base_time + halving_interval + halving_interval);
        
        // Test multiple halvings
        let mut rate = 1024u64;
        let mut next_halving = base_time;
        
        for i in 0..10 {
            let (halved, new_rate, new_halving_time) = check_and_apply_halving(
                next_halving,
                next_halving,
                rate,
                halving_interval,
            );
            
            assert!(halved, "Should halve at iteration {}", i);
            assert_eq!(new_rate, rate / 2, "Rate should halve at iteration {}", i);
            assert_eq!(new_halving_time, next_halving + halving_interval, "Next halving time incorrect at iteration {}", i);
            
            rate = new_rate;
            next_halving = new_halving_time;
        }
        
        // After 10 halvings, rate should be 1024 / 2^10 = 1
        assert_eq!(rate, 1);
    }

    #[test]
    fn test_probability_based_calculations() {
        // Test expected value calculation for mystery packs
        let expected_value = calculate_mystery_pack_expected_value();
        
        // Manual calculation verification
        let mut manual_expected_value = 0.0;
        for i in 0..SEED_GROW_POWERS.len() {
            manual_expected_value += SEED_GROW_POWERS[i] as f32 * SEED_PROBABILITIES[i];
        }
        
        assert!((expected_value - manual_expected_value).abs() < 0.01, 
               "Expected value calculation mismatch: {} vs {}", expected_value, manual_expected_value);
        
        // Expected value should be reasonable (between 200-500 based on probabilities)
        assert!(expected_value > 200.0, "Expected value too low: {}", expected_value);
        assert!(expected_value < 500.0, "Expected value too high: {}", expected_value);
        
        // Test ROI calculations for all seed types
        for i in 0..SEED_GROW_POWERS.len() {
            let roi = calculate_seed_roi(i);
            
            // ROI should be finite and reasonable
            assert!(roi.is_finite(), "ROI should be finite for seed type {}", i);
            assert!(roi > -100.0, "ROI should be > -100% for seed type {}", i);
            assert!(roi < 1000.0, "ROI should be < 1000% for seed type {}", i);
        }
    }

    #[test]
    fn test_capacity_and_level_calculations() {
        // Test total capacity calculation
        let total_capacity = calculate_total_possible_capacity();
        let expected_total: u8 = FARM_CAPACITIES.iter().sum();
        assert_eq!(total_capacity, expected_total);
        
        // Test individual level capacities
        for level in 1..=5 {
            let capacity = FarmSpace::get_capacity_for_level(level);
            let expected_capacity = FARM_CAPACITIES[level as usize - 1];
            assert_eq!(capacity, expected_capacity, "Capacity mismatch for level {}", level);
        }
        
        // Test capacity progression is reasonable
        for i in 1..FARM_CAPACITIES.len() {
            assert!(FARM_CAPACITIES[i] > FARM_CAPACITIES[i-1], 
                   "Capacity should increase with level");
            assert!(FARM_CAPACITIES[i] - FARM_CAPACITIES[i-1] == 4, 
                   "Capacity should increase by 4 each level");
        }
    }

    #[test]
    fn test_overflow_protection_edge_cases() {
        // Test maximum safe values that shouldn't overflow
        let max_grow_power = u64::MAX / 1000;
        let max_global_power = u64::MAX / 1000;
        let max_rate = 1000u64;
        let max_time = 1000i64;
        
        // This should not panic or overflow
        let result = calculate_user_rewards_across_halving(
            max_grow_power,
            max_global_power,
            max_rate,
            0,
            max_time,
            max_time + 3600,
            3600,
        );
        
        assert!(result.is_ok(), "Should handle large values without overflow");
        
        // Test user share calculation with large values
        let result = calculate_user_share_of_global_rewards(
            max_grow_power,
            max_global_power,
            max_rate,
            max_time as u64,
        );
        
        assert!(result.is_ok(), "User share calculation should handle large values");
        
        // Test values that would overflow with naive implementation
        let result = calculate_user_share_of_global_rewards(
            u64::MAX / 100,
            1,
            1000,
            1000,
        );
        
        assert!(result.is_ok(), "Should handle overflow-prone values safely");
    }

    #[test]
    fn test_economic_invariants() {
        // Test that referral percentages always sum to reasonable amount
        let base_reward = 1000u64;
        let (level1, level2) = calculate_referral_rewards(base_reward).unwrap();
        
        // Total referral rewards should not exceed base reward
        assert!(level1 + level2 <= base_reward, "Referral rewards should not exceed base reward");
        
        // Referral percentages should be exactly 10% and 5%
        if base_reward >= 10 {
            assert_eq!(level1, base_reward / 10, "Level 1 should be exactly 10%");
        }
        if base_reward >= 20 {
            assert_eq!(level2, base_reward / 20, "Level 2 should be exactly 5%");
        }
        
        // Test trading fee invariant
        for amount in [100, 1000, 10000, 100000] {
            let (fee, transfer) = calculate_trading_fee(amount).unwrap();
            assert_eq!(fee + transfer, amount, "Fee + transfer should equal original amount");
            assert!(fee <= amount * 3 / 100, "Fee should not exceed 3% due to rounding");
        }
        
        // Test halving invariant (rate should always decrease)
        let mut rate = 1000u64;
        for _ in 0..20 {
            let old_rate = rate;
            let (_, new_rate, _) = check_and_apply_halving(0, 0, rate, 3600);
            rate = new_rate;
            assert!(rate <= old_rate, "Rate should never increase during halving");
            assert!(rate == old_rate / 2, "Rate should be exactly half");
            
            if rate == 0 {
                break;
            }
        }
    }

    #[test]
    fn test_time_based_calculations() {
        // Test reward calculation across different time periods
        let time_periods = [1, 60, 3600, 86400, 604800]; // 1s, 1m, 1h, 1d, 1w
        let user_power = 1000u64;
        let global_power = 10000u64;
        let base_rate = 100u64;
        
        for &period in &time_periods {
            let reward = calculate_user_rewards_across_halving(
                user_power,
                global_power,
                base_rate,
                0,
                period,
                period + 86400, // Halving is 1 day later
                86400,
            ).unwrap();
            
            // Reward should be proportional to time
            let expected_proportion = period as u64 * user_power * base_rate / global_power;
            assert_eq!(reward, expected_proportion, 
                      "Reward should be proportional to time for period {}", period);
        }
        
        // Test that longer periods yield higher rewards (without halving)
        let short_reward = calculate_user_rewards_across_halving(
            user_power, global_power, base_rate, 0, 1800, 7200, 3600
        ).unwrap();
        
        let long_reward = calculate_user_rewards_across_halving(
            user_power, global_power, base_rate, 0, 3600, 7200, 3600
        ).unwrap();
        
        assert!(long_reward > short_reward, 
               "Longer periods should yield higher rewards when no halving occurs");
    }
}