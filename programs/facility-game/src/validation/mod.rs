/// Validation module for game logic
/// Refactored for better organization and performance

// Core validation modules organized by domain
pub mod common;
pub mod user_validation;
pub mod economic_validation;
pub mod time_validation;
pub mod game_validation;
pub mod admin_validation;

// Re-export common utilities and optimized functions
pub use common::{
    HasOwner, 
    validate_ownership,
    validate_token_balance,
    validate_quantity_range,
    validate_batch_plant_size,
    validate_batch_remove_size,
    validate_batch_plant_capacity,
    validate_no_duplicate_seed_ids,
};

// Namespace exports for organized access to domain-specific validations
pub mod user {
    pub use super::user_validation::*;
}

pub mod economic {
    pub use super::economic_validation::*;
}

pub mod time {
    pub use super::time_validation::*;
}

pub mod game {
    pub use super::game_validation::*;
}

pub mod admin {
    pub use super::admin_validation::*;
}

// Convenience re-exports for commonly used validations
pub use user_validation::{validate_user_ownership, validate_has_farm_space, validate_has_grow_power};
pub use game_validation::{validate_seed_ownership, validate_farm_space_ownership};
pub use economic_validation::{validate_sufficient_balance, validate_supply_cap};
pub use time_validation::{validate_claim_interval, validate_timestamp_not_future};