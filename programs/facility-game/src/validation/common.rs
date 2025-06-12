/// Common validation patterns used across the application
/// Centralized location for reusable validation logic

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::state::*;
use crate::error::GameError;
use crate::constants::*;

// ===== OWNERSHIP VALIDATIONS =====

/// Validate user ownership of account
pub fn validate_user_ownership(user_state: &UserState, expected_owner: Pubkey) -> Result<()> {
    require!(user_state.owner == expected_owner, GameError::Unauthorized);
    Ok(())
}

/// Validate seed ownership
pub fn validate_seed_ownership(seed: &Seed, expected_owner: Pubkey) -> Result<()> {
    require!(seed.owner == expected_owner, GameError::NotSeedOwner);
    Ok(())
}

/// Validate farm space ownership
pub fn validate_farm_space_ownership(farm_space: &FarmSpace, expected_owner: Pubkey) -> Result<()> {
    require!(farm_space.owner == expected_owner, GameError::Unauthorized);
    Ok(())
}

// ===== BALANCE AND CAPACITY VALIDATIONS =====

/// Validate token account balance is sufficient
pub fn validate_token_balance(account: &TokenAccount, required: u64) -> Result<()> {
    require!(account.amount >= required, GameError::InsufficientFunds);
    Ok(())
}

/// Validate farm space has capacity for more seeds
pub fn validate_farm_space_capacity(farm_space: &FarmSpace) -> Result<()> {
    require!(
        farm_space.seed_count < farm_space.capacity,
        GameError::FarmSpaceCapacityExceeded
    );
    Ok(())
}

/// Validate specific capacity requirements
pub fn validate_capacity_requirement(current: u8, max: u8, adding: u8) -> Result<()> {
    require!(
        current + adding <= max,
        GameError::FarmSpaceCapacityExceeded
    );
    Ok(())
}

// ===== USER STATE VALIDATIONS =====

/// Validate user has farm space
pub fn validate_has_farm_space(user_state: &UserState) -> Result<()> {
    require!(user_state.has_farm_space, GameError::NoFarmSpace);
    Ok(())
}

/// Validate user doesn't already have farm space
pub fn validate_no_farm_space(user_state: &UserState) -> Result<()> {
    require!(!user_state.has_farm_space, GameError::AlreadyHasFarmSpace);
    Ok(())
}

/// Validate user has grow power
pub fn validate_has_grow_power(user_state: &UserState) -> Result<()> {
    require!(user_state.total_grow_power > 0, GameError::NoGrowPower);
    Ok(())
}

/// Validate user has sufficient grow power
pub fn validate_sufficient_grow_power(user_state: &UserState, required: u64) -> Result<()> {
    require!(user_state.total_grow_power >= required, GameError::InsufficientFunds);
    Ok(())
}

// ===== QUANTITY AND RANGE VALIDATIONS =====

/// Validate quantity is within acceptable range
pub fn validate_quantity_range(quantity: u8, min: u8, max: u8) -> Result<()> {
    require!(quantity >= min && quantity <= max, GameError::InvalidQuantity);
    Ok(())
}

/// Validate seed pack quantity
pub fn validate_seed_pack_quantity(quantity: u8) -> Result<()> {
    validate_quantity_range(quantity, MIN_QUANTITY, MAX_SEED_PACK_QUANTITY)
}

/// Validate farm level
pub fn validate_farm_level(level: u8) -> Result<()> {
    require!(level >= 1 && level <= 5, GameError::InvalidConfig);
    Ok(())
}

/// Validate referral level
pub fn validate_referral_level(level: u8) -> Result<()> {
    require!(level >= 1 && level <= MAX_REFERRAL_DEPTH, GameError::InvalidReferralLevel);
    Ok(())
}

// ===== SEED VALIDATIONS =====

/// Validate seed is not already planted
pub fn validate_seed_not_planted(seed: &Seed) -> Result<()> {
    require!(!seed.is_planted, GameError::SeedAlreadyPlanted);
    Ok(())
}

/// Validate seed is planted
pub fn validate_seed_is_planted(seed: &Seed) -> Result<()> {
    require!(seed.is_planted, GameError::SeedNotPlanted);
    Ok(())
}

/// Validate seed is in specific farm space
pub fn validate_seed_in_farm_space(seed: &Seed, farm_space_key: Pubkey) -> Result<()> {
    require!(
        seed.planted_farm_space == Some(farm_space_key),
        GameError::SeedNotInThisFarmSpace
    );
    Ok(())
}

/// Validate seed type is valid
pub fn validate_seed_type(seed_type: &SeedType) -> Result<()> {
    require!(
        SeedType::is_valid(*seed_type as u8),
        GameError::InvalidConfig
    );
    Ok(())
}

// ===== UPGRADE VALIDATIONS =====

/// Validate farm space can be upgraded
pub fn validate_can_upgrade_farm_space(farm_space: &FarmSpace) -> Result<()> {
    require!(farm_space.level < 5, GameError::MaxLevelReached);
    require!(farm_space.upgrade_start_time == 0, GameError::AlreadyUpgrading);
    Ok(())
}

/// Validate upgrade is in progress
pub fn validate_upgrade_in_progress(farm_space: &FarmSpace) -> Result<()> {
    require!(farm_space.upgrade_start_time > 0, GameError::NoUpgradeInProgress);
    Ok(())
}

/// Validate upgrade cooldown is complete
pub fn validate_upgrade_cooldown_complete(farm_space: &FarmSpace, current_time: i64) -> Result<()> {
    require!(
        farm_space.is_upgrade_complete(current_time),
        GameError::UpgradeStillInProgress
    );
    Ok(())
}

// ===== INVITE CODE VALIDATIONS =====

/// Validate invite code format
pub fn validate_invite_code_format(code: &[u8; 8]) -> Result<()> {
    require!(
        crate::constants::validate_invite_code(code),
        GameError::InvalidInviteCode
    );
    Ok(())
}

/// Validate invite limit not exceeded
pub fn validate_invite_limit(invite_code: &InviteCode) -> Result<()> {
    require!(
        invite_code.invites_used < invite_code.invite_limit,
        GameError::InviteCodeLimitReached
    );
    Ok(())
}

/// Validate referrer is not the same as user
pub fn validate_different_referrer(user: Pubkey, referrer: Pubkey) -> Result<()> {
    require!(user != referrer, GameError::InvalidReferrer);
    Ok(())
}

// ===== TIME VALIDATIONS =====

/// Validate minimum time interval between claims
pub fn validate_claim_interval(last_claim_time: i64, current_time: i64) -> Result<()> {
    require!(
        current_time >= last_claim_time + MIN_CLAIM_INTERVAL,
        GameError::NoRewardToClaim
    );
    Ok(())
}

/// Validate time is not in the future (with small tolerance)
pub fn validate_time_not_future(time: i64, current_time: i64, tolerance: i64) -> Result<()> {
    require!(
        time <= current_time + tolerance,
        GameError::InvalidConfig
    );
    Ok(())
}

/// Validate time range is positive
pub fn validate_positive_time_range(start_time: i64, end_time: i64) -> Result<()> {
    require!(end_time > start_time, GameError::InvalidConfig);
    Ok(())
}

// ===== AMOUNT VALIDATIONS =====

/// Validate amount is not zero
pub fn validate_non_zero_amount(amount: u64) -> Result<()> {
    require!(amount > 0, GameError::InvalidAmount);
    Ok(())
}

/// Validate amount is within reasonable bounds
pub fn validate_amount_bounds(amount: u64, min: u64, max: u64) -> Result<()> {
    require!(amount >= min && amount <= max, GameError::InvalidAmount);
    Ok(())
}

/// Validate sufficient balance for operation
pub fn validate_sufficient_balance(balance: u64, required: u64) -> Result<()> {
    require!(balance >= required, GameError::InsufficientFunds);
    Ok(())
}

// ===== SUPPLY VALIDATIONS =====

/// Validate total supply cap is not exceeded
pub fn validate_supply_cap(current_supply: u64, additional_mint: u64) -> Result<()> {
    let new_total = current_supply.checked_add(additional_mint)
        .ok_or(GameError::CalculationOverflow)?;
    
    require!(new_total <= TOTAL_WEED_SUPPLY, GameError::SupplyCapExceeded);
    Ok(())
}

/// Validate global grow power consistency
pub fn validate_global_grow_power(global_stats: &GlobalStats) -> Result<()> {
    require!(global_stats.total_grow_power > 0, GameError::NoGlobalGrowPower);
    Ok(())
}

// ===== STORAGE VALIDATIONS =====

/// Validate seed storage has capacity
pub fn validate_seed_storage_capacity(seed_storage: &SeedStorage) -> Result<()> {
    require!(seed_storage.can_add_seed(), GameError::StorageFull);
    Ok(())
}

/// Validate seed storage is initialized
pub fn validate_seed_storage_initialized(seed_storage: &SeedStorage, expected_owner: Pubkey) -> Result<()> {
    require!(seed_storage.owner == expected_owner, GameError::SeedStorageNotInitialized);
    Ok(())
}

// ===== BATCH OPERATION VALIDATIONS =====

/// Validate batch size is within limits
pub fn validate_batch_size<T>(items: &[T], max_size: usize) -> Result<()> {
    require!(items.len() <= max_size, GameError::TooManyTransfers);
    require!(!items.is_empty(), GameError::InvalidQuantity);
    Ok(())
}

/// Validate all items in batch are unique
pub fn validate_unique_items<T: PartialEq>(items: &[T]) -> Result<()> {
    for (i, item) in items.iter().enumerate() {
        for other in items.iter().skip(i + 1) {
            require!(item != other, GameError::InvalidQuantity);
        }
    }
    Ok(())
}

// ===== COMPOSITE VALIDATIONS =====

/// Validate complete seed planting scenario
pub fn validate_seed_planting(
    seed: &Seed,
    farm_space: &FarmSpace,
    user: Pubkey,
) -> Result<()> {
    validate_seed_ownership(seed, user)?;
    validate_farm_space_ownership(farm_space, user)?;
    validate_seed_not_planted(seed)?;
    validate_farm_space_capacity(farm_space)?;
    Ok(())
}

/// Validate complete seed removal scenario
pub fn validate_seed_removal(
    seed: &Seed,
    farm_space: &FarmSpace,
    user: Pubkey,
) -> Result<()> {
    validate_seed_ownership(seed, user)?;
    validate_farm_space_ownership(farm_space, user)?;
    validate_seed_is_planted(seed)?;
    validate_seed_in_farm_space(seed, farm_space.owner)?;
    Ok(())
}

/// Validate complete reward claim scenario
pub fn validate_reward_claim(
    user_state: &UserState,
    global_stats: &GlobalStats,
    current_time: i64,
) -> Result<()> {
    validate_has_farm_space(user_state)?;
    validate_has_grow_power(user_state)?;
    validate_global_grow_power(global_stats)?;
    validate_claim_interval(user_state.last_harvest_time, current_time)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_validations() {
        // Valid quantities
        assert!(validate_quantity_range(1, 1, 10).is_ok());
        assert!(validate_quantity_range(5, 1, 10).is_ok());
        assert!(validate_quantity_range(10, 1, 10).is_ok());
        
        // Invalid quantities
        assert!(validate_quantity_range(0, 1, 10).is_err());
        assert!(validate_quantity_range(11, 1, 10).is_err());
    }

    #[test]
    fn test_farm_level_validation() {
        // Valid levels
        for level in 1..=5 {
            assert!(validate_farm_level(level).is_ok());
        }
        
        // Invalid levels
        assert!(validate_farm_level(0).is_err());
        assert!(validate_farm_level(6).is_err());
    }

    #[test]
    fn test_amount_validations() {
        assert!(validate_non_zero_amount(1).is_ok());
        assert!(validate_non_zero_amount(0).is_err());
        
        assert!(validate_amount_bounds(50, 10, 100).is_ok());
        assert!(validate_amount_bounds(5, 10, 100).is_err());
        assert!(validate_amount_bounds(150, 10, 100).is_err());
    }

    #[test]
    fn test_time_validations() {
        let current_time = 1000;
        let past_time = 900;
        let future_time = 1100;
        
        assert!(validate_claim_interval(past_time, current_time).is_ok());
        assert!(validate_claim_interval(current_time, current_time).is_err());
        
        assert!(validate_time_not_future(past_time, current_time, 10).is_ok());
        assert!(validate_time_not_future(current_time + 5, current_time, 10).is_ok());
        assert!(validate_time_not_future(future_time, current_time, 10).is_err());
        
        assert!(validate_positive_time_range(900, 1000).is_ok());
        assert!(validate_positive_time_range(1000, 900).is_err());
    }

    #[test]
    fn test_batch_validations() {
        let items = vec![1, 2, 3, 4, 5];
        assert!(validate_batch_size(&items, 10).is_ok());
        assert!(validate_batch_size(&items, 3).is_err());
        
        let empty_items: Vec<u32> = vec![];
        assert!(validate_batch_size(&empty_items, 10).is_err());
        
        let unique_items = vec![1, 2, 3];
        let duplicate_items = vec![1, 2, 1];
        assert!(validate_unique_items(&unique_items).is_ok());
        assert!(validate_unique_items(&duplicate_items).is_err());
    }

    #[test]
    fn test_supply_cap_validation() {
        let current_supply = 100_000_000 * 1_000_000; // 100M WEED
        let small_mint = 1_000_000 * 1_000_000; // 1M WEED
        let large_mint = 50_000_000 * 1_000_000; // 50M WEED (would exceed cap)
        
        assert!(validate_supply_cap(current_supply, small_mint).is_ok());
        assert!(validate_supply_cap(current_supply, large_mint).is_err());
    }
}