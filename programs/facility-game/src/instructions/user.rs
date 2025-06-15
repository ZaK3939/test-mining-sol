use anchor_lang::prelude::*;
use crate::state::*;

/// Context for user initialization (admin/operator only - without invite code)
#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(
        init,
        payer = admin,
        space = UserState::LEN,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    /// System configuration to verify admin/operator authority
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() || config.operator == admin.key() @ crate::error::GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    /// User account to be initialized (does not need to sign)
    /// CHECK: User account being initialized
    pub user: UncheckedAccount<'info>,
    
    /// Admin or operator who can create users without invite codes
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Initialize user account (admin/operator only - without invite code)
/// Only admin or operator can create users without going through the invite system
pub fn init_user(ctx: Context<InitUser>, referrer: Option<Pubkey>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    
    // Initialize user state
    user_state.owner = ctx.accounts.user.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = referrer;
    user_state.pending_referral_rewards = 0;
    user_state.total_packs_purchased = 0;
    user_state.reserve = [0; 28];

    msg!("User initialized by admin {} for user: {} with referrer: {:?}", 
         ctx.accounts.admin.key(),
         ctx.accounts.user.key(), 
         referrer);
    Ok(())
}