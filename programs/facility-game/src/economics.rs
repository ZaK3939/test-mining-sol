/// Economics module for game calculations
/// Centralized location for all economic formulas and calculations

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::error::GameError;

// ===== REWARD CALCULATIONS =====

/// Calculate base reward for a user
/// Formula: (elapsed_time × grow_power × base_rate) / 1000
pub fn calculate_base_reward(
    elapsed_time: u64,
    grow_power: u64,
    base_rate: u64
) -> Result<u64> {
    let time_power = elapsed_time.checked_mul(grow_power)
        .ok_or(GameError::CalculationOverflow)?;
    
    let total_reward = time_power.checked_mul(base_rate)
        .ok_or(GameError::CalculationOverflow)?;
    
    Ok(total_reward / 1000)
}

/// Calculate user's share of global rewards
/// Formula: (user_grow_power / total_grow_power) × base_rate × elapsed_time
pub fn calculate_user_share_reward(
    user_grow_power: u64,
    total_grow_power: u64,
    base_rate: u64,
    elapsed_time: u64
) -> Result<u64> {
    if total_grow_power == 0 {
        return Ok(0);
    }
    
    // Calculate user's percentage share (multiply by 1000 for precision)
    let user_share = user_grow_power.checked_mul(1000)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(total_grow_power)
        .unwrap_or(0);
    
    // Calculate time-based reward
    let time_reward = base_rate.checked_mul(elapsed_time)
        .ok_or(GameError::CalculationOverflow)?;
    
    // Apply user's share
    let final_reward = time_reward.checked_mul(user_share)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(1000)
        .unwrap_or(0);
    
    Ok(final_reward)
}

/// Calculate rewards across halving periods
/// Handles cases where reward claim spans multiple halving events
pub fn calculate_rewards_across_halving(
    user_grow_power: u64,
    total_grow_power: u64,
    base_rate: u64,
    last_harvest_time: i64,
    current_time: i64,
    next_halving_time: i64,
    halving_interval: i64,
) -> Result<u64> {
    if current_time <= last_harvest_time {
        return Ok(0);
    }
    
    let mut total_reward = 0u64;
    let mut start_time = last_harvest_time;
    let mut current_rate = base_rate;
    let mut current_halving_time = next_halving_time;
    
    // If we're past the first halving point, adjust starting parameters
    if start_time >= next_halving_time {
        let halvings_passed = ((start_time - next_halving_time) / halving_interval) + 1;
        current_rate = base_rate >> halvings_passed.min(63); // Prevent overflow in shift
        current_halving_time = next_halving_time + (halvings_passed * halving_interval);
    }
    
    while start_time < current_time {
        let end_time = if current_time <= current_halving_time {
            current_time
        } else {
            current_halving_time
        };
        
        let elapsed = (end_time - start_time) as u64;
        let period_reward = calculate_user_share_reward(
            user_grow_power,
            total_grow_power,
            current_rate,
            elapsed
        )?;
        
        total_reward = total_reward.checked_add(period_reward)
            .ok_or(GameError::CalculationOverflow)?;
        
        // Move to next period
        start_time = end_time;
        if start_time >= current_halving_time {
            current_rate = current_rate / 2; // Halve the rate
            current_halving_time += halving_interval;
        }
    }
    
    Ok(total_reward)
}

// ===== REFERRAL CALCULATIONS =====

/// Calculate referral rewards for Level 1 and Level 2
pub fn calculate_referral_rewards(base_reward: u64) -> Result<(u64, u64)> {
    let level1_reward = base_reward.checked_mul(LEVEL1_REFERRAL_PERCENTAGE as u64)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(100)
        .unwrap_or(0);
    
    let level2_reward = base_reward.checked_mul(LEVEL2_REFERRAL_PERCENTAGE as u64)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(100)
        .unwrap_or(0);
    
    Ok((level1_reward, level2_reward))
}

/// Calculate referral reward for a specific level
pub fn calculate_referral_reward_for_level(base_reward: u64, level: u8) -> Result<u64> {
    let percentage = match level {
        1 => LEVEL1_REFERRAL_PERCENTAGE,
        2 => LEVEL2_REFERRAL_PERCENTAGE,
        _ => return Ok(0),
    };
    
    let reward = base_reward.checked_mul(percentage as u64)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(100)
        .unwrap_or(0);
    
    Ok(reward)
}

// ===== UPGRADE CALCULATIONS =====

/// Get upgrade cost for a specific farm level
pub fn get_upgrade_cost(current_level: u8) -> Result<u64> {
    if current_level == 0 || current_level > 4 {
        return Err(GameError::MaxLevelReached.into());
    }
    
    Ok(LEGACY_UPGRADE_COSTS[(current_level - 1) as usize])
}

/// Calculate cost efficiency for upgrades (cost per capacity slot)
pub fn calculate_upgrade_efficiency(current_level: u8) -> Result<u64> {
    let cost = get_upgrade_cost(current_level)?;
    let current_capacity = FARM_CAPACITIES[(current_level - 1) as usize];
    let new_capacity = FARM_CAPACITIES[current_level as usize];
    let capacity_increase = new_capacity - current_capacity;
    
    Ok(cost / capacity_increase as u64)
}

// ===== TRADING CALCULATIONS =====

/// Calculate trading fee and transfer amount
pub fn calculate_trading_fee(amount: u64) -> Result<(u64, u64)> {
    let fee = amount.checked_mul(TRADING_FEE_PERCENTAGE as u64)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(100)
        .unwrap_or(0);
    
    let transfer_amount = amount.checked_sub(fee)
        .ok_or(GameError::CalculationOverflow)?;
    
    Ok((fee, transfer_amount))
}

// ===== HALVING MECHANISM =====

/// Check if halving should occur and return new parameters
pub fn check_and_apply_halving(
    current_time: i64,
    next_halving_time: i64,
    current_rate: u64,
    halving_interval: i64,
) -> (bool, u64, i64) {
    if current_time >= next_halving_time {
        let new_rate = current_rate / 2;
        let new_halving_time = next_halving_time + halving_interval;
        (true, new_rate, new_halving_time)
    } else {
        (false, current_rate, next_halving_time)
    }
}

/// Calculate next halving time
pub fn calculate_next_halving_time(current_time: i64, halving_interval: i64) -> i64 {
    current_time + halving_interval
}

// ===== SEED ECONOMICS =====

/// Calculate expected value of a seed pack
pub fn calculate_seed_pack_expected_value() -> f32 {
    let mut expected_value = 0.0f32;
    
    for (i, &grow_power) in SEED_GROW_POWERS.iter().enumerate() {
        let probability = SEED_PROBABILITIES[i] / 100.0; // Convert percentage to decimal
        expected_value += (grow_power as f32) * probability;
    }
    
    expected_value
}

/// Calculate ROI for a specific seed type
pub fn calculate_seed_roi(seed_index: usize) -> f32 {
    if seed_index >= SEED_GROW_POWERS.len() {
        return -100.0; // Invalid seed
    }
    
    let grow_power = SEED_GROW_POWERS[seed_index] as f32;
    let expected_pack_cost = SEED_PACK_COST as f32; // 300 WEED per pack
    let probability = SEED_PROBABILITIES[seed_index];
    
    // Expected cost to get this seed = pack_cost / probability
    let expected_cost_to_obtain = expected_pack_cost / probability;
    
    // ROI = (value - cost) / cost * 100
    ((grow_power - expected_cost_to_obtain) / expected_cost_to_obtain) * 100.0
}

// ===== CAPACITY CALCULATIONS =====

/// Calculate total capacity for all farm levels
pub fn calculate_total_possible_capacity() -> u8 {
    FARM_CAPACITIES.iter().sum()
}

/// Calculate capacity utilization percentage
pub fn calculate_capacity_utilization(current_seeds: u8, max_capacity: u8) -> f32 {
    if max_capacity == 0 {
        return 0.0;
    }
    (current_seeds as f32 / max_capacity as f32) * 100.0
}

// ===== GROWTH RATE CALCULATIONS =====

/// Calculate compound growth rate for rewards
pub fn calculate_compound_growth_rate(
    initial_amount: u64,
    growth_rate_per_second: u64,
    time_period: u64
) -> Result<u64> {
    // Simple compound interest: A = P(1 + r)^t
    // For small rates, we can approximate: A ≈ P(1 + rt)
    let growth = initial_amount.checked_mul(growth_rate_per_second)
        .ok_or(GameError::CalculationOverflow)?
        .checked_mul(time_period)
        .ok_or(GameError::CalculationOverflow)?
        .checked_div(1000) // Normalize the rate
        .unwrap_or(0);
    
    initial_amount.checked_add(growth)
        .ok_or(GameError::CalculationOverflow)
        .map_err(|e| e.into())
}

/// Calculate break-even time for farm space upgrade
pub fn calculate_upgrade_breakeven_time(
    upgrade_cost: u64,
    additional_capacity: u8,
    grow_power_per_seed: u64,
    reward_rate_per_grow_power: u64
) -> Result<u64> {
    if additional_capacity == 0 || grow_power_per_seed == 0 || reward_rate_per_grow_power == 0 {
        return Err(GameError::InvalidConfig.into());
    }
    
    let additional_grow_power = additional_capacity as u64 * grow_power_per_seed;
    let additional_rewards_per_second = additional_grow_power * reward_rate_per_grow_power / 1000;
    
    if additional_rewards_per_second == 0 {
        return Err(GameError::InvalidConfig.into());
    }
    
    Ok(upgrade_cost / additional_rewards_per_second)
}

// ===== PROBABILITY CALCULATIONS =====

/// Calculate cumulative probability for seed types
pub fn calculate_cumulative_probabilities() -> [f32; 9] {
    let mut cumulative = [0.0f32; 9];
    let mut sum = 0.0f32;
    
    for (i, &prob) in SEED_PROBABILITIES.iter().enumerate() {
        sum += prob;
        cumulative[i] = sum;
    }
    
    cumulative
}

/// Calculate rarity score for a seed type (lower is rarer)
pub fn calculate_rarity_score(seed_index: usize) -> f32 {
    if seed_index >= SEED_PROBABILITIES.len() {
        return 0.0;
    }
    
    100.0 - SEED_PROBABILITIES[seed_index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_reward_calculation() {
        // Basic calculation
        let reward = calculate_base_reward(3600, 100, 10).unwrap(); // 1 hour, 100 GP, 10 rate
        assert_eq!(reward, 3600); // 3600 * 100 * 10 / 1000

        // Zero values
        assert_eq!(calculate_base_reward(0, 100, 10).unwrap(), 0);
        assert_eq!(calculate_base_reward(3600, 0, 10).unwrap(), 0);
        assert_eq!(calculate_base_reward(3600, 100, 0).unwrap(), 0);
    }

    #[test]
    fn test_user_share_calculation() {
        // Standard case: user has 10% of total grow power
        let reward = calculate_user_share_reward(100, 1000, 10, 3600).unwrap();
        assert_eq!(reward, 3600); // (100/1000) * 10 * 3600

        // Edge case: user has all grow power
        let reward = calculate_user_share_reward(1000, 1000, 10, 3600).unwrap();
        assert_eq!(reward, 36000); // 100% of rewards

        // Edge case: zero total grow power
        let reward = calculate_user_share_reward(100, 0, 10, 3600).unwrap();
        assert_eq!(reward, 0);
    }

    #[test]
    fn test_referral_calculations() {
        let (level1, level2) = calculate_referral_rewards(1000).unwrap();
        assert_eq!(level1, 100); // 10% of 1000
        assert_eq!(level2, 50);  // 5% of 1000

        // Test individual level calculation
        assert_eq!(calculate_referral_reward_for_level(1000, 1).unwrap(), 100);
        assert_eq!(calculate_referral_reward_for_level(1000, 2).unwrap(), 50);
        assert_eq!(calculate_referral_reward_for_level(1000, 3).unwrap(), 0); // Invalid level
    }

    #[test]
    fn test_trading_fee_calculation() {
        let (fee, transfer_amount) = calculate_trading_fee(1000).unwrap();
        assert_eq!(fee, 20); // 2% of 1000
        assert_eq!(transfer_amount, 980); // 1000 - 20

        // Edge case: zero amount
        let (fee, transfer_amount) = calculate_trading_fee(0).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(transfer_amount, 0);
    }

    #[test]
    fn test_halving_mechanism() {
        let current_time = 1000;
        let next_halving_time = 1000;
        let current_rate = 100;
        let halving_interval = 86400;

        // At halving time
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            current_time, next_halving_time, current_rate, halving_interval
        );
        assert!(should_halve);
        assert_eq!(new_rate, 50); // Halved
        assert_eq!(new_time, 87400); // Next halving

        // Before halving time
        let (should_halve, new_rate, new_time) = check_and_apply_halving(
            999, next_halving_time, current_rate, halving_interval
        );
        assert!(!should_halve);
        assert_eq!(new_rate, current_rate); // Unchanged
        assert_eq!(new_time, next_halving_time); // Unchanged
    }

    #[test]
    fn test_upgrade_calculations() {
        // Test upgrade costs
        assert_eq!(get_upgrade_cost(1).unwrap(), LEGACY_UPGRADE_COSTS[0]);
        assert_eq!(get_upgrade_cost(4).unwrap(), LEGACY_UPGRADE_COSTS[3]);
        assert!(get_upgrade_cost(5).is_err()); // Max level

        // Test upgrade efficiency
        let efficiency = calculate_upgrade_efficiency(1).unwrap();
        let expected = LEGACY_UPGRADE_COSTS[0] / (FARM_CAPACITIES[1] - FARM_CAPACITIES[0]) as u64;
        assert_eq!(efficiency, expected);
    }

    #[test]
    fn test_seed_economics() {
        // Test expected value calculation
        let expected_value = calculate_seed_pack_expected_value();
        assert!(expected_value > 0.0);
        
        // Should be weighted average of all seed grow powers
        let manual_calculation: f32 = SEED_GROW_POWERS.iter()
            .enumerate()
            .map(|(i, &gp)| (gp as f32) * (SEED_PROBABILITIES[i] / 100.0))
            .sum();
        
        assert!((expected_value - manual_calculation).abs() < 0.01);

        // Test ROI calculation for each seed type
        for i in 0..SEED_GROW_POWERS.len() {
            let roi = calculate_seed_roi(i);
            println!("Seed{} ROI: {:.2}%", i + 1, roi);
            
            // Most seeds should have negative ROI due to probability vs cost
            // Only the highest rarity seeds might have positive ROI
            if i <= 4 {
                assert!(roi < 0.0, "Seed{} should have negative ROI, got {:.2}%", i + 1, roi);
            }
            // Higher seeds (Seed6-9) might have positive or negative ROI depending on probability
            assert!(roi > -100.0, "ROI should be reasonable, got {:.2}%", roi);
        }
    }

    #[test]
    fn test_capacity_calculations() {
        let total_capacity = calculate_total_possible_capacity();
        assert_eq!(total_capacity, FARM_CAPACITIES.iter().sum::<u8>());

        // Test utilization calculation
        assert_eq!(calculate_capacity_utilization(5, 10), 50.0);
        assert_eq!(calculate_capacity_utilization(10, 10), 100.0);
        assert_eq!(calculate_capacity_utilization(0, 10), 0.0);
        assert_eq!(calculate_capacity_utilization(5, 0), 0.0); // Edge case
    }

    #[test]
    fn test_probability_calculations() {
        let cumulative = calculate_cumulative_probabilities();
        
        // Last element should be close to 100%
        assert!((cumulative[8] - 100.0).abs() < 0.01);
        
        // Should be in ascending order
        for i in 1..cumulative.len() {
            assert!(cumulative[i] >= cumulative[i-1]);
        }

        // Test rarity scores
        let rarity1 = calculate_rarity_score(0); // Seed1 (common)
        let rarity9 = calculate_rarity_score(8); // Seed9 (rare)
        assert!(rarity9 > rarity1); // Rarer seeds have higher rarity scores
    }

    #[test]
    fn test_overflow_protection() {
        // Test with very large values that could cause overflow
        let large_value = u64::MAX / 1000;
        
        // These should either succeed or return overflow error, not panic
        let result = calculate_base_reward(large_value, 100, 100);
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("overflow"));
        
        let result = calculate_user_share_reward(large_value, large_value, 100, 100);
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("overflow"));
    }
}