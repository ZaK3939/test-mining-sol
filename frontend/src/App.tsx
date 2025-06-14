import React, { useState, useEffect } from 'react';
import { SolanaService } from './solana';
import { SimpleClient } from './simple-client';
import { logger } from './logger';
import type { WalletState, GameState } from './types';

// コンポーネントの分割
import ConnectionStatus from './components/ConnectionStatus';
import WalletSection from './components/WalletSection';
import InstructionSet from './components/InstructionSet';
import EconomicsFormulas from './components/EconomicsFormulas';
import InviteCodeSection from './components/InviteCodeSection';
import GameControls from './components/GameControls';
import SystemLogs from './components/SystemLogs';

const App: React.FC = () => {
  const [solanaService] = useState(() => new SolanaService());
  const [gameClient, setGameClient] = useState<SimpleClient | null>(null);
  const [walletState, setWalletState] = useState<WalletState | null>(null);
  const [gameState, setGameState] = useState<GameState | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<{ connected: boolean; network: string; rpcUrl: string; programId: string }>({
    connected: false,
    network: 'devnet',
    rpcUrl: '',
    programId: ''
  });

  // メッセージ表示のヘルパー
  const showMessage = (type: 'success' | 'error' | 'info', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), type === 'error' ? 8000 : 4000);
  };

  // UI コールバック
  const uiCallbacks = {
    showLoading: (_message: string) => setLoading(true),
    hideLoading: () => setLoading(false),
    showSuccess: (message: string) => showMessage('success', message),
    showError: (message: string) => showMessage('error', message),
    updateGameState: refreshGameData,
  };

  // ゲームデータの更新
  async function refreshGameData() {
    if (!walletState?.connected || !gameClient || !walletState.publicKey) return;

    try {
      const userState = await gameClient.getUserState(walletState.publicKey);
      const farmSpace = await gameClient.getFarmSpace(walletState.publicKey);
      const tokenBalance = await gameClient.getTokenBalance(walletState.publicKey);
      
      const gameState: GameState = {
        userInitialized: !!userState,
        hasFarmSpace: userState?.hasFarmSpace || false,
        growPower: farmSpace?.totalGrowPower?.toNumber() || 0,
        tokenBalance: tokenBalance * 1_000_000, // Convert to microlamports for display
        lastHarvestTime: userState?.lastHarvestTime?.toNumber() || 0,
        pendingReferralRewards: userState?.pendingReferralRewards?.toNumber() || 0,
        farmSpace: farmSpace
          ? {
              level: farmSpace.level,
              capacity: farmSpace.capacity,
              seedCount: farmSpace.seedCount,
              totalGrowPower: farmSpace.totalGrowPower.toNumber(),
            }
          : undefined,
      };
      setGameState(gameState);
    } catch (error) {
      logger.error(`データ更新エラー: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  // ウォレット接続
  const handleWalletConnect = async () => {
    try {
      setLoading(true);
      const wallet = await solanaService.connectWallet();
      setWalletState(wallet);
      
      // SimpleClientを初期化
      const provider = await solanaService.getProvider();
      const client = new SimpleClient(provider);
      setGameClient(client);
      
      if (wallet.balance > 0.01) {
        showMessage('success', 'ウォレット接続完了！');
      } else {
        showMessage('error', 'SOL残高が不足しています。まず「💰 2 SOL を取得する」ボタンでSOLを取得してください。');
      }
      
      await refreshGameData();
    } catch (error) {
      showMessage('error', `ウォレット接続失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setLoading(false);
    }
  };

  // SOLエアドロップ
  const handleAirdrop = async () => {
    if (!walletState?.connected) return;
    
    try {
      setLoading(true);
      await solanaService.airdropSol(2);
      const updatedWallet = solanaService.getWalletState();
      setWalletState(updatedWallet);
      if (updatedWallet.balance > 0.01) {
        showMessage('success', 'SOL取得完了！これでゲーム機能を使用できます。');
      }
    } catch (error) {
      showMessage('error', `エアドロップ失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setLoading(false);
    }
  };

  // 初期化
  useEffect(() => {
    const init = async () => {
      logger.info('🚀 React フロントエンド初期化開始');
      
      try {
        const networkInfo = solanaService.getNetworkInfo();
        setConnectionStatus({
          connected: false,
          network: networkInfo.network,
          rpcUrl: networkInfo.rpcUrl,
          programId: networkInfo.programId
        });
        
        const isConnected = await solanaService.testConnection();
        setConnectionStatus(prev => ({ ...prev, connected: isConnected }));
        
        if (isConnected) {
          logger.success('✅ 初期化完了 - React フロントエンド準備OK');
        } else {
          logger.error('❌ RPC接続失敗');
        }
      } catch (error) {
        logger.error(`初期化エラー: ${error instanceof Error ? error.message : String(error)}`);
      }
    };

    init();
  }, []);

  return (
    <div style={{
      fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
      margin: 0,
      padding: '20px',
      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
      color: '#333',
      minHeight: '100vh'
    }}>
      <div style={{
        maxWidth: '1200px',
        margin: '0 auto',
        background: 'white',
        borderRadius: '15px',
        padding: '30px',
        boxShadow: '0 10px 30px rgba(0,0,0,0.1)'
      }}>
        {/* ヘッダー */}
        <div style={{ textAlign: 'center', marginBottom: '40px' }}>
          <h1 style={{ color: '#2c3e50', marginBottom: '10px' }}>
            🏭 Solana Facility Game - React版
          </h1>
          <p>シンプルなReact UIでRust命令セットや計算式、招待コード処理を確認</p>
        </div>

        {/* メッセージ表示 */}
        {message && (
          <div style={{
            padding: '10px',
            marginBottom: '20px',
            borderRadius: '5px',
            backgroundColor: message.type === 'success' ? '#d4edda' : message.type === 'error' ? '#f8d7da' : '#d1ecf1',
            color: message.type === 'success' ? '#155724' : message.type === 'error' ? '#721c24' : '#0c5460',
            border: `1px solid ${message.type === 'success' ? '#c3e6cb' : message.type === 'error' ? '#f5c6cb' : '#bee5eb'}`
          }}>
            {message.text}
          </div>
        )}

        {/* 接続状況 */}
        <ConnectionStatus 
          connectionStatus={connectionStatus}
        />

        {/* ウォレットセクション */}
        <WalletSection
          walletState={walletState}
          onConnect={handleWalletConnect}
          onAirdrop={handleAirdrop}
          loading={loading}
        />

        {/* メインコンテンツ */}
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))', gap: '20px', margin: '30px 0' }}>
          {/* Rust命令セット */}
          <InstructionSet />
          
          {/* 計算式 */}
          <EconomicsFormulas />
          
          {/* 招待コード処理 */}
          <InviteCodeSection />
        </div>

        {/* ゲームコントロール */}
        {walletState?.connected && gameClient && (
          <GameControls
            gameClient={gameClient}
            gameState={gameState}
            uiCallbacks={uiCallbacks}
            onRefresh={refreshGameData}
          />
        )}

        {/* システムログ */}
        <SystemLogs />
      </div>
    </div>
  );
};

export default App;