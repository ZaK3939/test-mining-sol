// Game Service - Game logic abstraction layer
// ゲームロジックの抽象化レイヤー

import { PublicKey } from '@solana/web3.js';
import { AnchorClient } from '../anchor-client';
import { SolanaService } from '../solana';
import { logger } from '../logger';
import {
  executeTransaction,
  executeTransactionWithSpecialReturns,
  requireWalletConnection,
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

  // Config initialization (admin only)
  async initializeConfig(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().initializeConfig(), {
          operationName: '設定初期化',
          successMessage: '設定が初期化されました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Reward mint creation (admin only)
  async createRewardMint(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().createRewardMint(), {
          operationName: '報酬ミント作成',
          successMessage: '報酬ミントが作成されました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // User initialization
  async initializeUser(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransactionWithSpecialReturns(() => this.getAnchorClient().initUser(), {
          operationName: 'ユーザー初期化',
          successMessage: SUCCESS_MESSAGES.USER_INITIALIZED,
          specialReturns: {
            already_initialized: 'ユーザーアカウントは既に初期化済みです！',
          },
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
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
              already_initialized: 'ユーザーアカウントは既に初期化済みです！',
            },
            onSuccess: callbacks.updateGameState,
            ...callbacks,
          }
        );
      },
      callbacks.showError
    );
  }

  // Farm space purchase
  async purchaseFarmSpace(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransactionWithSpecialReturns(
          () => this.getAnchorClient().buyFarmSpace(),
          {
            operationName: '農場スペース購入',
            successMessage: `農場スペースの購入が完了しました！初期シードが付与されました`,
            specialReturns: {
              already_owned: '農場スペースは既に所有済みです！',
            },
            onSuccess: callbacks.updateGameState,
            ...callbacks,
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
        return await executeTransaction(() => this.getAnchorClient().claimRewards(), {
          operationName: '報酬請求',
          successMessage: SUCCESS_MESSAGES.REWARDS_CLAIMED,
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Referral rewards claiming
  async claimReferralRewards(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().claimReferralRewards(), {
          operationName: '紹介報酬請求',
          successMessage: '紹介報酬の請求が完了しました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Farm space upgrade (start upgrade with 24h cooldown)
  async upgradeFarmSpace(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().upgradeFarmSpace(), {
          operationName: '農場スペースアップグレード開始',
          successMessage: '農場スペースのアップグレードを開始しました！24時間後に完了できます',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Complete farm space upgrade (after 24h cooldown)
  async completeFarmSpaceUpgrade(callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().completeFarmSpaceUpgrade(), {
          operationName: '農場スペースアップグレード完了',
          successMessage: '農場スペースのアップグレードが完了しました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Plant seed in farm space
  async plantSeed(seedId: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().plantSeed(seedId), {
          operationName: 'シード植え付け',
          successMessage: 'シードの植え付けが完了しました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Remove seed from farm space
  async removeSeed(seedId: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().removeSeed(seedId), {
          operationName: 'シード除去',
          successMessage: 'シードの除去が完了しました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Token transfer with fee
  async transferTokens(
    recipient: PublicKey,
    amount: number,
    callbacks: UICallbacks
  ): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(
          () => this.getAnchorClient().transferWithFee(recipient, amount),
          {
            operationName: 'トークン転送',
            successMessage: 'トークンの転送が完了しました！',
            onSuccess: callbacks.updateGameState,
            ...callbacks,
          }
        );
      },
      callbacks.showError
    );
  }

  // Seed pack purchase
  async purchaseSeedPack(quantity: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().purchaseSeedPack(quantity), {
          operationName: 'シードパック購入',
          successMessage: `シードパック${quantity}個の購入が完了しました！`,
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Seed pack opening
  async openSeedPack(packId: number, quantity: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().openSeedPack(packId, quantity), {
          operationName: 'シードパック開封',
          successMessage: 'シードパックの開封が完了しました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Create invite code
  async createInviteCode(inviteCode: string, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().createInviteCode(inviteCode), {
          operationName: '招待コード作成',
          successMessage: `招待コード「${inviteCode}」が作成されました！`,
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Use invite code
  async useInviteCode(inviteCode: string, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().useInviteCode(inviteCode), {
          operationName: '招待コード使用',
          successMessage: `招待コード「${inviteCode}」を使用してアカウントを作成しました！`,
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // Distribute referral rewards
  async distributeReferralRewards(baseReward: number, callbacks: UICallbacks): Promise<string> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.getAnchorClient().distributeReferralRewards(baseReward), {
          operationName: '紹介報酬配布',
          successMessage: '紹介報酬の配布が完了しました！',
          onSuccess: callbacks.updateGameState,
          ...callbacks,
        });
      },
      callbacks.showError
    );
  }

  // SOL airdrop (development)
  async requestAirdrop(amount: number, callbacks: UICallbacks): Promise<void> {
    return await requireWalletConnection(
      this.solanaService.getWalletState(),
      async () => {
        return await executeTransaction(() => this.solanaService.airdropSol(amount), {
          operationName: 'SOLエアドロップ',
          successMessage: SUCCESS_MESSAGES.AIRDROP_COMPLETED(amount),
          onSuccess: async () => {
            // Update wallet display instead of game state
            // This would be handled by the UI layer
          },
          ...callbacks,
        });
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
        farmSpace: null,
        config: null,
        tokenBalance: 0,
        userInitialized: false,
        hasFarmSpace: false,
        growPower: 0,
        pendingReferralRewards: 0,
      };
    }

    try {
      return await this.getAnchorClient().fetchCompleteGameState(walletState.publicKey);
    } catch (error) {
      logger.error(
        `ゲーム状態取得エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      // Return default state on error
      return {
        userState: null,
        farmSpace: null,
        config: null,
        tokenBalance: 0,
        userInitialized: false,
        hasFarmSpace: false,
        growPower: 0,
        pendingReferralRewards: 0,
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
