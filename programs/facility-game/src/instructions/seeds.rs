use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::state::*;
use crate::error::*;
use crate::utils::*;

/// Context for purchasing mystery seed pack
#[derive(Accounts)]
pub struct PurchaseSeedPack<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        init,
        payer = user,
        space = SeedPack::LEN,
        seeds = [b"seed_pack", user.key().as_ref(), config.seed_pack_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub seed_pack: Account<'info, SeedPack>,
    
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
    pub system_program: Program<'info, System>,
}

/// Context for initializing seed storage
#[derive(Accounts)]
pub struct InitializeSeedStorage<'info> {
    #[account(
        init,
        payer = user,
        space = SeedStorage::LEN,
        seeds = [b"seed_storage", user.key().as_ref()],
        bump
    )]
    pub seed_storage: Account<'info, SeedStorage>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for opening seed pack
#[derive(Accounts)]
#[instruction(seed_id: u64)]
pub struct OpenSeedPack<'info> {
    #[account(
        mut,
        seeds = [b"seed_pack", user.key().as_ref(), seed_pack.pack_id.to_le_bytes().as_ref()],
        bump,
        constraint = seed_pack.purchaser == user.key()
    )]
    pub seed_pack: Account<'info, SeedPack>,
    
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        mut,
        seeds = [b"seed_storage", user.key().as_ref()],
        bump
    )]
    pub seed_storage: Account<'info, SeedStorage>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for planting seed in farm space
#[derive(Accounts)]
#[instruction(seed_id: u64)]
pub struct PlantSeed<'info> {
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
        seeds = [b"seed", user.key().as_ref(), seed_id.to_le_bytes().as_ref()],
        bump,
        constraint = seed.owner == user.key()
    )]
    pub seed: Account<'info, Seed>,
    
    #[account(
        mut,
        seeds = [b"global_stats"],
        bump
    )]
    pub global_stats: Account<'info, GlobalStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
}

/// Context for removing seed from farm space
#[derive(Accounts)]
#[instruction(seed_id: u64)]
pub struct RemoveSeed<'info> {
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
        seeds = [b"seed", user.key().as_ref(), seed_id.to_le_bytes().as_ref()],
        bump,
        constraint = seed.owner == user.key()
    )]
    pub seed: Account<'info, Seed>,
    
    #[account(
        mut,
        seeds = [b"global_stats"],
        bump
    )]
    pub global_stats: Account<'info, GlobalStats>,
    
    #[account(mut)]
    pub user: Signer<'info>,
}

/// Purchase mystery seed pack (300 $WEED) - 100% burned
pub fn purchase_seed_pack(ctx: Context<PurchaseSeedPack>, quantity: u8) -> Result<()> {
    // Validate quantity
    require!(quantity > 0 && quantity <= 100, GameError::InvalidQuantity);
    
    let user_token_account = &ctx.accounts.user_token_account;
    
    // Calculate total cost
    let total_cost = ctx.accounts.config.seed_pack_cost
        .checked_mul(quantity as u64)
        .ok_or(GameError::CalculationOverflow)?;
    
    // Validate user has sufficient tokens
    validate_token_balance(user_token_account, total_cost)?;
    
    // Burn tokens (100% burn mechanism)
    burn_seed_pack_payment(&ctx, total_cost)?;
    
    // Initialize seed pack
    let current_time = Clock::get()?.unix_timestamp;
    let pack_counter = ctx.accounts.config.seed_pack_counter;
    initialize_seed_pack(
        &mut ctx.accounts.seed_pack,
        ctx.accounts.user.key(),
        total_cost,
        current_time,
        ctx.accounts.user.key().as_ref(),
        pack_counter,
    );
    
    // Update global counter
    ctx.accounts.config.seed_pack_counter += 1;
    
    msg!("Seed pack purchased: pack_id {}, quantity: {}, cost: {} WEED burned, random_seed: {}", 
         ctx.accounts.seed_pack.pack_id, quantity, total_cost, ctx.accounts.seed_pack.random_seed);
    
    Ok(())
}

/// Burn tokens for seed pack payment
fn burn_seed_pack_payment(ctx: &Context<PurchaseSeedPack>, total_cost: u64) -> Result<()> {
    let burn_accounts = Burn {
        mint: ctx.accounts.reward_mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, burn_accounts);
    token::burn(cpi_ctx, total_cost)
}

/// Initialize seed pack with proper data
fn initialize_seed_pack(
    seed_pack: &mut SeedPack,
    purchaser: Pubkey,
    cost_paid: u64,
    current_time: i64,
    user_key: &[u8],
    pack_counter: u64,
) {
    seed_pack.purchaser = purchaser;
    seed_pack.purchased_at = current_time;
    seed_pack.cost_paid = cost_paid;
    seed_pack.is_opened = false;
    seed_pack.random_seed = generate_seed_pack_random(current_time, user_key, pack_counter);
    seed_pack.pack_id = pack_counter;
    seed_pack.reserve = [0; 32];
}

/// Initialize seed storage for a user
pub fn initialize_seed_storage_instruction(ctx: Context<InitializeSeedStorage>) -> Result<()> {
    let seed_storage = &mut ctx.accounts.seed_storage;
    
    initialize_seed_storage(seed_storage, ctx.accounts.user.key());
    
    msg!("Seed storage initialized for user: {}", ctx.accounts.user.key());
    Ok(())
}

/// Open seed pack to reveal seeds (creates individual Seed PDAs)
pub fn open_seed_pack(ctx: Context<OpenSeedPack>, quantity: u8) -> Result<()> {
    let seed_pack = &mut ctx.accounts.seed_pack;
    let config = &mut ctx.accounts.config;
    let seed_storage = &mut ctx.accounts.seed_storage;
    
    // Validate pack can be opened
    require!(!seed_pack.is_opened, GameError::SeedPackAlreadyOpened);
    require!(quantity > 0 && quantity <= 100, GameError::InvalidQuantity);
    
    // Validate seed storage is properly initialized
    require!(seed_storage.owner == ctx.accounts.user.key(), GameError::SeedStorageNotInitialized);
    
    // Generate seeds using probability system
    generate_seeds_from_pack(seed_pack, config, seed_storage, quantity)?;
    
    // Mark pack as opened
    seed_pack.is_opened = true;
    
    msg!("Seed pack opened: {} seeds generated for user: {}", 
         quantity, ctx.accounts.user.key());
    
    Ok(())
}

/// Generate seeds from pack using probability table
fn generate_seeds_from_pack(
    seed_pack: &SeedPack,
    config: &mut Config,
    seed_storage: &mut SeedStorage,
    quantity: u8,
) -> Result<()> {
    let base_random = seed_pack.random_seed;
    
    for i in 0..quantity {
        let seed_random = base_random
            .wrapping_add(i as u64)
            .wrapping_mul(7919); // Prime number for better distribution
            
        let seed_type = SeedType::from_random(seed_random);
        let seed_id = config.seed_counter;
        
        // Add seed to storage
        add_seed_to_storage(seed_storage, seed_id)?;
        config.seed_counter += 1;
        
        msg!("Seed generated: ID {}, Type: {:?}, Grow Power: {}", 
             seed_id, seed_type, seed_type.get_grow_power());
    }
    
    Ok(())
}

/// Plant seed in farm space
pub fn plant_seed(ctx: Context<PlantSeed>, seed_id: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let farm_space_key = ctx.accounts.farm_space.key();
    let seed_grow_power = ctx.accounts.seed.grow_power;
    
    // Validate planting prerequisites
    validate_planting_prerequisites(&ctx.accounts.farm_space, &ctx.accounts.seed, ctx.accounts.user.key())?;
    
    // Plant the seed
    plant_seed_in_farm(&mut ctx.accounts.seed, farm_space_key);
    
    // Update all statistics
    update_stats_on_plant(&mut ctx.accounts.farm_space, &mut ctx.accounts.user_state, &mut ctx.accounts.global_stats, seed_grow_power, current_time)?;
    
    msg!("Seed planted: ID {}, Grow Power: {}, Farm total: {}, User total: {}", 
         seed_id, seed_grow_power, ctx.accounts.farm_space.total_grow_power, ctx.accounts.user_state.total_grow_power);
    
    Ok(())
}

/// Validate all prerequisites for planting a seed
fn validate_planting_prerequisites(
    farm_space: &Account<FarmSpace>,
    seed: &Seed,
    user_key: Pubkey,
) -> Result<()> {
    validate_farm_space_capacity(farm_space)?;
    require!(!seed.is_planted, GameError::SeedAlreadyPlanted);
    require!(seed.owner == user_key, GameError::NotSeedOwner);
    Ok(())
}

/// Plant seed in the farm space
fn plant_seed_in_farm(seed: &mut Seed, farm_space_key: Pubkey) {
    seed.is_planted = true;
    seed.planted_farm_space = Some(farm_space_key);
}

/// Update statistics when seed is planted
fn update_stats_on_plant(
    farm_space: &mut FarmSpace,
    user_state: &mut UserState,
    global_stats: &mut GlobalStats,
    grow_power: u64,
    current_time: i64,
) -> Result<()> {
    // Update farm space
    farm_space.seed_count += 1;
    farm_space.total_grow_power += grow_power;
    
    // Update user stats
    user_state.total_grow_power += grow_power;
    
    // Update global stats
    update_global_grow_power(global_stats, grow_power as i64, current_time)?;
    
    Ok(())
}

/// Remove seed from farm space
pub fn remove_seed(ctx: Context<RemoveSeed>, seed_id: u64) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let seed_grow_power = ctx.accounts.seed.grow_power;
    
    // Validate removal prerequisites
    validate_removal_prerequisites(&ctx.accounts.farm_space, &ctx.accounts.seed, ctx.accounts.user.key())?;
    
    // Remove the seed from farm space
    remove_seed_from_farm(&mut ctx.accounts.seed);
    
    // Update all statistics
    update_stats_on_removal(&mut ctx.accounts.farm_space, &mut ctx.accounts.user_state, &mut ctx.accounts.global_stats, seed_grow_power, current_time)?;
    
    msg!("Seed removed: ID {}, Grow Power: {}, Farm total: {}, User total: {}", 
         seed_id, seed_grow_power, ctx.accounts.farm_space.total_grow_power, ctx.accounts.user_state.total_grow_power);
    
    Ok(())
}

/// Validate all prerequisites for removing a seed
fn validate_removal_prerequisites(
    farm_space: &Account<FarmSpace>,
    seed: &Seed,
    user_key: Pubkey,
) -> Result<()> {
    require!(seed.is_planted, GameError::SeedNotPlanted);
    require!(
        seed.planted_farm_space == Some(farm_space.key()),
        GameError::SeedNotInThisFarmSpace
    );
    require!(seed.owner == user_key, GameError::NotSeedOwner);
    Ok(())
}

/// Remove seed from the farm space
fn remove_seed_from_farm(seed: &mut Seed) {
    seed.is_planted = false;
    seed.planted_farm_space = None;
}

/// Update statistics when seed is removed
fn update_stats_on_removal(
    farm_space: &mut FarmSpace,
    user_state: &mut UserState,
    global_stats: &mut GlobalStats,
    grow_power: u64,
    current_time: i64,
) -> Result<()> {
    // Update farm space
    farm_space.seed_count -= 1;
    farm_space.total_grow_power -= grow_power;
    
    // Update user stats
    user_state.total_grow_power -= grow_power;
    
    // Update global stats
    update_global_grow_power(global_stats, -(grow_power as i64), current_time)?;
    
    Ok(())
}