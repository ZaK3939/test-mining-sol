import React from 'react';

interface ConnectionStatusProps {
  connectionStatus: {
    connected: boolean;
    network: string;
    rpcUrl: string;
    programId: string;
  };
}

const ConnectionStatus: React.FC<ConnectionStatusProps> = ({ connectionStatus }) => {
  return (
    <div style={{
      background: '#f8f9fa',
      border: '1px solid #dee2e6',
      borderRadius: '10px',
      padding: '20px',
      margin: '20px 0'
    }}>
      <h3 style={{ marginTop: 0, color: '#495057' }}>📡 接続状況</h3>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: '10px' }}>
        <span style={{
          display: 'inline-block',
          width: '12px',
          height: '12px',
          borderRadius: '50%',
          marginRight: '8px',
          backgroundColor: connectionStatus.connected ? '#28a745' : '#dc3545'
        }}></span>
        {connectionStatus.connected ? 'Helius RPC 接続済み' : '接続失敗'}
      </div>
      <div style={{ marginTop: '10px' }}>
        <div><strong>RPC Endpoint:</strong> {connectionStatus.rpcUrl || '-'}</div>
        <div><strong>Network:</strong> {connectionStatus.network.toUpperCase()}</div>
        <div><strong>Program ID:</strong> {connectionStatus.programId || 'FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B'}</div>
      </div>
    </div>
  );
};

export default ConnectionStatus;