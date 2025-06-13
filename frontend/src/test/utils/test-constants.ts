// テスト用定数
// マジックナンバーを削減し、テスト設定を一元管理

export const TEST_CONSTANTS = {
  // タイムアウト設定（ミリ秒）
  TIMEOUTS: {
    DEFAULT: 30000, // 30秒 - 通常のテスト
    STRESS: 45000, // 45秒 - ストレステスト
    NETWORK: 15000, // 15秒 - ネットワーク関連テスト
    QUICK: 10000, // 10秒 - 高速テスト
    LONG: 60000, // 60秒 - 長時間テスト
  },

  // 期待される成功率（0.0 - 1.0）
  SUCCESS_RATES: {
    HIGH: 0.8, // 80% - 高い成功率が期待される操作
    MEDIUM: 0.5, // 50% - 中程度の成功率
    LOW: 0.3, // 30% - 低い成功率でも許容される操作
    MINIMUM: 0.1, // 10% - 最低限の成功率
  },

  // 操作回数
  OPERATION_COUNTS: {
    SMALL: 5, // 小規模テスト
    MEDIUM: 10, // 中規模テスト
    LARGE: 20, // 大規模テスト
    STRESS: 50, // ストレステスト
  },

  // 資金調達設定
  FUNDING: {
    DEFAULT_SOL: 2, // デフォルトのSOL送金量
    LARGE_SOL: 5, // 大きな操作用のSOL送金量
    STRESS_SOL: 10, // ストレステスト用のSOL送金量
  },

  // ネットワーク設定
  NETWORK: {
    RETRY_ATTEMPTS: 3, // リトライ回数
    RETRY_DELAY: 1000, // リトライ間隔（ミリ秒）
    CONNECTION_TIMEOUT: 5000, // 接続タイムアウト（ミリ秒）
  },

  // パフォーマンス期待値
  PERFORMANCE: {
    BATCH_IMPROVEMENT_MIN: 0.2, // バッチ処理の最小改善率（20%）
    CACHE_HIT_RATE_MIN: 0.7, // キャッシュヒット率の最小値（70%）
    MAX_OPERATION_TIME: 10000, // 操作の最大許容時間（10秒）
  },

  // メモリ設定
  MEMORY: {
    PRESSURE_SIZE_MB: 50, // メモリプレッシャーテストのサイズ（MB）
    CLEANUP_DELAY: 100, // メモリクリーンアップの遅延（ミリ秒）
  },

  // テストデータ
  TEST_DATA: {
    FARM_SPACE_COST_SOL: 0.5, // 農場スペースコスト（SOL）
    SEED_PACK_COST: 300, // シードパックコスト（WEED）
    HALVING_INTERVAL: 200, // 半減期間隔（秒）
    TOTAL_SUPPLY: 240_000_000, // 総供給量（WEED）
    BASE_RATE: 200, // 基本報酬レート（WEED/秒）
  },

  // エラー設定
  ERROR_HANDLING: {
    EXPECTED_ERRORS: [
      'User already initialized',
      'Facility already owned',
      'Insufficient funds',
      'Account does not exist',
    ],
    NETWORK_ERRORS: ['Connection timeout', 'Network error', 'RPC error'],
  },

  // ログ設定
  LOGGING: {
    VERBOSE: true, // 詳細ログの有効/無効
    PERFORMANCE_LOGS: true, // パフォーマンスログの有効/無効
    ERROR_STACK_DEPTH: 3, // エラースタックの表示深度
  },

  // プログラム ID 設定
  PROGRAM_IDS: {
    // 実際のデプロイ済みプログラムID
    EXPECTED: '7r3R1S43BS9fQbh1eBhM63u8XZJd7bYRtgMrAQRNrfcB',
    LEGACY_OLD: 'FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B',
    LEGACY_OLDER: 'EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89',
  },
} as const;

// 型安全性のための型エクスポート
export type TimeoutType = keyof typeof TEST_CONSTANTS.TIMEOUTS;
export type SuccessRateType = keyof typeof TEST_CONSTANTS.SUCCESS_RATES;
export type OperationCountType = keyof typeof TEST_CONSTANTS.OPERATION_COUNTS;
