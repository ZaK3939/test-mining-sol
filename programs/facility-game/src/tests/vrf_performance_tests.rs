#[cfg(test)]
mod vrf_performance_tests {
    use anchor_lang::prelude::*;
    use crate::state::*;
    use crate::constants::*;
    use std::time::{Duration, Instant};

    // ===== VRF PERFORMANCE AND SCALABILITY TESTS =====

    #[test]
    fn test_vrf_entropy_mixing_performance() {
        // Test VRF entropy mixing performance
        let iterations = 10_000;
        let mut total_duration = Duration::new(0, 0);
        let mut results = Vec::with_capacity(iterations);
        
        for i in 0..iterations {
            let start = Instant::now();
            
            // Simulate VRF entropy mixing
            let result = mix_vrf_entropy_performance_test(
                i as u64,
                (i * 31337) as u64,
                (i * 12345) as u64,
                1640995200u64 + i as u64,
                1000000u64 + i as u64,
            );
            
            let duration = start.elapsed();
            total_duration += duration;
            results.push(result);
        }
        
        let avg_duration = total_duration / iterations as u32;
        
        // Performance assertions
        assert!(avg_duration < Duration::from_micros(10), 
               "VRF mixing should be very fast: average {:?}", avg_duration);
        
        // Verify all results are unique
        results.sort();
        results.dedup();
        assert_eq!(results.len(), iterations, "All VRF results should be unique");
        
        println!("VRF entropy mixing performance: {} operations in {:?}, average {:?}",
                 iterations, total_duration, avg_duration);
    }

    #[test]
    fn test_seed_generation_batch_performance() {
        // Test seed generation performance for different batch sizes
        let batch_sizes = [1, 5, 10, 25, 50, 100];
        
        for &batch_size in &batch_sizes {
            let start = Instant::now();
            
            let base_random = 0x123456789ABCDEFu64;
            let mut generated_seeds = Vec::new();
            
            for i in 0..batch_size {
                let seed_random = derive_seed_randomness_perf_test(base_random, i);
                let seed_type = determine_seed_type_perf_test(seed_random);
                generated_seeds.push((seed_random, seed_type));
            }
            
            let duration = start.elapsed();
            let per_seed_duration = duration / batch_size as u32;
            
            // Performance assertions
            assert!(per_seed_duration < Duration::from_micros(5),
                   "Seed generation should be fast: {} seeds in {:?}, {:?} per seed",
                   batch_size, duration, per_seed_duration);
            
            // Verify all seeds are unique
            let mut unique_randoms: Vec<_> = generated_seeds.iter().map(|(r, _)| *r).collect();
            unique_randoms.sort();
            unique_randoms.dedup();
            assert_eq!(unique_randoms.len(), batch_size as usize, "All seeds should be unique");
            
            println!("Batch size {}: {} seeds in {:?}, {:?} per seed",
                     batch_size, batch_size, duration, per_seed_duration);
        }
    }

    #[test]
    fn test_concurrent_vrf_operations() {
        // Test performance of multiple concurrent VRF operations
        let num_operations = 1000;
        let num_seeds_per_operation = 10;
        
        let start = Instant::now();
        
        let mut all_results = Vec::new();
        
        // Simulate concurrent operations (sequential for test simplicity)
        for operation_id in 0..num_operations {
            let user_entropy = (operation_id as u64 + 1) * 0x987654321u64;
            let vrf_sequence = user_entropy.wrapping_add(1640995200u64);
            
            // Generate VRF result
            let vrf_result = simulate_vrf_result_perf_test(vrf_sequence, user_entropy, operation_id as u64);
            
            // Generate seeds from VRF result
            let mut operation_seeds = Vec::new();
            for seed_index in 0..num_seeds_per_operation {
                let seed_random = derive_seed_randomness_perf_test(vrf_result, seed_index);
                let seed_type = determine_seed_type_perf_test(seed_random);
                operation_seeds.push((seed_random, seed_type));
            }
            
            all_results.push((vrf_result, operation_seeds));
        }
        
        let total_duration = start.elapsed();
        let total_seeds = num_operations * num_seeds_per_operation;
        let per_operation_duration = total_duration / num_operations as u32;
        let per_seed_duration = total_duration / total_seeds as u32;
        
        // Performance assertions
        assert!(per_operation_duration < Duration::from_millis(1),
               "VRF operations should be fast: {:?} per operation", per_operation_duration);
        
        assert!(per_seed_duration < Duration::from_micros(100),
               "Seed generation should be fast: {:?} per seed", per_seed_duration);
        
        // Verify uniqueness across all operations
        let mut all_vrf_results: Vec<_> = all_results.iter().map(|(vrf, _)| *vrf).collect();
        all_vrf_results.sort();
        all_vrf_results.dedup();
        assert_eq!(all_vrf_results.len(), num_operations as usize, "All VRF results should be unique");
        
        println!("Concurrent operations test: {} operations, {} total seeds in {:?}",
                 num_operations, total_seeds, total_duration);
        println!("Performance: {:?} per operation, {:?} per seed",
                 per_operation_duration, per_seed_duration);
    }

    #[test]
    fn test_memory_usage_efficiency() {
        // Test memory usage efficiency of VRF structures
        use std::mem;
        
        // Test SeedPack memory layout
        let seed_pack_size = mem::size_of::<SeedPack>();
        println!("SeedPack size: {} bytes", seed_pack_size);
        
        // Should be reasonably compact
        assert!(seed_pack_size <= 200, "SeedPack should be memory efficient: {} bytes", seed_pack_size);
        
        // Test individual field sizes
        println!("Field sizes:");
        println!("  purchaser: {} bytes", mem::size_of::<Pubkey>());
        println!("  purchased_at: {} bytes", mem::size_of::<i64>());
        println!("  cost_paid: {} bytes", mem::size_of::<u64>());
        println!("  vrf_fee_paid: {} bytes", mem::size_of::<u64>());
        println!("  is_opened: {} bytes", mem::size_of::<bool>());
        println!("  vrf_sequence: {} bytes", mem::size_of::<u64>());
        println!("  user_entropy_seed: {} bytes", mem::size_of::<u64>());
        println!("  final_random_value: {} bytes", mem::size_of::<u64>());
        println!("  pack_id: {} bytes", mem::size_of::<u64>());
        println!("  vrf_account: {} bytes", mem::size_of::<Pubkey>());
        println!("  reserve: {} bytes", mem::size_of::<[u8; 8]>());
        
        // Test memory allocation patterns
        let large_batch_size = 1000;
        let start = Instant::now();
        
        let mut seed_packs = Vec::with_capacity(large_batch_size);
        for i in 0..large_batch_size {
            let seed_pack = SeedPack {
                purchaser: Pubkey::new_unique(),
                purchased_at: 1640995200,
                cost_paid: 300_000_000,
                vrf_fee_paid: 2_077_400,
                is_opened: false,
                vrf_sequence: i as u64,
                user_entropy_seed: (i * 12345) as u64,
                final_random_value: 0,
                pack_id: i as u64,
                vrf_account: Pubkey::new_unique(),
                reserve: [0; 8],
            };
            seed_packs.push(seed_pack);
        }
        
        let allocation_duration = start.elapsed();
        
        // Memory allocation should be fast
        assert!(allocation_duration < Duration::from_millis(100),
               "Memory allocation should be fast: {} allocations in {:?}",
               large_batch_size, allocation_duration);
        
        // Verify all allocations are valid
        assert_eq!(seed_packs.len(), large_batch_size);
        
        println!("Memory efficiency test: {} SeedPacks allocated in {:?}",
                 large_batch_size, allocation_duration);
    }

    #[test]
    fn test_randomness_distribution_performance() {
        // Test performance of randomness distribution calculation
        let sample_sizes = [100, 1000, 10000];
        
        for &sample_size in &sample_sizes {
            let start = Instant::now();
            
            let mut distribution = [0u32; 9];
            let base_entropy = 0x987654321ABCDEFu64;
            
            for i in 0..sample_size {
                let seed_random = derive_seed_randomness_perf_test(
                    base_entropy.wrapping_add(i as u64),
                    (i % 256) as u8,
                );
                let seed_type = determine_seed_type_perf_test(seed_random);
                distribution[seed_type as usize] += 1;
            }
            
            let duration = start.elapsed();
            let per_sample_duration = duration / sample_size as u32;
            
            // Performance assertions
            assert!(per_sample_duration < Duration::from_micros(10),
                   "Distribution calculation should be fast: {:?} per sample", per_sample_duration);
            
            // Verify distribution is reasonable
            let total: u32 = distribution.iter().sum();
            assert_eq!(total, sample_size, "Total distribution should equal sample size");
            
            // At least some variety should exist for larger samples
            if sample_size >= 1000 {
                let non_zero_types = distribution.iter().filter(|&&count| count > 0).count();
                assert!(non_zero_types >= 7, "Large samples should have good type variety");
            }
            
            println!("Distribution performance - Sample size {}: {:?} total, {:?} per sample",
                     sample_size, duration, per_sample_duration);
        }
    }

    #[test]
    fn test_vrf_scalability_limits() {
        // Test VRF system scalability limits
        let max_operations = 100_000;
        let batch_size = 1000;
        
        let mut total_operations = 0;
        let start_time = Instant::now();
        
        for batch in 0..(max_operations / batch_size) {
            let batch_start = Instant::now();
            
            for i in 0..batch_size {
                let operation_id = batch * batch_size + i;
                let user_entropy = (operation_id as u64 + 1) * 0x123456789u64;
                
                // Simulate VRF operation
                let _vrf_result = simulate_vrf_result_perf_test(
                    user_entropy,
                    user_entropy.wrapping_add(1),
                    operation_id as u64,
                );
                
                // Generate single seed
                let _seed_random = derive_seed_randomness_perf_test(_vrf_result, 0);
                let _seed_type = determine_seed_type_perf_test(_seed_random);
                
                total_operations += 1;
            }
            
            let batch_duration = batch_start.elapsed();
            let batch_rate = batch_size as f64 / batch_duration.as_secs_f64();
            
            // Ensure consistent performance across batches
            assert!(batch_rate > 10_000.0, "Batch {} should maintain high throughput: {:.0} ops/sec", batch, batch_rate);
            
            // Log progress for larger batches
            if batch % 10 == 0 {
                println!("Batch {}: {} operations at {:.0} ops/sec", batch, batch_size, batch_rate);
            }
        }
        
        let total_duration = start_time.elapsed();
        let overall_rate = total_operations as f64 / total_duration.as_secs_f64();
        
        // Overall performance assertions
        assert!(overall_rate > 5_000.0, "Overall throughput should be high: {:.0} ops/sec", overall_rate);
        
        println!("Scalability test completed: {} operations in {:?}, {:.0} ops/sec",
                 total_operations, total_duration, overall_rate);
    }

    #[test]
    fn test_error_handling_performance() {
        // Test performance impact of error handling in VRF operations
        let num_tests = 10_000;
        
        // Test successful operations
        let success_start = Instant::now();
        let mut successful_operations = 0;
        
        for i in 0..num_tests {
            let result = simulate_vrf_validation_perf_test(
                i as u64 + 1, // Valid entropy
                5_000_000,    // Sufficient fee
                10,           // Valid quantity
            );
            
            if result.is_ok() {
                successful_operations += 1;
            }
        }
        
        let success_duration = success_start.elapsed();
        
        // Test error operations  
        let error_start = Instant::now();
        let mut error_operations = 0;
        
        for i in 0..num_tests {
            let result = simulate_vrf_validation_perf_test(
                0,         // Invalid entropy (will cause error)
                1_000_000, // Insufficient fee (will cause error)
                0,         // Invalid quantity (will cause error)
            );
            
            if result.is_err() {
                error_operations += 1;
            }
        }
        
        let error_duration = error_start.elapsed();
        
        // Performance comparisons
        let success_per_op = success_duration / successful_operations as u32;
        let error_per_op = error_duration / error_operations as u32;
        
        println!("Error handling performance:");
        println!("  Successful operations: {} in {:?}, {:?} per op", 
                 successful_operations, success_duration, success_per_op);
        println!("  Error operations: {} in {:?}, {:?} per op", 
                 error_operations, error_duration, error_per_op);
        
        // Error handling shouldn't be significantly slower
        let performance_ratio = error_per_op.as_nanos() as f64 / success_per_op.as_nanos() as f64;
        assert!(performance_ratio < 2.0, "Error handling shouldn't be much slower: {:.2}x", performance_ratio);
        
        // Both should be fast
        assert!(success_per_op < Duration::from_micros(10), "Success operations should be fast");
        assert!(error_per_op < Duration::from_micros(20), "Error operations should be reasonably fast");
    }

    // ===== PERFORMANCE TEST HELPER FUNCTIONS =====

    fn mix_vrf_entropy_performance_test(seq: u64, user: u64, pack: u64, time: u64, slot: u64) -> u64 {
        let mut mixed = seq;
        mixed = mixed.wrapping_add(user);
        mixed = mixed.wrapping_add(pack);
        mixed = mixed.wrapping_add(time);
        mixed = mixed.wrapping_add(slot);
        
        // xorshift64* algorithm
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

    fn derive_seed_randomness_perf_test(base_entropy: u64, index: u8) -> u64 {
        let mut derived = base_entropy;
        
        derived = derived.wrapping_add(index as u64);
        derived = derived.wrapping_mul(6364136223846793005u64);
        derived = derived.wrapping_add(1442695040888963407u64);
        
        derived ^= derived >> 32;
        derived = derived.wrapping_mul(0x9e3779b97f4a7c15u64);
        derived ^= derived >> 32;
        
        derived
    }

    fn determine_seed_type_perf_test(random_value: u64) -> u8 {
        let seed_type_value = (random_value % 10000) as u16;
        
        for (i, &threshold) in SEED_PROBABILITY_THRESHOLDS.iter().enumerate() {
            if seed_type_value < threshold {
                return i as u8;
            }
        }
        
        8 // Seed9
    }

    fn simulate_vrf_result_perf_test(vrf_sequence: u64, user_entropy: u64, pack_id: u64) -> u64 {
        mix_vrf_entropy_performance_test(
            vrf_sequence,
            user_entropy,
            pack_id,
            1640995200u64,
            1000000u64,
        )
    }

    fn simulate_vrf_validation_perf_test(user_entropy: u64, max_fee: u64, quantity: u8) -> Result<u64, &'static str> {
        if user_entropy == 0 {
            return Err("Invalid entropy");
        }
        
        if max_fee < 2_000_000 {
            return Err("Insufficient fee");
        }
        
        if quantity == 0 || quantity > 100 {
            return Err("Invalid quantity");
        }
        
        Ok(user_entropy.wrapping_mul(31337))
    }
}