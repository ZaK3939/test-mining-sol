// Anchorフレームワークの基本的な型や機能をインポート
// prelude::*は、よく使う型（Account, Signer, Program等）を一括でインポート
use anchor_lang::prelude::*;

// SPLトークンプログラムとの連携に必要な機能をインポート
// token::selfはトークン操作の関数、MintToはトークン発行の際に使用
use anchor_spl::token::{self, MintTo};

// このプログラムの一意のIDを宣言
// Solana上でプログラムを識別するために使用される
// anchor buildコマンドで自動生成され、target/deploy/に保存される
declare_id!("EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89");

// モジュールの宣言
// Rustのモジュールシステムを使って、コードを論理的に分割
pub mod state;        // アカウントの構造体定義
pub mod instructions; // 各命令のコンテキスト（必要なアカウント）定義
pub mod error;        // カスタムエラー定義

// 他のモジュールから必要な要素をインポート
use instructions::*;  // すべての命令コンテキストをインポート
use error::*;         // カスタムエラーをインポート

// #[program]属性は、このモジュールがSolanaプログラムのエントリーポイントであることを示す
// Anchorがこの属性を見つけると、各関数を呼び出し可能な命令として処理する
#[program]
pub mod facility_game {
    use super::*;

    /// Initialize global configuration
    /// グローバル設定を初期化する関数（管理者のみ実行可能）
    /// 
    /// # 引数
    /// * `ctx` - InitializeConfigコンテキスト（必要なアカウント情報を含む）
    /// * `base_rate` - 基本報酬レート（1000で割った値が実際のレート）
    /// * `halving_interval` - 半減期の間隔（秒単位）
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        base_rate: u64,
        halving_interval: i64,
    ) -> Result<()> {
        // ctx.accountsから設定アカウントの可変参照を取得
        // &mutは、このアカウントのデータを変更することを示す
        let config = &mut ctx.accounts.config;
        
        // 設定値を保存
        config.base_rate = base_rate;
        config.halving_interval = halving_interval;
        
        // Clock::get()?でSolanaのシステム時刻を取得
        // ?演算子は、エラーが発生した場合に早期リターンする
        // unix_timestampは1970年1月1日からの経過秒数
        config.next_halving_time = Clock::get()?.unix_timestamp + halving_interval;
        
        // 管理者のアドレスを保存
        // key()メソッドでSignerからPubkeyを取得
        config.admin = ctx.accounts.admin.key();
        
        // msg!マクロはSolanaのログに出力する
        // solana logsコマンドで確認できる
        msg!("Config initialized with base_rate: {}, halving_interval: {}", base_rate, halving_interval);
        
        // Ok(())は成功を示す
        // Result<()>型なので、エラーまたはOkを返す必要がある
        Ok(())
    }

    /// Create reward token mint account
    /// 報酬トークンのMintアカウントを作成
    /// MintはSPLトークンの発行元となるアカウント
    /// 
    /// # 注意
    /// - Mintの作成はinstructions/mod.rsの#[derive(Accounts)]で自動的に行われる
    /// - この関数では追加の処理のみ行う
    pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
        // Mint作成の確認ログ
        // mint_authorityはPDAで、このプログラムのみがトークンを発行できる
        msg!("Reward mint created with authority: {}", ctx.accounts.mint_authority.key());
        Ok(())
    }

    /// Initialize user account
    /// 新規ユーザーのアカウントを初期化
    /// 
    /// # 処理内容
    /// - ユーザー専用のPDAアカウントを作成
    /// - 初期状態を設定（施設なし、Grow Power 0）
    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        // ユーザー状態アカウントの可変参照を取得
        let user_state = &mut ctx.accounts.user_state;
        
        // ユーザーのウォレットアドレスを保存
        user_state.owner = ctx.accounts.user.key();
        
        // 初期値を設定
        user_state.total_grow_power = 0;  // 施設がないのでGrow Powerは0
        
        // 現在時刻を最終収穫時刻として設定
        // これにより、初回の報酬計算が正しく行われる
        user_state.last_harvest_time = Clock::get()?.unix_timestamp;
        
        user_state.has_facility = false;   // 施設未所有
        user_state.reserve = [0; 64];      // 将来の拡張用領域を0で初期化

        msg!("User initialized: {}", ctx.accounts.user.key());
        Ok(())
    }

    /// Purchase facility and place initial machine
    /// 施設を購入し、初期マシンを自動配置
    /// 
    /// # 制限
    /// - 各ユーザーは1つの施設のみ所有可能（MVP版の制限）
    /// - 購入と同時に1台のマシンが自動配置される
    pub fn buy_facility(ctx: Context<BuyFacility>) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;
        
        // Check if already has facility
        // require!マクロは条件をチェックし、falseの場合はエラーを返す
        // !user_state.has_facilityは「施設を持っていない」ことを確認
        require!(!user_state.has_facility, GameError::AlreadyHasFacility);
        
        let facility = &mut ctx.accounts.facility;
        // Initialize facility
        // Pubkey::default()は全て0のアドレス（未使用状態を示す）
        // 既に所有者が設定されている場合はエラー
        if facility.owner != Pubkey::default() {
            // into()でカスタムエラーをAnchorのエラー型に変換
            return Err(GameError::AlreadyHasFacility.into());
        }

        // 施設の初期設定
        facility.owner = ctx.accounts.user.key();        // 所有者を設定
        facility.machine_count = 1;                      // 初期マシン1台を自動配置
        facility.total_grow_power = 100;                 // 初期Grow Power（マシン1台分）
        facility.reserve = [0; 64];                      // 拡張用領域

        // Update user state
        // ユーザー状態を施設所有に更新
        user_state.has_facility = true;
        
        // 施設のGrow Powerをユーザーの総Grow Powerに反映
        user_state.total_grow_power = facility.total_grow_power;
        
        // 購入時刻を最終収穫時刻として設定
        // これにより、購入直後の報酬は0から始まる
        user_state.last_harvest_time = Clock::get()?.unix_timestamp;

        msg!("Facility purchased for user: {}, initial grow power: {}", 
             ctx.accounts.user.key(), facility.total_grow_power);
        Ok(())
    }

    /// Claim rewards
    /// 蓄積された報酬を請求してSPLトークンとして受け取る
    /// 
    /// # 報酬計算式
    /// 報酬 = (経過時間[秒] × Grow Power × base_rate) / 1000
    /// 
    /// # 処理フロー
    /// 1. 半減期のチェックと更新
    /// 2. 経過時間の計算
    /// 3. 報酬量の計算
    /// 4. SPLトークンのミント（発行）
    /// 5. 最終収穫時刻の更新
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;
        let config = &mut ctx.accounts.config;
        
        // Check if has facility
        // 施設を所有していない場合は報酬を請求できない
        require!(user_state.has_facility, GameError::NoFacility);

        // 現在時刻を取得
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check and update halving
        // 半減期のチェック：現在時刻が次の半減期時刻を過ぎている場合
        while current_time >= config.next_halving_time {
            // 基本レートを半分にする（ビットコインの半減期と同様の仕組み）
            config.base_rate = config.base_rate / 2;
            
            // 次の半減期時刻を更新
            config.next_halving_time += config.halving_interval;
            
            // 半減期発生をログに記録
            msg!("Halving occurred! New base rate: {}", config.base_rate);
        }

        // Calculate reward (elapsed time × Grow Power × base_rate)
        // 最後の収穫からの経過時間を計算（秒単位）
        let time_elapsed = current_time - user_state.last_harvest_time;
        
        // 報酬量を計算（オーバーフロー対策のためchecked演算を使用）
        // checked_mul: 乗算でオーバーフローした場合はNoneを返す
        // and_then: 前の計算が成功した場合のみ次の計算を実行
        let reward_amount = (time_elapsed as u64)
            .checked_mul(user_state.total_grow_power)      // 経過時間 × Grow Power
            .and_then(|result| result.checked_mul(config.base_rate))  // × base_rate
            .and_then(|result| result.checked_div(1000))   // ÷ 1000 でレート調整
            .ok_or(GameError::CalculationOverflow)?;        // Noneの場合はエラー

        // 報酬が0の場合はエラー（ガス代の無駄を防ぐ）
        require!(reward_amount > 0, GameError::NoRewardToClaim);

        // Mint tokens (fixed version)
        // CPI（Cross-Program Invocation）でSPLトークンプログラムを呼び出す準備
        // MintTo構造体に必要なアカウント情報を設定
        let cpi_accounts = MintTo {
            mint: ctx.accounts.reward_mint.to_account_info(),      // トークンのMintアカウント
            to: ctx.accounts.user_token_account.to_account_info(), // 送信先（ユーザーのトークンアカウント）
            authority: ctx.accounts.mint_authority.to_account_info(), // Mint権限（PDA）
        };

        // Fix: correct seed structure
        // PDAの署名を作成するための準備
        // bumpsはPDAを導出する際に使用された値（Anchorが自動的に提供）
        let authority_bump = ctx.bumps.mint_authority;
        
        // PDAのシード（種）を定義
        // b"mint_authority"は文字列をバイト配列に変換
        let seeds = &[
            b"mint_authority".as_ref(),  // シード文字列
            &[authority_bump],            // bump値（PDAを有効にする値）
        ];
        
        // CPIで使用する署名者情報
        let signer = &[&seeds[..]];

        // SPLトークンプログラムの参照を取得
        let cpi_program = ctx.accounts.token_program.to_account_info();
        
        // CPI実行コンテキストを作成
        // new_with_signerはPDAが署名者として動作することを示す
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        // SPLトークンプログラムのmint_to命令を実行
        // これにより、指定量のトークンがユーザーのアカウントに発行される
        token::mint_to(cpi_ctx, reward_amount)?;

        // Update last harvest time
        // 最終収穫時刻を現在時刻に更新
        // これにより、次回の報酬計算はこの時点から開始される
        user_state.last_harvest_time = current_time;

        msg!("Reward claimed: {} tokens for user: {}", 
             reward_amount, ctx.accounts.user.key());
        Ok(())
    }
}