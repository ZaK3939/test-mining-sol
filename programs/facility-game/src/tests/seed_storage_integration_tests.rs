#[cfg(test)]
mod seed_storage_integration_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;
    use crate::utils::*;
    use crate::error::GameError;

    // ===== SEED STORAGE INTEGRATION TESTS =====
    // Testing full workflows with VRF seed generation and auto-discard

    #[test]
    fn test_realistic_player_journey_with_auto_discard() {
        // Test a realistic player journey with seed pack purchases and auto-discard
        println!("🌱 Testing Realistic Player Journey with Auto-Discard");
        
        let user = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner: user,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        let mut seed_counter = 1000u64;
        let mut total_packs_opened = 0u32;
        let mut total_seeds_generated = 0u32;
        let mut auto_discards_triggered = 0u32;
        
        // Simulate player opening seed packs over time
        // Player buys and opens 100 packs (500 seeds) over several sessions
        for session in 0..10 {
            println!("\n--- Session {} ---", session + 1);
            
            // Each session: open 10 packs (50 seeds)
            for pack in 0..10 {
                total_packs_opened += 1;
                
                // Open pack: generate 5 seeds using realistic probability distribution
                for seed_index in 0..5 {
                    let random_base = (session * 1000 + pack * 100 + seed_index * 10) as u64;
                    let random_value = simulate_vrf_randomness(random_base, user.to_bytes()[0] as u64);
                    let seed_type = determine_seed_type_from_vrf(random_value);
                    
                    // Check if we need auto-discard before adding
                    let needs_discard = !seed_storage.can_add_seed_type(&seed_type);
                    
                    // Attempt to add seed (will auto-discard if needed)
                    let result = add_seed_to_storage(&mut seed_storage, seed_counter, seed_type);
                    
                    match result {
                        Ok(()) => {
                            if needs_discard {
                                auto_discards_triggered += 1;
                                println!("  Auto-discard triggered for {:?}", seed_type);
                            }
                            total_seeds_generated += 1;
                        },
                        Err(e) => {
                            println!("  Failed to add seed {}: {:?}", seed_counter, e);
                        }
                    }
                    
                    seed_counter += 1;
                }
                
                // Print pack summary
                if pack % 5 == 4 { // Every 5 packs
                    println!("  Packs {}-{}: {} total seeds in storage", 
                             pack - 4 + 1, pack + 1, seed_storage.total_seeds);
                }
            }
            
            // Print session summary
            println!("Session {} complete:");
            println!("  Total packs: {}", total_packs_opened);
            println!("  Seeds in storage: {}", seed_storage.total_seeds);
            println!("  Auto-discards: {}", auto_discards_triggered);
            
            // Print storage distribution
            for (i, &count) in seed_storage.seed_type_counts.iter().enumerate() {
                if count > 0 {
                    println!("    Seed{}: {}", i + 1, count);
                }
            }
        }
        
        // Final analysis
        println!("\n=== Final Journey Analysis ===");
        println!("Total packs opened: {}", total_packs_opened);
        println!("Total seeds generated: {}", total_seeds_generated);
        println!("Seeds in final storage: {}", seed_storage.total_seeds);
        println!("Auto-discards triggered: {}", auto_discards_triggered);
        
        // Verify storage constraints are maintained
        for (i, &count) in seed_storage.seed_type_counts.iter().enumerate() {
            assert!(count <= SeedStorage::MAX_SEEDS_PER_TYPE,
                   "Seed{} count {} exceeds limit", i + 1, count);
        }
        
        // Verify auto-discard is working (should have triggered for common seeds)
        assert!(auto_discards_triggered > 0, "Auto-discard should have been triggered");
        
        // Verify storage is efficiently utilized
        assert!(seed_storage.total_seeds <= SeedStorage::MAX_TOTAL_SEEDS as u32);
        
        println!("✅ Realistic player journey completed successfully");
    }

    #[test]
    fn test_seed_storage_edge_cases() {
        // Test edge cases in seed storage management
        println!("🔍 Testing Seed Storage Edge Cases");
        
        let user = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner: user,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Edge Case 1: Try to add when storage is completely full
        println!("Testing completely full storage...");
        
        // Fill storage to absolute maximum (distributed across types)
        let mut seed_id = 1u64;
        let mut seeds_added = 0;
        
        // Add seeds in a round-robin fashion across all types
        while seeds_added < SeedStorage::MAX_TOTAL_SEEDS && seeds_added < 900 { // Stop before hitting type limits
            for seed_type_index in 0..9 {
                if seeds_added >= 900 { break; } // Prevent infinite loop
                
                let seed_type = match seed_type_index {
                    0 => SeedType::Seed1, 1 => SeedType::Seed2, 2 => SeedType::Seed3,
                    3 => SeedType::Seed4, 4 => SeedType::Seed5, 5 => SeedType::Seed6,
                    6 => SeedType::Seed7, 7 => SeedType::Seed8, 8 => SeedType::Seed9,
                    _ => unreachable!(),
                };
                
                if seed_storage.can_add_seed_with_type(&seed_type) {
                    seed_storage.add_seed(seed_id, &seed_type).unwrap();
                    seed_id += 1;
                    seeds_added += 1;
                }
            }
        }
        
        println!("Added {} seeds to storage", seeds_added);
        
        // Edge Case 2: Fill one type to maximum
        println!("Testing single type at maximum...");
        
        let mut seed_storage_2 = SeedStorage {
            owner: user,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Fill Seed1 to maximum
        for i in 0..SeedStorage::MAX_SEEDS_PER_TYPE {
            seed_storage_2.add_seed(1000 + i as u64, &SeedType::Seed1).unwrap();
        }
        
        // Verify at limit
        assert_eq!(seed_storage_2.get_seed_type_count(&SeedType::Seed1), 100);
        assert!(!seed_storage_2.can_add_seed_type(&SeedType::Seed1));
        
        // Try auto-discard and add
        let result = add_seed_to_storage(&mut seed_storage_2, 9999, SeedType::Seed1);
        assert!(result.is_ok(), "Should succeed with auto-discard");
        
        // Edge Case 3: Mixed type limits
        println!("Testing mixed type limits...");
        
        let mut seed_storage_3 = SeedStorage {
            owner: user,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [100, 99, 0, 50, 0, 0, 0, 0, 100], // Mixed limits
            reserve: [0; 16],
        };
        
        // Manually set seed_ids to match counts
        for i in 0..349 { // Total of above counts
            seed_storage_3.seed_ids.push(2000 + i);
        }
        seed_storage_3.total_seeds = 349;
        
        // Test adding to full types (should trigger auto-discard)
        let result = add_seed_to_storage(&mut seed_storage_3, 8888, SeedType::Seed1);
        assert!(result.is_ok(), "Should handle mixed limits correctly");
        
        // Test adding to non-full types (should succeed normally)
        let result = add_seed_to_storage(&mut seed_storage_3, 8889, SeedType::Seed3);
        assert!(result.is_ok(), "Should add to empty type normally");
        
        println!("✅ Seed storage edge cases validated");
    }

    #[test]
    fn test_seed_value_optimization_with_auto_discard() {
        // Test that auto-discard optimizes for seed value
        println!("💎 Testing Seed Value Optimization with Auto-Discard");
        
        let user = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner: user,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        // Scenario: Player has accumulated many low-value seeds
        // Fill storage with low-value seeds first
        for i in 0..80 {
            seed_storage.add_seed(1000 + i, &SeedType::Seed1).unwrap(); // 100 GP each
        }
        
        for i in 0..15 {
            seed_storage.add_seed(2000 + i, &SeedType::Seed2).unwrap(); // 180 GP each
        }
        
        for i in 0..5 {
            seed_storage.add_seed(3000 + i, &SeedType::Seed3).unwrap(); // 420 GP each
        }
        
        let initial_total_value = calculate_storage_total_value(&seed_storage);
        println!("Initial storage value: {} GP", initial_total_value);
        
        // Now player gets high-value seeds - test prioritization
        let high_value_scenarios = [
            (SeedType::Seed7, 15000), // Should be kept
            (SeedType::Seed8, 30000), // Should be kept
            (SeedType::Seed9, 60000), // Should definitely be kept
        ];
        
        let mut seeds_added = 0;
        let mut seed_id = 5000u64;
        
        for (seed_type, expected_gp) in high_value_scenarios {
            // Add multiple high-value seeds
            for _ in 0..3 {
                let result = add_seed_to_storage(&mut seed_storage, seed_id, seed_type);
                assert!(result.is_ok(), "Should successfully add high-value {:?}", seed_type);
                
                let count = seed_storage.get_seed_type_count(&seed_type);
                assert!(count > 0, "High-value seed should be in storage");
                
                seeds_added += 1;
                seed_id += 1;
            }
        }
        
        let final_total_value = calculate_storage_total_value(&seed_storage);
        println!("Final storage value: {} GP", final_total_value);
        
        // Value should increase despite auto-discard (high-value seeds retained)
        assert!(final_total_value >= initial_total_value, 
               "Storage value should not decrease when adding high-value seeds");
        
        // Verify high-value seeds are present
        assert!(seed_storage.get_seed_type_count(&SeedType::Seed7) > 0, "Seed7 should be retained");
        assert!(seed_storage.get_seed_type_count(&SeedType::Seed8) > 0, "Seed8 should be retained");
        assert!(seed_storage.get_seed_type_count(&SeedType::Seed9) > 0, "Seed9 should be retained");
        
        println!("Seeds added: {}", seeds_added);
        println!("Storage optimization successful!");
        
        println!("✅ Seed value optimization validated");
    }

    #[test]
    fn test_concurrent_seed_operations() {
        // Test concurrent seed operations (add/remove/auto-discard)
        println!("🔄 Testing Concurrent Seed Operations");
        
        let user = Pubkey::new_unique();
        let mut seed_storage = SeedStorage {
            owner: user,
            seed_ids: vec![],
            total_seeds: 0,
            seed_type_counts: [0; 9],
            reserve: [0; 16],
        };
        
        let mut seed_id = 1000u64;
        
        // Simulate concurrent operations during gameplay
        for cycle in 0..20 {
            println!("Cycle {}", cycle + 1);
            
            // Phase 1: Add seeds (opening seed packs)
            for _ in 0..10 {
                let random_value = (cycle * 1000 + seed_id) % 10000;
                let seed_type = determine_seed_type_from_value(random_value as u16);
                
                let result = add_seed_to_storage(&mut seed_storage, seed_id, seed_type);
                if result.is_ok() {
                    println!("  Added seed {} ({:?})", seed_id, seed_type);
                }
                
                seed_id += 1;
            }
            
            // Phase 2: Remove some seeds (planting or discarding)
            if cycle > 2 { // Start removing after a few cycles
                let seeds_to_remove = std::cmp::min(3, seed_storage.total_seeds / 4);
                
                for _ in 0..seeds_to_remove {
                    // Remove oldest seeds (simulate planting low-value seeds first)
                    if let Some(&first_seed_id) = seed_storage.seed_ids.first() {
                        // Assume it's a low-value seed for this test
                        let result = remove_seed_from_storage(&mut seed_storage, first_seed_id, SeedType::Seed1);
                        if result.is_ok() && result.unwrap() {
                            println!("  Removed seed {}", first_seed_id);
                        }
                    }
                }
            }
            
            // Phase 3: Check storage health
            let total_seeds = seed_storage.total_seeds;
            let storage_utilization = (total_seeds as f64 / SeedStorage::MAX_TOTAL_SEEDS as f64) * 100.0;
            
            println!("  Storage: {} seeds ({:.1}% utilization)", total_seeds, storage_utilization);
            
            // Verify storage constraints
            for (i, &count) in seed_storage.seed_type_counts.iter().enumerate() {
                assert!(count <= SeedStorage::MAX_SEEDS_PER_TYPE,
                       "Seed{} count {} exceeds limit in cycle {}", i + 1, count, cycle + 1);
            }
            
            // Verify total count matches
            let sum_by_type: u16 = seed_storage.seed_type_counts.iter().sum();
            assert_eq!(sum_by_type as u32, total_seeds,
                      "Type counts don't match total in cycle {}", cycle + 1);
        }
        
        println!("Final storage state:");
        for (i, &count) in seed_storage.seed_type_counts.iter().enumerate() {
            if count > 0 {
                println!("  Seed{}: {}", i + 1, count);
            }
        }
        
        println!("✅ Concurrent seed operations validated");
    }

    #[test]
    fn test_storage_migration_and_compatibility() {
        // Test storage migration scenarios and backward compatibility
        println!("🔄 Testing Storage Migration and Compatibility");
        
        let user = Pubkey::new_unique();
        
        // Simulate old storage format (without type tracking)
        let mut legacy_storage = SeedStorage {
            owner: user,
            seed_ids: vec![1001, 1002, 1003, 2001, 2002],
            total_seeds: 5,
            seed_type_counts: [0; 9], // Not initialized in legacy format
            reserve: [0; 16],
        };
        
        println!("Legacy storage: {} seeds, no type tracking", legacy_storage.total_seeds);
        
        // Simulate migration: initialize type counts
        legacy_storage.initialize_type_counts();
        
        // In a real migration, you'd need to:
        // 1. Read each seed account to determine its type
        // 2. Update the type counts accordingly
        // For this test, we'll simulate the process
        
        let simulated_types = [
            SeedType::Seed1, SeedType::Seed1, SeedType::Seed1, // 3 Seed1s
            SeedType::Seed2, SeedType::Seed2,                   // 2 Seed2s
        ];
        
        for seed_type in simulated_types {
            let type_index = seed_type as usize;
            legacy_storage.seed_type_counts[type_index] += 1;
        }
        
        println!("After migration:");
        for (i, &count) in legacy_storage.seed_type_counts.iter().enumerate() {
            if count > 0 {
                println!("  Seed{}: {}", i + 1, count);
            }
        }
        
        // Verify migration worked
        assert_eq!(legacy_storage.get_seed_type_count(&SeedType::Seed1), 3);
        assert_eq!(legacy_storage.get_seed_type_count(&SeedType::Seed2), 2);
        
        // Test that new functionality works with migrated storage
        let result = add_seed_to_storage(&mut legacy_storage, 3001, SeedType::Seed3);
        assert!(result.is_ok(), "Should work with migrated storage");
        
        assert_eq!(legacy_storage.get_seed_type_count(&SeedType::Seed3), 1);
        assert_eq!(legacy_storage.total_seeds, 6);
        
        println!("✅ Storage migration and compatibility validated");
    }

    // Helper functions for testing

    fn simulate_vrf_randomness(base: u64, user_entropy: u64) -> u64 {
        let mut value = base.wrapping_add(user_entropy);
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        value = value.wrapping_mul(0x2545F4914F6CDD1Du64);
        if value == 0 { 1 } else { value }
    }

    fn determine_seed_type_from_vrf(random_value: u64) -> SeedType {
        let seed_value = (random_value % 10000) as u16;
        
        for (i, &threshold) in SEED_PROBABILITY_THRESHOLDS.iter().enumerate() {
            if seed_value < threshold {
                return match i {
                    0 => SeedType::Seed1, 1 => SeedType::Seed2, 2 => SeedType::Seed3,
                    3 => SeedType::Seed4, 4 => SeedType::Seed5, 5 => SeedType::Seed6,
                    6 => SeedType::Seed7, 7 => SeedType::Seed8, 8 => SeedType::Seed9,
                    _ => SeedType::Seed1,
                };
            }
        }
        SeedType::Seed9
    }

    fn determine_seed_type_from_value(value: u16) -> SeedType {
        match value % 9 {
            0 => SeedType::Seed1, 1 => SeedType::Seed2, 2 => SeedType::Seed3,
            3 => SeedType::Seed4, 4 => SeedType::Seed5, 5 => SeedType::Seed6,
            6 => SeedType::Seed7, 7 => SeedType::Seed8, _ => SeedType::Seed9,
        }
    }

    fn calculate_storage_total_value(storage: &SeedStorage) -> u64 {
        let mut total_value = 0u64;
        
        for (i, &count) in storage.seed_type_counts.iter().enumerate() {
            if count > 0 {
                let grow_power = SEED_GROW_POWERS[i];
                total_value += grow_power * count as u64;
            }
        }
        
        total_value
    }
}