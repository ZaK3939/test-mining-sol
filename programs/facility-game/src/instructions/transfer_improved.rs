use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use crate::state::*;
use crate::utils::*;
use crate::error::*;

/// 改善されたTransfer手数料システム
/// FeePool経由でMeteora自動変換に対応

// ===== IMPROVED TRANSFER CONTEXT =====

#[derive(Accounts)]
pub struct TransferWithImprovedFee<'info> {
    /// 送信者のトークンアカウント
    #[account(
        mut,
        constraint = from_token_account.owner == from.key(),
        constraint = from_token_account.mint == weed_mint.key()
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    
    /// 受信者のトークンアカウント
    #[account(
        mut,
        constraint = to_token_account.mint == weed_mint.key()
    )]
    pub to_token_account: Account<'info, TokenAccount>,
    
    /// システム設定
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    /// FeePool (手数料蓄積用)
    #[account(
        mut,
        seeds = [b"fee_pool"],
        bump
    )]
    pub fee_pool: Account<'info, FeePool>,

    /// FeePool のトークンアカウント
    #[account(
        mut,
        constraint = fee_pool_token_account.owner == fee_pool.key(),
        constraint = fee_pool_token_account.mint == weed_mint.key()
    )]
    pub fee_pool_token_account: Account<'info, TokenAccount>,
    
    /// WEED mint
    #[account(
        seeds = [b"reward_mint"],
        bump
    )]
    pub weed_mint: Account<'info, Mint>,
    
    /// 送信者
    #[account(mut)]
    pub from: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// ===== LEGACY COMPATIBILITY CONTEXT =====

/// レガシー互換用コンテキスト (Treasury直送)
#[derive(Accounts)]
pub struct TransferWithLegacyFee<'info> {
    #[account(
        mut,
        constraint = from_token_account.owner == from.key(),
        constraint = from_token_account.mint == weed_mint.key()
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = to_token_account.mint == weed_mint.key()
    )]
    pub to_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    /// Treasury token account (レガシー)
    #[account(
        mut,
        constraint = treasury_token_account.mint == weed_mint.key(),
        constraint = treasury_token_account.owner == config.treasury
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"reward_mint"],
        bump
    )]
    pub weed_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub from: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// ===== CORE TRANSFER IMPLEMENTATIONS =====

/// 改善された手数料付きTransfer (FeePool経由)
pub fn transfer_with_improved_fee(
    mut ctx: Context<TransferWithImprovedFee>, 
    amount: u64
) -> Result<()> {
    // 1. 残高検証
    validate_token_balance(&ctx.accounts.from_token_account, amount)?;
    
    // 2. 手数料計算
    let (fee_amount, transfer_amount) = calculate_transfer_fee(amount)?;
    
    // 3. 手数料をFeePoolに送金
    if fee_amount > 0 {
        transfer_fee_to_fee_pool(&ctx, fee_amount)?;
        
        // 4. FeePool累積額更新
        update_fee_pool_accumulation(&mut ctx, fee_amount)?;
        
        // 5. 自動変換トリガーチェック
        check_and_trigger_auto_conversion(&ctx)?;
    }
    
    // 6. 本体金額を受信者に送金
    if transfer_amount > 0 {
        transfer_to_recipient(&ctx, transfer_amount)?;
    }
    
    msg!("Improved transfer completed: {} total, {} fee, {} to recipient", 
         amount, fee_amount, transfer_amount);
    
    Ok(())
}

/// レガシー互換Transfer (Treasury直送)
pub fn transfer_with_legacy_fee(
    ctx: Context<TransferWithLegacyFee>, 
    amount: u64
) -> Result<()> {
    validate_token_balance(&ctx.accounts.from_token_account, amount)?;
    
    let (fee_amount, transfer_amount) = calculate_transfer_fee(amount)?;
    
    // レガシー: 直接Treasury送金
    if fee_amount > 0 {
        transfer_fee_to_treasury_legacy(&ctx, fee_amount)?;
    }
    
    if transfer_amount > 0 {
        transfer_to_recipient_legacy(&ctx, transfer_amount)?;
    }
    
    msg!("Legacy transfer completed: {} total, {} fee, {} to recipient", 
         amount, fee_amount, transfer_amount);
    
    Ok(())
}

// ===== HELPER FUNCTIONS =====

/// FeePoolに手数料送金
fn transfer_fee_to_fee_pool(
    ctx: &Context<TransferWithImprovedFee>, 
    fee_amount: u64
) -> Result<()> {
    let fee_transfer_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.fee_pool_token_account.to_account_info(),
        authority: ctx.accounts.from.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, fee_transfer_accounts);
    
    token::transfer(cpi_ctx, fee_amount)?;
    
    msg!("Fee transferred to FeePool: {} WEED", fee_amount);
    Ok(())
}

/// 受信者に本体金額送金
fn transfer_to_recipient(
    ctx: &Context<TransferWithImprovedFee>, 
    transfer_amount: u64
) -> Result<()> {
    let main_transfer_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.to_token_account.to_account_info(),
        authority: ctx.accounts.from.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, main_transfer_accounts);
    
    token::transfer(cpi_ctx, transfer_amount)
}

/// FeePool累積額更新
fn update_fee_pool_accumulation(
    ctx: &mut Context<TransferWithImprovedFee>, 
    fee_amount: u64
) -> Result<()> {
    let fee_pool = &mut ctx.accounts.fee_pool;
    
    fee_pool.accumulated_fees = fee_pool.accumulated_fees
        .checked_add(fee_amount)
        .ok_or(GameError::CalculationOverflow)?;
    
    msg!("FeePool accumulation updated: {} total", fee_pool.accumulated_fees);
    Ok(())
}

/// 自動変換トリガーチェック
fn check_and_trigger_auto_conversion(
    ctx: &Context<TransferWithImprovedFee>
) -> Result<()> {
    let fee_pool = &ctx.accounts.fee_pool;
    let current_balance = ctx.accounts.fee_pool_token_account.amount;
    
    // TODO: Config に auto_conversion_enabled フラグ追加が必要
    // let auto_conversion_enabled = ctx.accounts.config.meteora_auto_conversion;
    let auto_conversion_enabled = true; // 仮設定
    
    if !auto_conversion_enabled {
        return Ok(());
    }
    
    // しきい値チェック
    let conversion_threshold = 5000 * 1_000_000; // 5000 WEED
    if current_balance >= conversion_threshold {
        msg!("Auto-conversion threshold reached: {} WEED", current_balance);
        
        // TODO: 実際にはイベント発行してオフチェーンで処理
        // または別のトランザクションをトリガー
        emit!(AutoConversionTriggered {
            fee_pool: fee_pool.key(),
            weed_amount: current_balance,
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    
    Ok(())
}

// ===== LEGACY HELPERS =====

fn transfer_fee_to_treasury_legacy(
    ctx: &Context<TransferWithLegacyFee>, 
    fee_amount: u64
) -> Result<()> {
    let fee_transfer_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.from.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, fee_transfer_accounts);
    
    token::transfer(cpi_ctx, fee_amount)
}

fn transfer_to_recipient_legacy(
    ctx: &Context<TransferWithLegacyFee>, 
    transfer_amount: u64
) -> Result<()> {
    let main_transfer_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.to_token_account.to_account_info(),
        authority: ctx.accounts.from.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, main_transfer_accounts);
    
    token::transfer(cpi_ctx, transfer_amount)
}

// ===== EVENTS =====

#[event]
pub struct AutoConversionTriggered {
    pub fee_pool: Pubkey,
    pub weed_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeeCollected {
    pub from: Pubkey,
    pub to: Pubkey,
    pub fee_amount: u64,
    pub transfer_amount: u64,
    pub total_amount: u64,
    pub timestamp: i64,
}

// ===== BATCH OPERATIONS =====

/// バッチTransfer (複数受信者への効率的送金)
#[derive(Accounts)]
pub struct BatchTransferWithFee<'info> {
    #[account(
        mut,
        constraint = from_token_account.owner == from.key(),
        constraint = from_token_account.mint == weed_mint.key()
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        mut,
        seeds = [b"fee_pool"],
        bump
    )]
    pub fee_pool: Account<'info, FeePool>,

    #[account(
        mut,
        constraint = fee_pool_token_account.owner == fee_pool.key(),
        constraint = fee_pool_token_account.mint == weed_mint.key()
    )]
    pub fee_pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"reward_mint"],
        bump
    )]
    pub weed_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub from: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// バッチTransfer実行
pub fn batch_transfer_with_fee(
    mut ctx: Context<BatchTransferWithFee>,
    transfers: Vec<TransferInstruction>,
) -> Result<()> {
    require!(transfers.len() <= 10, GameError::TooManyTransfers); // 最大10件
    
    let mut total_fees = 0u64;
    let mut total_transfers = 0u64;
    
    for transfer_instruction in transfers.iter() {
        let (fee_amount, transfer_amount) = calculate_transfer_fee(transfer_instruction.amount)?;
        
        // TODO: 各受信者への転送実装
        // 現在は合計計算のみ
        total_fees = total_fees.checked_add(fee_amount)
            .ok_or(GameError::CalculationOverflow)?;
        total_transfers = total_transfers.checked_add(transfer_amount)
            .ok_or(GameError::CalculationOverflow)?;
    }
    
    // 総手数料をFeePoolに送金
    if total_fees > 0 {
        transfer_fee_to_fee_pool_batch(&ctx, total_fees)?;
        update_fee_pool_accumulation_batch(&mut ctx, total_fees)?;
    }
    
    msg!("Batch transfer completed: {} transfers, {} total fees", 
         transfers.len(), total_fees);
    
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TransferInstruction {
    pub recipient: Pubkey,
    pub amount: u64,
}

fn transfer_fee_to_fee_pool_batch(
    ctx: &Context<BatchTransferWithFee>, 
    total_fee_amount: u64
) -> Result<()> {
    let fee_transfer_accounts = Transfer {
        from: ctx.accounts.from_token_account.to_account_info(),
        to: ctx.accounts.fee_pool_token_account.to_account_info(),
        authority: ctx.accounts.from.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, fee_transfer_accounts);
    
    token::transfer(cpi_ctx, total_fee_amount)
}

fn update_fee_pool_accumulation_batch(
    ctx: &mut Context<BatchTransferWithFee>, 
    total_fee_amount: u64
) -> Result<()> {
    let fee_pool = &mut ctx.accounts.fee_pool;
    
    fee_pool.accumulated_fees = fee_pool.accumulated_fees
        .checked_add(total_fee_amount)
        .ok_or(GameError::CalculationOverflow)?;
    
    Ok(())
}