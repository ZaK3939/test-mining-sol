use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use spl_token_2022::extension::transfer_fee::instruction::initialize_transfer_fee_config;
// Removed unused DataV2 import
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

/// Context for creating reward token mint with transfer fee extension
#[derive(Accounts)]
pub struct CreateRewardMint<'info> {
    /// CHECK: Reward mint account with calculated space for Token2022 + TransferFee extension
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: UncheckedAccount<'info>,
    
    /// @dev Set mint authority as PDA
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    /// Transfer fee config authority (same as mint authority)
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: transfer fee config authority PDA
    pub transfer_fee_config_authority: UncheckedAccount<'info>,
    
    /// Withdraw withheld authority (treasury)
    /// CHECK: This should be the treasury pubkey
    pub withdraw_withheld_authority: UncheckedAccount<'info>,
    
    /// Token metadata account (optional for test environments)
    /// CHECK: Metadata account will be created by Metaplex if available
    pub metadata_account: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    
    /// Token Metadata Program (optional for test environments)
    /// CHECK: Token metadata program
    pub token_metadata_program: UncheckedAccount<'info>,
}

/// Initialize system configuration with secure defaults
pub fn initialize_config(
    ctx: Context<InitializeConfig>,
    base_rate: Option<u64>,
    halving_interval: Option<i64>,
    treasury: Pubkey,
    protocol_referral_address: Option<Pubkey>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Set core economic parameters
    config.base_rate = base_rate.unwrap_or(Config::DEFAULT_BASE_RATE);
    config.halving_interval = halving_interval.unwrap_or(Config::DEFAULT_HALVING_INTERVAL);
    config.next_halving_time = current_time + config.halving_interval;
    
    // Set authority and treasury
    config.admin = ctx.accounts.admin.key();
    config.treasury = treasury;
    
    // Set game economy constants
    config.seed_pack_cost = 300 * 1_000_000; // 300 WEED with 6 decimals
    config.farm_space_cost_sol = Config::DEFAULT_FARM_SPACE_COST;
    config.max_invite_limit = 5;
    config.trading_fee_percentage = 2;
    config.protocol_referral_address = protocol_referral_address.unwrap_or(Pubkey::default());
    
    // Initialize counters
    config.seed_counter = 0;
    config.seed_pack_counter = 0;
    
    // Initialize supply tracking
    config.total_supply_minted = 0;
    
    // Set operator address (with unlimited invites)
    config.operator = "43eUMnsf1QoFmE2ZkHxbXxZCAJht7pPpFFPUYicUYbjJ".parse::<Pubkey>().unwrap();
    
    // Zero out reserved space
    config.reserve = [0; 2];
    
    msg!("System config initialized: rate={}, halving={}s, treasury={}", 
         config.base_rate, config.halving_interval, treasury);
    
    Ok(())
}

/// Create WEED token mint with transfer fee extension and metadata
pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
    use spl_token_2022::instruction;
    use anchor_spl::token_2022::spl_token_2022::extension::ExtensionType;
    use anchor_spl::token_2022::spl_token_2022::state::Mint;
    
    let mint_authority = &ctx.accounts.mint_authority;
    let transfer_fee_config_authority = &ctx.accounts.transfer_fee_config_authority;
    let withdraw_withheld_authority = &ctx.accounts.withdraw_withheld_authority;
    
    // Calculate required space for mint with transfer fee extension
    let space = ExtensionType::try_calculate_account_len::<Mint>(
        &[ExtensionType::TransferFeeConfig]
    )?;
    let lamports = ctx.accounts.rent.minimum_balance(space);
    
    // Create account instruction with proper space
    let create_account_ix = anchor_lang::solana_program::system_instruction::create_account(
        &ctx.accounts.admin.key(),
        &ctx.accounts.reward_mint.key(),
        lamports,
        space as u64,
        &ctx.accounts.token_program.key(),
    );
    
    // Initialize transfer fee extension
    let transfer_fee_basis_points = 200u16; // 2.00%
    let maximum_fee = 1_000_000_000u64; // 1000 WEED max fee
    
    let init_transfer_fee_ix = initialize_transfer_fee_config(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.reward_mint.key(),
        Some(&transfer_fee_config_authority.key()),
        Some(&withdraw_withheld_authority.key()),
        transfer_fee_basis_points,
        maximum_fee,
    )?;
    
    // Initialize mint
    let init_mint_ix = instruction::initialize_mint2(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.reward_mint.key(),
        &mint_authority.key(),
        Some(&mint_authority.key()),
        6, // decimals
    )?;
    
    // Prepare PDA signer seeds
    let reward_mint_bump = ctx.bumps.reward_mint;
    let reward_mint_seeds: &[&[u8]] = &[b"reward_mint", &[reward_mint_bump]];
    
    // Create account with PDA signature
    anchor_lang::solana_program::program::invoke_signed(
        &create_account_ix,
        &[
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[reward_mint_seeds],
    )?;
    
    // Initialize transfer fee extension
    anchor_lang::solana_program::program::invoke(
        &init_transfer_fee_ix,
        &[
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;
    
    // Initialize mint
    anchor_lang::solana_program::program::invoke(
        &init_mint_ix,
        &[
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;
    
    msg!("WEED token mint created with 2% transfer fee using SPL Token 2022: {}", ctx.accounts.reward_mint.key());
    
    // Skip metadata creation for now to avoid signing issues
    msg!("Skipping metadata creation in test environment");
    
    Ok(())
}

// Removed unused create_token_metadata function

/// Context for initializing global stats
#[derive(Accounts)]
pub struct InitializeGlobalStats<'info> {
    #[account(
        init,
        payer = admin,
        space = GlobalStats::LEN,
        seeds = [b"global_stats"],
        bump
    )]
    pub global_stats: Account<'info, GlobalStats>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for initializing fee pool
#[derive(Accounts)]
pub struct InitializeFeePool<'info> {
    #[account(
        init,
        payer = admin,
        space = FeePool::LEN,
        seeds = [b"fee_pool"],
        bump
    )]
    pub fee_pool: Account<'info, FeePool>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Initialize global game statistics
pub fn initialize_global_stats(ctx: Context<InitializeGlobalStats>) -> Result<()> {
    let global_stats = &mut ctx.accounts.global_stats;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Initialize all counters to zero
    global_stats.total_grow_power = 0;
    global_stats.total_farm_spaces = 0;
    
    // Set economic parameters
    global_stats.total_supply = GlobalStats::INITIAL_TOTAL_SUPPLY;
    global_stats.current_rewards_per_second = Config::DEFAULT_BASE_RATE;
    global_stats.last_update_time = current_time;
    
    // Zero out reserved space
    global_stats.reserve = [0; 32];
    
    msg!("Global statistics initialized: supply={}, base_rate={}/sec", 
         global_stats.total_supply, global_stats.current_rewards_per_second);
    
    Ok(())
}

/// Initialize trading fee collection pool
pub fn initialize_fee_pool(ctx: Context<InitializeFeePool>, treasury_address: Pubkey) -> Result<()> {
    let fee_pool = &mut ctx.accounts.fee_pool;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Initialize fee accumulation
    fee_pool.accumulated_fees = 0;
    fee_pool.treasury_address = treasury_address;
    fee_pool.last_collection_time = current_time;
    
    // Zero out reserved space
    fee_pool.reserve = [0; 48];
    
    msg!("Trading fee pool initialized: treasury={}", treasury_address);
    
    Ok(())
}

/// Context for updating config settings
#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Update configuration settings (admin only)
pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_operator: Option<Pubkey>,
    new_base_rate: Option<u64>,
    new_halving_interval: Option<i64>,
    new_treasury: Option<Pubkey>,
    new_max_invite_limit: Option<u8>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    // Update operator if provided
    if let Some(operator) = new_operator {
        config.operator = operator;
        msg!("Config updated: new operator={}", operator);
    }
    
    // Update base rate if provided
    if let Some(base_rate) = new_base_rate {
        config.base_rate = base_rate;
        msg!("Config updated: new base_rate={}", base_rate);
    }
    
    // Update halving interval if provided
    if let Some(halving_interval) = new_halving_interval {
        config.halving_interval = halving_interval;
        // Also update next halving time
        let current_time = Clock::get()?.unix_timestamp;
        config.next_halving_time = current_time + halving_interval;
        msg!("Config updated: new halving_interval={}", halving_interval);
    }
    
    // Update treasury if provided
    if let Some(treasury) = new_treasury {
        config.treasury = treasury;
        msg!("Config updated: new treasury={}", treasury);
    }
    
    // Update max invite limit if provided
    if let Some(max_invite_limit) = new_max_invite_limit {
        config.max_invite_limit = max_invite_limit;
        msg!("Config updated: new max_invite_limit={}", max_invite_limit);
    }
    
    Ok(())
}

// ===== PROBABILITY TABLE MANAGEMENT =====

/// Context for initializing probability table
#[derive(Accounts)]
pub struct InitializeProbabilityTable<'info> {
    #[account(
        init,
        payer = admin,
        space = ProbabilityTable::LEN,
        seeds = [b"probability_table"],
        bump
    )]
    pub probability_table: Account<'info, ProbabilityTable>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for updating probability table
#[derive(Accounts)]
pub struct UpdateProbabilityTable<'info> {
    #[account(
        mut,
        seeds = [b"probability_table"],
        bump
    )]
    pub probability_table: Account<'info, ProbabilityTable>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Initialize probability table with Table 1 settings
pub fn initialize_probability_table(ctx: Context<InitializeProbabilityTable>) -> Result<()> {
    let probability_table = &mut ctx.accounts.probability_table;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Initialize with Table 1 settings
    let table_data = ProbabilityTable::init_table_1();
    
    // Set each field individually
    probability_table.version = table_data.version;
    probability_table.seed_count = table_data.seed_count;
    probability_table.grow_powers = table_data.grow_powers;
    probability_table.probability_thresholds = table_data.probability_thresholds;
    probability_table.probability_percentages = table_data.probability_percentages;
    probability_table.expected_value = table_data.expected_value;
    probability_table.name = table_data.name;
    probability_table.created_at = current_time;
    probability_table.updated_at = current_time;
    probability_table.reserve = [0; 14];
    
    msg!("Probability table initialized with Table 1 settings (6 seeds)");
    msg!("Expected value: {} GP per pack", probability_table.expected_value);
    
    Ok(())
}

/// Update probability table with new settings
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
    let probability_table = &mut ctx.accounts.probability_table;
    
    // Validate input constraints
    require!(seed_count <= 16, crate::error::GameError::InvalidQuantity);
    require!(grow_powers.len() == seed_count as usize, crate::error::GameError::InvalidQuantity);
    require!(probability_thresholds.len() == seed_count as usize, crate::error::GameError::InvalidQuantity);
    require!(probability_percentages.len() == seed_count as usize, crate::error::GameError::InvalidQuantity);
    require!(name.len() <= 32, crate::error::GameError::InvalidConfig);
    
    // Validate probability thresholds are in ascending order and end at 10000
    for i in 1..probability_thresholds.len() {
        require!(
            probability_thresholds[i] > probability_thresholds[i-1],
            crate::error::GameError::InvalidConfig
        );
    }
    require!(
        probability_thresholds[probability_thresholds.len()-1] == 10000,
        crate::error::GameError::InvalidConfig
    );
    
    // Update table
    probability_table.version = version;
    probability_table.seed_count = seed_count;
    probability_table.expected_value = expected_value;
    probability_table.updated_at = Clock::get()?.unix_timestamp;
    
    // Clear arrays first
    probability_table.grow_powers = [0; 16];
    probability_table.probability_thresholds = [0; 16];
    probability_table.probability_percentages = [0.0; 16];
    probability_table.name = [0; 32];
    
    // Set new values
    for i in 0..seed_count as usize {
        probability_table.grow_powers[i] = grow_powers[i];
        probability_table.probability_thresholds[i] = probability_thresholds[i];
        probability_table.probability_percentages[i] = probability_percentages[i];
    }
    
    // Set name
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len().min(32);
    probability_table.name[0..name_len].copy_from_slice(&name_bytes[0..name_len]);
    
    msg!("Probability table updated to version {}", version);
    msg!("Seed count: {}, Expected value: {} GP", seed_count, expected_value);
    msg!("Table name: {}", name);
    
    Ok(())
}

/// Context for updating seed pack cost
#[derive(Accounts)]
pub struct UpdateSeedPackCost<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        has_one = admin
    )]
    pub config: Account<'info, Config>,
    
    pub admin: Signer<'info>,
}

/// Context for revealing a new seed type
#[derive(Accounts)]
pub struct RevealSeed<'info> {
    #[account(
        mut,
        seeds = [b"probability_table"],
        bump
    )]
    pub probability_table: Account<'info, ProbabilityTable>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Context for updating seed values
#[derive(Accounts)]
pub struct UpdateSeedValues<'info> {
    #[account(
        mut,
        seeds = [b"probability_table"],
        bump
    )]
    pub probability_table: Account<'info, ProbabilityTable>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Update seed pack cost (admin only)
/// Allows dynamic price adjustment for seed packs
pub fn update_seed_pack_cost(
    ctx: Context<UpdateSeedPackCost>,
    new_seed_pack_cost: u64,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    // Validation: Prevent zero cost
    require!(new_seed_pack_cost > 0, crate::error::GameError::InvalidConfig);
    
    // Validation: Prevent excessively high costs (max 10,000 WEED)
    let max_cost = 10_000 * 1_000_000; // 10,000 WEED with 6 decimals
    require!(new_seed_pack_cost <= max_cost, crate::error::GameError::InvalidConfig);
    
    let old_cost = config.seed_pack_cost;
    config.seed_pack_cost = new_seed_pack_cost;
    
    msg!("Seed pack cost updated from {} to {} WEED", 
         old_cost / 1_000_000, 
         new_seed_pack_cost / 1_000_000);
    
    Ok(())
}

/// Reveal a new seed type (admin only)
/// Makes a previously hidden seed type visible to users with its values
pub fn reveal_seed(
    ctx: Context<RevealSeed>,
    seed_index: u8,
    grow_power: u64,
    probability_percentage: f32,
) -> Result<()> {
    let probability_table = &mut ctx.accounts.probability_table;
    
    // Validation: seed index must be valid
    require!(seed_index < 16, crate::error::GameError::InvalidConfig);
    
    // Validation: seed must not already be revealed
    require!(!probability_table.is_seed_revealed(seed_index), crate::error::GameError::InvalidConfig);
    
    // Validation: probability percentage should be reasonable (0-100%)
    require!(probability_percentage >= 0.0 && probability_percentage <= 100.0, crate::error::GameError::InvalidConfig);
    
    // Validation: grow power should be positive
    require!(grow_power > 0, crate::error::GameError::InvalidConfig);
    
    // Reveal the seed
    let success = probability_table.reveal_seed(seed_index, grow_power, probability_percentage);
    require!(success, crate::error::GameError::InvalidConfig);
    
    // Update the table timestamp
    probability_table.updated_at = Clock::get()?.unix_timestamp;
    
    msg!("Seed type {} revealed: grow_power={}, probability={}%", 
         seed_index + 1, grow_power, probability_percentage);
    
    Ok(())
}

/// Update values for an already revealed seed type (admin only)
/// Allows changing grow power and probability for existing revealed seeds
pub fn update_seed_values(
    ctx: Context<UpdateSeedValues>,
    seed_index: u8,
    grow_power: u64,
    probability_percentage: f32,
) -> Result<()> {
    let probability_table = &mut ctx.accounts.probability_table;
    
    // Validation: seed index must be valid
    require!(seed_index < 16, crate::error::GameError::InvalidConfig);
    
    // Validation: seed must already be revealed
    require!(probability_table.is_seed_revealed(seed_index), crate::error::GameError::InvalidConfig);
    
    // Validation: probability percentage should be reasonable (0-100%)
    require!(probability_percentage >= 0.0 && probability_percentage <= 100.0, crate::error::GameError::InvalidConfig);
    
    // Validation: grow power should be positive
    require!(grow_power > 0, crate::error::GameError::InvalidConfig);
    
    // Store old values for logging
    let old_grow_power = probability_table.grow_powers[seed_index as usize];
    let old_probability = probability_table.probability_percentages[seed_index as usize];
    
    // Update the seed values
    let success = probability_table.update_seed_values(seed_index, grow_power, probability_percentage);
    require!(success, crate::error::GameError::InvalidConfig);
    
    // Update the table timestamp
    probability_table.updated_at = Clock::get()?.unix_timestamp;
    
    msg!("Seed type {} values updated:", seed_index + 1);
    msg!("  Grow power: {} -> {}", old_grow_power, grow_power);
    msg!("  Probability: {}% -> {}%", old_probability, probability_percentage);
    
    Ok(())
}

// ===== FARM SPACE COST MANAGEMENT =====

/// Context for updating farm space cost
#[derive(Accounts)]
pub struct UpdateFarmSpaceCost<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
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
    let config = &mut ctx.accounts.config;
    
    // Validation: cost must be positive
    require!(new_farm_space_cost_sol > 0, crate::error::GameError::InvalidAmount);
    
    // Validation: cost must be reasonable (max 10 SOL = 10,000,000,000 lamports)
    require!(
        new_farm_space_cost_sol <= 10_000_000_000,
        crate::error::GameError::InvalidAmount
    );
    
    // Store old value for logging
    let old_cost = config.farm_space_cost_sol;
    
    // Update the farm space cost
    config.farm_space_cost_sol = new_farm_space_cost_sol;
    
    msg!("Farm space cost updated:");
    msg!("  Old cost: {} lamports ({} SOL)", old_cost, old_cost as f64 / 1_000_000_000.0);
    msg!("  New cost: {} lamports ({} SOL)", new_farm_space_cost_sol, new_farm_space_cost_sol as f64 / 1_000_000_000.0);
    
    Ok(())
}