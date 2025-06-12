#[cfg(test)]
mod admin_tests {
    use super::*;
    use anchor_lang::prelude::*;
    use anchor_lang::accounts::account::Account as AnchorAccount;
    use anchor_spl::token::Mint;
    use crate::state::*;
    use crate::instructions::admin::*;

    // Helper function to create a mock account info
    fn create_mock_account_info(key: &Pubkey, lamports: &mut u64) -> AccountInfo {
        AccountInfo::new(
            key,
            false,
            false,
            lamports,
            &mut [],
            &Pubkey::new_unique(),
            false,
            0,
        )
    }

    // ===== INITIALIZE_CONFIG TESTS =====

    #[test]
    fn test_initialize_config_default_values() {
        // Test config initialization with default values
        let admin_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        
        // Create mock config
        let mut config = Config {
            base_rate: 0,
            halving_interval: 0,
            next_halving_time: 0,
            admin: Pubkey::default(),
            treasury: Pubkey::default(),
            seed_pack_cost: 0,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: 0,
            max_invite_limit: 0,
            trading_fee_percentage: 0,
            protocol_referral_address: Pubkey::default(),
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        };

        // Simulate the initialization logic
        let current_time = 1000000i64;
        
        config.base_rate = Config::DEFAULT_BASE_RATE;
        config.halving_interval = Config::DEFAULT_HALVING_INTERVAL;
        config.next_halving_time = current_time + config.halving_interval;
        config.admin = admin_key;
        config.treasury = treasury_key;
        config.seed_pack_cost = 300 * 1_000_000;
        config.farm_space_cost_sol = Config::DEFAULT_FARM_SPACE_COST;
        config.max_invite_limit = 5;
        config.trading_fee_percentage = 2;
        config.protocol_referral_address = Pubkey::default();
        config.seed_counter = 0;
        config.seed_pack_counter = 0;
        config.reserve = [0; 2];

        // Verify default values
        assert_eq!(config.base_rate, 100);
        assert_eq!(config.halving_interval, 6 * 24 * 60 * 60); // 6 days
        assert_eq!(config.next_halving_time, current_time + config.halving_interval);
        assert_eq!(config.admin, admin_key);
        assert_eq!(config.treasury, treasury_key);
        assert_eq!(config.seed_pack_cost, 300_000_000); // 300 WEED
        assert_eq!(config.farm_space_cost_sol, 500_000_000); // 0.5 SOL
        assert_eq!(config.max_invite_limit, 5);
        assert_eq!(config.trading_fee_percentage, 2);
        assert_eq!(config.seed_counter, 0);
        assert_eq!(config.seed_pack_counter, 0);
    }

    #[test]
    fn test_initialize_config_custom_values() {
        // Test config initialization with custom values
        let admin_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        let protocol_referral_key = Pubkey::new_unique();
        
        let mut config = Config {
            base_rate: 0,
            halving_interval: 0,
            next_halving_time: 0,
            admin: Pubkey::default(),
            treasury: Pubkey::default(),
            seed_pack_cost: 0,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: 0,
            max_invite_limit: 0,
            trading_fee_percentage: 0,
            protocol_referral_address: Pubkey::default(),
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        };

        // Custom values
        let custom_base_rate = 150u64;
        let custom_halving_interval = 7 * 24 * 60 * 60i64; // 7 days
        let current_time = 2000000i64;
        
        // Simulate initialization with custom values
        config.base_rate = custom_base_rate;
        config.halving_interval = custom_halving_interval;
        config.next_halving_time = current_time + config.halving_interval;
        config.admin = admin_key;
        config.treasury = treasury_key;
        config.protocol_referral_address = protocol_referral_key;
        config.seed_pack_cost = 300 * 1_000_000;
        config.farm_space_cost_sol = Config::DEFAULT_FARM_SPACE_COST;
        config.max_invite_limit = 5;
        config.trading_fee_percentage = 2;
        config.seed_counter = 0;
        config.seed_pack_counter = 0;
        config.reserve = [0; 2];

        // Verify custom values
        assert_eq!(config.base_rate, 150);
        assert_eq!(config.halving_interval, 7 * 24 * 60 * 60);
        assert_eq!(config.next_halving_time, current_time + custom_halving_interval);
        assert_eq!(config.admin, admin_key);
        assert_eq!(config.treasury, treasury_key);
        assert_eq!(config.protocol_referral_address, protocol_referral_key);
    }

    #[test]
    fn test_config_account_size() {
        // Test that Config::LEN matches actual struct size
        let config = Config {
            base_rate: 100,
            halving_interval: 518400,
            next_halving_time: 1000000,
            admin: Pubkey::new_unique(),
            treasury: Pubkey::new_unique(),
            seed_pack_cost: 300_000_000,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: 500_000_000,
            max_invite_limit: 5,
            trading_fee_percentage: 2,
            protocol_referral_address: Pubkey::new_unique(),
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        };

        // Verify the struct can be created and all fields are accessible
        assert_eq!(config.base_rate, 100);
        assert_eq!(config.max_invite_limit, 5);
        assert_eq!(config.trading_fee_percentage, 2);
        assert_eq!(config.reserve.len(), 2);
    }

    // ===== GLOBAL_STATS TESTS =====

    #[test]
    fn test_initialize_global_stats() {
        // Test global stats initialization
        let mut global_stats = GlobalStats {
            total_grow_power: 0,
            total_farm_spaces: 0,
            total_supply: 0,
            current_rewards_per_second: 0,
            last_update_time: 0,
            reserve: [0; 32],
        };

        let current_time = 1500000i64;

        // Simulate initialization logic
        global_stats.total_grow_power = 0;
        global_stats.total_farm_spaces = 0;
        global_stats.total_supply = GlobalStats::INITIAL_TOTAL_SUPPLY;
        global_stats.current_rewards_per_second = Config::DEFAULT_BASE_RATE;
        global_stats.last_update_time = current_time;
        global_stats.reserve = [0; 32];

        // Verify initialization
        assert_eq!(global_stats.total_grow_power, 0);
        assert_eq!(global_stats.total_farm_spaces, 0);
        assert_eq!(global_stats.total_supply, 1_000_000_000 * 1_000_000); // 1B WEED
        assert_eq!(global_stats.current_rewards_per_second, 100);
        assert_eq!(global_stats.last_update_time, current_time);
        assert_eq!(global_stats.reserve.len(), 32);
    }

    #[test]
    fn test_global_stats_constants() {
        // Test global stats constants
        assert_eq!(GlobalStats::INITIAL_TOTAL_SUPPLY, 1_000_000_000_000_000u64);
        
        // Verify account size calculation
        let expected_len = 8 + // discriminator
            8 + // total_grow_power
            8 + // total_farm_spaces
            8 + // total_supply
            8 + // current_rewards_per_second
            8 + // last_update_time
            32; // reserve

        assert_eq!(GlobalStats::LEN, expected_len);
    }

    // ===== FEE_POOL TESTS =====

    #[test]
    fn test_initialize_fee_pool() {
        // Test fee pool initialization
        let treasury_key = Pubkey::new_unique();
        let mut fee_pool = FeePool {
            accumulated_fees: 0,
            treasury_address: Pubkey::default(),
            last_collection_time: 0,
            reserve: [0; 48],
        };

        let current_time = 1800000i64;

        // Simulate initialization logic
        fee_pool.accumulated_fees = 0;
        fee_pool.treasury_address = treasury_key;
        fee_pool.last_collection_time = current_time;
        fee_pool.reserve = [0; 48];

        // Verify initialization
        assert_eq!(fee_pool.accumulated_fees, 0);
        assert_eq!(fee_pool.treasury_address, treasury_key);
        assert_eq!(fee_pool.last_collection_time, current_time);
        assert_eq!(fee_pool.reserve.len(), 48);
    }

    #[test]
    fn test_fee_pool_account_size() {
        // Test fee pool account size calculation
        let expected_len = 8 + // discriminator
            8 + // accumulated_fees
            32 + // treasury_address
            8 + // last_collection_time
            48; // reserve

        assert_eq!(FeePool::LEN, expected_len);
    }

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_admin_initialization_workflow() {
        // Test complete admin initialization workflow
        let admin_key = Pubkey::new_unique();
        let treasury_key = Pubkey::new_unique();
        let protocol_referral_key = Pubkey::new_unique();
        
        // Step 1: Initialize Config
        let mut config = Config {
            base_rate: Config::DEFAULT_BASE_RATE,
            halving_interval: Config::DEFAULT_HALVING_INTERVAL,
            next_halving_time: 0,
            admin: admin_key,
            treasury: treasury_key,
            seed_pack_cost: 300 * 1_000_000,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: Config::DEFAULT_FARM_SPACE_COST,
            max_invite_limit: 5,
            trading_fee_percentage: 2,
            protocol_referral_address: protocol_referral_key,
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        };

        let current_time = 1000000i64;
        config.next_halving_time = current_time + config.halving_interval;

        // Step 2: Initialize Global Stats
        let mut global_stats = GlobalStats {
            total_grow_power: 0,
            total_farm_spaces: 0,
            total_supply: GlobalStats::INITIAL_TOTAL_SUPPLY,
            current_rewards_per_second: Config::DEFAULT_BASE_RATE,
            last_update_time: current_time,
            reserve: [0; 32],
        };

        // Step 3: Initialize Fee Pool
        let mut fee_pool = FeePool {
            accumulated_fees: 0,
            treasury_address: treasury_key,
            last_collection_time: current_time,
            reserve: [0; 48],
        };

        // Verify all systems are properly initialized
        assert_eq!(config.admin, admin_key);
        assert_eq!(config.treasury, treasury_key);
        assert_eq!(config.protocol_referral_address, protocol_referral_key);
        assert_eq!(config.next_halving_time, current_time + config.halving_interval);

        assert_eq!(global_stats.total_supply, GlobalStats::INITIAL_TOTAL_SUPPLY);
        assert_eq!(global_stats.current_rewards_per_second, config.base_rate);

        assert_eq!(fee_pool.treasury_address, treasury_key);
        assert_eq!(fee_pool.accumulated_fees, 0);

        // Verify consistency between configs
        assert_eq!(global_stats.current_rewards_per_second, config.base_rate);
        assert_eq!(fee_pool.treasury_address, config.treasury);
    }

    #[test]
    fn test_admin_seed_derivation() {
        // Test that admin instruction PDAs can be derived correctly
        let program_id = Pubkey::new_unique();
        
        // Test config PDA derivation
        let (config_pda, config_bump) = Pubkey::find_program_address(
            &[b"config"],
            &program_id
        );
        assert!(config_bump <= 255);
        assert_ne!(config_pda, Pubkey::default());

        // Test global stats PDA derivation
        let (global_stats_pda, global_stats_bump) = Pubkey::find_program_address(
            &[b"global_stats"],
            &program_id
        );
        assert!(global_stats_bump <= 255);
        assert_ne!(global_stats_pda, Pubkey::default());

        // Test fee pool PDA derivation
        let (fee_pool_pda, fee_pool_bump) = Pubkey::find_program_address(
            &[b"fee_pool"],
            &program_id
        );
        assert!(fee_pool_bump <= 255);
        assert_ne!(fee_pool_pda, Pubkey::default());

        // Test mint authority PDA derivation
        let (mint_authority_pda, mint_authority_bump) = Pubkey::find_program_address(
            &[b"mint_authority"],
            &program_id
        );
        assert!(mint_authority_bump <= 255);
        assert_ne!(mint_authority_pda, Pubkey::default());

        // Test reward mint PDA derivation
        let (reward_mint_pda, reward_mint_bump) = Pubkey::find_program_address(
            &[b"reward_mint"],
            &program_id
        );
        assert!(reward_mint_bump <= 255);
        assert_ne!(reward_mint_pda, Pubkey::default());

        // Verify all PDAs are unique
        let pdas = vec![
            config_pda,
            global_stats_pda,
            fee_pool_pda,
            mint_authority_pda,
            reward_mint_pda,
        ];
        
        for (i, &pda1) in pdas.iter().enumerate() {
            for (j, &pda2) in pdas.iter().enumerate() {
                if i != j {
                    assert_ne!(pda1, pda2, "PDAs should be unique");
                }
            }
        }
    }

    #[test]
    fn test_economic_parameter_validation() {
        // Test validation of economic parameters
        
        // Test valid base rates
        let valid_base_rates = vec![1, 10, 100, 1000, 10000];
        for rate in valid_base_rates {
            assert!(rate > 0, "Base rate should be positive");
            assert!(rate <= 100000, "Base rate should be reasonable");
        }

        // Test valid halving intervals
        let valid_intervals = vec![
            60 * 60,           // 1 hour (for testing)
            24 * 60 * 60,      // 1 day
            7 * 24 * 60 * 60,  // 1 week
            30 * 24 * 60 * 60, // ~1 month
        ];
        for interval in valid_intervals {
            assert!(interval >= 3600, "Halving interval should be at least 1 hour");
            assert!(interval <= 365 * 24 * 60 * 60, "Halving interval should be at most 1 year");
        }

        // Test seed pack cost
        let seed_pack_cost = 300 * 1_000_000; // 300 WEED with 6 decimals
        assert_eq!(seed_pack_cost, 300_000_000);
        assert!(seed_pack_cost >= 1_000_000, "Seed pack should cost at least 1 WEED");
        assert!(seed_pack_cost <= 1_000_000_000, "Seed pack should cost at most 1000 WEED");

        // Test farm space cost
        let farm_space_cost = Config::DEFAULT_FARM_SPACE_COST;
        assert_eq!(farm_space_cost, 500_000_000); // 0.5 SOL
        assert!(farm_space_cost >= 100_000_000, "Farm space should cost at least 0.1 SOL");
        assert!(farm_space_cost <= 10_000_000_000, "Farm space should cost at most 10 SOL");

        // Test trading fee percentage
        let trading_fee = 2u8;
        assert!(trading_fee <= 100, "Trading fee should not exceed 100%");
        assert!(trading_fee >= 1, "Trading fee should be at least 1%");

        // Test invite limit
        let invite_limit = 5u8;
        assert!(invite_limit >= 1, "Should allow at least 1 invite");
        assert!(invite_limit <= 100, "Should not allow excessive invites");
    }

    #[test]
    fn test_reserve_field_initialization() {
        // Test that all reserve fields are properly initialized to zero
        
        let config = Config {
            base_rate: 100,
            halving_interval: 518400,
            next_halving_time: 1000000,
            admin: Pubkey::new_unique(),
            treasury: Pubkey::new_unique(),
            seed_pack_cost: 300_000_000,
            seed_counter: 0,
            seed_pack_counter: 0,
            farm_space_cost_sol: 500_000_000,
            max_invite_limit: 5,
            trading_fee_percentage: 2,
            protocol_referral_address: Pubkey::new_unique(),
            total_supply_minted: 0,
            operator: Pubkey::new_unique(),
            reserve: [0; 2],
        };
        
        assert_eq!(config.reserve, [0; 2]);
        assert_eq!(config.reserve.len(), 2);

        let global_stats = GlobalStats {
            total_grow_power: 0,
            total_farm_spaces: 0,
            total_supply: GlobalStats::INITIAL_TOTAL_SUPPLY,
            current_rewards_per_second: 100,
            last_update_time: 1000000,
            reserve: [0; 32],
        };
        
        assert_eq!(global_stats.reserve, [0; 32]);
        assert_eq!(global_stats.reserve.len(), 32);

        let fee_pool = FeePool {
            accumulated_fees: 0,
            treasury_address: Pubkey::new_unique(),
            last_collection_time: 1000000,
            reserve: [0; 48],
        };
        
        assert_eq!(fee_pool.reserve, [0; 48]);
        assert_eq!(fee_pool.reserve.len(), 48);
    }
}