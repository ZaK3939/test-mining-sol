// Anchorフレームワークの基本的な型や機能をインポート
// prelude::*は、よく使う型（Account, Signer, Program等）を一括でインポート
#![allow(deprecated)] // Suppress Anchor framework's internal deprecation warnings
use anchor_lang::prelude::*;

// SPLトークンプログラムとの連携に必要な機能をインポート
// 個別の操作は各モジュールで実装される

// このプログラムの一意のIDを宣言
// Solana上でプログラムを識別するために使用される
// anchor buildコマンドで自動生成され、target/deploy/に保存される
declare_id!("FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B");

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

// 追加の高度なテストモジュール（一時的にコメントアウト）
// #[cfg(test)]
// mod test_modules {
//     pub mod state_advanced_tests;
//     pub mod error_comprehensive_tests;
//     pub mod economics_advanced_tests;
// }

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
    /// * `halving_interval` - 半減期間隔（デフォルト: 6日）
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

    // ===== USER MANAGEMENT INSTRUCTIONS =====

    /// ユーザーアカウントの初期化
    /// ゲームへの参加に必要な基本データ構造を作成
    /// 
    /// # Parameters
    /// * `referrer` - 紹介者のpubkey（招待コード使用時のみ）
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

    /// 農場スペースのアップグレード（即座実行）
    /// $WEEDを消費して即座に容量増加
    /// 
    /// # アップグレードコスト
    /// - Lv1→2: 3,500 WEED (容量 4→8)
    /// - Lv2→3: 18,000 WEED (容量 8→12)
    /// - Lv3→4: 20,000 WEED (容量 12→16)
    /// - Lv4→5: 25,000 WEED (容量 16→20)
    /// 
    /// # 即座実行
    /// - クールダウンなし、即座にアップグレード完了
    pub fn upgrade_farm_space(ctx: Context<UpgradeFarmSpace>) -> Result<()> {
        instructions::farm::upgrade_farm_space(ctx)
    }


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
    pub fn create_invite_code(ctx: Context<CreateInviteCode>, invite_code: [u8; 8]) -> Result<()> {
        instructions::invite::create_invite_code(ctx, invite_code)
    }

    /// 招待コード使用
    /// 招待コードでユーザー初期化と紹介関係確立
    pub fn use_invite_code(
        ctx: Context<UseInviteCode>, 
        invite_code: [u8; 8],
        inviter_pubkey: Pubkey
    ) -> Result<()> {
        instructions::invite::use_invite_code(ctx, invite_code, inviter_pubkey)
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


    // ===== TRANSFER FEE SYSTEM =====
    // Using SPL Token Transfer Fee Extension instead of custom implementation
    // The reward mint will be created with 2% transfer fee configuration
    // No custom transfer instructions needed - fees handled automatically by SPL Token
}