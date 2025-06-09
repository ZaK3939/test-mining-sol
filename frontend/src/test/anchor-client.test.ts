import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PublicKey, Connection } from '@solana/web3.js';
import { AnchorClient } from '../anchor-client';

// Mock dependencies
vi.mock('@coral-xyz/anchor', () => ({
  AnchorProvider: vi.fn(() => ({
    connection: {},
    wallet: { publicKey: new PublicKey('11111111111111111111111111111112') },
  })),
  Program: vi.fn(() => ({
    programId: new PublicKey('EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89'),
    methods: {
      initUser: vi.fn(() => ({
        accounts: vi.fn(() => ({ rpc: vi.fn(() => Promise.resolve('mock-tx-id')) })),
      })),
      buyFacility: vi.fn(() => ({
        accounts: vi.fn(() => ({ rpc: vi.fn(() => Promise.resolve('mock-tx-id')) })),
      })),
      claimReward: vi.fn(() => ({
        accounts: vi.fn(() => ({
          preInstructions: vi.fn(() => ({ rpc: vi.fn(() => Promise.resolve('mock-tx-id')) })),
          rpc: vi.fn(() => Promise.resolve('mock-tx-id')),
        })),
      })),
    },
    account: {
      userState: {
        fetchNullable: vi.fn(() => Promise.resolve(null)),
      },
      facility: {
        fetchNullable: vi.fn(() => Promise.resolve(null)),
      },
      config: {
        fetchNullable: vi.fn(() => Promise.resolve(null)),
      },
    },
  })),
  BN: vi.fn(),
}));

vi.mock('@solana/spl-token', () => ({
  getAssociatedTokenAddress: vi.fn(() =>
    Promise.resolve(new PublicKey('11111111111111111111111111111112'))
  ),
  TOKEN_PROGRAM_ID: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
  ASSOCIATED_TOKEN_PROGRAM_ID: new PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'),
  createAssociatedTokenAccountInstruction: vi.fn(),
}));

describe('AnchorClient', () => {
  let client: AnchorClient;
  let mockConnection: Connection;
  let mockWallet: any;

  beforeEach(() => {
    mockConnection = {} as Connection;
    mockWallet = {
      publicKey: new PublicKey('11111111111111111111111111111112'),
    };
    client = new AnchorClient(mockConnection, mockWallet);
  });

  describe('calculatePDAs', () => {
    it('should calculate correct PDAs for user', async () => {
      const userPublicKey = new PublicKey('11111111111111111111111111111112');
      const pdas = await client.calculatePDAs(userPublicKey);

      expect(pdas).toHaveProperty('userState');
      expect(pdas).toHaveProperty('facility');
      expect(pdas).toHaveProperty('config');
      expect(pdas).toHaveProperty('rewardMint');
      expect(pdas).toHaveProperty('mintAuthority');
    });
  });

  describe('initUser', () => {
    it('should initialize user successfully', async () => {
      const result = await client.initUser();
      expect(result).toBe('mock-tx-id');
    });

    it('should throw error if user already initialized', async () => {
      // Mock user already exists
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce({
        owner: mockWallet.publicKey,
        hasFacility: false,
      });

      const result = await client.initUser();
      expect(result).toBe('already_initialized');
    });
  });

  describe('buyFacility', () => {
    beforeEach(() => {
      // Mock user is initialized
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValue({
        owner: mockWallet.publicKey,
        hasFacility: false,
      });
    });

    it('should buy facility successfully', async () => {
      const result = await client.buyFacility();
      expect(result).toBe('mock-tx-id');
    });

    it('should throw error if user not initialized', async () => {
      // Mock user not initialized
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce(null);

      await expect(client.buyFacility()).rejects.toThrow('先にユーザーを初期化してください');
    });

    it('should throw error if user already has facility', async () => {
      // Mock user already has facility
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce({
        owner: mockWallet.publicKey,
        hasFacility: true,
      });

      const result = await client.buyFacility();
      expect(result).toBe('already_owned');
    });
  });

  describe('claimRewards', () => {
    beforeEach(() => {
      // Mock user is initialized and has facility
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValue({
        owner: mockWallet.publicKey,
        hasFacility: true,
      });

      // Mock connection methods
      const mockProvider = (client as any).provider;
      mockProvider.connection = {
        getAccountInfo: vi.fn(() => Promise.resolve(null)),
        getTokenAccountBalance: vi.fn(() =>
          Promise.resolve({
            value: { amount: '1000000', decimals: 6 },
          })
        ),
      };
    });

    it('should claim rewards successfully', async () => {
      const result = await client.claimRewards();
      expect(result).toBe('mock-tx-id');
    });

    it('should throw error if user not initialized', async () => {
      // Mock user not initialized
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce(null);

      await expect(client.claimRewards()).rejects.toThrow('先にユーザーを初期化してください');
    });

    it('should throw error if user has no facility', async () => {
      // Mock user has no facility
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce({
        owner: mockWallet.publicKey,
        hasFacility: false,
      });

      await expect(client.claimRewards()).rejects.toThrow('先に施設を購入してください');
    });
  });

  describe('fetchUserState', () => {
    it('should fetch user state successfully', async () => {
      const mockUserState = {
        owner: mockWallet.publicKey,
        hasFacility: false,
      };

      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce(mockUserState);

      const result = await client.fetchUserState(mockWallet.publicKey);
      expect(result).toEqual(mockUserState);
    });

    it('should return null if user state not found', async () => {
      const mockProgram = (client as any).program;
      mockProgram.account.userState.fetchNullable.mockResolvedValueOnce(null);

      const result = await client.fetchUserState(mockWallet.publicKey);
      expect(result).toBeNull();
    });
  });

  describe('getTokenBalance', () => {
    it('should return token balance', async () => {
      const mockProvider = (client as any).provider;
      mockProvider.connection = {
        getTokenAccountBalance: vi.fn(() =>
          Promise.resolve({
            value: { amount: '1000000', decimals: 6 },
          })
        ),
      };

      const balance = await client.getTokenBalance(mockWallet.publicKey);
      expect(balance).toBe(1); // 1000000 / 10^6
    });

    it('should return 0 if token account does not exist', async () => {
      const mockProvider = (client as any).provider;
      mockProvider.connection = {
        getTokenAccountBalance: vi.fn(() => Promise.reject(new Error('Account not found'))),
      };

      const balance = await client.getTokenBalance(mockWallet.publicKey);
      expect(balance).toBe(0);
    });
  });
});
