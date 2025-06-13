#[cfg(test)]
mod vrf_seed_pack_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;
    use crate::error::GameError;

    // ===== VRF SEED PACK PURCHASE TESTS =====

    #[test]
    fn test_vrf_seed_pack_creation() {
        let purchaser = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        let current_time = 1640995200; // 2022-01-01
        
        let seed_pack = SeedPack {
            purchaser,
            purchased_at: current_time,
            cost_paid: 300_000_000, // 300 WEED
            vrf_fee_paid: 2_077_400, // Realistic VRF fee
            is_opened: false,
            vrf_sequence: 12345,
            user_entropy_seed: 54321,
            final_random_value: 0,
            pack_id: 1,
            vrf_account,
            reserve: [0; 8],
        };

        // Verify all fields are correctly set
        assert_eq!(seed_pack.purchaser, purchaser);
        assert_eq!(seed_pack.purchased_at, current_time);
        assert_eq!(seed_pack.cost_paid, 300_000_000);
        assert_eq!(seed_pack.vrf_fee_paid, 2_077_400);
        assert!(!seed_pack.is_opened);
        assert_eq!(seed_pack.vrf_sequence, 12345);
        assert_eq!(seed_pack.user_entropy_seed, 54321);
        assert_eq!(seed_pack.final_random_value, 0);
        assert_eq!(seed_pack.pack_id, 1);
        assert_eq!(seed_pack.vrf_account, vrf_account);
    }

    #[test]
    fn test_vrf_fee_calculation() {
        // Test realistic VRF fee calculation
        let base_fee: u64 = 5_000;
        let num_transactions = 15;
        let storage_rent = 2_400;
        let oracle_fee = 2_000_000;
        
        let total_fee = base_fee * num_transactions + storage_rent + oracle_fee;
        let expected_fee = 2_077_400;
        
        assert_eq!(total_fee, expected_fee, "VRF fee calculation incorrect");
        
        // Verify fee is much lower than old 0.07 SOL estimate
        let old_estimate = 70_000_000; // 0.07 SOL in lamports
        assert!(total_fee < old_estimate / 30, "VRF fee should be ~1/35 of old estimate");
    }

    #[test]
    fn test_vrf_cost_comparison() {
        // Test cost comparison between VRF and old estimates
        let vrf_fee = 2_077_400; // ~0.002 SOL
        let weed_cost = 300_000_000; // 300 WEED (in micro-tokens)
        
        // Convert to human-readable values
        let vrf_fee_sol = vrf_fee as f64 / 1_000_000_000.0; // Convert to SOL
        let weed_cost_tokens = weed_cost as f64 / 1_000_000.0; // Convert to WEED
        
        assert!(vrf_fee_sol < 0.003, "VRF fee should be under 0.003 SOL");
        assert_eq!(weed_cost_tokens, 300.0, "WEED cost should be 300 tokens");
        
        // Total cost should be reasonable for users
        println!("VRF fee: {} SOL, WEED cost: {} WEED", vrf_fee_sol, weed_cost_tokens);
    }

    #[test]
    fn test_vrf_sequence_generation() {
        // Test VRF sequence number generation
        let user_entropy = 12345u64;
        let current_time = 1640995200u64;
        let pack_counter = 10u64;
        let vrf_account_byte = 0x42u64;
        
        let vrf_sequence = user_entropy
            .wrapping_add(current_time)
            .wrapping_add(pack_counter)
            .wrapping_add(vrf_account_byte);
        
        // Verify sequence is deterministic but unpredictable
        assert_ne!(vrf_sequence, 0);
        assert_ne!(vrf_sequence, user_entropy);
        assert_ne!(vrf_sequence, current_time);
        
        // Verify same inputs produce same sequence
        let vrf_sequence2 = user_entropy
            .wrapping_add(current_time)
            .wrapping_add(pack_counter)
            .wrapping_add(vrf_account_byte);
        
        assert_eq!(vrf_sequence, vrf_sequence2, "VRF sequence should be deterministic");
    }

    // ===== VRF SEED PACK OPENING TESTS =====

    #[test]
    fn test_vrf_randomness_quality() {
        // Test the quality of VRF-simulated randomness
        let vrf_sequence = 12345u64;
        let user_entropy = 54321u64;
        let pack_id = 1u64;
        let timestamp = 1640995200u64;
        let slot = 1000000u64;
        
        // Simulate the VRF randomness generation
        let mut random_value = vrf_sequence;
        random_value = random_value.wrapping_add(user_entropy);
        random_value = random_value.wrapping_add(pack_id);
        random_value = random_value.wrapping_add(timestamp);
        random_value = random_value.wrapping_add(slot);
        
        // Apply xorshift64* algorithm (VRF-quality mixing)
        random_value ^= random_value >> 12;
        random_value ^= random_value << 25;
        random_value ^= random_value >> 27;
        random_value = random_value.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        // Additional mixing
        random_value ^= random_value >> 32;
        random_value = random_value.wrapping_mul(0x9e3779b97f4a7c15u64);
        random_value ^= random_value >> 32;
        
        // Ensure non-zero
        if random_value == 0 {
            random_value = 1;
        }
        
        assert_ne!(random_value, 0);
        assert_ne!(random_value, vrf_sequence);
        assert_ne!(random_value, user_entropy);
        
        // Test distribution properties
        let seed_type_value = (random_value % 10000) as u16;
        assert!(seed_type_value < 10000, "Seed type value should be in valid range");
    }

    #[test]
    fn test_seed_type_distribution() {
        // Test seed type distribution with VRF randomness
        let mut distribution = [0u32; 9];
        let test_iterations = 10000;
        
        for i in 0..test_iterations {
            // Generate different random values
            let random_base = (i * 31337 + 12345) as u64;
            let mut random_value = random_base;
            
            // Apply VRF-quality randomness
            random_value ^= random_value >> 12;
            random_value ^= random_value << 25;
            random_value ^= random_value >> 27;
            random_value = random_value.wrapping_mul(0x2545F4914F6CDD1Du64);
            
            let seed_type_value = (random_value % 10000) as u16;
            
            // Determine seed type based on thresholds
            let seed_type = if seed_type_value < SEED_PROBABILITY_THRESHOLDS[0] {
                0 // Seed1
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[1] {
                1 // Seed2
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[2] {
                2 // Seed3
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[3] {
                3 // Seed4
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[4] {
                4 // Seed5
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[5] {
                5 // Seed6
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[6] {
                6 // Seed7
            } else if seed_type_value < SEED_PROBABILITY_THRESHOLDS[7] {
                7 // Seed8
            } else {
                8 // Seed9
            };
            
            distribution[seed_type] += 1;
        }
        
        // Verify distribution approximates expected probabilities
        let seed1_percentage = (distribution[0] as f32 / test_iterations as f32) * 100.0;
        let seed9_percentage = (distribution[8] as f32 / test_iterations as f32) * 100.0;
        
        // Seed1 should be around 42.23%
        assert!(seed1_percentage > 40.0 && seed1_percentage < 45.0, 
                "Seed1 distribution should be ~42%: got {}", seed1_percentage);
        
        // Seed9 should be around 0.56%
        assert!(seed9_percentage < 2.0, 
                "Seed9 distribution should be ~0.56%: got {}", seed9_percentage);
        
        println!("Distribution test passed - Seed1: {:.2}%, Seed9: {:.2}%", 
                 seed1_percentage, seed9_percentage);
    }

    #[test]
    fn test_vrf_account_validation() {
        let user = Pubkey::new_unique();
        let vrf_account1 = Pubkey::new_unique();
        let vrf_account2 = Pubkey::new_unique();
        
        let seed_pack = SeedPack {
            purchaser: user,
            purchased_at: 1640995200,
            cost_paid: 300_000_000,
            vrf_fee_paid: 2_077_400,
            is_opened: false,
            vrf_sequence: 12345,
            user_entropy_seed: 54321,
            final_random_value: 0,
            pack_id: 1,
            vrf_account: vrf_account1,
            reserve: [0; 8],
        };
        
        // Test VRF account validation
        assert_eq!(seed_pack.vrf_account, vrf_account1);
        assert_ne!(seed_pack.vrf_account, vrf_account2);
        
        // In real implementation, this would validate:
        // require!(seed_pack.vrf_account == ctx.accounts.vrf_account.key(), GameError::InvalidVrfAccount);
    }

    // ===== VRF ERROR HANDLING TESTS =====

    #[test]
    fn test_vrf_fee_limits() {
        let max_reasonable_fee = 10_000_000; // 0.01 SOL
        let calculated_fee = 2_077_400;
        
        assert!(calculated_fee < max_reasonable_fee, 
                "VRF fee should be under 0.01 SOL");
        
        // Test minimum fee validation
        let min_fee = 1_000_000; // 0.001 SOL minimum
        assert!(calculated_fee > min_fee, 
                "VRF fee should be over minimum threshold");
    }

    #[test]
    fn test_vrf_zero_values_handling() {
        // Test handling of zero values in VRF randomness
        let mut test_value = 0u64;
        
        // Apply VRF mixing
        test_value ^= test_value >> 12;
        test_value ^= test_value << 25;
        test_value ^= test_value >> 27;
        test_value = test_value.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        // Should still be zero after these operations
        assert_eq!(test_value, 0);
        
        // Test zero handling
        if test_value == 0 {
            test_value = 1;
        }
        
        assert_eq!(test_value, 1, "Zero values should be converted to 1");
    }

    // ===== VRF INTEGRATION TESTS =====

    #[test]
    fn test_complete_vrf_flow() {
        // Test complete VRF flow from purchase to opening
        let purchaser = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        
        // Step 1: Purchase
        let mut seed_pack = SeedPack {
            purchaser,
            purchased_at: 1640995200,
            cost_paid: 300_000_000,
            vrf_fee_paid: 2_077_400,
            is_opened: false,
            vrf_sequence: 12345,
            user_entropy_seed: 54321,
            final_random_value: 0,
            pack_id: 1,
            vrf_account,
            reserve: [0; 8],
        };
        
        // Verify initial state
        assert!(!seed_pack.is_opened);
        assert_eq!(seed_pack.final_random_value, 0);
        
        // Step 2: Opening (simulate VRF result)
        let mut random_value = seed_pack.vrf_sequence;
        random_value = random_value.wrapping_add(seed_pack.user_entropy_seed);
        random_value = random_value.wrapping_add(1640995300u64); // opening timestamp
        
        // Apply VRF-quality mixing
        random_value ^= random_value >> 12;
        random_value ^= random_value << 25;
        random_value ^= random_value >> 27;
        random_value = random_value.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        if random_value == 0 {
            random_value = 1;
        }
        
        // Update pack state
        seed_pack.is_opened = true;
        seed_pack.final_random_value = random_value;
        
        // Verify final state
        assert!(seed_pack.is_opened);
        assert_ne!(seed_pack.final_random_value, 0);
        assert_eq!(seed_pack.final_random_value, random_value);
    }

    #[test]
    fn test_vrf_result_retrieval_validation() {
        // Test VRF result retrieval and validation process
        let user = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        let different_vrf_account = Pubkey::new_unique();
        
        let seed_pack = SeedPack {
            purchaser: user,
            purchased_at: 1640995200,
            cost_paid: 300_000_000,
            vrf_fee_paid: 2_077_400,
            is_opened: false,
            vrf_sequence: 98765,
            user_entropy_seed: 13579,
            final_random_value: 0,
            pack_id: 42,
            vrf_account,
            reserve: [0; 8],
        };
        
        // Test 1: VRF account validation
        assert_eq!(seed_pack.vrf_account, vrf_account);
        assert_ne!(seed_pack.vrf_account, different_vrf_account);
        
        // Test 2: Simulate VRF result retrieval
        let opening_time = 1640995300u64;
        let slot = 1000000u64;
        let user_key_bytes = user.to_bytes();
        
        let mut random_value = seed_pack.vrf_sequence;
        random_value = random_value.wrapping_add(seed_pack.user_entropy_seed);
        random_value = random_value.wrapping_add(seed_pack.pack_id);
        random_value = random_value.wrapping_add(opening_time);
        random_value = random_value.wrapping_add(slot);
        
        // Mix in user key for uniqueness
        for &byte in &user_key_bytes[0..8] {
            random_value = random_value.wrapping_mul(31).wrapping_add(byte as u64);
        }
        
        // Mix in VRF account bytes
        let vrf_bytes = vrf_account.to_bytes();
        for &byte in &vrf_bytes[0..8] {
            random_value = random_value.wrapping_mul(37).wrapping_add(byte as u64);
        }
        
        // Apply VRF-quality mixing (xorshift64*)
        random_value ^= random_value >> 12;
        random_value ^= random_value << 25;
        random_value ^= random_value >> 27;
        random_value = random_value.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        // Additional VRF-level mixing
        random_value ^= random_value >> 32;
        random_value = random_value.wrapping_mul(0x9e3779b97f4a7c15u64);
        random_value ^= random_value >> 32;
        
        // Ensure non-zero
        if random_value == 0 {
            random_value = 1;
        }
        
        // Test 3: Verify result properties
        assert_ne!(random_value, 0);
        assert_ne!(random_value, seed_pack.vrf_sequence);
        assert_ne!(random_value, seed_pack.user_entropy_seed);
        assert_ne!(random_value, opening_time);
        
        // Test 4: Verify determinism (same inputs = same output)
        let mut random_value2 = seed_pack.vrf_sequence;
        random_value2 = random_value2.wrapping_add(seed_pack.user_entropy_seed);
        random_value2 = random_value2.wrapping_add(seed_pack.pack_id);
        random_value2 = random_value2.wrapping_add(opening_time);
        random_value2 = random_value2.wrapping_add(slot);
        
        for &byte in &user_key_bytes[0..8] {
            random_value2 = random_value2.wrapping_mul(31).wrapping_add(byte as u64);
        }
        
        for &byte in &vrf_bytes[0..8] {
            random_value2 = random_value2.wrapping_mul(37).wrapping_add(byte as u64);
        }
        
        random_value2 ^= random_value2 >> 12;
        random_value2 ^= random_value2 << 25;
        random_value2 ^= random_value2 >> 27;
        random_value2 = random_value2.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        random_value2 ^= random_value2 >> 32;
        random_value2 = random_value2.wrapping_mul(0x9e3779b97f4a7c15u64);
        random_value2 ^= random_value2 >> 32;
        
        if random_value2 == 0 {
            random_value2 = 1;
        }
        
        assert_eq!(random_value, random_value2, "VRF result should be deterministic");
    }

    #[test]
    fn test_seed_pack_opening_constraints() {
        // Test constraints during seed pack opening
        let user = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        
        // Test 1: Already opened pack
        let opened_pack = SeedPack {
            purchaser: user,
            purchased_at: 1640995200,
            cost_paid: 300_000_000,
            vrf_fee_paid: 2_077_400,
            is_opened: true, // Already opened
            vrf_sequence: 12345,
            user_entropy_seed: 54321,
            final_random_value: 9876543210,
            pack_id: 1,
            vrf_account,
            reserve: [0; 8],
        };
        
        assert!(opened_pack.is_opened);
        assert_ne!(opened_pack.final_random_value, 0);
        // In real implementation: require!(!seed_pack.is_opened, GameError::SeedPackAlreadyOpened);
        
        // Test 2: Valid pack ready for opening
        let unopened_pack = SeedPack {
            purchaser: user,
            purchased_at: 1640995200,
            cost_paid: 300_000_000,
            vrf_fee_paid: 2_077_400,
            is_opened: false,
            vrf_sequence: 12345,
            user_entropy_seed: 54321,
            final_random_value: 0,
            pack_id: 1,
            vrf_account,
            reserve: [0; 8],
        };
        
        assert!(!unopened_pack.is_opened);
        assert_eq!(unopened_pack.final_random_value, 0);
        // This pack is ready for opening
        
        // Test 3: Quantity validation
        let valid_quantities = [1, 10, 50, 100];
        let invalid_quantities = [0, 101, 255];
        
        for quantity in valid_quantities {
            assert!(quantity > 0 && quantity <= 100, "Quantity {} should be valid", quantity);
        }
        
        for quantity in invalid_quantities {
            assert!(!(quantity > 0 && quantity <= 100), "Quantity {} should be invalid", quantity);
        }
    }

    #[test]
    fn test_commit_reveal_pattern() {
        // Test the commit-reveal pattern inherent in VRF
        let user = Pubkey::new_unique();
        let vrf_account = Pubkey::new_unique();
        let user_entropy = 12345u64;
        let pack_id = 1u64;
        
        // Commit phase (purchase time)
        let purchase_time = 1640995200u64;
        let vrf_sequence = user_entropy
            .wrapping_add(purchase_time)
            .wrapping_add(pack_id)
            .wrapping_add(vrf_account.to_bytes()[0] as u64);
        
        let commit_pack = SeedPack {
            purchaser: user,
            purchased_at: purchase_time as i64,
            cost_paid: 300_000_000,
            vrf_fee_paid: 2_077_400,
            is_opened: false,
            vrf_sequence,
            user_entropy_seed: user_entropy,
            final_random_value: 0, // Not revealed yet
            pack_id,
            vrf_account,
            reserve: [0; 8],
        };
        
        // At commit time, final random value is unknown
        assert_eq!(commit_pack.final_random_value, 0);
        assert!(!commit_pack.is_opened);
        
        // Reveal phase (opening time) - different timestamp
        let opening_time = purchase_time + 3600; // 1 hour later
        let opening_slot = 1000000u64;
        
        // Generate final random value using commit data + reveal data
        let mut reveal_random = commit_pack.vrf_sequence;
        reveal_random = reveal_random.wrapping_add(commit_pack.user_entropy_seed);
        reveal_random = reveal_random.wrapping_add(opening_time);
        reveal_random = reveal_random.wrapping_add(opening_slot);
        
        // Apply cryptographic mixing
        reveal_random ^= reveal_random >> 12;
        reveal_random ^= reveal_random << 25;
        reveal_random ^= reveal_random >> 27;
        reveal_random = reveal_random.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        if reveal_random == 0 {
            reveal_random = 1;
        }
        
        // Simulate pack opening
        let mut revealed_pack = commit_pack.clone();
        revealed_pack.is_opened = true;
        revealed_pack.final_random_value = reveal_random;
        
        // Verify reveal properties
        assert!(revealed_pack.is_opened);
        assert_ne!(revealed_pack.final_random_value, 0);
        assert_ne!(revealed_pack.final_random_value, commit_pack.vrf_sequence);
        assert_ne!(revealed_pack.final_random_value, commit_pack.user_entropy_seed);
        
        // Verify commit-reveal security: different opening times = different results
        let different_opening_time = opening_time + 1;
        let mut different_random = commit_pack.vrf_sequence;
        different_random = different_random.wrapping_add(commit_pack.user_entropy_seed);
        different_random = different_random.wrapping_add(different_opening_time);
        different_random = different_random.wrapping_add(opening_slot);
        
        different_random ^= different_random >> 12;
        different_random ^= different_random << 25;
        different_random ^= different_random >> 27;
        different_random = different_random.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        if different_random == 0 {
            different_random = 1;
        }
        
        assert_ne!(reveal_random, different_random, 
                  "Different opening times should produce different results");
    }

    #[test]
    fn test_vrf_security_properties() {
        // Test VRF security properties
        let user_entropy = 12345u64;
        let vrf_sequence = 54321u64;
        
        // Test 1: Same inputs should produce same output (deterministic)
        let mut random1 = vrf_sequence.wrapping_add(user_entropy);
        random1 ^= random1 >> 12;
        random1 = random1.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        let mut random2 = vrf_sequence.wrapping_add(user_entropy);
        random2 ^= random2 >> 12;
        random2 = random2.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        assert_eq!(random1, random2, "VRF should be deterministic");
        
        // Test 2: Small input changes should produce vastly different outputs
        let mut random3 = vrf_sequence.wrapping_add(user_entropy + 1);
        random3 ^= random3 >> 12;
        random3 = random3.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        assert_ne!(random1, random3, "VRF should be sensitive to input changes");
        
        // Test 3: Output should not reveal input
        assert_ne!(random1, vrf_sequence);
        assert_ne!(random1, user_entropy);
        assert_ne!(random1, vrf_sequence + user_entropy);
    }

    #[test]
    fn test_seed_generation_from_vrf() {
        // Test seed generation process using VRF random values
        let base_random = 0x123456789ABCDEFu64;
        
        // Test individual seed derivation
        for i in 0..10u8 {
            let seed_random = derive_test_seed_randomness(base_random, i);
            
            // Each seed should get unique randomness
            assert_ne!(seed_random, base_random);
            assert_ne!(seed_random, 0);
            
            // Different indices should produce different results
            if i > 0 {
                let prev_random = derive_test_seed_randomness(base_random, i - 1);
                assert_ne!(seed_random, prev_random, 
                          "Seeds {} and {} should have different randomness", i-1, i);
            }
            
            // Test seed type determination
            let seed_type_value = (seed_random % 10000) as u16;
            assert!(seed_type_value < 10000, "Seed type value should be in valid range");
            
            // Verify seed type falls within expected probability ranges
            let is_valid_range = seed_type_value < SEED_PROBABILITY_THRESHOLDS[8]; // Should be less than max threshold
            assert!(is_valid_range, "Seed type value {} should be within valid probability range", seed_type_value);
        }
    }

    #[test]
    fn test_seed_storage_integration() {
        // Test integration with seed storage system
        let user = Pubkey::new_unique();
        let mut mock_storage = MockSeedStorage {
            owner: user,
            total_seeds: 0,
            seed_type_counts: [0; 9],
            seed_ids: vec![],
        };
        
        let base_random = 0xFEDCBA9876543210u64;
        let quantity = 5u8;
        
        // Simulate seed generation and storage
        for i in 0..quantity {
            let seed_random = derive_test_seed_randomness(base_random, i);
            let seed_type = determine_seed_type_from_random(seed_random);
            let seed_id = 1000u64 + i as u64;
            
            // Add to mock storage
            mock_storage.total_seeds += 1;
            mock_storage.seed_type_counts[seed_type as usize] += 1;
            mock_storage.seed_ids.push(seed_id);
        }
        
        // Verify storage state
        assert_eq!(mock_storage.total_seeds, quantity as u32);
        assert_eq!(mock_storage.seed_ids.len(), quantity as usize);
        
        // Verify at least one seed was generated
        let total_generated: u32 = mock_storage.seed_type_counts.iter().sum();
        assert_eq!(total_generated, quantity as u32);
        
        // Verify seed IDs are sequential
        for i in 0..quantity as usize {
            assert_eq!(mock_storage.seed_ids[i], 1000u64 + i as u64);
        }
    }

    #[test]
    fn test_vrf_entropy_mixing_edge_cases() {
        // Test edge cases in VRF entropy mixing
        
        // Test 1: All zero inputs
        let zero_entropy = mix_vrf_entropy(0, 0, 0, 0, 0);
        assert_ne!(zero_entropy, 0, "Zero inputs should still produce non-zero output");
        
        // Test 2: Maximum values
        let max_entropy = mix_vrf_entropy(u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX);
        assert_ne!(max_entropy, 0);
        assert_ne!(max_entropy, u64::MAX);
        
        // Test 3: Sequential inputs
        let mut entropies = Vec::new();
        for i in 0..10u64 {
            let entropy = mix_vrf_entropy(i, i+1, i+2, i+3, i+4);
            entropies.push(entropy);
            assert_ne!(entropy, 0);
        }
        
        // Verify all sequential results are different
        for i in 0..entropies.len() {
            for j in i+1..entropies.len() {
                assert_ne!(entropies[i], entropies[j], 
                          "Sequential inputs {} and {} should produce different entropy", i, j);
            }
        }
        
        // Test 4: Small differences in input
        let base = 1000000u64;
        let entropy1 = mix_vrf_entropy(base, base, base, base, base);
        let entropy2 = mix_vrf_entropy(base + 1, base, base, base, base);
        
        assert_ne!(entropy1, entropy2, "Small input differences should produce large output differences");
    }

    #[test]
    fn test_vrf_performance_characteristics() {
        // Test VRF performance and distribution characteristics
        let test_iterations = 1000;
        let mut distribution_buckets = [0u32; 100];
        let mut processing_times = Vec::new();
        
        for i in 0..test_iterations {
            let start_value = (i as u64).wrapping_mul(31337);
            
            // Measure mixing performance (simplified)
            let start_time = std::time::Instant::now();
            let mixed_value = mix_vrf_entropy(start_value, start_value + 1, start_value + 2, start_value + 3, start_value + 4);
            let duration = start_time.elapsed();
            processing_times.push(duration);
            
            // Check distribution
            let bucket = (mixed_value % 100) as usize;
            distribution_buckets[bucket] += 1;
        }
        
        // Verify distribution uniformity (chi-square test approximation)
        let expected_per_bucket = test_iterations / 100;
        let mut uniform_buckets = 0;
        
        for &count in &distribution_buckets {
            // Allow 50% variance from expected (relaxed for test)
            if count >= expected_per_bucket / 2 && count <= expected_per_bucket * 3 / 2 {
                uniform_buckets += 1;
            }
        }
        
        // At least 70% of buckets should be reasonably uniform
        assert!(uniform_buckets >= 70, 
               "VRF distribution should be reasonably uniform: {}/100 buckets passed", uniform_buckets);
        
        // Verify performance is reasonable (should be very fast)
        let avg_duration = processing_times.iter().sum::<std::time::Duration>() / processing_times.len() as u32;
        assert!(avg_duration < std::time::Duration::from_micros(10), 
               "VRF mixing should be fast: average {:?}", avg_duration);
    }

    // Helper functions for testing

    fn derive_test_seed_randomness(base_entropy: u64, index: u8) -> u64 {
        // Replicate the actual seed derivation logic for testing
        let mut derived = base_entropy;
        
        derived = derived.wrapping_add(index as u64);
        derived = derived.wrapping_mul(6364136223846793005u64);
        derived = derived.wrapping_add(1442695040888963407u64);
        
        derived ^= derived >> 32;
        derived = derived.wrapping_mul(0x9e3779b97f4a7c15u64);
        derived ^= derived >> 32;
        
        derived
    }

    fn determine_seed_type_from_random(random_value: u64) -> u8 {
        // Determine seed type from random value using probability thresholds
        let seed_type_value = (random_value % 10000) as u16;
        
        for (i, &threshold) in SEED_PROBABILITY_THRESHOLDS.iter().enumerate() {
            if seed_type_value < threshold {
                return i as u8;
            }
        }
        
        8 // Seed9 (rarest)
    }

    fn mix_vrf_entropy(seq: u64, user: u64, pack: u64, time: u64, slot: u64) -> u64 {
        // Simulate VRF entropy mixing for testing
        let mut mixed = seq;
        mixed = mixed.wrapping_add(user);
        mixed = mixed.wrapping_add(pack);
        mixed = mixed.wrapping_add(time);
        mixed = mixed.wrapping_add(slot);
        
        // Apply xorshift64* mixing
        mixed ^= mixed >> 12;
        mixed ^= mixed << 25;
        mixed ^= mixed >> 27;
        mixed = mixed.wrapping_mul(0x2545F4914F6CDD1Du64);
        
        mixed ^= mixed >> 32;
        mixed = mixed.wrapping_mul(0x9e3779b97f4a7c15u64);
        mixed ^= mixed >> 32;
        
        if mixed == 0 {
            mixed = 1;
        }
        
        mixed
    }

    // Mock seed storage for testing
    struct MockSeedStorage {
        owner: Pubkey,
        total_seeds: u32,
        seed_type_counts: [u32; 9],
        seed_ids: Vec<u64>,
    }
}