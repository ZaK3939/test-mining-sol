import React, { useState, useEffect } from 'react';
import { logger } from '../logger';

const SystemLogs: React.FC = () => {
  const [logs, setLogs] = useState<string[]>([]);

  useEffect(() => {
    // ログの更新を監視（簡単な実装）
    const interval = setInterval(() => {
      // 実際の実装では、loggerからログを取得
      // ここでは簡単な例として現在のログを表示
      const currentLogs = [
        `[${new Date().toLocaleTimeString()}] React フロントエンド準備完了`,
        `[${new Date().toLocaleTimeString()}] Rust命令セット表示中`,
        `[${new Date().toLocaleTimeString()}] 計算式確認機能アクティブ`,
        `[${new Date().toLocaleTimeString()}] 招待コード処理UI表示中`
      ];
      setLogs(currentLogs);
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const handleClearLogs = () => {
    setLogs([]);
    logger.clear();
  };

  return (
    <div style={{
      background: '#f8f9fa',
      border: '1px solid #dee2e6',
      borderRadius: '10px',
      padding: '20px',
      margin: '20px 0'
    }}>
      <h3 style={{ marginTop: 0, color: '#495057' }}>📋 システムログ</h3>
      
      <div style={{
        background: '#1e1e1e',
        color: '#d4d4d4',
        borderRadius: '8px',
        padding: '15px',
        fontFamily: 'Courier New, monospace',
        fontSize: '12px',
        maxHeight: '200px',
        overflowY: 'auto',
        margin: '15px 0',
        minHeight: '100px'
      }}>
        {logs.length > 0 ? (
          logs.map((log, index) => (
            <div key={index} style={{ marginBottom: '5px' }}>
              {log}
            </div>
          ))
        ) : (
          <div style={{ color: '#888', fontStyle: 'italic' }}>
            React フロントエンド初期化中...
          </div>
        )}
      </div>
      
      <button
        onClick={handleClearLogs}
        style={{
          background: '#6c757d',
          color: 'white',
          border: 'none',
          padding: '8px 16px',
          borderRadius: '6px',
          cursor: 'pointer',
          fontSize: '14px'
        }}
      >
        ログクリア
      </button>
    </div>
  );
};

export default SystemLogs;