// Anchorフレームワークの基本的な型や機能をインポート
// prelude::*は、よく使う型（Account, Signer, Program等）を一括でインポート
#![allow(deprecated)] // Suppress Anchor framework's internal deprecation warnings
use anchor_lang::prelude::*;

// SPLトークンプログラムとの連携に必要な機能をインポート
// 個別の操作は各モジュールで実装される

// このプログラムの一意のIDを宣言
// Solana上でプログラムを識別するために使用される
// anchor buildコマンドで自動生成され、target/deploy/に保存される
declare_id!("GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc");

// 核心モジュールの宣言
// Rustのモジュールシステムを使って、コードを論理的に分割
pub mod state;        // アカウントの構造体定義
pub mod instructions; // 各命令のコンテキスト（必要なアカウント）定義
pub mod error;        // カスタムエラー定義
pub mod utils;        // ヘルパー関数

// 新しい機能モジュール（より良い分離のため）
pub mod constants;      // ゲーム定数
pub mod validation;     // バリデーション機能
pub mod economics;      // 経済計算
pub mod error_handling; // エラーハンドリング

// テストモジュール（開発時のみ）
#[cfg(test)]
pub mod tests;

// Note: Advanced test modules removed - using standard /tests/ directory instead

// 他のモジュールから必要な要素をインポート
use instructions::*;  // すべての命令コンテキストをインポート

// #[program]属性は、このモジュールがSolanaプログラムのエントリーポイントであることを示す
// Anchorがこの属性を見つけると、各関数を呼び出し可能な命令として処理する
#[program]
pub mod farm_game {
    use super::*;

    // ===== ADMIN INSTRUCTIONS =====

    /// システム全体の設定を初期化
    /// 管理者のみが実行可能で、ゲーム経済の基本パラメータを設定
    /// 
    /// # Parameters
    /// * `base_rate` - 基本報酬レート（デフォルト: 100 WEED/秒）
    /// * `halving_interval` - 半減期間隔（デフォルト: 7日）
    /// * `treasury` - 手数料収集用ウォレット
    /// 
    /// # Security
    /// - admin署名必須
    /// - 一度のみ実行可能（PDAの重複初期化防止）
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        base_rate: Option<u64>,
        halving_interval: Option<i64>,
        treasury: Pubkey,
        protocol_referral_address: Option<Pubkey>,
    ) -> Result<()> {
        instructions::admin::initialize_config(ctx, base_rate, halving_interval, treasury, protocol_referral_address)
    }

    /// $WEEDトークンのミントアカウントを作成（SPL Token 2022 + Transfer Fee Extension）
    /// ゲーム内通貨として使用される報酬トークンを初期化
    /// 
    /// # 機能
    /// - PDAによるミント権限管理（セキュリティ向上）
    /// - SPL Token 2022のTransfer Fee Extension（2%手数料）
    /// - Metaplexメタデータ作成（トークン情報表示用）
    /// - 6桁精度での発行設定
    /// - 自動手数料回収（treasury宛）
    /// 
    /// # Transfer Fee Extension
    /// - 手数料: 2.00% (200 basis points)
    /// - 最大手数料: 1000 WEED
    /// - 手数料回収権限: treasury
    /// - 設定変更権限: mint_authority PDA
    /// 
    /// # Security
    /// - mint_authorityはPDAで制御
    /// - transfer_fee_config_authorityもPDAで制御
    /// - admin署名必須
    pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
        instructions::admin::create_reward_mint(ctx)
    }

    /// Initialize global statistics
    pub fn initialize_global_stats(ctx: Context<InitializeGlobalStats>) -> Result<()> {
        instructions::admin::initialize_global_stats(ctx)
    }

    /// Initialize fee pool
    pub fn initialize_fee_pool(ctx: Context<InitializeFeePool>, treasury_address: Pubkey) -> Result<()> {
        instructions::admin::initialize_fee_pool(ctx, treasury_address)
    }
    
    /// Update system configuration (admin only)
    /// Allows admin to modify various system parameters including operator address
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_operator: Option<Pubkey>,
        new_base_rate: Option<u64>,
        new_halving_interval: Option<i64>,
        new_treasury: Option<Pubkey>,
        new_max_invite_limit: Option<u8>,
    ) -> Result<()> {
        instructions::admin::update_config(ctx, new_operator, new_base_rate, new_halving_interval, new_treasury, new_max_invite_limit)
    }
    
    /// Update seed pack cost (admin only)
    /// Allows admin to dynamically adjust the WEED price for seed packs
    /// 
    /// # Parameters
    /// * `new_seed_pack_cost` - New cost in WEED tokens (with 6 decimals)
    /// 
    /// # Example
    /// - 300 WEED = 300_000_000 (with 6 decimals)
    /// - 500 WEED = 500_000_000 (with 6 decimals)
    /// 
    /// # Security
    /// - Admin signature required
    /// - Prevents zero cost (must be > 0)
    /// - Prevents excessively high costs (max 10,000 WEED)
    pub fn update_seed_pack_cost(
        ctx: Context<UpdateSeedPackCost>,
        new_seed_pack_cost: u64,
    ) -> Result<()> {
        instructions::admin::update_seed_pack_cost(ctx, new_seed_pack_cost)
    }
    
    /// Update farm space cost (admin only)
    /// Allows admin to dynamically adjust the SOL price for farm space purchases
    /// 
    /// # Parameters
    /// * `new_farm_space_cost_sol` - New cost in lamports (SOL)
    /// 
    /// # Example
    /// - 0.5 SOL = 500_000_000 lamports
    /// - 1.0 SOL = 1_000_000_000 lamports
    /// 
    /// # Security
    /// - Admin signature required
    /// - Prevents zero cost (must be > 0)
    /// - Prevents excessively high costs (max 10 SOL)
    pub fn update_farm_space_cost(
        ctx: Context<UpdateFarmSpaceCost>,
        new_farm_space_cost_sol: u64,
    ) -> Result<()> {
        instructions::admin::update_farm_space_cost(ctx, new_farm_space_cost_sol)
    }
    
    /// Reveal a new seed type (admin only)
    /// Makes a previously hidden seed type visible to users with its values
    /// 
    /// # Parameters
    /// * `seed_index` - Index of the seed type to reveal (0-15)
    /// * `grow_power` - Grow power value for this seed type
    /// * `probability_percentage` - Probability percentage (0.0-100.0)
    /// 
    /// # Example
    /// - seed_index: 8 (for Seed9, the first secret seed)
    /// - grow_power: 60000
    /// - probability_percentage: 1.5
    /// 
    /// # Security
    /// - Admin signature required
    /// - Seed must not already be revealed
    /// - Values must be within reasonable ranges
    pub fn reveal_seed(
        ctx: Context<RevealSeed>,
        seed_index: u8,
        grow_power: u64,
        probability_percentage: f32,
    ) -> Result<()> {
        instructions::admin::reveal_seed(ctx, seed_index, grow_power, probability_percentage)
    }
    
    /// Update values for an already revealed seed type (admin only)
    /// Allows changing grow power and probability for existing revealed seeds
    /// 
    /// # Parameters
    /// * `seed_index` - Index of the seed type to update (0-15)
    /// * `grow_power` - New grow power value for this seed type
    /// * `probability_percentage` - New probability percentage (0.0-100.0)
    /// 
    /// # Example
    /// - seed_index: 0 (for Seed1)
    /// - grow_power: 150 (increased from 100)
    /// - probability_percentage: 30.0 (same as before)
    /// 
    /// # Security
    /// - Admin signature required
    /// - Seed must already be revealed
    /// - Values must be within reasonable ranges
    pub fn update_seed_values(
        ctx: Context<UpdateSeedValues>,
        seed_index: u8,
        grow_power: u64,
        probability_percentage: f32,
    ) -> Result<()> {
        instructions::admin::update_seed_values(ctx, seed_index, grow_power, probability_percentage)
    }
    
    /// Initialize probability table with default Table 1 settings
    /// Creates the dynamic probability table for seed generation
    /// 
    /// # Default Settings (Table 1)
    /// - OG Kush (100 GP): 43%
    /// - Blue Dream (180 GP): 25%
    /// - Sour Diesel (420 GP): 14%
    /// - Girl Scout Cookies (720 GP): 9%
    /// - Gorilla Glue (1000 GP): 6%
    /// - Skywalker Kush (5000 GP): 3%
    /// 
    /// # Security
    /// - Admin signature required
    /// - One-time initialization only
    pub fn initialize_probability_table(ctx: Context<InitializeProbabilityTable>) -> Result<()> {
        instructions::admin::initialize_probability_table(ctx)
    }
    
    /// Update probability table with new settings (admin only)
    /// Allows runtime modification of seed probabilities without code deployment
    /// 
    /// # Parameters
    /// * `version` - Version number for tracking changes
    /// * `seed_count` - Number of seed types (1-9)
    /// * `grow_powers` - Grow power values for each seed type
    /// * `probability_thresholds` - Cumulative probability thresholds (must end at 10000)
    /// * `probability_percentages` - Human-readable percentage values
    /// * `expected_value` - Calculated expected grow power per pack
    /// * `name` - Table name/description (max 32 chars)
    /// 
    /// # Examples
    /// ## Table 1 (6 seeds):
    /// - grow_powers: [100, 180, 420, 720, 1000, 5000]
    /// - thresholds: [4300, 6800, 8200, 9100, 9700, 10000]
    /// - percentages: [43.0, 25.0, 14.0, 9.0, 6.0, 3.0]
    /// 
    /// ## Table 2 (9 seeds):
    /// - grow_powers: [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000]
    /// - thresholds: [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000]
    /// - percentages: [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56]
    /// 
    /// # Security
    /// - Admin signature required
    /// - Validates probability distribution sums to 100%
    /// - Ensures thresholds are in ascending order
    pub fn update_probability_table(
        ctx: Context<UpdateProbabilityTable>,
        version: u32,
        seed_count: u8,
        grow_powers: Vec<u64>,
        probability_thresholds: Vec<u16>,
        probability_percentages: Vec<f32>,
        expected_value: u64,
        name: String,
    ) -> Result<()> {
        instructions::admin::update_probability_table(
            ctx,
            version,
            seed_count,
            grow_powers,
            probability_thresholds,
            probability_percentages,
            expected_value,
            name,
        )
    }

    // ===== USER MANAGEMENT INSTRUCTIONS =====

    /// ユーザーアカウントの初期化
    /// 運営専用：招待コード抜きでユーザーアカウントを初期化
    /// 通常のユーザーは招待コード経由でのみゲームに参加可能
    /// 
    /// # Admin/Operator Only Access
    /// この機能は config.admin または config.operator のみが実行可能
    /// 
    /// # Parameters
    /// * `referrer` - 紹介者のpubkey（オプション）
    /// 
    /// # 作成されるデータ
    /// - UserState PDA: ユーザーの進行状況
    /// - 初期grow_power: 0（農場購入で増加）
    /// - 紹介関係の記録（多段階報酬システム用）
    pub fn init_user(ctx: Context<InitUser>, referrer: Option<Pubkey>) -> Result<()> {
        instructions::user::init_user(ctx, referrer)
    }

    // ===== FARM MANAGEMENT INSTRUCTIONS =====

    /// 農場スペースの購入（レベル1）
    /// 0.5 SOLで初期農場を購入し、種植えを開始可能にする
    /// 
    /// # 実行内容
    /// - 0.5 SOL → treasuryに送金
    /// - FarmSpace PDA作成（容量4、レベル1）
    /// - 初期Seed1を無料で付与（100 Grow Power）
    /// - グローバル統計の更新
    /// 
    /// # 制約
    /// - ユーザーは1つまで
    /// - UserState初期化済み必須
    pub fn buy_farm_space(ctx: Context<BuyFarmSpace>) -> Result<()> {
        instructions::farm::buy_farm_space(ctx)
    }

    // Note: Manual farm upgrades have been replaced with automatic upgrades
    // Farm spaces now upgrade automatically based on cumulative pack purchases:
    // Level 2: 30 packs, Level 3: 100 packs, Level 4: 300 packs, Level 5: 500 packs
    // See purchase_seed_pack instruction for auto-upgrade implementation


    // ===== REWARD SYSTEM INSTRUCTIONS =====





    
    // ===== REFERRAL REWARD ACCUMULATION SYSTEM =====
    
    /// 紹介報酬蓄積（内部処理用）
    /// 他のユーザーが報酬請求時に自動実行される
    pub fn accumulate_referral_reward(
        ctx: Context<AccumulateReferralReward>,
        reward_amount: u64,
        referral_level: u8,
    ) -> Result<()> {
        instructions::referral::accumulate_referral_reward(ctx, reward_amount, referral_level)
    }
    
    /// 未請求紹介報酬確認（読み取り専用）
    /// UI表示用、請求前の金額確認
    pub fn view_pending_referral_rewards(ctx: Context<ViewPendingReferralRewards>) -> Result<()> {
        instructions::referral::view_pending_referral_rewards(ctx)
    }
    
    /// 統合報酬請求（メイン関数）
    /// 農場報酬と紹介報酬をすべて一度に請求する
    /// 
    /// # 処理フロー
    /// 1. 半減期チェック・適用
    /// 2. 農場報酬計算（比例配分）
    /// 3. 蓄積された紹介報酬請求
    /// 4. 新規紹介報酬分配（L1: 10%, L2: 5%）
    /// 5. すべてのトークンを一括ミント・配布
    /// 
    /// # 統合処理のメリット
    /// - 複数のトランザクションが不要
    /// - ガス効率性向上
    /// - ユーザー体験の向上
    pub fn claim_reward_with_referral_rewards(
        ctx: Context<ClaimRewardWithReferralRewards>
    ) -> Result<()> {
        instructions::referral::claim_reward_with_referral_rewards(ctx)
    }

    // ===== INVITE SYSTEM INSTRUCTIONS =====

    /// 招待コード作成
    /// 8文字のコードで新規ユーザーを招待し、紹介報酬を獲得
    /// 
    /// # Parameters
    /// * `invite_code` - 8バイトの招待コード（英数字のみ）
    /// 
    /// # 招待制限
    /// - 運営者: 255回（事実上無制限）
    /// - 一般ユーザー: 5回まで
    /// 
    /// # セキュリティ
    /// - ハッシュベースでプライバシー確保
    /// - 英数字のみ使用可能
    pub fn create_invite_code(ctx: Context<CreateInviteCode>, invite_code: [u8; 12]) -> Result<()> {
        instructions::invite::create_invite_code(ctx, invite_code)
    }

    /// 招待コード使用
    /// 招待コードでユーザー初期化と紹介関係確立
    pub fn use_invite_code(
        ctx: Context<UseInviteCode>, 
        invite_code: [u8; 12]
    ) -> Result<()> {
        instructions::invite::use_invite_code(ctx, invite_code)
    }

    // ===== SEED SYSTEM INSTRUCTIONS =====

    /// Initialize seed storage for a user
    pub fn initialize_seed_storage(ctx: Context<InitializeSeedStorage>) -> Result<()> {
        instructions::seeds::initialize_seed_storage_instruction(ctx)
    }

    /// ミステリーシードパックの購入（Switchboard VRF統合）
    /// 300 $WEEDを燃焼 + VRF手数料で検証可能な乱数による高レアリティ種を獲得
    /// 
    /// # Parameters
    /// * `quantity` - 購入数量（1-100）
    /// * `user_entropy_seed` - ユーザー提供の乱数シード（追加の乱数性確保）
    /// * `max_vrf_fee` - 支払い可能な最大VRF手数料（lamports）
    /// 
    /// # Switchboard VRF統合
    /// - 検証可能な真正乱数による最高品質の公平性
    /// - コミット・リビール方式で操作不可能
    /// - 第三者オラクルによる透明性保証
    /// 
    /// # コスト構造（正確な計算）
    /// - WEED燃焼: 300 WEED × quantity
    /// - VRF手数料: 約0.002 SOL（2,000,000 lamports）
    ///   * 基本取引手数料: 5,000 × 15取引 = 75,000 lamports
    ///   * ストレージレント: 2,400 lamports
    ///   * オラクル処理費: 2,000,000 lamports
    ///   * 総計: ~2,077,400 lamports
    /// 
    /// # VRF処理フロー
    /// 1. **購入時（コミット）**: VRF要求 + 手数料支払い
    /// 2. **開封時（リビール）**: VRF結果取得 + 種生成
    /// 3. **透明性**: すべてオンチェーンで検証可能
    /// 
    /// # 確率テーブル（VRF保証済み）
    /// - Seed1 (100GP): 42.23%
    /// - Seed2 (180GP): 24.44%
    /// - Seed3 (420GP): 13.33%
    /// - Seed4 (720GP): 8.33%
    /// - Seed5 (1000GP): 5.56%
    /// - Seed6 (5000GP): 3.33%
    /// - Seed7 (15000GP): 1.33%
    /// - Seed8 (30000GP): 0.89%
    /// - Seed9 (60000GP): 0.56%
    /// 
    /// # セキュリティ
    /// - Switchboard VRFによる暗号学的証明
    /// - オラクルネットワークによる分散検証
    /// - 予測不可能性の数学的保証
    /// - 完全なオンチェーン透明性
    pub fn purchase_seed_pack(
        ctx: Context<PurchaseSeedPack>, 
        quantity: u8, 
        user_entropy_seed: u64,
        max_vrf_fee: u64
    ) -> Result<()> {
        instructions::seeds::purchase_seed_pack(ctx, quantity, user_entropy_seed, max_vrf_fee)
    }

    /// Open seed pack to reveal seeds
    pub fn open_seed_pack(ctx: Context<OpenSeedPack>, quantity: u8) -> Result<()> {
        instructions::seeds::open_seed_pack(ctx, quantity)
    }

    /// Plant seed in farm space
    pub fn plant_seed(ctx: Context<PlantSeed>, seed_id: u64) -> Result<()> {
        instructions::seeds::plant_seed(ctx, seed_id)
    }

    /// Remove seed from farm space
    pub fn remove_seed(ctx: Context<RemoveSeed>, seed_id: u64) -> Result<()> {
        instructions::seeds::remove_seed(ctx, seed_id)
    }

    /// Discard seed permanently from storage
    /// Allows users to permanently delete unwanted seeds to free up storage space
    /// and reclaim the rent from the seed account
    pub fn discard_seed(ctx: Context<DiscardSeed>, seed_id: u64) -> Result<()> {
        instructions::seeds::discard_seed(ctx, seed_id)
    }

    /// Batch discard multiple seeds permanently from storage
    /// Efficiently delete up to 100 unwanted seeds in a single transaction
    /// and reclaim all rent from the seed accounts
    pub fn batch_discard_seeds(ctx: Context<BatchDiscardSeeds>, seed_ids: Vec<u64>) -> Result<()> {
        instructions::seeds::batch_discard_seeds(ctx, seed_ids)
    }

    /// Batch plant multiple seeds in farm space
    /// Efficiently plant up to 25 seeds in a single transaction
    /// Validates farm capacity and updates all statistics
    pub fn batch_plant_seeds(ctx: Context<BatchPlantSeeds>, seed_ids: Vec<u64>) -> Result<()> {
        instructions::seeds::batch_plant_seeds(ctx, seed_ids)
    }

    /// Batch remove multiple seeds from farm space
    /// Efficiently remove up to 25 seeds in a single transaction
    /// Updates all statistics and makes seeds available for replanting
    pub fn batch_remove_seeds(ctx: Context<BatchRemoveSeeds>, seed_ids: Vec<u64>) -> Result<()> {
        instructions::seeds::batch_remove_seeds(ctx, seed_ids)
    }

    // ===== FARM LEVEL MANAGEMENT =====

    /// Initialize dynamic farm level configuration with default 5-level system
    pub fn initialize_farm_level_config(ctx: Context<InitializeFarmLevelConfig>) -> Result<()> {
        instructions::farm::initialize_farm_level_config(ctx)
    }

    /// Update farm level configuration (admin only)
    /// Allows adding new levels or modifying existing thresholds
    pub fn update_farm_level_config(
        ctx: Context<UpdateFarmLevelConfig>,
        max_level: u8,
        capacities: Vec<u8>,
        upgrade_thresholds: Vec<u32>,
        level_names: Option<Vec<String>>,
    ) -> Result<()> {
        instructions::farm::update_farm_level_config(
            ctx, max_level, capacities, upgrade_thresholds, level_names
        )
    }

    /// Migrate existing farms to new level configuration
    pub fn migrate_farm_to_new_levels(ctx: Context<MigrateFarmToNewLevels>) -> Result<()> {
        instructions::farm::migrate_farm_to_new_levels(ctx)
    }

    /// Get farm level configuration information (view function)
    pub fn get_farm_level_info(
        ctx: Context<ViewFarmLevelConfig>,
        level: Option<u8>,
    ) -> Result<FarmLevelInfo> {
        instructions::farm::get_farm_level_info(ctx, level)
    }

    // ===== TRANSFER FEE SYSTEM =====
    // Using SPL Token Transfer Fee Extension instead of custom implementation
    // The reward mint will be created with 2% transfer fee configuration
    // No custom transfer instructions needed - fees handled automatically by SPL Token
}