// 型定義ファイル

import { PublicKey } from '@solana/web3.js';

// ウォレット接続状態
export interface WalletState {
  connected: boolean;
  publicKey: PublicKey | null;
  balance: number;
}

// ゲーム状態
export interface GameState {
  userInitialized: boolean;
  hasFacility: boolean;
  growPower: number;
  tokenBalance: number;
  lastHarvestTime: number;
}

// プログラムアカウント（簡易版）
export interface UserStateAccount {
  owner: PublicKey;
  totalGrowPower: number;
  lastHarvestTime: number;
  hasFacility: boolean;
}

export interface FacilityAccount {
  owner: PublicKey;
  machineCount: number;
  totalGrowPower: number;
}

export interface ConfigAccount {
  baseRate: number;
  halvingInterval: number;
  nextHalvingTime: number;
  admin: PublicKey;
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
