// ===== 農場スペース管理モジュール =====
// このモジュールは農場スペースの購入、アップグレード、管理機能を提供します
// 農場スペースはユーザーが種を植えて報酬を得るための基本単位です

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::state::*;
use crate::error::*;
use crate::utils::*;

/// 農場スペース購入のためのアカウント定義
/// レベル1農場の初期購入（0.5 SOL + 無料Seed1付与）
#[derive(Accounts)]
pub struct BuyFarmSpace<'info> {
    /// ユーザーの状態アカウント（更新用）
    /// has_farm_space フラグをtrueに変更し、total_grow_powerを更新
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    /// 新規作成される農場スペースアカウント
    /// 初期設定: レベル1、容量4、シード数0
    #[account(
        init,
        payer = user,
        space = FarmSpace::LEN,
        seeds = [b"farm_space", user.key().as_ref()],
        bump
    )]
    pub farm_space: Account<'info, FarmSpace>,
    
    /// 無料で付与される初期シード（Seed1、100 Grow Power）
    /// シードID 0として作成され、自動的に農場に植えられる
    #[account(
        init,
        payer = user,
        space = Seed::LEN,
        seeds = [b"seed", user.key().as_ref(), 0u64.to_le_bytes().as_ref()], // シードID 0の初期ギフト
        bump
    )]
    pub initial_seed: Account<'info, Seed>,
    
    /// システム設定アカウント（農場購入コスト確認用）
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    /// グローバル統計アカウント（全体のGrow Power、農場数更新用）
    #[account(
        mut,
        seeds = [b"global_stats"],
        bump
    )]
    pub global_stats: Account<'info, GlobalStats>,
    
    /// 支払い先のトレジャリーアカウント（0.5 SOL受取用）
    /// configで設定されたtreasuryアドレスと一致する必要がある
    #[account(
        mut,
        constraint = treasury.key() == config.treasury
    )]
    /// CHECK: Treasury address from config
    pub treasury: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for upgrading farm space
#[derive(Accounts)]
pub struct UpgradeFarmSpace<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump,
        constraint = user_state.has_farm_space @ GameError::NoFarmSpace
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"farm_space", user.key().as_ref()],
        bump,
        constraint = farm_space.owner == user.key()
    )]
    pub farm_space: Account<'info, FarmSpace>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == reward_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Purchase farm space (Level 1)
/// Cost: 0.5 SOL + Seed 1 (100 Grow Power) gifted
pub fn buy_farm_space(ctx: Context<BuyFarmSpace>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let farm_space = &mut ctx.accounts.farm_space;
    let initial_seed = &mut ctx.accounts.initial_seed;
    let config = &mut ctx.accounts.config;
    let global_stats = &mut ctx.accounts.global_stats;
    
    // Validate prerequisites
    require!(!user_state.has_farm_space, GameError::AlreadyHasFarmSpace);
    require!(farm_space.owner == Pubkey::default(), GameError::AlreadyHasFarmSpace);

    // Process SOL payment to treasury
    transfer_sol_payment(
        &ctx.accounts.user,
        &ctx.accounts.treasury,
        &ctx.accounts.system_program,
        config.farm_space_cost_sol,
    )?;

    // Initialize farm space with helper function
    initialize_farm_space_level_1(farm_space, ctx.accounts.user.key())?;

    // Create initial Seed 1 PDA as a gift
    initial_seed.owner = ctx.accounts.user.key();
    initial_seed.seed_type = SeedType::Seed1;
    initial_seed.grow_power = SeedType::Seed1.get_grow_power(); // 100 Grow Power
    initial_seed.is_planted = true; // Automatically planted
    initial_seed.planted_farm_space = Some(farm_space.key());
    initial_seed.created_at = Clock::get()?.unix_timestamp;
    initial_seed.seed_id = 0; // Special ID for initial gift
    initial_seed.reserve = [0; 32];

    // Update seed counter (start from 1 for next seeds)
    config.seed_counter = 1;

    // Update user state
    let current_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = true;
    user_state.total_grow_power = farm_space.total_grow_power;
    user_state.last_harvest_time = current_time;
    
    // Update global statistics
    update_global_stats_on_farm_creation(
        global_stats,
        farm_space.total_grow_power,
        current_time,
    );

    msg!("Farm space (Level 1) purchased for user: {} with initial Seed 1 (ID: 0, Grow Power: {}), SOL paid: {} lamports", 
         ctx.accounts.user.key(), 
         initial_seed.grow_power,
         config.farm_space_cost_sol);
    Ok(())
}

/// Upgrade farm space instantly (no cooldown)
pub fn upgrade_farm_space(ctx: Context<UpgradeFarmSpace>) -> Result<()> {
    let user_token_account = &ctx.accounts.user_token_account;
    
    // Validate upgrade prerequisites
    validate_upgrade_prerequisites(&ctx.accounts.farm_space)?;
    
    // Get upgrade cost using helper function
    let upgrade_cost = get_upgrade_cost_for_level(ctx.accounts.farm_space.level)?;
    
    // Validate user has sufficient tokens
    validate_token_balance(user_token_account, upgrade_cost)?;
    
    // Burn tokens for upgrade payment
    burn_upgrade_payment(&ctx, upgrade_cost)?;
    
    // Perform instant upgrade
    let farm_space = &mut ctx.accounts.farm_space;
    let old_level = farm_space.level;
    let old_capacity = farm_space.capacity;
    
    farm_space.level += 1;
    farm_space.capacity = FarmSpace::get_capacity_for_level(farm_space.level);
    
    msg!("Farm space upgraded instantly for user: {}, from level {} to {}, capacity increased from {} to {} seeds, cost: {} tokens", 
         ctx.accounts.user.key(), 
         old_level, 
         farm_space.level,
         old_capacity,
         farm_space.capacity,
         upgrade_cost);
    Ok(())
}

/// Validate that farm space can be upgraded
fn validate_upgrade_prerequisites(farm_space: &FarmSpace) -> Result<()> {
    require!(farm_space.level < 5, GameError::MaxLevelReached);
    Ok(())
}

/// Burn tokens for upgrade payment
fn burn_upgrade_payment(ctx: &Context<UpgradeFarmSpace>, upgrade_cost: u64) -> Result<()> {
    let burn_accounts = Burn {
        mint: ctx.accounts.reward_mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, burn_accounts);
    token::burn(cpi_ctx, upgrade_cost)
}


