# テスト戦略とテスト内容

## 概要

Facility Gameは多層的なテスト戦略を採用し、単体テストから統合テスト、エンドツーエンドテストまで包括的なテストカバレッジを実現しています。テストは機能別・レイヤー別に整理され、継続的な品質保証を提供します。

## テスト構成

### ディレクトリ構造
```
tests/
├── unit/                      # 単体テスト
│   ├── instructions/         # 命令別テスト
│   │   ├── admin.test.ts     # 管理者機能
│   │   └── user.test.ts      # ユーザー機能
│   ├── state/                # ステート管理テスト
│   ├── error-cases.test.ts   # エラーケーステスト
│   └── storage-system.test.ts # ストレージシステム
├── integration/              # 統合テスト
│   ├── complete-user-journey.test.ts  # 完全ユーザージャーニー
│   └── user-journey.test.ts           # 基本ユーザージャーニー
├── e2e/                      # エンドツーエンドテスト
│   └── game-simulation.test.ts       # ゲーム全体シミュレーション
├── security/                 # セキュリティテスト
│   └── access-control.test.ts        # アクセス制御
└── helpers/                  # テストヘルパー
    ├── setup.ts              # セットアップユーティリティ
    ├── factories.ts          # テストデータファクトリー
    └── assertions.ts         # カスタムアサーション
```

## 1. 単体テスト（Unit Tests）

### 管理者機能テスト (admin.test.ts)

**テスト対象**:
- システム初期化
- 設定更新
- 確率テーブル管理
- 権限制御

**主要テストケース**:

```typescript
describe('Admin Instructions', () => {
  describe('initialize_config', () => {
    it('should initialize config with default values', async () => {
      const result = await program.methods
        .initializeConfig(null, null, treasury.publicKey, null)
        .rpc();
      
      const config = await program.account.config.fetch(configPDA);
      expect(config.baseRate.toNumber()).toBe(100);
      expect(config.halvingInterval.toNumber()).toBe(200);
    });

    it('should reject duplicate initialization', async () => {
      await expect(
        program.methods.initializeConfig(null, null, treasury.publicKey, null).rpc()
      ).rejects.toThrow('already in use');
    });

    it('should reject unauthorized access', async () => {
      await expect(
        program.methods
          .initializeConfig(null, null, treasury.publicKey, null)
          .accounts({ admin: unauthorized.publicKey })
          .rpc()
      ).rejects.toThrow('Unauthorized');
    });
  });

  describe('update_probability_table', () => {
    it('should update table with valid data', async () => {
      const newTable = {
        version: 2,
        seedCount: 6,
        growPowers: [120, 200, 450, 800, 1200, 6000],
        probabilityThresholds: [4000, 6500, 8000, 9000, 9600, 10000],
        expectedValue: 500
      };

      await program.methods.updateProbabilityTable(newTable).rpc();
      
      const table = await program.account.probabilityTable.fetch(tablePDA);
      expect(table.version).toBe(2);
      expect(table.expectedValue.toNumber()).toBe(500);
    });

    it('should reject invalid probability distribution', async () => {
      const invalidTable = {
        probabilityThresholds: [5000, 6000, 7000, 8000, 9000, 9999] // Doesn't sum to 10000
      };

      await expect(
        program.methods.updateProbabilityTable(invalidTable).rpc()
      ).rejects.toThrow('InvalidProbabilityTable');
    });
  });
});
```

### ユーザー機能テスト (user.test.ts)

**テスト対象**:
- ユーザー登録
- 農場購入
- シード操作
- 報酬請求

**主要テストケース**:

```typescript
describe('User Instructions', () => {
  describe('init_user', () => {
    it('should create user with no referrer', async () => {
      await program.methods.initUser(null).rpc();
      
      const userState = await program.account.userState.fetch(userPDA);
      expect(userState.owner.toString()).toBe(user.publicKey.toString());
      expect(userState.totalGrowPower.toNumber()).toBe(0);
      expect(userState.referrer).toBeNull();
    });

    it('should create user with referrer', async () => {
      const referrer = Keypair.generate();
      await setupUser(referrer); // Helper function
      
      await program.methods.initUser(referrer.publicKey).rpc();
      
      const userState = await program.account.userState.fetch(userPDA);
      expect(userState.referrer.toString()).toBe(referrer.publicKey.toString());
    });
  });

  describe('buy_farm_space', () => {
    it('should purchase farm space and receive initial seed', async () => {
      const initialBalance = await connection.getBalance(user.publicKey);
      
      await program.methods.buyFarmSpace().rpc();
      
      const finalBalance = await connection.getBalance(user.publicKey);
      const farmSpace = await program.account.farmSpace.fetch(farmPDA);
      
      expect(initialBalance - finalBalance).toBeCloseTo(0.5 * LAMPORTS_PER_SOL, -3);
      expect(farmSpace.level).toBe(1);
      expect(farmSpace.capacity).toBe(4);
      expect(farmSpace.seedCount).toBe(1);
      expect(farmSpace.totalGrowPower.toNumber()).toBe(100);
    });

    it('should reject duplicate farm purchase', async () => {
      await program.methods.buyFarmSpace().rpc(); // First purchase
      
      await expect(
        program.methods.buyFarmSpace().rpc()
      ).rejects.toThrow('AlreadyHasFarm');
    });
  });
});
```

### エラーケーステスト (error-cases.test.ts)

**テスト範囲**:
- 権限エラー
- 状態エラー
- 容量制限エラー
- 計算オーバーフローエラー

```typescript
describe('Error Cases', () => {
  describe('Authorization Errors', () => {
    it('should reject unauthorized config update', async () => {
      const unauthorized = Keypair.generate();
      
      await expect(
        program.methods
          .updateConfig({ baseRate: new BN(200) })
          .accounts({ admin: unauthorized.publicKey })
          .rpc()
      ).rejects.toThrow('Unauthorized');
    });
  });

  describe('Capacity Errors', () => {
    it('should reject planting when farm is full', async () => {
      await fillFarmToCapacity(user); // Helper function
      
      const newSeed = await createSeed(user, SeedType.Seed1);
      
      await expect(
        program.methods.plantSeed(newSeed.seedId).rpc()
      ).rejects.toThrow('FarmAtMaxCapacity');
    });
  });

  describe('Overflow Protection', () => {
    it('should prevent reward calculation overflow', async () => {
      const maxUser = await createUserWithMaxGrowPower(); // Helper
      
      await expect(
        program.methods.claimRewardWithReferralRewards().rpc()
      ).rejects.toThrow('CalculationOverflow');
    });
  });
});
```

## 2. 統合テスト（Integration Tests）

### 完全ユーザージャーニーテスト (complete-user-journey.test.ts)

**シナリオ**: 新規ユーザーから上級プレイヤーまでの完全な流れ

```typescript
describe('Complete User Journey', () => {
  it('should complete full game lifecycle', async () => {
    // Phase 1: Registration and Setup
    await program.methods.initUser(null).rpc();
    await program.methods.buyFarmSpace().rpc();
    
    let userState = await program.account.userState.fetch(userPDA);
    let farmSpace = await program.account.farmSpace.fetch(farmPDA);
    
    expect(userState.hasFarmSpace).toBe(true);
    expect(farmSpace.level).toBe(1);
    expect(farmSpace.seedCount).toBe(1); // Initial seed
    
    // Phase 2: First Seed Pack Purchase
    await airdropWEED(user, 1000); // Helper function
    await program.methods
      .purchaseSeedPack(1, randomSeed(), maxVrfFee)
      .rpc();
    
    // Phase 3: Pack Opening and Seed Management
    await program.methods.openSeedPack(1).rpc();
    
    const seedStorage = await program.account.seedStorage.fetch(storagePDA);
    expect(seedStorage.totalSeeds).toBeGreaterThan(1);
    
    // Phase 4: Farm Optimization
    const availableSeeds = await fetchUserSeeds(user);
    const bestSeed = findHighestGrowPowerSeed(availableSeeds);
    
    if (farmSpace.seedCount < farmSpace.capacity) {
      await program.methods.plantSeed(bestSeed.seedId).rpc();
    }
    
    // Phase 5: Reward Generation and Claiming
    await sleep(5000); // Wait for rewards to accumulate
    
    const beforeBalance = await getTokenBalance(user);
    await program.methods.claimRewardWithReferralRewards().rpc();
    const afterBalance = await getTokenBalance(user);
    
    expect(afterBalance).toBeGreaterThan(beforeBalance);
    
    // Phase 6: Farm Expansion
    for (let i = 0; i < 30; i++) {
      await program.methods
        .purchaseSeedPack(1, randomSeed(), maxVrfFee)
        .rpc();
      await program.methods.openSeedPack(1).rpc();
    }
    
    farmSpace = await program.account.farmSpace.fetch(farmPDA);
    expect(farmSpace.level).toBe(2); // Auto-upgraded to level 2
    expect(farmSpace.capacity).toBe(6);
    
    // Phase 7: Referral System
    const newUser = Keypair.generate();
    await program.methods
      .initUser(user.publicKey)
      .accounts({ user: newUser.publicKey })
      .signers([newUser])
      .rpc();
    
    // Generate rewards for referred user
    await buyFarmSpaceForUser(newUser);
    await sleep(3000);
    await claimRewardsForUser(newUser);
    
    // Check referral rewards
    userState = await program.account.userState.fetch(userPDA);
    expect(userState.pendingReferralRewards.toNumber()).toBeGreaterThan(0);
  });
});
```

### 基本ユーザージャーニーテスト (user-journey.test.ts)

**シナリオ**: 基本的なゲームプレイフローの検証

```typescript
describe('Basic User Journey', () => {
  it('should complete basic gameplay loop', async () => {
    // Setup
    await initializeGameState();
    await program.methods.initUser(null).rpc();
    await program.methods.buyFarmSpace().rpc();
    
    // Basic seed management
    await purchaseAndOpenSeedPack(user, 3);
    await optimizeFarmLayout(user);
    
    // Reward cycle
    await sleep(2000);
    const rewards = await program.methods.claimRewardWithReferralRewards().rpc();
    
    expect(rewards).toBeDefined();
  });
  
  it('should handle multiple users simultaneously', async () => {
    const users = await createMultipleUsers(5);
    
    // Parallel operations
    await Promise.all(users.map(async (user) => {
      await program.methods.buyFarmSpace()
        .accounts({ user: user.publicKey })
        .signers([user])
        .rpc();
    }));
    
    // Verify global stats
    const globalStats = await program.account.globalStats.fetch(globalStatsPDA);
    expect(globalStats.totalFarmSpaces.toNumber()).toBe(5);
  });
});
```

## 3. エンドツーエンドテスト（E2E Tests）

### ゲーム全体シミュレーション (game-simulation.test.ts)

**目的**: 実際のゲーム環境での長期間動作検証

```typescript
describe('Game Simulation', () => {
  it('should simulate 24-hour game operation', async () => {
    const SIMULATION_HOURS = 24;
    const USERS_COUNT = 50;
    const PACK_PURCHASES_PER_HOUR = 100;
    
    // Initialize game state
    await initializeGameWithUsers(USERS_COUNT);
    
    for (let hour = 0; hour < SIMULATION_HOURS; hour++) {
      console.log(`Simulating hour ${hour + 1}/${SIMULATION_HOURS}`);
      
      // Random user activities
      const activities = await simulateHourlyActivities({
        packPurchases: PACK_PURCHASES_PER_HOUR,
        rewardClaims: Math.floor(USERS_COUNT * 0.3),
        farmOptimizations: Math.floor(USERS_COUNT * 0.1),
        newUserRegistrations: Math.floor(Math.random() * 5)
      });
      
      // Verify system consistency
      await verifySystemIntegrity();
      
      // Check economic balance
      const economicMetrics = await calculateEconomicMetrics();
      expect(economicMetrics.totalSupply).toBeLessThanOrEqual(240_000_000);
      expect(economicMetrics.rewardDistribution).toBeBalanced();
    }
    
    // Final verification
    const finalStats = await program.account.globalStats.fetch(globalStatsPDA);
    expect(finalStats.totalGrowPower.toNumber()).toBeGreaterThan(0);
    
    // Performance metrics
    const performanceReport = generatePerformanceReport();
    expect(performanceReport.averageTransactionTime).toBeLessThan(2000); // 2 seconds
  });
  
  it('should handle halving mechanism correctly', async () => {
    // Setup with short halving interval for testing
    await program.methods
      .updateConfig({ halvingInterval: new BN(10) }) // 10 seconds
      .rpc();
    
    let config = await program.account.config.fetch(configPDA);
    const initialRate = config.baseRate.toNumber();
    
    // Wait for halving
    await sleep(11000);
    
    // Trigger halving through reward claim
    await program.methods.claimRewardWithReferralRewards().rpc();
    
    config = await program.account.config.fetch(configPDA);
    expect(config.baseRate.toNumber()).toBe(initialRate / 2);
  });
});
```

## 4. セキュリティテスト（Security Tests）

### アクセス制御テスト (access-control.test.ts)

```typescript
describe('Access Control', () => {
  describe('Admin Functions', () => {
    it('should prevent non-admin from updating config', async () => {
      const hacker = Keypair.generate();
      
      await expect(
        program.methods
          .updateConfig({ baseRate: new BN(999999) })
          .accounts({ admin: hacker.publicKey })
          .signers([hacker])
          .rpc()
      ).rejects.toThrow('Unauthorized');
    });
  });
  
  describe('User Data Protection', () => {
    it('should prevent users from accessing others\' seeds', async () => {
      const user1 = Keypair.generate();
      const user2 = Keypair.generate();
      
      await setupUser(user1);
      await setupUser(user2);
      
      const user1Seed = await createSeedForUser(user1);
      
      await expect(
        program.methods
          .plantSeed(user1Seed.seedId)
          .accounts({ user: user2.publicKey })
          .signers([user2])
          .rpc()
      ).rejects.toThrow('UnauthorizedUser');
    });
  });
  
  describe('PDA Security', () => {
    it('should validate PDA derivation', async () => {
      const maliciousPDA = Keypair.generate().publicKey;
      
      await expect(
        program.methods
          .initUser(null)
          .accounts({ userState: maliciousPDA })
          .rpc()
      ).rejects.toThrow('ConstraintSeeds');
    });
  });
});
```

## 5. パフォーマンステスト

### 負荷テスト

```typescript
describe('Performance Tests', () => {
  it('should handle concurrent transactions', async () => {
    const CONCURRENT_USERS = 20;
    const users = await createMultipleUsers(CONCURRENT_USERS);
    
    const startTime = Date.now();
    
    // Concurrent farm purchases
    await Promise.all(users.map(user =>
      program.methods.buyFarmSpace()
        .accounts({ user: user.publicKey })
        .signers([user])
        .rpc()
    ));
    
    const endTime = Date.now();
    const duration = endTime - startTime;
    
    expect(duration).toBeLessThan(10000); // 10 seconds
  });
  
  it('should handle large seed storage efficiently', async () => {
    const user = await createUserWithMaxSeeds(2000); // Max capacity
    
    const startTime = Date.now();
    await program.methods.batchDiscardSeeds(generateRandomSeedIds(100)).rpc();
    const endTime = Date.now();
    
    expect(endTime - startTime).toBeLessThan(5000); // 5 seconds
  });
});
```

## 6. テストユーティリティ

### セットアップヘルパー (setup.ts)

```typescript
export async function initializeGameState() {
  await program.methods
    .initializeConfig(null, null, treasury.publicKey, null)
    .rpc();
  
  await program.methods.createRewardMint().rpc();
  await program.methods.initializeGlobalStats().rpc();
  await program.methods.initializeProbabilityTable().rpc();
}

export async function setupUser(user: Keypair) {
  await airdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
  await program.methods.initUser(null)
    .accounts({ user: user.publicKey })
    .signers([user])
    .rpc();
}

export async function airdropWEED(user: Keypair, amount: number) {
  // Implementation for test token airdrop
}
```

### テストアサーション (assertions.ts)

```typescript
export function expectValidFarmState(farmSpace: any) {
  expect(farmSpace.level).toBeGreaterThan(0);
  expect(farmSpace.level).toBeLessThanOrEqual(5);
  expect(farmSpace.seedCount).toBeLessThanOrEqual(farmSpace.capacity);
  expect(farmSpace.totalGrowPower.toNumber()).toBeGreaterThanOrEqual(0);
}

export function expectValidEconomicState(config: any, globalStats: any) {
  expect(globalStats.totalSupply.toNumber()).toBeLessThanOrEqual(240_000_000);
  expect(config.baseRate.toNumber()).toBeGreaterThan(0);
}
```

## テスト実行とCI/CD

### ローカル実行

```bash
# 全テスト実行
anchor test

# 特定カテゴリのテスト
anchor test -- --grep "Unit Tests"
anchor test -- --grep "Integration Tests"

# 並列実行（高速化）
anchor test --parallel

# カバレッジレポート
anchor test --coverage
```

### CI/CD統合

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Anchor
        run: npm install -g @coral-xyz/anchor-cli
      - name: Run tests
        run: anchor test
      - name: Upload coverage
        uses: codecov/codecov-action@v2
```

## 品質メトリクス

### 目標カバレッジ
- **命令カバレッジ**: 100%
- **分岐カバレッジ**: 95%
- **エラーパスカバレッジ**: 100%
- **統合シナリオカバレッジ**: 90%

### パフォーマンス目標
- **平均応答時間**: < 2秒
- **並行処理能力**: 50トランザクション/秒
- **メモリ使用量**: < 512MB per test suite

この包括的なテスト戦略により、Facility Gameは高い品質と信頼性を維持し、ユーザーに安全で快適なゲーム体験を提供します。