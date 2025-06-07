use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("User already has a facility")]
    AlreadyHasFacility,
    
    #[msg("User does not have a facility")]
    NoFacility,
    
    #[msg("No reward to claim")]
    NoRewardToClaim,
    
    #[msg("Calculation overflow")]
    CalculationOverflow,
    
    #[msg("Invalid configuration")]
    InvalidConfig,
    
    #[msg("Unauthorized")]
    Unauthorized,
}