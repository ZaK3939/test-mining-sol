// Game Service - Game logic abstraction layer
// ゲームロジックの抽象化レイヤー

import { PublicKey } from '@solana/web3.js';
import { AnchorClient } from '../anchor-client';
import { SolanaService } from '../solana';
import { logger } from '../logger';
import { 
  executeTransaction, 
  executeTransactionWithSpecialReturns,
  requireWalletConnection 
} from '../utils/error-handler';
import { SUCCESS_MESSAGES, GAME_CONSTANTS } from '../utils/constants';
import type { DetailedGameState } from '../types';

export interface UICallbacks {
  showLoading: (message: string) => void;
  hideLoading: () => void;
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
  updateGameState: () => Promise<void>;
}

export class GameService {
  private solanaService: SolanaService;
  private anchorClient: AnchorClient | null = null;

  constructor(solanaService: SolanaService) {
    this.solanaService = solanaService;
  }

  private getAnchorClient(): AnchorClient {
    if (!this.anchorClient) {
      this.anchorClient = this.solanaService.getAnchorClient();
    }
    return this.anchorClient;
  }

  // User initialization
  async initializeUser(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransactionWithSpecialReturns(
          () => this.getAnchorClient().initUser(),
          {
            operationName: 'ユーザー初期化',
            successMessage: SUCCESS_MESSAGES.USER_INITIALIZED,
            specialReturns: {
              'already_initialized': 'ユーザーアカウントは既に初期化済みです！'
            },
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // User initialization with referrer
  async initializeUserWithReferrer(referrer: PublicKey, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransactionWithSpecialReturns(
          () => this.getAnchorClient().initUserWithReferrer(referrer),
          {
            operationName: '紹介者付きユーザー初期化',
            successMessage: '紹介者付きでユーザーアカウントが初期化されました！',
            specialReturns: {
              'already_initialized': 'ユーザーアカウントは既に初期化済みです！'
            },
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Facility purchase
  async purchaseFacility(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransactionWithSpecialReturns(
          () => this.getAnchorClient().buyFacility(),
          {
            operationName: '施設購入',
            successMessage: `施設の購入が完了しました！Grow Power: ${GAME_CONSTANTS.INITIAL_GROW_POWER}`,
            specialReturns: {
              'already_owned': '施設は既に所有済みです！'
            },
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Rewards claiming
  async claimRewards(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().claimRewards(),
          {
            operationName: '報酬請求',
            successMessage: SUCCESS_MESSAGES.REWARDS_CLAIMED,
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Referral rewards claiming
  async claimReferralRewards(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().claimReferralRewards(),
          {
            operationName: '紹介報酬請求',
            successMessage: '紹介報酬の請求が完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Facility upgrade
  async upgradeFacility(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().upgradeFacility(),
          {
            operationName: '施設アップグレード',
            successMessage: '施設のアップグレードが完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Add machine
  async addMachine(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().addMachine(),
          {
            operationName: 'マシン追加',
            successMessage: 'マシンの追加が完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Token transfer with fee
  async transferTokens(recipient: PublicKey, amount: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().transferWithFee(recipient, amount),
          {
            operationName: 'トークン転送',
            successMessage: 'トークンの転送が完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Mystery box purchase
  async purchaseMysteryBox(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().purchaseMysteryBox(),
          {
            operationName: 'ミステリーボックス購入',
            successMessage: 'ミステリーボックスの購入が完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Mystery box opening
  async openMysteryBox(mysteryBoxId: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().openMysteryBox(mysteryBoxId),
          {
            operationName: 'ミステリーボックス開封',
            successMessage: 'ミステリーボックスの開封が完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // SOL airdrop (development)
  async requestAirdrop(amount: number, callbacks: UICallbacks): Promise<void> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.solanaService.airdropSol(amount),
          {
            operationName: 'SOLエアドロップ',
            successMessage: SUCCESS_MESSAGES.AIRDROP_COMPLETED(amount),
            onSuccess: async () => {
              // Update wallet display instead of game state
              // This would be handled by the UI layer
            },
            ...callbacks
          }
        );
      },
      callbacks.showError
    );
  }

  // Get current game state
  async getGameState(): Promise<DetailedGameState> {
    const walletState = this.solanaService.getWalletState();
    if (!walletState.connected || !walletState.publicKey) {
      return {
        userState: null,
        facility: null,
        config: null,
        tokenBalance: 0,
        userInitialized: false,
        hasFacility: false,
        growPower: 0,
        pendingReferralRewards: 0
      };
    }

    try {
      return await this.getAnchorClient().fetchCompleteGameState(walletState.publicKey);
    } catch (error) {
      logger.error(`ゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`);
      // Return default state on error
      return {
        userState: null,
        facility: null,
        config: null,
        tokenBalance: 0,
        userInitialized: false,
        hasFacility: false,
        growPower: 0,
        pendingReferralRewards: 0
      };
    }
  }

  // Invalidate cache when needed
  invalidateCache(): void {
    const walletState = this.solanaService.getWalletState();
    if (walletState.connected && walletState.publicKey) {
      this.getAnchorClient().invalidateUserCache(walletState.publicKey);
    }
  }
}