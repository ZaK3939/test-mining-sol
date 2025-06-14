import React from 'react';
import { SimpleClient } from '../simple-client';
import type { GameState } from '../types';

interface UICallbacks {
  showLoading: (message: string) => void;
  hideLoading: () => void;
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
  updateGameState: () => Promise<void>;
}

interface GameControlsProps {
  gameClient: SimpleClient;
  gameState: GameState | null;
  uiCallbacks: UICallbacks;
  onRefresh: () => Promise<void>;
}

const GameControls: React.FC<GameControlsProps> = ({
  gameClient,
  gameState,
  uiCallbacks,
  onRefresh
}) => {
  const buttonStyle = {
    background: 'linear-gradient(45deg, #667eea, #764ba2)',
    color: 'white',
    border: 'none',
    padding: '10px 20px',
    borderRadius: '6px',
    cursor: 'pointer',
    fontSize: '14px',
    margin: '5px',
    transition: 'all 0.3s ease'
  };

  const adminButtonStyle = {
    ...buttonStyle,
    background: '#dc3545'
  };

  const successButtonStyle = {
    ...buttonStyle,
    background: '#28a745'
  };

  const handleInitConfig = async () => {
    try {
      uiCallbacks.showLoading('設定を初期化中...');
      await gameClient.initializeConfig();
      uiCallbacks.showSuccess('設定初期化完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`設定初期化失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handleCreateRewardMint = async () => {
    try {
      uiCallbacks.showLoading('報酬ミントを作成中...');
      await gameClient.createRewardMint();
      uiCallbacks.showSuccess('報酬ミント作成完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`報酬ミント作成失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handleInitUser = async () => {
    try {
      uiCallbacks.showLoading('ユーザーを初期化中...');
      await gameClient.initUser();
      uiCallbacks.showSuccess('ユーザー初期化完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`ユーザー初期化失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handleBuyFarmSpace = async () => {
    try {
      uiCallbacks.showLoading('ファームスペースを購入中...');
      await gameClient.buyFarmSpace();
      uiCallbacks.showSuccess('ファームスペース購入完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`ファームスペース購入失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handleClaimRewards = async () => {
    try {
      uiCallbacks.showLoading('報酬を請求中...');
      await gameClient.claimRewards();
      uiCallbacks.showSuccess('報酬請求完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`報酬請求失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handleClaimReferralRewards = async () => {
    try {
      uiCallbacks.showLoading('紹介報酬を請求中...');
      await gameClient.viewPendingReferralRewards();
      uiCallbacks.showSuccess('紹介報酬請求完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`紹介報酬請求失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handleUpgradeFarmSpace = async () => {
    try {
      uiCallbacks.showLoading('ファームスペースをアップグレード中...');
      // SimpleClientにはupgradeFarmSpace機能がないため、コメントアウト
      // await gameClient.upgradeFarmSpace();
      uiCallbacks.showError('アップグレード機能は未実装です');
    } catch (error) {
      uiCallbacks.showError(`アップグレード失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  const handlePurchaseSeedPack = async () => {
    try {
      uiCallbacks.showLoading('シードパックを購入中...');
      await gameClient.purchaseSeedPack(1);
      uiCallbacks.showSuccess('シードパック購入完了');
      await uiCallbacks.updateGameState();
    } catch (error) {
      uiCallbacks.showError(`シードパック購入失敗: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      uiCallbacks.hideLoading();
    }
  };

  return (
    <div>
      {/* ゲーム状況表示 */}
      <div style={{
        background: '#f8f9fa',
        border: '1px solid #dee2e6',
        borderRadius: '10px',
        padding: '20px',
        margin: '20px 0'
      }}>
        <h3 style={{ marginTop: 0, color: '#495057' }}>🎮 ゲーム状況</h3>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '15px' }}>
          <div>
            <label style={{ fontWeight: 'bold', color: '#495057' }}>ユーザー状態:</label>
            <div style={{ marginTop: '5px', fontFamily: 'Courier New, monospace', fontSize: '14px' }}>
              {gameState?.userInitialized ? '✅ 初期化済み' : '❌ 未初期化'}
            </div>
          </div>
          <div>
            <label style={{ fontWeight: 'bold', color: '#495057' }}>ファームスペース:</label>
            <div style={{ marginTop: '5px', fontFamily: 'Courier New, monospace', fontSize: '14px' }}>
              {gameState?.hasFarmSpace ? '✅ 所有済み' : '❌ 未所有'}
            </div>
          </div>
          <div>
            <label style={{ fontWeight: 'bold', color: '#495057' }}>Grow Power:</label>
            <div style={{ marginTop: '5px', fontFamily: 'Courier New, monospace', fontSize: '14px' }}>
              {gameState?.growPower || 0}
            </div>
          </div>
          <div>
            <label style={{ fontWeight: 'bold', color: '#495057' }}>$WEED 残高:</label>
            <div style={{ marginTop: '5px', fontFamily: 'Courier New, monospace', fontSize: '14px' }}>
              {gameState ? (gameState.tokenBalance / 1_000_000).toFixed(6) : '0.000000'} WEED
            </div>
          </div>
          <div>
            <label style={{ fontWeight: 'bold', color: '#495057' }}>紹介報酬:</label>
            <div style={{ marginTop: '5px', fontFamily: 'Courier New, monospace', fontSize: '14px' }}>
              {gameState ? ((gameState.pendingReferralRewards || 0) / 1_000_000).toFixed(6) : '0.000000'} WEED
            </div>
          </div>
        </div>

        {/* ファームスペース詳細 */}
        {gameState?.farmSpace && (
          <div style={{ marginTop: '15px', padding: '10px', background: '#e9ecef', borderRadius: '6px' }}>
            <h4 style={{ marginTop: 0, color: '#495057' }}>🌱 ファームスペース詳細</h4>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))', gap: '10px' }}>
              <div><strong>レベル:</strong> {gameState.farmSpace.level}</div>
              <div><strong>最大容量:</strong> {gameState.farmSpace.capacity}</div>
              <div><strong>シード数:</strong> {gameState.farmSpace.seedCount}</div>
              <div><strong>総GP:</strong> {gameState.farmSpace.totalGrowPower}</div>
            </div>
          </div>
        )}
      </div>

      {/* 管理者アクション */}
      <div style={{
        background: '#fff5f5',
        border: '2px solid #dc3545',
        borderRadius: '10px',
        padding: '20px',
        margin: '20px 0'
      }}>
        <h3 style={{ marginTop: 0, color: '#721c24' }}>⚙️ 管理者アクション</h3>
        <div style={{ marginBottom: '15px', color: '#721c24' }}>
          <strong>⚠️ 初回セットアップ必須</strong><br />
          ゲーム機能を使用する前に設定の初期化が必要です。
        </div>
        <button onClick={handleInitConfig} style={adminButtonStyle}>
          設定初期化
        </button>
        <button onClick={handleCreateRewardMint} style={adminButtonStyle}>
          報酬ミント作成
        </button>
        <button onClick={async () => {
          try {
            uiCallbacks.showLoading('グローバル統計を初期化中...');
            await gameClient.initializeGlobalStats();
            uiCallbacks.showSuccess('グローバル統計初期化完了');
            await uiCallbacks.updateGameState();
          } catch (error) {
            uiCallbacks.showError(`グローバル統計初期化失敗: ${error instanceof Error ? error.message : String(error)}`);
          } finally {
            uiCallbacks.hideLoading();
          }
        }} style={adminButtonStyle}>
          統計初期化
        </button>
      </div>

      {/* 基本アクション */}
      <div style={{
        background: '#f8f9fa',
        border: '1px solid #dee2e6',
        borderRadius: '10px',
        padding: '20px',
        margin: '20px 0'
      }}>
        <h3 style={{ marginTop: 0, color: '#495057' }}>⚡ 基本アクション</h3>
        <button onClick={handleInitUser} style={buttonStyle}>
          ユーザー初期化
        </button>
        <button onClick={handleBuyFarmSpace} style={buttonStyle}>
          ファームスペース購入
        </button>
        <button onClick={handleClaimRewards} style={successButtonStyle}>
          報酬請求
        </button>
        <button onClick={onRefresh} style={buttonStyle}>
          データ更新
        </button>
      </div>

      {/* 紹介システム */}
      <div style={{
        background: '#f8f9fa',
        border: '1px solid #dee2e6',
        borderRadius: '10px',
        padding: '20px',
        margin: '20px 0'
      }}>
        <h3 style={{ marginTop: 0, color: '#495057' }}>🤝 紹介システム</h3>
        
        {/* 招待コード作成 */}
        <div style={{ marginBottom: '15px' }}>
          <input
            type="text"
            placeholder="招待コード (8文字)"
            style={{
              padding: '8px 12px',
              borderRadius: '4px',
              border: '1px solid #ced4da',
              marginRight: '10px',
              fontFamily: 'Courier New, monospace',
              width: '120px'
            }}
            maxLength={8}
            id="invite-code-input"
          />
          <button 
            onClick={async () => {
              const input = document.getElementById('invite-code-input') as HTMLInputElement;
              const code = input.value.trim();
              if (code.length !== 8) {
                alert('招待コードは8文字で入力してください');
                return;
              }
              try {
                uiCallbacks.showLoading('招待コードを作成中...');
                await gameClient.createInviteCode(code);
                uiCallbacks.showSuccess('招待コード作成完了');
                input.value = '';
                await uiCallbacks.updateGameState();
              } catch (error) {
                uiCallbacks.showError(`招待コード作成失敗: ${error instanceof Error ? error.message : String(error)}`);
              } finally {
                uiCallbacks.hideLoading();
              }
            }}
            style={buttonStyle}
          >
            🎫 招待コード作成
          </button>
        </div>

        <button onClick={handleClaimReferralRewards} style={successButtonStyle}>
          紹介報酬請求
        </button>
        <div style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
          ※ Level 1: 10%, Level 2: 5%の自動分配<br />
          ※ 招待コード作成制限: 一般5回/運営255回
        </div>
      </div>

      {/* ファームスペース管理 */}
      <div style={{
        background: '#f8f9fa',
        border: '1px solid #dee2e6',
        borderRadius: '10px',
        padding: '20px',
        margin: '20px 0'
      }}>
        <h3 style={{ marginTop: 0, color: '#495057' }}>🔧 ファームスペース管理</h3>
        <button onClick={handleUpgradeFarmSpace} style={buttonStyle}>
          アップグレード
        </button>
        <div style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
          ※ 自動アップグレード: 30, 100, 300, 500パック購入時
        </div>
      </div>

      {/* シードパック */}
      <div style={{
        background: '#f8f9fa',
        border: '1px solid #dee2e6',
        borderRadius: '10px',
        padding: '20px',
        margin: '20px 0'
      }}>
        <h3 style={{ marginTop: 0, color: '#495057' }}>📦 シードパック</h3>
        <button onClick={handlePurchaseSeedPack} style={buttonStyle}>
          シードパック購入 (1個)
        </button>
        <div style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
          ※ コスト: 300 WEED + VRF手数料 (~0.002 SOL)<br />
          ※ Switchboard VRFによる検証可能な乱数
        </div>
      </div>
    </div>
  );
};

export default GameControls;