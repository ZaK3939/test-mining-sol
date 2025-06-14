# 命令セット詳細解説

## 概要

Facility Gameプログラムは5つのカテゴリに分類される30以上の命令を提供します。各命令は特定の役割を持ち、ゲームの経済とセキュリティを維持しながら豊富な機能を実現しています。

## 1. システム管理命令（Admin Instructions）

### initialize_config
**目的**: ゲーム全体の基本設定を初期化

**パラメータ**:
- `base_rate`: `Option<u64>` - 基本報酬レート（デフォルト: 100 WEED/秒）
- `halving_interval`: `Option<i64>` - 半減期間隔（デフォルト: 200秒）
- `treasury`: `Pubkey` - 手数料収集用ウォレットアドレス
- `protocol_referral_address`: `Option<Pubkey>` - プロトコル専用アドレス

**必要アカウント**:
- `config`: 作成されるConfigアカウント（PDA）
- `admin`: 管理者アカウント（署名必須）
- `system_program`: Solanaシステムプログラム

**実行制約**:
- 管理者署名必須
- システム全体で一度のみ実行可能
- Configアカウントが存在しない場合のみ

**使用例**:
```rust
// デフォルト設定での初期化
initialize_config(None, None, treasury_pubkey, None)

// カスタム設定での初期化
initialize_config(Some(200), Some(86400), treasury_pubkey, Some(protocol_addr))
```

---

### create_reward_mint
**目的**: WEEDトークンのミント作成（SPL Token 2022使用）

**機能**:
- 6桁精度（1,000,000 = 1 WEED）
- 2%転送手数料設定
- PDAによるミント権限管理
- Metaplexメタデータ作成

**必要アカウント**:
- `config`: Configアカウント
- `reward_mint`: 作成されるミントアカウント（PDA）
- `mint_authority`: ミント権限PDA
- `metadata`: Metaplexメタデータアカウント
- `admin`: 管理者アカウント

**設定値**:
- Decimals: 6
- Transfer Fee: 200 basis points (2%)
- Supply: 0（初期状態）
- Freeze Authority: None

---

### update_config
**目的**: システム設定の動的更新

**更新可能項目**:
- `operator`: オペレーターアドレス
- `base_rate`: 基本報酬レート
- `halving_interval`: 半減期間隔
- `treasury`: トレジャリーアドレス
- `max_invite_limit`: 招待上限数

**セキュリティ**:
- 管理者署名必須
- 各パラメータの妥当性検証
- 重要な変更はイベントログ出力

---

## 2. ユーザー管理命令

### init_user
**目的**: ユーザーアカウントの初期化

**パラメータ**:
- `referrer`: `Option<Pubkey>` - 紹介者のアドレス（招待コード使用時）

**作成されるアカウント**:
- `UserState`: ユーザーの基本情報
- `SeedStorage`: シード保管庫

**初期値**:
- `total_grow_power`: 0
- `last_harvest_time`: 現在時刻
- `has_farm_space`: false
- `pending_referral_rewards`: 0

**実行制約**:
- ユーザーあたり一度のみ
- 既存アカウント検証

---

## 3. 農場管理命令

### buy_farm_space
**目的**: 農場スペースの購入と初期セットアップ

**コスト**: 0.5 SOL

**処理フロー**:
1. SOL支払い検証
2. FarmSpaceアカウント作成
3. 初期Seed1の無料付与
4. グローバル統計更新

**作成される農場**:
- Level: 1
- Capacity: 4
- Seed Count: 1（初期シード）
- Total Grow Power: 100

**制約**:
- ユーザーあたり1つまで
- 十分なSOL残高必須

---

### initialize_farm_level_config
**目的**: 動的農場レベル設定の初期化

**デフォルト設定**:
```rust
capacities: [4, 6, 8, 10, 12]  // レベル1-5の容量
upgrade_thresholds: [0, 30, 100, 300, 500]  // 必要パック購入数
level_names: ["初心者農場", "成長農場", "発展農場", "上級農場", "マスター農場"]
```

**拡張性**:
- 最大20レベルまで対応
- 動的な容量・閾値設定
- 管理者による更新可能

---

## 4. シードシステム命令

### purchase_seed_pack
**目的**: ミステリーシードパックの購入（Switchboard VRF統合）

**パラメータ**:
- `quantity`: `u8` - 購入数量（1-100）
- `user_entropy_seed`: `u64` - ユーザー提供の乱数シード
- `max_vrf_fee`: `u64` - 最大VRF手数料（約0.002 SOL）

**コスト計算**:
```
総コスト = (300 WEED × 数量) + VRF手数料
```

**処理フロー**:
1. WEED残高検証
2. VRF手数料計算・検証
3. SeedPackアカウント作成
4. VRFリクエスト送信
5. 手数料分配（紹介報酬含む）

**作成されるパック**:
- 一意のpack_id
- VRFアカウント参照
- 開封待ち状態

---

### open_seed_pack
**目的**: シードパックの開封とシード生成

**パラメータ**:
- `quantity`: `u8` - 開封数量

**処理フロー**:
1. VRF結果取得・検証
2. ユーザー乱数シード組み合わせ
3. 確率テーブル参照
4. シード種類決定
5. Seedアカウント作成
6. SeedStorage更新

**ランダム性確保**:
```rust
final_random = hash(vrf_result + user_entropy + block_timestamp + pack_id)
seed_type = probability_table.determine_seed(final_random % 10000)
```

---

### plant_seed / remove_seed
**目的**: 農場でのシード植付・除去

**plant_seedパラメータ**:
- `seed_id`: `u64` - 植付するシードのID

**実行条件**:
- シードの所有権確認
- 農場容量内であること
- シードが未植付状態

**状態変更**:
- `seed.is_planted = true`
- `farm_space.seed_count += 1`
- `farm_space.total_grow_power += seed.grow_power`
- `user_state.total_grow_power += seed.grow_power`

**remove_seed**: 逆の処理でシードを除去

---

### discard_seed / batch_discard_seeds
**目的**: 不要シードの永久削除

**単体削除**:
- `seed_id`: `u64` - 削除するシードID

**一括削除**:
- `seed_ids`: `Vec<u64>` - 削除するシードIDリスト（最大100個）

**実行制約**:
- 植付中シードは削除不可
- 所有権確認必須
- SeedStorage更新

---

## 5. 報酬システム命令

### claim_reward_with_referral_rewards
**目的**: 統合報酬請求（農場報酬 + 紹介報酬）

**処理フロー**:
1. **半減期チェック**: `current_time >= next_halving_time`の場合、`base_rate /= 2`
2. **農場報酬計算**: `(経過時間 × user_gp × base_rate) / total_gp`
3. **紹介報酬蓄積**: 新規報酬の10%（L1）、5%（L2）を分配
4. **統合ミント**: 全報酬を一括でミント・配布
5. **統計更新**: 総供給量更新

**報酬分配の詳細**:
```rust
// 基本報酬
farm_reward = (elapsed_time * user_grow_power * base_rate) / total_grow_power

// 紹介報酬分配
if let Some(l1_referrer) = user.referrer {
    l1_reward = farm_reward * 10 / 100
    if let Some(l2_referrer) = l1_referrer.referrer {
        l2_reward = farm_reward * 5 / 100
    }
}

// 総受取額
total_reward = farm_reward + accumulated_referral_rewards
```

**制約**:
- Grow Power > 0必須
- 最大供給量チェック
- オーバーフロー防止

---

## 6. 招待システム命令

### create_invite_code
**目的**: プライバシー保護された招待コード作成

**パラメータ**:
- `invite_code`: `[u8; 8]` - 8バイトの招待コード

**セキュリティ機能**:
```rust
// ハッシュ生成
salt = generate_random_salt()
code_hash = SHA256(invite_code + salt)

// PDA作成
invite_code_pda = derive_pda(["invite_code", code_hash])
```

**使用制限**:
- 一般ユーザー: 5回
- オペレーター: 1,024回
- アクティブコードのみカウント

**作成されるアカウント**:
- 一意のハッシュベースPDA
- 使用回数追跡
- 作成者権限記録

---

### use_invite_code
**目的**: 招待コードを使用したユーザー登録

**パラメータ**:
- `invite_code`: `[u8; 8]` - 平文の招待コード
- `inviter_pubkey`: `Pubkey` - 招待者のアドレス

**検証プロセス**:
1. コードハッシュ計算
2. InviteCodeアカウント存在確認
3. 使用制限チェック
4. ユーザーアカウント作成
5. 紹介者関係設定

**失敗条件**:
- 無効なコード
- 使用制限超過
- 既存ユーザー

---

## 7. 確率管理命令

### initialize_probability_table
**目的**: シード確率テーブルの初期化

**デフォルト（Table 1）**:
```rust
seed_count: 6
grow_powers: [100, 180, 420, 720, 1000, 5000]
probability_thresholds: [4300, 6800, 8200, 9100, 9700, 10000]
probability_percentages: [43.0, 25.0, 14.0, 9.0, 6.0, 3.0]
expected_value: 421
```

---

### update_probability_table
**目的**: 確率テーブルの動的更新

**パラメータ**:
- `version`: `u32` - バージョン番号
- `seed_count`: `u8` - 有効シード数（1-9）
- `grow_powers`: `Vec<u64>` - 各シードのGrow Power値
- `probability_thresholds`: `Vec<u16>` - 累積確率（10000基準）
- `expected_value`: `u64` - 期待値計算結果

**検証**:
- 累積確率が10000で終了
- Grow Power値の妥当性
- バージョン番号の増加

---

## 8. 統計・管理命令

### initialize_global_stats
**目的**: グローバル統計の初期化

**追跡データ**:
- `total_grow_power`: 全ユーザーの総Grow Power
- `total_farm_spaces`: アクティブ農場数
- `current_rewards_per_second`: 現在の秒間報酬レート
- `last_update_time`: 最終更新時刻

---

### update_global_stats
**目的**: 統計データの更新（内部処理）

**更新タイミング**:
- 農場購入時
- シード植付・除去時
- 報酬請求時
- 半減期適用時

---

## エラーハンドリング

### 主要エラー分類

**権限エラー**:
- `Unauthorized`: 管理者権限不足
- `UnauthorizedUser`: ユーザー権限不足
- `UnauthorizedOperator`: オペレーター権限不足

**状態エラー**:
- `AlreadyHasFarm`: 既に農場所有
- `NoFarmSpace`: 農場未所有
- `SeedAlreadyPlanted`: シード既植付

**容量エラー**:
- `FarmAtMaxCapacity`: 農場容量満杯
- `StorageFull`: ストレージ満杯
- `SeedTypeLimitReached`: 種類別上限到達

**計算エラー**:
- `CalculationOverflow`: 計算オーバーフロー
- `SupplyCapExceeded`: 供給上限超過

**VRFエラー**:
- `VrfResultNotAvailable`: VRF結果未取得
- `InsufficientSolForVrf`: VRF手数料不足

### エラー処理パターン

```rust
// 安全な算術演算
let result = value1.checked_mul(value2)
    .ok_or(GameError::CalculationOverflow)?;

// 条件チェック
require!(farm_space.seed_count < farm_space.capacity, 
         GameError::FarmAtMaxCapacity);

// 権限検証
require!(config.admin == admin.key(), 
         GameError::Unauthorized);
```

この命令セットにより、Facility Gameは包括的で安全、かつユーザーフレンドリーなゲーム体験を提供します。