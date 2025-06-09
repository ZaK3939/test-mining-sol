import { describe, it, expect } from 'vitest';
import { config } from '../config';

describe('Config', () => {
  it('should load default config', () => {
    expect(config).toHaveProperty('programId');
    expect(config).toHaveProperty('network');
    expect(config).toHaveProperty('rpcUrl');
    expect(config.programId).toBe('EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89');
  });

  it('should have valid program ID format', () => {
    // Should be a valid base58 string of correct length
    expect(config.programId).toMatch(/^[1-9A-HJ-NP-Za-km-z]{32,44}$/);
  });

  it('should have valid network values', () => {
    expect(['mainnet-beta', 'devnet', 'testnet', 'localhost']).toContain(config.network);
  });

  it('should have valid RPC URL format', () => {
    expect(config.rpcUrl).toMatch(/^https?:\/\//);
  });
});
