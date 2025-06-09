// Test setup file for Vitest
import { beforeAll } from 'vitest';

// Mock DOM elements early - before any imports that might use them
Object.defineProperty(global, 'document', {
  value: {
    readyState: 'complete',
    getElementById: () => ({
      innerHTML: '',
      scrollTop: 0,
      scrollHeight: 0,
      addEventListener: () => {},
      style: { display: 'block' },
      disabled: false,
      textContent: '',
    }),
    addEventListener: () => {},
  },
  writable: true,
});

// Mock window for testing
Object.defineProperty(global, 'window', {
  value: {
    solana: {
      isPhantom: true,
      connect: () => Promise.resolve(),
      disconnect: () => Promise.resolve(),
      on: () => {},
      off: () => {},
      publicKey: null,
      isConnected: false,
    },
  },
  writable: true,
});

// Mock Phantom wallet for testing
beforeAll(() => {
  // Mock console methods to reduce noise in tests
  global.console = {
    ...console,
    log: () => {},
    info: () => {},
    warn: () => {},
    error: () => {},
  };
});
