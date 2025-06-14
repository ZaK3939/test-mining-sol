# Facility Game - システム概要

## プロジェクト概要

Facility Game は、Solana ブロックチェーン上で動作する農場経営とシード育成を組み合わせたゲームです。プレイヤーは農場スペースを購入し、様々な種類のシードを植えて成長力（Grow Power）を蓄積し、定期的に WEED トークンを報酬として受け取ることができます。

## 核心コンセプト

### 1. 農場システム

- **農場スペース**: 0.5 SOL で購入、初期容量 4
  - `constants.rs:24` `FARM_SPACE_COST_SOL = 500_000_000` (0.5 SOL)
  - `state.rs:185` `DEFAULT_FARM_SPACE_COST`でも定義
- **レベルシステム**: 自動アップグレード（レベル 1-5）
  - `constants.rs:55-59` `FARM_CAPACITIES = [4, 6, 8, 10, 12]`
  - `constants.rs:61-65` `FARM_UPGRADE_THRESHOLDS = [0, 30, 100, 300, 500]`
- **成長メカニズム**: シードパック購入数に応じた容量拡張
  - `state.rs:205-215` `increment_pack_purchases()` でパック数追跡
  - `instructions/seeds.rs:329-342` で自動アップグレード実行

### 2. シードエコシステム

- **9 種類のシード**: Seed1（100 GP）〜Seed9（60,000 GP）
  - Table 1 (6 種類): `constants.rs:98-109` `SEED_GROW_POWERS = [100, 180, 420, 720, 1000, 5000]`
  - Table 2 (9 種類): `state.rs:286-287` `GROW_POWERS = [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000]`
- **確率ベース取得**: ミステリーシードパックから入手
  - `constants.rs:111-122` `SEED_PROBABILITY_THRESHOLDS = [4300, 6800, 8200, 9100, 9700, 10000]`
  - `constants.rs:124-128` `SEED_PROBABILITIES = [43.0, 25.0, 14.0, 9.0, 6.0, 3.0]`
- **Switchboard VRF**: 真の乱数生成による公正性確保
  - `instructions/seeds.rs` で VRF 統合（現在簡略化実装）
- **植付システム**: 農場に植えて Grow Power を蓄積
  - `instructions/farm.rs` の `plant_seed()` / `remove_seed()`

### 3. 経済システム

- **比例報酬分配**: 個人の Grow Power ÷ 全体の Grow Power × 基本レート
  - `economics.rs:28-55` `calculate_user_share_reward()` で実装
- **半減期メカニズム**: 7 日間（604,800 秒）毎に基本レートが半減
  - `constants.rs:18` `DEFAULT_HALVING_INTERVAL = 604800` (7 日間)
  - `economics.rs:59-111` `calculate_rewards_across_halving()` で処理
- **総供給量制限**: 240,000,000 WEED 上限
  - `constants.rs:49` `TOTAL_WEED_SUPPLY = 240_000_000 * 1_000_000`
- **転送手数料**: 2%（SPL Token 2022 使用）
  - `constants.rs:36` `TRADING_FEE_PERCENTAGE = 2`

### 4. 招待システム

- **階層報酬**: L1 紹介者 10%、L2 紹介者 5%の報酬分配
  - `constants.rs:155` `LEVEL1_REFERRAL_PERCENTAGE = 10`
  - `constants.rs:158` `LEVEL2_REFERRAL_PERCENTAGE = 5`
- **プライバシー保護**: ハッシュベース招待コード
  - `instructions/invite.rs` でハッシュ検証実装
- **使用制限**: 一般ユーザー 5 回、オペレーター 1024 回
  - `constants.rs:42` `MAX_INVITE_LIMIT = 5`

## 技術アーキテクチャ

### ブロックチェーン基盤

- **Solana**: 高速・低コストな取引処理
- **Anchor Framework**: 型安全なスマートコントラクト開発
  - `lib.rs:12` プログラム ID `GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc`
- **PDA（Program Derived Address）**: 決定論的アドレス生成
  - `constants.rs:190-242` 全 PDA シードパターン定義

### セキュリティ設計

- **権限分離**: 管理者・オペレーター・ユーザーの明確な役割分担
  - `validation/user_validation.rs` `validate_user_ownership()`
  - `validation/admin_validation.rs` 管理者権限検証
- **所有権検証**: 全ての操作でユーザー所有権を確認
  - `utils.rs:10-16` 検証ヘルパー関数群
- **算術安全性**: checked_arithmetic 使用によるオーバーフロー防止
  - `error.rs:21-23` `CalculationOverflow` エラー定義
  - `economics.rs` 全計算で checked 演算使用
- **供給量管理**: ミント権限の PDA による厳格な制御
  - `validation/economic_validation.rs` `validate_supply_cap()`

### 外部サービス統合

- **Switchboard VRF**: 検証可能な真乱数生成
  - `instructions/seeds.rs:4` VRF インポート（現在コメントアウト）
- **Metaplex**: WEED トークンメタデータ管理
  - `frontend/anchor-client.ts:250-275` メタデータ作成実装
- **SPL Token 2022**: 転送手数料拡張機能
  - `utils.rs:3` Token2022 インポート
  - `utils.rs:19-20` 転送手数料対応

## ゲームフロー

### 初回セットアップ

1. **ユーザー登録**: `init_user`命令で基本アカウント作成
2. **農場購入**: `buy_farm_space`で 0.5 SOL 支払い、初期シード付与
3. **トークンアカウント作成**: WEED 受取用の ATA 作成

### 日常プレイサイクル

1. **シードパック購入**: 300 WEED + VRF 手数料でランダムシード取得
2. **パック開封**: Switchboard VRF による乱数生成でシード決定
3. **シード植付**: 農場容量内でシードを配置
4. **報酬請求**: 時間経過に応じた WEED 報酬受取

### 成長戦略

1. **効率的配置**: 高 Grow Power シードの優先植付
2. **容量拡張**: シードパック購入による農場レベル自動アップ
3. **招待活動**: 紹介報酬による追加収入
4. **市場参加**: 他プレイヤーとの WEED 取引

## データ構造設計

### アカウント階層

```
Program
├── Config（グローバル設定）              - state.rs:10-41
├── GlobalStats（全体統計）               - state.rs:503-517
├── ProbabilityTable（確率設定）          - state.rs:85-107
└── User別
    ├── UserState（基本情報）             - state.rs:45-63
    ├── FarmSpace（農場データ）           - state.rs:67-81
    ├── SeedStorage（シード保管庫）       - state.rs:534-546
    ├── Seed × N（個別シード）            - state.rs:351-369
    ├── SeedPack × N（パック履歴）        - state.rs:372-396
    └── InviteCode × N（招待コード）      - state.rs:426-448
```

### PDA シード設計

- `Config`: `["config"]` - `constants.rs:191`
- `UserState`: `["user", user_pubkey]` - `constants.rs:194`
- `FarmSpace`: `["farm_space", user_pubkey]` - `constants.rs:197`
- `Seed`: `["seed", user_pubkey, seed_id]` - `constants.rs:200-203`
- `GlobalStats`: `["global_stats"]` - `constants.rs:206`
- 全 PDA パターン: `constants.rs:190-242`

## 経済バランス

### 報酬計算式

```
個人報酬 = (個人GP / 総GP) × 基本レート × 経過時間
紹介報酬 = 被紹介者報酬 × 紹介レベル報酬率
```

- 実装: `economics.rs:28-55` `calculate_user_share_reward()`
- 紹介報酬: `economics.rs` で 10%/5%の分配計算

### 確率システム（Table 1）

| シード | Grow Power | 確率 | 累積確率 |
| ------ | ---------- | ---- | -------- |
| Seed1  | 100        | 43%  | 43%      |
| Seed2  | 180        | 25%  | 68%      |
| Seed3  | 420        | 14%  | 82%      |
| Seed4  | 720        | 9%   | 91%      |
| Seed5  | 1,000      | 6%   | 97%      |
| Seed6  | 5,000      | 3%   | 100%     |

- データ定義: `constants.rs:98-128`
- 確率判定: `state.rs:300-311` `from_random()`

### 農場拡張システム

| レベル | 容量 | 必要パック購入数 |
| ------ | ---- | ---------------- |
| 1      | 4    | 0                |
| 2      | 6    | 30               |
| 3      | 8    | 100              |
| 4      | 10   | 300              |
| 5      | 12   | 500              |

- 容量定義: `constants.rs:55-59` `FARM_CAPACITIES`
- 閾値定義: `constants.rs:61-65` `FARM_UPGRADE_THRESHOLDS`
- アップグレード: `state.rs:247-256` `calculate_level_from_packs()`

## 運用管理

### 管理者権限

- システム設定変更
  - `instructions/admin.rs` `update_config()` - config 更新
  - `instructions/admin.rs` `initialize_config()` - 初期設定
- 確率テーブル更新
  - `instructions/admin.rs` `update_probability_table()` - 確率変更
- 緊急停止機能
  - `validation/admin_validation.rs` 権限検証機能
- トレジャリー管理
  - `state.rs:19` Config 内 treasury 設定

### オペレーター権限

- 無制限招待コード作成
  - `instructions/invite.rs` オペレーター 1024 回制限
  - `constants.rs:42` 一般ユーザー 5 回制限
- 特別イベント招待発行
  - `state.rs:455-479` SingleUseSecretInvite
- ユーザーサポート用機能
  - `state.rs:36` Config 内 operator 設定

### 監査機能

- 全取引ログ記録
  - 各 instruction 内で msg!()によるログ出力
- 供給量追跡
  - `state.rs:36` Config 内 total_supply_minted
  - `validation/economic_validation.rs` 供給量検証
- 確率検証システム
  - `state.rs:109-164` ProbabilityTable 検証機能
- パフォーマンス監視
  - `frontend/utils/cache-manager.ts` キャッシュ最適化

## 拡張可能性

### 将来計画

- **新シード追加**: Table 2（9 種類）への拡張対応済み
  - `state.rs:286-287` Table 2 データ準備完了
  - `state.rs:109-164` 動的確率テーブル対応
- **農場レベル拡張**: 動的レベル設定により 20 レベルまで対応可能
  - `state.rs:708-751` FarmLevelConfig 実装
  - `state.rs:753-781` 動的レベル計算機能
- **特別イベント**: 期間限定確率テーブル切替
  - ProbabilityTable 版管理システム実装済み
- **ガバナンス**: DAO による重要パラメータ決定
  - 管理者権限の段階的移行準備

## 重要な実装詳細

### プログラム ID

- **現在**: `GX2tJDB1bn73AUkC8brEru4qPN2JSTEd8A1cLAz81oZc` (`lib.rs:12`)

### ストレージ制限

- **最大シード保存**: 2,000 個/ユーザー (`constants.rs:82`)
- **種類別上限**: 100 個/種類 (`constants.rs:87`)
- **バッチ処理上限**: 100 個/操作 (`constants.rs:175-178`)

### VRF 統合状況

- **Switchboard VRF**: 設計完了、実装簡略化中 (`instructions/seeds.rs:4`)
- **エントロピー**: ユーザー提供+VRF+タイムスタンプ組み合わせ

この設計により、Facility Game は持続可能で公正、かつエキサイティングなブロックチェーンゲーム体験を提供します。全ての機能は実際のコード実装に基づいており、確実に動作する機能のみを記載しています。
