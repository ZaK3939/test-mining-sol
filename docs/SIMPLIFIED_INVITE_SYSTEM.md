# 簡略化招待システム技術仕様

## 概要

固定ソルト + ハッシュベースPDAによる簡略化招待システム。プライバシー保護を維持しながら実装を大幅に簡素化。

## 技術仕様

### PDAシード構造
```rust
seeds = [
    b"invite_hash",
    inviter.key().as_ref(),
    &generate_invite_code_hash(&invite_code, &get_fixed_salt(), &inviter.key())
]
```

### 固定ソルト
```rust
pub fn get_fixed_salt() -> [u8; 16] {
    [
        0x46, 0x41, 0x43, 0x49, 0x4c, 0x49, 0x54, 0x59,  // "FACILITY"
        0x47, 0x41, 0x4d, 0x45, 0x32, 0x30, 0x32, 0x34,  // "GAME2024"
    ]
}
```

### ハッシュ生成
```rust
pub fn generate_invite_code_hash(
    plaintext_code: &[u8; 8],
    salt: &[u8; 16],
    inviter: &Pubkey
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(plaintext_code);
    data.extend_from_slice(salt);
    data.extend_from_slice(inviter.as_ref());
    hash(&data).to_bytes()
}
```

## フロントエンド実装

### PDA計算
```typescript
import { createHash } from 'crypto';

export function deriveInviteCodePDA(
  inviteCode: string,
  inviterPubkey: PublicKey,
  programId: PublicKey
): [PublicKey, number] {
  const hash = generateInviteCodeHash(inviteCode, inviterPubkey);
  
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("invite_hash"),
      inviterPubkey.toBuffer(),
      Buffer.from(hash)
    ],
    programId
  );
}

export function generateInviteCodeHash(
  inviteCode: string,
  inviterPubkey: PublicKey
): Uint8Array {
  const fixedSalt = new Uint8Array([
    0x46, 0x41, 0x43, 0x49, 0x4c, 0x49, 0x54, 0x59,  // "FACILITY"
    0x47, 0x41, 0x4d, 0x45, 0x32, 0x30, 0x32, 0x34,  // "GAME2024"
  ]);
  
  const codeBytes = new TextEncoder().encode(inviteCode);
  const hasher = createHash('sha256');
  hasher.update(codeBytes);
  hasher.update(fixedSalt);
  hasher.update(inviterPubkey.toBytes());
  
  return new Uint8Array(hasher.digest());
}
```

## 使用方法

### 招待コード作成
```typescript
const codeBytes = Array.from(new TextEncoder().encode(inviteCode));
const codeArray = new Array(8).fill(0);
for (let i = 0; i < Math.min(8, codeBytes.length); i++) {
  codeArray[i] = codeBytes[i];
}

const [inviteAccountPDA] = deriveInviteCodePDA(inviteCode, wallet.publicKey, program.programId);

await program.methods
  .createInviteCode(codeArray)
  .accounts({
    inviteAccount: inviteAccountPDA,
    config: configPDA,
    inviter: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### 招待コード使用
```typescript
const [inviteAccountPDA] = deriveInviteCodePDA(inviteCode, inviterPubkey, program.programId);

await program.methods
  .useInviteCode(codeArray, inviterPubkey)
  .accounts({
    inviteAccount: inviteAccountPDA,
    userState: userStatePDA,
    config: configPDA,
    invitee: wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

## セキュリティ

- **プライバシー保護**: 招待コードは平文でチェーン上に保存されない
- **ハッシュ検証**: use_invite_code時にハッシュで検証
- **固定ソルト**: セキュリティに影響なし、PDA計算を予測可能にする
- **一意性**: 招待者アドレス + ハッシュ値で重複防止