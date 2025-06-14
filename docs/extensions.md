# システム拡張ガイド

## 概要

Facility Gameは拡張性を重視した設計になっており、新機能追加、パラメータ調整、新システム統合を安全かつ効率的に実行できます。本ドキュメントでは、主要な拡張パターンとその実装方法を詳しく解説します。

## 1. 新シード種類の追加

### 現在の制限と拡張可能性

**現在の状況**:
- Table 1: 6種類のシード（Seed1-Seed6）
- Table 2対応済み: 9種類のシード（Seed1-Seed9）
- 最大9種類まで拡張可能

### 新シード追加手順

#### Step 1: 確率テーブル更新
```rust
// 新しいTable 2に更新
let new_table = ProbabilityTableData {
    version: 2,
    seed_count: 9,
    grow_powers: [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000],
    probability_thresholds: [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000],
    probability_percentages: [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56],
    expected_value: 1089, // 新期待値
    name: "Table2".as_bytes(),
};

await program.methods
    .updateProbabilityTable(new_table)
    .accounts({ admin: admin.publicKey })
    .rpc();
```

#### Step 2: フロントエンド対応
```typescript
// types/program-types.ts に追加
export enum SeedType {
  Seed1 = 'Seed1',
  Seed2 = 'Seed2', 
  Seed3 = 'Seed3',
  Seed4 = 'Seed4',
  Seed5 = 'Seed5',
  Seed6 = 'Seed6',
  Seed7 = 'Seed7',  // 新追加
  Seed8 = 'Seed8',  // 新追加
  Seed9 = 'Seed9',  // 新追加
}

// シード表示情報更新
const SEED_INFO = {
  Seed7: { name: "レジェンダリーシード", color: "#FF6B35", rarity: "★★★" },
  Seed8: { name: "ミシックシード", color: "#9B59B6", rarity: "★★★★" },
  Seed9: { name: "ディバインシード", color: "#F39C12", rarity: "★★★★★" },
};
```

#### Step 3: UI/UX更新
```typescript
// シード表示コンポーネント更新
function SeedCard({ seed }: { seed: SeedType }) {
  const info = SEED_INFO[seed];
  const isNewSeed = [SeedType.Seed7, SeedType.Seed8, SeedType.Seed9].includes(seed);
  
  return (
    <div className={`seed-card ${isNewSeed ? 'new-seed' : ''}`}>
      <div className="seed-rarity">{info.rarity}</div>
      <div className="seed-name">{info.name}</div>
      <div className="grow-power">{seed.getGrowPower()} GP</div>
      {isNewSeed && <div className="new-badge">NEW!</div>}
    </div>
  );
}
```

### バランス調整の考慮点

```typescript
// 新シードの経済影響分析
class EconomicImpactAnalyzer {
  calculateExpectedValue(table: ProbabilityTable): number {
    return table.grow_powers
      .slice(0, table.seed_count)
      .reduce((total, gp, index) => {
        const probability = this.getProbability(table, index);
        return total + (gp * probability);
      }, 0);
  }
  
  analyzeInflationImpact(oldTable: ProbabilityTable, newTable: ProbabilityTable): {
    expectedValueChange: number;
    inflationRate: number;
    recommendedPackCostAdjustment: number;
  } {
    const oldEV = this.calculateExpectedValue(oldTable);
    const newEV = this.calculateExpectedValue(newTable);
    const inflationRate = (newEV - oldEV) / oldEV;
    
    return {
      expectedValueChange: newEV - oldEV,
      inflationRate,
      recommendedPackCostAdjustment: Math.ceil(300 * (1 + inflationRate))
    };
  }
}
```

## 2. 農場レベル拡張

### 動的レベル設定の活用

現在の実装は動的農場レベル設定をサポートしており、最大20レベルまで拡張可能です。

#### Step 1: 新レベル設定の作成
```rust
// 新しい農場レベル設定（レベル1-10）
let extended_config = FarmLevelConfigData {
    max_level: 10,
    capacities: vec![4, 6, 8, 10, 12, 15, 18, 22, 26, 30],
    upgrade_thresholds: vec![0, 30, 100, 300, 500, 1000, 2000, 4000, 8000, 15000],
    level_names: vec![
        "初心者農場".to_string(),
        "成長農場".to_string(),
        "発展農場".to_string(),
        "上級農場".to_string(),
        "マスター農場".to_string(),
        "エリート農場".to_string(),     // 新追加
        "レジェンド農場".to_string(),   // 新追加
        "ミシック農場".to_string(),     // 新追加
        "アセンデッド農場".to_string(), // 新追加
        "ディバイン農場".to_string(),   // 新追加
    ],
    created_at: Clock::get()?.unix_timestamp,
    updated_at: Clock::get()?.unix_timestamp,
    reserve: [0; 32],
};
```

#### Step 2: 既存農場の移行
```rust
pub fn migrate_farm_to_new_levels(ctx: Context<MigrateFarm>) -> Result<()> {
    let farm_space = &mut ctx.accounts.farm_space;
    let user_state = &ctx.accounts.user_state;
    let config = &ctx.accounts.farm_level_config;
    
    // 現在のパック購入数に基づいて新レベル計算
    let new_level = FarmSpace::calculate_level_from_packs_with_config(
        user_state.total_packs_purchased, 
        config
    )?;
    
    if new_level > farm_space.level {
        farm_space.level = new_level;
        farm_space.capacity = FarmSpace::get_capacity_for_level_with_config(new_level, config)?;
        
        msg!("Farm upgraded from level {} to level {}", farm_space.level, new_level);
    }
    
    Ok(())
}
```

#### Step 3: UI対応
```typescript
// 新レベルの視覚表現
function FarmLevelDisplay({ level }: { level: number }) {
  const getLevelTheme = (level: number) => {
    if (level <= 5) return { color: '#4CAF50', tier: 'Standard' };
    if (level <= 7) return { color: '#FF9800', tier: 'Elite' };
    if (level <= 9) return { color: '#9C27B0', tier: 'Legendary' };
    return { color: '#FFD700', tier: 'Divine' };
  };
  
  const theme = getLevelTheme(level);
  
  return (
    <div className="farm-level" style={{ borderColor: theme.color }}>
      <div className="level-number">{level}</div>
      <div className="level-tier">{theme.tier}</div>
      <div className="level-name">{FARM_LEVEL_NAMES[level - 1]}</div>
    </div>
  );
}
```

## 3. 新ゲームメカニクスの追加

### シード進化システム

#### コンセプト設計
```rust
#[account]
pub struct SeedEvolution {
    pub base_seed_id: u64,
    pub evolution_level: u8,      // 0-3 (base, +1, +2, +3)
    pub required_materials: Vec<u64>,  // 必要な素材シードID
    pub evolution_cost: u64,      // 進化コスト（WEED）
    pub boosted_grow_power: u64,  // 強化後のGrow Power
    pub special_abilities: Vec<SpecialAbility>,
    pub created_at: i64,
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum SpecialAbility {
    DoubleReward,       // 報酬2倍
    HalvingResistance,  // 半減期無効
    AutoHarvest,        // 自動収穫
    BonusLuck,          // パック開封運UP
}
```

#### 実装手順
```rust
pub fn evolve_seed(ctx: Context<EvolveSeed>, material_seed_ids: Vec<u64>) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let base_seed = &mut ctx.accounts.base_seed;
    let evolution_config = &ctx.accounts.evolution_config;
    
    // 素材シードの所有権と条件チェック
    validate_evolution_materials(&material_seed_ids, &ctx.accounts.user_seeds)?;
    
    // 進化コスト支払い
    let evolution_cost = calculate_evolution_cost(base_seed.seed_type, base_seed.evolution_level + 1);
    validate_sufficient_balance(&ctx.accounts.user_token_account, evolution_cost)?;
    
    // 進化実行
    base_seed.evolution_level += 1;
    base_seed.grow_power = calculate_evolved_grow_power(base_seed);
    base_seed.special_abilities = assign_special_abilities(base_seed);
    
    // 素材シード消費
    consume_material_seeds(material_seed_ids)?;
    
    // コスト支払い
    transfer_tokens_to_treasury(&ctx.accounts, evolution_cost)?;
    
    msg!("Seed evolved to level {}", base_seed.evolution_level);
    Ok(())
}
```

### 季節イベントシステム

#### 動的イベント管理
```rust
#[account]
pub struct SeasonalEvent {
    pub event_id: u64,
    pub event_type: EventType,
    pub start_time: i64,
    pub end_time: i64,
    pub is_active: bool,
    pub event_config: EventConfig,
    pub participant_count: u32,
    pub rewards_distributed: u64,
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum EventType {
    DoubleRewards,          // 報酬2倍期間
    BonusProbability,       // 高レア確率UP
    FreeSeeds,              // 無料シード配布
    CommunityChallenge,     // コミュニティチャレンジ
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EventConfig {
    pub multiplier: f64,           // 効果倍率
    pub affected_seed_types: Vec<SeedType>,  // 対象シード
    pub special_rewards: Vec<SpecialReward>, // 特別報酬
    pub participation_requirements: ParticipationRequirements,
}
```

## 4. 新トークン統合

### マルチトークンサポート

#### セカンダリトークンの追加
```rust
#[account]
pub struct SecondaryToken {
    pub token_mint: Pubkey,
    pub token_name: [u8; 32],
    pub exchange_rate: u64,        // WEEDとの交換レート
    pub utility_functions: Vec<TokenUtility>,
    pub total_supply: u64,
    pub is_active: bool,
    pub created_at: i64,
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TokenUtility {
    FarmUpgrade,        // 農場アップグレード
    SeedEvolution,      // シード進化
    SpecialEvents,      // 特別イベント参加
    Governance,         // ガバナンス投票
}
```

#### クロスチェーン対応
```typescript
// Wormhole統合準備
class CrossChainBridge {
  async bridgeToEthereum(amount: number, targetAddress: string) {
    const message = {
      tokenAddress: WEED_MINT_ADDRESS,
      amount: amount,
      targetChain: 'ethereum',
      targetAddress: targetAddress,
      nonce: generateNonce()
    };
    
    return await wormhole.publishMessage(message);
  }
  
  async bridgeFromEthereum(signedVAA: Buffer) {
    const parsed = await wormhole.parseTransferMessage(signedVAA);
    
    // WEED tokens equivalent mint on Solana
    await this.program.methods
      .crossChainMint(parsed.amount, parsed.targetAddress)
      .rpc();
  }
}
```

## 5. ガバナンス機能の追加

### DAO統合設計

#### 投票システム
```rust
#[account]
pub struct Proposal {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub description: [u8; 256],
    pub voting_start: i64,
    pub voting_end: i64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub min_votes_required: u64,
    pub is_executed: bool,
    pub execution_data: Vec<u8>,
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ProposalType {
    ConfigUpdate,       // 設定変更
    ProbabilityUpdate,  // 確率テーブル変更
    NewFeature,         // 新機能追加
    EmergencyPause,     // 緊急停止
}

pub fn vote_on_proposal(ctx: Context<Vote>, proposal_id: u64, vote: bool) -> Result<()> {
    let voter = &ctx.accounts.voter;
    let proposal = &mut ctx.accounts.proposal;
    let user_state = &ctx.accounts.user_state;
    
    // 投票権重計算（Grow Powerベース）
    let voting_power = calculate_voting_power(user_state.total_grow_power);
    
    // 投票記録
    if vote {
        proposal.votes_for += voting_power;
    } else {
        proposal.votes_against += voting_power;
    }
    
    // 投票完了チェック
    if is_voting_complete(proposal) {
        execute_proposal_if_passed(proposal)?;
    }
    
    Ok(())
}
```

#### 投票権重計算
```rust
fn calculate_voting_power(grow_power: u64) -> u64 {
    // Square root voting to prevent whale dominance
    let sqrt_power = (grow_power as f64).sqrt() as u64;
    
    // Cap maximum voting power
    std::cmp::min(sqrt_power, 10000)
}
```

## 6. パフォーマンス最適化

### バッチ処理機能

#### 大量操作の最適化
```rust
pub fn batch_plant_seeds(ctx: Context<BatchPlantSeeds>, seed_ids: Vec<u64>) -> Result<()> {
    require!(seed_ids.len() <= 10, GameError::BatchSizeExceeded);
    
    let farm_space = &mut ctx.accounts.farm_space;
    let user_state = &mut ctx.accounts.user_state;
    
    // バッチ検証
    let total_capacity_needed = seed_ids.len() as u8;
    require!(
        farm_space.seed_count + total_capacity_needed <= farm_space.capacity,
        GameError::FarmAtMaxCapacity
    );
    
    let mut total_grow_power_added = 0u64;
    
    // バッチ実行
    for seed_id in seed_ids {
        let seed = validate_and_get_seed(&ctx.accounts, seed_id)?;
        require!(!seed.is_planted, GameError::SeedAlreadyPlanted);
        
        total_grow_power_added += seed.grow_power;
        seed.is_planted = true;
        seed.planted_farm_space = Some(farm_space.key());
    }
    
    // 一括更新
    farm_space.seed_count += total_capacity_needed;
    farm_space.total_grow_power += total_grow_power_added;
    user_state.total_grow_power += total_grow_power_added;
    
    Ok(())
}
```

### キャッシュ戦略
```typescript
// フロントエンド最適化
class GameStateCache {
  private cache = new Map<string, CacheEntry>();
  private readonly TTL = 30000; // 30秒
  
  async getGameState(userPubkey: string): Promise<GameState> {
    const cacheKey = `gamestate_${userPubkey}`;
    const cached = this.cache.get(cacheKey);
    
    if (cached && Date.now() - cached.timestamp < this.TTL) {
      return cached.data;
    }
    
    // Batch fetch for better performance
    const gameState = await this.batchFetchGameState(userPubkey);
    
    this.cache.set(cacheKey, {
      data: gameState,
      timestamp: Date.now()
    });
    
    return gameState;
  }
  
  private async batchFetchGameState(userPubkey: string): Promise<GameState> {
    const addresses = await this.calculateAllAddresses(userPubkey);
    const accounts = await this.connection.getMultipleAccountsInfo(addresses);
    
    return this.parseMultipleAccounts(accounts);
  }
}
```

## 7. 监控と分析

### 分析ダッシュボード
```typescript
// 管理者分析機能
class GameAnalytics {
  async generateDailyReport(): Promise<DailyReport> {
    const stats = await this.fetchGlobalStats();
    const transactions = await this.getTransactionHistory(24 * 60 * 60 * 1000);
    
    return {
      totalUsers: await this.countActiveUsers(),
      totalGrowPower: stats.totalGrowPower,
      dailyRewards: this.calculateDailyRewards(transactions),
      packSales: this.countPackSales(transactions),
      economicHealth: this.assessEconomicHealth(stats),
      topPlayers: await this.getTopPlayersByGrowPower(10),
      recentTrends: this.analyzeTrends(transactions)
    };
  }
  
  private assessEconomicHealth(stats: GlobalStats): EconomicHealthReport {
    const supplyUtilization = stats.totalSupplyMinted / 240_000_000;
    const rewardDistributionBalance = this.calculateGiniCoefficient();
    const inflationRate = this.calculateInflationRate();
    
    return {
      supplyUtilization,
      distributionBalance: rewardDistributionBalance,
      inflationRate,
      healthScore: this.calculateHealthScore(supplyUtilization, rewardDistributionBalance, inflationRate),
      recommendations: this.generateRecommendations()
    };
  }
}
```

## デプロイメント戦略

### アップグレード可能なプログラム設計
```rust
// バージョン管理
#[account]
pub struct ProgramVersion {
    pub version: [u8; 3],  // major.minor.patch
    pub features: Vec<FeatureFlag>,
    pub migration_required: bool,
    pub backward_compatible: bool,
    pub upgrade_authority: Pubkey,
    pub reserve: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeatureFlag {
    pub name: [u8; 32],
    pub enabled: bool,
    pub rollout_percentage: u8,  // 0-100
}
```

### 段階的ロールアウト
```typescript
// フィーチャーフラグによる段階的展開
class FeatureRollout {
  async isFeatureEnabled(feature: string, userPubkey: string): Promise<boolean> {
    const programVersion = await this.fetchProgramVersion();
    const featureFlag = programVersion.features.find(f => f.name === feature);
    
    if (!featureFlag || !featureFlag.enabled) return false;
    
    // ユーザーベースの段階的展開
    const userHash = this.hashUserPubkey(userPubkey);
    const rolloutThreshold = (featureFlag.rollout_percentage / 100) * 256;
    
    return userHash < rolloutThreshold;
  }
}
```

この拡張ガイドに従うことで、Facility Gameは将来の成長に対応し、ユーザーの期待に応える新機能を継続的に提供できます。