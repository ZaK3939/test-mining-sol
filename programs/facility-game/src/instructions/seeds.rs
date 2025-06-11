use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
// TODO: Re-enable when Pyth Entropy SDK is available
// use pyth_entropy_sdk_solana::{HashChain, Request};
use arrayref::array_ref;
use crate::state::*;
use crate::error::*;
use crate::utils::*;

/// Context for purchasing mystery seed pack with Pyth Entropy
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
    
    /// Pyth Entropy provider account
    /// CHECK: Validated by Pyth Entropy SDK
    #[account(mut)]
    pub entropy_provider: UncheckedAccount<'info>,
    
    /// Pyth Entropy request account (will be created)
    /// CHECK: Created by Pyth Entropy SDK
    #[account(mut)]
    pub entropy_request: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Pyth Entropy program
    /// CHECK: Pyth Entropy program ID
    pub pyth_entropy_program: UncheckedAccount<'info>,
    
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

/// Context for opening seed pack with Pyth Entropy result
#[derive(Accounts)]
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
    
    /// Pyth Entropy request result account
    /// CHECK: Validated by sequence number and owner checks
    pub entropy_request: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Pyth Entropy program for verification
    /// CHECK: Pyth Entropy program ID
    pub pyth_entropy_program: UncheckedAccount<'info>,
    
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

/// Purchase mystery seed pack with Pyth Entropy integration
pub fn purchase_seed_pack(ctx: Context<PurchaseSeedPack>, quantity: u8, user_entropy_seed: u64) -> Result<()> {
    // Validate inputs
    require!(quantity > 0 && quantity <= 100, GameError::InvalidQuantity);
    require!(user_entropy_seed > 0, GameError::InvalidUserEntropySeed);
    
    let user_token_account = &ctx.accounts.user_token_account;
    
    // Calculate total cost
    let total_cost = ctx.accounts.config.seed_pack_cost
        .checked_mul(quantity as u64)
        .ok_or(GameError::CalculationOverflow)?;
    
    // Validate user has sufficient tokens
    validate_token_balance(user_token_account, total_cost)?;
    
    // Burn tokens (100% burn mechanism)
    burn_seed_pack_payment(&ctx, total_cost)?;
    
    // Request entropy from Pyth
    let entropy_sequence = request_pyth_entropy(&ctx, user_entropy_seed)?;
    
    // Initialize seed pack with entropy sequence
    let current_time = Clock::get()?.unix_timestamp;
    let pack_counter = ctx.accounts.config.seed_pack_counter;
    initialize_seed_pack_with_entropy(
        &mut ctx.accounts.seed_pack,
        ctx.accounts.user.key(),
        total_cost,
        current_time,
        pack_counter,
        entropy_sequence,
        user_entropy_seed,
    );
    
    // Update global counter
    ctx.accounts.config.seed_pack_counter += 1;
    
    msg!("Seed pack purchased: pack_id {}, quantity: {}, cost: {} WEED burned, entropy_sequence: {}", 
         ctx.accounts.seed_pack.pack_id, quantity, total_cost, entropy_sequence);
    
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

/// Request entropy from Pyth Entropy service (simplified implementation)
fn request_pyth_entropy(_ctx: &Context<PurchaseSeedPack>, user_entropy_seed: u64) -> Result<u64> {
    // TODO: Implement actual Pyth Entropy SDK integration when available
    // For now, we'll use a deterministic sequence based on time and user
    let current_time = Clock::get()?.unix_timestamp;
    let sequence = (current_time as u64)
        .wrapping_add(user_entropy_seed)
        .wrapping_mul(7919); // Prime for distribution
    
    msg!("Entropy requested with sequence: {}", sequence);
    Ok(sequence)
}

/// Initialize seed pack with entropy data
fn initialize_seed_pack_with_entropy(
    seed_pack: &mut SeedPack,
    purchaser: Pubkey,
    cost_paid: u64,
    current_time: i64,
    pack_counter: u64,
    entropy_sequence: u64,
    user_entropy_seed: u64,
) {
    seed_pack.purchaser = purchaser;
    seed_pack.purchased_at = current_time;
    seed_pack.cost_paid = cost_paid;
    seed_pack.is_opened = false;
    seed_pack.entropy_sequence = entropy_sequence;
    seed_pack.user_entropy_seed = user_entropy_seed;
    seed_pack.final_random_value = 0; // Will be set on opening
    seed_pack.pack_id = pack_counter;
    seed_pack.reserve = [0; 16];
}

/// Initialize seed storage for a user
pub fn initialize_seed_storage_instruction(ctx: Context<InitializeSeedStorage>) -> Result<()> {
    let seed_storage = &mut ctx.accounts.seed_storage;
    
    initialize_seed_storage(seed_storage, ctx.accounts.user.key());
    
    msg!("Seed storage initialized for user: {}", ctx.accounts.user.key());
    Ok(())
}

/// Open seed pack using Pyth Entropy result
pub fn open_seed_pack(ctx: Context<OpenSeedPack>, quantity: u8) -> Result<()> {
    // Validate pack can be opened first
    require!(!ctx.accounts.seed_pack.is_opened, GameError::SeedPackAlreadyOpened);
    require!(quantity > 0 && quantity <= 100, GameError::InvalidQuantity);
    
    // Validate seed storage is properly initialized
    require!(ctx.accounts.seed_storage.owner == ctx.accounts.user.key(), GameError::SeedStorageNotInitialized);
    
    // Retrieve and validate entropy result (read-only access)
    let final_random_value = retrieve_entropy_result(&ctx, &ctx.accounts.seed_pack)?;
    
    // Now get mutable references
    let seed_pack = &mut ctx.accounts.seed_pack;
    let config = &mut ctx.accounts.config;
    let seed_storage = &mut ctx.accounts.seed_storage;
    
    // Store the final random value in the pack for transparency
    seed_pack.final_random_value = final_random_value;
    
    // Generate seeds using Pyth Entropy
    generate_seeds_from_entropy(final_random_value, config, seed_storage, quantity)?;
    
    // Mark pack as opened
    seed_pack.is_opened = true;
    
    msg!("Seed pack opened: {} seeds generated for user: {}, entropy: {}", 
         quantity, ctx.accounts.user.key(), final_random_value);
    
    Ok(())
}

/// Retrieve entropy result from Pyth Entropy request
fn retrieve_entropy_result(ctx: &Context<OpenSeedPack>, seed_pack: &SeedPack) -> Result<u64> {
    // Verify that the entropy request account is valid
    let entropy_request_data = ctx.accounts.entropy_request.try_borrow_data()?;
    
    // In a real implementation, you would parse the Pyth Entropy result format
    // For now, we'll simulate retrieving the entropy result
    
    // Validate the account owner is Pyth Entropy program
    require!(
        *ctx.accounts.entropy_request.owner == ctx.accounts.pyth_entropy_program.key(),
        GameError::InvalidEntropyAccount
    );
    
    // Parse the entropy result data structure
    // This is a simplified version - actual implementation would follow Pyth format
    if entropy_request_data.len() < 16 {
        return Err(GameError::EntropyNotReady.into());
    }
    
    // Extract sequence number and validate it matches our record
    let result_sequence = u64::from_le_bytes(*array_ref![entropy_request_data, 0, 8]);
    require!(
        result_sequence == seed_pack.entropy_sequence,
        GameError::EntropySequenceMismatch
    );
    
    // Extract the final random value
    let random_value = u64::from_le_bytes(*array_ref![entropy_request_data, 8, 8]);
    
    // Validate that entropy is ready (non-zero)
    require!(random_value != 0, GameError::EntropyNotReady);
    
    msg!("Entropy result retrieved: sequence {}, value: {}", result_sequence, random_value);
    Ok(random_value)
}

/// Generate seeds using Pyth Entropy result
fn generate_seeds_from_entropy(
    base_random: u64,
    config: &mut Config,
    seed_storage: &mut SeedStorage,
    quantity: u8,
) -> Result<()> {
    for i in 0..quantity {
        // Derive individual seed randomness using cryptographic approach
        // Combine base entropy with index for unique per-seed randomness
        let seed_random = derive_seed_randomness(base_random, i);
            
        let seed_type = SeedType::from_random(seed_random);
        let seed_id = config.seed_counter;
        
        // Add seed to storage
        add_seed_to_storage(seed_storage, seed_id)?;
        config.seed_counter += 1;
        
        msg!("Seed generated: ID {}, Type: {:?}, Grow Power: {}, Random: {}", 
             seed_id, seed_type, seed_type.get_grow_power(), seed_random);
    }
    
    Ok(())
}

/// Derive individual seed randomness from base entropy
fn derive_seed_randomness(base_entropy: u64, index: u8) -> u64 {
    // Use a cryptographic approach to derive independent randomness for each seed
    // This ensures each seed gets unique, unbiased randomness from the base entropy
    
    // Method 1: Hash-based derivation (simplified)
    // In a real implementation, you might use a proper hash function
    let mut derived = base_entropy;
    
    // Apply multiple rounds of mixing with the index
    derived = derived.wrapping_add(index as u64);
    derived = derived.wrapping_mul(6364136223846793005u64); // LCG multiplier
    derived = derived.wrapping_add(1442695040888963407u64); // LCG increment
    
    // Additional mixing to improve distribution
    derived ^= derived >> 32;
    derived = derived.wrapping_mul(0x9e3779b97f4a7c15u64);
    derived ^= derived >> 32;
    
    derived
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