/// Game-specific validation functions  
/// Handles farm spaces, seeds, storage, and invite system validation

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::GameError;
use crate::constants::validate_invite_code;

// ===== FARM SPACE VALIDATION =====

/// Validate farm space has capacity for more seeds
pub fn validate_farm_space_capacity(farm_space: &FarmSpace) -> Result<()> {
    require!(
        farm_space.seed_count < farm_space.capacity,
        GameError::FarmSpaceCapacityExceeded
    );
    Ok(())
}

/// Validate farm space ownership
pub fn validate_farm_space_ownership(farm_space: &FarmSpace, expected_owner: Pubkey) -> Result<()> {
    require!(
        farm_space.owner == expected_owner,
        GameError::Unauthorized
    );
    Ok(())
}

/// Validate farm space can be upgraded
pub fn validate_can_upgrade_farm_space(farm_space: &FarmSpace) -> Result<()> {
    // Check if already at max level
    require!(
        farm_space.level < 5,
        GameError::MaxLevelReached
    );
    
    // Check if not already upgrading
    require!(
        farm_space.upgrade_start_time == 0,
        GameError::AlreadyUpgrading
    );
    
    Ok(())
}

/// Validate farm space level
pub fn validate_farm_space_level(level: u8) -> Result<()> {
    require!(
        crate::constants::validate_farm_level(level),
        GameError::InvalidConfig
    );
    Ok(())
}

// ===== SEED VALIDATION =====

/// Validate seed ownership
pub fn validate_seed_ownership(seed: &Seed, expected_owner: Pubkey) -> Result<()> {
    require!(
        seed.owner == expected_owner,
        GameError::NotSeedOwner
    );
    Ok(())
}

/// Validate seed is not already planted
pub fn validate_seed_not_planted(seed: &Seed) -> Result<()> {
    require!(
        !seed.is_planted,
        GameError::SeedAlreadyPlanted
    );
    Ok(())
}

/// Validate seed is planted
pub fn validate_seed_is_planted(seed: &Seed) -> Result<()> {
    require!(
        seed.is_planted,
        GameError::SeedNotPlanted
    );
    Ok(())
}

/// Validate seed is planted in specific farm space
pub fn validate_seed_in_farm_space(seed: &Seed, farm_space_key: Pubkey) -> Result<()> {
    require!(
        seed.planted_farm_space == Some(farm_space_key),
        GameError::SeedNotInThisFarmSpace
    );
    Ok(())
}

/// Validate seed type
pub fn validate_seed_type(seed_type_value: u8) -> Result<()> {
    require!(
        SeedType::is_valid(seed_type_value),
        GameError::InvalidQuantity
    );
    Ok(())
}

// ===== SEED PACK VALIDATION =====

/// Validate seed pack is not already opened
pub fn validate_seed_pack_not_opened(seed_pack: &SeedPack) -> Result<()> {
    require!(
        !seed_pack.is_opened,
        GameError::SeedPackAlreadyOpened
    );
    Ok(())
}

/// Validate seed pack ownership
pub fn validate_seed_pack_ownership(seed_pack: &SeedPack, expected_owner: Pubkey) -> Result<()> {
    require!(
        seed_pack.purchaser == expected_owner,
        GameError::NotSeedOwner
    );
    Ok(())
}

// ===== STORAGE VALIDATION =====

/// Validate seed storage has capacity
pub fn validate_seed_storage_capacity(storage: &SeedStorage) -> Result<()> {
    require!(
        storage.can_add_seed(),
        GameError::StorageFull
    );
    Ok(())
}

/// Validate seed storage ownership
pub fn validate_seed_storage_ownership(storage: &SeedStorage, expected_owner: Pubkey) -> Result<()> {
    require!(
        storage.owner == expected_owner,
        GameError::Unauthorized
    );
    Ok(())
}

// ===== INVITE SYSTEM VALIDATION =====

/// Validate invite code format
pub fn validate_invite_code_format(code: &[u8; 8]) -> Result<()> {
    require!(
        validate_invite_code(code),
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

// ===== ENTROPY VALIDATION =====

/// Validate entropy sequence matches
pub fn validate_entropy_sequence(expected: u64, actual: u64) -> Result<()> {
    require!(
        expected == actual,
        GameError::EntropySequenceMismatch
    );
    Ok(())
}

/// Validate user entropy seed is non-zero
pub fn validate_user_entropy_seed(seed: u64) -> Result<()> {
    require!(
        seed > 0,
        GameError::InvalidUserEntropySeed
    );
    Ok(())
}

// ===== COMPOSITE GAME VALIDATION =====

/// Validate complete farm space upgrade request
pub fn validate_farm_space_upgrade_request(
    farm_space: &FarmSpace,
    user_token_balance: u64,
    owner: Pubkey
) -> Result<u64> {
    validate_farm_space_ownership(farm_space, owner)?;
    validate_can_upgrade_farm_space(farm_space)?;
    
    let upgrade_cost = FarmSpace::get_upgrade_cost(farm_space.level)
        .ok_or(GameError::MaxLevelReached)?;
    
    crate::validation::economic_validation::validate_sufficient_balance(user_token_balance, upgrade_cost)?;
    
    Ok(upgrade_cost)
}

/// Validate complete seed planting request
pub fn validate_seed_planting_request(
    seed: &Seed,
    farm_space: &FarmSpace,
    user_key: Pubkey
) -> Result<()> {
    validate_seed_ownership(seed, user_key)?;
    validate_seed_not_planted(seed)?;
    validate_farm_space_ownership(farm_space, user_key)?;
    validate_farm_space_capacity(farm_space)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_farm_space_validation() {
        let owner = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        
        let farm_space = FarmSpace {
            owner,
            level: 2,
            capacity: 8,
            seed_count: 4,
            total_grow_power: 800,
            upgrade_start_time: 0,
            upgrade_target_level: 2,
            reserve: [0; 32],
        };
        
        let full_farm_space = FarmSpace {
            seed_count: 8, // At capacity
            ..farm_space
        };
        
        let max_level_farm = FarmSpace {
            level: 5, // Max level
            ..farm_space
        };
        
        let upgrading_farm = FarmSpace {
            upgrade_start_time: 1000000, // Currently upgrading
            ..farm_space
        };
        
        // Valid ownership
        assert!(validate_farm_space_ownership(&farm_space, owner).is_ok());
        
        // Invalid ownership
        assert!(validate_farm_space_ownership(&farm_space, other).is_err());
        
        // Has capacity
        assert!(validate_farm_space_capacity(&farm_space).is_ok());
        
        // No capacity
        assert!(validate_farm_space_capacity(&full_farm_space).is_err());
        
        // Can upgrade
        assert!(validate_can_upgrade_farm_space(&farm_space).is_ok());
        
        // Cannot upgrade (max level)
        assert!(validate_can_upgrade_farm_space(&max_level_farm).is_err());
        
        // Cannot upgrade (already upgrading)
        assert!(validate_can_upgrade_farm_space(&upgrading_farm).is_err());
    }

    #[test]
    fn test_seed_validation() {
        let owner = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        let farm_space_key = Pubkey::new_unique();
        
        let unplanted_seed = Seed {
            owner,
            seed_type: SeedType::Seed1,
            grow_power: 100,
            is_planted: false,
            planted_farm_space: None,
            created_at: 1000000,
            seed_id: 1,
            reserve: [0; 32],
        };
        
        let planted_seed = Seed {
            is_planted: true,
            planted_farm_space: Some(farm_space_key),
            ..unplanted_seed
        };
        
        // Valid ownership
        assert!(validate_seed_ownership(&unplanted_seed, owner).is_ok());
        
        // Invalid ownership
        assert!(validate_seed_ownership(&unplanted_seed, other).is_err());
        
        // Seed not planted
        assert!(validate_seed_not_planted(&unplanted_seed).is_ok());
        assert!(validate_seed_not_planted(&planted_seed).is_err());
        
        // Seed is planted
        assert!(validate_seed_is_planted(&planted_seed).is_ok());
        assert!(validate_seed_is_planted(&unplanted_seed).is_err());
        
        // Seed in farm space
        assert!(validate_seed_in_farm_space(&planted_seed, farm_space_key).is_ok());
        assert!(validate_seed_in_farm_space(&unplanted_seed, farm_space_key).is_err());
    }

    #[test]
    fn test_seed_pack_validation() {
        let owner = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        
        let unopened_pack = SeedPack {
            purchaser: owner,
            purchased_at: 1000000,
            cost_paid: 300_000_000,
            is_opened: false,
            entropy_sequence: 1,
            user_entropy_seed: 12345,
            final_random_value: 0,
            pack_id: 1,
            reserve: [0; 16],
        };
        
        let opened_pack = SeedPack {
            is_opened: true,
            ..unopened_pack
        };
        
        // Valid ownership
        assert!(validate_seed_pack_ownership(&unopened_pack, owner).is_ok());
        
        // Invalid ownership
        assert!(validate_seed_pack_ownership(&unopened_pack, other).is_err());
        
        // Pack not opened
        assert!(validate_seed_pack_not_opened(&unopened_pack).is_ok());
        
        // Pack already opened
        assert!(validate_seed_pack_not_opened(&opened_pack).is_err());
    }

    #[test]
    fn test_entropy_validation() {
        // Valid entropy sequence match
        assert!(validate_entropy_sequence(12345, 12345).is_ok());
        
        // Invalid entropy sequence mismatch
        assert!(validate_entropy_sequence(12345, 54321).is_err());
        
        // Valid user entropy seed
        assert!(validate_user_entropy_seed(12345).is_ok());
        
        // Invalid user entropy seed (zero)
        assert!(validate_user_entropy_seed(0).is_err());
    }

    #[test]
    fn test_composite_validation() {
        let owner = Pubkey::new_unique();
        let sufficient_balance = 100_000_000u64;
        let insufficient_balance = 1000u64;
        
        let farm_space = FarmSpace {
            owner,
            level: 1, // Can upgrade
            capacity: 4,
            seed_count: 2, // Has capacity
            total_grow_power: 200,
            upgrade_start_time: 0, // Not upgrading
            upgrade_target_level: 1,
            reserve: [0; 32],
        };
        
        let seed = Seed {
            owner,
            seed_type: SeedType::Seed1,
            grow_power: 100,
            is_planted: false,
            planted_farm_space: None,
            created_at: 1000000,
            seed_id: 1,
            reserve: [0; 32],
        };
        
        // Valid farm space upgrade request
        let cost = validate_farm_space_upgrade_request(&farm_space, sufficient_balance, owner).unwrap();
        assert!(cost > 0);
        
        // Invalid farm space upgrade request (insufficient balance)
        assert!(validate_farm_space_upgrade_request(&farm_space, insufficient_balance, owner).is_err());
        
        // Valid seed planting request
        assert!(validate_seed_planting_request(&seed, &farm_space, owner).is_ok());
        
        // Invalid seed planting request (wrong owner)
        let other_owner = Pubkey::new_unique();
        assert!(validate_seed_planting_request(&seed, &farm_space, other_owner).is_err());
    }
}