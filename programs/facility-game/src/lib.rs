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
// pub mod state_meteora; // Meteora統合用の拡張構造体 - temporarily disabled
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

    /// $WEEDトークンのミントアカウントを作成
    /// ゲーム内通貨として使用される報酬トークンを初期化
    /// 
    /// # 機能
    /// - PDAによるミント権限管理（セキュリティ向上）
    /// - Metaplexメタデータ作成（トークン情報表示用）
    /// - 6桁精度での発行設定
    /// 
    /// # Security
    /// - mint_authorityはPDAで制御
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
    
    /// Accumulate referral reward to a referrer's pending balance
    /// Called when someone claims rewards to add commission to their referrer
    pub fn accumulate_referral_reward(
        ctx: Context<AccumulateReferralReward>,
        reward_amount: u64,
        referral_level: u8,
    ) -> Result<()> {
        instructions::referral_rewards::accumulate_referral_reward(ctx, reward_amount, referral_level)
    }
    
    /// View current pending referral rewards for a user
    /// Allows users to check their accumulated referral commission
    pub fn view_pending_referral_rewards(ctx: Context<ViewPendingReferralRewards>) -> Result<()> {
        instructions::referral_rewards::view_pending_referral_rewards(ctx)
    }
    
    /// Enhanced claim reward that includes both farming and referral rewards
    /// This is the main claim function users should call to get all accumulated rewards
    pub fn claim_reward_with_referral_rewards(
        ctx: Context<ClaimRewardWithReferralRewards>
    ) -> Result<()> {
        instructions::referral_rewards::claim_reward_with_referral_rewards(ctx)
    }

    // ===== INVITE SYSTEM INSTRUCTIONS =====

    /// 招待コードの作成
    /// 8文字のコードで新規ユーザーを招待し、紹介報酬を獲得
    /// 
    /// # Parameters
    /// * `invite_code` - 8バイトの招待コード（英数字のみ）
    /// 
    /// # 招待システム
    /// - 初期限度: 5人まで招待可能
    /// - 管理者による限度拡張可能
    /// - 多段階紹介報酬システム連携
    /// 
    /// # セキュリティ
    /// - ハッシュベースでプライバシー確保
    /// - 英数字のみ使用可能
    pub fn create_invite_code(ctx: Context<CreateInviteCode>, invite_code: [u8; 8]) -> Result<()> {
        instructions::invite::create_invite_code(ctx, invite_code)
    }

    /// Use invite code to initialize user with referrer
    /// Requires: plaintext code + inviter address
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

    /// ミステリーシードパックの購入（Pyth Entropy統合）
    /// 300 $WEEDを燃焼してPyth Entropyで真の乱数を使用した高レアリティ種を獲得
    /// 
    /// # Parameters
    /// * `quantity` - 購入数量（1-100）
    /// * `user_entropy_seed` - ユーザー提供の乱数シード（追加の乱数性確保）
    /// 
    /// # Pyth Entropy統合
    /// - コミットフェーズ: 購入時にPyth Entropyにランダム要求
    /// - リビールフェーズ: 開封時にPyth Entropyから結果取得
    /// - 二重ランダム性: ユーザーシード + オラクルシード
    /// 
    /// # 確率テーブル（Pyth Entropy使用）
    /// - Seed1 (100GP): 42.23%
    /// - Seed2 (180GP): 24.44%
    /// - Seed3 (420GP): 13.33%
    /// - ...最高レアSeed9 (60000GP): 0.56%
    /// 
    /// # セキュリティ
    /// - 真の乱数による公平性保証
    /// - 予測不可能な結果
    /// - 透明性のあるオンチェーン検証
    pub fn purchase_seed_pack(ctx: Context<PurchaseSeedPack>, quantity: u8, user_entropy_seed: u64) -> Result<()> {
        instructions::seeds::purchase_seed_pack(ctx, quantity, user_entropy_seed)
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

    // ===== TRADING SYSTEM INSTRUCTIONS =====


    /// Convert accumulated fees to SOL via Meteora DEX
    pub fn convert_fees_to_sol(ctx: Context<ConvertFeesToSol>) -> Result<()> {
        instructions::meteora::convert_fees_to_sol(ctx)
    }

    /// Update Meteora pool configuration (admin only)
    pub fn update_meteora_config(
        ctx: Context<UpdateMeteoraConfig>,
        meteora_pool: Pubkey,
        pool_weed_vault: Pubkey,
        pool_sol_vault: Pubkey,
    ) -> Result<()> {
        instructions::meteora::update_meteora_config(ctx, meteora_pool, pool_weed_vault, pool_sol_vault)
    }

    // ===== ADVANCED METEORA INTEGRATION =====
    // Temporarily disabled meteora_admin functions

    // /// Initialize Meteora configuration system (admin only)
    // pub fn initialize_meteora_config(ctx: Context<InitializeMeteoraConfig>) -> Result<()> {
    //     instructions::meteora_admin::initialize_meteora_config(ctx)
    // }

    // /// Configure DLMM pool settings (admin only)
    // pub fn configure_dlmm_pool(
    //     ctx: Context<AdminConfigureDlmmPool>, 
    //     pool_config: DlmmPoolConfig
    // ) -> Result<()> {
    //     instructions::meteora_admin::configure_dlmm_pool(ctx, pool_config)
    // }

    // /// Update conversion settings (admin only)
    // pub fn update_conversion_settings(
    //     ctx: Context<UpdateConversionSettings>,
    //     settings: ConversionSettings,
    // ) -> Result<()> {
    //     instructions::meteora_admin::update_conversion_settings(ctx, settings)
    // }

    // /// Swap WEED to SOL via DLMM with advanced features
    // pub fn swap_weed_to_sol_via_dlmm(
    //     ctx: Context<SwapWeedToSolViaDlmm>,
    //     min_sol_output: u64,
    //     slippage_tolerance_bps: Option<u16>,
    // ) -> Result<()> {
    //     instructions::meteora_minimal::swap_weed_to_sol_via_dlmm(ctx, min_sol_output, slippage_tolerance_bps)
    // }

    // /// Emergency pause/resume for Meteora conversions (admin only)
    // pub fn emergency_pause_toggle(ctx: Context<EmergencyControl>, pause: bool) -> Result<()> {
    //     instructions::meteora_admin::emergency_pause_toggle(ctx, pause)
    // }

    // /// Monitor pool health status
    // pub fn monitor_pool_health(ctx: Context<MonitorPoolHealth>) -> Result<()> {
    //     instructions::meteora_admin::monitor_pool_health(ctx)
    // }

    // /// View comprehensive Meteora statistics
    // pub fn view_meteora_stats(ctx: Context<ViewMeteoraStats>) -> Result<()> {
    //     instructions::meteora_admin::view_meteora_stats(ctx)
    // }

    // ===== TRANSFER SYSTEM =====

    /// Transfer with fee system (FeePool accumulation)
    pub fn transfer_with_fee(
        ctx: Context<TransferWithImprovedFee>, 
        amount: u64
    ) -> Result<()> {
        instructions::transfer_improved::transfer_with_improved_fee(ctx, amount)
    }


    /// Batch transfer with fee optimization
    pub fn batch_transfer_with_fee(
        ctx: Context<BatchTransferWithFee>,
        transfers: Vec<instructions::transfer_improved::TransferInstruction>,
    ) -> Result<()> {
        instructions::transfer_improved::batch_transfer_with_fee(ctx, transfers)
    }

    // /// Check automatic conversion trigger
    // pub fn check_auto_conversion_trigger(ctx: Context<SwapWeedToSolViaDlmm>) -> Result<bool> {
    //     instructions::meteora_minimal::check_auto_conversion_trigger(ctx)
    // }
}