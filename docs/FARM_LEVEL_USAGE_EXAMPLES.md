# FarmSpace レベル拡張 使用例

**実装場所**: `src/instructions/farm.rs`に全機能が統合されています。

## 基本的な使用フロー

### 1. システム初期化（管理者）

```typescript
// デフォルト5レベルでFarmLevelConfigを初期化
await program.methods
  .initializeFarmLevelConfig()
  .accounts({
    farmLevelConfig: farmLevelConfigPDA,
    config: configPDA,
    admin: adminKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();
```

### 2. レベル10まで拡張（管理者）

```typescript
// レベル10まで拡張
const capacities = [4, 6, 8, 10, 12, 15, 18, 22, 26, 30];
const thresholds = [0, 30, 100, 300, 500, 1000, 2000, 3500, 5000, 7500];
const levelNames = [
  "Starter Farm", "Growing Farm", "Expanding Farm", "Advanced Farm", "Master Farm",
  "Expert Farm", "Elite Farm", "Premium Farm", "Ultimate Farm", "Legendary Farm"
];

await program.methods
  .updateFarmLevelConfig(10, capacities, thresholds, levelNames)
  .accounts({
    farmLevelConfig: farmLevelConfigPDA,
    config: configPDA,
    admin: adminKeypair.publicKey,
  })
  .signers([adminKeypair])
  .rpc();
```

### 3. 既存ファームの移行（ユーザー）

```typescript
// レベル設定変更後、既存ファームを新しい設定に移行
await program.methods
  .migrateFarmToNewLevels()
  .accounts({
    farmSpace: farmSpacePDA,
    farmLevelConfig: farmLevelConfigPDA,
    user: userKeypair.publicKey,
  })
  .signers([userKeypair])
  .rpc();
```

## 実装上の重要なポイント

### 1. アカウントサイズ計算

```typescript
// レベル設定に応じた動的サイズ計算
function calculateFarmLevelConfigSpace(maxLevel: number, levelNames: string[]): number {
  const baseSpace = 8 + 1 + 4 + 4 + 4 + 8 + 8 + 32; // BASE_LEN
  const capacitiesSpace = maxLevel;
  const thresholdsSpace = maxLevel * 4;
  const namesSpace = levelNames.reduce((sum, name) => sum + 4 + name.length, 0);
  
  return baseSpace + capacitiesSpace + thresholdsSpace + namesSpace;
}
```

### 2. レベル情報取得

```typescript
// 特定レベルの情報取得
const levelInfo = await program.methods
  .getFarmLevelInfo(5) // レベル5の情報
  .accounts({
    farmLevelConfig: farmLevelConfigPDA,
  })
  .view();

console.log(`Level ${levelInfo.level}: ${levelInfo.name}`);
console.log(`Capacity: ${levelInfo.capacity}, Threshold: ${levelInfo.upgrade_threshold}`);
```

### 3. 互換性維持

```rust
// 既存コードとの互換性を保つラッパー関数
impl FarmSpace {
    pub fn get_capacity_for_level(level: u8) -> u8 {
        // レガシー関数（既存のコードで使用）
        crate::constants::FARM_CAPACITIES.get(level.saturating_sub(1) as usize)
            .copied()
            .unwrap_or(12) // デフォルトは最大容量
    }
    
    pub fn get_capacity_for_level_with_config(level: u8, config: &FarmLevelConfig) -> Result<u8> {
        // 新しい動的関数
        require!(level >= 1 && level <= config.max_level, GameError::InvalidFarmLevel);
        Ok(config.capacities[(level - 1) as usize])
    }
}
```

## レベル拡張例

### シナリオ1: レベル7まで拡張

```typescript
const newConfig = {
  maxLevel: 7,
  capacities: [4, 6, 8, 10, 12, 15, 20],
  thresholds: [0, 30, 100, 300, 500, 1000, 2000],
  levelNames: [
    "Beginner", "Novice", "Apprentice", "Skilled", 
    "Expert", "Master", "Grandmaster"
  ]
};
```

### シナリオ2: 大規模拡張（レベル15）

```typescript
const extensiveConfig = {
  maxLevel: 15,
  capacities: [4, 6, 8, 10, 12, 15, 18, 22, 26, 30, 35, 40, 45, 50, 60],
  thresholds: [
    0, 30, 100, 300, 500,           // 初期5レベル
    1000, 2000, 3500, 5000, 7500,  // 中級5レベル
    10000, 15000, 20000, 30000, 50000 // 上級5レベル
  ]
};
```

## 注意事項とベストプラクティス

### 1. 段階的移行

```typescript
// 一度に大幅な変更は避け、段階的に拡張
// 例: 5レベル → 7レベル → 10レベル
```

### 2. ユーザー通知

```typescript
// レベル変更前にユーザーに通知
// 移行期間を設けて、ユーザーが対応できるようにする
```

### 3. テスト

```typescript
// 新しいレベル設定のテスト
describe("Farm Level Extension", () => {
  it("should handle level 10 upgrade correctly", async () => {
    // テストロジック
  });
  
  it("should migrate existing farms without data loss", async () => {
    // 移行テストロジック
  });
});
```

### 4. 失敗時の対策

```typescript
// 移行失敗時のロールバック計画
// バックアップデータの保持
// エラーハンドリングの充実
```

このシステムにより、FarmSpaceのレベルを柔軟に拡張でき、既存のユーザーに影響を与えることなく新機能を追加できます。