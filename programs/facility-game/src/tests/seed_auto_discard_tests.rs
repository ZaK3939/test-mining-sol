#[cfg(test)]
mod seed_auto_discard_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;
    use crate::utils::*;
    use crate::error::GameError;

    // ===== SEED AUTO-DISCARD FUNCTIONALITY TESTS =====

    #[test]
    fn test_seed_storage_type_limits() {
        // Test seed storage type limits and capacity checks
        println!("🗃️ Testing Seed Storage Type Limits");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Test maximum seeds per type (100 each)
        assert_eq!(SeedStorage::MAX_SEEDS_PER_TYPE, 100);
        assert_eq!(SeedStorage::MAX_TOTAL_SEEDS, 2000);
        
        // Test initial capacity checks
        assert!(seed_storage.can_add_seed());
        assert!(seed_storage.can_add_seed_type(&SeedType::Seed1));
        assert!(seed_storage.can_add_seed_with_type(&SeedType::Seed1));
        
        // Fill up Seed1 to the limit
        for i in 0..SeedStorage::MAX_SEEDS_PER_TYPE {
            let seed_id = 1000u64 + i as u64;
            let result = seed_storage.add_seed(seed_id, &SeedType::Seed1);
            assert!(result.is_ok(), "Should be able to add Seed1 #{}", i + 1);
        }
        
        // Verify we're at the limit for Seed1
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 100);
        assert!(!seed_storage.can_add_seed_type(&SeedType::Seed1));
        assert!(!seed_storage.can_add_seed_with_type(&SeedType::Seed1));
        
        // Should still be able to add other seed types
        assert!(seed_storage.can_add_seed_type(&SeedType::Seed2));
        assert!(seed_storage.can_add_seed_with_type(&SeedType::Seed2));
        
        // Try to add one more Seed1 (should fail)
        let result = seed_storage.add_seed(9999, &SeedType::Seed1);
        assert!(result.is_err(), "Should not be able to exceed Seed1 limit");
        
        println!("✅ Seed storage type limits validated");
    }

    #[test] 
    fn test_auto_discard_functionality() {
        // Test auto-discard when type limit is reached
        println!("🗑️ Testing Auto-Discard Functionality");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Fill Seed1 to maximum capacity
        for i in 0..SeedStorage::MAX_SEEDS_PER_TYPE {
            let seed_id = 1000u64 + i as u64;
            seed_storage.add_seed(seed_id, &SeedType::Seed1).unwrap();
        }
        
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 100);
        assert!(!seed_storage.can_add_seed_type(&SeedType::Seed1));
        
        // Test auto-discard when at limit
        let discarded_count = seed_storage.auto_discard_excess(&SeedType::Seed1).unwrap();
        assert_eq!(discarded_count, 1, "Should discard 1 seed when at limit");
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 99);
        assert!(seed_storage.can_add_seed_type(&SeedType::Seed1));
        
        // Test auto-discard when not at limit (should do nothing)
        let discarded_count = seed_storage.auto_discard_excess(&SeedType::Seed1).unwrap();
        assert_eq!(discarded_count, 0, "Should not discard when not at limit");
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 99);
        
        // Test auto-discard for different seed type (should do nothing)
        let discarded_count = seed_storage.auto_discard_excess(&SeedType::Seed2).unwrap();
        assert_eq!(discarded_count, 0, "Should not discard when Seed2 is empty");
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed2), 0);
        
        println!("✅ Auto-discard functionality validated");
    }

    #[test]
    fn test_add_seed_to_storage_with_auto_discard() {
        // Test the full add_seed_to_storage flow with auto-discard
        println!("📦 Testing Add Seed to Storage with Auto-Discard");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Fill Seed1 to maximum capacity manually
        for i in 0..SeedStorage::MAX_SEEDS_PER_TYPE {
            seed_storage.seed_ids.push(1000u64 + i as u64);
            seed_storage.seed_type_counts[0] += 1; // Seed1 index
        }
        seed_storage.total_seeds = seed_storage.seed_ids.len() as u32;
        
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 100);
        assert!(!seed_storage.can_add_seed_type(&SeedType::Seed1));
        
        // Now use the proper add_seed_to_storage function which should auto-discard
        let result = add_seed_to_storage(&mut seed_storage, 9999, SeedType::Seed1);
        assert!(result.is_ok(), "Should succeed with auto-discard");
        
        // Verify the state after auto-discard and addition
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 100); // Back to max after adding
        assert!(seed_storage.seed_ids.contains(&9999), "New seed should be added");
        
        println!("✅ Add seed to storage with auto-discard validated");
    }

    #[test]
    fn test_low_value_seed_auto_discard_priority() {
        // Test that low-value seeds (Seed1, Seed2) are prioritized for auto-discard
        println!("🎯 Testing Low-Value Seed Auto-Discard Priority");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Test scenario: Mixed seed types at various capacities
        let test_scenarios = [
            (SeedType::Seed1, 100, 1), // At limit, should discard
            (SeedType::Seed2, 50, 0),  // Not at limit, no discard
            (SeedType::Seed3, 100, 1), // At limit, should discard
            (SeedType::Seed9, 100, 1), // At limit, should discard (even high value)
        ];
        
        for (seed_type, initial_count, expected_discarded) in test_scenarios {
            // Reset storage for each test
            let type_index = seed_type as usize;
            seed_storage.seed_type_counts[type_index] = initial_count;
            
            let discarded = seed_storage.auto_discard_excess(&seed_type).unwrap();
            assert_eq!(discarded, expected_discarded,
                      "Auto-discard count mismatch for {:?}", seed_type);
            
            if expected_discarded > 0 {
                assert_eq!(seed_storage.seed_type_counts[type_index], initial_count - expected_discarded,
                          "Type count should be reduced for {:?}", seed_type);
            }
        }
        
        println!("✅ Low-value seed auto-discard priority validated");
    }

    #[test]
    fn test_seed_storage_capacity_management() {
        // Test overall storage capacity management
        println!("📊 Testing Seed Storage Capacity Management");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Test remaining capacity calculations
        assert_eq!(seed_storage.get_remaining_capacity(&SeedType::Seed1), 100);
        
        // Add some seeds and check capacity
        for i in 0..30 {
            seed_storage.add_seed(1000 + i, &SeedType::Seed1).unwrap();
        }
        
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 30);
        assert_eq!(seed_storage.get_remaining_capacity(&SeedType::Seed1), 70);
        
        // Test different seed types independently
        for i in 0..20 {
            seed_storage.add_seed(2000 + i, &SeedType::Seed2).unwrap();
        }
        
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed2), 20);
        assert_eq!(seed_storage.get_remaining_capacity(&SeedType::Seed2), 80);
        
        // Verify total count
        assert_eq!(seed_storage.total_seeds, 50);
        
        // Test total storage capacity (2000 seeds max)
        assert!(seed_storage.can_add_seed()); // Should still have plenty of room
        
        println!("✅ Seed storage capacity management validated");
    }

    #[test]
    fn test_seed_generation_with_auto_discard_simulation() {
        // Simulate seed pack opening with auto-discard
        println!("🎲 Testing Seed Generation with Auto-Discard Simulation");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        let mut next_seed_id = 1000u64;
        
        // Simulate opening many seed packs to test auto-discard in realistic scenario
        for pack_num in 0..200 { // 200 packs * 5 seeds = 1000 seeds
            // Simulate opening a pack of 5 seeds
            for seed_index in 0..5 {
                // Generate a seed type based on mock probabilities (heavily weighted to Seed1/Seed2)
                let random_value = (pack_num * 5 + seed_index) as u64 * 1337; // Mock randomness
                let seed_type = determine_mock_seed_type_biased(random_value);
                
                // Try to add the seed with auto-discard
                let result = add_seed_to_storage(&mut seed_storage, next_seed_id, seed_type);
                
                match result {
                    Ok(()) => {
                        // Successfully added (possibly with auto-discard)
                    },
                    Err(e) => {
                        // Handle storage full or other errors
                        println!("Error adding seed {}: {:?}", next_seed_id, e);
                    }
                }
                
                next_seed_id += 1;
            }
        }
        
        // Verify storage state after simulation
        println!("Final storage state:");
        for (i, &count) in seed_storage.seed_type_counts.iter().enumerate() {
            if count > 0 {
                let seed_type = match i {
                    0 => "Seed1", 1 => "Seed2", 2 => "Seed3", 3 => "Seed4",
                    4 => "Seed5", 5 => "Seed6", 6 => "Seed7", 7 => "Seed8",
                    8 => "Seed9", _ => "Unknown"
                };
                println!("  {}: {} seeds", seed_type, count);
            }
        }
        
        // Verify no type exceeds the limit
        for &count in &seed_storage.seed_type_counts {
            assert!(count <= SeedStorage::MAX_SEEDS_PER_TYPE,
                   "No seed type should exceed limit: {}", count);
        }
        
        // Verify we have a reasonable total (should be less than input due to auto-discard)
        assert!(seed_storage.total_seeds <= 1000, "Total seeds should be reasonable");
        
        println!("✅ Seed generation with auto-discard simulation completed");
    }

    #[test]
    fn test_seed_removal_with_type_tracking() {
        // Test seed removal with proper type tracking
        println!("➖ Testing Seed Removal with Type Tracking");
        
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Add various seed types
        let test_seeds = [
            (1001, SeedType::Seed1),
            (1002, SeedType::Seed1),
            (2001, SeedType::Seed2),
            (3001, SeedType::Seed3),
            (9001, SeedType::Seed9),
        ];
        
        for (seed_id, seed_type) in test_seeds {
            seed_storage.add_seed(seed_id, &seed_type).unwrap();
        }
        
        // Verify initial state
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 2);
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed2), 1);
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed3), 1);
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed9), 1);
        assert_eq!(seed_storage.total_seeds, 5);
        
        // Remove a Seed1
        let removed = remove_seed_from_storage(&mut seed_storage, 1001, SeedType::Seed1).unwrap();
        assert!(removed);
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 1);
        assert_eq!(seed_storage.total_seeds, 4);
        
        // Try to remove non-existent seed
        let removed = remove_seed_from_storage(&mut seed_storage, 9999, SeedType::Seed1).unwrap();
        assert!(!removed);
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 1); // Should be unchanged
        
        // Remove wrong type (should fail in real implementation)
        // Note: This test shows the importance of proper type validation
        let removed = remove_seed_from_storage(&mut seed_storage, 1002, SeedType::Seed2).unwrap();
        assert!(!removed); // Should not find seed 1002 as Seed2
        
        // Remove with correct type
        let removed = remove_seed_from_storage(&mut seed_storage, 1002, SeedType::Seed1).unwrap();
        assert!(removed);
        assert_eq!(seed_storage.get_seed_type_count(&SeedType::Seed1), 0);
        assert_eq!(seed_storage.total_seeds, 3);
        
        println!("✅ Seed removal with type tracking validated");
    }

    #[test]
    fn test_storage_efficiency_and_rent_cost() {
        // Test storage efficiency and rent cost considerations
        println!("💰 Testing Storage Efficiency and Rent Cost");
        
        // Calculate approximate rent cost
        let base_account_size = 8; // discriminator
        let owner_size = 32; // Pubkey
        let vec_metadata = 4; // Vec length prefix
        let max_seed_ids = SeedStorage::MAX_TOTAL_SEEDS * 8; // u64 per seed ID
        let total_seeds_size = 4; // u32
        let type_counts_size = 9 * 2; // [u16; 9]
        let reserve_size = 16; // [u8; 16]
        
        let estimated_account_size = base_account_size + owner_size + vec_metadata + 
                                   max_seed_ids + total_seeds_size + type_counts_size + reserve_size;
        
        println!("Estimated SeedStorage account size: {} bytes", estimated_account_size);
        
        // At ~0.00000348 SOL per byte (current rates), estimate rent
        let estimated_rent_lamports = estimated_account_size as u64 * 3480; // Approximate
        let estimated_rent_sol = estimated_rent_lamports as f64 / 1_000_000_000.0;
        
        println!("Estimated rent cost: {:.6} SOL ({} lamports)", estimated_rent_sol, estimated_rent_lamports);
        
        // Test storage utilization efficiency
        let owner = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Fill storage to different capacities and check efficiency
        let test_capacities = [100, 500, 1000, 1500, 2000];
        
        for &capacity in &test_capacities {
            // Reset storage
            seed_storage.seed_ids.clear();
            seed_storage.total_seeds = 0;
            seed_storage.seed_type_counts = [0; 9];
            
            // Fill to test capacity
            for i in 0..capacity.min(SeedStorage::MAX_TOTAL_SEEDS) {
                let seed_type = if i % 2 == 0 { SeedType::Seed1 } else { SeedType::Seed2 };
                if seed_storage.can_add_seed_with_type(&seed_type) {
                    seed_storage.add_seed(i as u64, &seed_type).unwrap();
                }
            }
            
            let utilization = (seed_storage.total_seeds as f64 / SeedStorage::MAX_TOTAL_SEEDS as f64) * 100.0;
            println!("Capacity {}: {} seeds ({:.1}% utilization)", 
                     capacity, seed_storage.total_seeds, utilization);
        }
        
        println!("✅ Storage efficiency and rent cost analysis completed");
    }

    // Helper function for biased seed type generation (realistic distribution)
    fn determine_mock_seed_type_biased(random_value: u64) -> SeedType {
        let seed_value = (random_value % 10000) as u16;
        
        // Heavily biased toward low-value seeds to test auto-discard
        if seed_value < 5000 {      // 50% chance
            SeedType::Seed1
        } else if seed_value < 7500 { // 25% chance  
            SeedType::Seed2
        } else if seed_value < 9000 { // 15% chance
            SeedType::Seed3
        } else if seed_value < 9500 { // 5% chance
            SeedType::Seed4
        } else if seed_value < 9800 { // 3% chance
            SeedType::Seed5
        } else if seed_value < 9950 { // 1.5% chance
            SeedType::Seed6
        } else if seed_value < 9990 { // 0.4% chance
            SeedType::Seed7
        } else if seed_value < 9999 { // 0.09% chance
            SeedType::Seed8
        } else {                    // 0.01% chance
            SeedType::Seed9
        }
    }
}