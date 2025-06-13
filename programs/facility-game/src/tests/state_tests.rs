#[cfg(test)]
mod state_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;

    #[test]
    fn test_account_sizes() {
        // Test that account sizes are reasonable
        
        // Config account - should be small and fixed size
        assert!(Config::LEN > 0);
        assert!(Config::LEN < 1000); // Reasonable upper bound
        
        // UserState account
        assert!(UserState::LEN > 0);
        assert!(UserState::LEN < 500);
        
        // FarmSpace account
        assert!(FarmSpace::LEN > 0);
        assert!(FarmSpace::LEN < 300);
        
        // Seed account
        assert!(Seed::LEN > 0);
        assert!(Seed::LEN < 200);
        
        // SeedPack account
        assert!(SeedPack::LEN > 0);
        assert!(SeedPack::LEN < 300);
    }

    #[test]
    fn test_seed_type_enum() {
        // Test all seed types exist and have proper values
        let seed_types = [
            SeedType::Seed1,
            SeedType::Seed2,
            SeedType::Seed3,
            SeedType::Seed4,
            SeedType::Seed5,
            SeedType::Seed6,
            SeedType::Seed7,
            SeedType::Seed8,
            SeedType::Seed9,
        ];
        
        assert_eq!(seed_types.len(), 9);
        
        // Test grow power values are correct
        assert_eq!(SeedType::Seed1.get_grow_power(), 100);
        assert_eq!(SeedType::Seed9.get_grow_power(), 60000);
    }

    #[test]
    fn test_seed_type_from_random() {
        // Test seed type selection from random values
        
        // Test edge cases
        let seed_type_0 = SeedType::from_random(0);
        assert!(matches!(seed_type_0, SeedType::Seed1)); // Should be most common
        
        let seed_type_max = SeedType::from_random(u64::MAX);
        assert!(matches!(seed_type_max, SeedType::Seed9)); // Should be rarest
        
        // Test mid-range values
        let seed_type_mid = SeedType::from_random(u64::MAX / 2);
        // Should be one of the middle-tier seeds (Seed1-Seed5 are most likely)
    }

    #[test]
    fn test_config_defaults() {
        let admin = Pubkey::new_unique();
        let treasury = Pubkey::new_unique();
        
        let config = Config {
            base_rate: DEFAULT_BASE_RATE,
            halving_interval: DEFAULT_HALVING_INTERVAL,
            next_halving_time: DEFAULT_HALVING_INTERVAL,
            admin,
            treasury,
            seed_pack_cost: SEED_PACK_COST,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: FARM_SPACE_COST_SOL,
            max_invite_limit: 10,
            trading_fee_percentage: 5,
            protocol_referral_address: admin,
            total_supply_minted: 0,
            operator: admin,
            reserve: [0; 2],
        };
        
        assert_eq!(config.base_rate, 200);
        assert_eq!(config.halving_interval, 200);
        assert_eq!(config.seed_pack_cost, 300_000_000);
        assert_eq!(config.farm_space_cost_sol, 500_000_000);
    }

    #[test]
    fn test_user_state_defaults() {
        let owner = Pubkey::new_unique();
        
        let user_state = UserState {
            owner,
            total_grow_power: 0,
            last_harvest_time: 0,
            has_farm_space: false,
            referrer: None,
            pending_referral_rewards: 0,
            total_packs_purchased: 0,
            reserve: [0; 28],
        };
        
        assert_eq!(user_state.owner, owner);
        assert_eq!(user_state.total_grow_power, 0);
        assert!(!user_state.has_farm_space);
        assert!(user_state.referrer.is_none());
        assert_eq!(user_state.total_packs_purchased, 0);
    }

    #[test]
    fn test_farm_space_levels() {
        let owner = Pubkey::new_unique();
        
        // Test different farm space levels with new capacities
        for level in 1..=5 {
            let expected_capacity = match level {
                1 => 4,
                2 => 6,
                3 => 8,
                4 => 10,
                5 => 12,
                _ => 0,
            };
            
            let farm_space = FarmSpace {
                owner,
                level,
                capacity: expected_capacity,
                seed_count: 0,
                total_grow_power: 0,
                reserve: [0; 32],
            };
            
            assert_eq!(farm_space.level, level);
            assert_eq!(farm_space.capacity, expected_capacity);
            assert!(farm_space.seed_count <= farm_space.capacity);
        }
    }

    #[test]
    fn test_seed_storage_limits() {
        let owner = Pubkey::new_unique();
        
        let seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Test that seed_type_counts has 9 elements (for 9 seed types)
        assert_eq!(seed_storage.seed_type_counts.len(), 9);
        
        // Test that reserve has correct size
        assert_eq!(seed_storage.reserve.len(), 16);
    }

    #[test]
    fn test_seed_pack_validation() {
        let purchaser = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        
        let seed_pack = SeedPack {
            purchaser,
            purchased_at: 1640995200,
            cost_paid: SEED_PACK_COST * 5,
            vrf_fee_paid: 0,
            is_opened: false,
            vrf_sequence: 1,
            user_entropy_seed: 12345,
            final_random_value: 0,
            pack_id: 1,
            vrf_account,
            reserve: [0; 8],
        };
        
        // Validate cost calculation
        assert_eq!(seed_pack.cost_paid, SEED_PACK_COST * 5);
        
        // Validate initial state
        assert!(!seed_pack.is_opened);
        assert_eq!(seed_pack.final_random_value, 0); // Not set until opened
    }

    #[test]
    fn test_invite_code_structure() {
        let creator = Pubkey::new_unique();
        
        let invite_code = InviteCode {
            inviter: creator,
            invites_used: 0,
            invite_limit: 10,
            code_hash: [0; 32],
            salt: [0; 16],
            code_index: 0,
            created_at: 1640995200, // Mock timestamp
            is_active: true,
            created_as_operator: false,
            reserve: [0; 14],
        };
        
        assert_eq!(invite_code.inviter, creator);
        assert_eq!(invite_code.invites_used, 0);
        assert_eq!(invite_code.invite_limit, 10);
        assert!(invite_code.is_active);
        assert!(!invite_code.created_as_operator);
    }
}