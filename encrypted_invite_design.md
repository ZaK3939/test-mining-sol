# 暗号化招待コード実装設計

## 🔐 Option 2: エンクリプション方式詳細

### 基本フロー

1. **招待コード生成**
   ```
   招待者 → 8文字コード生成 → 招待者秘密鍵で暗号化 → オンチェーン保存
   ```

2. **招待コード共有**
   ```
   招待者 → 平文コードをオフチェーンで被招待者に送信（DM/QR等）
   ```

3. **招待コード使用**
   ```
   被招待者 → 平文コード入力 → 暗号化済みと照合 → ユーザー初期化
   ```

### 技術実装

#### 暗号化アルゴリズム
```rust
// ChaCha20-Poly1305 または AES-256-GCM
// Solana標準ライブラリ対応

use solana_program::secp256k1::{SecretKey, PublicKey};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

pub struct EncryptedInviteCode {
    pub inviter: Pubkey,                    // 招待者公開鍵
    pub encrypted_code: [u8; 32],           // 暗号化された招待コード
    pub nonce: [u8; 12],                    // 暗号化ノンス
    pub invites_used: u8,                   // 使用回数
    pub invite_limit: u8,                   // 上限
    pub created_at: i64,                    // 作成時刻
    pub reserve: [u8; 16],                  // 予約領域
}
```

#### 暗号化処理
```rust
pub fn create_encrypted_invite_code(
    ctx: Context<CreateEncryptedInviteCode>,
    plaintext_code: [u8; 8],
    recipient_pubkey: Pubkey,  // 被招待者の公開鍵
) -> Result<()> {
    let inviter_secret = &ctx.accounts.inviter_secret_key;
    
    // ECDH で共有秘密を生成
    let shared_secret = ecdh(inviter_secret, &recipient_pubkey)?;
    
    // ChaCha20-Poly1305 で暗号化
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&shared_secret[..32]));
    let nonce = Nonce::from_slice(&generate_nonce());
    
    let encrypted_code = cipher.encrypt(nonce, plaintext_code.as_ref())
        .map_err(|_| GameError::EncryptionFailed)?;
    
    // アカウントに保存
    let invite_account = &mut ctx.accounts.encrypted_invite_account;
    invite_account.inviter = ctx.accounts.inviter.key();
    invite_account.encrypted_code.copy_from_slice(&encrypted_code[..32]);
    invite_account.nonce.copy_from_slice(nonce.as_slice());
    invite_account.invites_used = 0;
    invite_account.invite_limit = ctx.accounts.config.max_invite_limit;
    invite_account.created_at = Clock::get()?.unix_timestamp;
    
    Ok(())
}
```

#### 復号・使用処理
```rust
pub fn use_encrypted_invite_code(
    ctx: Context<UseEncryptedInviteCode>,
    plaintext_code: [u8; 8],
) -> Result<()> {
    let invite_account = &ctx.accounts.encrypted_invite_account;
    
    // 被招待者の秘密鍵で復号
    let invitee_secret = &ctx.accounts.invitee_secret_key;
    let shared_secret = ecdh(invitee_secret, &invite_account.inviter)?;
    
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&shared_secret[..32]));
    let nonce = Nonce::from_slice(&invite_account.nonce);
    
    let decrypted_code = cipher.decrypt(nonce, invite_account.encrypted_code.as_ref())
        .map_err(|_| GameError::DecryptionFailed)?;
    
    // 平文コードと照合
    require!(
        decrypted_code[..8] == plaintext_code,
        GameError::InvalidInviteCode
    );
    
    // 使用回数制限チェック
    require!(
        invite_account.invites_used < invite_account.invite_limit,
        GameError::InviteCodeLimitReached
    );
    
    // ユーザー初期化処理
    let user_state = &mut ctx.accounts.user_state;
    user_state.owner = ctx.accounts.invitee.key();
    user_state.referrer = Some(invite_account.inviter);
    // ... 他の初期化処理
    
    // 使用回数増加
    invite_account.invites_used += 1;
    
    Ok(())
}
```

### 🎯 メリット・デメリット

#### ✅ メリット
1. **完全な秘匿性**: オンチェーンで招待コードが読み取れない
2. **ターゲット招待**: 特定の人だけが使用可能
3. **盗用防止**: 暗号化により第三者の悪用を防止
4. **プライバシー保護**: 招待関係が外部から見えない

#### ❌ デメリット
1. **実装複雑性**: 暗号化ライブラリとECDH実装が必要
2. **Gas コスト増加**: 暗号化・復号処理でCompute Unit消費
3. **UX複雑化**: 被招待者の公開鍵が事前に必要
4. **ライブラリ依存**: Solana対応暗号化ライブラリが必要

### 🔧 技術的課題と解決策

#### 課題1: Solana Program内での暗号化
```rust
// 解決策: solana-program対応ライブラリ使用
use solana_program::secp256k1;
use curve25519_dalek; // Ed25519 ECDH
use chacha20poly1305; // 軽量暗号化
```

#### 課題2: 秘密鍵の扱い
```rust
// 解決策: 一時的なアカウントまたはPDA使用
#[account(
    init,
    payer = inviter,
    space = 8 + 32,  // discriminator + secret_key
    seeds = [b"temp_secret", inviter.key().as_ref()],
    bump
)]
pub temp_secret_account: Account<'info, TempSecret>,
```

#### 課題3: ガス効率化
```rust
// 解決策: バッチ処理と事前計算
pub fn batch_create_encrypted_invites(
    ctx: Context<BatchCreateEncryptedInvites>,
    invite_data: Vec<(Pubkey, [u8; 8])>,  // (recipient, code) pairs
) -> Result<()> {
    // 複数招待を1トランザクションで処理
}
```

### 🚀 改良版実装案

#### 簡素化アプローチ
```rust
// より簡単な実装: 対称暗号化
pub struct SimpleEncryptedInviteCode {
    pub inviter: Pubkey,
    pub encrypted_code: [u8; 16],      // AES-128で暗号化
    pub salt: [u8; 16],                // ランダムソルト
    pub invites_used: u8,
    pub invite_limit: u8,
}

// 暗号化キー = PBKDF2(招待者署名, salt)
fn derive_encryption_key(
    inviter_signature: &[u8],
    salt: &[u8]
) -> [u8; 16] {
    // PBKDF2実装
}
```

### 💡 実装推奨レベル

**Phase 1**: 現在の平文システム（完了済み）
**Phase 2**: 運営直接招待（完了済み）
**Phase 3**: 暗号化招待コード（将来実装）

### 🎮 ユーザー体験フロー

1. **招待者側**:
   ```
   "GAME1234" 入力 → 被招待者アドレス指定 → 暗号化保存 → 平文コードをDM送信
   ```

2. **被招待者側**:
   ```
   "GAME1234" 入力 → 自動復号・照合 → ユーザー初期化 → ゲーム開始
   ```

この実装により、招待コードの完全な秘匿性を実現できますが、複雑性とコストが増加します。現在の運営直接招待システムとの使い分けが重要です。