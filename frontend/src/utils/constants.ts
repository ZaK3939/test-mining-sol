// 定数定義ファイル
// アプリケーション全体で使用される定数を統一管理

/**
 * ゲーム関連の定数
 */
export const GAME_CONSTANTS = {
  // 施設・マシン関連
  INITIAL_GROW_POWER: 100,
  MACHINE_COUNT_INITIAL: 1,

  // トークン関連
  TOKEN_DECIMALS: 6,
  TOKEN_SYMBOL: 'WEED',

  // 時間関連
  DEFAULT_HALVING_INTERVAL: 200, // 200秒

  // 報酬計算関連
  BASE_RATE_INITIAL: 10,
  REWARD_CALCULATION_DIVISOR: 1000,
} as const;

/**
 * ネットワーク関連の定数
 */
export const NETWORK_CONSTANTS = {
  // エアドロップ関連
  DEFAULT_AIRDROP_AMOUNT: 2, // SOL

  // RPC関連
  RPC_TIMEOUT_MS: 30000,
  COMMITMENT: 'confirmed' as const,

  // ローカル開発
  LOCAL_VALIDATOR_URL: 'http://localhost:8899',
  LOCAL_VALIDATOR_WS: 'ws://localhost:8900',
} as const;

/**
 * UI関連の定数
 */
export const UI_CONSTANTS = {
  // メッセージ表示時間
  SUCCESS_MESSAGE_DURATION: 5000, // ms
  ERROR_MESSAGE_DURATION: 8000, // ms

  // 数値フォーマット
  SOL_DECIMAL_PLACES: 4,
  TOKEN_DECIMAL_PLACES: 6,

  // DOM要素ID
  DOM_IDS: {
    CONNECTION_STATUS: 'connection-status',
    WALLET_STATUS: 'wallet-status',
    WALLET_ADDRESS: 'wallet-address',
    WALLET_BALANCE: 'wallet-balance',
    WALLET_INFO: 'wallet-info',
    USER_STATE: 'user-state',
    FACILITY_STATE: 'facility-state',
    GROW_POWER: 'grow-power',
    TOKEN_BALANCE: 'token-balance',
    STATUS_MESSAGE: 'status-message',
    LOGS: 'logs',
    RPC_URL: 'rpc-url',
    NETWORK: 'network',
    // ボタン
    CONNECT_WALLET: 'connect-wallet',
    INIT_USER: 'init-user',
    BUY_FACILITY: 'buy-facility',
    CLAIM_REWARDS: 'claim-rewards',
    AIRDROP_SOL: 'airdrop-sol',
    REFRESH_DATA: 'refresh-data',
    CLEAR_LOGS: 'clear-logs',
  },
} as const;

/**
 * エラーメッセージの定数
 */
export const ERROR_MESSAGES = {
  WALLET_NOT_CONNECTED: 'ウォレットが接続されていません',
  USER_NOT_INITIALIZED: '先にユーザーを初期化してください',
  USER_ALREADY_INITIALIZED: 'ユーザーアカウントは既に初期化されています',
  FACILITY_ALREADY_OWNED: '既に施設を所有しています',
  FACILITY_NOT_OWNED: '先に施設を購入してください',
  INSUFFICIENT_FUNDS:
    '残高不足です。「SOLエアドロップ (開発用)」ボタンでSOLを取得してから再試行してください。',
  CONNECTION_FAILED: 'RPC接続に失敗しました',
  TRANSACTION_FAILED: 'トランザクションが失敗しました',
} as const;

/**
 * 成功メッセージの定数
 */
export const SUCCESS_MESSAGES = {
  USER_INITIALIZED: 'ユーザーアカウントの初期化が完了しました！',
  USER_ALREADY_INITIALIZED: 'ユーザーアカウントは既に初期化済みです！',
  FACILITY_PURCHASED: '施設の購入が完了しました！Grow Power: 100',
  FACILITY_ALREADY_OWNED: '施設は既に所有済みです！',
  REWARDS_CLAIMED: '報酬の請求が完了しました！',
  AIRDROP_COMPLETED: (amount: number) => `${amount} SOL のエアドロップが完了しました！`,
  CONNECTION_ESTABLISHED: 'RPC接続が確立されました',
} as const;

/**
 * プログラム関連の定数
 */
export const PROGRAM_CONSTANTS = {
  // プログラムID（環境別で管理される場合は config.ts で上書き）
  PROGRAM_ID: 'EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89',

  // PDAシード
  SEEDS: {
    CONFIG: 'config',
    USER: 'user',
    FACILITY: 'facility',
    REWARD_MINT: 'reward_mint',
    MINT_AUTHORITY: 'mint_authority',
  },
} as const;

/**
 * 型安全な定数アクセスのためのヘルパー
 */
export type GameConstants = typeof GAME_CONSTANTS;
export type NetworkConstants = typeof NETWORK_CONSTANTS;
export type UIConstants = typeof UI_CONSTANTS;
export type ErrorMessages = typeof ERROR_MESSAGES;
export type SuccessMessages = typeof SUCCESS_MESSAGES;
export type ProgramConstants = typeof PROGRAM_CONSTANTS;
