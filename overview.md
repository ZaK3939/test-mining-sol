# Solana Facility Game - 完全技術仕様書

## 🎯 プログラム概要

Solana 上に構築された高度な農場シミュレーションゲームです。半減期報酬システム、多段階紹介システム、拡張ストレージ管理を特徴とする複雑な経済メカニズムを実装しています。ユーザーが農場スペースを管理し、種を栽培し、戦略的最適化を通じて WEED トークンを獲得する完全な農業エコシステムを提供します。

**プログラム ID**: `FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B`

### 🌟 主要な特徴

- **⚡ 即座アップグレード**: 24時間クールダウンを廃止し、即座に農場レベルアップが可能
- **📦 拡張ストレージ**: 2000個総容量、タイプ別100個制限、自動廃棄機能付き
- **🔐 招待システム**: ハッシュベースで運営用（無制限）と一般ユーザー用（5回限定）の2パターン
- **💰 高度な経済学**: 半減期メカニズムによる洗練された報酬分配システム
- **🤝 多段階紹介**: L1（10%）、L2（5%）の紹介報酬システム
- **🔬 数学的精密性**: 128 ビット算術によるオーバーフロー完全防止

## 📋 目次

- [🏗️ コアアーキテクチャ](#-コアアーキテクチャ)
- [📊 アカウント構造](#-アカウント構造)
- [🎮 ゲームメカニクス](#-ゲームメカニクス)
- [💰 経済システム](#-経済システム)
- [🚀 高度な機能](#-高度な機能)
- [📊 状態管理](#-状態管理)
- [🔒 セキュリティ & 検証](#-セキュリティ--検証)
- [📚 完全命令セット](#-完全命令セット)
- [🧪 実装詳細](#-実装詳細)

## 🏗️ コアアーキテクチャ

### 🔧 モジュール構造

```
programs/facility-game/src/
├── lib.rs                      # メインプログラムエントリー（30+ 命令）
├── state.rs                    # アカウント構造 & データモデル（12+ 構造体）
├── error.rs                    # カスタムエラー定義（112種類）
├── constants.rs                # 中央集権化ゲーム定数
├── economics.rs                # 経済計算 & 公式
├── utils.rs                    # ヘルパー関数 & ユーティリティ
├── error_handling.rs           # 拡張エラーハンドリング
├── instructions/               # 命令実装
│   ├── mod.rs                  # 命令モジュール統合
│   ├── admin.rs                # 管理者命令
│   ├── user.rs                 # ユーザー管理命令
│   ├── farm.rs                 # 農場管理命令
│   ├── rewards.rs              # 報酬システム命令
│   ├── referral_rewards.rs     # 紹介報酬命令
│   ├── seeds.rs                # 種システム命令
│   ├── invite_secret.rs        # 招待システム
│   ├── transfer.rs             # 転送システム
│   ├── transfer_improved.rs    # 改良転送システム
│   └── meteora.rs              # Meteora統合（基本機能）
├── validation/                 # ビジネスルール検証
│   ├── mod.rs                  # 検証モジュール統合
│   ├── admin_validation.rs     # 管理者権限検証
│   ├── economic_validation.rs  # 経済検証
│   ├── game_validation.rs      # ゲームロジック検証
│   ├── time_validation.rs      # 時間関連検証
│   ├── user_validation.rs      # ユーザー検証
│   └── common.rs               # 共通検証機能
├── test_modules/               # 高度なテストモジュール
│   ├── economics_advanced_tests.rs
│   ├── error_comprehensive_tests.rs
│   └── state_advanced_tests.rs
└── tests/                      # ユニットテスト
    ├── admin_tests.rs
    ├── economic_distribution_tests.rs
    ├── error_tests.rs
    ├── referral_system_tests.rs
    ├── state_tests.rs
    └── utils_tests.rs
```

### 🔑 重要な設計決定

**PDA Seeds Pattern**:
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- FarmSpace: `["farm_space", user_pubkey]`
- SeedStorage: `["seed_storage", user_pubkey]`
- InviteCode: `["secret_invite", inviter_pubkey, invite_code]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`
- GlobalStats: `["global_stats"]`
- FeePool: `["fee_pool"]`

**即座アップグレードシステム**:
- 24時間クールダウンを完全削除
- WEEDトークン消費で即座にレベルアップ完了
- アップグレード: Lv1→2: 3,500 WEED, Lv2→3: 18,000 WEED
- ユーザビリティ改善とゲーム体験の向上

**拡張ストレージシステム**:
- 総容量: 2,000個（従来の2倍）
- タイプ別制限: 各種類100個まで
- 自動廃棄機能: 制限到達時の最低価値種自動削除
- レント効率: 同じコスト/種比率を維持

**招待システム**:
- ハッシュベース（SHA256）による秘匿性確保
- 運営招待: 無制限使用（255回）
- 一般招待: 5回限定使用
- プライバシー保護とスパム防止

## 📊 アカウント構造

### 🔧 Config（グローバル設定）
```rust
pub struct Config {
    pub base_rate: u64,                    // 基本報酬レート（100 WEED/秒）
    pub halving_interval: i64,             // 半減期間隔（7日）
    pub next_halving_time: i64,            // 次回半減期タイムスタンプ
    pub admin: Pubkey,                     // システム管理者
    pub treasury: Pubkey,                  // 手数料収集ウォレット
    pub seed_pack_cost: u64,               // 種パック価格（300 WEED）
    pub seed_counter: u64,                 // グローバル種ID カウンター
    pub seed_pack_counter: u64,            // グローバル種パックIDカウンター
    pub farm_space_cost_sol: u64,          // 農場購入コスト（0.5 SOL）
    pub max_invite_limit: u8,              // 最大招待制限（5）
    pub trading_fee_percentage: u8,        // 取引手数料（2%）
    pub protocol_referral_address: Pubkey, // プロトコル紹介アドレス
    pub total_supply_minted: u64,          // 総発行WEED量
    pub operator: Pubkey,                  // 運営者アドレス（無制限招待権限）
    pub reserve: [u8; 2],                  // 将来拡張用
}
```

### 👤 UserState（ユーザー状態）
```rust
pub struct UserState {
    pub owner: Pubkey,                     // ユーザーウォレット
    pub total_grow_power: u64,             // 総グロウパワー
    pub last_harvest_time: i64,            // 最終報酬請求時刻
    pub has_farm_space: bool,              // 農場所有フラグ
    pub referrer: Option<Pubkey>,          // 紹介者（多段階報酬用）
    pub pending_referral_rewards: u64,     // 未請求紹介報酬
    pub reserve: [u8; 32],                 // 将来拡張用
}
```

### 🏭 FarmSpace（農場スペース）
```rust
pub struct FarmSpace {
    pub owner: Pubkey,                     // 農場所有者
    pub level: u8,                         // レベル（1-5）
    pub capacity: u8,                      // 容量（4,8,12,16,20）
    pub seed_count: u8,                    // 植付け済み種数
    pub total_grow_power: u64,             // 農場総グロウパワー
    pub reserve: [u8; 32],                 // 将来拡張用
}
```

### 📦 SeedStorage（拡張ストレージ）
```rust
pub struct SeedStorage {
    pub owner: Pubkey,                     // ストレージ所有者
    pub seed_ids: Vec<u64>,                // 種IDリスト（最大2000個）
    pub total_seeds: u32,                  // 総種数
    pub seed_type_counts: [u16; 9],        // タイプ別カウント（各100個制限）
    pub reserve: [u8; 16],                 // 将来拡張用
}

impl SeedStorage {
    pub const MAX_TOTAL_SEEDS: usize = 2_000;     // 総容量
    pub const MAX_SEEDS_PER_TYPE: u16 = 100;      // タイプ別制限
    
    // 容量チェック（総制限）
    pub fn can_add_seed(&self) -> bool;
    
    // 容量チェック（タイプ別制限）
    pub fn can_add_seed_type(&self, seed_type: &SeedType) -> bool;
    
    // 自動廃棄機能（タイプ制限到達時）
    pub fn auto_discard_excess(&mut self, new_seed_type: &SeedType) -> Result<u16>;
}
```

### 🔐 InviteCode（招待コード）
```rust
pub struct InviteCode {
    pub inviter: Pubkey,                   // 招待者
    pub invite_limit: u8,                  // 招待限度（運営255、一般5）
    pub invites_used: u8,                  // 使用済み招待数
    pub creation_time: i64,                // 作成時刻
    pub reserve: [u8; 32],                 // 将来拡張用
}
```

## 🎮 ゲームメカニクス

### ⚡ 即座農場アップグレードシステム

#### アップグレード仕様
- **即座実行**: クールダウンなし、WEEDトークン消費で即座完了
- **レベル別コスト**: 
  - Lv1→2: 3,500 WEED（容量4→8）
  - Lv2→3: 18,000 WEED（容量8→12）
  - Lv3→4: 20,000 WEED（容量12→16）
  - Lv4→5: 25,000 WEED（容量16→20）

#### 実装詳細
```rust
pub fn upgrade_farm_space(ctx: Context<UpgradeFarmSpace>) -> Result<()> {
    // 即座実行 - クールダウンチェックなし
    // WEEDトークン消費
    // レベル＆容量即座更新
}
```

### 📦 拡張ストレージシステム

#### ストレージ仕様
- **総容量**: 2,000個（従来の2倍）
- **タイプ別制限**: 各種類100個まで
- **アカウント費用**: ~0.12 SOL（コスト効率は従来と同等）
- **レント回収**: 種廃棄時に個別レント回収可能

#### 自動廃棄機能
```rust
// タイプ制限到達時の自動廃棄ロジック
pub fn auto_discard_excess(&mut self, new_seed_type: &SeedType) -> Result<u16> {
    // 制限チェック
    // 最低価値種特定
    // 自動廃棄実行
    // カウンター更新
}
```

#### 経済分析
- **容量効率**: 種あたりコスト変わらず、容量2倍
- **管理改善**: タイプ別制限でバランス取れたコレクション
- **戦略性向上**: レア種優先保管、コモン種自動整理

### 🔐 招待システム

#### ハッシュベース設計
```rust
// SHA256(invite_code + salt + inviter_pubkey)
pub fn create_invite_code(
    ctx: Context<CreateInviteCode>, 
    invite_code: [u8; 8]
) -> Result<()>;

pub fn use_invite_code(
    ctx: Context<UseInviteCode>, 
    invite_code: [u8; 8],
    inviter_pubkey: Pubkey
) -> Result<()>;
```

#### 招待パターン
1. **運営招待**:
   - 制限: 255回（事実上無制限）
   - 用途: マーケティング、イベント、初期ユーザー獲得
   - 管理: 運営者アドレスから発行

2. **一般ユーザー招待**:
   - 制限: 5回
   - 用途: 友達招待、コミュニティ成長
   - 管理: 個別ユーザーが発行

#### プライバシー保護
- 招待コードはハッシュ化され、プレーンテキストは保存されない
- 招待者アドレスが必要で、ブルートフォース攻撃を防ぐ
- PDAベースで重複防止

## 💰 経済システム

### 💎 報酬分配システム

#### 比例配分計算式
```
ユーザー報酬 = (ユーザーGrow Power / 全体Grow Power) × 基本レート × 経過時間
```

#### 半減期メカニズム
- **間隔**: 7日ごと（カスタマイズ可能）
- **効果**: 基本レートを50%削減
- **目的**: インフレ抑制、長期価値維持

### 🤝 多段階紹介システム

#### 紹介報酬率
- **Level 1（直接招待）**: 10%
- **Level 2（間接招待）**: 5%
- **最大深度**: 2レベル

#### 実装詳細
```rust
pub fn claim_reward_with_referral_rewards(
    ctx: Context<ClaimRewardWithReferralRewards>
) -> Result<()> {
    // 基本報酬請求
    // L1: base_reward × 10%
    // L2: base_reward × 5%
    // プロトコルアドレス除外
    // 統合処理で一度にすべて実行
}
```

### 💸 手数料システム

#### 取引手数料
- **レート**: 2%
- **収集先**: FeePool PDA
- **変換**: Meteora経由でSOLに変換（実装済み）

#### 手数料の流れ
1. ユーザー間転送 → 2%手数料徴収
2. FeePool蓄積
3. 管理者による定期的SOL変換
4. プロトコル運営資金として活用

## 🚀 高度な機能

### 🌱 種システム

#### 種タイプとレアリティ
```rust
pub enum SeedType {
    Seed1 = 0,  // 100GP (42.23%)
    Seed2 = 1,  // 180GP (24.44%)
    Seed3 = 2,  // 420GP (13.33%)
    Seed4 = 3,  // 720GP (8.33%)
    Seed5 = 4,  // 1000GP (5.56%)
    Seed6 = 5,  // 5000GP (3.33%)
    Seed7 = 6,  // 15000GP (1.33%)
    Seed8 = 7,  // 30000GP (0.89%)
    Seed9 = 8,  // 60000GP (0.56%)
}
```

#### ミステリーパック
- **コスト**: 300 WEED
- **期待値**: 1,226.79 グロウパワー
- **効率**: 4.09 GP/WEED
- **最大購入**: 100パック/回

### 🔄 Meteora統合

#### 基本機能（実装済み）
```rust
// 手数料のSOL変換
pub fn convert_fees_to_sol(ctx: Context<ConvertFeesToSol>) -> Result<()>;

// Meteora設定更新
pub fn update_meteora_config(
    ctx: Context<UpdateMeteoraConfig>,
    meteora_pool: Pubkey,
    pool_weed_vault: Pubkey,
    pool_sol_vault: Pubkey,
) -> Result<()>;
```

#### 高度機能（準備済み、コメントアウト）
- DLMM（Dynamic Liquidity Market Maker）統合
- 自動流動性管理
- スリッページ制御
- 緊急停止機能

## 📚 完全命令セット

### 👨‍💼 管理者命令
1. `initialize_config` - システム設定初期化
2. `create_reward_mint` - WEEDトークンミント作成
3. `initialize_global_stats` - グローバル統計初期化
4. `initialize_fee_pool` - 手数料プール初期化
5. `update_config` - システム設定更新

### 👤 ユーザー管理命令
6. `init_user` - ユーザーアカウント初期化

### 🏭 農場管理命令
7. `buy_farm_space` - 農場スペース購入（0.5 SOL）
8. `upgrade_farm_space` - **即座農場アップグレード**

### 💰 報酬システム命令
9. `claim_reward_with_referral_rewards` - **統合報酬請求**（農場＋紹介報酬すべて）
10. `accumulate_referral_reward` - 紹介報酬蓄積（内部処理用）
11. `view_pending_referral_rewards` - 未請求紹介報酬確認

### 🔐 招待システム命令
12. `create_invite_code` - **招待コード作成**
13. `use_invite_code` - **招待コード使用**

### 🌱 種システム命令
14. `initialize_seed_storage` - **拡張ストレージ初期化**
15. `purchase_seed_pack` - ミステリーパック購入
16. `open_seed_pack` - パック開封
17. `plant_seed` - 種植え
18. `remove_seed` - 種除去
19. `discard_seed` - 種廃棄（レント回収）
20. `batch_discard_seeds` - 一括種廃棄（効率的レント回収）

### 💸 取引システム命令
21. `transfer_with_improved_fee` - **手数料付き転送**（FeePool蓄積、推奨）
22. `batch_transfer_with_fee` - 一括手数料転送

### 🔄 Meteora統合命令
23. `convert_fees_to_sol` - **手数料SOL変換**
24. `update_meteora_config` - Meteora設定更新

### 🚀 実装された機能の完成度

#### ✅ 完全実装済み
- **即座アップグレード**: 24時間クールダウン完全削除
- **拡張ストレージ**: 2000個容量、タイプ別制限、自動廃棄
- **招待システム**: ハッシュベース、運営/一般パターン
- **多段階紹介**: L1/L2報酬分配
- **基本報酬システム**: 比例配分、半減期
- **手数料システム**: 2%徴収、FeePool蓄積

#### 🔄 基本実装済み（機能拡張可能）
- **Meteora統合**: 基本機能実装、高度機能準備済み
- **種システム**: 基本機能完備、Pyth Entropy統合準備済み

#### 📈 拡張可能性
- DLMM高度機能の有効化
- Pyth Entropy真正乱数統合
- NFT統合（種の個別特性）
- ガバナンストークン統合

## 🔒 セキュリティ & 検証

### 🛡️ セキュリティ設計原則

#### PDAベースアクセス制御
- 全アカウントがPDAsで管理
- 署名者ベース所有権検証
- クロスプログラム呼び出し制限

#### 数値オーバーフロー防止
```rust
use anchor_lang::__private::ZeroCopyAccessor;
// checked_add, checked_mul等を使用
// u64/u128による大容量計算対応
```

#### 包括的エラーハンドリング
- 112種類の詳細エラー定義
- ビジネスロジック検証
- 経済的制約チェック

### 🧪 テスト体系

#### 統合テスト
- 完全ユーザージャーニーテスト
- 運営/一般招待パターン検証
- ストレージシステム機能テスト
- エラーケース網羅テスト

#### ユニットテスト
- 個別モジュール機能テスト
- 経済計算正確性検証
- セキュリティ境界テスト

## 🎉 まとめ

Solana Facility Gameは、**即座アップグレード**、**拡張ストレージシステム**、**招待システム**を中核とする高度な農場シミュレーションゲームです。

### 主要な技術的達成
1. **ユーザビリティ改善**: 24時間クールダウン削除による即座アップグレード
2. **スケーラビリティ**: 2000個ストレージ容量、タイプ別自動管理
3. **プライバシー保護**: ハッシュベース招待システム
4. **経済バランス**: 持続可能な報酬分配と半減期メカニズム
5. **拡張性**: モジュラー設計による将来機能追加対応

この実装により、ユーザーは快適なゲーム体験を得ながら、プロトコルは長期的な経済安定性とコミュニティ成長を実現します。