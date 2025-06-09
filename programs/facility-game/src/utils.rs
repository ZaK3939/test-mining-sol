use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;

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

/// Determine seed rarity based on random seed
/// 
/// # 確率分布
/// - Common: 70% (0-69)
/// - Rare: 20% (70-89) 
/// - Epic: 8% (90-97)
/// - Legendary: 2% (98-99)
pub fn determine_rarity(random_seed: u64) -> SeedRarity {
    let roll = random_seed % 100; // 0-99の値を取得
    
    match roll {
        0..=69 => SeedRarity::Common,     // 70%
        70..=89 => SeedRarity::Rare,      // 20%
        90..=97 => SeedRarity::Epic,      // 8%
        98..=99 => SeedRarity::Legendary, // 2%
        _ => SeedRarity::Common, // フォールバック
    }
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