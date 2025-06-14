# 招待システム実装方針

## 概要

Facility Gameの招待システムは、プライバシー保護と公正性を重視した設計になっています。ハッシュベースの招待コード、階層的紹介報酬、きめ細かな権限管理により、健全で持続可能なユーザー成長エコシステムを構築しています。

## 設計理念

### 1. プライバシー保護
- **ハッシュベース招待コード**: 平文コードはブロックチェーン上に保存されない
- **塩付きハッシュ**: レインボーテーブル攻撃を防御
- **ユーザー匿名性**: 招待関係の第三者による推測を困難にする

### 2. 公正な報酬分配
- **明確な階層制限**: 2レベルまでの紹介報酬で無限連鎖を防止
- **固定報酬率**: L1=10%, L2=5%の透明な分配
- **リアルタイム分配**: 報酬発生と同時に蓄積・配布

### 3. 制御されたユーザー成長
- **段階的権限付与**: 一般ユーザー→オペレーター→管理者
- **使用制限管理**: スパム防止と品質維持
- **監査可能性**: 全ての招待活動を追跡可能

## アーキテクチャ設計

### アカウント構造

#### InviteCode（招待コードアカウント）
```rust
#[account]
pub struct InviteCode {
    /// 招待者のアドレス
    pub inviter: Pubkey,
    
    /// 現在の使用回数
    pub invites_used: u16,
    
    /// この招待者の使用上限
    pub invite_limit: u16,
    
    /// 招待コードのSHA256ハッシュ
    pub code_hash: [u8; 32],
    
    /// ハッシュ計算用のランダムソルト
    pub salt: [u8; 16],
    
    /// 招待者の招待コード内でのインデックス
    pub code_index: u16,
    
    /// 作成タイムスタンプ
    pub created_at: i64,
    
    /// アクティブ状態フラグ
    pub is_active: bool,
    
    /// オペレーター権限で作成されたかのフラグ
    pub created_as_operator: bool,
    
    /// 将来の拡張用
    pub reserve: [u8; 14],
}
```

#### InviterCodeRegistry（招待者管理アカウント）
```rust
#[account]
pub struct InviterCodeRegistry {
    /// 招待者のアドレス
    pub inviter: Pubkey,
    
    /// 総作成コード数
    pub total_codes_created: u16,
    
    /// 現在アクティブなコード数
    pub active_codes_count: u16,
    
    /// 全コード合計使用回数
    pub total_invites_used: u32,
    
    /// 最後にコード作成した時刻
    pub last_code_created_at: i64,
    
    /// 将来の拡張用
    pub reserve: [u8; 32],
}
```

#### SingleUseSecretInvite（特別招待コード）
```rust
#[account]
pub struct SingleUseSecretInvite {
    /// 作成者（オペレーターのみ）
    pub creator: Pubkey,
    
    /// コードハッシュ
    pub code_hash: [u8; 32],
    
    /// ソルト
    pub salt: [u8; 16],
    
    /// 使用済みフラグ
    pub is_used: bool,
    
    /// 使用者
    pub used_by: Option<Pubkey>,
    
    /// 作成・使用タイムスタンプ
    pub created_at: i64,
    pub used_at: Option<i64>,
    
    /// キャンペーンID
    pub campaign_id: [u8; 8],
    
    /// 将来の拡張用
    pub reserve: [u8; 16],
}
```

### PDA設計

#### セキュアなアドレス導出
```rust
// 招待コードPDA
pub fn derive_invite_code_pda(code_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"invite_code", code_hash],
        program_id
    )
}

// 招待者レジストリPDA
pub fn derive_inviter_registry_pda(inviter: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"inviter_registry", inviter.as_ref()],
        program_id
    )
}

// 特別招待コードPDA
pub fn derive_secret_invite_pda(code_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"secret_invite", code_hash],
        program_id
    )
}
```

## 招待コード生成とセキュリティ

### ハッシュベース暗号化

```rust
use sha2::{Sha256, Digest};
use rand::{Rng, thread_rng};

pub fn generate_secure_invite_code() -> ([u8; 8], [u8; 32], [u8; 16]) {
    let mut rng = thread_rng();
    
    // 8バイトの招待コード生成
    let mut invite_code = [0u8; 8];
    rng.fill(&mut invite_code);
    
    // 16バイトのランダムソルト生成
    let mut salt = [0u8; 16];
    rng.fill(&mut salt);
    
    // SHA256ハッシュ計算
    let mut hasher = Sha256::new();
    hasher.update(&invite_code);
    hasher.update(&salt);
    let hash = hasher.finalize();
    
    let mut code_hash = [0u8; 32];
    code_hash.copy_from_slice(&hash);
    
    (invite_code, code_hash, salt)
}

pub fn verify_invite_code(
    provided_code: &[u8; 8],
    stored_hash: &[u8; 32],
    salt: &[u8; 16]
) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(provided_code);
    hasher.update(salt);
    let computed_hash = hasher.finalize();
    
    computed_hash.as_slice() == stored_hash
}
```

### 招待コード作成処理

```rust
pub fn create_invite_code(
    ctx: Context<CreateInviteCode>,
    invite_code: [u8; 8]
) -> Result<()> {
    let inviter = &ctx.accounts.inviter;
    let config = &ctx.accounts.config;
    let invite_code_account = &mut ctx.accounts.invite_code_account;
    let inviter_registry = &mut ctx.accounts.inviter_registry;
    
    // 権限チェック
    let invite_limit = if inviter.key() == config.operator {
        1024  // オペレーター: 1024回
    } else {
        config.max_invite_limit as u16  // 一般ユーザー: 5回
    };
    
    // 使用制限チェック
    require!(
        inviter_registry.total_invites_used < invite_limit as u32,
        GameError::InviteCodeLimitReached
    );
    
    // ソルト生成とハッシュ計算
    let salt = generate_random_salt();
    let code_hash = calculate_hash(&invite_code, &salt);
    
    // アカウント初期化
    invite_code_account.inviter = inviter.key();
    invite_code_account.invites_used = 0;
    invite_code_account.invite_limit = invite_limit;
    invite_code_account.code_hash = code_hash;
    invite_code_account.salt = salt;
    invite_code_account.code_index = inviter_registry.total_codes_created;
    invite_code_account.created_at = Clock::get()?.unix_timestamp;
    invite_code_account.is_active = true;
    invite_code_account.created_as_operator = inviter.key() == config.operator;
    
    // レジストリ更新
    inviter_registry.total_codes_created += 1;
    inviter_registry.active_codes_count += 1;
    inviter_registry.last_code_created_at = invite_code_account.created_at;
    
    msg!("Invite code created for inviter: {}", inviter.key());
    
    Ok(())
}
```

## 招待コード使用システム

### 検証と使用処理

```rust
pub fn use_invite_code(
    ctx: Context<UseInviteCode>,
    invite_code: [u8; 8],
    inviter_pubkey: Pubkey
) -> Result<()> {
    let invite_code_account = &mut ctx.accounts.invite_code_account;
    let invitee = &ctx.accounts.invitee;
    let user_state = &mut ctx.accounts.user_state;
    
    // ハッシュ検証
    require!(
        verify_invite_code(&invite_code, &invite_code_account.code_hash, &invite_code_account.salt),
        GameError::HashVerificationFailed
    );
    
    // アクティブ状態チェック
    require!(invite_code_account.is_active, GameError::InviteCodeInactive);
    
    // 招待者一致チェック
    require!(
        invite_code_account.inviter == inviter_pubkey,
        GameError::InviterMismatch
    );
    
    // 使用制限チェック
    require!(
        invite_code_account.invites_used < invite_code_account.invite_limit,
        GameError::InviteCodeLimitReached
    );
    
    // ユーザーアカウント初期化
    user_state.owner = invitee.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = Some(inviter_pubkey);
    user_state.pending_referral_rewards = 0;
    user_state.total_packs_purchased = 0;
    
    // 使用回数増加
    invite_code_account.invites_used += 1;
    
    // レジストリ更新
    let inviter_registry = &mut ctx.accounts.inviter_registry;
    inviter_registry.total_invites_used += 1;
    
    msg!("User {} invited by {} using invite code", invitee.key(), inviter_pubkey);
    
    Ok(())
}
```

## 階層的紹介報酬システム

### 報酬分配メカニズム

```rust
pub fn distribute_referral_rewards(
    ctx: Context<DistributeReferralRewards>,
    base_reward_amount: u64
) -> Result<()> {
    let user_state = &ctx.accounts.user_state;
    let config = &ctx.accounts.config;
    
    // L1紹介者への報酬（10%）
    if let Some(l1_referrer) = user_state.referrer {
        if l1_referrer != config.protocol_referral_address {
            let l1_reward = base_reward_amount * 10 / 100;
            accumulate_referral_reward(
                ctx.remaining_accounts,
                l1_referrer,
                l1_reward,
                1  // Level 1
            )?;
            
            // L2紹介者への報酬（5%）
            if let Some(l1_user_state) = get_user_state_if_exists(ctx.remaining_accounts, l1_referrer) {
                if let Some(l2_referrer) = l1_user_state.referrer {
                    if l2_referrer != config.protocol_referral_address {
                        let l2_reward = base_reward_amount * 5 / 100;
                        accumulate_referral_reward(
                            ctx.remaining_accounts,
                            l2_referrer,
                            l2_reward,
                            2  // Level 2
                        )?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn accumulate_referral_reward(
    remaining_accounts: &[AccountInfo],
    referrer: Pubkey,
    reward_amount: u64,
    level: u8
) -> Result<()> {
    // 紹介者のUserStateアカウントを探索
    for account_info in remaining_accounts {
        if account_info.key() == referrer {
            let mut account_data = account_info.try_borrow_mut_data()?;
            let user_state = UserState::try_from_slice(&account_data[8..])?;
            
            // 紹介報酬蓄積
            let mut updated_user_state = user_state;
            updated_user_state.pending_referral_rewards += reward_amount;
            
            // アカウントデータ更新
            let updated_data = updated_user_state.try_to_vec()?;
            account_data[8..].copy_from_slice(&updated_data);
            
            msg!("Accumulated {} WEED referral reward for {} (Level {})", 
                 reward_amount, referrer, level);
            
            return Ok(());
        }
    }
    
    Err(GameError::ReferrerAccountNotFound.into())
}
```

### 統合報酬請求

```rust
pub fn claim_reward_with_referral_rewards(
    ctx: Context<ClaimRewardWithReferralRewards>
) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let farm_space = &ctx.accounts.farm_space;
    let config = &mut ctx.accounts.config;
    
    // 基本農場報酬計算
    let farm_reward = calculate_farm_reward(user_state, farm_space, config)?;
    
    // 蓄積された紹介報酬取得
    let referral_reward = user_state.pending_referral_rewards;
    user_state.pending_referral_rewards = 0;
    
    // 総報酬額
    let total_reward = farm_reward + referral_reward;
    
    // 新規紹介報酬分配（このユーザーの報酬から）
    distribute_referral_rewards(ctx, farm_reward)?;
    
    // トークンミント
    if total_reward > 0 {
        mint_reward_tokens(&ctx.accounts, total_reward)?;
        
        // 総供給量更新
        config.total_supply_minted += total_reward;
    }
    
    // タイムスタンプ更新
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    
    msg!("Claimed {} WEED (Farm: {}, Referral: {})", 
         total_reward, farm_reward, referral_reward);
    
    Ok(())
}
```

## 特別招待システム

### オペレーター専用機能

```rust
pub fn create_single_use_secret_invite(
    ctx: Context<CreateSingleUseSecretInvite>,
    invite_code: [u8; 8],
    campaign_id: [u8; 8]
) -> Result<()> {
    let creator = &ctx.accounts.creator;
    let config = &ctx.accounts.config;
    let secret_invite = &mut ctx.accounts.secret_invite;
    
    // オペレーター権限チェック
    require!(
        creator.key() == config.operator || creator.key() == config.admin,
        GameError::UnauthorizedOperator
    );
    
    // ソルト生成とハッシュ計算
    let salt = generate_random_salt();
    let code_hash = calculate_hash(&invite_code, &salt);
    
    // アカウント初期化
    secret_invite.creator = creator.key();
    secret_invite.code_hash = code_hash;
    secret_invite.salt = salt;
    secret_invite.is_used = false;
    secret_invite.used_by = None;
    secret_invite.created_at = Clock::get()?.unix_timestamp;
    secret_invite.used_at = None;
    secret_invite.campaign_id = campaign_id;
    
    msg!("Secret invite created for campaign: {:?}", campaign_id);
    
    Ok(())
}
```

### バッチ招待コード生成

```rust
pub fn batch_create_secret_invites(
    ctx: Context<BatchCreateSecretInvites>,
    campaign_id: [u8; 8],
    quantity: u8
) -> Result<Vec<[u8; 8]>> {
    require!(quantity <= 100, GameError::BatchSizeExceeded);
    
    let mut generated_codes = Vec::new();
    
    for i in 0..quantity {
        let (invite_code, _, _) = generate_secure_invite_code();
        
        // 個別の秘密招待コード作成処理
        create_individual_secret_invite(
            &ctx.accounts,
            invite_code,
            campaign_id,
            i as u16
        )?;
        
        generated_codes.push(invite_code);
    }
    
    msg!("Batch created {} secret invites for campaign: {:?}", quantity, campaign_id);
    
    Ok(generated_codes)
}
```

## 監査と分析

### 招待活動追跡

```rust
#[account]
pub struct InviteAnalytics {
    pub total_invites_created: u64,
    pub total_invites_used: u64,
    pub unique_inviters: u32,
    pub average_invites_per_user: f64,
    pub top_inviters: Vec<TopInviter>,
    pub daily_invite_stats: Vec<DailyStats>,
    pub conversion_rates: ConversionRates,
    pub last_updated: i64,
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TopInviter {
    pub inviter: Pubkey,
    pub total_invites: u32,
    pub successful_invites: u32,
    pub total_referral_rewards: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConversionRates {
    pub invite_to_registration: f32,     // 招待→登録率
    pub registration_to_purchase: f32,   // 登録→農場購入率
    pub purchase_to_active: f32,         // 購入→アクティブ率
}
```

### リアルタイム分析

```typescript
// フロントエンド分析ダッシュボード
class InviteAnalytics {
  async generateInviteReport(timeframe: 'daily' | 'weekly' | 'monthly'): Promise<InviteReport> {
    const analytics = await this.fetchInviteAnalytics();
    const recentInvites = await this.fetchRecentInvites(timeframe);
    
    return {
      overview: {
        totalInvites: analytics.totalInvitesCreated,
        successfulInvites: analytics.totalInvitesUsed,
        conversionRate: analytics.totalInvitesUsed / analytics.totalInvitesCreated,
        uniqueInviters: analytics.uniqueInviters,
      },
      
      performance: {
        topInviters: analytics.topInviters.slice(0, 10),
        averageInvitesPerUser: analytics.averageInvitesPerUser,
        conversionFunnel: analytics.conversionRates,
      },
      
      trends: {
        dailyGrowth: this.calculateGrowthRate(analytics.dailyInviteStats),
        seasonality: this.analyzeSeasonality(analytics.dailyInviteStats),
        predictions: this.generatePredictions(analytics.dailyInviteStats),
      },
      
      recommendations: this.generateOptimizationRecommendations(analytics)
    };
  }
  
  private generateOptimizationRecommendations(analytics: InviteAnalytics): string[] {
    const recommendations: string[] = [];
    
    if (analytics.conversionRates.inviteToRegistration < 0.3) {
      recommendations.push("招待から登録への転換率が低いです。招待インセンティブの見直しを検討してください。");
    }
    
    if (analytics.conversionRates.registrationToPurchase < 0.5) {
      recommendations.push("登録から農場購入への転換率が低いです。初心者チュートリアルの改善を推奨します。");
    }
    
    if (analytics.averageInvitesPerUser < 2) {
      recommendations.push("平均招待数が少ないです。招待報酬の拡充を検討してください。");
    }
    
    return recommendations;
  }
}
```

## セキュリティ対策

### スパム防止

```rust
#[account]
pub struct InviteRateLimit {
    pub user: Pubkey,
    pub last_invite_time: i64,
    pub invites_in_window: u8,
    pub window_start: i64,
    pub is_flagged: bool,
    pub reserve: [u8; 32],
}

pub fn check_rate_limit(
    ctx: Context<CheckRateLimit>
) -> Result<bool> {
    let rate_limit = &mut ctx.accounts.rate_limit;
    let current_time = Clock::get()?.unix_timestamp;
    
    // 24時間ウィンドウのリセット
    if current_time - rate_limit.window_start > 24 * 60 * 60 {
        rate_limit.window_start = current_time;
        rate_limit.invites_in_window = 0;
    }
    
    // レート制限チェック（24時間で最大10回）
    if rate_limit.invites_in_window >= 10 {
        rate_limit.is_flagged = true;
        return Err(GameError::RateLimitExceeded.into());
    }
    
    // 連続投稿チェック（最低5分間隔）
    if current_time - rate_limit.last_invite_time < 5 * 60 {
        return Err(GameError::TooFrequentInvites.into());
    }
    
    // 記録更新
    rate_limit.last_invite_time = current_time;
    rate_limit.invites_in_window += 1;
    
    Ok(true)
}
```

### 不正使用検出

```rust
pub fn detect_suspicious_activity(
    invite_pattern: &InvitePattern
) -> SuspiciousActivityReport {
    let mut flags = Vec::new();
    
    // 短時間での大量招待
    if invite_pattern.invites_per_hour > 50 {
        flags.push(SuspiciousFlag::RapidInviting);
    }
    
    // 同一IPからの複数アカウント
    if invite_pattern.accounts_per_ip > 5 {
        flags.push(SuspiciousFlag::MultipleAccountsPerIP);
    }
    
    // 異常に高い転換率
    if invite_pattern.conversion_rate > 0.95 {
        flags.push(SuspiciousFlag::UnusuallyHighConversion);
    }
    
    // 循環招待パターン
    if invite_pattern.has_circular_references {
        flags.push(SuspiciousFlag::CircularInvites);
    }
    
    SuspiciousActivityReport {
        risk_level: calculate_risk_level(&flags),
        flags,
        recommended_actions: generate_mitigation_actions(&flags),
        requires_manual_review: flags.len() > 2,
    }
}
```

## 運用管理

### 管理者ダッシュボード

```typescript
class InviteManagement {
  async getInviteOverview(): Promise<InviteOverview> {
    return {
      systemHealth: await this.assessSystemHealth(),
      activeInviters: await this.getActiveInviters(),
      recentActivity: await this.getRecentActivity(),
      suspiciousActivity: await this.getSuspiciousActivity(),
      systemConfiguration: await this.getSystemConfiguration(),
    };
  }
  
  async moderateInviteCode(codeHash: string, action: 'disable' | 'enable' | 'flag'): Promise<void> {
    switch (action) {
      case 'disable':
        await this.program.methods.disableInviteCode().accounts({ codeHash }).rpc();
        break;
      case 'enable':
        await this.program.methods.enableInviteCode().accounts({ codeHash }).rpc();
        break;
      case 'flag':
        await this.program.methods.flagInviteCode().accounts({ codeHash }).rpc();
        break;
    }
  }
  
  async adjustInviteLimits(userType: 'general' | 'operator', newLimit: number): Promise<void> {
    await this.program.methods
      .updateInviteLimits(userType, newLimit)
      .accounts({ admin: this.adminWallet.publicKey })
      .rpc();
  }
}
```

この招待システムにより、Facility Gameは安全で公正、かつ効果的なユーザー成長メカニズムを実現し、健全なコミュニティエコシステムを構築できます。