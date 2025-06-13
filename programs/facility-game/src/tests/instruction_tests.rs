#[cfg(test)]
mod instruction_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::error::GameError;
    use crate::constants::*;

    // Mock data for testing
    fn create_mock_config() -> Config {
        Config {
            base_rate: 200,
            halving_interval: 200,
            next_halving_time: 200,
            admin: Pubkey::new_unique(),
            treasury: Pubkey::new_unique(),
            seed_pack_cost: SEED_PACK_COST,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: FARM_SPACE_COST_SOL,
            max_invite_limit: 10,
            trading_fee_percentage: 5,
            protocol_referral_address: Pubkey::new_unique(),
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        }
    }

    fn create_mock_user_state(owner: Pubkey) -> UserState {
        UserState {
            owner,
            total_grow_power: 0,
            last_harvest_time: 0,
            has_farm_space: false,
            referrer: None,
            pending_referral_rewards: 0,
            total_packs_purchased: 0,
            reserve: [0; 28],
        }
    }
    
    fn create_mock_user_state_with_packs(owner: Pubkey, packs: u32) -> UserState {
        UserState {
            owner,
            total_grow_power: 0,
            last_harvest_time: 0,
            has_farm_space: false,
            referrer: None,
            pending_referral_rewards: 0,
            total_packs_purchased: packs,
            reserve: [0; 28],
        }
    }

    fn create_mock_farm_space(owner: Pubkey) -> FarmSpace {
        FarmSpace {
            owner,
            level: 1,
            capacity: 4,
            seed_count: 0,
            total_grow_power: 0,
            reserve: [0; 32],
        }
    }

    #[test]
    fn test_config_initialization() {
        let config = create_mock_config();
        
        assert_eq!(config.base_rate, 200);
        assert_eq!(config.halving_interval, 200);
        assert_eq!(config.seed_pack_cost, SEED_PACK_COST);
        assert_eq!(config.farm_space_cost_sol, FARM_SPACE_COST_SOL);
        assert_eq!(config.total_supply_minted, 0);
    }

    #[test]
    fn test_user_state_initialization() {
        let owner = Pubkey::new_unique();
        let user_state = create_mock_user_state(owner);
        
        assert_eq!(user_state.owner, owner);
        assert_eq!(user_state.total_grow_power, 0);
        assert!(!user_state.has_farm_space);
        assert_eq!(user_state.pending_referral_rewards, 0);
        assert_eq!(user_state.total_packs_purchased, 0);
    }

    #[test]
    fn test_farm_space_initialization() {
        let owner = Pubkey::new_unique();
        let farm_space = create_mock_farm_space(owner);
        
        assert_eq!(farm_space.owner, owner);
        assert_eq!(farm_space.level, 1);
        assert_eq!(farm_space.capacity, 4);
        assert_eq!(farm_space.seed_count, 0);
        assert_eq!(farm_space.total_grow_power, 0);
    }

    #[test]
    fn test_seed_creation() {
        let owner = Pubkey::new_unique();
        let seed = Seed {
            owner,
            seed_type: SeedType::Seed1,
            grow_power: 100,
            is_planted: false,
            planted_farm_space: None,
            created_at: 1640995200, // Mock timestamp
            seed_id: 1,
            reserve: [0; 32],
        };
        
        assert_eq!(seed.owner, owner);
        assert_eq!(seed.seed_type, SeedType::Seed1);
        assert_eq!(seed.grow_power, 100);
        assert!(!seed.is_planted);
        assert_eq!(seed.seed_id, 1);
    }

    #[test]
    fn test_seed_pack_creation() {
        let purchaser = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        
        let seed_pack = SeedPack {
            purchaser,
            purchased_at: 1640995200,
            cost_paid: SEED_PACK_COST * 3,
            vrf_fee_paid: 0,
            is_opened: false,
            vrf_sequence: 1,
            user_entropy_seed: 12345,
            final_random_value: 0,
            pack_id: 1,
            vrf_account,
            reserve: [0; 8],
        };
        
        assert_eq!(seed_pack.purchaser, purchaser);
        assert_eq!(seed_pack.cost_paid, SEED_PACK_COST * 3);
        assert!(!seed_pack.is_opened);
        assert_eq!(seed_pack.pack_id, 1);
    }

    #[test]
    fn test_seed_storage_initialization() {
        let owner = Pubkey::new_unique();
        let seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        assert_eq!(seed_storage.owner, owner);
        assert_eq!(seed_storage.seed_ids.len(), 0);
        assert_eq!(seed_storage.total_seeds, 0);
        assert_eq!(seed_storage.seed_type_counts.iter().sum::<u16>(), 0);
    }

    #[test]
    fn test_reward_calculation_basic() {
        // Test basic reward calculation
        // Formula: (elapsed_time * grow_power * base_rate) / 1000
        let elapsed_time = 200; // 200 seconds (1 halving interval)
        let grow_power = 100;
        let base_rate = 200;
        
        let expected_reward = (elapsed_time * grow_power * base_rate) / 1000;
        assert_eq!(expected_reward, 4000); // 200 * 100 * 200 / 1000 = 4000
    }

    #[test]
    fn test_halving_calculation() {
        let initial_rate = 200u64;
        let halved_rate = initial_rate / 2;
        assert_eq!(halved_rate, 100);
        
        let double_halved = halved_rate / 2;
        assert_eq!(double_halved, 50);
    }

    #[test]
    fn test_supply_cap_validation() {
        let current_supply = 200_000_000_000_000u64; // 200M WEED
        let mint_amount = 30_000_000_000_000u64; // 30M WEED
        let new_supply = current_supply + mint_amount;
        
        assert!(new_supply <= TOTAL_WEED_SUPPLY); // Should be within 240M cap
        
        let large_mint = 50_000_000_000_000u64; // 50M WEED
        let overflow_supply = current_supply + large_mint;
        assert!(overflow_supply > TOTAL_WEED_SUPPLY); // Should exceed cap
    }
    
    #[test]
    fn test_auto_upgrade_logic() {
        let owner = Pubkey::new_unique();
        let mut user_state = create_mock_user_state(owner);
        let mut farm_space = create_mock_farm_space(owner);
        
        // Test initial state
        assert_eq!(user_state.total_packs_purchased, 0);
        assert_eq!(farm_space.level, 1);
        assert_eq!(farm_space.capacity, 4);
        
        // Test purchasing 30 packs (should trigger level 2)
        let upgrade_needed = user_state.increment_pack_purchases(30);
        assert!(upgrade_needed);
        assert_eq!(user_state.total_packs_purchased, 30);
        
        let upgraded = farm_space.auto_upgrade(user_state.total_packs_purchased);
        assert!(upgraded);
        assert_eq!(farm_space.level, 2);
        assert_eq!(farm_space.capacity, 6);
        
        // Test purchasing 70 more packs (total 100, should trigger level 3)
        let upgrade_needed = user_state.increment_pack_purchases(70);
        assert!(upgrade_needed);
        assert_eq!(user_state.total_packs_purchased, 100);
        
        let upgraded = farm_space.auto_upgrade(user_state.total_packs_purchased);
        assert!(upgraded);
        assert_eq!(farm_space.level, 3);
        assert_eq!(farm_space.capacity, 8);
        
        // Test purchasing 5 more packs (total 105, should NOT trigger upgrade)
        let upgrade_needed = user_state.increment_pack_purchases(5);
        assert!(!upgrade_needed);
        assert_eq!(user_state.total_packs_purchased, 105);
        
        let upgraded = farm_space.auto_upgrade(user_state.total_packs_purchased);
        assert!(!upgraded); // No upgrade should occur
        assert_eq!(farm_space.level, 3); // Still level 3
        assert_eq!(farm_space.capacity, 8); // Still capacity 8
    }
}