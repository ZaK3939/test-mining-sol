// Program specific types for improved type safety

import { PublicKey, Transaction, VersionedTransaction } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// Wallet adapter interface for type safety
export interface WalletAdapter {
  publicKey: PublicKey;
  signTransaction: <T extends Transaction | VersionedTransaction>(tx: T) => Promise<T>;
  signAllTransactions: <T extends Transaction | VersionedTransaction>(txs: T[]) => Promise<T[]>;
}

// Program account interfaces matching current state.rs
export interface UserStateAccount {
  owner: PublicKey;
  totalGrowPower: BN;
  lastHarvestTime: BN;
  hasFarmSpace: boolean;
  referrer: PublicKey | null;
  pendingReferralRewards: BN;
  reserve: number[];
}

export interface FarmSpaceAccount {
  owner: PublicKey;
  level: number;
  capacity: number;
  seedCount: number;
  totalGrowPower: BN;
  reserve: number[];
}

export interface ConfigAccount {
  baseRate: BN;
  halvingInterval: BN;
  nextHalvingTime: BN;
  admin: PublicKey;
  treasury: PublicKey;
  seedPackCost: BN;
  seedCounter: BN;
  seedPackCounter: BN;
  farmSpaceCostSol: BN;
  maxInviteLimit: number;
  tradingFeePercentage: number;
  protocolReferralAddress: PublicKey;
  totalSupplyMinted: BN;
  operator: PublicKey;
  reserve: number[];
}

export interface SeedPackAccount {
  purchaser: PublicKey;
  purchasedAt: BN;
  costPaid: BN;
  vrfFeePaid: BN;
  isOpened: boolean;
  vrfSequence: BN;
  userEntropySeed: BN;
  finalRandomValue: BN;
  packId: BN;
  vrfAccount: PublicKey;
  reserve: number[];
}

// Seed types matching program enum
export enum SeedType {
  Seed1 = 'Seed1',
  Seed2 = 'Seed2', 
  Seed3 = 'Seed3',
  Seed4 = 'Seed4',
  Seed5 = 'Seed5',
  Seed6 = 'Seed6',
  Seed7 = 'Seed7',
  Seed8 = 'Seed8',
  Seed9 = 'Seed9',
}

export interface SeedAccount {
  owner: PublicKey;
  seedType: SeedType;
  growPower: BN;
  isPlanted: boolean;
  plantedFarmSpace: PublicKey | null;
  createdAt: BN;
  seedId: BN;
  reserve: number[];
}

// Seed storage for bulk operations
export interface SeedStorageAccount {
  owner: PublicKey;
  seedIds: BN[];
  totalSeeds: number;
  seedTypeCounts: number[];
  reserve: number[];
}

export interface GlobalStatsAccount {
  totalGrowPower: BN;
  totalFarmSpaces: BN;
  totalSupply: BN;
  currentRewardsPerSecond: BN;
  lastUpdateTime: BN;
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
    'hasFarmSpace' in account &&
    'pendingReferralRewards' in account
  );
}

export function isFarmSpaceAccount(account: unknown): account is FarmSpaceAccount {
  return (
    typeof account === 'object' &&
    account !== null &&
    'owner' in account &&
    'level' in account &&
    'capacity' in account &&
    'seedCount' in account &&
    'totalGrowPower' in account
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
