use anchor_lang::prelude::*;
use crate::state::*;

/// Context for user initialization
#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(
        init,
        payer = user,
        space = UserState::LEN,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Initialize user account
pub fn init_user(ctx: Context<InitUser>, referrer: Option<Pubkey>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    
    user_state.owner = ctx.accounts.user.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = referrer;
    user_state.pending_referral_rewards = 0;
    user_state.total_packs_purchased = 0;
    user_state.reserve = [0; 28];

    msg!("User initialized: {} with referrer: {:?}", ctx.accounts.user.key(), referrer);
    Ok(())
}