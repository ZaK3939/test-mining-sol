/// Validation module for game logic
/// Modular validation functions organized by domain

// Submodules for organized validation functions
pub mod user_validation;
pub mod economic_validation;
pub mod time_validation;
pub mod game_validation;
pub mod admin_validation;
pub mod common;

// Re-export common validations as the primary interface
pub use common::*;

// Re-export domain-specific validations with namespacing to avoid conflicts
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