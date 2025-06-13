/// Common validation patterns used across the application
/// Centralized location for reusable validation logic

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::state::*;
use crate::error::GameError;
use crate::constants::*;

// ===== OWNERSHIP VALIDATIONS =====
// Note: Specific ownership validations moved to their respective domain modules
// - validate_user_ownership -> user_validation.rs
// - validate_seed_ownership -> game_validation.rs  
// - validate_farm_space_ownership -> game_validation.rs

/// Generic ownership validation for any structure with an owner field
pub fn validate_ownership<T>(account: &T, expected_owner: Pubkey, error: GameError) -> Result<()>
where
    T: HasOwner,
{
    require!(account.get_owner() == expected_owner, error);
    Ok(())
}

/// Trait for structs that have an owner field
pub trait HasOwner {
    fn get_owner(&self) -> Pubkey;
}

impl HasOwner for UserState {
    fn get_owner(&self) -> Pubkey { self.owner }
}

impl HasOwner for Seed {
    fn get_owner(&self) -> Pubkey { self.owner }
}

impl HasOwner for FarmSpace {
    fn get_owner(&self) -> Pubkey { self.owner }
}

impl HasOwner for SeedStorage {
    fn get_owner(&self) -> Pubkey { self.owner }
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

// Note: User state validation functions moved to user_validation.rs for better organization
// Re-exported through module namespacing in mod.rs

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
// Note: Seed-specific validations moved to game_validation.rs for better organization

// ===== INVITE CODE VALIDATIONS =====

/// Validate invite code format
pub fn validate_invite_code_format(code: &[u8; 8]) -> Result<()> {
    require!(
        crate::constants::validate_invite_code(code),
        GameError::InvalidInviteCode
    );
    Ok(())
}

/// Validate invite limit not exceeded (for hash-based system)
pub fn validate_secret_invite_limit(secret_invite_code: &InviteCode) -> Result<()> {
    require!(
        secret_invite_code.invites_used < secret_invite_code.invite_limit,
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

/// Validate seed storage has capacity for specific seed type
pub fn validate_seed_storage_capacity_for_type(seed_storage: &SeedStorage, seed_type: &SeedType) -> Result<()> {
    require!(seed_storage.can_add_seed_with_type(seed_type), GameError::SeedTypeLimitReached);
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

// ===== OPTIMIZED COMPOSITE VALIDATIONS =====

/// Optimized reward claim validation (combines multiple checks for efficiency)
pub fn validate_reward_claim_optimized(
    user_state: &UserState,
    global_stats: &GlobalStats,
    current_time: i64,
) -> Result<()> {
    // Combined checks for better performance
    require!(
        user_state.has_farm_space 
        && user_state.total_grow_power > 0
        && global_stats.total_grow_power > 0
        && current_time >= user_state.last_harvest_time + MIN_CLAIM_INTERVAL,
        GameError::NoRewardToClaim
    );
    Ok(())
}

/// Batch validation for ownership and basic requirements
pub fn validate_user_action_requirements(
    user_state: &UserState,
    expected_owner: Pubkey,
    requires_farm_space: bool,
    requires_grow_power: bool,
) -> Result<()> {
    // Single combined check for common requirements
    require!(
        user_state.owner == expected_owner
        && (!requires_farm_space || user_state.has_farm_space)
        && (!requires_grow_power || user_state.total_grow_power > 0),
        GameError::Unauthorized
    );
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