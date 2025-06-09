use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use mpl_token_metadata::types::DataV2;
use crate::state::*;

/// Context for initializing global configuration
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for creating reward token mint
#[derive(Accounts)]
pub struct CreateRewardMint<'info> {
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = mint_authority,
        seeds = [b"reward_mint"],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    
    /// @dev Set mint authority as PDA
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: mint authority PDA
    pub mint_authority: UncheckedAccount<'info>,
    
    /// Token metadata account (optional for test environments)
    /// CHECK: Metadata account will be created by Metaplex if available
    pub metadata_account: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    
    /// Token Metadata Program (optional for test environments)
    /// CHECK: Token metadata program
    pub token_metadata_program: UncheckedAccount<'info>,
}

/// Initialize global configuration
pub fn initialize_config(
    ctx: Context<InitializeConfig>,
    base_rate: u64,
    halving_interval: i64,
    treasury: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    
    config.base_rate = base_rate;
    config.halving_interval = halving_interval;
    config.treasury = treasury;
    config.next_halving_time = Clock::get()?.unix_timestamp + halving_interval;
    config.admin = ctx.accounts.admin.key();
    config.mystery_box_cost = 1000; // デフォルト1000 $WEED
    config.seed_counter = 0;
    config.mystery_box_counter = 0;
    config.reserve = [0; 8];
    
    msg!("Config initialized with base_rate: {}, halving_interval: {}, treasury: {}", 
         base_rate, halving_interval, treasury);
    
    Ok(())
}

/// Create reward token mint account
pub fn create_reward_mint(ctx: Context<CreateRewardMint>) -> Result<()> {
    msg!("Reward mint created with authority: {}", ctx.accounts.mint_authority.key());
    
    // $WEEDトークンのメタデータを設定
    let token_data = DataV2 {
        name: "Weed Token".to_string(),
        symbol: "WEED".to_string(),
        uri: "".to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };
    
    // mint_authorityのシード
    let bump = ctx.bumps.mint_authority;
    let seeds = &[b"mint_authority".as_ref(), &[bump]];
    let signer_seeds = &[&seeds[..]];
    
    // メタデータアカウントを作成するためのCPIを構築
    let create_metadata_accounts = mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
        .metadata(ctx.accounts.metadata_account.key())
        .mint(ctx.accounts.reward_mint.key())
        .mint_authority(ctx.accounts.mint_authority.key())
        .payer(ctx.accounts.admin.key())
        .update_authority(ctx.accounts.mint_authority.key(), true)
        .system_program(ctx.accounts.system_program.key())
        .rent(Some(ctx.accounts.rent.key()))
        .data(token_data)
        .is_mutable(true)
        .instruction();
    
    // CPIを実行（メタデータプログラムが存在する場合のみ）
    let result = anchor_lang::solana_program::program::invoke_signed(
        &create_metadata_accounts,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
        ],
        signer_seeds,
    );
    
    match result {
        Ok(_) => msg!("$WEED token metadata created successfully"),
        Err(e) => msg!("Warning: Failed to create metadata (may not be available in test): {:?}", e),
    }
    
    Ok(())
}