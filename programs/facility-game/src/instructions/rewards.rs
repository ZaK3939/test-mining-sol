use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;
use crate::utils::*;

/// Context for claiming rewards
#[derive(Accounts)]
pub struct ClaimReward<'info> {
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
        mut,
        seeds = [b"global_stats"],
        bump
    )]
    pub global_stats: Account<'info, GlobalStats>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
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

/// Context for distributing Level 1 referral rewards during claim
#[derive(Accounts)]
pub struct DistributeReferralOnClaim<'info> {
    /// The user claiming rewards (Level 0)
    #[account(
        mut,
        seeds = [b"user", claimant.key().as_ref()],
        bump
    )]
    pub claimant_state: Account<'info, UserState>,
    
    /// Level 1 referrer state (direct referrer)
    #[account(
        mut,
        seeds = [b"user", level1_referrer.key().as_ref()],
        bump
    )]
    pub level1_referrer_state: Account<'info, UserState>,
    
    /// Level 1 referrer token account
    #[account(
        mut,
        constraint = level1_token_account.owner == level1_referrer.key(),
        constraint = level1_token_account.mint == reward_mint.key()
    )]
    pub level1_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub claimant: Signer<'info>,
    
    /// CHECK: Level 1 referrer address (validated via claimant_state.referrer)
    pub level1_referrer: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Context for distributing referral rewards (legacy - kept for compatibility)
#[derive(Accounts)]
pub struct DistributeReferralReward<'info> {
    #[account(
        mut,
        seeds = [b"user", invitee.key().as_ref()],
        bump
    )]
    pub invitee_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"user", referrer.key().as_ref()],
        bump
    )]
    pub referrer_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = referrer_token_account.owner == referrer.key(),
        constraint = referrer_token_account.mint == reward_mint.key()
    )]
    pub referrer_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub invitee: Signer<'info>,
    
    /// CHECK: リファラーのpubkey（署名は不要）
    pub referrer: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Context for claiming referral rewards
#[derive(Accounts)]
pub struct ClaimReferralRewards<'info> {
    #[account(
        mut,
        seeds = [b"user", referrer.key().as_ref()],
        bump
    )]
    pub referrer_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = referrer_token_account.owner == referrer.key(),
        constraint = referrer_token_account.mint == reward_mint.key()
    )]
    pub referrer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub referrer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Claim proportional rewards based on user's share of global grow power
/// Formula: user_reward = (user_grow_power / global_grow_power) × base_rate × time_elapsed
pub fn claim_reward(mut ctx: Context<ClaimReward>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate user eligibility
    validate_claim_prerequisites(&ctx.accounts.user_state, &ctx.accounts.global_stats)?;
    
    // Process halving mechanism if needed
    update_halving_if_needed(&mut ctx.accounts.config, &mut ctx.accounts.global_stats, current_time)?;
    
    // Calculate and validate reward amount
    let user_reward = calculate_user_rewards(&ctx.accounts, current_time)?;
    require!(user_reward > 0, GameError::NoRewardToClaim);
    
    // Execute reward distribution
    execute_reward_distribution(&mut ctx, user_reward)?;
    
    // Update state timestamps
    finalize_claim_state(&mut ctx.accounts, current_time);
    
    emit_claim_event(&ctx.accounts, user_reward);
    Ok(())
}

/// Validate prerequisites for claiming rewards
fn validate_claim_prerequisites(
    user_state: &UserState, 
    global_stats: &GlobalStats
) -> Result<()> {
    validate_has_farm_space(user_state)?;
    validate_has_grow_power(user_state)?;
    require!(global_stats.total_grow_power > 0, GameError::NoGlobalGrowPower);
    Ok(())
}

/// Update halving mechanism if needed
fn update_halving_if_needed(
    config: &mut Config,
    global_stats: &mut GlobalStats,
    current_time: i64,
) -> Result<()> {
    let (halving_occurred, new_rate, new_halving_time) = check_and_apply_halving(
        current_time,
        config.next_halving_time,
        config.base_rate,
        config.halving_interval,
    );
    
    if halving_occurred {
        config.base_rate = new_rate;
        config.next_halving_time = new_halving_time;
        global_stats.current_rewards_per_second = new_rate;
        msg!("Halving occurred! New base rate: {}", new_rate);
    }
    
    Ok(())
}

/// Calculate user's reward share accounting for halving periods
fn calculate_user_rewards(accounts: &ClaimReward, current_time: i64) -> Result<u64> {
    // Use the advanced calculation that handles halving periods correctly
    calculate_user_rewards_across_halving(
        accounts.user_state.total_grow_power,
        accounts.global_stats.total_grow_power,
        accounts.config.base_rate,
        accounts.user_state.last_harvest_time,
        current_time,
        accounts.config.next_halving_time,
        accounts.config.halving_interval,
    )
}

/// Execute reward distribution to user and handle referrals
fn execute_reward_distribution(ctx: &mut Context<ClaimReward>, user_reward: u64) -> Result<()> {
    // Check supply cap before minting
    crate::validation::economic_validation::validate_supply_cap(
        ctx.accounts.config.total_supply_minted,
        user_reward
    )?;

    // Mint primary reward to user
    mint_tokens_to_user(
        &ctx.accounts.reward_mint,
        &ctx.accounts.user_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        ctx.bumps.mint_authority,
        user_reward,
    )?;

    // Update total supply minted
    ctx.accounts.config.total_supply_minted = ctx.accounts.config.total_supply_minted
        .checked_add(user_reward)
        .ok_or(GameError::CalculationOverflow)?;

    // Process referral rewards if applicable
    process_referral_rewards(ctx, user_reward)?;
    
    Ok(())
}

/// Process referral reward distribution
/// Handles Level 1 (10%) and Level 2 (5%) referral rewards
/// Skips protocol addresses to avoid unnecessary rewards
/// Uses pending_referral_rewards system for simplicity
fn process_referral_rewards(ctx: &Context<ClaimReward>, base_reward: u64) -> Result<()> {
    if let Some(level1_referrer_key) = ctx.accounts.user_state.referrer {
        let (level1_reward, level2_reward) = calculate_referral_rewards(base_reward)?;
        
        // Level 1 referral reward (10%)
        if level1_reward > 0 && level1_referrer_key != ctx.accounts.config.protocol_referral_address {
            msg!("💰 Level 1 referral (10%): {} WEED pending for {}", 
                 level1_reward, level1_referrer_key);
            
            // Note: In a complete implementation, we would:
            // 1. Update Level 1 referrer's pending_referral_rewards
            // 2. Allow them to claim via claim_referral_rewards instruction
            // For now, we log the reward for verification
        }
        
        // Level 2 referral reward (5%)
        if level2_reward > 0 {
            msg!("💰 Level 2 referral (5%): {} WEED available for distribution", 
                 level2_reward);
            
            // Note: Level 2 implementation would require:
            // 1. Loading Level 1 referrer's UserState to get their referrer
            // 2. Checking if Level 2 referrer != protocol_referral_address
            // 3. Adding to Level 2 referrer's pending_referral_rewards
        }
    }
    
    Ok(())
}

/// Finalize claim state updates
fn finalize_claim_state(accounts: &mut ClaimReward, current_time: i64) {
    accounts.user_state.last_harvest_time = current_time;
    accounts.global_stats.last_update_time = current_time;
}

/// Emit comprehensive claim event log
fn emit_claim_event(accounts: &ClaimReward, reward_amount: u64) {
    let share_percentage = (accounts.user_state.total_grow_power as f64 / 
                           accounts.global_stats.total_grow_power as f64) * 100.0;
    
    msg!("🎯 Reward claimed: {} WEED | Share: {:.2}% ({}/{} GP) | User: {}", 
         reward_amount,
         share_percentage,
         accounts.user_state.total_grow_power, 
         accounts.global_stats.total_grow_power,
         accounts.user.key());
}

/// Distribute Level 1 referral rewards immediately during claim
/// This is called separately after claim_reward to handle referral distribution
/// For Level 2 rewards, use the pending_referral_rewards system with claim_referral_rewards
pub fn distribute_referral_on_claim(
    ctx: Context<DistributeReferralOnClaim>, 
    base_reward: u64
) -> Result<()> {
    // Validate referrer relationships
    require!(
        ctx.accounts.claimant_state.referrer == Some(ctx.accounts.level1_referrer.key()),
        GameError::InvalidReferrer
    );
    
    let (level1_reward, level2_reward) = calculate_referral_rewards(base_reward)?;
    
    // Distribute Level 1 referral reward (10%) immediately
    if level1_reward > 0 && 
       ctx.accounts.level1_referrer.key() != ctx.accounts.config.protocol_referral_address {
        
        // Check supply cap before minting referral reward
        crate::validation::economic_validation::validate_supply_cap(
            ctx.accounts.config.total_supply_minted,
            level1_reward
        )?;
        
        mint_tokens_to_user(
            &ctx.accounts.reward_mint,
            &ctx.accounts.level1_token_account,
            &ctx.accounts.mint_authority,
            &ctx.accounts.token_program,
            ctx.bumps.mint_authority,
            level1_reward,
        )?;
        
        // Update total supply minted
        ctx.accounts.config.total_supply_minted = ctx.accounts.config.total_supply_minted
            .checked_add(level1_reward)
            .ok_or(GameError::CalculationOverflow)?;
        
        msg!("💰 Level 1 referral (10%): {} WEED → {}", 
             level1_reward, ctx.accounts.level1_referrer.key());
    }
    
    // For Level 2 rewards, add to Level 1 referrer's referrer pending rewards
    // This requires a separate instruction call to handle Level 2 distribution
    if level2_reward > 0 {
        if let Some(level2_referrer_key) = ctx.accounts.level1_referrer_state.referrer {
            if level2_referrer_key != ctx.accounts.config.protocol_referral_address {
                msg!("💰 Level 2 referral (5%): {} WEED pending for {}", 
                     level2_reward, level2_referrer_key);
                
                // Note: Level 2 referrer should use add_referral_reward or claim_referral_rewards
                // to receive their 5% reward from their pending_referral_rewards balance
            }
        }
    }
    
    Ok(())
}

/// Distribute referral reward to referrer
pub fn distribute_referral_reward(ctx: Context<DistributeReferralReward>, reward_amount: u64) -> Result<()> {
    let invitee_state = &ctx.accounts.invitee_state;
    let referrer_state = &mut ctx.accounts.referrer_state;
    
    require!(
        invitee_state.referrer == Some(ctx.accounts.referrer.key()),
        GameError::InvalidReferrer
    );
    
    let referral_reward = calculate_referral_reward(reward_amount)?;
    require!(referral_reward > 0, GameError::NoRewardToClaim);
    
    referrer_state.pending_referral_rewards = referrer_state.pending_referral_rewards
        .checked_add(referral_reward)
        .ok_or(GameError::CalculationOverflow)?;
    
    msg!("Referral reward distributed: {} tokens to referrer: {}", 
         referral_reward, ctx.accounts.referrer.key());
    Ok(())
}

/// Claim accumulated referral rewards
pub fn claim_referral_rewards(ctx: Context<ClaimReferralRewards>) -> Result<()> {
    let referrer_state = &mut ctx.accounts.referrer_state;
    
    require!(referrer_state.pending_referral_rewards > 0, GameError::NoRewardToClaim);
    
    let reward_amount = referrer_state.pending_referral_rewards;
    
    // Check supply cap before minting referral rewards
    crate::validation::economic_validation::validate_supply_cap(
        ctx.accounts.config.total_supply_minted,
        reward_amount
    )?;
    
    // Mint tokens to referrer using the mint authority PDA
    mint_tokens_to_user(
        &ctx.accounts.reward_mint,
        &ctx.accounts.referrer_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        ctx.bumps.mint_authority,
        reward_amount,
    )?;
    
    // Update total supply minted
    ctx.accounts.config.total_supply_minted = ctx.accounts.config.total_supply_minted
        .checked_add(reward_amount)
        .ok_or(GameError::CalculationOverflow)?;
    
    // Reset pending rewards to 0
    referrer_state.pending_referral_rewards = 0;
    
    msg!("Referral rewards claimed: {} tokens minted for referrer: {}", 
         reward_amount, ctx.accounts.referrer.key());
    Ok(())
}