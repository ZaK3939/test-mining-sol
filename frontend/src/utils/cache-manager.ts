// Cache manager for improved performance

import { PublicKey } from '@solana/web3.js';
import type { PDAs } from './pda-helper';
import type { ConfigAccount, UserStateAccount, FarmSpaceAccount } from '../types/program-types';

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number; // Time to live in milliseconds
}

interface CacheData {
  pdas: Map<string, CacheEntry<PDAs>>;
  configs: Map<string, CacheEntry<ConfigAccount>>;
  userStates: Map<string, CacheEntry<UserStateAccount>>;
  farmSpaces: Map<string, CacheEntry<FarmSpaceAccount>>;
}

export class CacheManager {
  private cache: CacheData;
  private readonly DEFAULT_TTL = 30000; // 30 seconds
  private readonly CONFIG_TTL = 300000; // 5 minutes for config (rarely changes)
  private readonly USER_STATE_TTL = 15000; // 15 seconds for user state
  private readonly FARM_SPACE_TTL = 20000; // 20 seconds for farm space

  constructor() {
    this.cache = {
      pdas: new Map(),
      configs: new Map(),
      userStates: new Map(),
      farmSpaces: new Map(),
    };

    // Periodic cleanup of expired entries
    setInterval(() => this.cleanup(), 60000); // Every minute
  }

  // PDA Cache
  cachePDAs(userPublicKey: PublicKey, pdas: PDAs): void {
    const key = userPublicKey.toString();
    this.cache.pdas.set(key, {
      data: pdas,
      timestamp: Date.now(),
      ttl: this.DEFAULT_TTL,
    });
  }

  getCachedPDAs(userPublicKey: PublicKey): PDAs | null {
    const key = userPublicKey.toString();
    const entry = this.cache.pdas.get(key);

    if (!entry) return null;

    if (this.isExpired(entry)) {
      this.cache.pdas.delete(key);
      return null;
    }

    return entry.data;
  }

  // Config Cache
  cacheConfig(programId: PublicKey, config: ConfigAccount): void {
    const key = programId.toString();
    this.cache.configs.set(key, {
      data: config,
      timestamp: Date.now(),
      ttl: this.CONFIG_TTL,
    });
  }

  getCachedConfig(programId: PublicKey): ConfigAccount | null {
    const key = programId.toString();
    const entry = this.cache.configs.get(key);

    if (!entry) return null;

    if (this.isExpired(entry)) {
      this.cache.configs.delete(key);
      return null;
    }

    return entry.data;
  }

  // User State Cache
  cacheUserState(userPublicKey: PublicKey, userState: UserStateAccount): void {
    const key = userPublicKey.toString();
    this.cache.userStates.set(key, {
      data: userState,
      timestamp: Date.now(),
      ttl: this.USER_STATE_TTL,
    });
  }

  getCachedUserState(userPublicKey: PublicKey): UserStateAccount | null {
    const key = userPublicKey.toString();
    const entry = this.cache.userStates.get(key);

    if (!entry) return null;

    if (this.isExpired(entry)) {
      this.cache.userStates.delete(key);
      return null;
    }

    return entry.data;
  }

  // Farm Space Cache
  cacheFarmSpace(userPublicKey: PublicKey, farmSpace: FarmSpaceAccount): void {
    const key = userPublicKey.toString();
    this.cache.farmSpaces.set(key, {
      data: farmSpace,
      timestamp: Date.now(),
      ttl: this.FARM_SPACE_TTL,
    });
  }

  getCachedFarmSpace(userPublicKey: PublicKey): FarmSpaceAccount | null {
    const key = userPublicKey.toString();
    const entry = this.cache.farmSpaces.get(key);

    if (!entry) return null;

    if (this.isExpired(entry)) {
      this.cache.farmSpaces.delete(key);
      return null;
    }

    return entry.data;
  }

  // Cache invalidation methods
  invalidateUserCache(userPublicKey: PublicKey): void {
    const key = userPublicKey.toString();
    this.cache.userStates.delete(key);
    this.cache.farmSpaces.delete(key);
  }

  invalidateAllCache(): void {
    this.cache.pdas.clear();
    this.cache.configs.clear();
    this.cache.userStates.clear();
    this.cache.farmSpaces.clear();
  }

  // Utility methods
  private isExpired<T>(entry: CacheEntry<T>): boolean {
    return Date.now() - entry.timestamp > entry.ttl;
  }

  private cleanup(): void {
    const now = Date.now();

    // Clean PDAs
    for (const [key, entry] of this.cache.pdas.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.pdas.delete(key);
      }
    }

    // Clean configs
    for (const [key, entry] of this.cache.configs.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.configs.delete(key);
      }
    }

    // Clean user states
    for (const [key, entry] of this.cache.userStates.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.userStates.delete(key);
      }
    }

    // Clean farm spaces
    for (const [key, entry] of this.cache.farmSpaces.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.farmSpaces.delete(key);
      }
    }
  }

  // Debug methods
  getCacheStats(): {
    pdas: number;
    configs: number;
    userStates: number;
    farmSpaces: number;
    totalMemoryEntries: number;
  } {
    return {
      pdas: this.cache.pdas.size,
      configs: this.cache.configs.size,
      userStates: this.cache.userStates.size,
      farmSpaces: this.cache.farmSpaces.size,
      totalMemoryEntries:
        this.cache.pdas.size +
        this.cache.configs.size +
        this.cache.userStates.size +
        this.cache.farmSpaces.size,
    };
  }
}

// Singleton instance
export const cacheManager = new CacheManager();
