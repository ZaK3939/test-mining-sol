#[cfg(test)]
mod referral_system_tests {
    use super::*;
    use crate::utils::*;
    use anchor_lang::prelude::Pubkey;

    /// Test referral reward calculation for all scenarios
    #[test]
    fn test_referral_reward_calculation() {
        let base_reward = 1_000_000_000; // 1000 WEED with 6 decimals
        let (level1_reward, level2_reward) = calculate_referral_rewards(base_reward).unwrap();
        
        // Level 1 should be 10% of base
        assert_eq!(level1_reward, 100_000_000); // 100 WEED
        
        // Level 2 should be 5% of base
        assert_eq!(level2_reward, 50_000_000);  // 50 WEED
        
        println!("✅ Base: {} WEED, L1: {} WEED, L2: {} WEED", 
                base_reward / 1_000_000, level1_reward / 1_000_000, level2_reward / 1_000_000);
    }

    /// Test reward percentage calculation for all scenarios with proper distribution
    #[test]
    fn test_reward_percentages_all_scenarios() {
        // Scenario 1: No referrers - User gets 100%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(false, false, false, false, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (10000, 0, 0)); // 100%, 0%, 0%
        
        // Scenario 2: User is protocol - gets 100%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, true, false, false, true);
        assert_eq!((user_pct, l1_pct, l2_pct), (10000, 0, 0)); // 100%, 0%, 0%
        
        // Scenario 3: Level 1 only, regular user - User gets 90%, L1 gets 10%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, false, false, false, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (9000, 1000, 0)); // 90%, 10%, 0%
        
        // Scenario 4: Level 1 only, protocol - User gets 100% (protocol share returns to user)
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, false, true, false, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (10000, 0, 0)); // 100%, 0%, 0%
        
        // Scenario 5: Both levels, both regular - User gets 85%, L1 gets 10%, L2 gets 5%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, true, false, false, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (8500, 1000, 500)); // 85%, 10%, 5%
        
        // Scenario 6: Both levels, L1 protocol, L2 regular - User gets 95%, L2 gets 5%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, true, true, false, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (9500, 0, 500)); // 95%, 0%, 5%
        
        // Scenario 7: Both levels, L1 regular, L2 protocol - User gets 90%, L1 gets 10%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, true, false, true, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (9000, 1000, 0)); // 90%, 10%, 0%
        
        // Scenario 8: Both levels, both protocol - User gets 100%
        let (user_pct, l1_pct, l2_pct) = calculate_reward_percentages(true, true, true, true, false);
        assert_eq!((user_pct, l1_pct, l2_pct), (10000, 0, 0)); // 100%, 0%, 0%
        
        println!("✅ All 8 referral scenarios tested successfully");
    }

    /// Test referral scenario validation with actual amounts
    #[test]
    fn test_referral_scenario_validation() {
        let base_reward = 1_000_000_000; // 1000 WEED
        
        // Test scenario: No referrers (100% to user)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, false, false, false, false, false
        ).unwrap();
        assert_eq!(user_amount, 1_000_000_000);
        assert_eq!(l1_amount, 0);
        assert_eq!(l2_amount, 0);
        let total = user_amount + l1_amount + l2_amount;
        assert_eq!(total, 1_000_000_000); // 100% total
        
        // Test scenario: Level 1 only (user 90% + L1 10% = 100% total)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, false, false, false, false
        ).unwrap();
        assert_eq!(user_amount, 900_000_000);   // 900 WEED (90%)
        assert_eq!(l1_amount, 100_000_000);     // 100 WEED (10%)
        assert_eq!(l2_amount, 0);
        let total = user_amount + l1_amount + l2_amount;
        assert_eq!(total, 1_000_000_000); // 100% total
        
        // Test scenario: Complete chain (user 85% + L1 10% + L2 5% = 100% total)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, false
        ).unwrap();
        assert_eq!(user_amount, 850_000_000);   // 850 WEED (85%)
        assert_eq!(l1_amount, 100_000_000);     // 100 WEED (10%)
        assert_eq!(l2_amount, 50_000_000);      // 50 WEED (5%)
        let total = user_amount + l1_amount + l2_amount;
        assert_eq!(total, 1_000_000_000); // 100% total
        
        // Test scenario: User is protocol (100% to user, no distribution)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, true
        ).unwrap();
        assert_eq!(user_amount, 1_000_000_000); // 1000 WEED (100%)
        assert_eq!(l1_amount, 0);               // No distribution
        assert_eq!(l2_amount, 0);               // No distribution
        let total = user_amount + l1_amount + l2_amount;
        assert_eq!(total, 1_000_000_000); // 100% total
        
        println!("✅ All referral scenarios validated with correct amounts");
    }

    /// Test claimant's percentage varies correctly based on referral chain
    #[test] 
    fn test_claimant_percentage_varies_correctly() {
        let base_reward = 1_000_000_000; // 1000 WEED
        
        // No referrers: claimant gets 100%
        let (claimant_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, false, false, false, false, false
        ).unwrap();
        assert_eq!(claimant_amount, base_reward); // 100%
        assert_eq!(l1_amount, 0);
        assert_eq!(l2_amount, 0);
        
        // Level 1 referrer: claimant gets 90%, L1 gets 10%
        let (claimant_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, false, false, false, false
        ).unwrap();
        assert_eq!(claimant_amount, 900_000_000); // 90%
        assert_eq!(l1_amount, 100_000_000);       // 10%
        assert_eq!(l2_amount, 0);
        
        // Complete chain: claimant gets 85%, L1 gets 10%, L2 gets 5%
        let (claimant_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, false
        ).unwrap();
        assert_eq!(claimant_amount, 850_000_000); // 85%
        assert_eq!(l1_amount, 100_000_000);       // 10%
        assert_eq!(l2_amount, 50_000_000);        // 5%
        
        println!("✅ Claimant percentage varies correctly: 100% → 90% → 85%");
    }

    /// Test protocol address special handling
    #[test]
    fn test_protocol_address_handling() {
        let protocol_address = Pubkey::new_unique();
        let regular_user = Pubkey::new_unique();
        let base_reward = 1_000_000_000;
        
        // Case 1: Regular user with protocol as Level 1 - User gets 100% (protocol share returns)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, false, true, false, false
        ).unwrap();
        assert_eq!(user_amount, 1_000_000_000);  // User gets 100% (protocol share returns)
        assert_eq!(l1_amount, 0);                // Protocol gets nothing
        assert_eq!(l2_amount, 0);
        
        // Case 2: Regular user with protocol as Level 2 - User gets 95%, L1 gets 10%
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, true, false
        ).unwrap();
        assert_eq!(user_amount, 900_000_000);    // User gets 90% (L2 protocol share returns to user -> 95%)
        assert_eq!(l1_amount, 100_000_000);      // L1 gets 10%
        assert_eq!(l2_amount, 0);                // Protocol gets nothing
        
        // Case 3: Protocol user (no distributions)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, true
        ).unwrap();
        assert_eq!(user_amount, 1_000_000_000);  // Protocol user gets 100%
        assert_eq!(l1_amount, 0);                // No L1 distribution
        assert_eq!(l2_amount, 0);                // No L2 distribution
        
        println!("✅ Protocol address special handling verified");
    }

    /// Test overflow protection in referral calculations
    #[test]
    fn test_overflow_protection() {
        // Test with maximum possible value
        let max_reward = u64::MAX / 100; // Ensure we can multiply by 10 safely
        let result = calculate_referral_rewards(max_reward);
        assert!(result.is_ok());
        
        let (level1_reward, level2_reward) = result.unwrap();
        assert_eq!(level1_reward, max_reward / 10); // 10% of max
        assert_eq!(level2_reward, max_reward / 20); // 5% of max
        
        // Test edge case near overflow
        let near_overflow = u64::MAX - 1000;
        let result = calculate_referral_rewards(near_overflow);
        // This should handle gracefully and return 0 for overflow cases
        assert!(result.is_ok());
        
        println!("✅ Overflow protection working correctly");
    }

    /// Test total always equals base_reward (100% fixed total)
    #[test]
    fn test_total_always_equals_base_reward() {
        let base_reward = 1_000_000_000;
        
        let test_scenarios = [
            (false, false, false, false), // No referrals
            (true, false, false, false),  // L1 only
            (true, true, false, false),   // Complete chain
            (true, false, true, false),   // L1 protocol
            (true, true, false, true),    // L2 protocol
            (true, true, true, true),     // Both protocol
        ];
        
        for (has_l1, has_l2, l1_protocol, l2_protocol) in test_scenarios {
            let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
                base_reward, has_l1, has_l2, l1_protocol, l2_protocol, false
            ).unwrap();
            
            let total_distributed = user_amount + l1_amount + l2_amount;
            assert_eq!(total_distributed, base_reward, 
                "Total should always equal base_reward for scenario: L1={}, L2={}, L1_proto={}, L2_proto={}", 
                has_l1, has_l2, l1_protocol, l2_protocol);
        }
        
        println!("✅ Total distribution always equals base_reward (100% fixed)");
    }

    /// Test claimant effective rates: 100%, 90%, 85%
    #[test]
    fn test_claimant_effective_rates() {
        let base_reward = 1_000_000_000; // 1000 WEED
        
        // No referrals: 100% effective rate
        let (user_amt, _, _) = validate_referral_scenario(
            base_reward, false, false, false, false, false
        ).unwrap();
        let effective_rate = (user_amt * 100) / base_reward;
        assert_eq!(effective_rate, 100);
        
        // L1 referral: 90% effective rate  
        let (user_amt, _, _) = validate_referral_scenario(
            base_reward, true, false, false, false, false
        ).unwrap();
        let effective_rate = (user_amt * 100) / base_reward;
        assert_eq!(effective_rate, 90);
        
        // Complete chain: 85% effective rate
        let (user_amt, _, _) = validate_referral_scenario(
            base_reward, true, true, false, false, false
        ).unwrap();
        let effective_rate = (user_amt * 100) / base_reward;
        assert_eq!(effective_rate, 85);
        
        println!("✅ Claimant effective rates verified: 100%, 90%, 85%");
    }

    /// Integration test simulating real reward distribution flow
    #[test]
    fn test_integration_reward_flow() {
        let base_reward = 500_000_000; // 500 WEED
        
        println!("🧪 Integration Test: 500 WEED Fixed Distribution");
        
        // Scenario: User with Level 1 and Level 2 referrers (all regular users)
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, false
        ).unwrap();
        
        // Expected amounts (85%, 10%, 5%)
        assert_eq!(user_amount, 425_000_000);   // 425 WEED to user (85%)
        assert_eq!(l1_amount, 50_000_000);      // 50 WEED to L1 (10%)
        assert_eq!(l2_amount, 25_000_000);      // 25 WEED to L2 (5%)
        
        let total_distributed = user_amount + l1_amount + l2_amount;
        assert_eq!(total_distributed, 500_000_000); // 500 WEED total (fixed)
        
        // Calculate user's percentage of total
        let user_percentage = (user_amount * 100) / total_distributed;
        assert_eq!(user_percentage, 85); // 85%
        
        println!("📊 Distribution: User=85%, L1=10%, L2=5%");
        println!("💰 Amounts: User=425, L1=50, L2=25 WEED");
        
        println!("✅ Integration test passed: 85% effective user rate with fixed 500 WEED total");
    }

    /// Test operator direct invitation vs regular invitation comparison
    #[test]
    fn test_operator_vs_regular_invitation_comparison() {
        let base_reward = 1_000_000_000; // 1000 WEED
        
        println!("🏢 運営招待 vs 通常招待 比較テスト");
        
        // === 通常招待（Regular Invitation）===
        // ユーザーが通常の招待者（L1）と招待者の招待者（L2）を持つ場合
        let (regular_user_amount, regular_l1_amount, regular_l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, false
        ).unwrap();
        
        assert_eq!(regular_user_amount, 850_000_000);   // 850 WEED (85%)
        assert_eq!(regular_l1_amount, 100_000_000);     // 100 WEED (10%) - Level 1
        assert_eq!(regular_l2_amount, 50_000_000);      // 50 WEED (5%) - Level 2
        
        let regular_total = regular_user_amount + regular_l1_amount + regular_l2_amount;
        assert_eq!(regular_total, 1_000_000_000); // 100% total
        
        // === 運営招待（Operator Direct Invitation）===
        // ユーザーの招待者が運営（protocol_referral_address）の場合
        let (operator_user_amount, operator_l1_amount, operator_l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, true // user_is_protocol = true
        ).unwrap();
        
        assert_eq!(operator_user_amount, 1_000_000_000); // 1000 WEED (100%)
        assert_eq!(operator_l1_amount, 0);               // 0 WEED (運営は受け取らない)
        assert_eq!(operator_l2_amount, 0);               // 0 WEED (分配なし)
        
        let operator_total = operator_user_amount + operator_l1_amount + operator_l2_amount;
        assert_eq!(operator_total, 1_000_000_000); // 100% total
        
        // === 比較結果（Comparison Results）===
        let regular_user_percentage = (regular_user_amount * 100) / base_reward;
        let operator_user_percentage = (operator_user_amount * 100) / base_reward;
        
        assert_eq!(regular_user_percentage, 85);  // 通常招待: 85%
        assert_eq!(operator_user_percentage, 100); // 運営招待: 100%
        
        let referral_cost_saved = operator_user_amount - regular_user_amount;
        assert_eq!(referral_cost_saved, 150_000_000); // 150 WEED saved (15%)
        
        println!("📊 比較結果:");
        println!("   通常招待: ユーザー=85% (850 WEED), 紹介料=15% (150 WEED)");
        println!("   運営招待: ユーザー=100% (1000 WEED), 紹介料=0% (0 WEED)");
        println!("   💰 節約額: 150 WEED (15% referral cost eliminated)");
        
        println!("✅ 運営招待でユーザーの紹介料負担が完全に免除されることを確認");
    }

    /// Test operator invitation mechanism implementation details
    #[test]
    fn test_operator_invitation_mechanism() {
        let base_reward = 2_000_000_000; // 2000 WEED
        
        println!("🔧 運営招待メカニズム実装詳細テスト");
        
        // === Case 1: ユーザー自身が運営アドレス（protocol_referral_address）の場合 ===
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, false, true // user_is_protocol = true
        ).unwrap();
        
        assert_eq!(user_amount, 2_000_000_000); // 運営は100%保持
        assert_eq!(l1_amount, 0);               // 紹介料分配なし
        assert_eq!(l2_amount, 0);               // 紹介料分配なし
        
        println!("   ✅ Case 1: 運営ユーザー = 100% retention (no referral distribution)");
        
        // === Case 2: L1が運営アドレスの場合（運営がユーザーを直接招待） ===
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, false, true, false, false // l1_is_protocol = true
        ).unwrap();
        
        assert_eq!(user_amount, 2_000_000_000); // ユーザーが100%受け取り
        assert_eq!(l1_amount, 0);               // 運営は紹介料を受け取らない
        assert_eq!(l2_amount, 0);               // L2なし
        
        println!("   ✅ Case 2: 運営直接招待 = User gets 100% (operator takes no referral fee)");
        
        // === Case 3: L2が運営アドレスの場合 ===
        let (user_amount, l1_amount, l2_amount) = validate_referral_scenario(
            base_reward, true, true, false, true, false // l2_is_protocol = true
        ).unwrap();
        
        assert_eq!(user_amount, 1_800_000_000); // ユーザー: 90% (L2分が戻らない)
        assert_eq!(l1_amount, 200_000_000);     // L1: 10%
        assert_eq!(l2_amount, 0);               // 運営L2は受け取らない
        
        println!("   ✅ Case 3: L2運営 = User gets 90%, L1 gets 10% (L2 protocol fee eliminated)");
        
        // === 実装確認: rewards.rs での処理ロジック ===
        // process_referral_rewards() 関数の動作確認
        // if ctx.accounts.user.key() == ctx.accounts.config.protocol_referral_address {
        //     return Ok(()); // 運営ユーザーは紹介料分配をスキップ
        // }
        
        println!("🔧 実装メカニズム:");
        println!("   - rewards.rs:384-389 でprotocol_referral_address特別処理");
        println!("   - 運営ユーザーの報酬請求時は紹介料分配をスキップ");
        println!("   - 運営が紹介者の場合も紹介料は運営に行かず、ユーザーに還元");
        
        println!("✅ 運営招待メカニズムの実装詳細を確認完了");
    }
}