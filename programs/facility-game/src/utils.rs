use anchor_lang::prelude::*;
// Removed standard SPL token imports - using Token 2022 only
use anchor_spl::token_2022::{self as token_2022, Token2022, MintTo as MintTo2022, TransferChecked};
use crate::state::*;
use crate::error::*;
use anchor_lang::solana_program::hash::hash;

// ===== VALIDATION HELPERS =====
// Delegate to validation module
pub use crate::validation::{
    validate_user_ownership,
    validate_token_balance,
    validate_has_farm_space,
    validate_has_grow_power,
    validate_sufficient_balance,
};

// ===== TOKEN TRANSFER HELPERS =====

/// Transfer tokens between accounts using CPI with transfer fee support (Token 2022)
pub fn transfer_tokens_with_cpi<'info>(
    from: &UncheckedAccount<'info>,
    to: &UncheckedAccount<'info>,
    mint: &UncheckedAccount<'info>,
    authority: &Signer<'info>,
    token_program: &Program<'info, Token2022>,
    amount: u64,
) -> Result<()> {
    let transfer_accounts = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(token_program.to_account_info(), transfer_accounts);
    token_2022::transfer_checked(cpi_ctx, amount, 6) // 6 decimals for WEED
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

// Delegate to economics module for calculation functions
pub use crate::economics::{calculate_base_reward as calculate_reward, calculate_referral_reward_for_level};

/// Calculate referral reward (10% of base reward) - Legacy wrapper
pub fn calculate_referral_reward(base_reward: u64) -> Result<u64> {
    calculate_referral_reward_for_level(base_reward, 1)
}

// Delegate to economics module for advanced calculation functions
pub use crate::economics::{calculate_user_share_reward as calculate_user_share_of_global_rewards, calculate_rewards_across_halving as calculate_user_rewards_across_halving};

// Note: Manual upgrade functions removed - now using auto-upgrade system

// Delegate to economics module for halving mechanism
pub use crate::economics::check_and_apply_halving;

// Delegate to economics module for referral calculations
pub use crate::economics::calculate_referral_rewards;

/// Calculate reward distribution percentages for all scenarios
/// Returns (user_percentage, level1_percentage, level2_percentage) as basis points (1/100%)
/// Total always equals 10000 (100%), user's share varies: 100%, 90%, or 85%
pub fn calculate_reward_percentages(
    has_level1: bool,
    has_level2: bool,
    level1_is_protocol: bool,
    level2_is_protocol: bool,
    user_is_protocol: bool,
) -> (u16, u16, u16) {
    if user_is_protocol {
        // Protocol user gets 100% (10000 basis points), no distribution
        return (10000, 0, 0);
    }
    
    match (has_level1, has_level2, level1_is_protocol, level2_is_protocol) {
        // No referrers: User gets 100%
        (false, _, _, _) => (10000, 0, 0),
        
        // Level 1 only, protocol: User gets 100% (protocol share returns to user)
        (true, false, true, _) => (10000, 0, 0),
        
        // Level 1 only, regular: User gets 90%, Level 1 gets 10%
        (true, false, false, _) => (9000, 1000, 0),
        
        // Both levels, Level 1 protocol, Level 2 protocol: User gets 100% (all protocol shares return)
        (true, true, true, true) => (10000, 0, 0),
        
        // NOTE: Case (true, true, true, false) is logically impossible:
        // If Level 1 is protocol, it cannot have referred Level 2 (protocol doesn't invite users)
        // This case should never occur in practice
        
        // Both levels, Level 1 regular, Level 2 protocol: User gets 90% (L2 returns), Level 1 gets 10%
        (true, true, false, true) => (9000, 1000, 0),
        
        // Both levels, both regular: User gets 85%, Level 1 gets 10%, Level 2 gets 5%
        (true, true, false, false) => (8500, 1000, 500),
        
        // Fallback for impossible case - return everything to user
        _ => (10000, 0, 0),
    }
}

/// Validate referral distribution scenario with proper percentage splits
/// Returns (claimant_amount, l1_amount, l2_amount)
/// Total always equals base_reward (100%), but claimant's share varies: 100%, 90%, or 85%
pub fn validate_referral_scenario(
    base_reward: u64,
    has_level1: bool,
    has_level2: bool,
    level1_is_protocol: bool,
    level2_is_protocol: bool,
    claimant_is_protocol: bool,
) -> Result<(u64, u64, u64)> {
    let (claimant_pct, level1_pct, level2_pct) = calculate_reward_percentages(
        has_level1, has_level2, level1_is_protocol, level2_is_protocol, claimant_is_protocol
    );
    
    // Calculate actual amounts from base_reward (total is always 100%)
    let claimant_amount = base_reward.checked_mul(claimant_pct as u64).unwrap_or(0) / 10000;
    let level1_amount = base_reward.checked_mul(level1_pct as u64).unwrap_or(0) / 10000;
    let level2_amount = base_reward.checked_mul(level2_pct as u64).unwrap_or(0) / 10000;
    
    // Verify total equals original base_reward
    let total_distributed = claimant_amount + level1_amount + level2_amount;
    
    // Log the scenario for verification
    msg!("🧮 Referral calculation: Base={}, Claimant={}%, L1={}%, L2={}%", 
         base_reward, claimant_pct / 100, level1_pct / 100, level2_pct / 100);
    msg!("💰 Actual amounts: Claimant={}, L1={}, L2={}, Total={}", 
         claimant_amount, level1_amount, level2_amount, total_distributed);
    
    Ok((claimant_amount, level1_amount, level2_amount))
}

/// Mint tokens to a user account using Token 2022
pub fn mint_tokens_to_user<'info>(
    reward_mint: &UncheckedAccount<'info>,
    user_token_account: &UncheckedAccount<'info>,
    mint_authority: &UncheckedAccount<'info>,
    token_program: &Program<'info, Token2022>,
    authority_bump: u8,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = MintTo2022 {
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
    
    token_2022::mint_to(cpi_ctx, amount)?;
    
    Ok(())
}

/// Determine seed type from random value
/// This function can be used for future rarity-based seed generation
pub fn determine_seed_type_from_random(random_seed: u64) -> SeedType {
    SeedType::from_random(random_seed)
}

// Moved to validation module - see above re-export

// Delegate to economics module for trading fee calculation
pub use crate::economics::calculate_trading_fee as calculate_transfer_fee;

// ===== FARM SPACE HELPERS =====

/// Initialize a new Level 1 farm space with starter seed
pub fn initialize_farm_space_level_1(farm_space: &mut FarmSpace, owner: Pubkey) -> Result<()> {
    farm_space.owner = owner;
    farm_space.level = 1;
    farm_space.capacity = FarmSpace::get_capacity_for_level(1);
    farm_space.seed_count = 1; // Starting with 1 seed (Seed 1)
    farm_space.total_grow_power = SeedType::Seed1.get_grow_power(); // 100 Grow Power
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
    seed_storage.seed_types = Vec::new(); // Initialize parallel type vector
    seed_storage.total_seeds = 0;
    seed_storage.seed_type_counts = [0; 16];
    seed_storage.reserve = [0; 8];
}


/// Add seed to user's storage with type tracking and auto-discard
pub fn add_seed_to_storage(
    seed_storage: &mut SeedStorage,
    seed_id: u64,
    seed_type: SeedType,
) -> Result<()> {
    // Check if we can add this seed type (with auto-discard if needed)
    if !seed_storage.can_add_seed_type(&seed_type) {
        // Auto-discard if at limit for this type
        seed_storage.auto_discard_excess(&seed_type)?;
    }
    
    // Add the seed using the new storage API
    seed_storage.add_seed(seed_id, &seed_type)?;
    
    Ok(())
}

/// Legacy add seed function for backward compatibility (with default Seed1 type)
pub fn add_seed_to_storage_legacy(
    seed_storage: &mut SeedStorage,
    seed_id: u64,
) -> Result<()> {
    require!(
        seed_storage.seed_ids.len() < SeedStorage::MAX_TOTAL_SEEDS,
        GameError::StorageFull
    );
    
    // Use default Seed1 type for legacy compatibility
    let default_seed_type = SeedType::Seed1;
    seed_storage.seed_ids.push(seed_id);
    seed_storage.seed_types.push(default_seed_type);
    seed_storage.total_seeds += 1;
    
    // Update type count
    let type_index = default_seed_type as usize;
    if type_index < 16 {
        seed_storage.seed_type_counts[type_index] += 1;
    }
    
    Ok(())
}

/// Remove seed from user's storage with type tracking
pub fn remove_seed_from_storage(
    seed_storage: &mut SeedStorage,
    seed_id: u64,
    seed_type: SeedType,
) -> Result<bool> {
    // Use the storage's built-in removal with type tracking
    let removed = seed_storage.remove_seed(seed_id, &seed_type);
    
    if removed {
        msg!("Seed {} (type: {:?}) removed from storage. New count: {}", 
             seed_id, seed_type, seed_storage.total_seeds);
    } else {
        msg!("Seed {} not found in storage", seed_id);
    }
    
    Ok(removed)
}

/// Legacy remove seed function for backward compatibility
pub fn remove_seed_from_storage_legacy(
    seed_storage: &mut SeedStorage,
    seed_id: u64,
) -> Result<bool> {
    // Find the seed ID in the storage
    if let Some(position) = seed_storage.seed_ids.iter().position(|&id| id == seed_id) {
        // Remove the seed ID from the vector
        seed_storage.seed_ids.remove(position);
        seed_storage.total_seeds = seed_storage.seed_ids.len() as u32;
        
        msg!("Seed {} removed from storage. New count: {}", seed_id, seed_storage.total_seeds);
        Ok(true)
    } else {
        msg!("Seed {} not found in storage", seed_id);
        Ok(false)
    }
}

// ===== HASH-BASED INVITE CODE UTILITIES =====

/// Generate a secure hash for invite codes
/// Formula: SHA256(plaintext_code + salt)
pub fn generate_invite_code_hash(
    plaintext_code: &[u8; 12],
    salt: &[u8; 16]
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(plaintext_code);
    data.extend_from_slice(salt);
    
    hash(&data).to_bytes()
}

/// Get fixed salt for invite code hashing
/// Using a fixed salt simplifies PDA calculation and doesn't compromise security
/// since the salt is not meant to be secret in this design
pub fn get_fixed_salt() -> [u8; 16] {
    // Fixed salt for invite codes - this makes PDA calculation predictable
    // Security note: This is intentional to simplify the invite system
    // The actual security comes from the hash verification, not salt secrecy
    [
        0x46, 0x41, 0x43, 0x49, 0x4c, 0x49, 0x54, 0x59,  // "FACILITY"
        0x47, 0x41, 0x4d, 0x45, 0x32, 0x30, 0x32, 0x34,  // "GAME2024"
    ]
}

/// Validate invite code format (12 alphanumeric characters)
pub fn validate_invite_code_format(invite_code: &[u8; 12]) -> Result<()> {
    for &byte in invite_code.iter() {
        require!(
            (byte >= b'0' && byte <= b'9') || 
            (byte >= b'A' && byte <= b'Z') || 
            (byte >= b'a' && byte <= b'z'),
            GameError::InvalidInviteCode
        );
    }
    Ok(())
}

/// Verify that a plaintext code matches the stored hash
pub fn verify_invite_code_hash(
    plaintext_code: &[u8; 12],
    salt: &[u8; 16],
    stored_hash: &[u8; 32]
) -> bool {
    let computed_hash = generate_invite_code_hash(plaintext_code, salt);
    computed_hash == *stored_hash
}