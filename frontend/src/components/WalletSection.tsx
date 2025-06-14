import React from 'react';
import type { WalletState } from '../types';

interface WalletSectionProps {
  walletState: WalletState | null;
  onConnect: () => void;
  onAirdrop: () => void;
  loading: boolean;
}

const WalletSection: React.FC<WalletSectionProps> = ({
  walletState,
  onConnect,
  onAirdrop,
  loading
}) => {
  const buttonStyle = {
    background: 'linear-gradient(45deg, #667eea, #764ba2)',
    color: 'white',
    border: 'none',
    padding: '12px 24px',
    borderRadius: '8px',
    cursor: 'pointer',
    fontSize: '16px',
    margin: '10px 5px',
    transition: 'all 0.3s ease',
    opacity: loading ? 0.7 : 1
  };

  const successButtonStyle = {
    ...buttonStyle,
    background: '#28a745',
    fontSize: '18px',
    padding: '15px 30px'
  };

  return (
    <div style={{
      background: '#f8f9fa',
      border: '1px solid #dee2e6',
      borderRadius: '10px',
      padding: '20px',
      margin: '20px 0'
    }}>
      <h3 style={{ marginTop: 0, color: '#495057' }}>👛 ウォレット</h3>
      
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: '15px' }}>
        <span style={{
          display: 'inline-block',
          width: '12px',
          height: '12px',
          borderRadius: '50%',
          marginRight: '8px',
          backgroundColor: walletState?.connected ? '#28a745' : '#dc3545'
        }}></span>
        {walletState?.connected ? '接続済み' : '未接続'}
      </div>

      {!walletState?.connected ? (
        <button
          onClick={onConnect}
          style={buttonStyle}
          disabled={loading}
        >
          ウォレット接続
        </button>
      ) : (
        <>
          <div style={{ marginBottom: '15px' }}>
            <div><strong>アドレス:</strong> {walletState.publicKey?.toString()}</div>
            <div><strong>残高:</strong> {walletState.balance.toFixed(4)} SOL</div>
          </div>
          
          {/* SOLエアドロップ */}
          <div style={{ 
            border: '2px solid #28a745', 
            background: '#f8fff9', 
            borderRadius: '10px', 
            padding: '15px', 
            margin: '15px 0' 
          }}>
            <h4 style={{ marginTop: 0, color: '#155724' }}>💰 devnet SOL取得 (テスト用)</h4>
            <div style={{ marginBottom: '15px', color: '#155724' }}>
              <strong>最初にSOLを取得してください！</strong><br />
              すべての操作にはSOL（手数料）が必要です。
            </div>
            <button
              onClick={onAirdrop}
              style={successButtonStyle}
              disabled={loading}
            >
              💰 2 SOL を取得する
            </button>
            <div style={{ marginTop: '10px', fontSize: '12px', color: '#666' }}>
              ※ devnetのテスト用SOLです。実際の価値はありません。
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default WalletSection;