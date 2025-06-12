/// Game constants and configuration values
/// Centralized location for all game balance and system parameters

// ===== ECONOMIC CONSTANTS =====
// ゲーム経済の基盤となる重要な定数群
// 全体的なバランス設計は持続可能な成長とプレイヤー参加促進を目的としている

/// 基本報酬レート（秒あたりWEEDトークン）
/// 全プレイヤーの合計Grow Powerで分配されるため、個人報酬は相対的
/// 設計思想：ベースラインとして1秒で100WEEDを全体で分配
/// プレイヤー増加により個人収益は自動調整される
pub const DEFAULT_BASE_RATE: u64 = 100;

/// デフォルト半減期間隔（7日 = 604,800秒）
/// インフレ抑制とトークン価値維持のための重要メカニズム
/// 設計思想：週次で報酬を半減し、長期的な希少性を確保
/// ビットコインの半減期をモデルとしたデフレ機構
pub const DEFAULT_HALVING_INTERVAL: i64 = 7 * 24 * 60 * 60; // 604,800秒

/// 農場スペース購入コスト（0.5 SOL = 500,000,000 lamports）
/// ゲーム参加の初期投資額、経済圏への参入障壁として機能
/// 設計思想：適度な参入コストでスパム防止と真剣なプレイヤー獲得
/// SOL価格変動を考慮し、管理者による調整可能
pub const FARM_SPACE_COST_SOL: u64 = 500_000_000;

/// ミステリーシードパック購入コスト（300 WEED、6桁精度）
/// ゲーム内通貨の主要な消費先、経済循環の要
/// 設計思想：期待値を考慮した適正価格設定
/// 平均的なプレイヤーが1-2日で購入可能な価格帯
pub const SEED_PACK_COST: u64 = 300 * 1_000_000;

/// 取引手数料率（2%）
/// プロトコル収益源とトークン流動性確保のバランス
/// 設計思想：過度な投機を抑制しつつ、取引を阻害しない水準
/// DEXの標準的な手数料率を参考に設定
pub const TRADING_FEE_PERCENTAGE: u8 = 2;

/// ユーザーあたり最大招待上限数
/// 招待システムの悪用防止と有機的成長促進
/// 設計思想：質の高い招待を促し、スパム的招待を抑制
/// 管理者による個別拡張が可能
pub const MAX_INVITE_LIMIT: u8 = 5;

/// 総WEED供給量（1億2千万トークン、6桁精度）
/// 固定供給上限によるインフレ防止とトークン価値保護
/// 設計思想：長期的な希少性確保と経済安定性の両立
/// 報酬分配、取引手数料、初期配布を含む全供給量
/// 半減期メカニズムにより実際の流通量はさらに制限される
pub const TOTAL_WEED_SUPPLY: u64 = 120_000_000 * 1_000_000;

// ===== FARM SPACE CONSTANTS =====
// 農場システムの成長とアップグレード経済設計

/// 農場アップグレードクールダウン（24時間 = 86,400秒）
/// ユーザー影響：アップグレード開始から完了まで24時間待機必須
/// 設計思想：即時強化を防ぎ、計画的なプレイと継続参加を促進
/// 戦略的要素：タイミングを考慮したアップグレード計画が重要
pub const UPGRADE_COOLDOWN: i64 = 24 * 60 * 60;

/// 農場レベル別容量（種植え可能数）
/// レベル1:4, レベル2:8, レベル3:12, レベル4:16, レベル5:20
/// 設計思想：線形成長でシンプルな理解と予測可能性を確保
/// 戦略的要素：高レベル種の効率的配置が重要になる
pub const FARM_CAPACITIES: [u8; 5] = [4, 8, 12, 16, 20];

/// 農場アップグレードコスト（WEEDトークン、6桁精度）
/// 設計思想：指数的コスト増加で初期は手軽、後期は戦略的投資
/// 経済バランス：種パック購入との機会コスト比較が重要
/// ゲーム進行：レベル2-3が中級者の壁、レベル4-5は上級者向け
pub const UPGRADE_COSTS: [u64; 4] = [
    3_500 * 1_000_000,   // Level 1→2: 3,500 WEED (約12日分の報酬)
    18_000 * 1_000_000,  // Level 2→3: 18,000 WEED (約60日分の報酬)
    20_000 * 1_000_000,  // Level 3→4: 20,000 WEED (約67日分の報酬)
    25_000 * 1_000_000,  // Level 4→5: 25,000 WEED (約83日分の報酬)
];

// ===== SEED SYSTEM CONSTANTS =====
// ミステリーシードパック系統の確率設計とゲーム経済への影響

/// ユーザーあたり最大種保存数（レント最適化のための実用的容量）
/// ユーザー影響：ストレージ上限により戦略的な種管理が必要
/// 設計思想：無制限保存を防ぎ、アカウントレント負担を制限
pub const MAX_SEEDS_PER_USER: usize = 1_000;

/// 最大種パック購入数量（一括購入上限）
/// ユーザー影響：大量購入でコスト効率化、ただし上限による制限
/// 設計思想：極端な大量購入によるネットワーク負荷を防止
pub const MAX_SEED_PACK_QUANTITY: u8 = 100;

/// 最小種パック購入数量
/// ユーザー影響：最低1パックから購入可能で参入障壁を軽減
pub const MIN_SEED_PACK_QUANTITY: u8 = 1;

/// 種タイプ別グロウパワー値
/// 設計思想：レアリティと性能の相関関係でゲーム内階層を明確化
/// 経済バランス：高レア種の圧倒的優位性で長期目標を提供
/// 期待値考慮：平均グロウパワーは約300（後述の期待値計算参照）
pub const SEED_GROW_POWERS: [u64; 9] = [
    100,    // Seed1: 基本種（42.23%）
    180,    // Seed2: 一般種（24.44%）
    420,    // Seed3: 良質種（13.33%）
    720,    // Seed4: 上質種（8.33%）
    1000,   // Seed5: 稀少種（5.56%）
    5000,   // Seed6: 希少種（3.33%）
    15000,  // Seed7: 激レア種（1.33%）
    30000,  // Seed8: 超レア種（0.89%）
    60000,  // Seed9: 伝説種（0.56%）
];

/// 種確率閾値（10,000分率による精密制御）
/// 数学的根拠：累積確率分布による効率的な乱数変換
/// Pyth Entropyの真正乱数を0-9999範囲にマッピング
/// 透明性：オンチェーンで完全に検証可能な確率制御
pub const SEED_PROBABILITY_THRESHOLDS: [u16; 9] = [
    4222,  // Seed1: 42.23% (0-4221)
    6666,  // Seed2: 24.44% (4222-6665) 
    7999,  // Seed3: 13.33% (6666-7998)
    8832,  // Seed4: 8.33% (7999-8831)
    9388,  // Seed5: 5.56% (8832-9387)
    9721,  // Seed6: 3.33% (9388-9720)
    9854,  // Seed7: 1.33% (9721-9853)
    9943,  // Seed8: 0.89% (9854-9942)
    10000, // Seed9: 0.56% (9943-9999)
];

/// 種確率パーセンテージ（表示・文書化用）
/// ユーザー体験：分かりやすい%表示でドロップ率を明示
pub const SEED_PROBABILITIES: [f32; 9] = [
    42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56
];

// ===== 期待値計算とゲーム経済分析 =====

/// シードパック期待グロウパワー値計算（数学的根拠）
/// 計算式：Σ(確率 × グロウパワー) for all seed types
/// 
/// 詳細計算：
/// Seed1: 0.4223 × 100 = 42.23
/// Seed2: 0.2444 × 180 = 43.99
/// Seed3: 0.1333 × 420 = 55.99
/// Seed4: 0.0833 × 720 = 59.98
/// Seed5: 0.0556 × 1000 = 55.60
/// Seed6: 0.0333 × 5000 = 166.50
/// Seed7: 0.0133 × 15000 = 199.50
/// Seed8: 0.0089 × 30000 = 267.00
/// Seed9: 0.0056 × 60000 = 336.00
/// 
/// 合計期待値：1,226.79 グロウパワー
/// 
/// 経済分析：
/// - コスト：300 WEED per pack
/// - 期待グロウパワー：1,226.79
/// - 効率：約4.09 GP/WEED
/// - ROI分析：高レア種が期待値の大部分を占める構造
pub const EXPECTED_GROW_POWER_PER_PACK: f32 = 1226.79;

// ===== REFERRAL SYSTEM CONSTANTS =====

/// Level 1 referral reward percentage (10%)
pub const LEVEL1_REFERRAL_PERCENTAGE: u8 = 10;

/// Level 2 referral reward percentage (5%)
pub const LEVEL2_REFERRAL_PERCENTAGE: u8 = 5;

/// Maximum referral chain depth
pub const MAX_REFERRAL_DEPTH: u8 = 2;

// ===== VALIDATION CONSTANTS =====

/// Minimum quantity for operations
pub const MIN_QUANTITY: u8 = 1;

/// Maximum invite code length
pub const INVITE_CODE_LENGTH: usize = 8;

/// Minimum time interval for reward claims (prevent spam)
pub const MIN_CLAIM_INTERVAL: i64 = 1; // 1 second

/// Maximum batch transfer size
pub const MAX_BATCH_TRANSFER_SIZE: usize = 100;

/// Maximum batch discard size
pub const MAX_BATCH_DISCARD_SIZE: usize = 100;

/// Time tolerance for future time validation (seconds)
pub const TIME_TOLERANCE: i64 = 30;

// ===== PDA SEEDS =====
// Program Derived Address（プログラム派生アドレス）の種子定数
// セキュリティ設計：決定論的アドレス生成による一意性とアクセス制御

/// PDA種子の決定論的アドレス生成
/// セキュリティ原則：各アカウントタイプが一意のアドレス空間を持つ
/// 衝突防止：異なるシード文字列による完全な分離
pub mod seeds {
    /// グローバル設定PDAの種子
    /// 用途：システム全体の設定（管理者のみアクセス）
    /// 一意性：プログラム内で単一のConfigアカウント保証
    pub const CONFIG: &[u8] = b"config";
    
    /// ユーザー状態PDAの種子プレフィックス
    /// 用途：["user", user_pubkey] でユーザー固有アカウント生成
    /// セキュリティ：署名者のpubkeyによる所有権保証
    pub const USER: &[u8] = b"user";
    
    /// 農場スペースPDAの種子プレフィックス
    /// 用途：["farm_space", user_pubkey] でユーザー固有農場生成
    /// 設計：1ユーザー1農場の制限をPDAレベルで保証
    pub const FARM_SPACE: &[u8] = b"farm_space";
    
    /// 報酬ミントPDAの種子
    /// 用途：WEEDトークンのミントアカウント
    /// セキュリティ：プログラムが完全制御する中央ミント
    pub const REWARD_MINT: &[u8] = b"reward_mint";
    
    /// ミント権限PDAの種子
    /// 用途：報酬ミントの権限制御
    /// セキュリティ：人間によるアクセス不可、プログラムのみ制御
    pub const MINT_AUTHORITY: &[u8] = b"mint_authority";
    
    /// Global stats PDA seed
    pub const GLOBAL_STATS: &[u8] = b"global_stats";
    
    /// Fee pool PDA seed
    pub const FEE_POOL: &[u8] = b"fee_pool";
    
    /// Seed storage PDA seed prefix
    pub const SEED_STORAGE: &[u8] = b"seed_storage";
    
    /// Seed PDA seed prefix
    pub const SEED: &[u8] = b"seed";
    
    /// Seed pack PDA seed prefix
    pub const SEED_PACK: &[u8] = b"seed_pack";
    
    /// Invite code PDA seed prefix
    pub const INVITE_CODE: &[u8] = b"invite_code";
    
    /// Reward account PDA seed prefix
    pub const REWARD_ACCOUNT: &[u8] = b"reward_account";
    
    /// Meteora config PDA seed
    pub const METEORA_CONFIG: &[u8] = b"meteora_config";
    
    /// Pyth entropy request PDA seed prefix
    pub const ENTROPY_REQUEST: &[u8] = b"entropy_request";
}

// ===== TOKEN CONSTANTS =====

/// WEED token decimals
pub const WEED_DECIMALS: u8 = 6;

/// WEED token symbol
pub const WEED_SYMBOL: &str = "WEED";

/// WEED token name
pub const WEED_NAME: &str = "Weed Token";

// ===== TIME CONSTANTS =====
// ゲーム内時間システムの基盤定数
// Solanaブロックチェーンの正確なタイムスタンプを活用

/// 1時間の秒数（3,600秒）
/// 報酬計算、クールダウン、統計集計の基本単位
/// ユーザー影響：報酬は毎秒計算されるため、時間単位での収益予測が可能
pub const SECONDS_PER_HOUR: i64 = 60 * 60;

/// 1日の秒数（86,400秒）
/// アップグレードクールダウン、日次統計、レポートサイクルに使用
/// ユーザー影響：農場アップグレードは24時間待機が必要
/// ゲームプレイ設計：毎日のログインインセンティブ時間枠
pub const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

/// 1週間の秒数（604,800秒）
/// 半減期計算、週次イベント、長期統計に使用
/// ユーザー影響：デフォルト半減期サイクルは7日ごと
/// 経済設計：週次でトークン発行量が半減し、希少性が増大
pub const SECONDS_PER_WEEK: i64 = 7 * SECONDS_PER_DAY;

// ===== PRECISION AND CALCULATION CONSTANTS =====

/// Calculation precision divisor for reward calculations
pub const CALCULATION_PRECISION: u64 = 1000;

/// Basis points for percentage calculations (1/100 of a percent)
pub const BASIS_POINTS_TOTAL: u16 = 10000;

/// Percentage to basis points multiplier
pub const PERCENTAGE_TO_BASIS_POINTS: u16 = 100;

// ===== CALCULATION HELPERS =====

/// Helper functions for common calculations
impl crate::state::FarmSpace {
    /// Get capacity for a given level using constants
    pub fn capacity_for_level(level: u8) -> u8 {
        if level == 0 || level > FARM_CAPACITIES.len() as u8 {
            return FARM_CAPACITIES[FARM_CAPACITIES.len() - 1]; // Max capacity
        }
        FARM_CAPACITIES[(level - 1) as usize]
    }
    
    /// Get upgrade cost for a given level using constants
    pub fn upgrade_cost_for_level(level: u8) -> Option<u64> {
        if level == 0 || level > UPGRADE_COSTS.len() as u8 {
            return None;
        }
        Some(UPGRADE_COSTS[(level - 1) as usize])
    }
}

/// Helper functions for seed type calculations
impl crate::state::SeedType {
    /// Get grow power using constants
    pub fn grow_power_from_constants(&self) -> u64 {
        SEED_GROW_POWERS[*self as usize]
    }
    
    /// Get probability percentage using constants
    pub fn probability_from_constants(&self) -> f32 {
        SEED_PROBABILITIES[*self as usize]
    }
    
    /// Convert random value to seed type using constants
    pub fn from_random_with_constants(random: u64) -> Self {
        let value = (random % 10000) as u16;
        
        for (i, &threshold) in SEED_PROBABILITY_THRESHOLDS.iter().enumerate() {
            if value < threshold {
                return unsafe { std::mem::transmute(i as u8) };
            }
        }
        
        // Fallback to highest rarity
        Self::Seed9
    }
}

// ===== VALIDATION HELPERS =====

/// Validate quantity is within acceptable range
pub fn validate_quantity(quantity: u8) -> bool {
    quantity >= MIN_QUANTITY && quantity <= MAX_SEED_PACK_QUANTITY
}

/// Validate farm level
pub fn validate_farm_level(level: u8) -> bool {
    level >= 1 && level <= 5
}

/// Validate invite code format
pub fn validate_invite_code(code: &[u8; 8]) -> bool {
    code.iter().all(|&b| {
        (b >= b'A' && b <= b'Z') || 
        (b >= b'a' && b <= b'z') || 
        (b >= b'0' && b <= b'9')
    })
}

/// Calculate referral rewards using constants
pub fn calculate_referral_rewards_with_constants(base_amount: u64) -> (u64, u64) {
    let level1 = base_amount * LEVEL1_REFERRAL_PERCENTAGE as u64 / PERCENTAGE_TO_BASIS_POINTS as u64;
    let level2 = base_amount * LEVEL2_REFERRAL_PERCENTAGE as u64 / PERCENTAGE_TO_BASIS_POINTS as u64;
    (level1, level2)
}

/// Calculate amount from basis points
pub fn calculate_amount_from_basis_points(base_amount: u64, basis_points: u16) -> u64 {
    base_amount * basis_points as u64 / BASIS_POINTS_TOTAL as u64
}

/// Calculate percentage from basis points
pub fn basis_points_to_percentage(basis_points: u16) -> f32 {
    basis_points as f32 / PERCENTAGE_TO_BASIS_POINTS as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_consistency() {
        // Verify array lengths match
        assert_eq!(SEED_GROW_POWERS.len(), 9);
        assert_eq!(SEED_PROBABILITY_THRESHOLDS.len(), 9);
        assert_eq!(SEED_PROBABILITIES.len(), 9);
        assert_eq!(FARM_CAPACITIES.len(), 5);
        assert_eq!(UPGRADE_COSTS.len(), 4);
        
        // Verify probability thresholds are in ascending order
        for i in 1..SEED_PROBABILITY_THRESHOLDS.len() {
            assert!(SEED_PROBABILITY_THRESHOLDS[i] > SEED_PROBABILITY_THRESHOLDS[i-1]);
        }
        
        // Verify last threshold is 10000 (100%)
        assert_eq!(SEED_PROBABILITY_THRESHOLDS[8], 10000);
        
        // Verify farm capacities are in ascending order
        for i in 1..FARM_CAPACITIES.len() {
            assert!(FARM_CAPACITIES[i] > FARM_CAPACITIES[i-1]);
        }
    }
    
    #[test]
    fn test_validation_helpers() {
        // Test quantity validation
        assert!(!validate_quantity(0));
        assert!(validate_quantity(1));
        assert!(validate_quantity(50));
        assert!(validate_quantity(100));
        assert!(!validate_quantity(101));
        
        // Test farm level validation
        assert!(!validate_farm_level(0));
        assert!(validate_farm_level(1));
        assert!(validate_farm_level(5));
        assert!(!validate_farm_level(6));
        
        // Test invite code validation
        assert!(validate_invite_code(b"ABCD1234"));
        assert!(validate_invite_code(b"abcd1234"));
        assert!(validate_invite_code(b"AbCd1234"));
        assert!(!validate_invite_code(b"ABC@1234")); // Contains invalid character
        assert!(!validate_invite_code(b"ABC 1234")); // Contains space
    }
    
    #[test]
    fn test_referral_calculations() {
        let (level1, level2) = calculate_referral_rewards_with_constants(1000);
        assert_eq!(level1, 100); // 10% of 1000
        assert_eq!(level2, 50);  // 5% of 1000
        
        let (level1, level2) = calculate_referral_rewards_with_constants(0);
        assert_eq!(level1, 0);
        assert_eq!(level2, 0);
    }
}