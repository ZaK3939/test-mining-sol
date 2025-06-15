/// Game constants and configuration values
/// Centralized location for all game balance and system parameters

// ===== ECONOMIC CONSTANTS =====
// ゲーム経済の基盤となる重要な定数群
// 全体的なバランス設計は持続可能な成長とプレイヤー参加促進を目的としている

/// 基本報酬レート（秒あたりWEEDトークン）
/// 全プレイヤーの合計Grow Powerで分配されるため、個人報酬は相対的
/// 設計思想：ベースラインとして1秒で200WEEDを全体で分配
/// プレイヤー増加により個人収益は自動調整される
pub const DEFAULT_BASE_RATE: u64 = 200;

/// デフォルト半減期間隔（7日）
/// インフレ抑制とトークン価値維持のための重要メカニズム
/// 設計思想：毎週報酬を半減し、段階的に希少性を確保
/// ビットコインの半減期をモデルとしたデフレ機構
pub const DEFAULT_HALVING_INTERVAL: i64 = 604800; // 7日（604,800秒）

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

/// 総WEED供給量（2億4千万トークン、6桁精度）
/// 固定供給上限によるインフレ防止とトークン価値保護
/// 設計思想：長期的な希少性確保と経済安定性の両立
/// 報酬分配、取引手数料、初期配布を含む全供給量
/// 半減期メカニズムにより実際の流通量はさらに制限される
pub const TOTAL_WEED_SUPPLY: u64 = 240_000_000 * 1_000_000;

// ===== FARM SPACE CONSTANTS =====
// 農場システムの成長とアップグレード経済設計


/// 農場レベル別容量（種植え可能数）最大レベル10まで拡張可能、最大100以下
/// 現在解放：レベル1-5（レベル6-10は将来実装予定）
/// レベル1:4, レベル2:6, レベル3:10, レベル4:16, レベル5:25, レベル6:35, レベル7:50, レベル8:65, レベル9:80, レベル10:100
/// 設計思想：段階的成長でバランスの取れた進行を確保、最大100以下に制限
/// 戦略的要素：高レベル種の効率的配置が重要になる
pub const FARM_CAPACITIES: [u8; 10] = [4, 6, 10, 16, 25, 35, 50, 65, 80, 100];

/// 農場自動アップグレード条件（累積シードパック購入数）
/// 現在解放：レベル1-5（レベル6-10は将来実装予定）
/// 設計思想：パック購入数に応じた自動アップグレードシステム
/// ゲーム進行：より多くのパックを購入することで自動的に農場が拡張される
/// レベル1: 0, レベル2: 30, レベル3: 100, レベル4: 300, レベル5: 500, 
/// レベル6: 800, レベル7: 1200, レベル8: 1800, レベル9: 2500, レベル10: 3500
pub const FARM_UPGRADE_THRESHOLDS: [u32; 10] = [0, 30, 100, 300, 500, 800, 1200, 1800, 2500, 3500];

/// 従来のアップグレードコスト（後方互換性のため保持、現在は未使用）
/// 注意：新システムでは累積パック購入数による自動アップグレードを使用
pub const LEGACY_UPGRADE_COSTS: [u64; 4] = [
    3_500 * 1_000_000,   // Level 1→2: 3,500 WEED (廃止)
    18_000 * 1_000_000,  // Level 2→3: 18,000 WEED (廃止)
    20_000 * 1_000_000,  // Level 3→4: 20,000 WEED (廃止)
    25_000 * 1_000_000,  // Level 4→5: 25,000 WEED (廃止)
];

// ===== SEED SYSTEM CONSTANTS =====
// ミステリーシードパック系統の確率設計とゲーム経済への影響

/// ユーザーあたり最大種保存数（総合計）
/// ユーザー影響：ストレージ上限により戦略的な種管理が必要
/// 設計思想：大容量化により本格的なコレクションを可能にする
pub const MAX_SEEDS_PER_USER: usize = 2_000;

/// 各種類あたり最大保有数（種類別制限）
/// ユーザー影響：特定の種類のため込みを防止
/// 設計思想：バランスの取れたコレクションを促進する
pub const MAX_SEEDS_PER_TYPE: u16 = 100;

/// 最大種パック購入数量（一括購入上限）
/// ユーザー影響：大量購入でコスト効率化、ただし上限による制限
/// 設計思想：極端な大量購入によるネットワーク負荷を防止
pub const MAX_SEED_PACK_QUANTITY: u8 = 100;

/// 最小種パック購入数量
/// ユーザー影響：最低1パックから購入可能で参入障壁を軽減
pub const MIN_SEED_PACK_QUANTITY: u8 = 1;

/// 種タイプ別グロウパワー値（確率テーブル1: 6種類）
/// 設計思想：シンプルな6段階レアリティシステム
/// 経済バランス：バランスの取れた成長カーブ
/// 期待値計算：43%×100 + 25%×180 + 14%×420 + 9%×720 + 6%×1000 + 3%×5000 = 499 GP
pub const SEED_GROW_POWERS: [u64; 6] = [
    100,    // OG Kush: 基本種（43%）
    180,    // Blue Dream: 一般種（25%）
    420,    // Sour Diesel: 良質種（14%）
    720,    // Girl Scout Cookies: 上質種（9%）
    1000,   // Gorilla Glue: 稀少種（6%）
    5000,   // Skywalker Kush: 希少種（3%）
];

/// 種確率閾値（10,000分率による精密制御）
/// 数学的根拠：累積確率分布による効率的な乱数変換
/// Pyth Entropyの真正乱数を0-9999範囲にマッピング
/// 透明性：オンチェーンで完全に検証可能な確率制御
pub const SEED_PROBABILITY_THRESHOLDS: [u16; 6] = [
    4300,  // OG Kush: 43% (0-4299)
    6800,  // Blue Dream: 25% (4300-6799)
    8200,  // Sour Diesel: 14% (6800-8199)
    9100,  // Girl Scout Cookies: 9% (8200-9099)
    9700,  // Gorilla Glue: 6% (9100-9699)
    10000, // Skywalker Kush: 3% (9700-9999)
];

/// 種確率パーセンテージ（表示・文書化用）
/// ユーザー体験：分かりやすい%表示でドロップ率を明示
pub const SEED_PROBABILITIES: [f32; 6] = [
    43.0, 25.0, 14.0, 9.0, 6.0, 3.0
];

// ===== 期待値計算とゲーム経済分析 =====

/// シードパック期待グロウパワー値計算（確率テーブル1）
/// 計算式：Σ(確率 × グロウパワー) for all seed types
/// 
/// 詳細計算：
/// OG Kush: 0.43 × 100 = 43.0
/// Blue Dream: 0.25 × 180 = 45.0
/// Sour Diesel: 0.14 × 420 = 58.8
/// Girl Scout Cookies: 0.09 × 720 = 64.8
/// Gorilla Glue: 0.06 × 1000 = 60.0
/// Skywalker Kush: 0.03 × 5000 = 150.0
/// 
/// 合計期待値：421.6 グロウパワー
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

/// Maximum invite code length (increased to 12 to reduce collision probability)
pub const INVITE_CODE_LENGTH: usize = 12;

/// Minimum time interval for reward claims (prevent spam)
pub const MIN_CLAIM_INTERVAL: i64 = 1; // 1 second

/// Maximum batch transfer size
pub const MAX_BATCH_TRANSFER_SIZE: usize = 100;

/// Maximum batch discard size
pub const MAX_BATCH_DISCARD_SIZE: usize = 100;

/// Maximum batch plant size
pub const MAX_BATCH_PLANT_SIZE: usize = 25;

/// Maximum batch remove size
pub const MAX_BATCH_REMOVE_SIZE: usize = 25;

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
/// 週次イベント、長期統計に使用
/// ユーザー影響：半減期は7日ごとの標準サイクル
/// 経済設計：毎週トークン発行量が半減し、段階的に希少性が増大
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
    
    /// Get upgrade cost for a given level using constants (legacy)
    pub fn upgrade_cost_for_level(level: u8) -> Option<u64> {
        if level == 0 || level > LEGACY_UPGRADE_COSTS.len() as u8 {
            return None;
        }
        Some(LEGACY_UPGRADE_COSTS[(level - 1) as usize])
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
        
        // Fallback to highest rarity (6th seed type in 0-based indexing is index 5)
        unsafe { std::mem::transmute(5u8) }
    }
}

// ===== VALIDATION HELPERS =====

/// Validate quantity is within acceptable range
pub fn validate_quantity(quantity: u8) -> bool {
    quantity >= MIN_QUANTITY && quantity <= MAX_SEED_PACK_QUANTITY
}

/// Validate farm level (currently level 5, future expansion to 10)
pub fn validate_farm_level(level: u8) -> bool {
    level >= 1 && level <= 5
}

/// Validate invite code format
pub fn validate_invite_code(code: &[u8; 12]) -> bool {
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
        // Verify array lengths match (updated to reflect actual array sizes)
        assert_eq!(SEED_GROW_POWERS.len(), 6);
        assert_eq!(SEED_PROBABILITY_THRESHOLDS.len(), 6);
        assert_eq!(SEED_PROBABILITIES.len(), 6);
        assert_eq!(FARM_CAPACITIES.len(), 10);
        assert_eq!(FARM_UPGRADE_THRESHOLDS.len(), 10);
        assert_eq!(LEGACY_UPGRADE_COSTS.len(), 4);
        
        // Verify probability thresholds are in ascending order
        for i in 1..SEED_PROBABILITY_THRESHOLDS.len() {
            assert!(SEED_PROBABILITY_THRESHOLDS[i] > SEED_PROBABILITY_THRESHOLDS[i-1]);
        }
        
        // Verify last threshold is 10000 (100%)
        assert_eq!(SEED_PROBABILITY_THRESHOLDS[SEED_PROBABILITY_THRESHOLDS.len()-1], 10000);
        
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
        
        // Test farm level validation (currently level 5)
        assert!(!validate_farm_level(0));
        assert!(validate_farm_level(1));
        assert!(validate_farm_level(5));
        assert!(!validate_farm_level(6));
        
        // Test invite code validation (12 characters)
        assert!(validate_invite_code(b"ABCD12345678"));
        assert!(validate_invite_code(b"abcd12345678"));
        assert!(validate_invite_code(b"AbCd12345678"));
        assert!(!validate_invite_code(b"ABC@12345678")); // Contains invalid character
        assert!(!validate_invite_code(b"ABC 12345678")); // Contains space
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