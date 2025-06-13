// 設定ファイル - 環境変数から安全に値を取得

export interface AppConfig {
  programId: string;
  network: 'devnet' | 'testnet' | 'mainnet-beta';
  rpcUrl: string;
}

// 環境変数から設定を取得
function getConfig(): AppConfig {
  // Viteの環境変数は import.meta.env から取得
  const heliusApiKey = import.meta.env.VITE_HELIUS_API_KEY_DEVNET;

  console.log('🔧 環境変数デバッグ:', {
    heliusApiKey: heliusApiKey ? `${heliusApiKey.slice(0, 8)}...` : 'undefined',
    mode: import.meta.env.MODE,
    dev: import.meta.env.DEV,
  });

  if (!heliusApiKey) {
    console.warn(
      '⚠️ VITE_HELIUS_API_KEY_DEVNET が設定されていません。デフォルトのpublic RPCを使用します。'
    );
  }

  const config: AppConfig = {
    programId: '7r3R1S43BS9fQbh1eBhM63u8XZJd7bYRtgMrAQRNrfcB',
    network: 'devnet',
    rpcUrl: 'https://api.devnet.solana.com', // Always use public devnet for stability
  };

  console.log('📋 設定読み込み完了:');
  console.log(`  - Program ID: ${config.programId}`);
  console.log(`  - Network: ${config.network}`);
  console.log(`  - RPC URL: ${config.rpcUrl.split('?')[0]}${heliusApiKey ? '?api-key=***' : ''}`);

  return config;
}

export const config = getConfig();
