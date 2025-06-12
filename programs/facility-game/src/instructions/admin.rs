use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use mpl_token_metadata::types::DataV2;
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
    
    /// Token metadata account (optional for test environments)
    /// CHECK: Metadata account will be created by Metaplex if available
    pub metadata_account: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
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

/// Create WEED token mint with metadata
pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
    let mint_authority = &ctx.accounts.mint_authority;
    
    msg!("WEED token mint created with PDA authority: {}", mint_authority.key());
    
    // Attempt to create token metadata if Metaplex is available
    if let Err(err) = create_token_metadata(&ctx) {
        msg!("Warning: Metadata creation failed (test environment?): {:?}", err);
    }
    
    Ok(())
}

/// Helper function to create token metadata
fn create_token_metadata(ctx: &Context<CreateRewardMint>) -> Result<()> {
    let token_data = DataV2 {
        name: "Weed Token".to_string(),
        symbol: "WEED".to_string(),
        uri: "".to_string(), // Can be updated later with IPFS metadata
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };
    
    let bump = ctx.bumps.mint_authority;
    let seeds = &[b"mint_authority".as_ref(), &[bump]];
    let signer_seeds = &[&seeds[..]];
    
    let create_metadata_ix = mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
        .metadata(ctx.accounts.metadata_account.key())
        .mint(ctx.accounts.reward_mint.key())
        .mint_authority(ctx.accounts.mint_authority.key())
        .payer(ctx.accounts.admin.key())
        .update_authority(ctx.accounts.mint_authority.key(), true)
        .system_program(ctx.accounts.system_program.key())
        .rent(Some(ctx.accounts.rent.key()))
        .data(token_data)
        .is_mutable(true)
        .instruction();
    
    anchor_lang::solana_program::program::invoke_signed(
        &create_metadata_ix,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    
    msg!("WEED token metadata created successfully");
    Ok(())
}

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