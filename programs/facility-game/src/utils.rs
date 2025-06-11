use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer, Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;

// ===== VALIDATION HELPERS =====

/// Validate user ownership of account
pub fn validate_user_ownership(user_state: &UserState, expected_owner: Pubkey) -> Result<()> {
    require!(user_state.owner == expected_owner, GameError::Unauthorized);
    Ok(())
}

/// Validate token account balance is sufficient
pub fn validate_token_balance(account: &TokenAccount, required: u64) -> Result<()> {
    require!(account.amount >= required, GameError::InsufficientFunds);
    Ok(())
}

/// Validate farm space has capacity for more seeds
pub fn validate_farm_space_capacity(farm_space: &FarmSpace) -> Result<()> {
    require!(
        farm_space.seed_count < farm_space.capacity,
        GameError::FarmSpaceCapacityExceeded
    );
    Ok(())
}

/// Validate user has farm space
pub fn validate_has_farm_space(user_state: &UserState) -> Result<()> {
    require!(user_state.has_farm_space, GameError::NoFarmSpace);
    Ok(())
}

/// Validate user has grow power
pub fn validate_has_grow_power(user_state: &UserState) -> Result<()> {
    require!(user_state.total_grow_power > 0, GameError::NoGrowPower);
    Ok(())
}

// ===== TOKEN TRANSFER HELPERS =====

/// Transfer tokens between accounts using CPI
pub fn transfer_tokens_with_cpi<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let transfer_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(token_program.to_account_info(), transfer_accounts);
    token::transfer(cpi_ctx, amount)
}

/// Transfer SOL using system program
pub fn transfer_sol_payment<'info>(
    from: &Signer<'info>,
    to: &UncheckedAccount<'info>,
    system_program: &Program<'info, System>,
    amount: u64,
) -> Result<()> {
    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &from.key(),
        &to.key(),
        amount,
    );
    
    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            from.to_account_info(),
            to.to_account_info(),
            system_program.to_account_info(),
        ],
    )?;
    
    Ok(())
}

// ===== CALCULATION HELPERS =====

/// Calculate reward amount based on time, grow power, and base rate
pub fn calculate_reward(time_elapsed: u64, grow_power: u64, base_rate: u64) -> Result<u64> {
    let reward_amount = time_elapsed
        .checked_mul(grow_power)
        .and_then(|result| result.checked_mul(base_rate))
        .and_then(|result| result.checked_div(1000))
        .ok_or(GameError::CalculationOverflow)?;
    
    Ok(reward_amount)
}

/// Calculate referral reward (10% of base reward)
pub fn calculate_referral_reward(base_reward: u64) -> Result<u64> {
    let referral_reward = base_reward
        .checked_mul(10)
        .and_then(|result| result.checked_div(100))
        .unwrap_or(0);
    
    Ok(referral_reward)
}

/// Calculate user's share of global rewards across halving periods
/// This function properly handles rewards calculation when halving occurs during the elapsed time
pub fn calculate_user_share_of_global_rewards(
    user_grow_power: u64,
    global_grow_power: u64,
    current_rewards_per_second: u64,
    time_elapsed_seconds: u64,
) -> Result<u64> {
    // Early return for edge cases
    if global_grow_power == 0 || user_grow_power == 0 || current_rewards_per_second == 0 {
        return Ok(0);
    }
    
    // Use 128-bit arithmetic to prevent overflow during intermediate calculations
    let user_power = user_grow_power as u128;
    let total_rewards = (current_rewards_per_second as u128)
        .checked_mul(time_elapsed_seconds as u128)
        .ok_or(GameError::CalculationOverflow)?;
    
    // Calculate proportional share with precision
    let user_reward = user_power
        .checked_mul(total_rewards)
        .and_then(|result| result.checked_div(global_grow_power as u128))
        .and_then(|result| u64::try_from(result).ok())
        .ok_or(GameError::CalculationOverflow)?;
    
    Ok(user_reward)
}

/// Calculate user rewards across halving periods
/// This function properly splits rewards when halving occurs during the elapsed time
pub fn calculate_user_rewards_across_halving(
    user_grow_power: u64,
    global_grow_power: u64,
    base_rate: u64,
    last_harvest_time: i64,
    current_time: i64,
    next_halving_time: i64,
    halving_interval: i64,
) -> Result<u64> {
    if global_grow_power == 0 || user_grow_power == 0 || base_rate == 0 {
        return Ok(0);
    }
    
    let mut total_reward = 0u64;
    let mut period_start = last_harvest_time;
    let mut current_rate = base_rate;
    let mut next_halving = next_halving_time;
    
    // Calculate rewards for each period between halvings
    while period_start < current_time {
        let period_end = if current_time <= next_halving {
            // No halving occurs in this period
            current_time
        } else {
            // Halving occurs during this period
            next_halving
        };
        
        let period_duration = (period_end - period_start) as u64;
        
        if period_duration > 0 {
            let period_reward = calculate_user_share_of_global_rewards(
                user_grow_power,
                global_grow_power,
                current_rate,
                period_duration,
            )?;
            
            total_reward = total_reward
                .checked_add(period_reward)
                .ok_or(GameError::CalculationOverflow)?;
        }
        
        // Move to next period
        period_start = period_end;
        
        // Apply halving if we reached a halving point
        if period_end == next_halving && current_time > next_halving {
            current_rate = current_rate / 2;
            next_halving += halving_interval;
        }
    }
    
    Ok(total_reward)
}

/// Calculate upgrade cost for farm space level
pub fn get_upgrade_cost_for_level(current_level: u8) -> Result<u64> {
    match current_level {
        1 => Ok(3_500 * 1_000_000),   // 3500 $WEED with 6 decimals
        2 => Ok(18_000 * 1_000_000),  // 18000 $WEED with 6 decimals
        3 => Ok(20_000 * 1_000_000),  // 20000 $WEED with 6 decimals
        4 => Ok(25_000 * 1_000_000),  // 25000 $WEED with 6 decimals
        _ => Err(GameError::MaxLevelReached.into()),
    }
}

/// Check if halving should occur and return new rate
pub fn check_and_apply_halving(
    current_time: i64,
    next_halving_time: i64,
    current_rate: u64,
    halving_interval: i64,
) -> (bool, u64, i64) {
    if current_time >= next_halving_time {
        let new_rate = current_rate / 2;
        let new_halving_time = next_halving_time + halving_interval;
        (true, new_rate, new_halving_time)
    } else {
        (false, current_rate, next_halving_time)
    }
}

/// Calculate referral rewards: Level 1 (10%), Level 2 (5%)
pub fn calculate_referral_rewards(base_reward: u64) -> Result<(u64, u64)> {
    let level1_reward = base_reward
        .checked_mul(10)
        .and_then(|result| result.checked_div(100))
        .unwrap_or(0);
    
    let level2_reward = base_reward
        .checked_mul(5)
        .and_then(|result| result.checked_div(100))
        .unwrap_or(0);
    
    Ok((level1_reward, level2_reward))
}

/// Mint tokens to a user account
pub fn mint_tokens_to_user<'info>(
    reward_mint: &Account<'info, Mint>,
    user_token_account: &Account<'info, TokenAccount>,
    mint_authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token>,
    authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = MintTo {
        mint: reward_mint.to_account_info(),
        to: user_token_account.to_account_info(),
        authority: mint_authority.to_account_info(),
    };
    
    let seeds = &[
        b"mint_authority".as_ref(),
        &[authority_bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::mint_to(cpi_ctx, amount)?;
    
    Ok(())
}

/// Determine seed type from random value
/// This function can be used for future rarity-based seed generation
pub fn determine_seed_type_from_random(random_seed: u64) -> SeedType {
    SeedType::from_random(random_seed)
}

/// Validate that user has sufficient balance for operation
pub fn validate_sufficient_balance(balance: u64, required: u64) -> Result<()> {
    require!(balance >= required, GameError::InsufficientFunds);
    Ok(())
}

/// Calculate fee amount (2% of total)
pub fn calculate_transfer_fee(amount: u64) -> Result<(u64, u64)> {
    let fee_amount = amount
        .checked_mul(2)
        .and_then(|result| result.checked_div(100))
        .ok_or(GameError::CalculationOverflow)?;
        
    let transfer_amount = amount
        .checked_sub(fee_amount)
        .ok_or(GameError::CalculationOverflow)?;
    
    Ok((fee_amount, transfer_amount))
}

// ===== FARM SPACE HELPERS =====

/// Initialize a new Level 1 farm space with starter seed
pub fn initialize_farm_space_level_1(farm_space: &mut FarmSpace, owner: Pubkey) -> Result<()> {
    farm_space.owner = owner;
    farm_space.level = 1;
    farm_space.capacity = FarmSpace::get_capacity_for_level(1);
    farm_space.seed_count = 1; // Starting with 1 seed (Seed 1)
    farm_space.total_grow_power = SeedType::Seed1.get_grow_power(); // 100 Grow Power
    farm_space.upgrade_start_time = 0;
    farm_space.upgrade_target_level = 0;
    farm_space.reserve = [0; 32];
    Ok(())
}

/// Update global stats when farm space is created
pub fn update_global_stats_on_farm_creation(
    global_stats: &mut GlobalStats,
    farm_grow_power: u64,
    current_time: i64,
) {
    global_stats.total_grow_power += farm_grow_power;
    global_stats.total_farm_spaces += 1;
    global_stats.last_update_time = current_time;
}

/// Update global stats when grow power changes
pub fn update_global_grow_power(
    global_stats: &mut GlobalStats,
    power_change: i64, // Can be positive or negative
    current_time: i64,
) -> Result<()> {
    if power_change >= 0 {
        global_stats.total_grow_power += power_change as u64;
    } else {
        global_stats.total_grow_power = global_stats.total_grow_power
            .saturating_sub((-power_change) as u64);
    }
    global_stats.last_update_time = current_time;
    Ok(())
}

// ===== SEED MANAGEMENT HELPERS =====

/// Initialize seed storage for a user
pub fn initialize_seed_storage(seed_storage: &mut SeedStorage, owner: Pubkey) {
    seed_storage.owner = owner;
    seed_storage.seed_ids = Vec::new();
    seed_storage.total_seeds = 0;
    seed_storage.reserve = [0; 32];
}

/// Generate random seed for seed pack opening
pub fn generate_seed_pack_random(
    timestamp: i64,
    user_key: &[u8],
    counter: u64,
) -> u64 {
    (timestamp as u64)
        .wrapping_mul(user_key[0] as u64)
        .wrapping_add(counter)
        .wrapping_mul(7919) // Prime number for better distribution
}

/// Add seed to user's storage
pub fn add_seed_to_storage(
    seed_storage: &mut SeedStorage,
    seed_id: u64,
) -> Result<()> {
    require!(
        seed_storage.seed_ids.len() < 100, // Max 100 seeds per storage
        GameError::InvalidQuantity
    );
    
    seed_storage.seed_ids.push(seed_id);
    seed_storage.total_seeds += 1;
    Ok(())
}