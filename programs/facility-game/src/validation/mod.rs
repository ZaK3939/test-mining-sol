/// Validation module for game logic
/// Modular validation functions organized by domain

// Submodules for organized validation functions
pub mod user_validation;
pub mod economic_validation;
pub mod time_validation;
pub mod game_validation;
pub mod admin_validation;

// Re-export all validation functions for backward compatibility
pub use user_validation::*;
pub use economic_validation::*;
pub use time_validation::*;
pub use game_validation::*;
pub use admin_validation::*;

// Legacy composite validation functions for backwards compatibility
use anchor_lang::prelude::*;

/// Validate complete referral setup (legacy function)
pub fn validate_referral_setup(
    user_key: Pubkey,
    referrer_key: Pubkey,
    protocol_address: Pubkey
) -> Result<()> {
    user_validation::validate_referral_setup(user_key, referrer_key, protocol_address)
}

/// Validate complete seed pack purchase request (legacy function)
pub fn validate_seed_pack_purchase_request(
    quantity: u8,
    user_token_balance: u64,
    user_entropy_seed: u64
) -> Result<u64> {
    economic_validation::validate_seed_pack_purchase_request(
        quantity,
        user_token_balance,
        user_entropy_seed
    )
}