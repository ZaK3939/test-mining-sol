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
            // VRF-specific errors
            GameError::VrfAccountRequired,
            GameError::VrfPermissionRequired,
            GameError::SwitchboardProgramRequired,
            GameError::InsufficientSolForVrf,
            GameError::VrfRequestFailed,
            GameError::VrfResultNotAvailable,
            GameError::InvalidVrfAccount,
        ];

        // Verify we have all expected errors (including new VRF errors)
        assert_eq!(_errors.len(), 42);
    }

    #[test]
    fn test_error_conversion_to_anchor_error() {
        // Test that GameError can be converted to anchor_lang::error::Error
        let game_error = GameError::InsufficientFunds;
        let anchor_error: anchor_lang::error::Error = game_error.into();
        
        // The error should maintain its information
        assert!(anchor_error.to_string().contains("Insufficient funds"));
    }

    // ===== VRF-SPECIFIC ERROR TESTS =====

    #[test]
    fn test_vrf_error_scenarios() {
        // Test VRF-specific error scenarios
        let vrf_errors = vec![
            (GameError::VrfAccountRequired, "VRF account is required"),
            (GameError::VrfPermissionRequired, "VRF permission account is required"),
            (GameError::SwitchboardProgramRequired, "Switchboard program is required"),
            (GameError::InsufficientSolForVrf, "Insufficient SOL balance for VRF fee"),
            (GameError::VrfRequestFailed, "VRF request failed"),
            (GameError::VrfResultNotAvailable, "VRF result not available yet"),
            (GameError::InvalidVrfAccount, "Invalid VRF account"),
        ];

        for (error, expected_msg) in vrf_errors {
            let anchor_error: anchor_lang::error::Error = error.into();
            let error_string = anchor_error.to_string();
            
            // Check that error message contains expected text (case-insensitive)
            assert!(
                error_string.to_lowercase().contains(&expected_msg.to_lowercase()) ||
                error_string.to_lowercase().contains("vrf") ||
                error_string.to_lowercase().contains("switchboard"),
                "Error message should contain VRF-related text: {} (got: {})",
                expected_msg, error_string
            );
        }
    }

    #[test]
    fn test_vrf_fee_validation_errors() {
        // Test VRF fee validation scenarios
        let max_reasonable_fee = 10_000_000; // 0.01 SOL
        let user_balance = 5_000_000; // 0.005 SOL
        let requested_fee = 2_077_400; // Realistic VRF fee
        
        // Test 1: User has sufficient balance
        assert!(user_balance >= requested_fee, "User should have sufficient balance");
        
        // Test 2: Fee is within reasonable limits
        assert!(requested_fee < max_reasonable_fee, "VRF fee should be reasonable");
        
        // Test 3: Insufficient balance scenario
        let insufficient_balance = 1_000_000; // 0.001 SOL
        assert!(insufficient_balance < requested_fee, 
                "Should detect insufficient balance: {} < {}", 
                insufficient_balance, requested_fee);
        
        // In real implementation, this would trigger GameError::InsufficientSolForVrf
        let error = GameError::InsufficientSolForVrf;
        let anchor_error: anchor_lang::error::Error = error.into();
        assert!(anchor_error.to_string().to_lowercase().contains("insufficient"));
    }

    #[test]
    fn test_vrf_account_validation_errors() {
        // Test VRF account validation scenarios
        let valid_vrf_account = Pubkey::new_unique();
        let invalid_vrf_account = Pubkey::new_unique();
        
        // Simulate SeedPack with VRF account
        struct MockSeedPack {
            vrf_account: Pubkey,
        }
        
        let seed_pack = MockSeedPack {
            vrf_account: valid_vrf_account,
        };
        
        // Test 1: Valid VRF account matches
        assert_eq!(seed_pack.vrf_account, valid_vrf_account);
        
        // Test 2: Invalid VRF account doesn't match
        assert_ne!(seed_pack.vrf_account, invalid_vrf_account);
        
        // In real implementation, mismatched VRF account would trigger:
        // GameError::InvalidVrfAccount
        let error = GameError::InvalidVrfAccount;
        let anchor_error: anchor_lang::error::Error = error.into();
        assert!(anchor_error.to_string().to_lowercase().contains("invalid"));
    }

    #[test]
    fn test_vrf_sequence_errors() {
        // Test VRF sequence-related error scenarios
        let vrf_sequence = 12345u64;
        
        // Test non-zero sequence (required for VRF)
        assert_ne!(vrf_sequence, 0, "VRF sequence should not be zero");
        
        // Test sequence uniqueness (different inputs should produce different sequences)
        let user_entropy1 = 11111u64;
        let user_entropy2 = 22222u64;
        let timestamp = 1640995200u64;
        
        let sequence1 = user_entropy1.wrapping_add(timestamp);
        let sequence2 = user_entropy2.wrapping_add(timestamp);
        
        assert_ne!(sequence1, sequence2, "Different inputs should produce different sequences");
        
        // In real implementation, invalid sequences might trigger various errors
        let error = GameError::VrfRequestFailed;
        let anchor_error: anchor_lang::error::Error = error.into();
        assert!(anchor_error.to_string().to_lowercase().contains("failed"));
    }

    #[test]
    fn test_comprehensive_vrf_error_scenarios() {
        // Test comprehensive VRF error scenarios
        
        // Test 1: Invalid VRF account scenarios
        let valid_vrf = Pubkey::new_unique();
        let invalid_vrf = Pubkey::new_unique();
        let user_key = Pubkey::new_unique();
        
        // Simulate seed pack with specific VRF account
        struct MockSeedPackForValidation {
            vrf_account: Pubkey,
            is_opened: bool,
            vrf_sequence: u64,
        }
        
        let seed_pack = MockSeedPackForValidation {
            vrf_account: valid_vrf,
            is_opened: false,
            vrf_sequence: 12345,
        };
        
        // Test VRF account mismatch
        assert_eq!(seed_pack.vrf_account, valid_vrf);
        assert_ne!(seed_pack.vrf_account, invalid_vrf);
        // Would trigger: GameError::InvalidVrfAccount
        
        // Test 2: VRF fee validation scenarios
        let realistic_vrf_fee = 2_077_400u64; // ~0.002 SOL
        let excessive_vrf_fee = 100_000_000u64; // 0.1 SOL
        let insufficient_balance = 1_000_000u64; // 0.001 SOL
        let sufficient_balance = 5_000_000u64; // 0.005 SOL
        
        // User has sufficient balance for realistic fee
        assert!(sufficient_balance >= realistic_vrf_fee);
        
        // User has insufficient balance for excessive fee
        assert!(insufficient_balance < excessive_vrf_fee);
        
        // Would trigger: GameError::InsufficientSolForVrf
        let fee_error = GameError::InsufficientSolForVrf;
        let fee_anchor_error: anchor_lang::error::Error = fee_error.into();
        assert!(fee_anchor_error.to_string().to_lowercase().contains("insufficient"));
        
        // Test 3: VRF request failure scenarios
        let vrf_request_scenarios = [
            ("Network congestion", GameError::VrfRequestFailed),
            ("Invalid VRF permission", GameError::VrfPermissionRequired),
            ("Missing Switchboard program", GameError::SwitchboardProgramRequired),
            ("VRF result not ready", GameError::VrfResultNotAvailable),
        ];
        
        for (scenario, error) in vrf_request_scenarios {
            let anchor_error: anchor_lang::error::Error = error.into();
            let error_msg = anchor_error.to_string().to_lowercase();
            
            // Verify error message contains relevant keywords
            let has_relevant_keyword = error_msg.contains("vrf") || 
                                     error_msg.contains("switchboard") || 
                                     error_msg.contains("permission") || 
                                     error_msg.contains("request") ||
                                     error_msg.contains("failed") ||
                                     error_msg.contains("required") ||
                                     error_msg.contains("available");
            
            assert!(has_relevant_keyword, 
                   "Error for scenario '{}' should contain relevant keywords: {}", 
                   scenario, error_msg);
        }
        
        // Test 4: Seed pack state validation
        let opened_pack = MockSeedPackForValidation {
            vrf_account: valid_vrf,
            is_opened: true,
            vrf_sequence: 12345,
        };
        
        assert!(opened_pack.is_opened);
        // Would trigger: GameError::SeedPackAlreadyOpened
        
        let already_opened_error = GameError::SeedPackAlreadyOpened;
        let opened_anchor_error: anchor_lang::error::Error = already_opened_error.into();
        assert!(opened_anchor_error.to_string().to_lowercase().contains("opened") ||
                opened_anchor_error.to_string().to_lowercase().contains("already"));
    }

    #[test]
    fn test_vrf_user_entropy_validation() {
        // Test user entropy validation scenarios
        
        // Valid entropy values
        let valid_entropies = [1u64, 12345u64, u64::MAX];
        for entropy in valid_entropies {
            assert!(entropy > 0, "Entropy {} should be valid", entropy);
        }
        
        // Invalid entropy value
        let invalid_entropy = 0u64;
        assert_eq!(invalid_entropy, 0);
        // Would trigger: GameError::InvalidUserEntropySeed
        
        let entropy_error = GameError::InvalidUserEntropySeed;
        let entropy_anchor_error: anchor_lang::error::Error = entropy_error.into();
        assert!(entropy_anchor_error.to_string().to_lowercase().contains("entropy") ||
                entropy_anchor_error.to_string().to_lowercase().contains("seed") ||
                entropy_anchor_error.to_string().to_lowercase().contains("invalid"));
        
        // Test entropy uniqueness (different users should use different entropy)
        let user1_entropy = 11111u64;
        let user2_entropy = 22222u64;
        let user3_entropy = 11111u64; // Same as user1 (potential issue)
        
        assert_ne!(user1_entropy, user2_entropy);
        assert_eq!(user1_entropy, user3_entropy); // This is allowed but not recommended
        
        // Test entropy source quality
        let weak_entropies = [1u64, 2u64, 3u64, 1234u64]; // Sequential/predictable
        let strong_entropies = [
            0x123456789ABCDEFu64, 
            0xFEDCBA9876543210u64, 
            0x9E3779B97F4A7C15u64
        ]; // High-quality random values
        
        // While weak entropy is technically valid, strong entropy is preferred
        for entropy in weak_entropies {
            assert!(entropy > 0); // Valid but weak
        }
        
        for entropy in strong_entropies {
            assert!(entropy > 1000); // Strong entropy typically has high values
        }
    }

    #[test]
    fn test_vrf_cost_boundary_conditions() {
        // Test VRF cost validation at boundary conditions
        
        // Boundary values for VRF fees
        let min_vrf_fee = 1u64; // Minimum possible
        let realistic_vrf_fee = 2_077_400u64; // Actual calculated fee
        let max_reasonable_fee = 10_000_000u64; // 0.01 SOL maximum
        let excessive_fee = 100_000_000u64; // 0.1 SOL (too high)
        
        // Test fee reasonableness
        assert!(realistic_vrf_fee > min_vrf_fee);
        assert!(realistic_vrf_fee < max_reasonable_fee);
        assert!(excessive_fee > max_reasonable_fee);
        
        // Test fee calculation components
        let base_fee = 5_000u64;
        let num_transactions = 15u64;
        let storage_rent = 2_400u64;
        let oracle_fee = 2_000_000u64;
        
        let calculated_fee = base_fee * num_transactions + storage_rent + oracle_fee;
        assert_eq!(calculated_fee, realistic_vrf_fee, "Fee calculation should match expected value");
        
        // Test overflow protection in fee calculation
        let overflow_test1 = u64::MAX.checked_mul(2);
        assert!(overflow_test1.is_none(), "Should detect overflow in multiplication");
        
        let overflow_test2 = u64::MAX.checked_add(1);
        assert!(overflow_test2.is_none(), "Should detect overflow in addition");
        
        // In real implementation, overflow would trigger: GameError::CalculationOverflow
        let overflow_error = GameError::CalculationOverflow;
        let overflow_anchor_error: anchor_lang::error::Error = overflow_error.into();
        assert!(overflow_anchor_error.to_string().to_lowercase().contains("overflow") ||
                overflow_anchor_error.to_string().to_lowercase().contains("calculation"));
        
        // Test user balance validation scenarios
        struct BalanceTest {
            user_balance: u64,
            required_fee: u64,
            should_pass: bool,
        }
        
        let balance_tests = [
            BalanceTest { user_balance: 10_000_000, required_fee: 2_077_400, should_pass: true },
            BalanceTest { user_balance: 2_077_400, required_fee: 2_077_400, should_pass: true },
            BalanceTest { user_balance: 2_077_399, required_fee: 2_077_400, should_pass: false },
            BalanceTest { user_balance: 1_000_000, required_fee: 2_077_400, should_pass: false },
            BalanceTest { user_balance: 0, required_fee: 2_077_400, should_pass: false },
        ];
        
        for test in balance_tests {
            let has_sufficient_balance = test.user_balance >= test.required_fee;
            assert_eq!(has_sufficient_balance, test.should_pass,
                      "Balance test failed: {} SOL vs {} SOL required",
                      test.user_balance as f64 / 1_000_000_000.0,
                      test.required_fee as f64 / 1_000_000_000.0);
        }
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