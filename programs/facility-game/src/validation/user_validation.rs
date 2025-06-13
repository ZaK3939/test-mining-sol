/// User-related validation functions
/// Handles user ownership, permissions, and state validation

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::GameError;

// ===== USER OWNERSHIP & PERMISSIONS =====

/// Validate user ownership of an account (centralized in user module)
pub fn validate_user_ownership(user_state: &UserState, expected_owner: Pubkey) -> Result<()> {
    require!(
        user_state.owner == expected_owner,
        GameError::Unauthorized
    );
    Ok(())
}

/// Validate user has a farm space
pub fn validate_has_farm_space(user_state: &UserState) -> Result<()> {
    require!(
        user_state.has_farm_space,
        GameError::NoFarmSpace
    );
    Ok(())
}

/// Validate user doesn't already have farm space
pub fn validate_no_existing_farm_space(user_state: &UserState) -> Result<()> {
    require!(
        !user_state.has_farm_space,
        GameError::AlreadyHasFarmSpace
    );
    Ok(())
}

/// Validate user has grow power
pub fn validate_has_grow_power(user_state: &UserState) -> Result<()> {
    require!(
        user_state.total_grow_power > 0,
        GameError::NoGrowPower
    );
    Ok(())
}

/// Validate user has sufficient grow power
pub fn validate_sufficient_grow_power(user_state: &UserState, required: u64) -> Result<()> {
    require!(
        user_state.total_grow_power >= required, 
        GameError::InsufficientFunds
    );
    Ok(())
}

// ===== REFERRAL VALIDATION =====

/// Validate referrer is not the same as the user
pub fn validate_referrer_not_self(user_key: Pubkey, referrer_key: Pubkey) -> Result<()> {
    require!(
        user_key != referrer_key,
        GameError::InvalidReferrer
    );
    Ok(())
}

/// Validate referrer is not protocol address
pub fn validate_referrer_not_protocol(referrer_key: Pubkey, protocol_address: Pubkey) -> Result<()> {
    require!(
        referrer_key != protocol_address,
        GameError::InvalidReferrer
    );
    Ok(())
}

/// Validate complete referral setup
pub fn validate_referral_setup(
    user_key: Pubkey,
    referrer_key: Pubkey,
    protocol_address: Pubkey
) -> Result<()> {
    validate_referrer_not_self(user_key, referrer_key)?;
    validate_referrer_not_protocol(referrer_key, protocol_address)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_ownership_validation() {
        let owner = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        
        let user_state = UserState {
            owner,
            total_grow_power: 100,
            last_harvest_time: 0,
            has_farm_space: true,
            referrer: None,
            pending_referral_rewards: 0,
            total_packs_purchased: 0,
            reserve: [0; 28],
        };

        // Valid ownership
        assert!(validate_user_ownership(&user_state, owner).is_ok());
        
        // Invalid ownership
        assert!(validate_user_ownership(&user_state, other).is_err());
    }

    #[test]
    fn test_farm_space_validation() {
        let user_state_with_farm = UserState {
            owner: Pubkey::new_unique(),
            total_grow_power: 100,
            last_harvest_time: 0,
            has_farm_space: true,
            referrer: None,
            pending_referral_rewards: 0,
            total_packs_purchased: 0,
            reserve: [0; 28],
        };

        let user_state_without_farm = UserState {
            has_farm_space: false,
            ..user_state_with_farm
        };

        // Has farm space
        assert!(validate_has_farm_space(&user_state_with_farm).is_ok());
        assert!(validate_no_existing_farm_space(&user_state_without_farm).is_ok());
        
        // No farm space
        assert!(validate_has_farm_space(&user_state_without_farm).is_err());
        assert!(validate_no_existing_farm_space(&user_state_with_farm).is_err());
    }

    #[test]
    fn test_grow_power_validation() {
        let user_with_power = UserState {
            owner: Pubkey::new_unique(),
            total_grow_power: 100,
            last_harvest_time: 0,
            has_farm_space: true,
            referrer: None,
            pending_referral_rewards: 0,
            total_packs_purchased: 0,
            reserve: [0; 28],
        };

        let user_without_power = UserState {
            total_grow_power: 0,
            ..user_with_power
        };

        // Has grow power
        assert!(validate_has_grow_power(&user_with_power).is_ok());
        
        // No grow power
        assert!(validate_has_grow_power(&user_without_power).is_err());
    }

    #[test]
    fn test_referral_validation() {
        let user = Pubkey::new_unique();
        let referrer = Pubkey::new_unique();
        let protocol = Pubkey::new_unique();

        // Valid referral setup
        assert!(validate_referral_setup(user, referrer, protocol).is_ok());
        
        // Invalid: self-referral
        assert!(validate_referral_setup(user, user, protocol).is_err());
        
        // Invalid: protocol referral
        assert!(validate_referral_setup(user, protocol, protocol).is_err());
    }
}