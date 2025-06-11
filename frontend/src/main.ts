// メインアプリケーション

import { PublicKey } from '@solana/web3.js';
import { SolanaService } from './solana';
import { GameService, type UICallbacks } from './services/game-service';
import { logger } from './logger';
import { GAME_CONSTANTS, UI_CONSTANTS, NETWORK_CONSTANTS } from './utils/constants';
import type { WalletState, GameState } from './types';

class FarmGameApp {
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
      updateGameState: () => this.handleRefreshData(),
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

    // 管理者アクション
    document.getElementById('init-config')?.addEventListener('click', () => {
      this.handleInitConfig();
    });

    document.getElementById('create-reward-mint')?.addEventListener('click', () => {
      this.handleCreateRewardMint();
    });

    // 基本ゲームアクション
    document.getElementById('init-user')?.addEventListener('click', () => {
      this.handleInitUser();
    });

    document.getElementById('buy-farm-space')?.addEventListener('click', () => {
      this.handleBuyFarmSpace();
    });

    document.getElementById('claim-rewards')?.addEventListener('click', () => {
      this.handleClaimRewards();
    });

    // 紹介システム
    document.getElementById('claim-referral-rewards')?.addEventListener('click', () => {
      this.handleClaimReferralRewards();
    });

    // ファームスペース管理
    document.getElementById('upgrade-farm-space')?.addEventListener('click', () => {
      this.handleUpgradeFarmSpace();
    });

    document.getElementById('complete-upgrade')?.addEventListener('click', () => {
      this.handleCompleteUpgrade();
    });

    // 転送システム
    document.getElementById('transfer-tokens')?.addEventListener('click', () => {
      this.handleTransferTokens();
    });

    // シードパック
    document.getElementById('purchase-seed-pack')?.addEventListener('click', () => {
      this.handlePurchaseSeedPack();
    });

    document.getElementById('open-seed-pack')?.addEventListener('click', () => {
      this.handleOpenSeedPack();
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

    // 管理者ダッシュボード機能
    document.getElementById('run-health-check')?.addEventListener('click', () => {
      this.handleHealthCheck();
    });

    document.getElementById('auto-health-toggle')?.addEventListener('click', () => {
      this.handleAutoHealthToggle();
    });

    document.getElementById('refresh-global-stats')?.addEventListener('click', () => {
      this.handleRefreshGlobalStats();
    });

    document.getElementById('check-init-flow')?.addEventListener('click', () => {
      this.handleCheckInitFlow();
    });
  }

  private async handleWalletConnect() {
    try {
      const walletState = await this.solanaService.connectWallet();
      this.currentWallet = walletState;
      this.updateWalletDisplay(walletState);

      // SOLエアドロップボタンを最初に有効化
      this.enableAirdropButton();

      // 残高をチェックしてから他のボタンを有効化
      if (walletState.balance > 0.01) {
        this.enableGameButtons();
      } else {
        // 残高が少ない場合は警告を表示
        this.showError(
          'SOL残高が不足しています。まず「💰 2 SOL を取得する」ボタンでSOLを取得してください。'
        );
      }

      // 設定情報を取得
      await this.updateConfigDisplay();

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
        hasFarmSpace: detailedState.hasFarmSpace,
        growPower: detailedState.growPower,
        tokenBalance: detailedState.tokenBalance,
        lastHarvestTime: detailedState.userState?.lastHarvestTime.toNumber() || 0,
        pendingReferralRewards: detailedState.pendingReferralRewards,
        farmSpace: detailedState.farmSpace
          ? {
              level: detailedState.farmSpace.level,
              capacity: detailedState.farmSpace.capacity,
              seedCount: detailedState.farmSpace.seedCount,
              totalGrowPower: detailedState.farmSpace.totalGrowPower.toNumber(),
            }
          : undefined,
      };
      this.updateGameDisplay(gameState);

      // 設定情報も更新
      await this.updateConfigDisplay();
    } catch (error) {
      logger.error(`データ更新エラー: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  private async updateConfigDisplay() {
    try {
      const config = await this.solanaService.getAnchorClient()?.fetchConfig();
      const configStatusEl = document.getElementById('config-status');
      const adminAddressEl = document.getElementById('admin-address');

      if (config) {
        if (configStatusEl) {
          configStatusEl.textContent = '初期化済み';
          configStatusEl.style.color = '#155724';
        }
        if (adminAddressEl) {
          adminAddressEl.innerHTML = `<strong>管理者:</strong> ${config.admin.toString()}`;
        }
      } else {
        if (configStatusEl) {
          configStatusEl.textContent = '未初期化';
          configStatusEl.style.color = '#721c24';
        }
        if (adminAddressEl) {
          adminAddressEl.textContent = '';
        }
      }
    } catch {
      logger.warn('設定情報の取得に失敗しました');
    }
  }

  private async handleInitConfig() {
    logger.info('⚙️ 設定初期化機能');
    await this.gameService.initializeConfig(this.uiCallbacks);
  }

  private async handleCreateRewardMint() {
    logger.info('🪙 報酬ミント作成機能');
    await this.gameService.createRewardMint(this.uiCallbacks);
  }

  private async handleInitUser() {
    logger.info('👤 ユーザー初期化機能');
    await this.gameService.initializeUser(this.uiCallbacks);
  }

  private async handleBuyFarmSpace() {
    logger.info('🌱 ファームスペース購入機能');
    await this.gameService.purchaseFarmSpace(this.uiCallbacks);
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

        // 十分な残高があれば他のボタンも有効化
        if (walletState.balance > 0.01) {
          this.enableGameButtons();
          this.showSuccess(
            'SOL取得完了！これでゲーム機能を使用できます。「ユーザー初期化」から始めましょう！'
          );
        }
      },
    });
  }

  private async handleClaimReferralRewards() {
    logger.info('💰 紹介報酬請求機能');
    await this.gameService.claimReferralRewards(this.uiCallbacks);
  }

  private async handleUpgradeFarmSpace() {
    logger.info('🔧 ファームスペースアップグレード機能');
    await this.gameService.upgradeFarmSpace(this.uiCallbacks);
  }

  private async handleCompleteUpgrade() {
    logger.info('✅ アップグレード完了機能');
    await this.gameService.completeFarmSpaceUpgrade(this.uiCallbacks);
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

  private async handlePurchaseSeedPack() {
    logger.info('📦 シードパック購入機能');
    await this.gameService.purchaseSeedPack(this.uiCallbacks);
  }

  private async handleOpenSeedPack() {
    logger.info('📦 シードパック開封機能');

    try {
      // 簡単な例として固定値を使用（実際のUIでは選択リストから取得）
      const seedPackId = prompt('開封するシードパックのIDを入力してください:');

      if (!seedPackId) {
        throw new Error('シードパックIDが入力されていません');
      }

      await this.gameService.openSeedPack(parseInt(seedPackId), this.uiCallbacks);
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
    const farmSpaceStateEl = document.getElementById('farm-space-state');
    const growPowerEl = document.getElementById('grow-power');
    const tokenBalanceEl = document.getElementById('token-balance');
    const farmSpaceDetailsEl = document.getElementById('farm-space-details');
    const referralRewardsEl = document.getElementById('referral-rewards');

    if (userStateEl) {
      userStateEl.textContent = gameState.userInitialized ? '初期化済み' : '未初期化';
    }

    if (farmSpaceStateEl) {
      farmSpaceStateEl.textContent = gameState.hasFarmSpace ? '所有済み' : '未所有';
    }

    if (growPowerEl) {
      growPowerEl.textContent = gameState.growPower.toString();
    }

    if (tokenBalanceEl) {
      // トークンの decimals を定数から取得
      const tokenBalance = gameState.tokenBalance / Math.pow(10, GAME_CONSTANTS.TOKEN_DECIMALS);
      tokenBalanceEl.textContent = `${tokenBalance.toFixed(UI_CONSTANTS.TOKEN_DECIMAL_PLACES)} ${
        GAME_CONSTANTS.TOKEN_SYMBOL
      }`;
    }

    // ファームスペース詳細情報の更新
    if (farmSpaceDetailsEl && gameState.farmSpace) {
      farmSpaceDetailsEl.innerHTML = `
        <div><strong>レベル:</strong> ${gameState.farmSpace.level}</div>
        <div><strong>最大容量:</strong> ${gameState.farmSpace.capacity}</div>
        <div><strong>シード数:</strong> ${gameState.farmSpace.seedCount}</div>
      `;
    } else if (farmSpaceDetailsEl) {
      farmSpaceDetailsEl.innerHTML = '<div>ファームスペースなし</div>';
    }

    // 紹介報酬の更新
    if (referralRewardsEl && gameState.pendingReferralRewards !== undefined) {
      const referralRewards =
        gameState.pendingReferralRewards / Math.pow(10, GAME_CONSTANTS.TOKEN_DECIMALS);
      referralRewardsEl.textContent = `${referralRewards.toFixed(
        UI_CONSTANTS.TOKEN_DECIMAL_PLACES
      )} ${GAME_CONSTANTS.TOKEN_SYMBOL}`;
    }
  }

  private enableGameButtons() {
    const buttons = [
      'init-config',
      'create-reward-mint',
      'init-user',
      'buy-farm-space',
      'claim-rewards',
      'airdrop-sol',
      'claim-referral-rewards',
      'upgrade-farm-space',
      'complete-upgrade',
      'transfer-tokens',
      'purchase-seed-pack',
      'open-seed-pack',
    ];
    buttons.forEach((id) => {
      const button = document.getElementById(id) as HTMLButtonElement;
      if (button) button.disabled = false;
    });
  }

  private enableAirdropButton() {
    const airdropButton = document.getElementById('airdrop-sol') as HTMLButtonElement;
    if (airdropButton) {
      airdropButton.disabled = false;
      airdropButton.style.background = '#28a745';
    }
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
        message.includes('insufficient funds') ||
        message.includes('Transaction simulation failed') ||
        message.includes('残高不足')
      ) {
        displayMessage = `
          <div style="margin-bottom: 10px;">
            <strong>⚠️ SOL残高不足です</strong><br>
            devnetでテストするには少なくとも0.01 SOLが必要です。
          </div>
          <div style="margin-bottom: 10px;">
            <button onclick="document.getElementById('airdrop-sol').click()" 
                    class="button" 
                    style="margin: 5px 0; background: #28a745;">
              💰 今すぐSOLを取得する
            </button>
          </div>
          <div style="font-size: 12px; color: #666;">
            または手動で「SOLエアドロップ (開発用)」ボタンをクリックしてください。
          </div>
        `;
      }

      statusEl.innerHTML = `<div class="error">${displayMessage}</div>`;
      setTimeout(() => this.hideLoading(), UI_CONSTANTS.ERROR_MESSAGE_DURATION);
    }
  }

  // 管理者ダッシュボード機能
  private autoHealthCheckInterval: number | null = null;

  private async handleHealthCheck() {
    try {
      this.showLoading('🏥 システムヘルスチェック実行中...');
      
      const results = {
        config: await this.checkConfigStatus(),
        mint: await this.checkMintStatus(),
        globalStats: await this.checkGlobalStatsStatus(),
        feePool: await this.checkFeePoolStatus(),
        programConnection: await this.checkProgramConnection(),
      };

      this.updateHealthDisplay(results);
      this.updateLastCheckTime();
      
      // アラート表示
      this.showHealthAlert(results);
      
      this.hideLoading();
      logger.success('✅ ヘルスチェック完了');
    } catch (error) {
      this.hideLoading();
      this.showError(`ヘルスチェックエラー: ${error}`);
      logger.error('ヘルスチェックエラー:', error);
    }
  }

  private async checkConfigStatus(): Promise<{ status: string; details: string; isHealthy: boolean }> {
    try {
      const config = await this.gameService.fetchConfig();
      if (config) {
        return {
          status: '✅ 初期化済み',
          details: `管理者: ${config.admin.toBase58().slice(0, 8)}...`,
          isHealthy: true
        };
      } else {
        return {
          status: '❌ 未初期化',
          details: 'Config account not found',
          isHealthy: false
        };
      }
    } catch (error) {
      return {
        status: '⚠️ エラー',
        details: `Error: ${error}`,
        isHealthy: false
      };
    }
  }

  private async checkMintStatus(): Promise<{ status: string; details: string; isHealthy: boolean }> {
    try {
      const mintInfo = await this.gameService.fetchRewardMintInfo();
      if (mintInfo) {
        return {
          status: '✅ 作成済み',
          details: `供給量: ${mintInfo.supply.toString()}`,
          isHealthy: true
        };
      } else {
        return {
          status: '❌ 未作成',
          details: 'Reward mint not found',
          isHealthy: false
        };
      }
    } catch (error) {
      return {
        status: '⚠️ エラー',
        details: `Error: ${error}`,
        isHealthy: false
      };
    }
  }

  private async checkGlobalStatsStatus(): Promise<{ status: string; details: string; isHealthy: boolean }> {
    try {
      const stats = await this.gameService.fetchGlobalStats();
      if (stats) {
        return {
          status: '✅ 動作中',
          details: `総Grow Power: ${stats.totalGrowPower}`,
          isHealthy: true
        };
      } else {
        return {
          status: '❌ 未初期化',
          details: 'Global stats not found',
          isHealthy: false
        };
      }
    } catch (error) {
      return {
        status: '⚠️ エラー',
        details: `Error: ${error}`,
        isHealthy: false
      };
    }
  }

  private async checkFeePoolStatus(): Promise<{ status: string; details: string; isHealthy: boolean }> {
    try {
      const feePool = await this.gameService.fetchFeePool();
      if (feePool) {
        return {
          status: '✅ 動作中',
          details: `残高: ${feePool.accumulatedFees} WEED`,
          isHealthy: true
        };
      } else {
        return {
          status: '❌ 未初期化',
          details: 'Fee pool not found',
          isHealthy: false
        };
      }
    } catch (error) {
      return {
        status: '⚠️ エラー',
        details: `Error: ${error}`,
        isHealthy: false
      };
    }
  }

  private async checkProgramConnection(): Promise<{ status: string; details: string; isHealthy: boolean }> {
    try {
      const programAccount = await this.solanaService.getConnection().getAccountInfo(
        this.gameService.getProgramId()
      );
      if (programAccount) {
        return {
          status: '✅ 接続OK',
          details: 'プログラムアカウント確認済み',
          isHealthy: true
        };
      } else {
        return {
          status: '❌ 接続エラー',
          details: 'Program account not found',
          isHealthy: false
        };
      }
    } catch (error) {
      return {
        status: '⚠️ エラー',
        details: `Error: ${error}`,
        isHealthy: false
      };
    }
  }

  private updateHealthDisplay(results: Record<string, { status: string; details: string; isHealthy: boolean }>) {
    document.getElementById('health-config-status')!.textContent = results.config.status;
    document.getElementById('health-mint-status')!.textContent = results.mint.status;
    document.getElementById('health-global-stats')!.textContent = results.globalStats.status;
    document.getElementById('health-fee-pool')!.textContent = results.feePool.status;
    document.getElementById('health-program-connection')!.textContent = results.programConnection.status;
  }

  private updateLastCheckTime() {
    const now = new Date().toLocaleTimeString('ja-JP');
    document.getElementById('health-last-check')!.textContent = now;
  }

  private showHealthAlert(results: Record<string, { status: string; details: string; isHealthy: boolean }>) {
    const alertEl = document.getElementById('health-alert')!;
    const healthyCount = Object.values(results).filter(r => r.isHealthy).length;
    const totalCount = Object.values(results).length;
    
    if (healthyCount === totalCount) {
      alertEl.className = 'health-alert success';
      alertEl.innerHTML = '✅ <strong>システムは正常に動作しています</strong><br>すべての機能が利用可能です。';
    } else if (healthyCount > totalCount / 2) {
      alertEl.className = 'health-alert warning';
      alertEl.innerHTML = `⚠️ <strong>一部の機能に問題があります</strong><br>${healthyCount}/${totalCount} の機能が正常です。`;
    } else {
      alertEl.className = 'health-alert error';
      alertEl.innerHTML = `❌ <strong>システムに重大な問題があります</strong><br>${healthyCount}/${totalCount} の機能のみ正常です。`;
    }
    
    alertEl.style.display = 'block';
  }

  private handleAutoHealthToggle() {
    const button = document.getElementById('auto-health-toggle') as HTMLButtonElement;
    
    if (this.autoHealthCheckInterval) {
      // 自動チェックを停止
      clearInterval(this.autoHealthCheckInterval);
      this.autoHealthCheckInterval = null;
      button.textContent = '🔄 自動チェック: OFF';
      button.classList.remove('auto-refresh-active');
    } else {
      // 自動チェックを開始（30秒間隔）
      this.autoHealthCheckInterval = window.setInterval(() => {
        this.handleHealthCheck();
      }, 30000);
      button.textContent = '🔄 自動チェック: ON (30s)';
      button.classList.add('auto-refresh-active');
      
      // 即座に最初のチェックを実行
      this.handleHealthCheck();
    }
  }

  private async handleRefreshGlobalStats() {
    try {
      this.showLoading('📊 グローバル統計取得中...');
      
      const stats = await this.gameService.fetchGlobalStats();
      const config = await this.gameService.fetchConfig();
      const feePool = await this.gameService.fetchFeePool();
      
      if (stats) {
        document.getElementById('global-total-grow-power')!.textContent = stats.totalGrowPower.toString();
        document.getElementById('global-total-farms')!.textContent = stats.totalFarmSpaces.toString();
        document.getElementById('global-total-supply')!.textContent = `${(stats.totalSupply / 1_000_000).toFixed(6)} WEED`;
        document.getElementById('global-reward-rate')!.textContent = `${stats.currentRewardsPerSecond} WEED/秒`;
        
        const lastUpdate = new Date(stats.lastUpdateTime * 1000).toLocaleString('ja-JP');
        document.getElementById('global-last-update')!.textContent = lastUpdate;
      }
      
      if (feePool) {
        document.getElementById('global-fee-pool-balance')!.textContent = `${(feePool.accumulatedFees / 1_000_000).toFixed(6)} WEED`;
      }
      
      this.hideLoading();
      this.showSuccess('📊 グローバル統計を更新しました');
    } catch (error) {
      this.hideLoading();
      this.showError(`統計取得エラー: ${error}`);
      logger.error('グローバル統計取得エラー:', error);
    }
  }

  private async handleCheckInitFlow() {
    try {
      this.showLoading('🔄 初期化フロー確認中...');
      
      // 各初期化ステップをチェック
      const steps = [
        { id: 'config', check: () => this.gameService.fetchConfig() },
        { id: 'mint', check: () => this.gameService.fetchRewardMintInfo() },
        { id: 'global-stats', check: () => this.gameService.fetchGlobalStats() },
        { id: 'fee-pool', check: () => this.gameService.fetchFeePool() },
      ];
      
      for (const step of steps) {
        try {
          const result = await step.check();
          this.updateInitStepStatus(step.id, result ? 'completed' : 'error', 
            result ? '✅ 完了' : '❌ 未完了');
        } catch (error) {
          this.updateInitStepStatus(step.id, 'error', `❌ エラー: ${error}`);
        }
      }
      
      this.hideLoading();
      this.showSuccess('🔄 初期化フロー確認完了');
    } catch (error) {
      this.hideLoading();
      this.showError(`初期化フロー確認エラー: ${error}`);
      logger.error('初期化フロー確認エラー:', error);
    }
  }

  private updateInitStepStatus(stepId: string, status: 'completed' | 'error' | 'pending', details: string) {
    const stepEl = document.getElementById(`step-${stepId}`)!;
    const indicatorEl = document.getElementById(`indicator-${stepId}`)!;
    const detailsEl = document.getElementById(`details-${stepId}`)!;
    
    // クラスをリセット
    stepEl.classList.remove('completed', 'error');
    
    switch (status) {
      case 'completed':
        stepEl.classList.add('completed');
        indicatorEl.textContent = '✅';
        break;
      case 'error':
        stepEl.classList.add('error');
        indicatorEl.textContent = '❌';
        break;
      case 'pending':
        indicatorEl.textContent = '⏳';
        break;
    }
    
    detailsEl.textContent = details;
  }
}

// DOM読み込み完了後にアプリを初期化 (テスト環境では実行しない)
if (typeof document !== 'undefined') {
  document.addEventListener('DOMContentLoaded', () => {
    new FarmGameApp();

    // デバッグ用グローバル関数
    (window as unknown as { testHeliusConnection: () => Promise<unknown> }).testHeliusConnection =
      async () => {
        const service = new SolanaService();
        return await service.testConnection();
      };

    // Manual test function for browser console
    (window as unknown as { runManualTest: () => Promise<unknown> }).runManualTest = async () => {
      const { runManualFrontendTest } = await import('./test/manual-frontend-test');
      return await runManualFrontendTest();
    };

    // Debug information for development
    logger.info('🎮 Facility Game Frontend loaded!');
    logger.info('💡 Available console commands:');
    logger.info('  - testHeliusConnection(): Test RPC connection');
    logger.info('  - runManualTest(): Run comprehensive frontend test');
  });
}

// TypeScriptのエクスポート（必要に応じて）
export { FarmGameApp };
