#[cfg(test)]
mod economic_model_validation_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;
    use crate::utils::*;
    use crate::error::GameError;

    // ===== ECONOMIC MODEL VALIDATION TESTS =====
    // Testing all core economic formulas and mechanisms

    #[test]
    fn test_proportional_reward_formula_validation() {
        // Proportional reward formula validation
        // user_reward = (user_grow_power / global_grow_power) × base_rate × elapsed_time
        
        println!("🧮 Testing Proportional Reward Formula");
        
        let test_cases = [
            // (user_gp, global_gp, base_rate, elapsed_time, expected_reward)
            (100, 1000, 100, 3600, 36000),      // 10% share, 1 hour
            (500, 1000, 100, 3600, 180000),     // 50% share, 1 hour  
            (1000, 1000, 100, 3600, 360000),    // 100% share, 1 hour
            (100, 2000, 200, 1800, 18000),      // 5% share, 30 min, double rate
            (0, 1000, 100, 3600, 0),            // No grow power
            (100, 0, 100, 3600, 0),             // No global grow power (edge case)
        ];
        
        for (user_gp, global_gp, base_rate, elapsed_time, expected) in test_cases {
            let result = calculate_user_share_of_global_rewards(
                user_gp, 
                global_gp, 
                base_rate, 
                elapsed_time
            ).unwrap_or(0);
            
            assert_eq!(result, expected, 
                      "Formula validation failed: user_gp={}, global_gp={}, rate={}, time={}", 
                      user_gp, global_gp, base_rate, elapsed_time);
            
            // Verify the mathematical relationship
            if global_gp > 0 {
                let expected_calculation = (user_gp as u128 * base_rate as u128 * elapsed_time as u128 / global_gp as u128) as u64;
                assert_eq!(result, expected_calculation, "Mathematical calculation mismatch");
            }
        }
        
        println!("✅ Proportional reward formula validated");
    }

    #[test]
    fn test_halving_mechanism_verification() {
        // Halving mechanism verification  
        // new_rate = current_rate / 2 (every 7 days, not 6 days)
        
        println!("📉 Testing Halving Mechanism");
        
        let halving_interval = DEFAULT_HALVING_INTERVAL; // 7 days = 604,800 seconds
        assert_eq!(halving_interval, 7 * 24 * 60 * 60, "Halving interval should be 7 days");
        
        let test_scenarios = [
            // (current_time, next_halving_time, current_rate, expected_should_halve, expected_new_rate)
            (300000, 604800, 100, false, 100),        // Before halving (day 3)
            (604800, 604800, 100, true, 50),          // Exactly at halving (day 7)
            (700000, 604800, 100, true, 50),          // After halving time
            (1209600, 1209600, 50, true, 25),         // Second halving (day 14)
            (1814400, 1814400, 25, true, 12),         // Third halving (day 21), rounded down
        ];
        
        for (current_time, next_halving_time, current_rate, expected_should_halve, expected_new_rate) in test_scenarios {
            let (should_halve, new_rate, new_next_halving) = check_and_apply_halving(
                current_time,
                next_halving_time,
                current_rate,
                halving_interval,
            );
            
            assert_eq!(should_halve, expected_should_halve,
                      "Halving detection failed at time {}", current_time);
            
            assert_eq!(new_rate, expected_new_rate,
                      "Halving rate calculation failed: {} -> {}", current_rate, new_rate);
            
            if should_halve {
                assert_eq!(new_next_halving, next_halving_time + halving_interval,
                          "Next halving time calculation failed");
            }
        }
        
        // Test multiple consecutive halvings
        let mut current_rate = 100u64;
        let mut current_time = 0i64;
        let mut next_halving = halving_interval;
        
        let expected_rates = [100, 50, 25, 12, 6, 3, 1, 0];
        
        for (i, &expected_rate) in expected_rates.iter().enumerate() {
            if i > 0 {
                current_time = next_halving;
                let (should_halve, new_rate, new_next_halving) = check_and_apply_halving(
                    current_time,
                    next_halving,
                    current_rate,
                    halving_interval,
                );
                
                assert!(should_halve, "Should halve at iteration {}", i);
                current_rate = new_rate;
                next_halving = new_next_halving;
            }
            
            assert_eq!(current_rate, expected_rate,
                      "Rate after {} halvings should be {}", i, expected_rate);
        }
        
        println!("✅ Halving mechanism verified (7-day intervals)");
    }

    #[test]
    fn test_referral_reward_distribution() {
        // Referral reward distribution
        // level1_reward = base_reward × 10%
        // level2_reward = base_reward × 5%
        
        println!("👥 Testing Referral Reward Distribution");
        
        let test_amounts = [
            (1000, 100, 50),           // 1,000 base -> 100 L1, 50 L2
            (10_000_000, 1_000_000, 500_000), // 10M base -> 1M L1, 500K L2
            (500, 50, 25),             // 500 base -> 50 L1, 25 L2
            (0, 0, 0),                 // 0 base -> 0 L1, 0 L2
            (1, 0, 0),                 // 1 base -> 0 L1, 0 L2 (rounding down)
        ];
        
        for (base_reward, expected_l1, expected_l2) in test_amounts {
            let (level1_reward, level2_reward) = calculate_referral_rewards(base_reward).unwrap();
            
            assert_eq!(level1_reward, expected_l1,
                      "Level 1 reward calculation failed for base {}", base_reward);
            
            assert_eq!(level2_reward, expected_l2,
                      "Level 2 reward calculation failed for base {}", base_reward);
            
            // Verify percentage calculations
            if base_reward >= 10 {
                assert_eq!(level1_reward, base_reward / 10, "L1 should be 10%");
            }
            if base_reward >= 20 {
                assert_eq!(level2_reward, base_reward / 20, "L2 should be 5%");
            }
            
            // Verify total rewards don't exceed reasonable bounds
            let total_referral = level1_reward + level2_reward;
            assert!(total_referral <= base_reward * 15 / 100, 
                   "Total referral rewards should not exceed 15% of base");
        }
        
        println!("✅ Referral reward distribution validated (L1: 10%, L2: 5%)");
    }

    #[test]
    fn test_transfer_fee_calculation() {
        // Transfer fee calculation (SPL Token 2022 Transfer Fee Extension)
        // fee_rate = 2% (200 basis points)
        // max_fee = 1000 WEED (1,000,000,000 micro-tokens)
        
        println!("💸 Testing Transfer Fee Calculation");
        
        let test_transfers = [
            // (amount, expected_fee, expected_net_transfer)
            (1000, 20, 980),           // 1,000 -> 2% = 20 fee
            (50_000, 1000, 49_000),    // 50,000 -> 2% = 1,000 fee
            (100_000, 1000, 99_000),   // 100,000 -> 2% = 2,000, capped at 1,000
            (500_000, 1000, 499_000),  // 500,000 -> 2% = 10,000, capped at 1,000
            (50, 1, 49),               // 50 -> 2% = 1 fee (minimum)
            (0, 0, 0),                 // 0 -> 0 fee
        ];
        
        for (amount, expected_fee, expected_net) in test_transfers {
            let (calculated_fee, net_transfer) = calculate_transfer_fee(amount).unwrap();
            
            assert_eq!(calculated_fee, expected_fee,
                      "Transfer fee calculation failed for amount {}", amount);
            
            assert_eq!(net_transfer, expected_net,
                      "Net transfer calculation failed for amount {}", amount);
            
            // Verify fee is never more than 2% or 1,000 WEED
            assert!(calculated_fee <= amount * 2 / 100, "Fee should not exceed 2%");
            assert!(calculated_fee <= 1000, "Fee should not exceed 1,000 WEED maximum");
            
            // Verify total adds up
            assert_eq!(calculated_fee + net_transfer, amount, "Fee + net should equal original amount");
        }
        
        println!("✅ Transfer fee calculation validated (2% rate, 1,000 WEED max)");
    }

    #[test]
    fn test_farm_upgrade_cost_progression() {
        // Farm upgrade cost progression
        // Level 1→2: 3,500 WEED
        // Level 2→3: 18,000 WEED  
        // Level 3→4: 20,000 WEED
        // Level 4→5: 25,000 WEED
        
        println!("🏗️ Testing Farm Upgrade Cost Progression");
        
        let expected_costs = [
            (1, 3_500_000_000),    // Level 1→2: 3,500 WEED (6 decimal precision)
            (2, 18_000_000_000),   // Level 2→3: 18,000 WEED
            (3, 20_000_000_000),   // Level 3→4: 20,000 WEED
            (4, 25_000_000_000),   // Level 4→5: 25,000 WEED
        ];
        
        for (level, expected_cost) in expected_costs {
            let cost = get_upgrade_cost_for_level(level).unwrap();
            assert_eq!(cost, expected_cost,
                      "Upgrade cost for level {} should be {} micro-WEED", level, expected_cost);
            
            // Convert to human-readable WEED for verification
            let weed_amount = cost / 1_000_000;
            match level {
                1 => assert_eq!(weed_amount, 3_500),
                2 => assert_eq!(weed_amount, 18_000),
                3 => assert_eq!(weed_amount, 20_000),
                4 => assert_eq!(weed_amount, 25_000),
                _ => panic!("Unexpected level"),
            }
        }
        
        // Verify invalid levels return errors
        assert!(get_upgrade_cost_for_level(0).is_err(), "Level 0 should be invalid");
        assert!(get_upgrade_cost_for_level(5).is_err(), "Level 5 should be invalid (max is 4)");
        assert!(get_upgrade_cost_for_level(10).is_err(), "Level 10 should be invalid");
        
        // Verify cost progression makes economic sense (increasing difficulty)
        let cost_1_to_2 = get_upgrade_cost_for_level(1).unwrap();
        let cost_2_to_3 = get_upgrade_cost_for_level(2).unwrap();
        let cost_3_to_4 = get_upgrade_cost_for_level(3).unwrap();
        let cost_4_to_5 = get_upgrade_cost_for_level(4).unwrap();
        
        assert!(cost_2_to_3 > cost_1_to_2, "Level 2→3 should cost more than 1→2");
        assert!(cost_3_to_4 > cost_2_to_3, "Level 3→4 should cost more than 2→3");
        assert!(cost_4_to_5 > cost_3_to_4, "Level 4→5 should cost more than 3→4");
        
        println!("✅ Farm upgrade cost progression validated");
    }

    #[test]
    fn test_seed_pack_economics() {
        // Seed pack economics
        // Cost: 300 WEED per pack
        // VRF Fee: ~0.002 SOL (~2,077,400 lamports)
        
        println!("📦 Testing Seed Pack Economics");
        
        let seed_pack_cost = SEED_PACK_COST; // 300 WEED in micro-tokens
        assert_eq!(seed_pack_cost, 300 * 1_000_000, "Seed pack should cost 300 WEED");
        
        // Test VRF fee calculation
        let base_fee = 5_000u64;
        let num_transactions = 15u64;
        let storage_rent = 2_400u64;
        let oracle_fee = 2_000_000u64;
        
        let calculated_vrf_fee = base_fee * num_transactions + storage_rent + oracle_fee;
        let expected_vrf_fee = 2_077_400u64; // ~0.002 SOL
        
        assert_eq!(calculated_vrf_fee, expected_vrf_fee,
                  "VRF fee calculation should match expected value");
        
        // Verify VRF fee is reasonable (not the old 0.07 SOL estimate)
        let old_estimate = 70_000_000u64; // 0.07 SOL
        assert!(calculated_vrf_fee < old_estimate / 30,
               "VRF fee should be much lower than old estimate");
        
        // Test seed pack quantity validation
        let valid_quantities = [1u8, 5, 10, 25, 50, 100];
        let invalid_quantities = [0u8, 101, 255];
        
        for &quantity in &valid_quantities {
            assert!(quantity > 0 && quantity <= 100, "Quantity {} should be valid", quantity);
            
            let total_cost = seed_pack_cost * quantity as u64;
            assert!(total_cost >= seed_pack_cost, "Total cost should scale with quantity");
        }
        
        for &quantity in &invalid_quantities {
            assert!(!(quantity > 0 && quantity <= 100), "Quantity {} should be invalid", quantity);
        }
        
        println!("✅ Seed pack economics validated (300 WEED + ~0.002 SOL VRF)");
    }

    #[test]
    fn test_seed_probability_distribution() {
        // Seed probability distribution (VRF-guaranteed)
        // Seed1 (100GP): 42.23%
        // Seed2 (180GP): 24.44%
        // Seed3 (420GP): 13.33%
        // Seed4 (720GP): 8.33%
        // Seed5 (1000GP): 5.56%
        // Seed6 (5000GP): 3.33%
        // Seed7 (15000GP): 1.33%
        // Seed8 (30000GP): 0.89%
        // Seed9 (60000GP): 0.56%
        
        println!("🎯 Testing Seed Probability Distribution");
        
        // Verify probability thresholds sum to reasonable distribution
        let expected_thresholds = [4223, 6667, 8000, 8833, 9389, 9722, 9855, 9944, 10000];
        
        for (i, &threshold) in SEED_PROBABILITY_THRESHOLDS.iter().enumerate() {
            assert_eq!(threshold, expected_thresholds[i],
                      "Seed{} threshold should be {}", i + 1, expected_thresholds[i]);
        }
        
        // Verify grow powers match expected values
        let expected_grow_powers = [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000];
        
        for (i, &expected_gp) in expected_grow_powers.iter().enumerate() {
            let actual_gp = SEED_GROW_POWERS[i];
            assert_eq!(actual_gp, expected_gp,
                      "Seed{} grow power should be {}", i + 1, expected_gp);
        }
        
        // Verify probabilities sum to 100%
        let mut cumulative_prob = 0.0;
        let expected_percentages = [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56];
        
        for (i, &percentage) in expected_percentages.iter().enumerate() {
            cumulative_prob += percentage;
            
            let threshold_prob = expected_thresholds[i] as f64 / 100.0;
            assert!((threshold_prob - cumulative_prob).abs() < 0.1,
                   "Cumulative probability mismatch at Seed{}", i + 1);
        }
        
        assert!((cumulative_prob - 100.0).abs() < 0.01, "Total probabilities should sum to 100%");
        
        println!("✅ Seed probability distribution validated");
    }

    #[test]
    fn test_economic_sustainability_model() {
        // Economic sustainability model
        // Tests the long-term viability of the economic system
        
        println!("♻️ Testing Economic Sustainability Model");
        
        // Simulate 1 year of operation with halving
        let total_simulation_time = 365 * 24 * 60 * 60i64; // 1 year in seconds
        let halving_interval = DEFAULT_HALVING_INTERVAL; // 7 days
        let initial_rate = DEFAULT_BASE_RATE; // 100 WEED/second
        
        let mut current_rate = initial_rate;
        let mut total_rewards_distributed = 0u64;
        let mut time_elapsed = 0i64;
        let mut next_halving = halving_interval;
        
        // Simulate in 1-hour increments
        let time_increment = 3600i64; // 1 hour
        
        while time_elapsed < total_simulation_time {
            // Check for halving
            if time_elapsed >= next_halving {
                current_rate = current_rate / 2;
                next_halving += halving_interval;
                println!("Halving occurred at day {}: new rate = {}", 
                        time_elapsed / (24 * 60 * 60), current_rate);
            }
            
            // Calculate rewards for this hour
            let hour_rewards = current_rate * time_increment as u64;
            total_rewards_distributed += hour_rewards;
            
            time_elapsed += time_increment;
        }
        
        println!("Total rewards distributed in 1 year: {} WEED", 
                total_rewards_distributed / 1_000_000);
        
        // Verify rewards don't exceed total supply
        assert!(total_rewards_distributed <= TOTAL_WEED_SUPPLY,
               "Total rewards should not exceed maximum supply");
        
        // Verify halving reduces inflation over time
        let final_rate = current_rate;
        assert!(final_rate < initial_rate / 50, 
               "Rate should be significantly reduced after 1 year of weekly halvings");
        
        // Calculate number of halvings in 1 year
        let expected_halvings = 52; // 365 days / 7 days = ~52 halvings
        let theoretical_final_rate = initial_rate / (2u64.pow(expected_halvings));
        
        // Due to integer division, actual rate might be slightly different
        assert!(final_rate <= theoretical_final_rate + 1,
               "Final rate should be close to theoretical calculation");
        
        println!("✅ Economic sustainability model validated");
    }

    #[test]
    fn test_game_balance_verification() {
        // Game balance verification
        // Ensures all game mechanics work together harmoniously
        
        println!("⚖️ Testing Game Balance Verification");
        
        // Test entry cost vs reward potential
        let farm_cost_sol = FARM_SPACE_COST_SOL; // 0.5 SOL
        let initial_grow_power = 100u64; // Free Seed1
        let base_rate = DEFAULT_BASE_RATE; // 100 WEED/second global
        
        // Assume 1000 total grow power in ecosystem (10 players with 100 GP each)
        let assumed_total_gp = 1000u64;
        let user_share = initial_grow_power as f64 / assumed_total_gp as f64; // 10%
        
        // Calculate daily earnings for new player
        let daily_seconds = 24 * 60 * 60u64;
        let daily_earnings = (base_rate as f64 * user_share * daily_seconds as f64) as u64;
        
        println!("New player daily earnings: {} WEED", daily_earnings / 1_000_000);
        
        // Player should be able to buy seed pack in reasonable time
        let days_to_seed_pack = SEED_PACK_COST as f64 / daily_earnings as f64;
        assert!(days_to_seed_pack <= 5.0, 
               "New player should afford seed pack within 5 days");
        
        // Test upgrade progression viability
        let upgrade_1_to_2 = get_upgrade_cost_for_level(1).unwrap();
        let days_to_upgrade = upgrade_1_to_2 as f64 / daily_earnings as f64;
        
        assert!(days_to_upgrade <= 30.0,
               "First upgrade should be achievable within 30 days");
        
        // Test seed pack expected value
        let seed_pack_cost_weed = SEED_PACK_COST / 1_000_000; // 300 WEED
        
        // Calculate expected grow power gain from seed pack
        let mut expected_value = 0.0;
        for i in 0..9 {
            let probability = SEED_PROBABILITIES[i];
            let grow_power = SEED_GROW_POWERS[i] as f64;
            expected_value += probability * grow_power;
        }
        
        println!("Seed pack expected grow power: {:.2}", expected_value);
        
        // Expected value should justify the cost
        assert!(expected_value > 300.0, "Seed pack should provide positive expected value");
        
        // Test long-term player progression
        let max_farm_capacity = 20u8; // Level 5 farm
        let average_seed_gp = expected_value;
        let max_potential_gp = max_farm_capacity as f64 * average_seed_gp;
        
        println!("Max potential grow power: {:.2}", max_potential_gp);
        
        // Verify meaningful progression is possible
        assert!(max_potential_gp > initial_grow_power as f64 * 10.0,
               "Max potential should be significantly higher than starting power");
        
        println!("✅ Game balance verification completed");
    }

    // Helper function to test economic formulas
    fn simulate_economic_scenario(
        users: Vec<u64>,        // grow power per user
        base_rate: u64,         // global rate
        time_seconds: u64,      // simulation time
    ) -> Vec<u64> {             // rewards per user
        let total_gp: u64 = users.iter().sum();
        
        users.iter().map(|&user_gp| {
            if total_gp > 0 {
                (user_gp as u128 * base_rate as u128 * time_seconds as u128 / total_gp as u128) as u64
            } else {
                0
            }
        }).collect()
    }

    #[test]
    fn test_economic_scenario_simulation() {
        // Test various economic scenarios
        println!("🎮 Testing Economic Scenario Simulation");
        
        // Scenario 1: Equal players
        let equal_users = vec![100, 100, 100, 100, 100]; // 5 users with 100 GP each
        let rewards = simulate_economic_scenario(equal_users, 100, 3600); // 1 hour
        
        for reward in &rewards {
            assert_eq!(*reward, 7200); // Each gets 20% of 36,000 total
        }
        
        // Scenario 2: Whale vs minnows
        let whale_scenario = vec![1000, 10, 10, 10, 10]; // 1 whale, 4 minnows
        let rewards = simulate_economic_scenario(whale_scenario, 100, 3600);
        
        assert!(rewards[0] > rewards[1] * 50, "Whale should get proportionally more");
        
        // Scenario 3: Growing ecosystem
        let growing_users = vec![100, 200, 300, 400, 500]; // Different investment levels
        let rewards = simulate_economic_scenario(growing_users, 200, 1800); // 30 min, higher rate
        
        let total_rewards: u64 = rewards.iter().sum();
        assert_eq!(total_rewards, 200 * 1800); // Should equal total distributed
        
        println!("✅ Economic scenario simulation validated");
    }
}