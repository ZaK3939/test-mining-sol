// ===== モジュールの概要 =====
// このファイルは各命令（instruction）で必要となるアカウントの集合を定義します。
// Solanaでは、プログラムが使用するすべてのアカウントを事前に宣言する必要があり、
// それを「コンテキスト」として管理します。

// Anchorの基本的な型や機能をインポート
use anchor_lang::prelude::*;

// SPLトークンプログラムの型をインポート
// Mint: トークンの発行元アカウント（総供給量、小数点以下桁数などを管理）
// Token: SPLトークンプログラムそのもの
// TokenAccount: ユーザーがトークンを保持するアカウント
use anchor_spl::token::{Mint, Token, TokenAccount};

// 同じクレート内のstate.rsから構造体をインポート
use crate::state::*;

/// Context for initializing global configuration
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for creating reward token mint
#[derive(Accounts)]
pub struct CreateRewardMint<'info> {
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = mint_authority,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    /// @dev Set mint authority as PDA
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Context for user initialization
/// ユーザー初期化のコンテキスト
/// 
/// # 役割
/// - 新規ユーザーがゲームに参加する際に、専用のPDAアカウントを作成
/// - ユーザーの状態（施設所有、Grow Power、最終収穫時刻）を管理
/// 
/// # PDAの利点
/// - 各ユーザーごとに一意のアドレスが決定論的に生成される
/// - 同じユーザーは常に同じアドレスを持つ（再作成を防ぐ）
#[derive(Accounts)]
pub struct InitUser<'info> {
    // ===== ユーザー状態アカウント =====
    // 各ユーザーのゲーム内状態を保存するPDA
    #[account(
        init,                                    // 新規作成
        payer = user,                           // ユーザー自身が費用を支払う
        space = UserState::LEN,                 // 必要なスペース（state.rsで定義）
        seeds = [b"user", user.key().as_ref()], // シード: "user" + ユーザーのウォレットアドレス
        bump                                    // これにより各ユーザーごとに一意のPDA
    )]
    pub user_state: Account<'info, UserState>,  // ユーザーのゲーム内状態を保存
    
    // ===== ユーザーアカウント =====
    #[account(mut)]              // mut: レント支払いで残高が変化
    pub user: Signer<'info>,     // ユーザー自身の署名が必要
    
    // システムプログラム（PDA作成に必要）
    pub system_program: Program<'info, System>,
}

/// Context for facility purchase
/// 施設購入のコンテキスト
/// 
/// # 役割
/// - ユーザーが施設を購入する際の処理
/// - 施設購入と同時に初期マシン（Grow Power: 100）を自動配置
/// - 各ユーザー1施設の制限を自然に実現（PDAの一意性により）
/// 
/// # 処理フロー
/// 1. ユーザー状態を確認（既に施設を持っていないか）
/// 2. 新しい施設PDAを作成
/// 3. ユーザー状態を更新（has_facility = true）
#[derive(Accounts)]
pub struct BuyFacility<'info> {
    // ===== ユーザー状態アカウント（既存） =====
    // 施設所有フラグとGrow Powerを更新するため可変（mut）
    #[account(
        mut,                                     // 施設所有フラグを更新するため可変
        seeds = [b"user", user.key().as_ref()], // 同じシードで既存アカウントを特定
        bump                                     // PDAの検証
    )]
    pub user_state: Account<'info, UserState>,  // has_facilityやgrow_powerを更新
    
    // ===== 施設アカウント（新規作成） =====
    // ユーザーが所有する施設の情報を保存
    #[account(
        init,                                        // 新規作成
        payer = user,                               // ユーザーが作成費用を支払う
        space = Facility::LEN,                      // 施設データのサイズ
        seeds = [b"facility", user.key().as_ref()], // "facility" + ユーザーアドレス
        bump                                        // 各ユーザー1施設の制限を自然に実現
    )]
    pub facility: Account<'info, Facility>,         // 施設の詳細情報を保存
    
    // ===== ユーザーアカウント =====
    #[account(mut)]              // レント支払いで残高が変化
    pub user: Signer<'info>,     // ユーザーの署名が必要
    
    // システムプログラム（施設PDA作成に必要）
    pub system_program: Program<'info, System>,
}

/// Context for claiming rewards
/// 報酬請求のコンテキスト
/// 
/// # 役割
/// - 時間経過に基づいて計算された報酬をSPLトークンとして受け取る
/// - 最も複雑な処理で、複数のアカウントが連携して動作
/// 
/// # 処理フロー
/// 1. 半減期のチェックと更新（config）
/// 2. 経過時間の計算（user_state.last_harvest_time）
/// 3. 報酬量の計算（経過時間 × Grow Power × base_rate）
/// 4. トークンのミント（reward_mint → user_token_account）
/// 5. 最終収穫時刻の更新（user_state）
/// 
/// # CPI（Cross-Program Invocation）
/// - SPLトークンプログラムを呼び出してトークンを発行
/// - mint_authorityがPDAのため、プログラムが署名を生成
#[derive(Accounts)]
pub struct ClaimReward<'info> {
    // ===== ユーザー状態 =====
    // 最終収穫時刻を更新するため可変
    #[account(
        mut,                                     // last_harvest_timeを更新
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    // ===== グローバル設定 =====
    // 半減期の確認と更新のため可変
    #[account(
        mut,                  // 半減期でbase_rateを更新する可能性
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    // ===== 報酬トークンMint =====
    // トークン発行で総供給量が増加するため可変
    #[account(
        mut,                      // トークン発行で総供給量が変化
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    // ===== Mint権限PDA =====
    // トークン発行の署名者として使用
    #[account(
        seeds = [b"mint_authority"],
        bump                          // このbump値はCPIで署名生成時に使用
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    // ===== ユーザーのトークンアカウント =====
    // 報酬トークンの受け取り先（事前に作成が必要）
    #[account(
        mut,                                                        // トークン残高が増加
        constraint = user_token_account.owner == user.key(),        // 所有者の確認（セキュリティ）
        constraint = user_token_account.mint == reward_mint.key()   // 正しいトークン種類の確認
    )]
    pub user_token_account: Account<'info, TokenAccount>,           // ここに報酬が振り込まれる
    
    // ===== ユーザーアカウント =====
    #[account(mut)]               // ガス代の支払いで残高が変化
    pub user: Signer<'info>,      // トランザクションの署名者
    
    // ===== SPLトークンプログラム =====
    // mint_to命令の実行に必要（CPI）
    pub token_program: Program<'info, Token>,
}