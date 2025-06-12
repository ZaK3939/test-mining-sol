#[cfg(test)]
mod error_comprehensive_tests {
    use super::super::*;
    use crate::error::GameError;
    use crate::validation::*;
    use crate::constants::*;
    use crate::state::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_all_error_variants() {
        // Test that all error variants can be created and have correct codes
        let errors = [
            GameError::Unauthorized,
            GameError::InsufficientFunds,
            GameError::InvalidQuantity,
            GameError::NoFarmSpace,
            GameError::NoGrowPower,
            GameError::NoRewardToClaim,
            GameError::CalculationOverflow,
            GameError::InvalidConfig,
            GameError::AlreadyHasFarmSpace,
            GameError::FarmSpaceCapacityExceeded,
            GameError::MaxLevelReached,
            GameError::AlreadyUpgrading,
            GameError::NoUpgradeInProgress,
            GameError::UpgradeStillInProgress,
            GameError::NotSeedOwner,
            GameError::SeedAlreadyPlanted,
            GameError::SeedNotPlanted,
            GameError::SeedNotInThisFarmSpace,
            GameError::SeedPackAlreadyOpened,
            GameError::StorageFull,
            GameError::SeedStorageNotInitialized,
            GameError::InvalidInviteCode,
            GameError::InviteCodeLimitReached,
            GameError::InvalidReferrer,
            GameError::NoGlobalGrowPower,
            GameError::InvalidUserEntropySeed,
            GameError::EntropyNotReady,
            GameError::EntropySequenceMismatch,
            GameError::InvalidEntropyAccount,
            GameError::MeteoraIntegrationNotReady,
            GameError::SlippageExceeded,
            GameError::InvalidTokenAccount,
            GameError::InvalidMintAuthority,
            GameError::TransferFailed,
            GameError::MintFailed,
            GameError::BurnFailed,
            GameError::InvalidProgramId,
        ];

        for error in &errors {
            // Each error should have a unique code
            let error_code = *error as u32;
            assert!(error_code >= 6000 && error_code < 6100, 
                   "Error code {} should be in range 6000-6099", error_code);
        }

        // Test error messages are meaningful
        assert!(format!("{}", GameError::Unauthorized).contains("Unauthorized"));
        assert!(format!("{}", GameError::InsufficientFunds).contains("Insufficient"));
        assert!(format!("{}", GameError::NoFarmSpace).contains("farm space"));
        assert!(format!("{}", GameError::CalculationOverflow).contains("overflow"));
    }

    #[test]
    fn test_validation_error_scenarios() {
        let owner = Pubkey::new_unique();
        let other_user = Pubkey::new_unique();
        
        // Test user ownership validation
        let user_state = UserState {
            owner,
            total_grow_power: 0,
            last_harvest_time: 0,
            has_farm_space: false,
            referrer: None,
            pending_referral_rewards: 0,
            reserve: [0; 32],
        };
        
        assert!(validate_user_ownership(&user_state, owner).is_ok());
        assert!(validate_user_ownership(&user_state, other_user).is_err());
        
        // Test farm space validation
        assert!(validate_has_farm_space(&user_state).is_err());
        let user_with_farm = UserState {
            has_farm_space: true,
            ..user_state
        };
        assert!(validate_has_farm_space(&user_with_farm).is_ok());
        
        // Test grow power validation
        assert!(validate_has_grow_power(&user_state).is_err());
        let user_with_power = UserState {
            total_grow_power: 100,
            ..user_state
        };
        assert!(validate_has_grow_power(&user_with_power).is_ok());
    }

    #[test]
    fn test_economic_validation_errors() {
        // Test insufficient balance
        assert!(validate_sufficient_balance(100, 50).is_ok());
        assert!(validate_sufficient_balance(50, 100).is_err());
        assert!(validate_sufficient_balance(100, 100).is_ok());
        assert!(validate_sufficient_balance(0, 1).is_err());
        
        // Test reward amount validation
        assert!(validate_reward_amount(1000).is_ok());
        assert!(validate_reward_amount(TOTAL_WEED_SUPPLY / 1000).is_ok());
        assert!(validate_reward_amount(TOTAL_WEED_SUPPLY / 1000 + 1).is_err());
        
        // Test halving configuration validation
        assert!(validate_halving_config(100, SECONDS_PER_DAY).is_ok());
        assert!(validate_halving_config(0, SECONDS_PER_DAY).is_err()); // Zero base rate
        assert!(validate_halving_config(100, SECONDS_PER_HOUR - 1).is_err()); // Too short interval
        assert!(validate_halving_config(100, 366 * SECONDS_PER_DAY).is_err()); // Too long interval
        
        // Test global grow power validation
        assert!(validate_global_grow_power(1).is_ok());
        assert!(validate_global_grow_power(1000000).is_ok());
        assert!(validate_global_grow_power(0).is_err());
    }

    #[test]
    fn test_farm_space_validation_errors() {
        let owner = Pubkey::new_unique();
        let other_user = Pubkey::new_unique();
        
        // Test farm space at capacity
        let full_farm = FarmSpace {
            owner,
            level: 1,
            capacity: 4,
            seed_count: 4,
            total_grow_power: 400,
            upgrade_start_time: 0,
            upgrade_target_level: 0,
            reserve: [0; 32],
        };
        
        assert!(validate_farm_space_capacity(&full_farm).is_err());
        
        let available_farm = FarmSpace {
            seed_count: 3,
            ..full_farm
        };
        assert!(validate_farm_space_capacity(&available_farm).is_ok());
        
        // Test farm space ownership
        assert!(validate_farm_space_ownership(&full_farm, owner).is_ok());
        assert!(validate_farm_space_ownership(&full_farm, other_user).is_err());
        
        // Test upgrade validation
        let max_level_farm = FarmSpace {
            level: 5,
            ..full_farm
        };
        assert!(validate_can_upgrade_farm_space(&max_level_farm).is_err());
        
        let upgrading_farm = FarmSpace {
            level: 2,
            upgrade_start_time: 1000000,
            ..full_farm
        };
        assert!(validate_can_upgrade_farm_space(&upgrading_farm).is_err());
        
        let upgradeable_farm = FarmSpace {
            level: 2,
            upgrade_start_time: 0,
            ..full_farm
        };
        assert!(validate_can_upgrade_farm_space(&upgradeable_farm).is_ok());
        
        // Test upgrade completion validation
        let current_time = 1000000i64;
        let upgrading_farm = FarmSpace {
            upgrade_start_time: current_time - UPGRADE_COOLDOWN + 100, // Still in cooldown
            ..full_farm
        };
        assert!(validate_upgrade_complete(&upgrading_farm, current_time).is_err());
        
        let completed_upgrade_farm = FarmSpace {
            upgrade_start_time: current_time - UPGRADE_COOLDOWN - 100, // Cooldown passed
            ..full_farm
        };
        assert!(validate_upgrade_complete(&completed_upgrade_farm, current_time).is_ok());
        
        // Test no upgrade in progress
        let no_upgrade_farm = FarmSpace {
            upgrade_start_time: 0,
            ..full_farm
        };
        assert!(validate_upgrade_complete(&no_upgrade_farm, current_time).is_err());
    }

    #[test]
    fn test_seed_validation_errors() {
        let owner = Pubkey::new_unique();
        let other_user = Pubkey::new_unique();
        let farm_space_key = Pubkey::new_unique();
        let other_farm_key = Pubkey::new_unique();
        
        // Test seed ownership
        let seed = Seed {
            owner,
            seed_type: SeedType::Seed1,
            grow_power: 100,
            is_planted: false,
            planted_farm_space: None,
            planted_at: 0,
            reserve: [0; 16],
        };
        
        assert!(validate_seed_ownership(&seed, owner).is_ok());
        assert!(validate_seed_ownership(&seed, other_user).is_err());
        
        // Test seed planting status
        assert!(validate_seed_not_planted(&seed).is_ok());
        assert!(validate_seed_is_planted(&seed).is_err());
        
        let planted_seed = Seed {
            is_planted: true,
            planted_farm_space: Some(farm_space_key),
            planted_at: 1000000,
            ..seed
        };
        
        assert!(validate_seed_not_planted(&planted_seed).is_err());
        assert!(validate_seed_is_planted(&planted_seed).is_ok());
        
        // Test seed farm space validation
        assert!(validate_seed_in_farm_space(&planted_seed, farm_space_key).is_ok());
        assert!(validate_seed_in_farm_space(&planted_seed, other_farm_key).is_err());
        assert!(validate_seed_in_farm_space(&seed, farm_space_key).is_err());
        
        // Test seed type validation
        for i in 0..9 {
            assert!(validate_seed_type(i).is_ok());
        }
        assert!(validate_seed_type(9).is_err());
        assert!(validate_seed_type(255).is_err());
    }

    #[test]
    fn test_seed_pack_validation_errors() {
        let owner = Pubkey::new_unique();
        let other_user = Pubkey::new_unique();
        
        let seed_pack = SeedPack {
            purchaser: owner,
            purchased_at: 1000000,
            cost_paid: SEED_PACK_COST,
            is_opened: false,
            entropy_sequence: 12345,
            user_entropy_seed: 67890,
            final_random_value: 0,
            pack_id: 1,
            reserve: [0; 16],
        };
        
        // Test seed pack ownership
        assert!(validate_seed_pack_ownership(&seed_pack, owner).is_ok());
        assert!(validate_seed_pack_ownership(&seed_pack, other_user).is_err());
        
        // Test seed pack not opened
        assert!(validate_seed_pack_not_opened(&seed_pack).is_ok());
        
        let opened_pack = SeedPack {
            is_opened: true,
            final_random_value: 99999,
            ..seed_pack
        };
        assert!(validate_seed_pack_not_opened(&opened_pack).is_err());
    }

    #[test]
    fn test_quantity_validation_errors() {
        // Test purchase quantity validation
        assert!(validate_purchase_quantity(1).is_ok());
        assert!(validate_purchase_quantity(50).is_ok());
        assert!(validate_purchase_quantity(100).is_ok());
        assert!(validate_purchase_quantity(0).is_err());
        assert!(validate_purchase_quantity(101).is_err());
        assert!(validate_purchase_quantity(255).is_err());
        
        // Test seed pack quantity validation
        assert!(validate_seed_pack_quantity(1).is_ok());
        assert!(validate_seed_pack_quantity(MAX_SEED_PACK_QUANTITY).is_ok());
        assert!(validate_seed_pack_quantity(0).is_err());
        assert!(validate_seed_pack_quantity(MAX_SEED_PACK_QUANTITY + 1).is_err());
    }

    #[test]
    fn test_time_validation_errors() {
        let current_time = 1000000i64;
        let past_time = current_time - 100;
        let future_time = current_time + 100;
        
        // Test claim interval validation
        assert!(validate_claim_interval(past_time, current_time).is_ok());
        assert!(validate_claim_interval(current_time - MIN_CLAIM_INTERVAL, current_time).is_ok());
        assert!(validate_claim_interval(current_time, current_time).is_err());
        assert!(validate_claim_interval(current_time - MIN_CLAIM_INTERVAL + 1, current_time).is_err());
        
        // Test timestamp validation
        assert!(validate_timestamp_not_future(past_time, current_time).is_ok());
        assert!(validate_timestamp_not_future(current_time, current_time).is_ok());
        assert!(validate_timestamp_not_future(future_time, current_time).is_err());
    }

    #[test]
    fn test_invite_validation_errors() {
        let user = Pubkey::new_unique();
        let referrer = Pubkey::new_unique();
        let protocol = Pubkey::new_unique();
        
        // Test referrer validation
        assert!(validate_referrer_not_self(user, referrer).is_ok());
        assert!(validate_referrer_not_self(user, user).is_err());
        assert!(validate_referrer_not_protocol(referrer, protocol).is_ok());
        assert!(validate_referrer_not_protocol(protocol, protocol).is_err());
        
        // Test invite code format validation
        let valid_code = [65, 66, 67, 68, 69, 70, 71, 72]; // "ABCDEFGH"
        let invalid_code1 = [65, 66, 67, 68, 69, 70, 71, 32]; // Space
        let invalid_code2 = [65, 66, 67, 68, 69, 70, 71, 0]; // Null
        let invalid_code3 = [65, 66, 67, 68, 69, 70, 71, 255]; // Invalid char
        
        assert!(validate_invite_code_format(&valid_code).is_ok());
        assert!(validate_invite_code_format(&invalid_code1).is_err());
        assert!(validate_invite_code_format(&invalid_code2).is_err());
        assert!(validate_invite_code_format(&invalid_code3).is_err());
        
        // Test invite limit validation
        let available_invite = InviteCode {
            creator: user,
            code: valid_code,
            invite_limit: 5,
            invites_used: 3,
            created_at: 1000000,
            reserve: [0; 16],
        };
        assert!(validate_invite_limit(&available_invite).is_ok());
        
        let exhausted_invite = InviteCode {
            invites_used: 5,
            ..available_invite
        };
        assert!(validate_invite_limit(&exhausted_invite).is_err());
    }

    #[test]
    fn test_composite_validation_errors() {
        let user = Pubkey::new_unique();
        let farm_space_key = Pubkey::new_unique();
        
        // Test farm space upgrade request validation
        let farm_space = FarmSpace {
            owner: user,
            level: 2,
            capacity: 8,
            seed_count: 4,
            total_grow_power: 400,
            upgrade_start_time: 0,
            upgrade_target_level: 0,
            reserve: [0; 32],
        };
        
        let sufficient_balance = 20_000 * 1_000_000; // More than L2→L3 cost
        let insufficient_balance = 10_000 * 1_000_000; // Less than L2→L3 cost
        
        assert!(validate_farm_space_upgrade_request(&farm_space, sufficient_balance, user).is_ok());
        assert!(validate_farm_space_upgrade_request(&farm_space, insufficient_balance, user).is_err());
        
        // Test seed planting request validation
        let seed = Seed {
            owner: user,
            seed_type: SeedType::Seed2,
            grow_power: 180,
            is_planted: false,
            planted_farm_space: None,
            planted_at: 0,
            reserve: [0; 16],
        };
        
        assert!(validate_seed_planting_request(&seed, &farm_space, user).is_ok());
        
        let planted_seed = Seed {
            is_planted: true,
            planted_farm_space: Some(farm_space_key),
            ..seed
        };
        assert!(validate_seed_planting_request(&planted_seed, &farm_space, user).is_err());
        
        let full_farm = FarmSpace {
            seed_count: 8, // At capacity
            ..farm_space
        };
        assert!(validate_seed_planting_request(&seed, &full_farm, user).is_err());
        
        // Test seed pack purchase request validation
        let valid_quantity = 5u8;
        let invalid_quantity = 0u8;
        let sufficient_tokens = 2_000_000_000u64; // Enough for many packs
        let insufficient_tokens = 100_000u64; // Not enough for even one pack
        let valid_entropy = 12345u64;
        let invalid_entropy = 0u64;
        
        assert!(validate_seed_pack_purchase_request(valid_quantity, sufficient_tokens, valid_entropy).is_ok());
        assert!(validate_seed_pack_purchase_request(invalid_quantity, sufficient_tokens, valid_entropy).is_err());
        assert!(validate_seed_pack_purchase_request(valid_quantity, insufficient_tokens, valid_entropy).is_err());
        assert!(validate_seed_pack_purchase_request(valid_quantity, sufficient_tokens, invalid_entropy).is_err());
        
        // Test referral setup validation
        let referrer = Pubkey::new_unique();
        let protocol = Pubkey::new_unique();
        
        assert!(validate_referral_setup(user, referrer, protocol).is_ok());
        assert!(validate_referral_setup(user, user, protocol).is_err()); // Self-referral
        assert!(validate_referral_setup(user, protocol, protocol).is_err()); // Protocol referral
    }

    #[test]
    fn test_entropy_validation_errors() {
        // Test entropy sequence validation
        let expected_sequence = 12345u64;
        let actual_sequence = 12345u64;
        let wrong_sequence = 54321u64;
        
        assert!(validate_entropy_sequence(expected_sequence, actual_sequence).is_ok());
        assert!(validate_entropy_sequence(expected_sequence, wrong_sequence).is_err());
        
        // Test user entropy seed validation
        assert!(validate_user_entropy_seed(1).is_ok());
        assert!(validate_user_entropy_seed(12345).is_ok());
        assert!(validate_user_entropy_seed(u64::MAX).is_ok());
        assert!(validate_user_entropy_seed(0).is_err());
    }

    #[test]
    fn test_admin_validation_errors() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        let zero_address = Pubkey::default();
        
        let config = Config {
            admin,
            treasury,
            protocol_referral_address: Pubkey::new_unique(),
            base_rate: DEFAULT_BASE_RATE,
            halving_interval: DEFAULT_HALVING_INTERVAL,
            next_halving_time: 0,
            seed_pack_cost: SEED_PACK_COST,
            farm_space_cost_sol: FARM_SPACE_COST_SOL,
            seed_pack_counter: 0,
            seed_counter: 0,
            trading_fee_percentage: 2,
            max_invite_limit: MAX_INVITE_LIMIT,
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        };
        
        // Test admin authority validation
        assert!(validate_admin_authority(&config, admin).is_ok());
        assert!(validate_admin_authority(&config, non_admin).is_err());
        
        // Test treasury address validation
        assert!(validate_treasury_address(treasury).is_ok());
        assert!(validate_treasury_address(zero_address).is_err());
    }

    #[test]
    fn test_storage_validation_errors() {
        let owner = Pubkey::new_unique();
        let other_user = Pubkey::new_unique();
        
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            reserve: [0; 32],
        };
        
        // Test storage ownership
        assert!(validate_seed_storage_ownership(&seed_storage, owner).is_ok());
        assert!(validate_seed_storage_ownership(&seed_storage, other_user).is_err());
        
        // Test storage capacity
        assert!(validate_seed_storage_capacity(&seed_storage).is_ok());
        
        // Fill up storage to capacity
        for i in 0..MAX_SEEDS_PER_USER {
            seed_storage.seed_ids.push(i as u64);
        }
        seed_storage.total_seeds = MAX_SEEDS_PER_USER as u32;
        
        assert!(validate_seed_storage_capacity(&seed_storage).is_err());
        
        // Reduce by one to make space
        seed_storage.seed_ids.pop();
        seed_storage.total_seeds -= 1;
        
        assert!(validate_seed_storage_capacity(&seed_storage).is_ok());
    }
}