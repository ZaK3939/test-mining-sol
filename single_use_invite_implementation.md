# 一回限り招待コード実装

## 🎫 運営専用・一回限り招待コード

### 1. 状態定義

```rust
/// 一回限り招待コード（運営専用）
#[account]
pub struct SingleUseInviteCode {
    pub creator: Pubkey,           // 運営アドレス（operatorのみ）
    pub code: [u8; 8],            // 8文字招待コード
    pub is_used: bool,            // 使用済みフラグ
    pub used_by: Option<Pubkey>,  // 使用者のアドレス
    pub created_at: i64,          // 作成時刻
    pub used_at: Option<i64>,     // 使用時刻
    pub reserve: [u8; 32],        // 予約領域
}

impl SingleUseInviteCode {
    pub const LEN: usize = 8 + 32 + 8 + 1 + 1 + 32 + 8 + 1 + 8 + 32;
}
```

### 2. 作成命令（運営専用）

```rust
/// 一回限り招待コード作成コンテキスト
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct CreateSingleUseInviteCode<'info> {
    #[account(
        init,
        payer = operator,
        space = SingleUseInviteCode::LEN,
        seeds = [b"single_use_invite", invite_code.as_ref()],
        bump
    )]
    pub single_use_invite_account: Account<'info, SingleUseInviteCode>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.operator == operator.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub operator: Signer<'info>,  // 運営のみ
    
    pub system_program: Program<'info, System>,
}

/// 一回限り招待コード作成（運営専用）
pub fn create_single_use_invite_code(
    ctx: Context<CreateSingleUseInviteCode>, 
    invite_code: [u8; 8]
) -> Result<()> {
    // 招待コード形式検証
    for &byte in invite_code.iter() {
        require!(
            (byte >= b'0' && byte <= b'9') || 
            (byte >= b'A' && byte <= b'Z') || 
            (byte >= b'a' && byte <= b'z'),
            GameError::InvalidInviteCode
        );
    }
    
    let single_use_invite = &mut ctx.accounts.single_use_invite_account;
    
    single_use_invite.creator = ctx.accounts.operator.key();
    single_use_invite.code = invite_code;
    single_use_invite.is_used = false;
    single_use_invite.used_by = None;
    single_use_invite.created_at = Clock::get()?.unix_timestamp;
    single_use_invite.used_at = None;
    single_use_invite.reserve = [0; 32];
    
    msg!("Single-use invite code created: {:?} by operator: {}", 
         core::str::from_utf8(&invite_code).unwrap_or("INVALID"), 
         ctx.accounts.operator.key());
    
    Ok(())
}
```

### 3. 使用命令（誰でも、一回限り）

```rust
/// 一回限り招待コード使用コンテキスト
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct UseSingleUseInviteCode<'info> {
    #[account(
        mut,
        seeds = [b"single_use_invite", invite_code.as_ref()],
        bump,
        constraint = !single_use_invite_account.is_used @ GameError::InviteCodeAlreadyUsed
    )]
    pub single_use_invite_account: Account<'info, SingleUseInviteCode>,
    
    #[account(
        init,
        payer = user,
        space = UserState::LEN,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub user: Signer<'info>,  // 誰でも使用可能
    
    pub system_program: Program<'info, System>,
}

/// 一回限り招待コード使用
pub fn use_single_use_invite_code(
    ctx: Context<UseSingleUseInviteCode>, 
    invite_code: [u8; 8]
) -> Result<()> {
    let single_use_invite = &mut ctx.accounts.single_use_invite_account;
    
    // 招待コード照合
    require!(
        single_use_invite.code == invite_code,
        GameError::InvalidInviteCode
    );
    
    // 未使用確認（二重チェック）
    require!(
        !single_use_invite.is_used,
        GameError::InviteCodeAlreadyUsed
    );
    
    // ユーザー初期化
    let user_state = &mut ctx.accounts.user_state;
    user_state.owner = ctx.accounts.user.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = Some(ctx.accounts.config.protocol_referral_address); // 運営が紹介者
    user_state.pending_referral_rewards = 0;
    user_state.reserve = [0; 32];
    
    // 使用済みマーク
    single_use_invite.is_used = true;
    single_use_invite.used_by = Some(ctx.accounts.user.key());
    single_use_invite.used_at = Some(Clock::get()?.unix_timestamp);
    
    msg!("Single-use invite code used: {:?} by user: {} (created by operator: {})", 
         core::str::from_utf8(&invite_code).unwrap_or("INVALID"),
         ctx.accounts.user.key(),
         single_use_invite.creator);
    
    Ok(())
}
```

### 4. エラー定義追加

```rust
// error.rs に追加
#[error_code]
pub enum GameError {
    // ... 既存エラー
    
    #[msg("Invite code has already been used")]
    InviteCodeAlreadyUsed,
    
    #[msg("Only operator can create single-use invite codes")]
    OperatorOnlyFunction,
}
```

### 5. 利用シナリオ

#### 🎯 キャンペーン配布
```bash
# 運営が100個の一回限りコードを生成
CAMPAIGN01, CAMPAIGN02, ..., CAMPAIGN99

# SNS/Discord/Twitterで配布
"先着100名様限定！ CAMPAIGN01 でゲーム参加！"

# 誰か1人が使用 → コード無効化
# 他99個は引き続き有効
```

#### 🎁 イベント配布
```bash
# 特別イベント用コード
EVENT2024, LAUNCH01, BETA001, etc.

# 各コードは1回のみ有効
# 使用者は運営直接招待扱い（紹介料0%）
```

#### 📈 成長ハッキング
```bash
# インフルエンサー専用コード
STREAMER1, YOUTUBE1, TIKTOK01

# 使用状況の追跡可能
# 効果測定とマーケティング最適化
```

### 6. 管理機能

```rust
/// 一回限り招待コード状況確認（読み取り専用）
pub fn get_single_use_invite_status(
    ctx: Context<GetSingleUseInviteStatus>,
    invite_code: [u8; 8]
) -> Result<SingleUseInviteStatus> {
    let invite_account = &ctx.accounts.single_use_invite_account;
    
    Ok(SingleUseInviteStatus {
        code: invite_account.code,
        is_used: invite_account.is_used,
        used_by: invite_account.used_by,
        created_at: invite_account.created_at,
        used_at: invite_account.used_at,
        creator: invite_account.creator,
    })
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SingleUseInviteStatus {
    pub code: [u8; 8],
    pub is_used: bool,
    pub used_by: Option<Pubkey>,
    pub created_at: i64,
    pub used_at: Option<i64>,
    pub creator: Pubkey,
}
```

### 7. 運営ダッシュボード機能

```typescript
// フロントエンド管理機能
class SingleUseInviteManager {
    async createBatchCodes(prefix: string, count: number) {
        // "EVENT" + 01~99 の形式で大量生成
        const codes = [];
        for (let i = 1; i <= count; i++) {
            const code = `${prefix}${i.toString().padStart(2, '0')}`;
            codes.push(code);
        }
        
        // バッチ作成処理
        return await this.batchCreateCodes(codes);
    }
    
    async getUsageStatistics() {
        // 使用率、残り数、効果的なコードの分析
        return {
            totalCreated: 100,
            totalUsed: 67,
            remaining: 33,
            usageRate: 67,
            topPerformingCodes: ["EVENT01", "LAUNCH1"]
        };
    }
}
```

## 🎯 メリット

1. **運営完全制御**: コード生成は運営のみ
2. **公平性**: 誰でも使用可能（早い者勝ち）
3. **一回限り**: 確実な使用制限
4. **追跡可能**: 使用者と使用時刻を記録
5. **紹介料免除**: 運営招待扱いでユーザー負担なし
6. **マーケティング活用**: キャンペーン・イベント対応

## ⚡ 実装優先度

**Phase 1**: 基本機能（作成・使用）
**Phase 2**: バッチ生成機能
**Phase 3**: 統計・分析ダッシュボード

この実装により、運営主導のマーケティング戦略と公平なユーザー獲得を両立できます。