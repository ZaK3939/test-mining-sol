use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::state::*;
use crate::error::*;

/// Context for facility purchase
#[derive(Accounts)]
pub struct BuyFacility<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        init,
        payer = user,
        space = Facility::LEN,
        seeds = [b"facility", user.key().as_ref()],
        bump
    )]
    pub facility: Account<'info, Facility>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for upgrading facility
#[derive(Accounts)]
pub struct UpgradeFacility<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump,
        constraint = user_state.has_facility @ GameError::NoFacility
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"facility", user.key().as_ref()],
        bump,
        constraint = facility.owner == user.key()
    )]
    pub facility: Account<'info, Facility>,
    
    #[account(
        mut,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
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

/// Purchase facility and place initial machine
pub fn buy_facility(ctx: Context<BuyFacility>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    
    require!(!user_state.has_facility, GameError::AlreadyHasFacility);
    
    let facility = &mut ctx.accounts.facility;
    if facility.owner != Pubkey::default() {
        return Err(GameError::AlreadyHasFacility.into());
    }

    facility.owner = ctx.accounts.user.key();
    facility.facility_size = 1;
    facility.max_capacity = crate::state::Facility::calculate_max_capacity(1);
    facility.machine_count = 1;
    facility.total_grow_power = 100;
    facility.reserve = [0; 56];

    user_state.has_facility = true;
    user_state.total_grow_power = facility.total_grow_power;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;

    msg!("Facility purchased for user: {}, initial grow power: {}", 
         ctx.accounts.user.key(), facility.total_grow_power);
    Ok(())
}

/// Upgrade facility size
pub fn upgrade_facility(ctx: Context<UpgradeFacility>) -> Result<()> {
    let facility = &mut ctx.accounts.facility;
    let user_token_account = &ctx.accounts.user_token_account;
    
    let upgrade_cost = crate::state::Facility::calculate_upgrade_cost(facility.facility_size);
    
    require!(
        user_token_account.amount >= upgrade_cost,
        GameError::InsufficientFunds
    );
    
    // トークンをバーン（破棄）
    let burn_accounts = Burn {
        mint: ctx.accounts.reward_mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, burn_accounts);
    token::burn(cpi_ctx, upgrade_cost)?;
    
    let old_size = facility.facility_size;
    facility.facility_size += 1;
    facility.max_capacity = crate::state::Facility::calculate_max_capacity(facility.facility_size);
    
    msg!("Facility upgraded from size {} to {} for user: {}, new capacity: {}, cost: {} tokens", 
         old_size, facility.facility_size, ctx.accounts.user.key(), facility.max_capacity, upgrade_cost);
    Ok(())
}

/// Add machine to facility
pub fn add_machine(ctx: Context<UpgradeFacility>) -> Result<()> {
    let facility = &mut ctx.accounts.facility;
    let user_state = &mut ctx.accounts.user_state;
    let user_token_account = &ctx.accounts.user_token_account;
    
    require!(
        facility.machine_count < facility.max_capacity,
        GameError::FacilityAtMaxCapacity
    );
    
    let machine_cost = 500u64;
    
    require!(
        user_token_account.amount >= machine_cost,
        GameError::InsufficientFunds
    );
    
    // トークンをバーン（破棄）
    let burn_accounts = Burn {
        mint: ctx.accounts.reward_mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, burn_accounts);
    token::burn(cpi_ctx, machine_cost)?;
    
    facility.machine_count += 1;
    facility.total_grow_power += 100;
    
    user_state.total_grow_power = facility.total_grow_power;
    
    msg!("Machine added to facility for user: {}, new machine count: {}, total grow power: {}, cost: {} tokens", 
         ctx.accounts.user.key(), facility.machine_count, facility.total_grow_power, machine_cost);
    Ok(())
}