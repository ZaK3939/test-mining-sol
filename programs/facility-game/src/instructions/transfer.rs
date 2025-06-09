use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;

/// Context for transferring tokens with fee
#[derive(Accounts)]
pub struct TransferWithFee<'info> {
    #[account(
        mut,
        constraint = from_token_account.owner == from.key(),
        constraint = from_token_account.mint == reward_mint.key()
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = to_token_account.mint == reward_mint.key()
    )]
    pub to_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        mut,
        constraint = treasury_token_account.mint == reward_mint.key(),
        constraint = treasury_token_account.owner == config.treasury
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub from: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Transfer tokens with 2% fee
pub fn transfer_with_fee(ctx: Context<TransferWithFee>, amount: u64) -> Result<()> {
    // 手数料を計算（2%）
    let fee_amount = amount
        .checked_mul(2)
        .and_then(|result| result.checked_div(100))
        .ok_or(GameError::CalculationOverflow)?;
        
    // 受信者が受け取る実際の金額（98%）
    let transfer_amount = amount
        .checked_sub(fee_amount)
        .ok_or(GameError::CalculationOverflow)?;
    
    // 送信者の残高をチェック
    require!(
        ctx.accounts.from_token_account.amount >= amount,
        GameError::InsufficientFunds
    );
    
    // 1. 手数料をトレジャリーに転送
    if fee_amount > 0 {
        let fee_transfer_accounts = Transfer {
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.treasury_token_account.to_account_info(),
            authority: ctx.accounts.from.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, fee_transfer_accounts);
        token::transfer(cpi_ctx, fee_amount)?;
    }
    
    // 2. 残りの金額を受信者に転送
    if transfer_amount > 0 {
        let main_transfer_accounts = Transfer {
            from: ctx.accounts.from_token_account.to_account_info(),
            to: ctx.accounts.to_token_account.to_account_info(),
            authority: ctx.accounts.from.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, main_transfer_accounts);
        token::transfer(cpi_ctx, transfer_amount)?;
    }
    
    msg!("Transfer completed: {} total, {} fee (2%), {} to recipient from: {} to: {}", 
         amount, fee_amount, transfer_amount, ctx.accounts.from.key(), ctx.accounts.to_token_account.owner);
    Ok(())
}