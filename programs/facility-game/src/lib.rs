// Anchorフレームワークの基本的な型や機能をインポート
// prelude::*は、よく使う型（Account, Signer, Program等）を一括でインポート
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
pub mod constants;    // ゲーム定数
pub mod validation;   // バリデーション機能
pub mod economics;    // 経済計算

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

    /// 農場スペースのアップグレード開始
    /// $WEEDを消費して24時間後に容量増加
    /// 
    /// # アップグレードコスト
    /// - Lv1→2: 3,500 WEED (容量 4→8)
    /// - Lv2→3: 18,000 WEED (容量 8→12)
    /// - Lv3→4: 20,000 WEED (容量 12→16)
    /// - Lv4→5: 25,000 WEED (容量 16→20)
    /// 
    /// # クールダウン機構
    /// - 24時間の待機時間
    /// - complete_farm_space_upgradeで完了
    pub fn upgrade_farm_space(ctx: Context<UpgradeFarmSpace>) -> Result<()> {
        instructions::farm::upgrade_farm_space(ctx)
    }

    /// Complete farm space upgrade after cooldown
    pub fn complete_farm_space_upgrade(ctx: Context<CompleteFarmSpaceUpgrade>) -> Result<()> {
        instructions::farm::complete_farm_space_upgrade(ctx)
    }

    // ===== REWARD SYSTEM INSTRUCTIONS =====

    /// 報酬の請求
    /// ユーザーの貢献度に応じた比例配分で$WEEDトークンを獲得
    /// 
    /// # 計算式
    /// ```
    /// 報酬 = (ユーザーGrow Power / 全体Grow Power) × 基本レート × 経過時間
    /// ```
    /// 
    /// # 処理フロー
    /// 1. 半減期チェック・適用
    /// 2. ユーザーシェア計算
    /// 3. トークンミント・配布
    /// 4. 紹介報酬処理（L1: 10%, L2: 5%）
    /// 5. タイムスタンプ更新
    /// 
    /// # 紹介システム
    /// - Level 1 (直接招待): 10%の報酬
    /// - Level 2 (間接招待): 5%の報酬
    /// - プロトコル指定アドレスは報酬対象外
    /// - 紹介者のアカウントが提供された場合に即座にミント
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        instructions::rewards::claim_reward(ctx)
    }

    /// Distribute referral reward to referrer
    pub fn distribute_referral_reward(ctx: Context<DistributeReferralReward>, reward_amount: u64) -> Result<()> {
        instructions::rewards::distribute_referral_reward(ctx, reward_amount)
    }

    /// Distribute referral rewards during claim process
    /// Called separately to handle Level 1 (10%) and Level 2 (5%) rewards
    /// Automatically excludes protocol referral addresses
    pub fn distribute_referral_on_claim(ctx: Context<DistributeReferralOnClaim>, base_reward: u64) -> Result<()> {
        instructions::rewards::distribute_referral_on_claim(ctx, base_reward)
    }

    /// Claim accumulated referral rewards
    pub fn claim_referral_rewards(ctx: Context<ClaimReferralRewards>) -> Result<()> {
        instructions::rewards::claim_referral_rewards(ctx)
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
    /// # 制約
    /// - コードの重複不可
    /// - 英数字のみ使用可能
    pub fn create_invite_code(ctx: Context<CreateInviteCode>, invite_code: [u8; 8]) -> Result<()> {
        instructions::invite::create_invite_code(ctx, invite_code)
    }

    /// Use invite code to initialize user with referrer
    pub fn use_invite_code(ctx: Context<UseInviteCode>, invite_code: [u8; 8]) -> Result<()> {
        instructions::invite::use_invite_code(ctx, invite_code)
    }

    /// Expand invite limit (admin only)
    pub fn expand_invite_limit(ctx: Context<ExpandInviteLimit>, additional_invites: u8) -> Result<()> {
        instructions::invite::expand_invite_limit(ctx, additional_invites)
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

    // ===== TRADING SYSTEM INSTRUCTIONS =====

    /// Transfer tokens with 2% fee
    pub fn transfer_with_fee(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
        instructions::transfer::transfer_with_fee(ctx, amount)
    }

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
}