# ハッシュシステム招待移行計画

## 🧠 Hard Think: 完全設計

### 🎯 核心設計原則

1. **完全秘匿性**: オンチェーンで平文コード非表示
2. **Zero Knowledge**: 招待者アドレスのみ公開、コード内容は秘匿
3. **効率的検索**: O(n) 複雑度での招待者コード検索
4. **UX最適化**: ユーザーは平文コード + 招待者アドレスのみ入力

### 🏗️ アーキテクチャ設計

#### 状態構造
```rust
/// 秘匿招待コード（ハッシュベース）
#[account]
pub struct SecretInviteCode {
    pub inviter: Pubkey,                    // 招待者
    pub code_hash: [u8; 32],               // SHA256(code + salt + inviter)  
    pub salt: [u8; 16],                    // ランダムソルト
    pub invites_used: u8,                  // 使用回数
    pub invite_limit: u8,                  // 上限
    pub code_index: u16,                   // 招待者内でのコード番号
    pub created_at: i64,                   // 作成時刻
    pub is_active: bool,                   // アクティブ状態
    pub reserve: [u8; 15],                 // 予約領域
}

/// 招待者のコード管理
#[account]
pub struct InviterCodeRegistry {
    pub inviter: Pubkey,                   // 招待者
    pub total_codes_created: u16,          // 作成済みコード数
    pub active_codes_count: u16,           // アクティブコード数
    pub total_invites_used: u32,           // 総使用回数
    pub last_code_created_at: i64,         // 最終作成時刻
    pub reserve: [u8; 32],                 // 予約領域
}

/// 一回限り秘匿招待（運営専用）
#[account]
pub struct SingleUseSecretInvite {
    pub creator: Pubkey,                   // 運営のみ
    pub code_hash: [u8; 32],              // SHA256(code + salt + creator)
    pub salt: [u8; 16],                   // ランダムソルト
    pub is_used: bool,                    // 使用済み
    pub used_by: Option<Pubkey>,          // 使用者
    pub created_at: i64,                  // 作成時刻
    pub used_at: Option<i64>,             // 使用時刻
    pub campaign_id: [u8; 8],             // キャンペーンID（オプション）
    pub reserve: [u8; 16],                // 予約領域
}
```

#### PDA設計
```rust
// 通常招待コード
secret_invite_pda = [b"secret_invite", inviter.key(), code_index.to_le_bytes()]

// 招待者レジストリ  
inviter_registry_pda = [b"inviter_registry", inviter.key()]

// 一回限り招待（ハッシュベース）
single_use_pda = [b"single_use_secret", code_hash[0..8]]
```

### 🔧 実装詳細

#### 1. ハッシュ生成関数
```rust
pub fn generate_code_hash(
    plaintext_code: &[u8; 8],
    salt: &[u8; 16], 
    inviter: &Pubkey
) -> [u8; 32] {
    use solana_program::hash::{hash, Hash};
    
    let mut data = Vec::new();
    data.extend_from_slice(plaintext_code);
    data.extend_from_slice(salt);
    data.extend_from_slice(inviter.as_ref());
    
    hash(&data).to_bytes()
}

pub fn generate_random_salt() -> [u8; 16] {
    use solana_program::clock::Clock;
    use solana_program::hash::hash;
    
    let clock = Clock::get().unwrap();
    let seed_data = [
        clock.unix_timestamp.to_le_bytes(),
        clock.slot.to_le_bytes(),
    ].concat();
    
    let hash_result = hash(&seed_data);
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&hash_result.to_bytes()[0..16]);
    salt
}
```

#### 2. 招待コード作成命令
```rust
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct CreateSecretInviteCode<'info> {
    #[account(
        init_if_needed,
        payer = inviter,
        space = InviterCodeRegistry::LEN,
        seeds = [b"inviter_registry", inviter.key().as_ref()],
        bump
    )]
    pub inviter_registry: Account<'info, InviterCodeRegistry>,
    
    #[account(
        init,
        payer = inviter,
        space = SecretInviteCode::LEN,
        seeds = [
            b"secret_invite", 
            inviter.key().as_ref(), 
            &inviter_registry.total_codes_created.to_le_bytes()
        ],
        bump
    )]
    pub secret_invite_account: Account<'info, SecretInviteCode>,
    
    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub inviter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_secret_invite_code(
    ctx: Context<CreateSecretInviteCode>,
    invite_code: [u8; 8]
) -> Result<()> {
    // 招待コード形式検証
    validate_invite_code_format(&invite_code)?;
    
    let inviter_registry = &mut ctx.accounts.inviter_registry;
    let secret_invite = &mut ctx.accounts.secret_invite_account;
    let config = &ctx.accounts.config;
    
    // 招待上限チェック
    let invite_limit = if ctx.accounts.inviter.key() == config.operator {
        u8::MAX // 運営は無制限
    } else {
        config.max_invite_limit
    };
    
    // レジストリ初期化（必要な場合）
    if inviter_registry.inviter == Pubkey::default() {
        inviter_registry.inviter = ctx.accounts.inviter.key();
        inviter_registry.total_codes_created = 0;
        inviter_registry.active_codes_count = 0;
        inviter_registry.total_invites_used = 0;
    }
    
    // ハッシュ生成
    let salt = generate_random_salt();
    let code_hash = generate_code_hash(
        &invite_code,
        &salt,
        &ctx.accounts.inviter.key()
    );
    
    // 秘匿招待コード設定
    secret_invite.inviter = ctx.accounts.inviter.key();
    secret_invite.code_hash = code_hash;
    secret_invite.salt = salt;
    secret_invite.invites_used = 0;
    secret_invite.invite_limit = invite_limit;
    secret_invite.code_index = inviter_registry.total_codes_created;
    secret_invite.created_at = Clock::get()?.unix_timestamp;
    secret_invite.is_active = true;
    
    // レジストリ更新
    inviter_registry.total_codes_created += 1;
    inviter_registry.active_codes_count += 1;
    inviter_registry.last_code_created_at = Clock::get()?.unix_timestamp;
    
    msg!("Secret invite code created: Hash={:?}, Index={}, Inviter={}", 
         &code_hash[0..8], 
         secret_invite.code_index,
         ctx.accounts.inviter.key());
    
    Ok(())
}
```

#### 3. 招待コード使用命令
```rust
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8], inviter_pubkey: Pubkey, code_index: u16)]
pub struct UseSecretInviteCode<'info> {
    #[account(
        mut,
        seeds = [b"secret_invite", inviter_pubkey.as_ref(), &code_index.to_le_bytes()],
        bump,
        constraint = secret_invite_account.is_active @ GameError::InviteCodeInactive,
        constraint = secret_invite_account.invites_used < secret_invite_account.invite_limit @ GameError::InviteCodeLimitReached
    )]
    pub secret_invite_account: Account<'info, SecretInviteCode>,
    
    #[account(
        mut,
        seeds = [b"inviter_registry", inviter_pubkey.as_ref()],
        bump
    )]
    pub inviter_registry: Account<'info, InviterCodeRegistry>,
    
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

pub fn use_secret_invite_code(
    ctx: Context<UseSecretInviteCode>,
    invite_code: [u8; 8],
    inviter_pubkey: Pubkey,
    code_index: u16
) -> Result<()> {
    let secret_invite = &mut ctx.accounts.secret_invite_account;
    let inviter_registry = &mut ctx.accounts.inviter_registry;
    
    // ハッシュ検証
    let computed_hash = generate_code_hash(
        &invite_code,
        &secret_invite.salt,
        &inviter_pubkey
    );
    
    require!(
        computed_hash == secret_invite.code_hash,
        GameError::InvalidInviteCode
    );
    
    // 招待者検証
    require!(
        secret_invite.inviter == inviter_pubkey,
        GameError::InvalidInviter
    );
    
    // ユーザー初期化
    let user_state = &mut ctx.accounts.user_state;
    user_state.owner = ctx.accounts.invitee.key();
    user_state.total_grow_power = 0;
    user_state.last_harvest_time = Clock::get()?.unix_timestamp;
    user_state.has_farm_space = false;
    user_state.referrer = Some(inviter_pubkey);
    user_state.pending_referral_rewards = 0;
    user_state.reserve = [0; 32];
    
    // 使用カウント更新
    secret_invite.invites_used += 1;
    inviter_registry.total_invites_used += 1;
    
    msg!("Secret invite code used: User={}, Inviter={}, CodeIndex={}", 
         ctx.accounts.invitee.key(),
         inviter_pubkey,
         code_index);
    
    Ok(())
}
```

#### 4. 一回限り秘匿招待（運営専用）
```rust
#[derive(Accounts)]
#[instruction(invite_code: [u8; 8])]
pub struct CreateSingleUseSecretInvite<'info> {
    #[account(
        init,
        payer = operator,
        space = SingleUseSecretInvite::LEN,
        seeds = [b"single_use_secret", &generate_code_hash(&invite_code, &generate_random_salt(), &operator.key())[0..8]],
        bump
    )]
    pub single_use_invite: Account<'info, SingleUseSecretInvite>,
    
    #[account(
        seeds = [b"config"],
        bump,
        constraint = config.operator == operator.key() @ GameError::Unauthorized
    )]
    pub config: Account<'info, Config>,
    
    #[account(mut)]
    pub operator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_single_use_secret_invite(
    ctx: Context<CreateSingleUseSecretInvite>,
    invite_code: [u8; 8],
    campaign_id: Option<[u8; 8]>
) -> Result<()> {
    validate_invite_code_format(&invite_code)?;
    
    let salt = generate_random_salt();
    let code_hash = generate_code_hash(
        &invite_code,
        &salt,
        &ctx.accounts.operator.key()
    );
    
    let single_use_invite = &mut ctx.accounts.single_use_invite;
    single_use_invite.creator = ctx.accounts.operator.key();
    single_use_invite.code_hash = code_hash;
    single_use_invite.salt = salt;
    single_use_invite.is_used = false;
    single_use_invite.used_by = None;
    single_use_invite.created_at = Clock::get()?.unix_timestamp;
    single_use_invite.used_at = None;
    single_use_invite.campaign_id = campaign_id.unwrap_or([0; 8]);
    
    msg!("Single-use secret invite created: Hash={:?}, Campaign={:?}", 
         &code_hash[0..8], 
         campaign_id);
    
    Ok(())
}
```

### 🎮 フロントエンド実装

#### コード検索アルゴリズム
```typescript
class SecretInviteCodeFinder {
    async findInviteCode(
        plaintext_code: string,
        inviter_pubkey: PublicKey
    ): Promise<{account: PublicKey, index: number} | null> {
        
        // 1. 招待者レジストリ取得
        const registryPDA = this.getInviterRegistryPDA(inviter_pubkey);
        const registry = await this.program.account.inviterCodeRegistry.fetch(registryPDA);
        
        // 2. 全コードアカウントを順次チェック
        for (let i = 0; i < registry.totalCodesCreated; i++) {
            const secretInvitePDA = this.getSecretInvitePDA(inviter_pubkey, i);
            
            try {
                const inviteAccount = await this.program.account.secretInviteCode.fetch(secretInvitePDA);
                
                // 3. ハッシュ検証
                if (this.verifyCodeHash(plaintext_code, inviteAccount.salt, inviter_pubkey, inviteAccount.codeHash)) {
                    return { account: secretInvitePDA, index: i };
                }
            } catch (e) {
                // アカウントが存在しない場合はスキップ
                continue;
            }
        }
        
        return null;
    }
    
    private verifyCodeHash(
        plaintext_code: string,
        salt: Uint8Array,
        inviter: PublicKey,
        stored_hash: Uint8Array
    ): boolean {
        const computed_hash = this.generateCodeHash(plaintext_code, salt, inviter);
        return this.arraysEqual(computed_hash, stored_hash);
    }
    
    private generateCodeHash(
        plaintext_code: string,
        salt: Uint8Array,
        inviter: PublicKey
    ): Uint8Array {
        const encoder = new TextEncoder();
        const data = new Uint8Array([
            ...encoder.encode(plaintext_code),
            ...salt,
            ...inviter.toBytes()
        ]);
        
        return crypto.subtle.digest('SHA-256', data);
    }
}
```

#### UX フロー
```typescript
// ユーザー入力
const inviteCode = "LAUNCH01";         // 平文コード
const inviterAddress = "ABC123...";    // 招待者アドレス

// 1. コード検索
const codeInfo = await finder.findInviteCode(inviteCode, new PublicKey(inviterAddress));
if (!codeInfo) {
    throw new Error("Invalid invite code or inviter");
}

// 2. 使用実行
await program.methods
    .useSecretInviteCode(
        Array.from(Buffer.from(inviteCode)), 
        new PublicKey(inviterAddress),
        codeInfo.index
    )
    .accounts({
        secretInviteAccount: codeInfo.account,
        // ... other accounts
    })
    .rpc();
```

### 🔄 移行戦略

#### Phase 1: 新システム並行実装
- 既存の平文システム維持
- 新しいハッシュシステム追加
- 両方のコード作成・使用をサポート

#### Phase 2: 段階的移行
- 新規コードはハッシュシステムのみ
- 既存の平文コードは使用可能
- ユーザーへの移行案内

#### Phase 3: 完全移行
- 平文システムの廃止
- 既存コードの無効化または移行
- ドキュメント・テスト更新

### 📊 実装優先度

**High Priority:**
1. 状態構造定義
2. 基本命令実装（作成・使用）
3. フロントエンド検索ロジック

**Medium Priority:**
1. 一回限り招待実装
2. 管理機能
3. 統計・分析機能

**Low Priority:**
1. 旧システムからの移行ツール
2. 最適化・パフォーマンス向上
3. 高度な分析機能

### 🎯 成功指標

1. **秘匿性**: オンチェーンで平文コード完全非表示
2. **実用性**: 既存UXと同等の使いやすさ
3. **効率性**: O(n) 検索で実用的なパフォーマンス
4. **安全性**: ハッシュ衝突・辞書攻撃の防止

この設計により、完全な秘匿性と実用性を両立できます。