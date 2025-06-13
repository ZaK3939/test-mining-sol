use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint};
use anchor_spl::token_2022::Token2022;
use crate::state::*;
use crate::error::*;
use crate::utils::*;

/// Context for accumulating referral rewards to a referrer's pending balance
#[derive(Accounts)]
pub struct AccumulateReferralReward<'info> {
    /// The referrer who will receive the reward
    #[account(
        mut,
        seeds = [b"user", referrer.key().as_ref()],
        bump
    )]
    pub referrer_state: Account<'info, UserState>,
    
    /// The referrer's public key  
    /// CHECK: Used only for derivation verification
    pub referrer: UncheckedAccount<'info>,
    
    /// System configuration for protocol address checks
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
}

/// Context for viewing pending referral rewards
#[derive(Accounts)]
pub struct ViewPendingReferralRewards<'info> {
    /// The user whose pending rewards we want to check
    #[account(
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    /// The user's public key
    pub user: Signer<'info>,
}

/// Accumulate referral reward to a referrer's pending balance
/// This is called when someone claims rewards and their referrer should get a bonus
pub fn accumulate_referral_reward(
    ctx: Context<AccumulateReferralReward>,
    reward_amount: u64,
    referral_level: u8, // 1 or 2 for Level 1 (10%) or Level 2 (5%)
) -> Result<()> {
    // Validate referral level
    require!(referral_level == 1 || referral_level == 2, GameError::InvalidReferralLevel);
    
    // Check if referrer is protocol address (they don't receive referral rewards)
    if ctx.accounts.referrer.key() == ctx.accounts.config.protocol_referral_address {
        msg!("🏢 Protocol address referrer - no reward accumulated");
        return Ok(());
    }
    
    // Calculate referral reward based on level
    let referral_percentage = if referral_level == 1 { 10 } else { 5 };
    let referral_reward = reward_amount
        .checked_mul(referral_percentage)
        .and_then(|result| result.checked_div(100))
        .ok_or(GameError::CalculationOverflow)?;
    
    // Add to pending referral rewards with safety checks
    let current_pending = ctx.accounts.referrer_state.pending_referral_rewards;
    let max_allowed_pending = 1_000_000_000u64; // 1B tokens maximum pending
    
    require!(
        current_pending < max_allowed_pending,
        GameError::PendingRewardsLimitExceeded
    );
    
    ctx.accounts.referrer_state.pending_referral_rewards = current_pending
        .checked_add(referral_reward)
        .ok_or(GameError::CalculationOverflow)?;
    
    msg!("💰 Level {} referral accumulated: {} WEED for {} (Total pending: {})", 
         referral_level, 
         referral_reward, 
         ctx.accounts.referrer.key(),
         ctx.accounts.referrer_state.pending_referral_rewards);
    
    Ok(())
}

/// View current pending referral rewards for a user
/// This allows users to check how much referral commission they have accumulated
pub fn view_pending_referral_rewards(ctx: Context<ViewPendingReferralRewards>) -> Result<()> {
    let pending_amount = ctx.accounts.user_state.pending_referral_rewards;
    
    msg!("👀 Pending referral rewards for {}: {} WEED", 
         ctx.accounts.user.key(), 
         pending_amount);
    
    // Emit an event for frontend integration
    emit!(PendingReferralRewardsEvent {
        user: ctx.accounts.user.key(),
        pending_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

/// Event emitted when checking pending referral rewards
#[event]
pub struct PendingReferralRewardsEvent {
    pub user: Pubkey,
    pub pending_amount: u64,
    pub timestamp: i64,
}

/// Context for the enhanced claim reward that includes pending referral rewards
#[derive(Accounts)]
pub struct ClaimRewardWithReferralRewards<'info> {
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
    
    pub token_program: Program<'info, Token2022>,
    
    // Optional referrer accounts for accumulating referral rewards
    pub level1_referrer_state: Option<Account<'info, UserState>>,
    
    /// CHECK: Level 1 referrer public key
    pub level1_referrer: Option<UncheckedAccount<'info>>,
    
    pub level2_referrer_state: Option<Account<'info, UserState>>,
    
    /// CHECK: Level 2 referrer public key
    pub level2_referrer: Option<UncheckedAccount<'info>>,
}

/// Enhanced claim reward function that handles both farming rewards and pending referral rewards
/// This is the main claim function that users should call to get all their accumulated rewards
pub fn claim_reward_with_referral_rewards(
    mut ctx: Context<ClaimRewardWithReferralRewards>
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate prerequisites
    crate::validation::user_validation::validate_has_farm_space(&ctx.accounts.user_state)?;
    crate::validation::user_validation::validate_has_grow_power(&ctx.accounts.user_state)?;
    require!(ctx.accounts.global_stats.total_grow_power > 0, GameError::NoGlobalGrowPower);
    
    // Calculate farming reward based on grow power and time
    let farming_reward = calculate_user_rewards_across_halving(
        ctx.accounts.user_state.total_grow_power,
        ctx.accounts.global_stats.total_grow_power,
        ctx.accounts.config.base_rate,
        ctx.accounts.user_state.last_harvest_time,
        current_time,
        ctx.accounts.config.next_halving_time,
        ctx.accounts.config.halving_interval,
    )?;
    
    // Calculate distribution based on referral scenario
    let (claimant_amount, _l1_amount, _l2_amount) = validate_referral_scenario(
        farming_reward,
        ctx.accounts.user_state.referrer.is_some(),
        ctx.accounts.level2_referrer_state.is_some(),
        ctx.accounts.level1_referrer.as_ref()
            .map(|r| r.key() == ctx.accounts.config.protocol_referral_address)
            .unwrap_or(false),
        ctx.accounts.level2_referrer.as_ref()
            .map(|r| r.key() == ctx.accounts.config.protocol_referral_address)
            .unwrap_or(false),
        ctx.accounts.user.key() == ctx.accounts.config.protocol_referral_address,
    )?;
    
    // Add pending referral rewards
    let pending_referral_rewards = ctx.accounts.user_state.pending_referral_rewards;
    let total_reward = claimant_amount + pending_referral_rewards;
    
    // Check supply cap
    crate::validation::economic_validation::validate_supply_cap(
        ctx.accounts.config.total_supply_minted,
        total_reward
    )?;
    
    // Mint total reward to user
    mint_tokens_to_user(
        &ctx.accounts.reward_mint,
        &ctx.accounts.user_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        ctx.bumps.mint_authority,
        total_reward,
    )?;
    
    // Clear pending referral rewards
    ctx.accounts.user_state.pending_referral_rewards = 0;
    
    // Update timestamps and supply
    ctx.accounts.user_state.last_harvest_time = current_time;
    ctx.accounts.global_stats.last_update_time = current_time;
    ctx.accounts.config.total_supply_minted = ctx.accounts.config.total_supply_minted
        .checked_add(total_reward)
        .ok_or(GameError::CalculationOverflow)?;
    
    // Accumulate referral rewards for this user's referrers
    accumulate_referral_rewards_for_referrers(&mut ctx, farming_reward)?;
    
    // Log the complete transaction
    msg!("🎯 Farming reward: {} WEED ({}% of base)", claimant_amount, (claimant_amount * 100) / farming_reward);
    if pending_referral_rewards > 0 {
        msg!("💰 Referral rewards: {} WEED", pending_referral_rewards);
    }
    msg!("💎 Total claimed: {} WEED", total_reward);
    
    Ok(())
}

/// Accumulate referral rewards for this user's referrers
fn accumulate_referral_rewards_for_referrers(
    ctx: &mut Context<ClaimRewardWithReferralRewards>,
    base_reward: u64,
) -> Result<()> {
    // Don't process referrals for protocol address
    if ctx.accounts.user.key() == ctx.accounts.config.protocol_referral_address {
        return Ok(());
    }
    
    let (level1_reward, level2_reward) = calculate_referral_rewards(base_reward)?;
    
    // Level 1 referral accumulation
    if let (Some(l1_state), Some(l1_key)) = (
        &mut ctx.accounts.level1_referrer_state,
        &ctx.accounts.level1_referrer,
    ) {
        if l1_key.key() != ctx.accounts.config.protocol_referral_address {
            l1_state.pending_referral_rewards = l1_state.pending_referral_rewards
                .checked_add(level1_reward)
                .ok_or(GameError::CalculationOverflow)?;
            
            msg!("💰 Level 1 referral accumulated: {} WEED for {}", level1_reward, l1_key.key());
        }
    }
    
    // Level 2 referral accumulation
    if let (Some(l2_state), Some(l2_key)) = (
        &mut ctx.accounts.level2_referrer_state,
        &ctx.accounts.level2_referrer,
    ) {
        if l2_key.key() != ctx.accounts.config.protocol_referral_address {
            l2_state.pending_referral_rewards = l2_state.pending_referral_rewards
                .checked_add(level2_reward)
                .ok_or(GameError::CalculationOverflow)?;
            
            msg!("💰 Level 2 referral accumulated: {} WEED for {}", level2_reward, l2_key.key());
        }
    }
    
    Ok(())
}