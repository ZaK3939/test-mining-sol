# 確率テーブル管理システム

## 概要

Facility Gameの確率テーブルシステムは、シードパックから出現するシードの種類と確率を動的に管理する核心的なコンポーネントです。管理者はゲームバランスの調整、新しいシードの追加、イベント用の特別確率設定を柔軟に実行できます。

## 確率テーブルの構造

### データ構造

```rust
#[account]
pub struct ProbabilityTable {
    /// テーブルバージョン（更新追跡用）
    pub version: u32,
    
    /// 有効なシード種類数（1-9）
    pub seed_count: u8,
    
    /// 各シードのGrow Power値（最大9種類）
    pub grow_powers: [u64; 9],
    
    /// 累積確率閾値（10000基準）
    pub probability_thresholds: [u16; 9],
    
    /// 表示用確率パーセント
    pub probability_percentages: [f32; 9],
    
    /// パック期待値（経済バランス用）
    pub expected_value: u64,
    
    /// テーブル名（32バイト）
    pub name: [u8; 32],
    
    /// 作成・更新タイムスタンプ
    pub created_at: i64,
    pub updated_at: i64,
    
    /// 将来の拡張用
    pub reserve: [u8; 32],
}
```

### 現在の確率テーブル設定

#### Table 1（デフォルト：6種類）
| シード | Grow Power | 確率 | 累積確率 | 閾値 |
|--------|------------|------|----------|------|
| Seed1  | 100        | 43.0% | 43.0%    | 4300 |
| Seed2  | 180        | 25.0% | 68.0%    | 6800 |
| Seed3  | 420        | 14.0% | 82.0%    | 8200 |
| Seed4  | 720        | 9.0%  | 91.0%    | 9100 |
| Seed5  | 1,000      | 6.0%  | 97.0%    | 9700 |
| Seed6  | 5,000      | 3.0%  | 100.0%   | 10000|

**期待値**: 421 Grow Power

#### Table 2（拡張版：9種類）
| シード | Grow Power | 確率 | 累積確率 | 閾値 |
|--------|------------|------|----------|------|
| Seed1  | 100        | 42.23% | 42.23%  | 4223 |
| Seed2  | 180        | 24.44% | 66.67%  | 6667 |
| Seed3  | 420        | 13.33% | 80.00%  | 8000 |
| Seed4  | 720        | 8.33%  | 88.33%  | 8833 |
| Seed5  | 1,000      | 5.56%  | 93.89%  | 9389 |
| Seed6  | 5,000      | 3.33%  | 97.22%  | 9722 |
| Seed7  | 15,000     | 1.33%  | 98.55%  | 9855 |
| Seed8  | 30,000     | 0.89%  | 99.44%  | 9944 |
| Seed9  | 60,000     | 0.56%  | 100.00% | 10000|

**期待値**: 1,089 Grow Power

## 確率テーブル管理

### 初期化

```rust
pub fn initialize_probability_table(ctx: Context<InitializeProbabilityTable>) -> Result<()> {
    let table = &mut ctx.accounts.probability_table;
    
    // Table 1のデフォルト設定
    *table = ProbabilityTable::init_table_1();
    table.created_at = Clock::get()?.unix_timestamp;
    table.updated_at = table.created_at;
    
    msg!("Probability table initialized: {}", 
         String::from_utf8_lossy(&table.name));
    
    Ok(())
}
```

### 動的更新

```rust
pub fn update_probability_table(
    ctx: Context<UpdateProbabilityTable>,
    version: u32,
    seed_count: u8,
    grow_powers: Vec<u64>,
    probability_thresholds: Vec<u16>,
    probability_percentages: Vec<f32>,
    expected_value: u64,
    name: String,
) -> Result<()> {
    let table = &mut ctx.accounts.probability_table;
    
    // バージョン検証
    require!(version > table.version, GameError::InvalidVersion);
    
    // データ整合性検証
    validate_probability_data(
        seed_count,
        &grow_powers,
        &probability_thresholds,
        &probability_percentages
    )?;
    
    // テーブル更新
    table.version = version;
    table.seed_count = seed_count;
    table.expected_value = expected_value;
    table.updated_at = Clock::get()?.unix_timestamp;
    
    // 配列更新
    update_table_arrays(table, &grow_powers, &probability_thresholds, &probability_percentages)?;
    
    // 名前設定
    let name_bytes = name.as_bytes();
    let copy_len = std::cmp::min(name_bytes.len(), 32);
    table.name[0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
    
    msg!("Probability table updated to version {}", version);
    
    Ok(())
}
```

### 検証ロジック

```rust
fn validate_probability_data(
    seed_count: u8,
    grow_powers: &[u64],
    probability_thresholds: &[u16],
    probability_percentages: &[f32],
) -> Result<()> {
    // 基本範囲チェック
    require!(seed_count >= 1 && seed_count <= 9, GameError::InvalidSeedCount);
    require!(grow_powers.len() >= seed_count as usize, GameError::InsufficientGrowPowers);
    require!(probability_thresholds.len() >= seed_count as usize, GameError::InsufficientThresholds);
    
    // Grow Power値検証
    for (i, &gp) in grow_powers.iter().take(seed_count as usize).enumerate() {
        require!(gp > 0, GameError::ZeroGrowPower);
        
        // 昇順チェック（通常は低いGPから高いGPへ）
        if i > 0 {
            require!(gp >= grow_powers[i-1], GameError::InvalidGrowPowerOrder);
        }
    }
    
    // 累積確率検証
    let mut prev_threshold = 0u16;
    for (i, &threshold) in probability_thresholds.iter().take(seed_count as usize).enumerate() {
        require!(threshold > prev_threshold, GameError::InvalidProbabilityOrder);
        require!(threshold <= 10000, GameError::ProbabilityExceedsMax);
        prev_threshold = threshold;
    }
    
    // 最後の閾値は必ず10000
    require!(
        probability_thresholds[seed_count as usize - 1] == 10000,
        GameError::FinalProbabilityNotComplete
    );
    
    // 確率パーセント合計検証
    let total_percentage: f32 = probability_percentages
        .iter()
        .take(seed_count as usize)
        .sum();
    require!(
        (total_percentage - 100.0).abs() < 0.01, // 誤差許容
        GameError::ProbabilityPercentageMismatch
    );
    
    Ok(())
}
```

## 確率計算メカニズム

### ランダム値生成

```rust
pub fn determine_seed_from_random(table: &ProbabilityTable, random_value: u64) -> SeedType {
    // 0-9999の範囲に正規化
    let normalized_value = (random_value % 10000) as u16;
    
    // 線形探索（9要素なので高速）
    for (i, &threshold) in table.get_active_thresholds().iter().enumerate() {
        if normalized_value < threshold {
            return SeedType::from_index(i as u8).unwrap();
        }
    }
    
    // フォールバック（通常ここには到達しない）
    SeedType::from_index((table.seed_count - 1) as u8).unwrap()
}
```

### Switchboard VRF統合

```rust
pub fn open_seed_pack(ctx: Context<OpenSeedPack>, quantity: u8) -> Result<()> {
    let seed_pack = &mut ctx.accounts.seed_pack;
    let vrf_account = &ctx.accounts.vrf_account;
    let probability_table = &ctx.accounts.probability_table;
    
    // VRF結果取得
    let vrf_result = vrf_account.get_result()
        .map_err(|_| GameError::VrfResultNotAvailable)?;
    
    // 複数の乱数源を組み合わせ
    let combined_entropy = combine_entropy_sources(
        &vrf_result,
        seed_pack.user_entropy_seed,
        Clock::get()?.unix_timestamp as u64,
        seed_pack.pack_id,
    );
    
    // 各パックに対してシード生成
    for i in 0..quantity {
        let pack_random = hash_with_index(combined_entropy, i as u64);
        let seed_type = determine_seed_from_random(probability_table, pack_random);
        
        // シードアカウント作成
        create_seed_account(ctx, seed_type, pack_random)?;
    }
    
    seed_pack.is_opened = true;
    seed_pack.final_random_value = combined_entropy;
    
    Ok(())
}

fn combine_entropy_sources(
    vrf_result: &[u8],
    user_entropy: u64,
    timestamp: u64,
    pack_id: u64,
) -> u64 {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(vrf_result);
    hasher.update(&user_entropy.to_le_bytes());
    hasher.update(&timestamp.to_le_bytes());
    hasher.update(&pack_id.to_le_bytes());
    
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[0..8].try_into().unwrap())
}
```

## 経済分析と調整

### 期待値計算

```rust
impl ProbabilityTable {
    pub fn calculate_expected_value(&self) -> u64 {
        let mut expected_value = 0u64;
        let mut prev_threshold = 0u16;
        
        for i in 0..self.seed_count as usize {
            let probability = (self.probability_thresholds[i] - prev_threshold) as f64 / 10000.0;
            let weighted_value = self.grow_powers[i] as f64 * probability;
            expected_value += weighted_value as u64;
            prev_threshold = self.probability_thresholds[i];
        }
        
        expected_value
    }
    
    pub fn calculate_variance(&self) -> f64 {
        let expected_value = self.calculate_expected_value() as f64;
        let mut variance = 0.0;
        let mut prev_threshold = 0u16;
        
        for i in 0..self.seed_count as usize {
            let probability = (self.probability_thresholds[i] - prev_threshold) as f64 / 10000.0;
            let deviation = self.grow_powers[i] as f64 - expected_value;
            variance += probability * deviation * deviation;
            prev_threshold = self.probability_thresholds[i];
        }
        
        variance
    }
    
    pub fn calculate_standard_deviation(&self) -> f64 {
        self.calculate_variance().sqrt()
    }
}
```

### バランス分析ツール

```typescript
// フロントエンド分析ツール
class ProbabilityAnalyzer {
  analyzeTable(table: ProbabilityTable): TableAnalysis {
    const expectedValue = this.calculateExpectedValue(table);
    const variance = this.calculateVariance(table);
    const standardDeviation = Math.sqrt(variance);
    const giniCoefficient = this.calculateGiniCoefficient(table);
    
    return {
      expectedValue,
      variance,
      standardDeviation,
      giniCoefficient,
      economicImpact: this.assessEconomicImpact(table),
      recommendations: this.generateRecommendations(table),
      comparisonWithCurrent: this.compareWithCurrentTable(table),
    };
  }
  
  private assessEconomicImpact(table: ProbabilityTable): EconomicImpact {
    const currentTable = this.getCurrentTable();
    const valueChange = this.calculateExpectedValue(table) - 
                       this.calculateExpectedValue(currentTable);
    
    const inflationImpact = valueChange / this.calculateExpectedValue(currentTable);
    const recommendedPackCostAdjustment = this.calculateOptimalPackCost(table);
    
    return {
      expectedValueChange: valueChange,
      inflationImpact,
      recommendedPackCost: recommendedPackCostAdjustment,
      playerSegmentImpact: this.analyzePlayerSegmentImpact(table),
    };
  }
  
  private generateRecommendations(table: ProbabilityTable): string[] {
    const recommendations: string[] = [];
    
    // 期待値チェック
    if (this.calculateExpectedValue(table) > 500) {
      recommendations.push("期待値が高すぎます。パックコストの調整を検討してください。");
    }
    
    // 分散チェック
    if (this.calculateVariance(table) > 1000000) {
      recommendations.push("分散が大きすぎます。中間レアリティのシードを増やすことを検討してください。");
    }
    
    // ジニ係数チェック
    const gini = this.calculateGiniCoefficient(table);
    if (gini > 0.8) {
      recommendations.push("不平等度が高すぎます。確率分布の再調整を推奨します。");
    }
    
    return recommendations;
  }
}
```

## 特別イベント用確率テーブル

### 期間限定テーブル

```rust
#[account]
pub struct EventProbabilityTable {
    pub base_table_version: u32,
    pub event_id: u64,
    pub event_name: [u8; 32],
    pub start_time: i64,
    pub end_time: i64,
    pub multipliers: [f32; 9],          // シード別確率倍率
    pub bonus_seeds: Vec<BonusSeed>,    // イベント限定シード
    pub is_active: bool,
    pub usage_count: u32,               // 使用回数追跡
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BonusSeed {
    pub seed_type: SeedType,
    pub bonus_grow_power: u64,
    pub probability: u16,               // 追加確率
    pub max_drops: u32,                 // 最大出現数
    pub current_drops: u32,             // 現在の出現数
}
```

### イベントテーブルの適用

```rust
pub fn apply_event_modifiers(
    base_table: &ProbabilityTable,
    event_table: &EventProbabilityTable,
    random_value: u64,
) -> Result<SeedType> {
    // イベント期間チェック
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time >= event_table.start_time && 
        current_time <= event_table.end_time,
        GameError::EventNotActive
    );
    
    // ボーナスシードチェック（優先処理）
    let bonus_roll = (random_value >> 32) % 10000;
    for bonus_seed in &event_table.bonus_seeds {
        if bonus_seed.current_drops < bonus_seed.max_drops &&
           bonus_roll < bonus_seed.probability {
            // ボーナスシード出現
            return Ok(bonus_seed.seed_type);
        }
    }
    
    // 通常テーブルに倍率適用
    let modified_table = apply_multipliers(base_table, &event_table.multipliers);
    determine_seed_from_random(&modified_table, random_value)
}
```

## 管理ツールとモニタリング

### 確率検証システム

```typescript
class ProbabilityVerifier {
  async verifyTableIntegrity(tableAddress: string): Promise<VerificationResult> {
    const table = await this.program.account.probabilityTable.fetch(tableAddress);
    
    const checks = [
      this.verifyProbabilitySum(table),
      this.verifyGrowPowerOrder(table),
      this.verifyThresholdOrder(table),
      this.verifyExpectedValue(table),
    ];
    
    const results = await Promise.all(checks);
    const allPassed = results.every(r => r.passed);
    
    return {
      passed: allPassed,
      checks: results,
      recommendations: this.generateFixes(results.filter(r => !r.passed))
    };
  }
  
  private verifyExpectedValue(table: ProbabilityTable): VerificationCheck {
    const calculated = this.calculateExpectedValue(table);
    const stored = table.expectedValue;
    const tolerance = 1; // 1 GP tolerance
    
    return {
      name: "Expected Value Verification",
      passed: Math.abs(calculated - stored) <= tolerance,
      details: `Calculated: ${calculated}, Stored: ${stored}`,
      severity: Math.abs(calculated - stored) > tolerance ? 'error' : 'warning'
    };
  }
}
```

### リアルタイム統計

```rust
#[account]
pub struct ProbabilityStats {
    pub table_version: u32,
    pub total_packs_opened: u64,
    pub seed_counts: [u64; 9],          // 実際の出現回数
    pub observed_probabilities: [f32; 9], // 観測確率
    pub chi_square_statistic: f64,       // 適合度検定統計量
    pub last_updated: i64,
    pub reserve: [u8; 32],
}

pub fn update_probability_stats(
    ctx: Context<UpdateProbabilityStats>,
    seed_type: SeedType,
) -> Result<()> {
    let stats = &mut ctx.accounts.probability_stats;
    
    // 出現回数更新
    stats.seed_counts[seed_type as usize] += 1;
    stats.total_packs_opened += 1;
    
    // 観測確率計算
    for i in 0..9 {
        stats.observed_probabilities[i] = 
            stats.seed_counts[i] as f32 / stats.total_packs_opened as f32;
    }
    
    // カイ二乗統計量計算（適合度検定）
    stats.chi_square_statistic = calculate_chi_square(
        &stats.observed_probabilities,
        &ctx.accounts.probability_table.probability_percentages
    );
    
    stats.last_updated = Clock::get()?.unix_timestamp;
    
    Ok(())
}
```

## テーブル移行戦略

### 段階的移行

```rust
pub fn migrate_to_new_table(
    ctx: Context<MigrateTable>,
    migration_percentage: u8, // 0-100
) -> Result<()> {
    require!(migration_percentage <= 100, GameError::InvalidMigrationPercentage);
    
    let user_hash = hash_user_pubkey(ctx.accounts.user.key());
    let migration_threshold = (migration_percentage as u64 * 256) / 100;
    
    let table_to_use = if (user_hash % 256) < migration_threshold {
        &ctx.accounts.new_table
    } else {
        &ctx.accounts.current_table
    };
    
    // 選択されたテーブルで処理続行
    process_with_table(ctx, table_to_use)
}
```

### A/Bテスト機能

```typescript
class ABTestManager {
  async setupABTest(
    testName: string,
    tableA: ProbabilityTable,
    tableB: ProbabilityTable,
    splitPercentage: number = 50
  ): Promise<string> {
    const testId = generateTestId();
    
    await this.program.methods
      .setupAbTest(testId, testName, splitPercentage)
      .accounts({
        tableA: tableA.address,
        tableB: tableB.address,
      })
      .rpc();
    
    return testId;
  }
  
  async analyzeABTest(testId: string): Promise<ABTestResults> {
    const stats = await this.fetchABTestStats(testId);
    
    return {
      statistical_significance: this.calculateStatisticalSignificance(stats),
      conversion_rates: this.calculateConversionRates(stats),
      revenue_impact: this.calculateRevenueImpact(stats),
      recommendation: this.generateRecommendation(stats)
    };
  }
}
```

この確率テーブル管理システムにより、Facility Gameは柔軟で透明性の高いゲームバランス調整を実現し、プレイヤーに公正で楽しいゲーム体験を提供できます。