// Program specific types for improved type safety

import { PublicKey, Transaction, VersionedTransaction } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// Wallet adapter interface for type safety
export interface WalletAdapter {
  publicKey: PublicKey;
  signTransaction: <T extends Transaction | VersionedTransaction>(tx: T) => Promise<T>;
  signAllTransactions: <T extends Transaction | VersionedTransaction>(txs: T[]) => Promise<T[]>;
}

// Program account interfaces with strict typing
export interface UserStateAccount {
  owner: PublicKey;
  totalGrowPower: BN;
  lastHarvestTime: BN;
  hasFacility: boolean;
  referrer: PublicKey | null;
  pendingReferralRewards: BN;
  reserve: number[];
}

export interface FacilityAccount {
  owner: PublicKey;
  machineCount: number;
  totalGrowPower: BN;
  facilitySize: number;
  maxCapacity: number;
  reserve: number[];
}

export interface ConfigAccount {
  baseRate: BN;
  halvingInterval: BN;
  nextHalvingTime: BN;
  admin: PublicKey;
  treasury: PublicKey;
  mysteryBoxCost: BN;
  facilityCounter: BN;
  mysteryBoxCounter: BN;
  seedCounter: BN;
  reserve: number[];
}

export interface MysteryBoxAccount {
  owner: PublicKey;
  id: BN;
  isOpened: boolean;
  reserve: number[];
}

// Seed rarity enum for type safety
export enum SeedRarity {
  Common = 'common',
  Rare = 'rare',
  Epic = 'epic',
  Legendary = 'legendary',
}

export interface SeedAccount {
  owner: PublicKey;
  id: BN;
  rarity: { [key in SeedRarity]?: {} };
  growPowerBonus: BN;
  reserve: number[];
}

// Transaction result types
export type TransactionResult = string | 'already_initialized' | 'already_owned';

// Error types for better error handling
export interface ProgramError extends Error {
  code?: number;
  logs?: string[];
}

// Type guards for runtime validation
export function isUserStateAccount(account: unknown): account is UserStateAccount {
  return (
    typeof account === 'object' &&
    account !== null &&
    'owner' in account &&
    'totalGrowPower' in account &&
    'lastHarvestTime' in account &&
    'hasFacility' in account
  );
}

export function isFacilityAccount(account: unknown): account is FacilityAccount {
  return (
    typeof account === 'object' &&
    account !== null &&
    'owner' in account &&
    'machineCount' in account &&
    'totalGrowPower' in account &&
    'facilitySize' in account &&
    'maxCapacity' in account
  );
}

export function isConfigAccount(account: unknown): account is ConfigAccount {
  return (
    typeof account === 'object' &&
    account !== null &&
    'baseRate' in account &&
    'halvingInterval' in account &&
    'admin' in account &&
    'treasury' in account
  );
}

// Utility types for instruction contexts
export interface InstructionContext<T = Record<string, PublicKey>> {
  accounts: T;
  signers?: PublicKey[];
}

// Token amount utility type
export interface TokenAmount {
  amount: BN;
  decimals: number;
  uiAmount: number;
  uiAmountString: string;
}

// Helper function for safe BN conversion
export function safeBNToNumber(bn?: BN | null): number {
  if (!bn) {
    return 0;
  }
  
  const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;
  const bnNumber = bn.toNumber();
  
  if (bnNumber > MAX_SAFE_INTEGER) {
    throw new Error(`BN value ${bn.toString()} exceeds MAX_SAFE_INTEGER`);
  }
  
  return bnNumber;
}

// Helper function for safe token amount conversion
export function formatTokenAmount(amount: BN, decimals: number = 6): TokenAmount {
  const uiAmount = amount.toNumber() / Math.pow(10, decimals);
  
  return {
    amount,
    decimals,
    uiAmount,
    uiAmountString: uiAmount.toFixed(decimals),
  };
}