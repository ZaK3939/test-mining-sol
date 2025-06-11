// 型定義ファイル - program-types.tsへの統合により廃止予定
// 後方互換性のため、program-types.tsからの再エクスポート

export {
  type WalletAdapter,
  type UserStateAccount,
  type FarmSpaceAccount,
  type ConfigAccount,
  type SeedPackAccount,
  type SeedAccount,
  type TransactionResult,
  type ProgramError,
  type TokenAmount,
  SeedRarity,
  isUserStateAccount,
  isFarmSpaceAccount,
  isConfigAccount,
  safeBNToNumber,
  formatTokenAmount,
} from './types/program-types';

import { PublicKey } from '@solana/web3.js';

// UI固有の型定義のみここに残す
// ウォレット接続状態
export interface WalletState {
  connected: boolean;
  publicKey: PublicKey | null;
  balance: number;
}

// ゲーム状態（UI表示用）
export interface GameState {
  userInitialized: boolean;
  hasFarmSpace: boolean;
  growPower: number;
  tokenBalance: number;
  lastHarvestTime: number;
  pendingReferralRewards?: number;
  farmSpace?: {
    level: number;
    capacity: number;
    seedCount: number;
    totalGrowPower: number;
  };
}

// UI状態
export interface UIState {
  loading: boolean;
  error: string | null;
  success: string | null;
}

// ログエントリ
export interface LogEntry {
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'success';
  message: string;
}

// ネットワーク情報
export interface NetworkInfo {
  network: string;
  rpcUrl: string;
  programId: string;
}

// バッチリクエスト
export interface BatchRequest {
  name: string;
  address: PublicKey;
}

// 詳細なゲーム状態（内部処理用）
export interface DetailedGameState {
  userState: import('./types/program-types').UserStateAccount | null;
  farmSpace: import('./types/program-types').FarmSpaceAccount | null;
  config: import('./types/program-types').ConfigAccount | null;
  tokenBalance: number;
  userInitialized: boolean;
  hasFarmSpace: boolean;
  growPower: number;
  pendingReferralRewards: number;
}
