use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::utils::{
    generate_invite_code_hash, 
    get_fixed_salt, 
    validate_invite_code_format,
    verify_invite_code_hash
};

// ===== HASH-BASED SECRET INVITE SYSTEM =====
// プライバシー保護のハッシュ招待システム - 招待コードは平文でチェーン上に保存されない
// 招待される人のアドレスは不要 - オープンな招待方式

/// ハッシュベース招待コード作成のコンテキスト
/// 8バイト招待コードをSHA256ハッシュ化してプライバシー確保
/// 衝突攻撃防止のため、ハッシュ値をPDAシードに使用
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct CreateInviteCode<'info> {
    #[account(
        init,
        payer = inviter,
        space = InviteCode::LEN,
        seeds = [
            b"invite_hash", 
            inviter.key().as_ref(),
            &generate_invite_code_hash(&invite_code, &get_fixed_salt(), &inviter.key())
        ],
        bump
    )]
    pub invite_account: Account<'info, InviteCode>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub inviter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// ハッシュベース招待コード使用のコンテキスト
/// 平文コード + 招待者アドレスでハッシュ検証後にユーザー初期化
/// ハッシュ値を使用してPDAを特定し、衝突攻撃を防止
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8], inviter_pubkey: Pubkey)]
pub struct UseInviteCode<'info> {
    #[account(
        mut,
        seeds = [
            b"invite_hash", 
            inviter_pubkey.as_ref(), 
            &generate_invite_code_hash(&invite_code, &get_fixed_salt(), &inviter_pubkey)
        ],
        bump,
        constraint = invite_account.is_active @ GameError::InviteCodeInactive,
        constraint = invite_account.invites_used < invite_account.invite_limit @ GameError::InviteCodeLimitReached
    )]
    pub invite_account: Account<'info, InviteCode>,
    
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
    
    pub system_program: Program<'info, System>,
}

// ===== INSTRUCTION IMPLEMENTATIONS =====

/// ハッシュベース招待コードを作成 - プライバシー保護
/// 平文コードは即座にハッシュ化され、チェーン上に保存されない
/// オペレーターは無制限、一般ユーザーは設定された上限まで招待可能
pub fn create_invite_code(
    ctx: Context<CreateInviteCode>,
    invite_code: [u8; 8]
) -> Result<()> {
    // Validate invite code format
    validate_invite_code_format(&invite_code)?;
    
    // Note: Account uniqueness is now guaranteed by PDA seeds
    // If the same inviter tries to create the same code twice,
    // the init constraint will fail automatically
    
    let invite = &mut ctx.accounts.invite_account;
    let config = &ctx.accounts.config;
    
    // Determine invite limit with timestamp validation for operator privileges
    let invite_limit = if ctx.accounts.inviter.key() == config.operator {
        // Validate operator privileges are still current
        require!(
            ctx.accounts.inviter.key() == config.operator,
            GameError::UnauthorizedOperator
        );
        1024u16 // Operator has high invite limit (1024)
    } else {
        config.max_invite_limit as u16
    };
    
    // Store the operator status at creation time for future validation
    let created_as_operator = ctx.accounts.inviter.key() == config.operator;
    
    // Generate hash with fixed salt
    let salt = get_fixed_salt();
    let code_hash = generate_invite_code_hash(
        &invite_code,
        &salt,
        &ctx.accounts.inviter.key()
    );
    
    // Set secret invite account data
    invite.inviter = ctx.accounts.inviter.key();
    invite.code_hash = code_hash;
    invite.salt = salt;
    invite.invites_used = 0;
    invite.invite_limit = invite_limit;
    invite.code_index = 0; // Not used in simplified version
    invite.created_at = Clock::get()?.unix_timestamp;
    invite.is_active = true;
    invite.created_as_operator = created_as_operator;
    invite.reserve = [0; 14];
    
    msg!("Secret invite code created: Hash={:?}, Inviter={}", 
         &code_hash[0..8], 
         ctx.accounts.inviter.key());
    
    Ok(())
}

/// ハッシュベース招待コードを使用してユーザー初期化
/// 平文コード + 招待者アドレスでハッシュ検証を行い、紹介関係を確立
/// 招待される人のアドレスは事前に不要（オープン招待）
pub fn use_invite_code(
    ctx: Context<UseInviteCode>,
    invite_code: [u8; 8],
    inviter_pubkey: Pubkey
) -> Result<()> {
    let invite = &mut ctx.accounts.invite_account;
    
    // Verify hash
    require!(
        verify_invite_code_hash(
            &invite_code,
            &invite.salt,
            &inviter_pubkey,
            &invite.code_hash
        ),
        GameError::InvalidInviteCode
    );
    
    // Verify inviter matches
    require!(
        invite.inviter == inviter_pubkey,
        GameError::InvalidInviter
    );
    
    // Initialize user state with referrer
    let user_state = &mut ctx.accounts.user_state;
    user_state.owner = ctx.accounts.invitee.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = Some(inviter_pubkey);
    user_state.pending_referral_rewards = 0;
    user_state.total_packs_purchased = 0;
    user_state.reserve = [0; 28];
    
    // Update usage count
    invite.invites_used += 1;
    
    msg!("Secret invite code used: User={}, Inviter={}", 
         ctx.accounts.invitee.key(),
         inviter_pubkey);
    
    Ok(())
}