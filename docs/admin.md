# 管理者設定ガイド

このドキュメントでは、Facility Gameプログラムで利用可能な全ての設定可能パラメータと管理者専用機能について説明します。

## 概要

Facility Gameプログラムは、管理者専用命令を通じて広範な設定オプションを提供します。全ての管理者機能は、管理者ウォレットの署名が必要で、グローバルConfig PDAで動作します。

## グローバル設定（Config PDA）

### 基本経済パラメータ

#### 基本報酬レート
- **現在のデフォルト**: 100 WEED/秒
- **関数**: `update_config(new_base_rate: Option<u64>)`
- **説明**: 半減機構適用前の基本報酬生成レート
- **影響**: 全ユーザーの報酬計算に影響
- **考慮事項**: 高いレートはトークンインフレを増加させる

#### 半減機構
- **現在のデフォルト**: 200秒（テスト用）、31,536,000秒（1年、本番用）
- **関数**: `update_config(new_halving_interval: Option<i64>)`
- **説明**: 自動報酬レート半減の間隔
- **影響**: 長期的なトークン供給インフレをコントロール
- **考慮事項**: 短い間隔はトークン供給をより早く減少させる

#### シードパックコスト（WEED価格）
- **現在のデフォルト**: 300 WEED（6桁精度で300,000,000）
- **関数**: `update_seed_pack_cost(new_seed_pack_cost: u64)`
- **範囲**: 1 - 10,000 WEED
- **説明**: ミステリーシードパックのWEEDトークンでのコスト
- **影響**: アクセス障壁とトークン燃焼率をコントロール
- **例**: `500_000_000` = 500 WEED

### システムアドレス

#### Treasuryウォレット
- **関数**: `update_config(new_treasury: Option<Pubkey>)`
- **説明**: SOL支払いと送金手数料を受け取るウォレット
- **用途**: 農場スペース購入（0.5 SOL）、手数料回収
- **セキュリティ**: 本番環境ではマルチシグウォレットを推奨

#### Operatorアドレス
- **関数**: `update_config(new_operator: Option<Pubkey>)`
- **説明**: 無制限の招待特権を持つ特別なアドレス
- **用途**: マーケティングキャンペーン、特別イベント
- **特権**: 招待制限をバイパス、秘密招待コードの作成

#### プロトコル紹介アドレス
- **一度だけ設定**: `initialize_config`時
- **説明**: 紹介報酬を得ないアドレス
- **用途**: プロトコル運営の循環紹介報酬を防止

### 農場システムパラメータ

#### 農場スペースコスト
- **現在のデフォルト**: 500,000,000 lamports（0.5 SOL）
- **場所**: `Config::DEFAULT_FARM_SPACE_COST`
- **説明**: 初期農場スペース購入のSOLコスト
- **注意**: 現在はハードコード済み、設定可能にすることも可能

#### 最大招待制限
- **現在のデフォルト**: ユーザー当たり5招待
- **関数**: `update_config(new_max_invite_limit: Option<u8>)`
- **説明**: 通常ユーザーのデフォルト招待制限
- **上書き**: Operatorは無制限招待（255）

#### 取引手数料率
- **現在のデフォルト**: 2%
- **場所**: `Config.trading_fee_percentage`
- **説明**: トークン転送の手数料率（SPL Token 2022 Transfer Fee Extension）
- **注意**: ミント作成時に設定、別途更新機構が必要

## 動的設定システム

### 確率テーブル

ゲームはシード生成に動的確率テーブルを使用し、コードデプロイなしでランタイム調整が可能です。最大16種類のシードまでサポートし、頻繁な更新に対応します。

#### 確率テーブル初期化
```rust
initialize_probability_table(ctx: Context<InitializeProbabilityTable>)
```

#### 確率テーブル更新
```rust
update_probability_table(
    ctx: Context<UpdateProbabilityTable>,
    version: u32,
    seed_count: u8,
    grow_powers: Vec<u64>,
    probability_thresholds: Vec<u16>,
    probability_percentages: Vec<f32>,
    expected_value: u64,
    name: String,
)
```

**パラメータ**:
- `version`: 更新追跡用のバージョン（頻繁な更新に対応）
- `seed_count`: シード種類数（1-16、将来的な拡張対応）
- `grow_powers`: 各シード種類のGrow Power値（最大16種類）
- `probability_thresholds`: 累積閾値（10,000中）
- `probability_percentages`: 人間が読みやすいパーセンテージ
- `expected_value`: パック当たりの計算期待Grow Power
- `name`: テーブル識別子（最大32文字、例："SeasonSpring2024"）

#### テーブル設計パターン

**基本テーブル（6-9シード）**:
```javascript
// スタンダードテーブル - バランス重視
{
  version: 1,
  seedCount: 6,
  growPowers: [100, 180, 420, 720, 1000, 5000],
  thresholds: [4300, 6800, 8200, 9100, 9700, 10000],
  percentages: [43.0, 25.0, 14.0, 9.0, 6.0, 3.0],
  expectedValue: 421,
  name: "Standard Table v1"
}
```

**拡張テーブル（10-12シード）**:
```javascript
// プレミアムテーブル - より多様性重視
{
  version: 5,
  seedCount: 12,
  growPowers: [50, 100, 200, 350, 500, 750, 1200, 2000, 3500, 6000, 12000, 25000],
  thresholds: [3000, 5500, 7200, 8300, 9000, 9400, 9650, 9800, 9900, 9960, 9990, 10000],
  percentages: [30.0, 25.0, 17.0, 11.0, 7.0, 4.0, 2.5, 1.5, 1.0, 0.6, 0.3, 0.1],
  expectedValue: 389,
  name: "Premium Diversity v5"
}
```

**極レアテーブル（14-16シード）**:
```javascript
// ウルトラプレミアム - 最高レアリティ対応
{
  version: 12,
  seedCount: 16,
  growPowers: [25, 50, 100, 180, 300, 500, 800, 1300, 2100, 3400, 5500, 8900, 14400, 23300, 37700, 100000],
  thresholds: [2500, 4500, 6200, 7500, 8400, 9000, 9350, 9550, 9700, 9800, 9870, 9920, 9955, 9975, 9990, 10000],
  percentages: [25.0, 20.0, 17.0, 13.0, 9.0, 6.0, 3.5, 2.0, 1.5, 1.0, 0.7, 0.5, 0.35, 0.2, 0.15, 0.1],
  expectedValue: 425,
  name: "Ultra Premium v12"
}
```

#### 特別イベントテーブル例

**季節限定テーブル**:
```javascript
// 春イベント - 成長ボーナス
{
  version: 20,
  seedCount: 8,
  growPowers: [150, 270, 630, 1080, 1500, 7500, 22500, 45000], // 1.5倍ボーナス
  thresholds: [4000, 6500, 8000, 9000, 9600, 9900, 9980, 10000],
  percentages: [40.0, 25.0, 15.0, 10.0, 6.0, 3.0, 0.8, 0.2],
  expectedValue: 632,
  name: "Spring Boost 2024"
}
```

**レアリティフェス**:
```javascript
// 高レアリティ確率アップ
{
  version: 25,
  seedCount: 9,
  growPowers: [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000],
  thresholds: [3000, 5000, 6500, 7500, 8300, 9200, 9700, 9900, 10000], // レア確率2倍
  percentages: [30.0, 20.0, 15.0, 10.0, 8.0, 9.0, 5.0, 2.0, 1.0],
  expectedValue: 1247,
  name: "Rarity Fest v25"
}
```

### 秘密シード管理システム

#### シード公開機能
新しい秘密シードを段階的に公開する管理機能：

**シード公開命令**:
```rust
reveal_seed(
    ctx: Context<RevealSeed>,
    seed_index: u8,        // 0-15 (Seed1-Seed16)
    grow_power: u64,       // シードのGrow Power値
    probability_percentage: f32,  // 確率パーセンテージ
)
```

**使用例**:
```javascript
// Seed9（index 8）を公開 - 60,000 GP、1.5%確率
await client.revealSeed(8, 60000, 1.5);

// Seed16（index 15）を公開 - 10,000,000 GP、0.1%確率
await client.revealSeed(15, 10000000, 0.1);
```

#### シード値更新機能
既に公開済みのシードの値を変更する管理機能：

**シード値更新命令**:
```rust
update_seed_values(
    ctx: Context<UpdateSeedValues>,
    seed_index: u8,        // 0-15 (Seed1-Seed16)
    grow_power: u64,       // 新しいGrow Power値
    probability_percentage: f32,  // 新しい確率パーセンテージ
)
```

**使用例**:
```javascript
// Seed1のGrow Powerを100から150に増加
await client.updateSeedValues(0, 150, 30.0);

// Seed8の確率を3%から5%に変更
await client.updateSeedValues(7, 30000, 5.0);
```

#### 秘密シード管理戦略

**段階的公開スケジュール**:
```javascript
// 第1段階: ゲーム開始（8シード公開済み）
const initialSeeds = {
  revealed: [0, 1, 2, 3, 4, 5, 6, 7], // Seed1-8
  hidden: [8, 9, 10, 11, 12, 13, 14, 15] // Seed9-16
};

// 第2段階: 1ヶ月後（中級シード公開）
const phase2 = {
  newReveals: [8, 9], // Seed9-10公開
  revealedTotal: 10
};

// 第3段階: 3ヶ月後（高級シード公開）
const phase3 = {
  newReveals: [10, 11, 12], // Seed11-13公開
  revealedTotal: 13
};

// 最終段階: 6ヶ月後（最上級シード公開）
const finalPhase = {
  newReveals: [13, 14, 15], // Seed14-16公開
  revealedTotal: 16
};
```

**イベント連動公開**:
```javascript
// 特別イベント時の限定公開
const eventReveal = {
  event: "Anniversary Event",
  duration: "2週間限定",
  specialSeed: {
    index: 14, // Seed15
    growPower: 5000000,
    probability: 0.2,
    temporaryBoost: true // イベント終了後に確率調整
  }
};
```

#### バランス調整システム

**定期バランス更新**:
```javascript
// 週次調整例
const weeklyBalance = {
  schedule: "毎週月曜日",
  adjustmentRange: "±10%以内",
  targetSeeds: [0, 1, 2, 3], // 基本シードのみ
  approval: "ゲームマスター"
};

// 月次大型調整例
const monthlyBalance = {
  schedule: "月初",
  adjustmentRange: "±50%以内", 
  targetSeeds: [4, 5, 6, 7], // 中級シード
  approval: "運営チーム承認"
};
```

**メタゲーム対応**:
```javascript
// プレイヤー行動に基づく動的調整
const metaAdjustment = {
  trigger: "特定シード使用率 > 60%",
  action: "確率調整で分散化",
  example: {
    seedIndex: 5, // Seed6が人気過ぎる場合
    oldProbability: 4.0,
    newProbability: 2.5, // 確率を下げてバランス調整
    compensation: "他シードの確率を微増"
  }
};
```

#### 頻繁更新の管理戦略

**バージョン管理**:
- **メジャー更新**: 10単位（v10, v20, v30...）- シーズン変更
- **マイナー更新**: 1単位（v11, v12, v13...）- バランス調整
- **ホットフィックス**: 0.1単位（v11.1, v11.2...）- 緊急修正

**更新スケジュール例**:
```javascript
// 週次バランス調整
const weeklyUpdate = {
  schedule: "毎週月曜日 JST 10:00",
  scope: "確率微調整（±2%以内）",
  approval: "ゲームマスター承認"
};

// 月次大型更新
const monthlyUpdate = {
  schedule: "毎月第1月曜日 JST 10:00",
  scope: "新シード追加、大幅バランス変更",
  approval: "運営チーム全体承認"
};

// 季節イベント
const seasonalUpdate = {
  schedule: "四半期（3/6/9/12月）",
  scope: "特別テーブル、限定シード",
  approval: "コミュニティ投票 + 運営承認"
};
```

#### A/Bテスト対応

**同時複数テーブル運用**:
```javascript
// テストグループA（保守的）
const tableA = {
  version: 30,
  name: "Conservative Test A",
  expectedValue: 400,
  // 安定した確率分布
};

// テストグループB（アグレッシブ）
const tableB = {
  version: 31,
  name: "Aggressive Test B", 
  expectedValue: 600,
  // 高リスク高リターン分布
};
```

#### 期待値計算ツール

**自動計算式**:
```javascript
function calculateExpectedValue(growPowers, percentages) {
  return growPowers.reduce((sum, power, index) => {
    return sum + (power * percentages[index] / 100);
  }, 0);
}

// バランス検証
function validateBalance(thresholds) {
  const total = thresholds[thresholds.length - 1];
  return total === 10000; // 必ず100%になること
}
```

### 農場レベル設定

コード変更なしで新レベル追加が可能な動的農場レベルシステム。

#### 農場レベル設定初期化
```rust
initialize_farm_level_config(ctx: Context<InitializeFarmLevelConfig>)
```

#### 農場レベル設定更新
```rust
update_farm_level_config(
    ctx: Context<UpdateFarmLevelConfig>,
    max_level: u8,
    capacities: Vec<u8>,
    upgrade_thresholds: Vec<u32>,
    level_names: Option<Vec<String>>,
)
```

**現在のデフォルト（5レベル）**:
- レベル1: 容量4、必要パック数0
- レベル2: 容量6、必要パック数30
- レベル3: 容量8、必要パック数100
- レベル4: 容量12、必要パック数300
- レベル5: 容量16、必要パック数500

## システム初期化機能

### 必要な設定手順

1. **設定初期化**
   ```rust
   initialize_config(
       base_rate: Option<u64>,
       halving_interval: Option<i64>, 
       treasury: Pubkey,
       protocol_referral_address: Option<Pubkey>
   )
   ```

2. **報酬ミント作成**（SPL Token 2022 with Transfer Fee Extension）
   ```rust
   create_reward_mint()
   ```

3. **グローバル統計初期化**
   ```rust
   initialize_global_stats()
   ```

4. **手数料プール初期化**
   ```rust
   initialize_fee_pool(treasury_address: Pubkey)
   ```

5. **確率テーブル初期化**
   ```rust
   initialize_probability_table()
   ```

6. **農場レベル設定初期化**
   ```rust
   initialize_farm_level_config()
   ```

## セキュリティ考慮事項

### 管理者キー管理
- **本番環境**: 管理者機能にはマルチシグウォレットを使用
- **テスト環境**: 開発では単一キーペアでも可
- **ローテーション**: 管理者キーは`update_config`で変更可能

### パラメータ検証
- **シードパックコスト**: 1 - 10,000 WEED範囲で強制
- **確率テーブル**: 100%（10,000/10,000）の合計必須
- **農場レベル**: 最大20レベルサポート
- **招待制限**: 0-255範囲（u8）

### 経済影響評価

#### 変更前の確認事項
1. 既存ユーザーへの**影響をモデル化**
2. 新しいトークン発行レートを**計算**
3. 既存アカウントの**移行ニーズを検討**
4. Devnetで**徹底的にテスト**
5. コミュニティに変更を**周知**

#### 高影響変更
- **基本レート**: 既存全農場に即座に影響
- **半減間隔**: 長期トークノミクスを変更
- **シードパックコスト**: ゲームアクセス性に影響
- **確率テーブル**: 期待値を変更

#### 低影響変更
- **Treasuryアドレス**: 管理のみ
- **Operatorアドレス**: 招待システムのみに影響
- **最大招待制限**: 新しい招待コードのみに影響

## フロントエンド統合

### TypeScriptクライアントメソッド

```typescript
// シードパックコスト更新
await client.updateSeedPackCost(500_000_000); // 500 WEED

// 一般設定更新
await client.updateConfig({
  newOperator: operatorPubkey,
  newBaseRate: 150, // 150 WEED/秒
  newHalvingInterval: 86400, // 1日
  newTreasury: treasuryPubkey,
  newMaxInviteLimit: 10
});

// 確率テーブル更新
await client.updateProbabilityTable({
  version: 2,
  seedCount: 6,
  growPowers: [100, 180, 420, 720, 1000, 5000],
  // ... 他のパラメータ
});
```

## 監視とアナリティクス

### 追跡すべき主要メトリクス
- **トークン供給**: 発行総量 vs 燃焼量
- **報酬レート**: 半減後の現在の実効レート
- **農場分布**: レベルと容量利用率
- **シードパック販売**: ボリュームと収益
- **ユーザー成長**: 招待コード経由の新規登録

### 推奨ダッシュボード
- **経済概要**: 供給量、燃焼率、報酬分配
- **ゲームメトリクス**: 農場レベル、シードパック購入、ユーザー活動
- **システムヘルス**: トランザクション成功率、エラーパターン

## 緊急時対応

### パラメータロールバック
全ての設定変更は、同じ更新機能で元に戻すことができます。

### サーキットブレーカー
実装を検討：
- パラメータの急激な変動を防ぐ**1日最大変更数**
- 重要な経済変更の**時間遅延**
- 高影響変更の**マルチ署名要求**

### インシデント対応
1. 問題のあるパラメータを**特定**
2. 安全なロールバック値を**計算**
3. 更新トランザクションを**実行**
4. システム回復を**監視**
5. インシデントと予防策を**文書化**

## ベストプラクティス

### テストプロトコル
1. 全パラメータ変更の**Devnetテスト**
2. 本番デプロイ前の**経済モデリング**
3. 重要変更の**段階的ロールアウト**
4. 実装前の**ユーザーコミュニケーション**

### 変更管理
- 理由付きで全変更を**文書化**
- 設定履歴の**バージョン管理**
- 経済変更の**ステークホルダー承認**
- ユーザー向け変更の**コミュニティ通知**

### 定期メンテナンス
- パラメータ効果の**月次レビュー**
- 経済メトリクスの**四半期分析**
- 変更後の**ドキュメント更新**
- 参考用の**古い設定をアーカイブ**

## 使用例

### 基本的な価格調整
```bash
# シードパック価格を500 WEEDに変更
anchor run update-seed-price -- --new-cost 500000000

# 基本報酬レートを150 WEED/秒に変更
anchor run update-config -- --base-rate 150
```

### 確率テーブルの更新
```javascript
// より高いレアリティ設定への変更
const newTable = {
  version: 3,
  seedCount: 6,
  growPowers: [100, 200, 500, 800, 1200, 6000],
  thresholds: [4000, 6500, 8000, 9000, 9600, 10000],
  percentages: [40.0, 25.0, 15.0, 10.0, 6.0, 4.0],
  expectedValue: 480,
  name: "Enhanced Rarity Table"
};
```

### 農場レベル拡張
```javascript
// レベル6を追加
const expandedFarm = {
  maxLevel: 6,
  capacities: [4, 6, 8, 12, 16, 20],
  thresholds: [0, 30, 100, 300, 500, 1000],
  names: ["Seedling", "Sprout", "Growing", "Flourishing", "Blooming", "Master"]
};
```

これらの設定により、ゲームの経済バランスを動的に調整し、プレイヤー体験を最適化できます。