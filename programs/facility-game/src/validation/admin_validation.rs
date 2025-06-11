/// Admin-related validation functions
/// Handles administrative privileges, configuration validation, and system constraints

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::GameError;

// ===== ADMIN AUTHORIZATION =====

/// Validate admin signature
pub fn validate_admin_authority(config: &Config, signer: Pubkey) -> Result<()> {
    require!(
        config.admin == signer,
        GameError::Unauthorized
    );
    Ok(())
}

// Note: validate_treasury_address is defined in economic_validation.rs
// to avoid duplicate function conflicts

/// Validate system is not paused for user actions
/// Note: Current Config struct doesn't have is_paused field
/// This is a placeholder for future implementation
pub fn validate_system_not_paused(_config: &Config) -> Result<()> {
    // For now, system is never paused since Config doesn't have is_paused field
    // This can be implemented when the pause functionality is added
    Ok(())
}

/// Validate admin can pause/unpause system
pub fn validate_admin_can_pause(config: &Config, admin: Pubkey) -> Result<()> {
    validate_admin_authority(config, admin)?;
    // Additional checks can be added here (e.g., emergency conditions)
    Ok(())
}

// ===== CONFIGURATION VALIDATION =====

/// Validate complete system configuration
pub fn validate_system_config(
    admin: Pubkey,
    treasury: Pubkey,
    base_rate: u64,
    halving_interval: i64
) -> Result<()> {
    // Validate admin is not zero address
    require!(
        admin != Pubkey::default(),
        GameError::InvalidConfig
    );
    
    // Validate treasury
    crate::validation::economic_validation::validate_treasury_address(treasury)?;
    
    // Validate economic parameters
    crate::validation::economic_validation::validate_halving_config(base_rate, halving_interval)?;
    
    Ok(())
}

/// Validate reward mint configuration
pub fn validate_reward_mint_config(
    decimals: u8,
    total_supply: u64
) -> Result<()> {
    // Validate decimals are reasonable (6-9 is typical for tokens)
    require!(
        decimals >= 6 && decimals <= 9,
        GameError::InvalidConfig
    );
    
    // Validate total supply is reasonable (not zero, not excessive)
    require!(
        total_supply > 0 && total_supply <= u64::MAX / 1000, // Leave room for calculations
        GameError::InvalidConfig
    );
    
    Ok(())
}

/// Validate admin update request
pub fn validate_admin_update_request(
    current_config: &Config,
    current_admin: Pubkey,
    new_admin: Pubkey
) -> Result<()> {
    // Validate current admin authority
    validate_admin_authority(current_config, current_admin)?;
    
    // Validate new admin is not zero address
    require!(
        new_admin != Pubkey::default(),
        GameError::InvalidConfig
    );
    
    // Prevent setting same admin (unnecessary operation)
    require!(
        new_admin != current_admin,
        GameError::InvalidConfig
    );
    
    Ok(())
}

/// Validate emergency pause conditions
pub fn validate_emergency_pause_conditions(
    config: &Config,
    admin: Pubkey,
    total_grow_power: u64
) -> Result<()> {
    validate_admin_authority(config, admin)?;
    
    // Only allow pause if system has grown beyond initial state
    // This prevents accidental pausing during initial setup
    require!(
        total_grow_power > 0,
        GameError::InvalidConfig
    );
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_authority_validation() {
        let admin = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        
        let config = Config {
            admin,
            treasury: Pubkey::new_unique(),
            base_rate: 100,
            halving_interval: 518400, // 6 days
            next_halving_time: 0,
            seed_pack_cost: 300_000_000,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: 500_000_000,
            max_invite_limit: 5,
            trading_fee_percentage: 2,
            protocol_referral_address: Pubkey::new_unique(),
            total_supply_minted: 0,
            reserve: [0; 2],
        };
        
        // Valid admin authority
        assert!(validate_admin_authority(&config, admin).is_ok());
        
        // Invalid admin authority
        assert!(validate_admin_authority(&config, other).is_err());
    }

    // Note: Treasury validation tests are in economic_validation.rs

    #[test]
    fn test_system_config_validation() {
        let admin = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let zero_address = Pubkey::default();
        
        // Valid system configuration
        assert!(validate_system_config(admin, treasury, 100, 518400).is_ok());
        
        // Invalid admin (zero address)
        assert!(validate_system_config(zero_address, treasury, 100, 518400).is_err());
        
        // Invalid treasury (zero address)
        assert!(validate_system_config(admin, zero_address, 100, 518400).is_err());
        
        // Invalid base rate (zero)
        assert!(validate_system_config(admin, treasury, 0, 518400).is_err());
        
        // Invalid halving interval (too short)
        assert!(validate_system_config(admin, treasury, 100, 1800).is_err()); // 30 minutes
    }

    #[test]
    fn test_reward_mint_config_validation() {
        let reasonable_supply = 1_000_000_000_000_000u64; // 1 quadrillion (within limit)
        let max_allowed_supply = u64::MAX / 1000;
        
        // Valid configurations
        assert!(validate_reward_mint_config(6, reasonable_supply).is_ok()); // 6 decimals
        assert!(validate_reward_mint_config(9, reasonable_supply).is_ok()); // 9 decimals
        assert!(validate_reward_mint_config(6, max_allowed_supply).is_ok()); // At limit
        
        // Invalid decimals (too few)
        assert!(validate_reward_mint_config(5, reasonable_supply).is_err());
        
        // Invalid decimals (too many)
        assert!(validate_reward_mint_config(10, reasonable_supply).is_err());
        
        // Invalid total supply (zero)
        assert!(validate_reward_mint_config(6, 0).is_err());
        
        // Invalid total supply (excessive)
        assert!(validate_reward_mint_config(6, max_allowed_supply + 1).is_err());
        assert!(validate_reward_mint_config(6, u64::MAX).is_err());
    }
}