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
- システム初期化 (`initialize_config`)
- 報酬ミント作成 (`create_reward_mint`)
- グローバル統計初期化 (`initialize_global_stats`)
- 確率テーブル管理 (`initialize_probability_table`, `update_probability_table`)
- 農場レベル設定管理 (`initialize_farm_level_config`, `update_farm_level_config`)
- シードパック価格更新 (`update_seed_pack_cost`)
- 秘密シード公開 (`reveal_seed`, `update_seed_values`)
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
      expect(config.seedPackCost.toNumber()).toBe(300_000_000); // 300 WEED
      expect(config.farmSpaceCostSol.toNumber()).toBe(500_000_000); // 0.5 SOL
    });

    it('should reject duplicate initialization', async () => {
      await expect(
        program.methods.initializeConfig(null, null, treasury.publicKey, null).rpc()
      ).rejects.toThrow('already in use');
    });
  });

  describe('create_reward_mint', () => {
    it('should create mint with transfer fee extension', async () => {
      await program.methods.createRewardMint().rpc();
      
      const mintInfo = await connection.getAccountInfo(rewardMintPDA);
      expect(mintInfo).toBeDefined();
      
      // Verify SPL Token 2022 with Transfer Fee Extension
      const mintData = unpackMint(rewardMintPDA, mintInfo, TOKEN_2022_PROGRAM_ID);
      expect(mintData.decimals).toBe(6);
      expect(mintData.mintAuthority.toString()).toBe(mintAuthorityPDA.toString());
    });
  });

  describe('update_probability_table', () => {
    it('should update table with valid 9-seed configuration', async () => {
      const newTable = {
        version: 2,
        seedCount: 9,
        growPowers: [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000],
        probabilityThresholds: [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000],
        probabilityPercentages: [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56],
        expectedValue: 1590,
        name: "Enhanced9Seeds"
      };

      await program.methods.updateProbabilityTable(
        newTable.version,
        newTable.seedCount,
        newTable.growPowers,
        newTable.probabilityThresholds,
        newTable.probabilityPercentages,
        newTable.expectedValue,
        newTable.name
      ).rpc();
      
      const table = await program.account.probabilityTable.fetch(tablePDA);
      expect(table.version).toBe(2);
      expect(table.seedCount).toBe(9);
      expect(table.expectedValue.toNumber()).toBe(1590);
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

  describe('reveal_seed', () => {
    it('should reveal secret seed type', async () => {
      // Reveal Seed9 (first secret seed)
      await program.methods.revealSeed(
        8, // seed_index for Seed9
        60000, // grow_power
        0.56 // probability_percentage
      ).rpc();
      
      const table = await program.account.probabilityTable.fetch(tablePDA);
      expect(table.isSeededRevealed(8)).toBe(true);
      expect(table.growPowers[8].toNumber()).toBe(60000);
    });
  });
});
```

### ユーザー機能テスト (user.test.ts)

**テスト対象**:
- ユーザー初期化 (`init_user` - admin/operator専用)
- 招待システム (`create_invite_code`, `use_invite_code`)
- 農場購入 (`buy_farm_space`)
- シード管理 (`initialize_seed_storage`, `purchase_seed_pack`, `open_seed_pack`, `plant_seed`, `remove_seed`)
- 報酬請求 (`claim_reward_with_referral_rewards`)

**主要テストケース**:

```typescript
describe('User Instructions', () => {
  describe('init_user (admin only)', () => {
    it('should allow admin to create user without invite', async () => {
      await program.methods.initUser(null)
        .accounts({ admin: admin.publicKey })
        .signers([admin])
        .rpc();
      
      const userState = await program.account.userState.fetch(userPDA);
      expect(userState.owner.toString()).toBe(user.publicKey.toString());
      expect(userState.totalGrowPower.toNumber()).toBe(0);
      expect(userState.referrer).toBeNull();
    });

    it('should reject non-admin user creation', async () => {
      const unauthorized = Keypair.generate();
      await expect(
        program.methods.initUser(null)
          .accounts({ admin: unauthorized.publicKey })
          .signers([unauthorized])
          .rpc()
      ).rejects.toThrow('Unauthorized');
    });
  });

  describe('invite_system', () => {
    it('should create and use invite code', async () => {
      const inviter = Keypair.generate();
      await setupUser(inviter);
      
      // Create invite code
      const inviteCode = Buffer.from('TESTCODE');
      await program.methods.createInviteCode(Array.from(inviteCode))
        .accounts({ inviter: inviter.publicKey })
        .signers([inviter])
        .rpc();
      
      // Use invite code
      const invitee = Keypair.generate();
      await program.methods.useInviteCode(Array.from(inviteCode), inviter.publicKey)
        .accounts({ 
          invitee: invitee.publicKey,
          inviterPubkey: inviter.publicKey 
        })
        .signers([invitee])
        .rpc();
      
      const userState = await program.account.userState.fetch(
        getUserStatePDA(invitee.publicKey)
      );
      expect(userState.referrer.toString()).toBe(inviter.publicKey.toString());
    });
  });

  describe('buy_farm_space', () => {
    it('should purchase farm space and receive initial seed', async () => {
      const initialBalance = await connection.getBalance(user.publicKey);
      
      await program.methods.buyFarmSpace().rpc();
      
      const finalBalance = await connection.getBalance(user.publicKey);
      const farmSpace = await program.account.farmSpace.fetch(farmPDA);
      const userState = await program.account.userState.fetch(userPDA);
      
      expect(initialBalance - finalBalance).toBeCloseTo(0.5 * LAMPORTS_PER_SOL, -6);
      expect(farmSpace.level).toBe(1);
      expect(farmSpace.capacity).toBe(4);
      expect(farmSpace.seedCount).toBe(1);
      expect(farmSpace.totalGrowPower.toNumber()).toBe(100); // Initial Seed1
      expect(userState.hasFarmSpace).toBe(true);
    });

    it('should reject duplicate farm purchase', async () => {
      await program.methods.buyFarmSpace().rpc(); // First purchase
      
      await expect(
        program.methods.buyFarmSpace().rpc()
      ).rejects.toThrow('AlreadyHasFarm');
    });
  });

  describe('seed_pack_system', () => {
    it('should purchase and open seed pack with VRF', async () => {
      await airdropWEED(user, 1000); // Helper function
      
      // Purchase pack
      await program.methods.purchaseSeedPack(
        1, // quantity
        randomSeed(), // user_entropy_seed
        2_000_000 // max_vrf_fee in lamports
      ).rpc();
      
      // Open pack (VRF simulation)
      await program.methods.openSeedPack(1).rpc();
      
      const seedStorage = await program.account.seedStorage.fetch(storagePDA);
      expect(seedStorage.totalSeeds).toBeGreaterThan(0);
    });

    it('should auto-upgrade farm based on pack purchases', async () => {
      await airdropWEED(user, 10000);
      
      // Purchase 30 packs to trigger level 2 upgrade
      for (let i = 0; i < 30; i++) {
        await program.methods.purchaseSeedPack(1, randomSeed(), 2_000_000).rpc();
        await program.methods.openSeedPack(1).rpc();
      }
      
      const farmSpace = await program.account.farmSpace.fetch(farmPDA);
      expect(farmSpace.level).toBe(2);
      expect(farmSpace.capacity).toBe(6);
    });
  });
});
```

### エラーケーステスト (error-cases.test.ts)

**テスト範囲**:
- 権限エラー (`Unauthorized`, `UnauthorizedOperator`)
- 状態エラー (`AlreadyHasFarm`, `InvalidConfig`)
- 容量制限エラー (`FarmAtMaxCapacity`, `StorageFull`, `SeedTypeLimitReached`)
- 計算オーバーフローエラー (`CalculationOverflow`)
- 招待システムエラー (`InvalidInviteCode`, `InviteCodeLimitReached`)

```typescript
describe('Error Cases', () => {
  describe('Authorization Errors', () => {
    it('should reject unauthorized config update', async () => {
      const unauthorized = Keypair.generate();
      
      await expect(
        program.methods
          .updateConfig(null, new BN(200), null, null, null)
          .accounts({ admin: unauthorized.publicKey })
          .signers([unauthorized])
          .rpc()
      ).rejects.toThrow('Unauthorized');
    });

    it('should reject non-admin user initialization', async () => {
      const regularUser = Keypair.generate();
      
      await expect(
        program.methods.initUser(null)
          .accounts({ admin: regularUser.publicKey })
          .signers([regularUser])
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

    it('should reject storage when seed type limit reached', async () => {
      await fillSeedTypeToLimit(user, SeedType.Seed1, 100); // Helper function
      
      await expect(
        program.methods.purchaseSeedPack(1, randomSeed(), 2_000_000).rpc()
      ).rejects.toThrow('SeedTypeLimitReached');
    });
  });

  describe('Transfer Fee Extension Errors', () => {
    it('should handle insufficient balance with transfer fees', async () => {
      // User has exactly 1000 WEED, but needs 300 + 2% fee for transfer
      await airdropWEED(user, 300);
      
      await expect(
        program.methods.purchaseSeedPack(1, randomSeed(), 2_000_000).rpc()
      ).rejects.toThrow('InsufficientFunds');
    });
  });

  describe('VRF System Errors', () => {
    it('should reject insufficient VRF fee', async () => {
      await airdropWEED(user, 1000);
      
      await expect(
        program.methods.purchaseSeedPack(
          1, 
          randomSeed(), 
          1_000_000 // Too low VRF fee
        ).rpc()
      ).rejects.toThrow('InsufficientVrfFee');
    });
  });
});
```

## 2. 統合テスト（Integration Tests）

### 完全ユーザージャーニーテスト (complete-user-journey.test.ts)

**シナリオ**: 新規ユーザーから上級プレイヤーまでの完全な流れ

```typescript
describe('Complete User Journey', () => {
  it('should complete full game lifecycle with invite system', async () => {
    // Phase 1: Admin Setup
    await initializeGameState();
    
    // Phase 2: Invite Creation and User Registration
    const inviter = Keypair.generate();
    await program.methods.initUser(null)
      .accounts({ admin: admin.publicKey, user: inviter.publicKey })
      .signers([admin])
      .rpc();
    
    const inviteCode = Buffer.from('WELCOME1');
    await program.methods.createInviteCode(Array.from(inviteCode))
      .accounts({ inviter: inviter.publicKey })
      .signers([inviter])
      .rpc();
    
    const newUser = Keypair.generate();
    await program.methods.useInviteCode(Array.from(inviteCode), inviter.publicKey)
      .accounts({ 
        invitee: newUser.publicKey,
        inviterPubkey: inviter.publicKey 
      })
      .signers([newUser])
      .rpc();
    
    // Phase 3: Farm Setup
    await program.methods.buyFarmSpace()
      .accounts({ user: newUser.publicKey })
      .signers([newUser])
      .rpc();
    
    let userState = await program.account.userState.fetch(getUserStatePDA(newUser.publicKey));
    let farmSpace = await program.account.farmSpace.fetch(getFarmSpacePDA(newUser.publicKey));
    
    expect(userState.hasFarmSpace).toBe(true);
    expect(userState.referrer.toString()).toBe(inviter.publicKey.toString());
    expect(farmSpace.level).toBe(1);
    expect(farmSpace.seedCount).toBe(1); // Initial seed
    
    // Phase 4: Seed Pack Purchasing and Opening
    await airdropWEED(newUser, 5000);
    
    for (let i = 0; i < 5; i++) {
      await program.methods.purchaseSeedPack(1, randomSeed(), 2_000_000)
        .accounts({ user: newUser.publicKey })
        .signers([newUser])
        .rpc();
      
      await program.methods.openSeedPack(1)
        .accounts({ user: newUser.publicKey })
        .signers([newUser])
        .rpc();
    }
    
    const seedStorage = await program.account.seedStorage.fetch(
      getSeedStoragePDA(newUser.publicKey)
    );
    expect(seedStorage.totalSeeds).toBeGreaterThan(5);
    
    // Phase 5: Farm Optimization
    const availableSeeds = await fetchUserSeeds(newUser);
    const bestSeeds = findHighestGrowPowerSeeds(availableSeeds, 3);
    
    for (const seed of bestSeeds) {
      if (farmSpace.seedCount < farmSpace.capacity) {
        await program.methods.plantSeed(seed.seedId)
          .accounts({ user: newUser.publicKey })
          .signers([newUser])
          .rpc();
      }
    }
    
    // Phase 6: Reward Generation and Claiming
    await sleep(10000); // Wait for rewards to accumulate
    
    const beforeBalance = await getTokenBalance(newUser);
    await program.methods.claimRewardWithReferralRewards()
      .accounts({ user: newUser.publicKey })
      .signers([newUser])
      .rpc();
    const afterBalance = await getTokenBalance(newUser);
    
    expect(afterBalance).toBeGreaterThan(beforeBalance);
    
    // Verify referral rewards were accumulated
    const inviterState = await program.account.userState.fetch(
      getUserStatePDA(inviter.publicKey)
    );
    expect(inviterState.pendingReferralRewards.toNumber()).toBeGreaterThan(0);
    
    // Phase 7: Farm Auto-Upgrade
    await airdropWEED(newUser, 15000);
    
    // Purchase 30 packs to trigger level 2 upgrade
    for (let i = 0; i < 30; i++) {
      await program.methods.purchaseSeedPack(1, randomSeed(), 2_000_000)
        .accounts({ user: newUser.publicKey })
        .signers([newUser])
        .rpc();
      await program.methods.openSeedPack(1)
        .accounts({ user: newUser.publicKey })
        .signers([newUser])
        .rpc();
    }
    
    farmSpace = await program.account.farmSpace.fetch(getFarmSpacePDA(newUser.publicKey));
    expect(farmSpace.level).toBe(2);
    expect(farmSpace.capacity).toBe(6);
    
    // Phase 8: Transfer Fee System Verification
    const recipientUser = Keypair.generate();
    await createTokenAccountForUser(recipientUser);
    
    const transferAmount = 1000_000_000; // 1000 WEED
    const expectedFee = transferAmount * 200 / 10000; // 2% fee
    
    const senderBefore = await getTokenBalance(newUser);
    const recipientBefore = await getTokenBalance(recipientUser);
    
    await transferTokens(newUser, recipientUser, transferAmount);
    
    const senderAfter = await getTokenBalance(newUser);
    const recipientAfter = await getTokenBalance(recipientUser);
    
    expect(senderBefore - senderAfter).toBe(transferAmount);
    expect(recipientAfter - recipientBefore).toBe(transferAmount - expectedFee);
  });
});
```

## 3. エンドツーエンドテスト（E2E Tests）

### ゲーム全体シミュレーション (game-simulation.test.ts)

```typescript
describe('Game Simulation', () => {
  it('should simulate realistic game ecosystem', async () => {
    const SIMULATION_HOURS = 12;
    const INITIAL_USERS = 20;
    const HOURLY_NEW_USERS = 2;
    const PACK_PURCHASES_PER_HOUR = 50;
    
    // Initialize game with admin setup
    await initializeCompleteGameState();
    
    // Create initial user base with invite system
    const users = await createUserBaseWithInvites(INITIAL_USERS);
    
    for (let hour = 0; hour < SIMULATION_HOURS; hour++) {
      console.log(`Simulating hour ${hour + 1}/${SIMULATION_HOURS}`);
      
      // New user registrations via invites
      const newUsers = await simulateNewUserRegistrations(HOURLY_NEW_USERS);
      users.push(...newUsers);
      
      // Existing user activities
      await simulateHourlyActivities({
        users,
        packPurchases: PACK_PURCHASES_PER_HOUR,
        rewardClaims: Math.floor(users.length * 0.3),
        farmOptimizations: Math.floor(users.length * 0.1),
        transferActivities: Math.floor(users.length * 0.05)
      });
      
      // System integrity checks
      await verifySystemIntegrity();
      await verifyTransferFeeAccumulation();
      
      // Economic balance verification
      const economicMetrics = await calculateEconomicMetrics();
      expect(economicMetrics.totalSupply).toBeLessThanOrEqual(240_000_000);
      expect(economicMetrics.inflationRate).toBeLessThan(0.1); // 10% annual
    }
    
    // Final verification
    const finalStats = await program.account.globalStats.fetch(globalStatsPDA);
    expect(finalStats.totalGrowPower.toNumber()).toBeGreaterThan(0);
    expect(finalStats.totalFarmSpaces.toNumber()).toBe(users.length);
    
    // Verify halving mechanism
    await verifyHalvingMechanism();
    
    // Performance metrics
    const performanceReport = generatePerformanceReport();
    expect(performanceReport.averageTransactionTime).toBeLessThan(3000); // 3 seconds
  });
  
  it('should handle transfer fee economics correctly', async () => {
    await initializeGameState();
    
    // Create trading scenario
    const traders = await createMultipleUsers(10);
    await Promise.all(traders.map(user => airdropWEED(user, 10000)));
    
    // Simulate trading activity
    const totalTradingVolume = 50000_000_000; // 50,000 WEED
    const expectedFees = totalTradingVolume * 200 / 10000; // 2%
    
    await simulateTradingActivity(traders, totalTradingVolume);
    
    // Verify fee accumulation
    const treasuryBalance = await getTokenBalance(treasury);
    expect(treasuryBalance).toBeCloseTo(expectedFees, -6);
  });
});
```

## 4. セキュリティテスト（Security Tests）

### アクセス制御テスト (access-control.test.ts)

```typescript
describe('Access Control', () => {
  describe('Admin Functions', () => {
    it('should enforce admin-only operations', async () => {
      const hacker = Keypair.generate();
      
      const adminOnlyFunctions = [
        () => program.methods.updateConfig(null, new BN(999999), null, null, null),
        () => program.methods.updateSeedPackCost(new BN(999_000_000)),
        () => program.methods.revealSeed(8, new BN(999999), 0.1),
        () => program.methods.updateProbabilityTable(2, 9, [], [], [], new BN(0), "hack")
      ];
      
      for (const fn of adminOnlyFunctions) {
        await expect(
          fn().accounts({ admin: hacker.publicKey }).signers([hacker]).rpc()
        ).rejects.toThrow('Unauthorized');
      }
    });
  });
  
  describe('User Data Isolation', () => {
    it('should prevent cross-user seed access', async () => {
      const user1 = Keypair.generate();
      const user2 = Keypair.generate();
      
      await setupUser(user1);
      await setupUser(user2);
      
      const user1Seed = await createSeedForUser(user1);
      
      await expect(
        program.methods.plantSeed(user1Seed.seedId)
          .accounts({ user: user2.publicKey })
          .signers([user2])
          .rpc()
      ).rejects.toThrow('ConstraintSeeds');
    });
  });
  
  describe('PDA Security', () => {
    it('should validate all PDA derivations', async () => {
      const maliciousPDAs = [
        Keypair.generate().publicKey, // Fake user state
        Keypair.generate().publicKey, // Fake farm space
        Keypair.generate().publicKey, // Fake config
      ];
      
      for (const maliciousPDA of maliciousPDAs) {
        await expect(
          program.methods.initUser(null)
            .accounts({ userState: maliciousPDA })
            .rpc()
        ).rejects.toThrow('ConstraintSeeds');
      }
    });
  });

  describe('Invite System Security', () => {
    it('should prevent invite code replay attacks', async () => {
      const inviter = Keypair.generate();
      await setupUser(inviter);
      
      const inviteCode = Buffer.from('TESTCODE');
      await program.methods.createInviteCode(Array.from(inviteCode))
        .accounts({ inviter: inviter.publicKey })
        .signers([inviter])
        .rpc();
      
      // First use should succeed
      const user1 = Keypair.generate();
      await program.methods.useInviteCode(Array.from(inviteCode), inviter.publicKey)
        .accounts({ invitee: user1.publicKey })
        .signers([user1])
        .rpc();
      
      // Second use with same code should fail if limit is 1
      const user2 = Keypair.generate();
      await expect(
        program.methods.useInviteCode(Array.from(inviteCode), inviter.publicKey)
          .accounts({ invitee: user2.publicKey })
          .signers([user2])
          .rpc()
      ).rejects.toThrow('InviteCodeLimitReached');
    });
  });
});
```

## 5. パフォーマンステスト

### 負荷テスト

```typescript
describe('Performance Tests', () => {
  it('should handle concurrent operations efficiently', async () => {
    const CONCURRENT_USERS = 50;
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
    
    expect(duration).toBeLessThan(30000); // 30 seconds for 50 users
    
    // Verify all farms were created
    const farmSpaces = await Promise.all(
      users.map(user => 
        program.account.farmSpace.fetch(getFarmSpacePDA(user.publicKey))
      )
    );
    expect(farmSpaces).toHaveLength(CONCURRENT_USERS);
  });
  
  it('should handle large-scale seed operations', async () => {
    const user = await createUserWithMaxSeeds(1500); // Near max capacity
    
    const startTime = Date.now();
    
    // Batch operations
    const operations = [
      () => program.methods.batchDiscardSeeds(generateRandomSeedIds(100)).rpc(),
      () => program.methods.purchaseSeedPack(10, randomSeed(), 20_000_000).rpc(),
      () => program.methods.openSeedPack(10).rpc()
    ];
    
    await Promise.all(operations.map(op => op()));
    
    const endTime = Date.now();
    expect(endTime - startTime).toBeLessThan(10000); // 10 seconds
  });

  it('should maintain performance with transfer fee calculations', async () => {
    const users = await createMultipleUsers(20);
    await Promise.all(users.map(user => airdropWEED(user, 5000)));
    
    const startTime = Date.now();
    
    // Simulate heavy transfer activity
    for (let i = 0; i < 100; i++) {
      const sender = users[Math.floor(Math.random() * users.length)];
      const recipient = users[Math.floor(Math.random() * users.length)];
      
      if (sender !== recipient) {
        await transferTokens(sender, recipient, 100_000_000); // 100 WEED
      }
    }
    
    const endTime = Date.now();
    expect(endTime - startTime).toBeLessThan(60000); // 1 minute for 100 transfers
  });
});
```

## 6. テストユーティリティ

### セットアップヘルパー (setup.ts)

```typescript
export async function initializeCompleteGameState() {
  // Core system initialization
  await program.methods
    .initializeConfig(null, null, treasury.publicKey, null)
    .rpc();
  
  await program.methods.createRewardMint().rpc();
  await program.methods.initializeGlobalStats().rpc();
  await program.methods.initializeFeePool(treasury.publicKey).rpc();
  await program.methods.initializeProbabilityTable().rpc();
  await program.methods.initializeFarmLevelConfig().rpc();
  
  // Advanced configuration
  await program.methods.updateProbabilityTable(
    2, // version
    9, // seed_count
    [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000], // grow_powers
    [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000], // thresholds
    [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56], // percentages
    1590, // expected_value
    "Enhanced9Seeds" // name
  ).rpc();
}

export async function setupUserWithInvite(inviter?: Keypair): Promise<Keypair> {
  const user = Keypair.generate();
  await airdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
  
  if (inviter) {
    const inviteCode = generateRandomInviteCode();
    await program.methods.createInviteCode(Array.from(inviteCode))
      .accounts({ inviter: inviter.publicKey })
      .signers([inviter])
      .rpc();
    
    await program.methods.useInviteCode(Array.from(inviteCode), inviter.publicKey)
      .accounts({ invitee: user.publicKey })
      .signers([user])
      .rpc();
  } else {
    // Admin creates user directly
    await program.methods.initUser(null)
      .accounts({ admin: admin.publicKey, user: user.publicKey })
      .signers([admin])
      .rpc();
  }
  
  return user;
}

export async function airdropWEED(user: Keypair, amount: number) {
  // Mint WEED tokens to user for testing
  await program.methods.mintTokensForTesting(new BN(amount * 1_000_000))
    .accounts({ user: user.publicKey })
    .signers([admin])
    .rpc();
}

export async function createTokenAccountForUser(user: Keypair) {
  // Create SPL Token 2022 account for user
  const tokenAccount = await createAssociatedTokenAccount(
    connection,
    user,
    rewardMintPDA,
    user.publicKey,
    {},
    TOKEN_2022_PROGRAM_ID
  );
  return tokenAccount;
}
```

### テストアサーション (assertions.ts)

```typescript
export function expectValidFarmState(farmSpace: any) {
  expect(farmSpace.level).toBeGreaterThan(0);
  expect(farmSpace.level).toBeLessThanOrEqual(5);
  expect(farmSpace.seedCount).toBeLessThanOrEqual(farmSpace.capacity);
  expect(farmSpace.totalGrowPower.toNumber()).toBeGreaterThanOrEqual(0);
  
  // Verify level-capacity relationship
  const expectedCapacities = [4, 6, 10, 16, 25];
  expect(farmSpace.capacity).toBe(expectedCapacities[farmSpace.level - 1]);
}

export function expectValidEconomicState(config: any, globalStats: any) {
  expect(globalStats.totalSupply.toNumber()).toBeLessThanOrEqual(240_000_000);
  expect(config.baseRate.toNumber()).toBeGreaterThan(0);
  expect(config.seedPackCost.toNumber()).toBe(300_000_000); // 300 WEED
  expect(config.farmSpaceCostSol.toNumber()).toBe(500_000_000); // 0.5 SOL
}

export function expectValidTransferFeeConfig(mintInfo: any) {
  expect(mintInfo.extensions).toContain('TransferFeeConfig');
  // Verify 2% transfer fee (200 basis points)
  expect(mintInfo.transferFeeConfig.transferFeeBasisPoints).toBe(200);
  expect(mintInfo.transferFeeConfig.maximumFee.toNumber()).toBe(1_000_000_000);
}

export function expectValidProbabilityTable(table: any) {
  expect(table.seedCount).toBeGreaterThan(0);
  expect(table.seedCount).toBeLessThanOrEqual(16);
  
  // Last threshold should be 10000 (100%)
  const lastIndex = table.seedCount - 1;
  expect(table.probabilityThresholds[lastIndex]).toBe(10000);
  
  // Thresholds should be ascending
  for (let i = 1; i < table.seedCount; i++) {
    expect(table.probabilityThresholds[i])
      .toBeGreaterThan(table.probabilityThresholds[i - 1]);
  }
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
anchor test -- --grep "Security Tests"

# 並列実行（高速化）
anchor test --parallel

# カバレッジレポート
anchor test --coverage

# 特定の命令のテスト
anchor test -- --grep "purchase_seed_pack"
```

### CI/CD統合

```yaml
# .github/workflows/test.yml
name: Comprehensive Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Anchor
        run: npm install -g @coral-xyz/anchor-cli
      - name: Install Solana
        run: sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"
      - name: Run unit tests
        run: anchor test -- --grep "Unit Tests"
      - name: Run integration tests
        run: anchor test -- --grep "Integration Tests"
      - name: Run security tests
        run: anchor test -- --grep "Security Tests"
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## 品質メトリクス

### 目標カバレッジ
- **命令カバレッジ**: 100% (全23命令)
- **分岐カバレッジ**: 95%
- **エラーパスカバレッジ**: 100%
- **統合シナリオカバレッジ**: 90%

### パフォーマンス目標
- **平均応答時間**: < 3秒
- **並行処理能力**: 50トランザクション/秒
- **メモリ使用量**: < 1GB per test suite
- **Transfer Fee処理時間**: < 100ms additional overhead

### セキュリティ検証項目
- **PDA検証**: 100%カバレッジ
- **権限制御**: 全権限パターンをテスト
- **入力検証**: 全境界値テスト
- **Transfer Fee検証**: 手数料計算の正確性

この包括的なテスト戦略により、Facility Gameは高い品質と信頼性を維持し、ユーザーに安全で快適なゲーム体験を提供します。SPL Token 2022のTransfer Fee Extension、招待システム、VRF統合、自動農場アップグレードなど、すべての主要機能が完全にテストされます。