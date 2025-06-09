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

/// Context for distributing referral rewards
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
    
    #[account(mut)]
    pub referrer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Claim rewards
pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let config = &mut ctx.accounts.config;
    
    require!(user_state.has_facility, GameError::NoFacility);

    let current_time = Clock::get()?.unix_timestamp;
    
    // Check and update halving
    while current_time >= config.next_halving_time {
        config.base_rate = config.base_rate / 2;
        config.next_halving_time += config.halving_interval;
        msg!("Halving occurred! New base rate: {}", config.base_rate);
    }

    // Calculate reward
    let time_elapsed = current_time - user_state.last_harvest_time;
    let reward_amount = calculate_reward(time_elapsed as u64, user_state.total_grow_power, config.base_rate)?;

    require!(reward_amount > 0, GameError::NoRewardToClaim);

    // Mint tokens
    mint_tokens_to_user(
        &ctx.accounts.reward_mint,
        &ctx.accounts.user_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        ctx.bumps.mint_authority,
        reward_amount,
    )?;

    // Handle referral rewards if referrer exists
    if let Some(referrer_pubkey) = user_state.referrer {
        let referral_reward = calculate_referral_reward(reward_amount)?;
        if referral_reward > 0 {
            msg!("Referral reward calculated: {} tokens for referrer: {}", 
                 referral_reward, referrer_pubkey);
        }
    }

    user_state.last_harvest_time = current_time;

    msg!("Reward claimed: {} tokens for user: {}", 
         reward_amount, ctx.accounts.user.key());
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
    
    // Mint tokens
    mint_tokens_to_user(
        &ctx.accounts.reward_mint,
        &ctx.accounts.referrer_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        ctx.bumps.mint_authority,
        reward_amount,
    )?;
    
    referrer_state.pending_referral_rewards = 0;
    
    msg!("Referral rewards claimed: {} tokens for referrer: {}", 
         reward_amount, ctx.accounts.referrer.key());
    Ok(())
}