// メインアプリケーション

import { PublicKey } from '@solana/web3.js';
import { SolanaService } from './solana';
import { GameService, type UICallbacks } from './services/game-service';
import { logger } from './logger';
import {
  GAME_CONSTANTS,
  UI_CONSTANTS,
  NETWORK_CONSTANTS,
} from './utils/constants';
import type { WalletState, GameState } from './types';

class FacilityGameApp {
  private solanaService: SolanaService;
  private gameService: GameService;
  private currentWallet: WalletState | null = null;
  private uiCallbacks: UICallbacks;

  constructor() {
    this.solanaService = new SolanaService();
    this.gameService = new GameService(this.solanaService);
    
    // UI コールバックを初期化
    this.uiCallbacks = {
      showLoading: (message: string) => this.showLoading(message),
      hideLoading: () => this.hideLoading(),
      showSuccess: (message: string) => this.showSuccess(message),
      showError: (message: string) => this.showError(message),
      updateGameState: () => this.handleRefreshData()
    };
    
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

    // 基本ゲームアクション
    document.getElementById('init-user')?.addEventListener('click', () => {
      this.handleInitUser();
    });

    document.getElementById('buy-facility')?.addEventListener('click', () => {
      this.handleBuyFacility();
    });

    document.getElementById('claim-rewards')?.addEventListener('click', () => {
      this.handleClaimRewards();
    });

    // 紹介システム
    document.getElementById('claim-referral-rewards')?.addEventListener('click', () => {
      this.handleClaimReferralRewards();
    });

    // 施設管理
    document.getElementById('upgrade-facility')?.addEventListener('click', () => {
      this.handleUpgradeFacility();
    });

    document.getElementById('add-machine')?.addEventListener('click', () => {
      this.handleAddMachine();
    });

    // 転送システム
    document.getElementById('transfer-tokens')?.addEventListener('click', () => {
      this.handleTransferTokens();
    });

    // ミステリーボックス
    document.getElementById('purchase-mystery-box')?.addEventListener('click', () => {
      this.handlePurchaseMysteryBox();
    });

    document.getElementById('open-mystery-box')?.addEventListener('click', () => {
      this.handleOpenMysteryBox();
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
      const detailedState = await this.gameService.getGameState();
      // Convert to UI-compatible GameState
      const gameState: GameState = {
        userInitialized: detailedState.userInitialized,
        hasFacility: detailedState.hasFacility,
        growPower: detailedState.growPower,
        tokenBalance: detailedState.tokenBalance,
        lastHarvestTime: detailedState.userState?.lastHarvestTime.toNumber() || 0,
        pendingReferralRewards: detailedState.pendingReferralRewards,
        facility: detailedState.facility ? {
          facilitySize: detailedState.facility.facilitySize,
          maxCapacity: detailedState.facility.maxCapacity,
          machineCount: detailedState.facility.machineCount,
          totalGrowPower: detailedState.facility.totalGrowPower.toNumber()
        } : undefined
      };
      this.updateGameDisplay(gameState);
    } catch (error) {
      logger.error(`データ更新エラー: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  private async handleInitUser() {
    logger.info('👤 ユーザー初期化機能');
    await this.gameService.initializeUser(this.uiCallbacks);
  }

  private async handleBuyFacility() {
    logger.info('🏭 施設購入機能');
    await this.gameService.purchaseFacility(this.uiCallbacks);
  }

  private async handleClaimRewards() {
    logger.info('💰 報酬請求機能');
    await this.gameService.claimRewards(this.uiCallbacks);
  }

  private async handleAirdrop() {
    logger.info('💰 SOLエアドロップ（開発用）');
    await this.gameService.requestAirdrop(NETWORK_CONSTANTS.DEFAULT_AIRDROP_AMOUNT, {
      ...this.uiCallbacks,
      updateGameState: async () => {
        // エアドロップ後はウォレット表示を更新
        const walletState = this.solanaService.getWalletState();
        this.updateWalletDisplay(walletState);
      }
    });
  }

  private async handleClaimReferralRewards() {
    logger.info('💰 紹介報酬請求機能');
    await this.gameService.claimReferralRewards(this.uiCallbacks);
  }

  private async handleUpgradeFacility() {
    logger.info('🔧 施設アップグレード機能');
    await this.gameService.upgradeFacility(this.uiCallbacks);
  }

  private async handleAddMachine() {
    logger.info('⚙️ マシン追加機能');
    await this.gameService.addMachine(this.uiCallbacks);
  }

  private async handleTransferTokens() {
    logger.info('💸 トークン転送機能');

    try {
      // 簡単な例として固定値を使用（実際のUIでは入力フィールドから取得）
      const recipientAddress = prompt('送信先アドレスを入力してください:');
      const amount = prompt('送信量を入力してください:');

      if (!recipientAddress || !amount) {
        throw new Error('アドレスまたは送信量が入力されていません');
      }

      const recipientPublicKey = new PublicKey(recipientAddress);
      await this.gameService.transferTokens(recipientPublicKey, parseInt(amount), this.uiCallbacks);
    } catch (error) {
      this.showError(`入力エラー: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  private async handlePurchaseMysteryBox() {
    logger.info('📦 ミステリーボックス購入機能');
    await this.gameService.purchaseMysteryBox(this.uiCallbacks);
  }

  private async handleOpenMysteryBox() {
    logger.info('📦 ミステリーボックス開封機能');

    try {
      // 簡単な例として固定値を使用（実際のUIでは選択リストから取得）
      const mysteryBoxId = prompt('開封するミステリーボックスのIDを入力してください:');

      if (!mysteryBoxId) {
        throw new Error('ミステリーボックスIDが入力されていません');
      }

      await this.gameService.openMysteryBox(parseInt(mysteryBoxId), this.uiCallbacks);
    } catch (error) {
      this.showError(`入力エラー: ${error instanceof Error ? error.message : String(error)}`);
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
    const facilityDetailsEl = document.getElementById('facility-details');
    const referralRewardsEl = document.getElementById('referral-rewards');

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

    // 施設詳細情報の更新
    if (facilityDetailsEl && gameState.facility) {
      facilityDetailsEl.innerHTML = `
        <div><strong>サイズ:</strong> ${gameState.facility.facilitySize}</div>
        <div><strong>最大容量:</strong> ${gameState.facility.maxCapacity}</div>
        <div><strong>マシン数:</strong> ${gameState.facility.machineCount}</div>
      `;
    } else if (facilityDetailsEl) {
      facilityDetailsEl.innerHTML = '<div>施設なし</div>';
    }

    // 紹介報酬の更新
    if (referralRewardsEl && gameState.pendingReferralRewards !== undefined) {
      const referralRewards = gameState.pendingReferralRewards / Math.pow(10, GAME_CONSTANTS.TOKEN_DECIMALS);
      referralRewardsEl.textContent = `${referralRewards.toFixed(UI_CONSTANTS.TOKEN_DECIMAL_PLACES)} ${GAME_CONSTANTS.TOKEN_SYMBOL}`;
    }
  }

  private enableGameButtons() {
    const buttons = [
      'init-user', 'buy-facility', 'claim-rewards', 'airdrop-sol',
      'claim-referral-rewards', 'upgrade-facility', 'add-machine', 
      'transfer-tokens', 'purchase-mystery-box', 'open-mystery-box'
    ];
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
    
    // Manual test function for browser console
    (window as any).runManualTest = async () => {
      const { runManualFrontendTest } = await import('./test/manual-frontend-test');
      return await runManualFrontendTest();
    };
    
    console.log('🎮 Facility Game Frontend loaded!');
    console.log('💡 Available console commands:');
    console.log('  - testHeliusConnection(): Test RPC connection');
    console.log('  - runManualTest(): Run comprehensive frontend test');
  });
}

// TypeScriptのエクスポート（必要に応じて）
export { FacilityGameApp };
