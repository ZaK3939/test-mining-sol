use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;

/// Context for creating an invite code
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct CreateInviteCode<'info> {
    #[account(
        init,
        payer = inviter,
        space = InviteCode::LEN,
        seeds = [b"invite_code", invite_code.as_ref()],
        bump
    )]
    pub invite_code_account: Account<'info, InviteCode>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub inviter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for using an invite code during user initialization
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct UseInviteCode<'info> {
    #[account(
        mut,
        seeds = [b"invite_code", invite_code.as_ref()],
        bump
    )]
    pub invite_code_account: Account<'info, InviteCode>,
    
    #[account(
        init,
        payer = invitee,
        space = UserState::LEN,
        seeds = [b"user", invitee.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub invitee: Signer<'info>,
    
    /// CHECK: Inviter address (does not need to sign)
    pub inviter: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for expanding invite limit (admin only)
#[derive(Accounts)]
pub struct ExpandInviteLimit<'info> {
    #[account(
        mut,
        seeds = [b"invite_code", invite_code_account.code.as_ref()],
        bump
    )]
    pub invite_code_account: Account<'info, InviteCode>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.admin == admin.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Create an invite code for a user
pub fn create_invite_code(ctx: Context<CreateInviteCode>, invite_code: [u8; 8]) -> Result<()> {
    let invite_code_account = &mut ctx.accounts.invite_code_account;
    let config = &ctx.accounts.config;
    
    // Validate invite code format (8 alphanumeric characters)
    for &byte in invite_code.iter() {
        require!(
            (byte >= b'0' && byte <= b'9') || 
            (byte >= b'A' && byte <= b'Z') || 
            (byte >= b'a' && byte <= b'z'),
            GameError::InvalidInviteCode
        );
    }
    
    invite_code_account.inviter = ctx.accounts.inviter.key();
    invite_code_account.invites_used = 0;
    invite_code_account.invite_limit = config.max_invite_limit;
    invite_code_account.code = invite_code;
    invite_code_account.created_at = Clock::get()?.unix_timestamp;
    invite_code_account.reserve = [0; 32];
    
    msg!("Invite code created: {:?} for inviter: {}", 
         core::str::from_utf8(&invite_code).unwrap_or("INVALID"), 
         ctx.accounts.inviter.key());
    
    Ok(())
}

/// Use an invite code to initialize a user with referrer
pub fn use_invite_code(
    ctx: Context<UseInviteCode>, 
    invite_code: [u8; 8]
) -> Result<()> {
    let invite_code_account = &mut ctx.accounts.invite_code_account;
    let user_state = &mut ctx.accounts.user_state;
    
    // Verify invite code hasn't reached limit
    require!(
        invite_code_account.invites_used < invite_code_account.invite_limit,
        GameError::InviteCodeLimitReached
    );
    
    // Verify the invite code matches
    require!(
        invite_code_account.code == invite_code,
        GameError::InvalidInviteCode
    );
    
    // Verify inviter matches the account
    require!(
        invite_code_account.inviter == ctx.accounts.inviter.key(),
        GameError::InvalidInviter
    );
    
    // Initialize user state with referrer
    user_state.owner = ctx.accounts.invitee.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = Some(invite_code_account.inviter);
    user_state.pending_referral_rewards = 0;
    user_state.reserve = [0; 32];
    
    // Increment invite usage
    invite_code_account.invites_used += 1;
    
    msg!("User {} initialized with referrer: {} using invite code: {:?}", 
         ctx.accounts.invitee.key(), 
         invite_code_account.inviter,
         core::str::from_utf8(&invite_code).unwrap_or("INVALID"));
    
    Ok(())
}

/// Expand invite limit for a specific invite code (admin only)
pub fn expand_invite_limit(
    ctx: Context<ExpandInviteLimit>, 
    additional_invites: u8
) -> Result<()> {
    let invite_code_account = &mut ctx.accounts.invite_code_account;
    
    invite_code_account.invite_limit = invite_code_account.invite_limit
        .checked_add(additional_invites)
        .ok_or(GameError::CalculationOverflow)?;
    
    msg!("Invite limit expanded by {} for invite code owned by: {}", 
         additional_invites, 
         invite_code_account.inviter);
    
    Ok(())
}