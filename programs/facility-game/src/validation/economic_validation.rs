/// Economic validation functions
/// Handles balance checks, reward limits, and economic constraints

use anchor_lang::prelude::*;
use crate::error::GameError;
use crate::constants::*;

// ===== BALANCE & PAYMENT VALIDATION =====

/// Validate token account has sufficient balance
pub fn validate_sufficient_balance(current_balance: u64, required_amount: u64) -> Result<()> {
    require!(
        current_balance >= required_amount,
        GameError::InsufficientFunds
    );
    Ok(())
}

/// Validate token account ownership
pub fn validate_token_account_owner(token_account: &anchor_spl::token::TokenAccount, expected_owner: Pubkey) -> Result<()> {
    require!(
        token_account.owner == expected_owner,
        GameError::Unauthorized
    );
    Ok(())
}

/// Validate mint authority  
pub fn validate_mint_authority(mint: &anchor_spl::token::Mint, expected_authority: Pubkey) -> Result<()> {
    // Convert COption to Option for easier handling
    let authority_option: Option<Pubkey> = mint.mint_authority.into();
    
    if let Some(authority) = authority_option {
        require!(
            authority == expected_authority,
            GameError::Unauthorized
        );
    } else {
        return Err(GameError::Unauthorized.into());
    }
    Ok(())
}

// ===== ECONOMIC CONSTRAINTS =====

/// Validate reward amount is reasonable
pub fn validate_reward_amount(amount: u64) -> Result<()> {
    // Prevent unreasonably large rewards (potential overflow attacks)
    require!(
        amount <= TOTAL_WEED_SUPPLY / 1000, // Max 0.1% of total supply per claim
        GameError::CalculationOverflow
    );
    Ok(())
}

/// Validate that minting would not exceed fixed supply cap
pub fn validate_supply_cap(current_minted: u64, amount_to_mint: u64) -> Result<()> {
    let new_total = current_minted.checked_add(amount_to_mint)
        .ok_or(GameError::CalculationOverflow)?;
    
    require!(
        new_total <= TOTAL_WEED_SUPPLY,
        GameError::SupplyCapExceeded
    );
    
    Ok(())
}

/// Check if supply cap has been reached
pub fn is_supply_exhausted(current_minted: u64) -> bool {
    current_minted >= TOTAL_WEED_SUPPLY
}

/// Calculate remaining supply
pub fn get_remaining_supply(current_minted: u64) -> u64 {
    TOTAL_WEED_SUPPLY.saturating_sub(current_minted)
}

/// Validate halving parameters
pub fn validate_halving_config(base_rate: u64, halving_interval: i64) -> Result<()> {
    require!(
        base_rate > 0,
        GameError::InvalidConfig
    );
    
    require!(
        halving_interval >= SECONDS_PER_HOUR, // At least 1 hour
        GameError::InvalidConfig
    );
    
    require!(
        halving_interval <= 365 * SECONDS_PER_DAY, // At most 1 year
        GameError::InvalidConfig
    );
    
    Ok(())
}

/// Validate global grow power is positive
pub fn validate_global_grow_power(total_grow_power: u64) -> Result<()> {
    require!(
        total_grow_power > 0,
        GameError::NoGlobalGrowPower
    );
    Ok(())
}

/// Validate treasury address is not zero
pub fn validate_treasury_address(treasury: Pubkey) -> Result<()> {
    require!(
        treasury != Pubkey::default(),
        GameError::InvalidConfig
    );
    Ok(())
}

// ===== QUANTITY VALIDATION =====

/// Validate purchase quantity
pub fn validate_purchase_quantity(quantity: u8) -> Result<()> {
    require!(
        validate_quantity(quantity),
        GameError::InvalidQuantity
    );
    Ok(())
}

/// Validate seed pack quantity
pub fn validate_seed_pack_quantity(quantity: u8) -> Result<()> {
    require!(
        quantity > 0 && quantity <= MAX_SEED_PACK_QUANTITY,
        GameError::InvalidQuantity
    );
    Ok(())
}

// ===== COMPOSITE ECONOMIC VALIDATION =====

/// Validate complete seed pack purchase request
pub fn validate_seed_pack_purchase_request(
    quantity: u8,
    user_token_balance: u64,
    user_entropy_seed: u64
) -> Result<u64> {
    validate_seed_pack_quantity(quantity)?;
    require!(user_entropy_seed > 0, GameError::InvalidUserEntropySeed);
    
    let total_cost = SEED_PACK_COST * quantity as u64;
    validate_sufficient_balance(user_token_balance, total_cost)?;
    
    Ok(total_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_validation() {
        // Sufficient balance
        assert!(validate_sufficient_balance(100, 50).is_ok());
        assert!(validate_sufficient_balance(100, 100).is_ok());
        
        // Insufficient balance
        assert!(validate_sufficient_balance(50, 100).is_err());
        assert!(validate_sufficient_balance(0, 1).is_err());
    }

    #[test]
    fn test_reward_amount_validation() {
        // Valid reward amounts
        assert!(validate_reward_amount(1000).is_ok());
        assert!(validate_reward_amount(TOTAL_WEED_SUPPLY / 1000).is_ok());
        
        // Invalid reward amount (too large)
        assert!(validate_reward_amount(TOTAL_WEED_SUPPLY / 1000 + 1).is_err());
    }

    #[test]
    fn test_halving_config_validation() {
        // Valid configuration
        assert!(validate_halving_config(100, SECONDS_PER_DAY).is_ok());
        
        // Invalid base rate
        assert!(validate_halving_config(0, SECONDS_PER_DAY).is_err());
        
        // Invalid interval (too short)
        assert!(validate_halving_config(100, SECONDS_PER_HOUR - 1).is_err());
        
        // Invalid interval (too long)
        assert!(validate_halving_config(100, 366 * SECONDS_PER_DAY).is_err());
    }

    #[test]
    fn test_quantity_validation() {
        // Valid quantities
        assert!(validate_seed_pack_quantity(1).is_ok());
        assert!(validate_seed_pack_quantity(MAX_SEED_PACK_QUANTITY).is_ok());
        
        // Invalid quantities
        assert!(validate_seed_pack_quantity(0).is_err());
        assert!(validate_seed_pack_quantity(MAX_SEED_PACK_QUANTITY + 1).is_err());
    }

    #[test]
    fn test_seed_pack_purchase_validation() {
        let sufficient_balance = 2_000_000_000u64;
        let insufficient_balance = 100_000u64;
        let valid_entropy = 12345u64;
        let invalid_entropy = 0u64;

        // Valid purchase
        let cost = validate_seed_pack_purchase_request(5, sufficient_balance, valid_entropy).unwrap();
        assert_eq!(cost, 5 * SEED_PACK_COST);

        // Invalid quantity
        assert!(validate_seed_pack_purchase_request(0, sufficient_balance, valid_entropy).is_err());

        // Insufficient balance
        assert!(validate_seed_pack_purchase_request(5, insufficient_balance, valid_entropy).is_err());

        // Invalid entropy
        assert!(validate_seed_pack_purchase_request(5, sufficient_balance, invalid_entropy).is_err());
    }

    #[test]
    fn test_supply_cap_validation() {
        let current_minted = 100_000_000 * 1_000_000u64; // 100M tokens minted
        let small_amount = 1_000_000u64; // 1 token
        let large_amount = 30_000_000 * 1_000_000u64; // 30M tokens
        
        // Valid minting within cap
        assert!(validate_supply_cap(current_minted, small_amount).is_ok());
        
        // Valid minting that reaches exactly the cap
        let remaining = TOTAL_WEED_SUPPLY - current_minted;
        assert!(validate_supply_cap(current_minted, remaining).is_ok());
        
        // Invalid minting that exceeds cap
        assert!(validate_supply_cap(current_minted, large_amount).is_err());
        
        // Invalid minting when already at cap
        assert!(validate_supply_cap(TOTAL_WEED_SUPPLY, 1).is_err());
    }

    #[test]
    fn test_supply_exhaustion_checks() {
        let current_minted = 119_999_999 * 1_000_000u64; // Almost at cap
        let at_cap = TOTAL_WEED_SUPPLY;
        
        // Not exhausted
        assert!(!is_supply_exhausted(current_minted));
        assert!(get_remaining_supply(current_minted) > 0);
        
        // Exhausted
        assert!(is_supply_exhausted(at_cap));
        assert_eq!(get_remaining_supply(at_cap), 0);
        
        // Over cap (shouldn't happen but test anyway)
        assert!(is_supply_exhausted(at_cap + 1));
        assert_eq!(get_remaining_supply(at_cap + 1), 0);
    }

    #[test]
    fn test_supply_calculation_overflow_protection() {
        let max_minted = u64::MAX - 1000;
        let amount_to_mint = 2000;
        
        // Should fail due to overflow in checked_add
        assert!(validate_supply_cap(max_minted, amount_to_mint).is_err());
    }
}