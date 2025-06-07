use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};

declare_id!("EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89");

pub mod state;
pub mod instructions;
pub mod error;

use instructions::*;
use error::*;

#[program]
pub mod facility_game {
    use super::*;

    /// グローバル設定を初期化
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        base_rate: u64,
        halving_interval: i64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.base_rate = base_rate;
        config.halving_interval = halving_interval;
        config.next_halving_time = Clock::get()?.unix_timestamp + halving_interval;
        config.admin = ctx.accounts.admin.key();
        
        msg!("Config initialized with base_rate: {}, halving_interval: {}", base_rate, halving_interval);
        Ok(())
    }

    /// 報酬トークンのMintアカウントを作成
    pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
        msg!("Reward mint created with authority: {}", ctx.accounts.mint_authority.key());
        Ok(())
    }

    /// ユーザーアカウントを初期化
    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;
        user_state.owner = ctx.accounts.user.key();
        user_state.total_grow_power = 0;
        user_state.last_harvest_time = Clock::get()?.unix_timestamp;
        user_state.has_facility = false;
        user_state.reserve = [0; 64];

        msg!("User initialized: {}", ctx.accounts.user.key());
        Ok(())
    }

    /// 施設を購入し、初期マシンを配置
    pub fn buy_facility(ctx: Context<BuyFacility>) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;
        
        // 既に施設を持っているかチェック
        require!(!user_state.has_facility, GameError::AlreadyHasFacility);
        
        let facility = &mut ctx.accounts.facility;
        // 施設を初期化
        if facility.owner != Pubkey::default() {
        return Err(GameError::AlreadyHasFacility.into());
        }

        facility.owner = ctx.accounts.user.key();
        facility.machine_count = 1; // 初期マシン1台
        facility.total_grow_power = 100; // 初期Grow Power
        facility.reserve = [0; 64];

        // ユーザー状態を更新
        user_state.has_facility = true;
        user_state.total_grow_power = facility.total_grow_power;
        user_state.last_harvest_time = Clock::get()?.unix_timestamp;

        msg!("Facility purchased for user: {}, initial grow power: {}", 
             ctx.accounts.user.key(), facility.total_grow_power);
        Ok(())
    }

    /// 報酬を請求
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;
        let config = &mut ctx.accounts.config;
        
        // 施設を持っているかチェック
        require!(user_state.has_facility, GameError::NoFacility);

        let current_time = Clock::get()?.unix_timestamp;
        
        // 半減期のチェックと更新
        while current_time >= config.next_halving_time {
            config.base_rate = config.base_rate / 2;
            config.next_halving_time += config.halving_interval;
            msg!("Halving occurred! New base rate: {}", config.base_rate);
        }

        // 報酬計算（経過時間 × Grow Power × base_rate）
        let time_elapsed = current_time - user_state.last_harvest_time;
        let reward_amount = (time_elapsed as u64)
            .checked_mul(user_state.total_grow_power)
            .and_then(|result| result.checked_mul(config.base_rate))
            .and_then(|result| result.checked_div(1000)) // 1000で割ってレートを調整
            .ok_or(GameError::CalculationOverflow)?;

        require!(reward_amount > 0, GameError::NoRewardToClaim);

        // トークンをミント（修正版）
        let cpi_accounts = MintTo {
            mint: ctx.accounts.reward_mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };

        // 修正：正しいseedの構造
        let authority_bump = ctx.bumps.mint_authority;
        let seeds = &[
            b"mint_authority".as_ref(),
            &[authority_bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token::mint_to(cpi_ctx, reward_amount)?;

        // 最終収穫時刻を更新
        user_state.last_harvest_time = current_time;

        msg!("Reward claimed: {} tokens for user: {}", 
             reward_amount, ctx.accounts.user.key());
        Ok(())
    }
}