use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;

/// Context for purchasing mystery box
#[derive(Accounts)]
#[instruction(box_id: u64)]
pub struct PurchaseMysteryBox<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        init,
        payer = purchaser,
        space = MysteryBox::LEN,
        seeds = [b"mystery_box", box_id.to_le_bytes().as_ref()],
        bump
    )]
    pub mystery_box: Account<'info, MysteryBox>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = purchaser_token_account.owner == purchaser.key(),
        constraint = purchaser_token_account.mint == reward_mint.key()
    )]
    pub purchaser_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = treasury_token_account.mint == reward_mint.key(),
        constraint = treasury_token_account.owner == config.treasury
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub purchaser: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Context for opening mystery box
#[derive(Accounts)]
#[instruction(box_id: u64, seed_id: u64)]
pub struct OpenMysteryBox<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(
        mut,
        seeds = [b"mystery_box", box_id.to_le_bytes().as_ref()],
        bump,
        constraint = mystery_box.purchaser == opener.key(),
        constraint = !mystery_box.is_opened
    )]
    pub mystery_box: Account<'info, MysteryBox>,
    
    #[account(
        init,
        payer = opener,
        space = Seed::LEN,
        seeds = [b"seed", seed_id.to_le_bytes().as_ref()],
        bump
    )]
    pub seed: Account<'info, Seed>,
    
    #[account(mut)]
    pub opener: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Purchase mystery box
pub fn purchase_mystery_box(ctx: Context<PurchaseMysteryBox>, box_id: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let mystery_box = &mut ctx.accounts.mystery_box;
    let purchaser_token_account = &ctx.accounts.purchaser_token_account;
    
    let cost = config.mystery_box_cost;
    
    require!(
        purchaser_token_account.amount >= cost,
        GameError::InsufficientFunds
    );
    
    // $WEEDトークンをトレジャリーに転送
    let transfer_accounts = Transfer {
        from: ctx.accounts.purchaser_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.purchaser.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);
    token::transfer(cpi_ctx, cost)?;
    
    // ランダムシードを生成（簡易版：現在時刻 + スロット + ユーザーキー）
    let clock = Clock::get()?;
    let random_seed = clock
        .unix_timestamp
        .wrapping_add(clock.slot as i64)
        .wrapping_add(ctx.accounts.purchaser.key().to_bytes()[0] as i64) as u64;
    
    // ミステリーボックスを初期化
    mystery_box.purchaser = ctx.accounts.purchaser.key();
    mystery_box.purchased_at = clock.unix_timestamp;
    mystery_box.cost_paid = cost;
    mystery_box.is_opened = false;
    mystery_box.random_seed = random_seed;
    mystery_box.box_id = box_id;
    mystery_box.reserve = [0; 32];
    
    // カウンターを更新
    config.mystery_box_counter += 1;
    
    msg!("Mystery box purchased: ID {}, cost: {} $WEED, random_seed: {}", 
         box_id, cost, random_seed);
    Ok(())
}

/// Open mystery box and reveal seed
pub fn open_mystery_box(ctx: Context<OpenMysteryBox>, box_id: u64, seed_id: u64) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let mystery_box = &mut ctx.accounts.mystery_box;
    let seed = &mut ctx.accounts.seed;
    
    // ランダムシードに基づいてレアリティを決定
    let rarity = crate::utils::determine_rarity(mystery_box.random_seed);
    let multiplier = rarity.get_multiplier();
    
    // シードNFTを初期化
    seed.owner = ctx.accounts.opener.key();
    seed.rarity = rarity;
    seed.grow_power_multiplier = multiplier;
    seed.is_planted = false;
    seed.planted_facility = None;
    seed.created_at = Clock::get()?.unix_timestamp;
    seed.seed_id = seed_id;
    seed.reserve = [0; 32];
    
    // ミステリーボックスを開封済みにマーク
    mystery_box.is_opened = true;
    
    // シードカウンターを更新
    config.seed_counter += 1;
    
    msg!("Mystery box opened: ID {}, seed ID {}, rarity: {}, multiplier: {}x", 
         box_id, seed_id, rarity.as_str(), multiplier);
    Ok(())
}