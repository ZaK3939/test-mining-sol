/// Time-related validation functions
/// Handles timing constraints, intervals, and timestamp validation

use anchor_lang::prelude::*;
use crate::error::GameError;
use crate::constants::*;
use crate::state::*;

// ===== TIME CONSTRAINTS =====

/// Validate enough time has passed for reward claim
pub fn validate_claim_interval(last_claim_time: i64, current_time: i64) -> Result<()> {
    require!(
        current_time >= last_claim_time + MIN_CLAIM_INTERVAL,
        GameError::NoRewardToClaim
    );
    Ok(())
}

/// Validate timestamp is not in the future
pub fn validate_timestamp_not_future(timestamp: i64, current_time: i64) -> Result<()> {
    require!(
        timestamp <= current_time,
        GameError::InvalidConfig
    );
    Ok(())
}

/// Validate farm space upgrade is complete
pub fn validate_upgrade_complete(farm_space: &FarmSpace, current_time: i64) -> Result<()> {
    // Check if upgrade is in progress
    require!(
        farm_space.upgrade_start_time > 0,
        GameError::NoUpgradeInProgress
    );
    
    // Check if cooldown has passed
    require!(
        current_time >= farm_space.upgrade_start_time + UPGRADE_COOLDOWN,
        GameError::UpgradeStillInProgress
    );
    
    Ok(())
}

/// Validate timing for halving mechanism
pub fn validate_halving_timing(
    config_start_time: i64,
    current_time: i64,
    halving_interval: i64
) -> Result<u32> {
    require!(
        current_time >= config_start_time,
        GameError::InvalidConfig
    );
    
    let elapsed_time = current_time - config_start_time;
    let halving_count = (elapsed_time / halving_interval) as u32;
    
    // Prevent excessive halving that would make rewards negligible
    require!(
        halving_count <= 64, // After 64 halvings, rewards become essentially zero
        GameError::CalculationOverflow
    );
    
    Ok(halving_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_interval_validation() {
        let base_time = 1000000i64;
        let valid_claim_time = base_time + MIN_CLAIM_INTERVAL;
        let invalid_claim_time = base_time + MIN_CLAIM_INTERVAL - 1;
        
        // Valid claim interval
        assert!(validate_claim_interval(base_time, valid_claim_time).is_ok());
        
        // Invalid claim interval (too soon)
        assert!(validate_claim_interval(base_time, invalid_claim_time).is_err());
        
        // Same time should fail
        assert!(validate_claim_interval(base_time, base_time).is_err());
    }

    #[test]
    fn test_timestamp_future_validation() {
        let current_time = 1000000i64;
        let past_time = current_time - 1000;
        let future_time = current_time + 1000;
        
        // Valid timestamps (past and present)
        assert!(validate_timestamp_not_future(past_time, current_time).is_ok());
        assert!(validate_timestamp_not_future(current_time, current_time).is_ok());
        
        // Invalid timestamp (future)
        assert!(validate_timestamp_not_future(future_time, current_time).is_err());
    }

    #[test]
    fn test_upgrade_completion_validation() {
        let current_time = 1000000i64;
        let start_time = current_time - UPGRADE_COOLDOWN;
        
        let farm_space_ready = FarmSpace {
            owner: Pubkey::new_unique(),
            level: 1,
            capacity: 4,
            seed_count: 0,
            total_grow_power: 0,
            upgrade_start_time: start_time,
            upgrade_target_level: 2,
            reserve: [0; 32],
        };
        
        let farm_space_not_ready = FarmSpace {
            upgrade_start_time: current_time - UPGRADE_COOLDOWN + 1,
            ..farm_space_ready
        };
        
        let farm_space_no_upgrade = FarmSpace {
            upgrade_start_time: 0,
            ..farm_space_ready
        };
        
        // Valid upgrade completion
        assert!(validate_upgrade_complete(&farm_space_ready, current_time).is_ok());
        
        // Invalid - not enough time passed
        assert!(validate_upgrade_complete(&farm_space_not_ready, current_time).is_err());
        
        // Invalid - no upgrade in progress
        assert!(validate_upgrade_complete(&farm_space_no_upgrade, current_time).is_err());
    }

    #[test]
    fn test_halving_timing_validation() {
        let start_time = 1000000i64;
        let halving_interval = SECONDS_PER_DAY * 6; // 6 days
        
        // Just started
        let halving_count = validate_halving_timing(start_time, start_time, halving_interval).unwrap();
        assert_eq!(halving_count, 0);
        
        // After 6 days (1 halving)
        let halving_count = validate_halving_timing(
            start_time, 
            start_time + halving_interval, 
            halving_interval
        ).unwrap();
        assert_eq!(halving_count, 1);
        
        // After 12 days (2 halvings)
        let halving_count = validate_halving_timing(
            start_time, 
            start_time + halving_interval * 2, 
            halving_interval
        ).unwrap();
        assert_eq!(halving_count, 2);
        
        // Invalid - before start time
        assert!(validate_halving_timing(start_time, start_time - 1, halving_interval).is_err());
        
        // Valid but extreme - 64 halvings
        let extreme_time = start_time + halving_interval * 64;
        let halving_count = validate_halving_timing(start_time, extreme_time, halving_interval).unwrap();
        assert_eq!(halving_count, 64);
        
        // Invalid - too many halvings
        let excessive_time = start_time + halving_interval * 65;
        assert!(validate_halving_timing(start_time, excessive_time, halving_interval).is_err());
    }
}