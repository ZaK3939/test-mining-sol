/**
 * Comprehensive service mocks for testing
 * Provides realistic mock implementations of all major services
 */

import { vi, type MockedFunction } from 'vitest';
import { PublicKey } from '@solana/web3.js';
import type { Connection } from '@solana/web3.js';
import type { WalletAdapter } from '../../types/program-types';
import type { UICallbacks } from '../../types';

// =============== WALLET MOCKS ===============

export interface MockWalletState {
  isConnected: boolean;
  publicKey: PublicKey | null;
  connecting: boolean;
  disconnecting: boolean;
  balance?: number;
}

export class MockWalletService {
  private state: MockWalletState = {
    isConnected: false,
    publicKey: null,
    connecting: false,
    disconnecting: false,
    balance: 0,
  };

  // Mock methods
  connect = vi.fn().mockImplementation(async () => {
    this.state.connecting = true;
    await new Promise(resolve => setTimeout(resolve, 100)); // Simulate connection delay
    this.state.isConnected = true;
    this.state.connecting = false;
    return { publicKey: this.state.publicKey };
  });

  disconnect = vi.fn().mockImplementation(async () => {
    this.state.disconnecting = true;
    await new Promise(resolve => setTimeout(resolve, 50));
    this.state.isConnected = false;
    this.state.publicKey = null;
    this.state.disconnecting = false;
  });

  getWalletState = vi.fn().mockImplementation(() => ({ ...this.state }));

  // Test utilities
  setMockState(newState: Partial<MockWalletState>) {
    this.state = { ...this.state, ...newState };
  }

  simulateConnection(publicKey: PublicKey, balance = 2.5) {
    this.state = {
      isConnected: true,
      publicKey,
      connecting: false,
      disconnecting: false,
      balance,
    };
  }

  simulateDisconnection() {
    this.state = {
      isConnected: false,
      publicKey: null,
      connecting: false,
      disconnecting: false,
    };
  }

  simulateConnectionError() {
    this.connect.mockRejectedValueOnce(new Error('User rejected the request'));
  }
}

// =============== ANCHOR CLIENT MOCKS ===============

export interface MockGameState {
  hasUserState: boolean;
  hasFarmSpace: boolean;
  farmLevel: number;
  farmCapacity: number;
  seedCount: number;
  totalGrowPower: number;
  lastHarvestTime: number;
  claimableRewards: number;
  balance: number;
}

export class MockAnchorClient {
  private gameState: MockGameState = {
    hasUserState: false,
    hasFarmSpace: false,
    farmLevel: 1,
    farmCapacity: 4,
    seedCount: 1,
    totalGrowPower: 100,
    lastHarvestTime: Date.now() - 3600000, // 1 hour ago
    claimableRewards: 150.5,
    balance: 1000,
  };

  // User management
  initializeUser = vi.fn().mockImplementation(async (_referrer?: string) => {
    await this.simulateTransaction();
    this.gameState.hasUserState = true;
    return 'user_initialized_signature';
  });

  // Farm space management
  buyFarmSpace = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    this.gameState.hasFarmSpace = true;
    this.gameState.farmLevel = 1;
    this.gameState.farmCapacity = 4;
    this.gameState.seedCount = 1; // Initial seed
    return 'farm_space_purchased_signature';
  });

  upgradeFarmSpace = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    if (this.gameState.farmLevel < 5) {
      this.gameState.farmLevel += 1;
      this.gameState.farmCapacity = this.gameState.farmLevel * 4;
    }
    return 'farm_space_upgraded_signature';
  });

  completeFarmSpaceUpgrade = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    return 'upgrade_completed_signature';
  });

  // Rewards
  claimReward = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    const rewards = this.gameState.claimableRewards;
    this.gameState.claimableRewards = 0;
    this.gameState.lastHarvestTime = Date.now();
    this.gameState.balance += rewards;
    return 'rewards_claimed_signature';
  });

  claimRewardWithReferralRewards = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    const rewards = this.gameState.claimableRewards;
    const referralRewards = 25; // Mock referral rewards
    this.gameState.claimableRewards = 0;
    this.gameState.lastHarvestTime = Date.now();
    this.gameState.balance += rewards + referralRewards;
    return 'rewards_with_referral_claimed_signature';
  });

  // Seed management
  purchaseSeedPack = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    this.gameState.balance -= 300; // Seed pack cost
    return 'seed_pack_purchased_signature';
  });

  openSeedPack = vi.fn().mockImplementation(async (_packId: string) => {
    await this.simulateTransaction();
    const seedType = Math.floor(Math.random() * 9) + 1; // Random seed 1-9
    return { seedType, signature: 'seed_pack_opened_signature' };
  });

  plantSeed = vi.fn().mockImplementation(async (_seedId: string) => {
    await this.simulateTransaction();
    if (this.gameState.seedCount < this.gameState.farmCapacity) {
      this.gameState.seedCount += 1;
      this.gameState.totalGrowPower += 100; // Base grow power
    }
    return 'seed_planted_signature';
  });

  removeSeed = vi.fn().mockImplementation(async (_seedId: string) => {
    await this.simulateTransaction();
    if (this.gameState.seedCount > 0) {
      this.gameState.seedCount -= 1;
      this.gameState.totalGrowPower -= 100;
    }
    return 'seed_removed_signature';
  });

  // Referral system
  createInviteCode = vi.fn().mockImplementation(async (_code: string) => {
    await this.simulateTransaction();
    return 'invite_code_created_signature';
  });

  useInviteCode = vi.fn().mockImplementation(async (_code: string) => {
    await this.simulateTransaction();
    return 'invite_code_used_signature';
  });

  claimReferralRewards = vi.fn().mockImplementation(async () => {
    await this.simulateTransaction();
    const rewards = 50; // Mock referral rewards
    this.gameState.balance += rewards;
    return 'referral_rewards_claimed_signature';
  });

  // State queries
  getUserState = vi.fn().mockImplementation(async () => {
    if (!this.gameState.hasUserState) {
      throw new Error('User not initialized');
    }
    return {
      owner: new PublicKey('11111111111111111111111111111111'),
      totalGrowPower: this.gameState.totalGrowPower,
      lastHarvestTime: this.gameState.lastHarvestTime,
      hasFarmSpace: this.gameState.hasFarmSpace,
      referrer: null,
      pendingReferralRewards: 25,
    };
  });

  getFarmSpace = vi.fn().mockImplementation(async () => {
    if (!this.gameState.hasFarmSpace) {
      throw new Error('No farm space');
    }
    return {
      owner: new PublicKey('11111111111111111111111111111111'),
      level: this.gameState.farmLevel,
      capacity: this.gameState.farmCapacity,
      seedCount: this.gameState.seedCount,
      totalGrowPower: this.gameState.totalGrowPower,
      upgradeStartTime: 0,
      upgradeTargetLevel: 0,
    };
  });

  getSeeds = vi.fn().mockImplementation(async () => {
    return Array(this.gameState.seedCount)
      .fill(null)
      .map((_, i) => ({
        id: i + 1,
        seedType: 1,
        growPower: 100,
        isPlanted: true,
      }));
  });

  getClaimableReward = vi.fn().mockImplementation(async () => {
    return this.gameState.claimableRewards;
  });

  getTokenBalance = vi.fn().mockImplementation(async () => {
    return this.gameState.balance;
  });

  // Test utilities
  private async simulateTransaction() {
    await new Promise(resolve => setTimeout(resolve, 100 + Math.random() * 200));
  }

  setMockGameState(newState: Partial<MockGameState>) {
    this.gameState = { ...this.gameState, ...newState };
  }

  resetMockState() {
    this.gameState = {
      hasUserState: false,
      hasFarmSpace: false,
      farmLevel: 1,
      farmCapacity: 4,
      seedCount: 1,
      totalGrowPower: 100,
      lastHarvestTime: Date.now() - 3600000,
      claimableRewards: 150.5,
      balance: 1000,
    };
  }

  simulateError(method: keyof MockAnchorClient, error: Error) {
    if (typeof this[method] === 'function') {
      (this[method] as MockedFunction<any>).mockRejectedValueOnce(error);
    }
  }

  simulateSpecialReturn(method: keyof MockAnchorClient, specialCase: string) {
    if (typeof this[method] === 'function') {
      (this[method] as MockedFunction<any>).mockRejectedValueOnce(
        new Error(`Special case: ${specialCase}`)
      );
    }
  }
}

// =============== GAME SERVICE MOCKS ===============

export class MockGameService {
  private mockAnchorClient: MockAnchorClient;

  constructor() {
    this.mockAnchorClient = new MockAnchorClient();
  }

  // User operations
  initializeUser = vi.fn().mockImplementation(async (callbacks: UICallbacks) => {
    try {
      const result = await this.mockAnchorClient.initializeUser('');
      callbacks.showSuccess?.('ユーザーアカウントが初期化されました！');
      callbacks.updateGameState?.();
      return result;
    } catch (error) {
      callbacks.showError?.(`初期化エラー: ${error}`);
      throw error;
    }
  });

  purchaseFarmSpace = vi.fn().mockImplementation(async (callbacks: UICallbacks) => {
    try {
      const result = await this.mockAnchorClient.buyFarmSpace();
      callbacks.showSuccess?.('農場スペースを購入しました！');
      callbacks.updateGameState?.();
      return result;
    } catch (error) {
      if (error instanceof Error && error.message.includes('already_owned')) {
        return '農場スペースは既に所有済みです！';
      }
      callbacks.showError?.(`購入エラー: ${error}`);
      throw error;
    }
  });

  claimRewards = vi.fn().mockImplementation(async (callbacks: UICallbacks) => {
    try {
      const rewards = await this.mockAnchorClient.getClaimableReward();
      if (rewards === 0) {
        callbacks.showInfo?.('請求可能な報酬がありません');
        return;
      }
      const result = await this.mockAnchorClient.claimReward('');
      callbacks.showSuccess?.(`${rewards.toFixed(2)} WEEDを獲得しました！`);
      callbacks.updateGameState?.();
      return result;
    } catch (error) {
      callbacks.showError?.(`報酬請求エラー: ${error}`);
      throw error;
    }
  });

  // Seed operations
  purchaseMysteryPack = vi.fn().mockImplementation(async (callbacks: UICallbacks) => {
    try {
      const result = await this.mockAnchorClient.purchaseSeedPack('');
      callbacks.showSuccess?.('ミステリーパックを購入しました！');
      callbacks.updateGameState?.();
      return result;
    } catch (error) {
      callbacks.showError?.(`パック購入エラー: ${error}`);
      throw error;
    }
  });

  // Get the mock client for test setup
  getAnchorClient(): MockAnchorClient {
    return this.mockAnchorClient;
  }

  resetMockState() {
    this.mockAnchorClient.resetMockState();
  }
}

// =============== SOLANA SERVICE MOCKS ===============

export class MockSolanaService {
  private mockWalletService: MockWalletService;

  constructor() {
    this.mockWalletService = new MockWalletService();
  }

  connect = vi.fn().mockImplementation(async () => {
    return this.mockWalletService.connect();
  });

  disconnect = vi.fn().mockImplementation(async () => {
    return this.mockWalletService.disconnect();
  });

  getWalletState = vi.fn().mockImplementation(() => {
    return this.mockWalletService.getWalletState();
  });

  getConnection = vi.fn().mockImplementation(() => {
    return {
      getSlot: vi.fn().mockResolvedValue(12345),
      getBalance: vi.fn().mockResolvedValue(2500000000), // 2.5 SOL
      requestAirdrop: vi.fn().mockResolvedValue('airdrop_signature'),
      confirmTransaction: vi.fn().mockResolvedValue({ value: { err: null } }),
    } as Partial<Connection>;
  });

  getWalletAdapter = vi.fn().mockImplementation(() => {
    return {
      publicKey: this.mockWalletService.getWalletState().publicKey,
      signTransaction: vi.fn(),
      signAllTransactions: vi.fn(),
    } as Partial<WalletAdapter>;
  });

  // Test utilities
  getMockWalletService(): MockWalletService {
    return this.mockWalletService;
  }
}

// =============== MOCK FACTORY ===============

export interface MockServices {
  walletService: MockWalletService;
  anchorClient: MockAnchorClient;
  gameService: MockGameService;
  solanaService: MockSolanaService;
}

export class MockServiceFactory {
  static createFullMockSuite(): MockServices {
    const solanaService = new MockSolanaService();
    const gameService = new MockGameService();
    
    return {
      walletService: solanaService.getMockWalletService(),
      anchorClient: gameService.getAnchorClient(),
      gameService,
      solanaService,
    };
  }

  static resetAllMocks(services: MockServices) {
    // Reset all vi.fn() mocks
    Object.values(services).forEach(service => {
      Object.values(service).forEach(method => {
        if (vi.isMockFunction(method)) {
          method.mockClear();
        }
      });
    });

    // Reset state
    services.walletService.simulateDisconnection();
    services.gameService.resetMockState();
  }
}