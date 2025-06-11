use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;

/// Context for converting fees to SOL via Meteora
#[derive(Accounts)]
pub struct ConvertFeesToSol<'info> {
    #[account(
        mut,
        seeds = [b"fee_pool"],
        bump
    )]
    pub fee_pool: Account<'info, FeePool>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    /// Fee pool's $WEED token account
    #[account(
        mut,
        constraint = fee_pool_token_account.owner == fee_pool.key(),
        constraint = fee_pool_token_account.mint == reward_mint.key()
    )]
    pub fee_pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    /// Treasury account to receive SOL
    #[account(
        mut,
        constraint = treasury_account.key() == config.treasury
    )]
    /// CHECK: Treasury address from config
    pub treasury_account: UncheckedAccount<'info>,
    
    // Meteora DEX accounts
    /// CHECK: Meteora pool account for WEED/SOL pair
    pub meteora_pool: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: Pool's $WEED token vault
    pub pool_weed_vault: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: Pool's SOL vault
    pub pool_sol_vault: UncheckedAccount<'info>,
    
    #[account(mut)]
    /// CHECK: Pool authority
    pub pool_authority: UncheckedAccount<'info>,
    
    /// CHECK: Meteora program ID
    pub meteora_program: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Convert accumulated fees to SOL via Meteora DEX
pub fn convert_fees_to_sol(ctx: Context<ConvertFeesToSol>) -> Result<()> {
    let fee_pool = &mut ctx.accounts.fee_pool;
    let fee_pool_token_account = &ctx.accounts.fee_pool_token_account;
    
    // Check if there are any fees to convert
    let weed_amount = fee_pool_token_account.amount;
    require!(weed_amount > 0, GameError::NoRewardToClaim);
    
    // Create swap instruction for Meteora
    // Note: This is a placeholder - actual Meteora integration would require
    // their specific program interface and instruction format
    let swap_instruction = create_meteora_swap_instruction(
        &ctx.accounts.meteora_pool.key(),
        &ctx.accounts.pool_weed_vault.key(),
        &ctx.accounts.pool_sol_vault.key(),
        &fee_pool_token_account.key(),
        &ctx.accounts.treasury_account.key(),
        weed_amount,
        0, // minimum SOL out (TODO: implement slippage protection)
    )?;
    
    // Execute CPI to Meteora
    let fee_pool_seeds = &[b"fee_pool".as_ref(), &[ctx.bumps.fee_pool]];
    let signer_seeds = &[&fee_pool_seeds[..]];
    
    anchor_lang::solana_program::program::invoke_signed(
        &swap_instruction,
        &[
            ctx.accounts.meteora_pool.to_account_info(),
            ctx.accounts.pool_weed_vault.to_account_info(),
            ctx.accounts.pool_sol_vault.to_account_info(),
            ctx.accounts.fee_pool_token_account.to_account_info(),
            ctx.accounts.treasury_account.to_account_info(),
            ctx.accounts.pool_authority.to_account_info(),
            ctx.accounts.meteora_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    
    // Update fee pool state
    fee_pool.last_collection_time = Clock::get()?.unix_timestamp;
    
    msg!("Converted {} WEED to SOL via Meteora for treasury: {}", 
         weed_amount, ctx.accounts.treasury_account.key());
    
    Ok(())
}

/// Create Meteora swap instruction
/// This is a placeholder function - actual implementation would depend on Meteora's interface
fn create_meteora_swap_instruction(
    _pool: &Pubkey,
    _pool_weed_vault: &Pubkey,
    _pool_sol_vault: &Pubkey,
    _source_account: &Pubkey,
    _destination_account: &Pubkey,
    _amount_in: u64,
    _minimum_amount_out: u64,
) -> Result<anchor_lang::solana_program::instruction::Instruction> {
    // This is a placeholder - actual Meteora integration would require:
    // 1. Meteora program ID
    // 2. Correct instruction format for their swap function
    // 3. Proper account ordering and constraints
    
    // For now, return an error indicating this needs implementation
    Err(GameError::InvalidConfig.into())
}

/// Context for updating Meteora pool configuration
#[derive(Accounts)]
pub struct UpdateMeteoraConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Update Meteora pool configuration (admin only)
pub fn update_meteora_config(
    ctx: Context<UpdateMeteoraConfig>,
    meteora_pool: Pubkey,
    pool_weed_vault: Pubkey,
    pool_sol_vault: Pubkey,
) -> Result<()> {
    let _config = &mut ctx.accounts.config;
    
    // Store Meteora configuration in config
    // This would require adding fields to Config struct
    msg!("Meteora configuration updated by admin: {}", ctx.accounts.admin.key());
    msg!("Pool: {}, WEED Vault: {}, SOL Vault: {}", 
         meteora_pool, pool_weed_vault, pool_sol_vault);
    
    Ok(())
}