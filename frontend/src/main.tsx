// React アプリケーションのエントリーポイント

import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { logger } from './logger';

// Buffer polyfill for Solana
if (typeof global === 'undefined') {
  (globalThis as any).global = globalThis;
}

// React アプリケーションの初期化
const container = document.getElementById('root');
if (!container) {
  throw new Error('Root element not found');
}

const root = createRoot(container);

// React アプリをレンダリング
root.render(React.createElement(App));

// テスト機能はコメントアウト（削除済みファイル）
// import './test-journey';

// デバッグ用グローバル関数
if (typeof window !== 'undefined') {
  // デバッグ情報
  logger.info('🎮 React Facility Game Frontend loaded!');
  logger.info('💡 Rust命令セット、計算式、招待コード処理を確認できます');
  logger.info('🧪 ユーザージャーニーテスト: testUserJourney() をコンソールで実行');
}
