// Anchorフレームワークの基本的な型や機能をインポート
// prelude::*は、よく使う型（Account, Signer, Program等）を一括でインポート
use anchor_lang::prelude::*;

// SPLトークンプログラムとの連携に必要な機能をインポート
// 個別の操作は各モジュールで実装される

// このプログラムの一意のIDを宣言
// Solana上でプログラムを識別するために使用される
// anchor buildコマンドで自動生成され、target/deploy/に保存される
declare_id!("7r3R1S43BS9fQbh1eBhM63u8XZJd7bYRtgMrAQRNrfcB");

// モジュールの宣言
// Rustのモジュールシステムを使って、コードを論理的に分割
pub mod state;        // アカウントの構造体定義
pub mod instructions; // 各命令のコンテキスト（必要なアカウント）定義
pub mod error;        // カスタムエラー定義
pub mod utils;        // ヘルパー関数

// 他のモジュールから必要な要素をインポート
use instructions::*;  // すべての命令コンテキストをインポート

// #[program]属性は、このモジュールがSolanaプログラムのエントリーポイントであることを示す
// Anchorがこの属性を見つけると、各関数を呼び出し可能な命令として処理する
#[program]
pub mod facility_game {
    use super::*;

    /// Initialize global configuration  
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        base_rate: u64,
        halving_interval: i64,
        treasury: Pubkey,
    ) -> Result<()> {
        instructions::admin::initialize_config(ctx, base_rate, halving_interval, treasury)
    }

    /// Create reward token mint account
    pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
        instructions::admin::create_reward_mint(ctx)
    }

    /// Initialize user account
    pub fn init_user(ctx: Context<InitUser>, referrer: Option<Pubkey>) -> Result<()> {
        instructions::user::init_user(ctx, referrer)
    }

    /// Purchase facility and place initial machine
    pub fn buy_facility(ctx: Context<BuyFacility>) -> Result<()> {
        instructions::facility::buy_facility(ctx)
    }

    /// Claim rewards
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        instructions::rewards::claim_reward(ctx)
    }

    /// Distribute referral reward to referrer
    pub fn distribute_referral_reward(ctx: Context<DistributeReferralReward>, reward_amount: u64) -> Result<()> {
        instructions::rewards::distribute_referral_reward(ctx, reward_amount)
    }

    /// Claim accumulated referral rewards
    pub fn claim_referral_rewards(ctx: Context<ClaimReferralRewards>) -> Result<()> {
        instructions::rewards::claim_referral_rewards(ctx)
    }

    /// Upgrade facility size
    pub fn upgrade_facility(ctx: Context<UpgradeFacility>) -> Result<()> {
        instructions::facility::upgrade_facility(ctx)
    }

    /// Add machine to facility
    pub fn add_machine(ctx: Context<UpgradeFacility>) -> Result<()> {
        instructions::facility::add_machine(ctx)
    }

    /// Transfer tokens with 2% fee
    pub fn transfer_with_fee(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
        instructions::transfer::transfer_with_fee(ctx, amount)
    }

    /// Purchase mystery box
    pub fn purchase_mystery_box(ctx: Context<PurchaseMysteryBox>, box_id: u64) -> Result<()> {
        instructions::mystery::purchase_mystery_box(ctx, box_id)
    }

    /// Open mystery box and reveal seed
    pub fn open_mystery_box(ctx: Context<OpenMysteryBox>, box_id: u64, seed_id: u64) -> Result<()> {
        instructions::mystery::open_mystery_box(ctx, box_id, seed_id)
    }
}