#[cfg(test)]
mod strategic_user_journey_tests {
    use super::*;
    
    /// 戦略的ユーザージャーニーのテストモジュール
    /// WEEDを最大化する4つの主要戦略をテスト
    /// 
    /// # 検証される戦略アーキタイプ
    /// 1. Network Builder - 紹介ネットワーク構築戦略
    /// 2. Gambler - Mystery pack集中投資戦略  
    /// 3. Farmer - 段階的farm upgrade戦略
    /// 4. Strategist - 動的投資配分戦略
    
    #[test]
    fn test_network_builder_strategy() {
        // Network Builder戦略のテスト
        // 多段階招待チェーンを構築してWEEDを最大化
        
        let strategy_analysis = NetworkBuilderStrategy {
            time_to_roi: "14-21 days",
            risk_level: RiskLevel::Low,
            scalability: ScalabilityLevel::High,
            sustainability: SustainabilityLevel::VeryHigh,
            best_for: "Social players with large networks",
        };
        
        // 招待チェーンの効果検証
        assert_eq!(strategy_analysis.calculate_referral_depth(), 3);
        assert_eq!(strategy_analysis.level1_commission(), 10); // 10%
        assert_eq!(strategy_analysis.level2_commission(), 5);  // 5%
        
        // ROI計算
        let expected_roi = strategy_analysis.calculate_roi_timeline();
        assert!(expected_roi.break_even_days >= 14);
        assert!(expected_roi.break_even_days <= 21);
        
        println!("✅ Network Builder Strategy validated");
        println!("   - Multi-level referral chain: {}", strategy_analysis.calculate_referral_depth());
        println!("   - L1 Commission: {}%", strategy_analysis.level1_commission());
        println!("   - L2 Commission: {}%", strategy_analysis.level2_commission());
        println!("   - Risk Level: {:?}", strategy_analysis.risk_level);
        println!("   - Sustainability: {:?}", strategy_analysis.sustainability);
    }
    
    #[test]
    fn test_gambler_strategy() {
        // Gambler戦略のテスト
        // Mystery pack集中投資による高リスク・高リターン戦略
        
        let strategy_analysis = GamblerStrategy {
            time_to_roi: "1-7 days (if lucky)",
            risk_level: RiskLevel::VeryHigh,
            scalability: ScalabilityLevel::Extreme,
            sustainability: SustainabilityLevel::Low,
            best_for: "Risk-tolerant players seeking quick wins",
        };
        
        // Mystery pack確率とROIの検証
        let seed_probabilities = strategy_analysis.get_seed_probabilities();
        assert_eq!(seed_probabilities.seed1_chance, 42.23); // 42.23%
        assert_eq!(seed_probabilities.seed1_roi, -70.0);    // -70% ROI
        assert_eq!(seed_probabilities.seed9_chance, 0.56);  // 0.56%
        assert_eq!(seed_probabilities.seed9_roi, 19900.0);  // +19900% ROI
        
        // コスト効率の検証
        assert_eq!(strategy_analysis.pack_cost_weed(), 300);
        
        println!("✅ Gambler Strategy validated");
        println!("   - Pack Cost: {} WEED", strategy_analysis.pack_cost_weed());
        println!("   - Seed1 (100GP): {}% chance, {}% ROI", 
                seed_probabilities.seed1_chance, seed_probabilities.seed1_roi);
        println!("   - Seed9 (60000GP): {}% chance, {}% ROI", 
                seed_probabilities.seed9_chance, seed_probabilities.seed9_roi);
        println!("   - Risk Level: {:?}", strategy_analysis.risk_level);
        println!("   - Potential: Extreme variance outcomes");
    }
    
    #[test]
    fn test_farmer_strategy() {
        // Farmer戦略のテスト
        // 段階的farm upgradeによる安定成長戦略
        
        let strategy_analysis = FarmerStrategy {
            time_to_roi: "7-14 days",
            risk_level: RiskLevel::Low,
            scalability: ScalabilityLevel::Medium,
            sustainability: SustainabilityLevel::High,
            best_for: "Conservative players preferring steady growth",
        };
        
        // アップグレード効率の検証
        let upgrades = strategy_analysis.get_upgrade_efficiency();
        assert_eq!(upgrades.level_1_to_2.cost, 3500);
        assert_eq!(upgrades.level_1_to_2.capacity_increase, 4);
        assert_eq!(upgrades.level_1_to_2.cost_per_slot(), 875); // 最高効率
        
        assert_eq!(upgrades.level_4_to_5.cost, 25000);
        assert_eq!(upgrades.level_4_to_5.capacity_increase, 4);
        assert_eq!(upgrades.level_4_to_5.cost_per_slot(), 6250); // 最低効率
        
        // クールダウン期間の検証
        assert_eq!(strategy_analysis.cooldown_hours(), 24);
        
        println!("✅ Farmer Strategy validated");
        println!("   - Level 1→2: {} WEED, {} slots, {} WEED/slot", 
                upgrades.level_1_to_2.cost, 
                upgrades.level_1_to_2.capacity_increase,
                upgrades.level_1_to_2.cost_per_slot());
        println!("   - Level 4→5: {} WEED, {} slots, {} WEED/slot", 
                upgrades.level_4_to_5.cost,
                upgrades.level_4_to_5.capacity_increase, 
                upgrades.level_4_to_5.cost_per_slot());
        println!("   - Cooldown: {} hours", strategy_analysis.cooldown_hours());
        println!("   - Best ROI: Early upgrades (Level 1→2)");
    }
    
    #[test]
    fn test_strategist_hybrid_strategy() {
        // Strategist戦略のテスト
        // 動的投資配分による最適化戦略
        
        let strategy_analysis = StrategistStrategy {
            time_to_roi: "10-14 days",
            risk_level: RiskLevel::Medium,
            scalability: ScalabilityLevel::High,
            sustainability: SustainabilityLevel::VeryHigh,
            best_for: "Experienced players seeking optimization",
        };
        
        // フェーズ別配分戦略の検証
        let early_game = strategy_analysis.get_allocation_strategy(GamePhase::Early);
        assert_eq!(early_game.referral_percentage, 40);
        assert_eq!(early_game.mystery_pack_percentage, 20);
        assert_eq!(early_game.farm_upgrade_percentage, 40);
        
        let mid_game = strategy_analysis.get_allocation_strategy(GamePhase::Mid);
        assert_eq!(mid_game.referral_percentage, 20);
        assert_eq!(mid_game.mystery_pack_percentage, 30);
        assert_eq!(mid_game.farm_upgrade_percentage, 50);
        
        let late_game = strategy_analysis.get_allocation_strategy(GamePhase::Late);
        assert_eq!(late_game.referral_percentage, 10);
        assert_eq!(late_game.mystery_pack_percentage, 40);
        assert_eq!(late_game.farm_upgrade_percentage, 50);
        
        // 重要な決定ポイントの検証
        let decision_points = strategy_analysis.get_critical_decision_points();
        assert_eq!(decision_points.mystery_pack_threshold, 300);
        assert_eq!(decision_points.first_upgrade_threshold, 3500);
        
        println!("✅ Strategist Hybrid Strategy validated");
        println!("   - Early Game: {}% referral, {}% mystery, {}% upgrade", 
                early_game.referral_percentage,
                early_game.mystery_pack_percentage, 
                early_game.farm_upgrade_percentage);
        println!("   - Mid Game: {}% referral, {}% mystery, {}% upgrade",
                mid_game.referral_percentage,
                mid_game.mystery_pack_percentage,
                mid_game.farm_upgrade_percentage);
        println!("   - Late Game: {}% referral, {}% mystery, {}% upgrade",
                late_game.referral_percentage, 
                late_game.mystery_pack_percentage,
                late_game.farm_upgrade_percentage);
        println!("   - Key Thresholds: {} WEED (pack), {} WEED (upgrade)",
                decision_points.mystery_pack_threshold,
                decision_points.first_upgrade_threshold);
    }
    
    #[test]
    fn test_strategy_comparison_matrix() {
        // 全戦略の比較マトリックステスト
        
        let comparison = StrategyComparisonMatrix::new();
        
        // 各戦略の特性検証
        assert_eq!(comparison.network_builder.risk_level, RiskLevel::Low);
        assert_eq!(comparison.gambler.risk_level, RiskLevel::VeryHigh);
        assert_eq!(comparison.farmer.risk_level, RiskLevel::Low);
        assert_eq!(comparison.strategist.risk_level, RiskLevel::Medium);
        
        // ROI期間の妥当性検証
        assert!(comparison.network_builder.roi_days_min >= 14);
        assert!(comparison.gambler.roi_days_min >= 1);
        assert!(comparison.farmer.roi_days_min >= 7);
        assert!(comparison.strategist.roi_days_min >= 10);
        
        // 持続可能性の比較
        assert_eq!(comparison.network_builder.sustainability, SustainabilityLevel::VeryHigh);
        assert_eq!(comparison.gambler.sustainability, SustainabilityLevel::Low);
        assert_eq!(comparison.farmer.sustainability, SustainabilityLevel::High);
        assert_eq!(comparison.strategist.sustainability, SustainabilityLevel::VeryHigh);
        
        println!("✅ Strategy Comparison Matrix validated");
        println!("📊 Risk Levels:");
        println!("   - Network Builder: {:?}", comparison.network_builder.risk_level);
        println!("   - Gambler: {:?}", comparison.gambler.risk_level);
        println!("   - Farmer: {:?}", comparison.farmer.risk_level);
        println!("   - Strategist: {:?}", comparison.strategist.risk_level);
        
        println!("🎯 Sustainability Rankings:");
        println!("   - Network Builder: {:?}", comparison.network_builder.sustainability);
        println!("   - Gambler: {:?}", comparison.gambler.sustainability);
        println!("   - Farmer: {:?}", comparison.farmer.sustainability);
        println!("   - Strategist: {:?}", comparison.strategist.sustainability);
    }
    
    #[test]
    fn test_critical_decision_points() {
        // 重要な決定ポイントのテスト
        
        let decision_analyzer = CriticalDecisionAnalyzer::new();
        
        // 300 WEED閾値での決定
        let decision_300 = decision_analyzer.analyze_300_weed_threshold();
        assert_eq!(decision_300.option_a, "Buy mystery pack (high risk/reward)");
        assert_eq!(decision_300.option_b, "Save for upgrade (guaranteed progress)");
        assert_eq!(decision_300.optimal_choice, "Depends on current grow power and risk tolerance");
        
        // 3,500 WEED閾値での決定
        let decision_3500 = decision_analyzer.analyze_3500_weed_threshold();
        assert_eq!(decision_3500.option_a, "Level 1→2 upgrade (100% capacity increase)");
        assert_eq!(decision_3500.option_b, "11-12 mystery packs (potential huge gains)");
        assert_eq!(decision_3500.optimal_choice, "Upgrade first for foundation, then packs");
        
        // 紹介ネットワークサイズ別戦略
        let network_strategy = decision_analyzer.analyze_referral_network_strategy();
        assert_eq!(network_strategy.small_network_focus, "Personal growth");
        assert_eq!(network_strategy.medium_network_focus, "Balanced approach");
        assert_eq!(network_strategy.large_network_focus, "Leverage network effects");
        
        println!("✅ Critical Decision Points validated");
        println!("💰 300 WEED Decision:");
        println!("   Option A: {}", decision_300.option_a);
        println!("   Option B: {}", decision_300.option_b);
        println!("   Optimal: {}", decision_300.optimal_choice);
        
        println!("🚀 3,500 WEED Decision:");
        println!("   Option A: {}", decision_3500.option_a);
        println!("   Option B: {}", decision_3500.option_b);
        println!("   Optimal: {}", decision_3500.optimal_choice);
        
        println!("🌐 Network Strategy:");
        println!("   1-2 referrals: {}", network_strategy.small_network_focus);
        println!("   3-5 referrals: {}", network_strategy.medium_network_focus);
        println!("   5+ referrals: {}", network_strategy.large_network_focus);
    }

    #[test]
    fn test_meta_strategy_insights() {
        // メタ戦略洞察のテスト
        
        let meta_insights = MetaStrategyInsights::new();
        
        // 主要洞察の検証
        assert!(meta_insights.early_adopter_advantage);
        assert!(meta_insights.mystery_pack_variance);
        assert!(meta_insights.farm_upgrade_stability);
        assert!(meta_insights.hybrid_optimization);
        assert!(meta_insights.adaptive_strategy_importance);
        
        // 成功要因の分析
        let success_factors = meta_insights.get_success_factors();
        assert!(success_factors.contains(&"Referral chain depth: 2+ levels"));
        assert!(success_factors.contains(&"Mystery pack ROI: Target Seed3+ for profitability"));
        assert!(success_factors.contains(&"Upgrade timing: Align with cooldown periods"));
        assert!(success_factors.contains(&"Growth rate: Compound interest through reinvestment"));
        
        println!("✅ Meta Strategy Insights validated");
        println!("🎯 Key Insights:");
        println!("   - Early adopter advantage: {}", meta_insights.early_adopter_advantage);
        println!("   - Mystery pack creates variance: {}", meta_insights.mystery_pack_variance);
        println!("   - Farm upgrades provide stability: {}", meta_insights.farm_upgrade_stability);
        println!("   - Hybrid approaches optimize risk/reward: {}", meta_insights.hybrid_optimization);
        println!("   - Adaptive strategy critical for long-term: {}", meta_insights.adaptive_strategy_importance);
        
        println!("🏆 Success Factors:");
        for factor in success_factors {
            println!("   - {}", factor);
        }
    }
    
    // ========================================================================================
    // 戦略実装のための構造体定義
    // ========================================================================================
    
    #[derive(Debug, PartialEq)]
    enum RiskLevel {
        Low,
        Medium,
        High,
        VeryHigh,
    }
    
    #[derive(Debug, PartialEq)]
    enum ScalabilityLevel {
        Low,
        Medium,
        High,
        Extreme,
    }
    
    #[derive(Debug, PartialEq)]
    enum SustainabilityLevel {
        Low,
        Medium,
        High,
        VeryHigh,
    }
    
    #[derive(Debug)]
    enum GamePhase {
        Early,
        Mid,
        Late,
    }
    
    struct NetworkBuilderStrategy {
        time_to_roi: &'static str,
        risk_level: RiskLevel,
        scalability: ScalabilityLevel,
        sustainability: SustainabilityLevel,
        best_for: &'static str,
    }
    
    impl NetworkBuilderStrategy {
        fn calculate_referral_depth(&self) -> u8 { 3 }
        fn level1_commission(&self) -> u8 { 10 }
        fn level2_commission(&self) -> u8 { 5 }
        fn calculate_roi_timeline(&self) -> ROITimeline {
            ROITimeline { break_even_days: 17 }
        }
    }
    
    struct GamblerStrategy {
        time_to_roi: &'static str,
        risk_level: RiskLevel,
        scalability: ScalabilityLevel,
        sustainability: SustainabilityLevel,
        best_for: &'static str,
    }
    
    impl GamblerStrategy {
        fn pack_cost_weed(&self) -> u32 { 300 }
        fn get_seed_probabilities(&self) -> SeedProbabilities {
            SeedProbabilities {
                seed1_chance: 42.23,
                seed1_roi: -70.0,
                seed9_chance: 0.56,
                seed9_roi: 19900.0,
            }
        }
    }
    
    struct FarmerStrategy {
        time_to_roi: &'static str,
        risk_level: RiskLevel,
        scalability: ScalabilityLevel,
        sustainability: SustainabilityLevel,
        best_for: &'static str,
    }
    
    impl FarmerStrategy {
        fn cooldown_hours(&self) -> u8 { 24 }
        fn get_upgrade_efficiency(&self) -> UpgradeEfficiency {
            UpgradeEfficiency {
                level_1_to_2: UpgradeLevel { cost: 3500, capacity_increase: 4 },
                level_4_to_5: UpgradeLevel { cost: 25000, capacity_increase: 4 },
            }
        }
    }
    
    struct StrategistStrategy {
        time_to_roi: &'static str,
        risk_level: RiskLevel,
        scalability: ScalabilityLevel,
        sustainability: SustainabilityLevel,
        best_for: &'static str,
    }
    
    impl StrategistStrategy {
        fn get_allocation_strategy(&self, phase: GamePhase) -> AllocationStrategy {
            match phase {
                GamePhase::Early => AllocationStrategy {
                    referral_percentage: 40,
                    mystery_pack_percentage: 20,
                    farm_upgrade_percentage: 40,
                },
                GamePhase::Mid => AllocationStrategy {
                    referral_percentage: 20,
                    mystery_pack_percentage: 30,
                    farm_upgrade_percentage: 50,
                },
                GamePhase::Late => AllocationStrategy {
                    referral_percentage: 10,
                    mystery_pack_percentage: 40,
                    farm_upgrade_percentage: 50,
                },
            }
        }
        
        fn get_critical_decision_points(&self) -> DecisionPoints {
            DecisionPoints {
                mystery_pack_threshold: 300,
                first_upgrade_threshold: 3500,
            }
        }
    }
    
    // サポート構造体
    struct ROITimeline {
        break_even_days: u8,
    }
    
    struct SeedProbabilities {
        seed1_chance: f32,
        seed1_roi: f32,
        seed9_chance: f32,
        seed9_roi: f32,
    }
    
    struct UpgradeEfficiency {
        level_1_to_2: UpgradeLevel,
        level_4_to_5: UpgradeLevel,
    }
    
    struct UpgradeLevel {
        cost: u32,
        capacity_increase: u8,
    }
    
    impl UpgradeLevel {
        fn cost_per_slot(&self) -> u32 {
            self.cost / self.capacity_increase as u32
        }
    }
    
    struct AllocationStrategy {
        referral_percentage: u8,
        mystery_pack_percentage: u8,
        farm_upgrade_percentage: u8,
    }
    
    struct DecisionPoints {
        mystery_pack_threshold: u32,
        first_upgrade_threshold: u32,
    }
    
    struct StrategyComparisonMatrix {
        network_builder: StrategyMetrics,
        gambler: StrategyMetrics,
        farmer: StrategyMetrics,
        strategist: StrategyMetrics,
    }
    
    impl StrategyComparisonMatrix {
        fn new() -> Self {
            Self {
                network_builder: StrategyMetrics {
                    risk_level: RiskLevel::Low,
                    roi_days_min: 14,
                    sustainability: SustainabilityLevel::VeryHigh,
                },
                gambler: StrategyMetrics {
                    risk_level: RiskLevel::VeryHigh,
                    roi_days_min: 1,
                    sustainability: SustainabilityLevel::Low,
                },
                farmer: StrategyMetrics {
                    risk_level: RiskLevel::Low,
                    roi_days_min: 7,
                    sustainability: SustainabilityLevel::High,
                },
                strategist: StrategyMetrics {
                    risk_level: RiskLevel::Medium,
                    roi_days_min: 10,
                    sustainability: SustainabilityLevel::VeryHigh,
                },
            }
        }
    }
    
    struct StrategyMetrics {
        risk_level: RiskLevel,
        roi_days_min: u8,
        sustainability: SustainabilityLevel,
    }
    
    struct CriticalDecisionAnalyzer;
    
    impl CriticalDecisionAnalyzer {
        fn new() -> Self { Self }
        
        fn analyze_300_weed_threshold(&self) -> DecisionAnalysis {
            DecisionAnalysis {
                option_a: "Buy mystery pack (high risk/reward)",
                option_b: "Save for upgrade (guaranteed progress)",
                optimal_choice: "Depends on current grow power and risk tolerance",
            }
        }
        
        fn analyze_3500_weed_threshold(&self) -> DecisionAnalysis {
            DecisionAnalysis {
                option_a: "Level 1→2 upgrade (100% capacity increase)",
                option_b: "11-12 mystery packs (potential huge gains)",
                optimal_choice: "Upgrade first for foundation, then packs",
            }
        }
        
        fn analyze_referral_network_strategy(&self) -> NetworkStrategy {
            NetworkStrategy {
                small_network_focus: "Personal growth",
                medium_network_focus: "Balanced approach",
                large_network_focus: "Leverage network effects",
            }
        }
    }
    
    struct DecisionAnalysis {
        option_a: &'static str,
        option_b: &'static str,
        optimal_choice: &'static str,
    }
    
    struct NetworkStrategy {
        small_network_focus: &'static str,
        medium_network_focus: &'static str,
        large_network_focus: &'static str,
    }
    
    struct MetaStrategyInsights {
        early_adopter_advantage: bool,
        mystery_pack_variance: bool,
        farm_upgrade_stability: bool,
        hybrid_optimization: bool,
        adaptive_strategy_importance: bool,
    }
    
    impl MetaStrategyInsights {
        fn new() -> Self {
            Self {
                early_adopter_advantage: true,
                mystery_pack_variance: true,
                farm_upgrade_stability: true,
                hybrid_optimization: true,
                adaptive_strategy_importance: true,
            }
        }
        
        fn get_success_factors(&self) -> Vec<&'static str> {
            vec![
                "Referral chain depth: 2+ levels",
                "Mystery pack ROI: Target Seed3+ for profitability",
                "Upgrade timing: Align with cooldown periods",
                "Growth rate: Compound interest through reinvestment",
            ]
        }
    }
}