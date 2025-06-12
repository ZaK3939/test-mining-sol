use anchor_lang::prelude::*;
use crate::state::*;
use crate::state_meteora::*;
use crate::error::*;

/// Meteora統合の管理者制御機能
/// プール設定、緊急制御、統計管理等

// ===== METEORA CONFIG INITIALIZATION =====

#[derive(Accounts)]
pub struct InitializeMeteoraConfig<'info> {
    /// Meteora設定アカウント (新規作成)
    #[account(
        init,
        payer = admin,
        space = MeteoraConfig::LEN,
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    /// システム設定 (管理者確認用)
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    /// 管理者
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Meteora設定の初期化
pub fn initialize_meteora_config(ctx: Context<InitializeMeteoraConfig>) -> Result<()> {
    let meteora_config = &mut ctx.accounts.meteora_config;
    let admin = ctx.accounts.admin.key();
    
    meteora_config.initialize_default(admin)?;
    
    msg!("Meteora configuration initialized by admin: {}", admin);
    Ok(())
}

// ===== POOL CONFIGURATION =====

#[derive(Accounts)]
pub struct AdminConfigureDlmmPool<'info> {
    /// Meteora設定アカウント
    #[account(
        mut,
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    /// システム設定 (管理者確認用)
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    /// 管理者
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// DLMM プール設定
pub fn configure_dlmm_pool(
    ctx: Context<AdminConfigureDlmmPool>,
    pool_config: DlmmPoolConfig,
) -> Result<()> {
    let meteora_config = &mut ctx.accounts.meteora_config;
    
    // 設定前検証
    validate_pool_config(&pool_config)?;
    
    // プール設定更新
    meteora_config.dlmm_pool = pool_config.pool_address;
    meteora_config.pool_weed_reserve = pool_config.weed_reserve;
    meteora_config.pool_sol_reserve = pool_config.sol_reserve;
    meteora_config.pool_authority = pool_config.pool_authority;
    meteora_config.oracle_account = pool_config.oracle_account;
    
    // 設定後検証
    meteora_config.validate_settings()?;
    
    msg!("DLMM pool configured: {}", pool_config.pool_address);
    
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DlmmPoolConfig {
    pub pool_address: Pubkey,
    pub weed_reserve: Pubkey,
    pub sol_reserve: Pubkey,
    pub pool_authority: Pubkey,
    pub oracle_account: Option<Pubkey>,
}

fn validate_pool_config(config: &DlmmPoolConfig) -> Result<()> {
    require!(config.pool_address != Pubkey::default(), GameError::InvalidConfig);
    require!(config.weed_reserve != Pubkey::default(), GameError::InvalidConfig);
    require!(config.sol_reserve != Pubkey::default(), GameError::InvalidConfig);
    require!(config.pool_authority != Pubkey::default(), GameError::InvalidConfig);
    
    Ok(())
}

// ===== CONVERSION SETTINGS =====

#[derive(Accounts)]
pub struct UpdateConversionSettings<'info> {
    #[account(
        mut,
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// 変換設定の更新
pub fn update_conversion_settings(
    ctx: Context<UpdateConversionSettings>,
    settings: ConversionSettings,
) -> Result<()> {
    let meteora_config = &mut ctx.accounts.meteora_config;
    
    // 設定更新
    if let Some(enabled) = settings.auto_conversion_enabled {
        meteora_config.auto_conversion_enabled = enabled;
    }
    
    if let Some(min_amount) = settings.min_conversion_amount {
        meteora_config.min_conversion_amount = min_amount;
    }
    
    if let Some(slippage) = settings.default_slippage_bps {
        meteora_config.default_slippage_bps = slippage;
    }
    
    if let Some(max_slippage) = settings.max_slippage_bps {
        meteora_config.max_slippage_bps = max_slippage;
    }
    
    if let Some(interval) = settings.min_conversion_interval {
        meteora_config.min_conversion_interval = interval;
    }
    
    // 設定検証
    meteora_config.validate_settings()?;
    
    msg!("Conversion settings updated by admin");
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct ConversionSettings {
    pub auto_conversion_enabled: Option<bool>,
    pub min_conversion_amount: Option<u64>,
    pub default_slippage_bps: Option<u16>,
    pub max_slippage_bps: Option<u16>,
    pub min_conversion_interval: Option<i64>,
}

// ===== EMERGENCY CONTROLS =====

#[derive(Accounts)]
pub struct EmergencyControl<'info> {
    #[account(
        mut,
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// 緊急停止/再開制御
pub fn emergency_pause_toggle(
    ctx: Context<EmergencyControl>,
    pause: bool,
) -> Result<()> {
    let meteora_config = &mut ctx.accounts.meteora_config;
    
    meteora_config.emergency_pause = pause;
    
    let action = if pause { "paused" } else { "resumed" };
    msg!("Emergency {}: Auto-conversion {}", action, action);
    
    emit!(EmergencyAction {
        admin: ctx.accounts.admin.key(),
        action: if pause { "pause" } else { "resume" }.to_string(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// 緊急時の手動変換実行
pub fn emergency_manual_conversion(
    ctx: Context<EmergencyControl>,
    force_conversion: bool,
) -> Result<()> {
    let meteora_config = &mut ctx.accounts.meteora_config;
    
    if force_conversion {
        // 緊急時の強制変換許可
        meteora_config.emergency_pause = false;
        meteora_config.min_conversion_interval = 0; // 時間制限無効化
        
        msg!("Emergency manual conversion authorized");
        
        emit!(EmergencyAction {
            admin: ctx.accounts.admin.key(),
            action: "force_conversion".to_string(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    
    Ok(())
}

// ===== STATISTICS AND MONITORING =====

#[derive(Accounts)]
pub struct ViewMeteoraStats<'info> {
    #[account(
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    #[account(
        seeds = [b"fee_pool"],
        bump
    )]
    pub fee_pool: Account<'info, FeePool>,
}

/// Meteora統計情報の表示
pub fn view_meteora_stats(ctx: Context<ViewMeteoraStats>) -> Result<()> {
    let meteora_config = &ctx.accounts.meteora_config;
    let fee_pool = &ctx.accounts.fee_pool;
    
    msg!("=== Meteora Statistics ===");
    msg!("Pool: {}", meteora_config.dlmm_pool);
    msg!("Auto-conversion enabled: {}", meteora_config.auto_conversion_enabled);
    msg!("Emergency pause: {}", meteora_config.emergency_pause);
    msg!("Total conversions: {}", meteora_config.total_conversions);
    msg!("Total WEED converted: {}", meteora_config.total_weed_converted);
    msg!("Total SOL received: {}", meteora_config.total_sol_received);
    msg!("Accumulated fees: {}", fee_pool.accumulated_fees);
    msg!("Last conversion: {}", meteora_config.last_conversion_time);
    
    // 平均変換レート計算
    if meteora_config.total_conversions > 0 {
        let avg_weed_per_conversion = meteora_config.total_weed_converted / meteora_config.total_conversions;
        let avg_sol_per_conversion = meteora_config.total_sol_received / meteora_config.total_conversions;
        
        msg!("Average per conversion: {} WEED → {} SOL", 
             avg_weed_per_conversion, avg_sol_per_conversion);
    }
    
    Ok(())
}

// ===== POOL HEALTH MONITORING =====

#[derive(Accounts)]
pub struct MonitorPoolHealth<'info> {
    #[account(
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    /// DLMM pool account
    /// CHECK: Pool validation in instruction
    pub dlmm_pool: UncheckedAccount<'info>,
    
    /// Pool reserves
    /// CHECK: Reserve validation in instruction
    pub pool_weed_reserve: UncheckedAccount<'info>,
    
    /// CHECK: Reserve validation in instruction
    pub pool_sol_reserve: UncheckedAccount<'info>,
}

/// プール健全性監視
pub fn monitor_pool_health(ctx: Context<MonitorPoolHealth>) -> Result<()> {
    let meteora_config = &ctx.accounts.meteora_config;
    
    // プール設定チェック
    require!(
        meteora_config.dlmm_pool == ctx.accounts.dlmm_pool.key(),
        GameError::InvalidConfig
    );
    
    // プール状態取得 (モック実装)
    let weed_balance = get_token_balance(&ctx.accounts.pool_weed_reserve)?;
    let sol_balance = get_token_balance(&ctx.accounts.pool_sol_reserve)?;
    
    // 健全性判定
    let min_weed_liquidity = 10_000 * 1_000_000; // 10K WEED
    let min_sol_liquidity = 20 * 1_000_000_000;   // 20 SOL
    
    let is_healthy = weed_balance >= min_weed_liquidity && sol_balance >= min_sol_liquidity;
    
    msg!("Pool health check:");
    msg!("WEED balance: {} (min: {})", weed_balance, min_weed_liquidity);
    msg!("SOL balance: {} (min: {})", sol_balance, min_sol_liquidity);
    msg!("Pool healthy: {}", is_healthy);
    
    if !is_healthy {
        emit!(PoolHealthWarning {
            pool: meteora_config.dlmm_pool,
            weed_balance,
            sol_balance,
            is_healthy,
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    
    Ok(())
}

// モック関数 (実装時に実際のトークンアカウント読み取りに置き換え)
fn get_token_balance(_account: &UncheckedAccount) -> Result<u64> {
    // TODO: 実際のトークンアカウント残高取得
    Ok(50_000 * 1_000_000) // 50K トークン (モック)
}

// ===== BATCH OPERATIONS =====

#[derive(Accounts)]
pub struct BatchConfigUpdate<'info> {
    #[account(
        mut,
        seeds = [b"meteora_config"],
        bump
    )]
    pub meteora_config: Account<'info, MeteoraConfig>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// バッチ設定更新
pub fn batch_config_update(
    ctx: Context<BatchConfigUpdate>,
    pool_config: Option<DlmmPoolConfig>,
    conversion_settings: Option<ConversionSettings>,
    emergency_settings: Option<EmergencySettings>,
) -> Result<()> {
    let meteora_config = &mut ctx.accounts.meteora_config;
    
    // プール設定更新
    if let Some(pool_cfg) = pool_config {
        validate_pool_config(&pool_cfg)?;
        meteora_config.dlmm_pool = pool_cfg.pool_address;
        meteora_config.pool_weed_reserve = pool_cfg.weed_reserve;
        meteora_config.pool_sol_reserve = pool_cfg.sol_reserve;
        meteora_config.pool_authority = pool_cfg.pool_authority;
        meteora_config.oracle_account = pool_cfg.oracle_account;
    }
    
    // 変換設定更新
    if let Some(conv_settings) = conversion_settings {
        if let Some(enabled) = conv_settings.auto_conversion_enabled {
            meteora_config.auto_conversion_enabled = enabled;
        }
        if let Some(min_amount) = conv_settings.min_conversion_amount {
            meteora_config.min_conversion_amount = min_amount;
        }
        if let Some(slippage) = conv_settings.default_slippage_bps {
            meteora_config.default_slippage_bps = slippage;
        }
    }
    
    // 緊急設定更新
    if let Some(emergency_cfg) = emergency_settings {
        if let Some(pause) = emergency_cfg.emergency_pause {
            meteora_config.emergency_pause = pause;
        }
    }
    
    // 全体設定検証
    meteora_config.validate_settings()?;
    
    msg!("Batch configuration update completed");
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct EmergencySettings {
    pub emergency_pause: Option<bool>,
}

// ===== EVENTS =====

#[event]
pub struct EmergencyAction {
    pub admin: Pubkey,
    pub action: String,
    pub timestamp: i64,
}

#[event] 
pub struct PoolHealthWarning {
    pub pool: Pubkey,
    pub weed_balance: u64,
    pub sol_balance: u64,
    pub is_healthy: bool,
    pub timestamp: i64,
}

#[event]
pub struct ConfigurationUpdated {
    pub admin: Pubkey,
    pub config_type: String,
    pub timestamp: i64,
}