# 初期設定ガイド

## 概要

Facility Gameを運用開始するために必要な初期設定項目と手順を説明します。これらの設定は、ゲームの経済システム、トークン発行、確率テーブル、農場レベルなど、システム全体の基盤となる重要な要素です。

## 必須初期設定項目

### 1. システムコンフィグ (`initialize_config`)

システム全体の基本設定を初期化します。

**パラメータ**:
```rust
pub fn initialize_config(
    base_rate: Option<u64>,           // 基本報酬レート（デフォルト: 100 WEED/秒）
    halving_interval: Option<i64>,    // 半減期間隔（デフォルト: 200秒）
    treasury: Pubkey,                 // 財務ウォレット（必須）
    protocol_referral_address: Option<Pubkey>, // プロトコル紹介報酬除外アドレス
)
```

**自動設定される値**:
| 項目 | 値 | 説明 |
|------|-----|------|
| `seed_pack_cost` | 300 WEED | ミステリーシードパックの価格 |
| `farm_space_cost_sol` | 0.5 SOL | 農場スペースの価格 |
| `max_invite_limit` | 5回 | 一般ユーザーの招待可能回数 |
| `trading_fee_percentage` | 2% | 取引手数料率 |
| `operator` | `43eUMnsf1QoFmE2ZkHxbXxZCAJht7pPpFFPUYicUYbjJ` | 運営者アドレス（無制限招待権限） |
| `seed_counter` | 0 | グローバルシードカウンター |
| `seed_pack_counter` | 0 | グローバルシードパックカウンター |
| `total_supply_minted` | 0 | 総発行量トラッカー |

### 2. 報酬ミント作成 (`create_reward_mint`)

$WEEDトークンをSPL Token 2022 + Transfer Fee Extensionで作成します。

**自動設定される値**:
| 項目 | 値 | 説明 |
|------|-----|------|
| Transfer Fee | 200 basis points (2%) | 転送時の手数料率 |
| Maximum Fee | 1,000 WEED | 一回の転送での最大手数料 |
| Decimals | 6 | トークンの小数点桁数 |
| Mint Authority | `mint_authority` PDA | ミント権限（プログラム制御） |
| Transfer Fee Config Authority | `mint_authority` PDA | 手数料設定変更権限 |
| Withdraw Withheld Authority | treasury | 徴収済み手数料の引き出し権限 |

### 3. グローバル統計 (`initialize_global_stats`)

ゲーム全体の統計情報を追跡するアカウントを初期化します。

**初期値**:
| 項目 | 初期値 | 説明 |
|------|--------|------|
| `total_grow_power` | 0 | 全プレイヤーの合計Grow Power |
| `total_farm_spaces` | 0 | 作成された農場の総数 |
| `total_supply` | 0 | 発行済みWEEDトークン総量 |
| `current_rewards_per_second` | base_rate | 現在の報酬レート |
| `last_update_time` | 初期化時刻 | 最終更新タイムスタンプ |

### 4. 確率テーブル (`initialize_probability_table`)

シードパックから出現するシードの確率分布を設定します。

**デフォルト設定（Table 1 - 6シード）**:
| シード | Grow Power | 確率 | 説明 |
|--------|------------|------|------|
| Seed1 (OG Kush) | 100 | 43.0% | 最も一般的な基本シード |
| Seed2 (Blue Dream) | 180 | 25.0% | 一般的なシード |
| Seed3 (Sour Diesel) | 420 | 14.0% | 中級シード |
| Seed4 (Girl Scout Cookies) | 720 | 9.0% | レアシード |
| Seed5 (Gorilla Glue) | 1000 | 6.0% | 超レアシード |
| Seed6 (Skywalker Kush) | 5000 | 3.0% | 伝説級シード |

**期待値**: 434 Grow Power/パック

### 5. 農場レベル設定 (`initialize_farm_level_config`)

農場のレベルシステムと自動アップグレード閾値を設定します。

**デフォルトレベル設定**:
| レベル | 容量 | 必要累計パック購入数 | アップグレードコスト |
|--------|------|---------------------|-------------------|
| 1 | 4スロット | 0 | 初期レベル |
| 2 | 6スロット | 30パック | 自動アップグレード |
| 3 | 10スロット | 100パック | 自動アップグレード |
| 4 | 16スロット | 300パック | 自動アップグレード |
| 5 | 25スロット | 500パック | 自動アップグレード |

### 6. 手数料プール (`initialize_fee_pool`)

取引手数料を蓄積するためのプールを初期化します。

**パラメータ**:
- `treasury_address`: 手数料引き出し先アドレス

## 初期設定手順

### 準備

```typescript
import { Keypair, PublicKey, Connection } from '@solana/web3.js';
import { Program, AnchorProvider } from '@coral-xyz/anchor';

// 1. 管理者ウォレット準備
const admin = Keypair.fromSecretKey(/* your admin secret key */);
const treasury = new PublicKey("YOUR_TREASURY_WALLET_ADDRESS");

// 2. 接続設定
const connection = new Connection("https://api.devnet.solana.com");
const provider = new AnchorProvider(connection, wallet, {});
const program = new Program(IDL, PROGRAM_ID, provider);
```

### 実行順序

初期設定は以下の順序で実行する必要があります：

#### ステップ1: システムコンフィグ
```typescript
await program.methods
  .initializeConfig(
    null,     // デフォルトbase_rate (100 WEED/秒)を使用
    null,     // デフォルトhalving_interval (200秒)を使用
    treasury, // 財務ウォレットアドレス
    null      // プロトコル紹介報酬除外アドレスなし
  )
  .accounts({ 
    admin: admin.publicKey,
    systemProgram: SystemProgram.programId
  })
  .signers([admin])
  .rpc();

console.log("✅ System config initialized");
```

#### ステップ2: 報酬ミント作成
```typescript
await program.methods
  .createRewardMint()
  .accounts({ 
    admin: admin.publicKey,
    withdrawWithheldAuthority: treasury,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY
  })
  .signers([admin])
  .rpc();

console.log("✅ WEED token mint created with 2% transfer fee");
```

#### ステップ3: グローバル統計
```typescript
await program.methods
  .initializeGlobalStats()
  .accounts({ 
    admin: admin.publicKey,
    systemProgram: SystemProgram.programId
  })
  .signers([admin])
  .rpc();

console.log("✅ Global statistics initialized");
```

#### ステップ4: 確率テーブル
```typescript
await program.methods
  .initializeProbabilityTable()
  .accounts({ 
    admin: admin.publicKey,
    systemProgram: SystemProgram.programId
  })
  .signers([admin])
  .rpc();

console.log("✅ Probability table initialized with 6 seed types");
```

#### ステップ5: 農場レベル設定
```typescript
await program.methods
  .initializeFarmLevelConfig()
  .accounts({ 
    admin: admin.publicKey,
    systemProgram: SystemProgram.programId
  })
  .signers([admin])
  .rpc();

console.log("✅ Farm level configuration initialized");
```

#### ステップ6: 手数料プール
```typescript
await program.methods
  .initializeFeePool(treasury)
  .accounts({ 
    admin: admin.publicKey,
    systemProgram: SystemProgram.programId
  })
  .signers([admin])
  .rpc();

console.log("✅ Fee pool initialized");
```

## 初期設定後の調整

### 確率テーブルを9シードに更新

より多様性のある9シード構成に更新する場合：

```typescript
await program.methods.updateProbabilityTable(
  2,    // version
  9,    // seed_count
  [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000], // grow_powers
  [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000], // thresholds
  [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56], // percentages
  1590, // expected_value
  "Enhanced9Seeds" // name
)
.accounts({ admin: admin.publicKey })
.signers([admin])
.rpc();
```

### 本番環境用の半減期設定

テスト用の200秒から本番用の1年に変更：

```typescript
await program.methods.updateConfig(
  null,                    // operator変更なし
  null,                    // base_rate変更なし
  new BN(31_536_000),     // 1年 = 365日 × 24時間 × 3600秒
  null,                    // treasury変更なし
  null                     // max_invite_limit変更なし
)
.accounts({ admin: admin.publicKey })
.signers([admin])
.rpc();
```

### シードパック価格の調整

経済バランスに応じて価格を調整：

```typescript
await program.methods.updateSeedPackCost(
  new BN(500_000_000) // 500 WEED (6 decimals)
)
.accounts({ admin: admin.publicKey })
.signers([admin])
.rpc();
```

### オペレーターアドレスの変更

運営者権限（無制限招待）を別のアドレスに移譲：

```typescript
await program.methods.updateConfig(
  new PublicKey("NEW_OPERATOR_WALLET_ADDRESS"),
  null, null, null, null
)
.accounts({ admin: admin.publicKey })
.signers([admin])
.rpc();
```

## 環境別推奨設定

### 開発環境
```typescript
{
  base_rate: 1000,        // 10倍速で報酬生成
  halving_interval: 60,   // 1分で半減（テスト用）
  seed_pack_cost: 100,    // 安価なパック（テスト用）
}
```

### テストネット
```typescript
{
  base_rate: 100,         // 標準レート
  halving_interval: 3600, // 1時間で半減
  seed_pack_cost: 300,    // 標準価格
}
```

### メインネット
```typescript
{
  base_rate: 100,              // 標準レート
  halving_interval: 31536000,  // 1年で半減
  seed_pack_cost: 300,         // 標準価格
}
```

## セキュリティ考慮事項

### 権限管理
- **Admin権限**: `initialize_config`を実行したウォレットが永続的な管理者となります
- **Treasury権限**: Transfer Fee回収とSOL支払いの受取先。慎重に管理してください
- **Operator権限**: 無制限招待権限を持つため、信頼できるアドレスのみに付与

### 初期化の一意性
- 各初期化関数は**一度のみ**実行可能（PDAによる制約）
- 初期化後の変更は`update_*`系の関数を使用
- 重要な変更はマルチシグウォレットの使用を推奨

### 経済パラメータ
- `base_rate`と`halving_interval`は経済バランスに直結
- 本番環境では慎重な計算とシミュレーションを推奨
- Transfer Fee (2%)は変更不可のため、初期設定時に確認必須

## トラブルシューティング

### よくあるエラー

1. **"already in use"エラー**
   - 原因: 既に初期化済みのアカウント
   - 解決: 新しい環境でやり直すか、既存の設定を使用

2. **"Unauthorized"エラー**
   - 原因: Admin権限のないウォレットで実行
   - 解決: 正しいAdminウォレットを使用

3. **"insufficient funds"エラー**
   - 原因: SOL残高不足
   - 解決: 実行ウォレットに十分なSOLを送金

### 初期化状態の確認

```typescript
// Config状態の確認
const config = await program.account.config.fetch(configPDA);
console.log("Base rate:", config.baseRate.toNumber());
console.log("Treasury:", config.treasury.toString());

// Mint状態の確認
const mintInfo = await getMint(connection, rewardMintPDA, undefined, TOKEN_2022_PROGRAM_ID);
console.log("Decimals:", mintInfo.decimals);
console.log("Supply:", mintInfo.supply.toString());
```

## チェックリスト

初期設定を完了するための確認項目：

- [ ] 管理者ウォレットの準備とバックアップ
- [ ] 財務ウォレットアドレスの決定
- [ ] テスト環境での初期設定テスト実行
- [ ] `initialize_config`の実行
- [ ] `create_reward_mint`の実行
- [ ] `initialize_global_stats`の実行
- [ ] `initialize_probability_table`の実行
- [ ] `initialize_farm_level_config`の実行
- [ ] `initialize_fee_pool`の実行
- [ ] 必要に応じて確率テーブルを9シードに更新
- [ ] 本番環境用の半減期間隔に更新
- [ ] 初期設定の検証とログ記録
- [ ] 運用開始前の最終確認

これらの手順に従うことで、Facility Gameを適切に初期化し、安全に運用を開始できます。