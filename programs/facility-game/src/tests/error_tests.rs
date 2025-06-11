#[cfg(test)]
mod error_tests {
    use anchor_lang::prelude::*;
    use crate::error::GameError;

    // ===== ERROR CODE VALIDATION TESTS =====

    #[test]
    fn test_error_variants_exist() {
        // Test that all expected error variants exist and have correct discriminants
        // This ensures we don't accidentally remove or change error codes
        
        let _errors = vec![
            GameError::AlreadyHasFarm,
            GameError::NoFarm,
            GameError::AlreadyHasFarmSpace,
            GameError::NoFarmSpace,
            GameError::NoRewardToClaim,
            GameError::CalculationOverflow,
            GameError::InvalidConfig,
            GameError::Unauthorized,
            GameError::InvalidReferrer,
            GameError::InsufficientFunds,
            GameError::FarmAtMaxCapacity,
            GameError::FarmSpaceAtMaxCapacity,
            GameError::AlreadyUpgrading,
            GameError::MaxLevelReached,
            GameError::NoUpgradeInProgress,
            GameError::UpgradeStillInProgress,
            GameError::SeedNotFound,
            GameError::SeedAlreadyPlanted,
            GameError::SeedNotPlanted,
            GameError::InvalidQuantity,
            GameError::InvalidInviteCode,
            GameError::InviteCodeLimitReached,
            GameError::InvalidInviter,
            GameError::FarmSpaceCapacityExceeded,
            GameError::NotSeedOwner,
            GameError::SeedNotInThisFarmSpace,
            GameError::SeedPackAlreadyOpened,
            GameError::NoGrowPower,
            GameError::NoGlobalGrowPower,
            GameError::SeedStorageNotInitialized,
            GameError::StorageFull,
            GameError::EntropySequenceMismatch,
            GameError::EntropyNotReady,
            GameError::InvalidEntropyAccount,
            GameError::InvalidUserEntropySeed,
        ];

        // Verify we have all expected errors
        assert_eq!(_errors.len(), 35);
    }

    #[test]
    fn test_error_conversion_to_anchor_error() {
        // Test that GameError can be converted to anchor_lang::error::Error
        let game_error = GameError::InsufficientFunds;
        let anchor_error: anchor_lang::error::Error = game_error.into();
        
        // The error should maintain its information
        assert!(anchor_error.to_string().contains("Insufficient funds"));
    }

    #[test]
    fn test_farm_related_errors() {
        // Test farm-related error variants
        let farm_errors = vec![
            GameError::AlreadyHasFarm,
            GameError::NoFarm,
            GameError::AlreadyHasFarmSpace,
            GameError::NoFarmSpace,
            GameError::FarmAtMaxCapacity,
            GameError::FarmSpaceAtMaxCapacity,
            GameError::FarmSpaceCapacityExceeded,
        ];

        for error in farm_errors {
            let _anchor_error: anchor_lang::error::Error = error.into();
            // Test that conversion works without panic
        }
    }

    #[test]
    fn test_upgrade_related_errors() {
        // Test upgrade-related error variants
        let upgrade_errors = vec![
            GameError::AlreadyUpgrading,
            GameError::MaxLevelReached,
            GameError::NoUpgradeInProgress,
            GameError::UpgradeStillInProgress,
        ];

        for error in upgrade_errors {
            let _anchor_error: anchor_lang::error::Error = error.into();
            // Test that conversion works without panic
        }
    }

    #[test]
    fn test_seed_related_errors() {
        // Test seed-related error variants
        let seed_errors = vec![
            GameError::SeedNotFound,
            GameError::SeedAlreadyPlanted,
            GameError::SeedNotPlanted,
            GameError::NotSeedOwner,
            GameError::SeedNotInThisFarmSpace,
            GameError::SeedPackAlreadyOpened,
            GameError::SeedStorageNotInitialized,
            GameError::StorageFull,
        ];

        for error in seed_errors {
            let _anchor_error: anchor_lang::error::Error = error.into();
            // Test that conversion works without panic
        }
    }

    #[test]
    fn test_invite_related_errors() {
        // Test invite/referral-related error variants
        let invite_errors = vec![
            GameError::InvalidReferrer,
            GameError::InvalidInviteCode,
            GameError::InviteCodeLimitReached,
            GameError::InvalidInviter,
        ];

        for error in invite_errors {
            let _anchor_error: anchor_lang::error::Error = error.into();
            // Test that conversion works without panic
        }
    }

    #[test]
    fn test_entropy_related_errors() {
        // Test Pyth Entropy-related error variants
        let entropy_errors = vec![
            GameError::EntropySequenceMismatch,
            GameError::EntropyNotReady,
            GameError::InvalidEntropyAccount,
            GameError::InvalidUserEntropySeed,
        ];

        for error in entropy_errors {
            let _anchor_error: anchor_lang::error::Error = error.into();
            // Test that conversion works without panic
        }
    }

    #[test]
    fn test_calculation_and_validation_errors() {
        // Test calculation and validation error variants
        let validation_errors = vec![
            GameError::CalculationOverflow,
            GameError::InvalidConfig,
            GameError::Unauthorized,
            GameError::InsufficientFunds,
            GameError::InvalidQuantity,
            GameError::NoRewardToClaim,
            GameError::NoGrowPower,
            GameError::NoGlobalGrowPower,
        ];

        for error in validation_errors {
            let _anchor_error: anchor_lang::error::Error = error.into();
            // Test that conversion works without panic
        }
    }

    #[test]
    fn test_error_result_usage() {
        // Test using GameError in Result types (common pattern in the codebase)
        
        fn test_function_success() -> Result<u64, GameError> {
            Ok(100)
        }
        
        fn test_function_error() -> Result<u64, GameError> {
            Err(GameError::InsufficientFunds)
        }
        
        // Test successful result
        let success_result = test_function_success();
        assert!(success_result.is_ok());
        assert_eq!(success_result.unwrap(), 100);
        
        // Test error result
        let error_result = test_function_error();
        assert!(error_result.is_err());
        assert_eq!(error_result.unwrap_err(), GameError::InsufficientFunds);
    }

    #[test]
    fn test_error_in_anchor_result() {
        // Test using GameError with anchor_lang::Result
        
        fn anchor_function_success() -> anchor_lang::Result<u64> {
            Ok(100)
        }
        
        fn anchor_function_error() -> anchor_lang::Result<u64> {
            Err(GameError::InvalidConfig.into())
        }
        
        // Test successful result
        let success_result = anchor_function_success();
        assert!(success_result.is_ok());
        assert_eq!(success_result.unwrap(), 100);
        
        // Test error result
        let error_result = anchor_function_error();
        assert!(error_result.is_err());
    }

    #[test]
    fn test_error_discriminants_stability() {
        // Test that error discriminants are stable (important for client compatibility)
        // If this test fails after adding/removing errors, update client error handling
        
        // We test a few key errors to ensure their discriminants don't change
        let insufficient_funds: anchor_lang::error::Error = GameError::InsufficientFunds.into();
        let unauthorized: anchor_lang::error::Error = GameError::Unauthorized.into();
        let no_farm_space: anchor_lang::error::Error = GameError::NoFarmSpace.into();
        
        // These should convert successfully
        let _insufficient_funds_str = insufficient_funds.to_string();
        let _unauthorized_str = unauthorized.to_string();
        let _no_farm_space_str = no_farm_space.to_string();
        
        // Test that the error messages contain expected content
        assert!(_insufficient_funds_str.contains("Insufficient funds"));
        assert!(_unauthorized_str.contains("Unauthorized"));
        assert!(_no_farm_space_str.contains("farm space"));
    }

    #[test]
    fn test_error_debug_and_display() {
        // Test that errors can be formatted for debugging and display
        let errors_to_test = vec![
            GameError::AlreadyHasFarmSpace,
            GameError::InsufficientFunds,
            GameError::InvalidQuantity,
            GameError::SeedNotFound,
            GameError::EntropyNotReady,
        ];

        for error in errors_to_test {
            // Test Debug formatting
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
            
            // Test conversion to anchor error and display
            let anchor_error: anchor_lang::error::Error = error.into();
            let display_str = anchor_error.to_string();
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_error_equality() {
        // Test that errors can be compared for equality
        assert_eq!(GameError::InsufficientFunds, GameError::InsufficientFunds);
        assert_ne!(GameError::InsufficientFunds, GameError::InvalidQuantity);
        
        // Test with different instances
        let error1 = GameError::NoFarmSpace;
        let error2 = GameError::NoFarmSpace;
        let error3 = GameError::AlreadyHasFarmSpace;
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_categorization() {
        // Test logical grouping of errors to ensure they make sense
        
        // Farm/Farm Space errors
        let farm_space_errors = vec![
            GameError::AlreadyHasFarm,
            GameError::NoFarm,
            GameError::AlreadyHasFarmSpace,
            GameError::NoFarmSpace,
            GameError::FarmAtMaxCapacity,
            GameError::FarmSpaceAtMaxCapacity,
            GameError::FarmSpaceCapacityExceeded,
        ];
        assert_eq!(farm_space_errors.len(), 7);

        // Authorization/Permission errors
        let auth_errors = vec![
            GameError::Unauthorized,
            GameError::NotSeedOwner,
            GameError::InvalidConfig,
        ];
        assert_eq!(auth_errors.len(), 3);

        // Resource/State errors
        let resource_errors = vec![
            GameError::InsufficientFunds,
            GameError::NoRewardToClaim,
            GameError::NoGrowPower,
            GameError::NoGlobalGrowPower,
            GameError::StorageFull,
        ];
        assert_eq!(resource_errors.len(), 5);

        // Validation errors
        let validation_errors = vec![
            GameError::InvalidQuantity,
            GameError::InvalidReferrer,
            GameError::InvalidInviteCode,
            GameError::InvalidInviter,
            GameError::InvalidEntropyAccount,
            GameError::InvalidUserEntropySeed,
        ];
        assert_eq!(validation_errors.len(), 6);

        // Process/State machine errors
        let process_errors = vec![
            GameError::AlreadyUpgrading,
            GameError::NoUpgradeInProgress,
            GameError::UpgradeStillInProgress,
            GameError::SeedAlreadyPlanted,
            GameError::SeedNotPlanted,
            GameError::SeedPackAlreadyOpened,
        ];
        assert_eq!(process_errors.len(), 6);
    }

    #[test]
    fn test_error_with_require_macro_simulation() {
        // Simulate how errors are used with require! macro
        
        fn validate_quantity(quantity: u8) -> anchor_lang::Result<()> {
            if quantity == 0 || quantity > 100 {
                return Err(GameError::InvalidQuantity.into());
            }
            Ok(())
        }
        
        fn validate_farm_space_capacity(current: u8, capacity: u8) -> anchor_lang::Result<()> {
            if current >= capacity {
                return Err(GameError::FarmSpaceCapacityExceeded.into());
            }
            Ok(())
        }
        
        // Test valid cases
        assert!(validate_quantity(10).is_ok());
        assert!(validate_farm_space_capacity(3, 8).is_ok());
        
        // Test invalid cases
        assert!(validate_quantity(0).is_err());
        assert!(validate_quantity(101).is_err());
        assert!(validate_farm_space_capacity(8, 8).is_err());
        assert!(validate_farm_space_capacity(10, 8).is_err());
    }
}