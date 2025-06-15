// ===== 農場スペース管理モジュール =====
// このモジュールは農場スペースの購入、アップグレード、管理機能を提供します
// 農場スペースはユーザーが種を植えて報酬を得るための基本単位です

use anchor_lang::prelude::*;
// Note: Token imports removed since manual upgrade functions were removed
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

// Manual farm upgrades have been replaced with automatic pack-based upgrades
// See purchase_seed_pack instruction in seeds.rs for auto-upgrade implementation

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
    initial_seed.reserve = [0; 16];

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

// ===== FARM LEVEL MANAGEMENT =====

/// Context for initializing farm level configuration
#[derive(Accounts)]
pub struct InitializeFarmLevelConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = FarmLevelConfig::DEFAULT_SPACE,
        seeds = [b"farm_level_config"],
        bump
    )]
    pub farm_level_config: Account<'info, FarmLevelConfig>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for updating farm level configuration
#[derive(Accounts)]
pub struct UpdateFarmLevelConfig<'info> {
    #[account(
        mut,
        seeds = [b"farm_level_config"],
        bump
    )]
    pub farm_level_config: Account<'info, FarmLevelConfig>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Context for migrating existing farms to new level system
#[derive(Accounts)]
pub struct MigrateFarmToNewLevels<'info> {
    #[account(
        mut,
        seeds = [b"farm_space", user.key().as_ref()],
        bump,
        constraint = farm_space.owner == user.key() @ GameError::UnauthorizedUser
    )]
    pub farm_space: Account<'info, FarmSpace>,
    
    #[account(
        seeds = [b"farm_level_config"],
        bump
    )]
    pub farm_level_config: Account<'info, FarmLevelConfig>,
    
    #[account(mut)]
    pub user: Signer<'info>,
}

/// Context for viewing farm level configuration
#[derive(Accounts)]
pub struct ViewFarmLevelConfig<'info> {
    #[account(
        seeds = [b"farm_level_config"],
        bump
    )]
    pub farm_level_config: Account<'info, FarmLevelConfig>,
}

/// Initialize farm level configuration with defaults
pub fn initialize_farm_level_config(ctx: Context<InitializeFarmLevelConfig>) -> Result<()> {
    let config = &mut ctx.accounts.farm_level_config;
    
    // Initialize with current 5-level system
    config.max_level = 5;
    
    // Initialize arrays with zeros then set values
    config.capacities = [0; 20];
    config.capacities[0..5].copy_from_slice(&[4, 6, 8, 10, 12]);
    
    config.upgrade_thresholds = [0; 20];
    config.upgrade_thresholds[0..5].copy_from_slice(&[0, 30, 100, 300, 500]);
    
    // Initialize level names
    config.level_names = [[0; 32]; 20];
    let names = ["Starter Farm", "Growing Farm", "Expanding Farm", "Advanced Farm", "Master Farm"];
    for (i, name) in names.iter().enumerate() {
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(32);
        config.level_names[i][0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
    }
    config.created_at = Clock::get()?.unix_timestamp;
    config.updated_at = Clock::get()?.unix_timestamp;
    config.reserve = [0; 32];
    
    msg!("Farm level config initialized with {} levels", config.max_level);
    Ok(())
}

/// Update farm level configuration
pub fn update_farm_level_config(
    ctx: Context<UpdateFarmLevelConfig>,
    max_level: u8,
    capacities: Vec<u8>,
    upgrade_thresholds: Vec<u32>,
    level_names: Option<Vec<String>>,
) -> Result<()> {
    let config = &mut ctx.accounts.farm_level_config;
    
    // Validation
    require!(max_level >= 1 && max_level <= 20, GameError::InvalidConfig);
    require!(capacities.len() == max_level as usize, GameError::InvalidConfig);
    require!(upgrade_thresholds.len() == max_level as usize, GameError::InvalidConfig);
    
    // Validate ascending order for capacities
    for i in 1..capacities.len() {
        require!(capacities[i] > capacities[i-1], GameError::InvalidConfig);
    }
    
    // Validate ascending order for thresholds
    for i in 1..upgrade_thresholds.len() {
        require!(upgrade_thresholds[i] > upgrade_thresholds[i-1], GameError::InvalidConfig);
    }
    
    // Validate level names if provided
    if let Some(names) = &level_names {
        require!(names.len() == max_level as usize, GameError::InvalidConfig);
        for name in names {
            require!(name.len() <= 32, GameError::InvalidConfig);
        }
    }
    
    // Update configuration
    config.max_level = max_level;
    
    // Initialize arrays with zeros then copy values
    config.capacities = [0; 20];
    config.capacities[0..capacities.len()].copy_from_slice(&capacities);
    
    config.upgrade_thresholds = [0; 20];
    config.upgrade_thresholds[0..upgrade_thresholds.len()].copy_from_slice(&upgrade_thresholds);
    
    if let Some(names) = level_names {
        config.level_names = [[0; 32]; 20];
        for (i, name) in names.iter().enumerate().take(20) {
            let name_bytes = name.as_bytes();
            let copy_len = name_bytes.len().min(32);
            config.level_names[i][0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
        }
    }
    
    config.updated_at = Clock::get()?.unix_timestamp;
    
    msg!("Farm level config updated: max_level={}", max_level);
    Ok(())
}

/// Migrate existing farms to new level configuration
pub fn migrate_farm_to_new_levels(ctx: Context<MigrateFarmToNewLevels>) -> Result<()> {
    let farm = &mut ctx.accounts.farm_space;
    let level_config = &ctx.accounts.farm_level_config;
    
    let old_level = farm.level;
    let old_capacity = farm.capacity;
    
    // Adjust level if it exceeds new maximum
    if farm.level > level_config.max_level {
        farm.level = level_config.max_level;
    }
    
    // Recalculate capacity based on new configuration
    if farm.level > 0 && (farm.level as usize) <= level_config.capacities.len() {
        farm.capacity = level_config.capacities[(farm.level - 1) as usize];
    }
    
    // If capacity decreased and farm has more seeds than new capacity, need to handle overflow
    if farm.capacity < old_capacity && farm.seed_count > farm.capacity {
        msg!("Warning: Farm capacity reduced from {} to {}. Seed count: {}", 
             old_capacity, farm.capacity, farm.seed_count);
        // Note: In production, you might want to:
        // 1. Move excess seeds to storage
        // 2. Compensate the user
        // 3. Or gradually reduce through natural harvest
    }
    
    msg!("Farm migrated: level {} -> {}, capacity {} -> {}", 
         old_level, farm.level, old_capacity, farm.capacity);
    
    Ok(())
}

/// Return type for level information queries
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FarmLevelInfo {
    pub level: u8,
    pub capacity: u8,
    pub upgrade_threshold: u32,
    pub name: String,
    pub is_max_level: bool,
    pub next_threshold: Option<u32>,
}

/// Get current farm level configuration (view function)
pub fn get_farm_level_info(
    ctx: Context<ViewFarmLevelConfig>,
    level: Option<u8>,
) -> Result<FarmLevelInfo> {
    let config = &ctx.accounts.farm_level_config;
    
    if let Some(l) = level {
        require!(l >= 1 && l <= config.max_level, GameError::InvalidFarmLevel);
        
        let capacity = config.capacities[(l - 1) as usize];
        let threshold = config.upgrade_thresholds[(l - 1) as usize];
        let name_bytes = &config.level_names[(l - 1) as usize];
        let name = std::str::from_utf8(name_bytes).unwrap_or("Unknown").trim_end_matches('\0').to_string();
        
        Ok(FarmLevelInfo {
            level: l,
            capacity,
            upgrade_threshold: threshold,
            name,
            is_max_level: l == config.max_level,
            next_threshold: if l < config.max_level {
                Some(config.upgrade_thresholds[l as usize])
            } else {
                None
            },
        })
    } else {
        // Return summary info
        Ok(FarmLevelInfo {
            level: 0, // Indicates summary
            capacity: config.capacities[config.capacities.len() - 1], // Max capacity
            upgrade_threshold: 0,
            name: format!("Farm System ({} levels)", config.max_level),
            is_max_level: false,
            next_threshold: None,
        })
    }
}
