// メインアプリケーション

import { SolanaService } from './solana';
import { logger } from './logger';
import {
  GAME_CONSTANTS,
  UI_CONSTANTS,
  SUCCESS_MESSAGES,
  NETWORK_CONSTANTS,
} from './utils/constants';
import type { WalletState, GameState } from './types';

class FacilityGameApp {
  private solanaService: SolanaService;
  private currentWallet: WalletState | null = null;

  constructor() {
    this.solanaService = new SolanaService();
    this.init();
  }

  private async init() {
    logger.info('🚀 TypeScript フロントエンド初期化開始');

    try {
      // イベントリスナー設定（最初に実行）
      this.setupEventListeners();

      // ネットワーク情報表示
      const networkInfo = this.solanaService.getNetworkInfo();
      this.updateNetworkInfo(networkInfo);
      logger.info(`ネットワーク設定: ${networkInfo.network} - ${networkInfo.rpcUrl}`);

      // RPC接続テスト
      logger.info('RPC接続テスト開始...');
      const connectionSuccess = await this.solanaService.testConnection();
      this.updateConnectionStatus(connectionSuccess);

      if (connectionSuccess) {
        logger.success('✅ 初期化完了 - フロントエンド準備OK');
      } else {
        logger.error('❌ RPC接続失敗');
      }
    } catch (error) {
      logger.error(`初期化エラー: ${error instanceof Error ? error.message : String(error)}`);
      this.updateConnectionStatus(false);
    }
  }

  private setupEventListeners() {
    // ウォレット接続
    document.getElementById('connect-wallet')?.addEventListener('click', async () => {
      await this.handleWalletConnect();
    });

    // ゲームアクション（プレースホルダー）
    document.getElementById('init-user')?.addEventListener('click', () => {
      this.handleInitUser();
    });

    document.getElementById('buy-facility')?.addEventListener('click', () => {
      this.handleBuyFacility();
    });

    document.getElementById('claim-rewards')?.addEventListener('click', () => {
      this.handleClaimRewards();
    });

    // データ更新
    document.getElementById('refresh-data')?.addEventListener('click', async () => {
      await this.handleRefreshData();
    });

    // SOLエアドロップ（開発用）
    document.getElementById('airdrop-sol')?.addEventListener('click', async () => {
      await this.handleAirdrop();
    });

    // ログクリア
    document.getElementById('clear-logs')?.addEventListener('click', () => {
      logger.clear();
    });
  }

  private async handleWalletConnect() {
    try {
      const walletState = await this.solanaService.connectWallet();
      this.currentWallet = walletState;
      this.updateWalletDisplay(walletState);
      this.enableGameButtons();

      // ゲーム状態を取得
      await this.handleRefreshData();
    } catch (error) {
      logger.error(`ウォレット接続失敗: ${error instanceof Error ? error.message : String(error)}`);
      this.showError(
        `ウォレット接続に失敗しました: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private async handleRefreshData() {
    if (!this.currentWallet?.connected) {
      logger.warn('ウォレットが接続されていません');
      return;
    }

    try {
      const gameState = await this.solanaService.getGameState();
      this.updateGameDisplay(gameState);
    } catch (error) {
      logger.error(`データ更新エラー: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  private async handleInitUser() {
    logger.info('👤 ユーザー初期化機能');

    try {
      if (!this.solanaService.getWalletState().connected) {
        throw new Error('ウォレットが接続されていません');
      }

      const anchorClient = this.solanaService.getAnchorClient();

      this.showLoading('ユーザーアカウントを初期化中...');

      const tx = await anchorClient.initUser();

      if (tx === 'already_initialized') {
        this.showSuccess('ユーザーアカウントは既に初期化済みです！');
      } else {
        logger.success(`ユーザー初期化成功: ${tx}`);
        this.showSuccess('ユーザーアカウントの初期化が完了しました！');
      }

      // ゲーム状態を更新
      await this.handleRefreshData();
    } catch (error) {
      logger.error(
        `ユーザー初期化エラー: ${error instanceof Error ? error.message : String(error)}`
      );
      this.showError(
        `初期化に失敗しました: ${error instanceof Error ? error.message : String(error)}`
      );
    } finally {
      this.hideLoading();
    }
  }

  private async handleBuyFacility() {
    logger.info('🏭 施設購入機能');

    try {
      if (!this.solanaService.getWalletState().connected) {
        throw new Error('ウォレットが接続されていません');
      }

      const anchorClient = this.solanaService.getAnchorClient();

      this.showLoading('施設を購入中...');

      const tx = await anchorClient.buyFacility();

      if (tx === 'already_owned') {
        this.showSuccess('施設は既に所有済みです！');
      } else {
        logger.success(`施設購入成功: ${tx}`);
        this.showSuccess(
          `施設の購入が完了しました！Grow Power: ${GAME_CONSTANTS.INITIAL_GROW_POWER}`
        );
      }

      // ゲーム状態を更新
      await this.handleRefreshData();
    } catch (error) {
      logger.error(`施設購入エラー: ${error instanceof Error ? error.message : String(error)}`);
      this.showError(
        `購入に失敗しました: ${error instanceof Error ? error.message : String(error)}`
      );
    } finally {
      this.hideLoading();
    }
  }

  private async handleClaimRewards() {
    logger.info('💰 報酬請求機能');

    try {
      if (!this.solanaService.getWalletState().connected) {
        throw new Error('ウォレットが接続されていません');
      }

      const anchorClient = this.solanaService.getAnchorClient();

      this.showLoading('報酬を請求中...');

      const tx = await anchorClient.claimRewards();
      logger.success(`報酬請求成功: ${tx}`);

      // ゲーム状態を更新
      await this.handleRefreshData();

      this.showSuccess('報酬の請求が完了しました！');
    } catch (error) {
      logger.error(`報酬請求エラー: ${error instanceof Error ? error.message : String(error)}`);
      this.showError(
        `請求に失敗しました: ${error instanceof Error ? error.message : String(error)}`
      );
    } finally {
      this.hideLoading();
    }
  }

  private async handleAirdrop() {
    logger.info('💰 SOLエアドロップ（開発用）');

    try {
      if (!this.solanaService.getWalletState().connected) {
        throw new Error('ウォレットが接続されていません');
      }

      this.showLoading('SOLをエアドロップ中...');

      await this.solanaService.airdropSol(NETWORK_CONSTANTS.DEFAULT_AIRDROP_AMOUNT);

      // ウォレット表示を更新
      const walletState = this.solanaService.getWalletState();
      this.updateWalletDisplay(walletState);

      this.showSuccess(
        SUCCESS_MESSAGES.AIRDROP_COMPLETED(NETWORK_CONSTANTS.DEFAULT_AIRDROP_AMOUNT)
      );
    } catch (error) {
      logger.error(`エアドロップエラー: ${error instanceof Error ? error.message : String(error)}`);
      this.showError(
        `エアドロップに失敗しました: ${error instanceof Error ? error.message : String(error)}`
      );
    } finally {
      this.hideLoading();
    }
  }

  private showLoading(message: string) {
    const statusEl = document.getElementById('status-message');
    if (statusEl) {
      statusEl.innerHTML = `<div class="info-message">${message}</div>`;
    }
  }

  private hideLoading() {
    const statusEl = document.getElementById('status-message');
    if (statusEl) {
      statusEl.innerHTML = '';
    }
  }

  private showSuccess(message: string) {
    const statusEl = document.getElementById('status-message');
    if (statusEl) {
      statusEl.innerHTML = `<div class="success">${message}</div>`;
      setTimeout(() => this.hideLoading(), UI_CONSTANTS.SUCCESS_MESSAGE_DURATION);
    }
  }

  // UI更新メソッド
  private updateConnectionStatus(connected: boolean) {
    const statusEl = document.getElementById('connection-status');
    if (!statusEl) return;

    if (connected) {
      statusEl.innerHTML =
        '<span class="status-indicator status-connected"></span>Helius RPC 接続済み';
    } else {
      statusEl.innerHTML = '<span class="status-indicator status-disconnected"></span>接続失敗';
    }
  }

  private updateNetworkInfo(networkInfo: { network: string; rpcUrl: string; programId: string }) {
    const rpcUrlEl = document.getElementById('rpc-url');
    const networkEl = document.getElementById('network');

    if (rpcUrlEl) rpcUrlEl.textContent = networkInfo.rpcUrl;
    if (networkEl) networkEl.textContent = networkInfo.network.toUpperCase();
  }

  private updateWalletDisplay(walletState: WalletState) {
    const statusEl = document.getElementById('wallet-status');
    const addressEl = document.getElementById('wallet-address');
    const balanceEl = document.getElementById('wallet-balance');
    const infoEl = document.getElementById('wallet-info');

    if (statusEl) {
      statusEl.innerHTML = walletState.connected
        ? '<span class="status-indicator status-connected"></span>接続済み'
        : '<span class="status-indicator status-disconnected"></span>未接続';
    }

    if (walletState.connected && walletState.publicKey) {
      if (addressEl) addressEl.textContent = walletState.publicKey.toString();
      if (balanceEl) balanceEl.textContent = walletState.balance.toFixed(4);
      if (infoEl) infoEl.style.display = 'block';
    } else {
      if (infoEl) infoEl.style.display = 'none';
    }
  }

  private updateGameDisplay(gameState: GameState) {
    const userStateEl = document.getElementById('user-state');
    const facilityStateEl = document.getElementById('facility-state');
    const growPowerEl = document.getElementById('grow-power');
    const tokenBalanceEl = document.getElementById('token-balance');

    if (userStateEl) {
      userStateEl.textContent = gameState.userInitialized ? '初期化済み' : '未初期化';
    }

    if (facilityStateEl) {
      facilityStateEl.textContent = gameState.hasFacility ? '所有済み' : '未所有';
    }

    if (growPowerEl) {
      growPowerEl.textContent = gameState.growPower.toString();
    }

    if (tokenBalanceEl) {
      // トークンの decimals を定数から取得
      const tokenBalance = gameState.tokenBalance / Math.pow(10, GAME_CONSTANTS.TOKEN_DECIMALS);
      tokenBalanceEl.textContent = `${tokenBalance.toFixed(UI_CONSTANTS.TOKEN_DECIMAL_PLACES)} ${GAME_CONSTANTS.TOKEN_SYMBOL}`;
    }
  }

  private enableGameButtons() {
    const buttons = ['init-user', 'buy-facility', 'claim-rewards', 'airdrop-sol'];
    buttons.forEach((id) => {
      const button = document.getElementById(id) as HTMLButtonElement;
      if (button) button.disabled = false;
    });
  }

  private showError(message: string) {
    logger.error(message);

    // UIにもエラーメッセージを表示
    const statusEl = document.getElementById('status-message');
    if (statusEl) {
      let displayMessage = message;

      // 残高不足の場合の案内を追加
      if (
        message.includes('Attempt to debit an account but found no record of a prior credit') ||
        message.includes('insufficient funds')
      ) {
        displayMessage =
          '残高不足です。「SOLエアドロップ (開発用)」ボタンでSOLを取得してから再試行してください。';
      }

      statusEl.innerHTML = `<div class="error">${displayMessage}</div>`;
      setTimeout(() => this.hideLoading(), UI_CONSTANTS.ERROR_MESSAGE_DURATION);
    }
  }
}

// DOM読み込み完了後にアプリを初期化 (テスト環境では実行しない)
if (typeof document !== 'undefined') {
  document.addEventListener('DOMContentLoaded', () => {
    new FacilityGameApp();

    // デバッグ用グローバル関数
    (window as any).testHeliusConnection = async () => {
      const service = new SolanaService();
      return await service.testConnection();
    };
  });
}

// TypeScriptのエクスポート（必要に応じて）
export { FacilityGameApp };
