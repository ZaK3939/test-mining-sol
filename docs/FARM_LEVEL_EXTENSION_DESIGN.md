# FarmSpace レベル拡張設計書

## 実装場所

**統合先**: `src/instructions/farm.rs`  
農場関連の全機能を1つのモジュールに集約し、管理しやすくしました。

## 現状の問題点

1. **ハードコード依存**: レベル1-5が固定配列で実装
2. **拡張困難**: 新レベル追加時に多数のファイル修正が必要
3. **バリデーション固定**: レベル範囲チェックが1-5で固定

## 改善案

### 1. 動的レベル設定システム

#### A. Config拡張
```rust
// state.rs に追加
pub struct FarmLevelConfig {
    pub max_level: u8,
    pub capacities: Vec<u8>,
    pub upgrade_thresholds: Vec<u32>,
    pub names: Vec<String>, // オプション: レベル名
    pub reserve: [u8; 64],
}
```

#### B. 定数の整理
```rust
// constants.rs を更新
pub const INITIAL_MAX_LEVEL: u8 = 5;
pub const MAX_POSSIBLE_LEVELS: u8 = 20; // システム上限

// デフォルト値（初期5レベル用）
pub const DEFAULT_FARM_CAPACITIES: [u8; 5] = [4, 6, 8, 10, 12];
pub const DEFAULT_UPGRADE_THRESHOLDS: [u32; 5] = [0, 30, 100, 300, 500];

// 拡張例（レベル10まで）
pub const EXTENDED_FARM_CAPACITIES: [u8; 10] = [
    4, 6, 8, 10, 12,      // レベル1-5（既存）
    15, 18, 22, 26, 30    // レベル6-10（新規）
];

pub const EXTENDED_UPGRADE_THRESHOLDS: [u32; 10] = [
    0, 30, 100, 300, 500,           // レベル1-5（既存）
    1000, 2000, 3500, 5000, 7500    // レベル6-10（新規）
];
```

### 2. 実装の柔軟化

#### A. FarmSpace メソッド更新
```rust
impl FarmSpace {
    pub fn get_capacity_for_level(level: u8, config: &FarmLevelConfig) -> u8 {
        if level == 0 || level > config.max_level {
            return config.capacities[config.max_level as usize - 1];
        }
        config.capacities[(level - 1) as usize]
    }
    
    pub fn calculate_level_from_packs(total_packs: u32, config: &FarmLevelConfig) -> u8 {
        for (i, &threshold) in config.upgrade_thresholds.iter().enumerate().rev() {
            if total_packs >= threshold {
                return (i + 1) as u8;
            }
        }
        1
    }
    
    pub fn auto_upgrade(&mut self, total_packs: u32, config: &FarmLevelConfig) -> bool {
        let new_level = Self::calculate_level_from_packs(total_packs, config);
        if new_level > self.level && new_level <= config.max_level {
            self.level = new_level;
            self.capacity = Self::get_capacity_for_level(new_level, config);
            true
        } else {
            false
        }
    }
}
```

#### B. バリデーション更新
```rust
pub fn validate_farm_space_level(level: u8, config: &FarmLevelConfig) -> Result<()> {
    require!(
        level >= 1 && level <= config.max_level,
        GameError::InvalidFarmLevel
    );
    Ok(())
}
```

### 3. 命令セット追加

#### A. レベル設定更新命令
```rust
// 新しい instruction
pub fn update_farm_level_config(
    ctx: Context<UpdateFarmLevelConfig>,
    max_level: u8,
    capacities: Vec<u8>,
    upgrade_thresholds: Vec<u32>,
) -> Result<()> {
    let config = &mut ctx.accounts.farm_level_config;
    
    // バリデーション
    require!(max_level <= MAX_POSSIBLE_LEVELS, GameError::InvalidConfig);
    require!(capacities.len() == max_level as usize, GameError::InvalidConfig);
    require!(upgrade_thresholds.len() == max_level as usize, GameError::InvalidConfig);
    
    // 昇順チェック
    for i in 1..capacities.len() {
        require!(capacities[i] > capacities[i-1], GameError::InvalidConfig);
    }
    for i in 1..upgrade_thresholds.len() {
        require!(upgrade_thresholds[i] > upgrade_thresholds[i-1], GameError::InvalidConfig);
    }
    
    // 更新
    config.max_level = max_level;
    config.capacities = capacities;
    config.upgrade_thresholds = upgrade_thresholds;
    
    msg!("Farm level config updated: max_level={}", max_level);
    Ok(())
}
```

#### B. 既存ファームの移行命令
```rust
pub fn migrate_farm_to_new_levels(
    ctx: Context<MigrateFarm>,
) -> Result<()> {
    let farm = &mut ctx.accounts.farm_space;
    let config = &ctx.accounts.farm_level_config;
    
    // 現在のレベルが新しい最大レベルを超えている場合の処理
    if farm.level > config.max_level {
        farm.level = config.max_level;
    }
    
    // 容量の再計算
    farm.capacity = FarmSpace::get_capacity_for_level(farm.level, config);
    
    msg!("Farm migrated: level={}, capacity={}", farm.level, farm.capacity);
    Ok(())
}
```

### 4. 段階的実装計画

#### Phase 1: 基盤整備
1. FarmLevelConfig アカウント追加
2. 既存の定数をデフォルト値として保持
3. 新しいヘルパー関数の実装

#### Phase 2: 動的化
1. レベル関連メソッドをconfig参照に変更
2. バリデーション関数の更新
3. テストの追加

#### Phase 3: 運用対応
1. 管理者用レベル設定更新命令
2. 既存ファームの移行ツール
3. フロントエンド対応

### 5. 互換性維持

```rust
// 後方互換性のためのラッパー
impl FarmSpace {
    // 既存のコード用
    pub fn get_capacity_for_level_legacy(level: u8) -> u8 {
        if level == 0 || level > 5 {
            return DEFAULT_FARM_CAPACITIES[4];
        }
        DEFAULT_FARM_CAPACITIES[(level - 1) as usize]
    }
}
```

### 6. 使用例

```rust
// レベル10まで拡張
let capacities = vec![4, 6, 8, 10, 12, 15, 18, 22, 26, 30];
let thresholds = vec![0, 30, 100, 300, 500, 1000, 2000, 3500, 5000, 7500];

update_farm_level_config(
    ctx,
    10, // max_level
    capacities,
    thresholds,
)?;
```

## メリット

1. **柔軟性**: レベル数を動的に変更可能
2. **互換性**: 既存のファームはそのまま動作
3. **拡張性**: 将来的なレベル追加が容易
4. **管理性**: 運営側でレベル設定を調整可能

## 注意点

1. **アカウントサイズ**: Vecを使うため、適切なspace計算が必要
2. **移行処理**: 既存ユーザーへの影響を最小化
3. **テスト**: 全レベルでの動作確認が必要