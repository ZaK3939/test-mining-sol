/**
 * Modern Test Setup for Vitest
 * Comprehensive setup with mocking, polyfills, and environment configuration
 */

import { beforeAll, beforeEach, afterEach, vi } from 'vitest';
import { TextEncoder, TextDecoder } from 'util';

// =============== GLOBAL POLYFILLS ===============

// Node.js polyfills for browser APIs
Object.assign(global, {
  TextEncoder,
  TextDecoder,
  performance: {
    now: () => Date.now(),
    mark: () => {},
    measure: () => {},
  },
});

// Buffer polyfill for Solana libraries
import { Buffer } from 'buffer';
global.Buffer = Buffer;

// =============== DOM MOCKING ===============

// Comprehensive DOM mock
Object.defineProperty(global, 'document', {
  value: {
    readyState: 'complete',
    createElement: (tagName: string) => ({
      tagName: tagName.toUpperCase(),
      innerHTML: '',
      textContent: '',
      style: {},
      setAttribute: vi.fn(),
      getAttribute: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      click: vi.fn(),
      focus: vi.fn(),
      blur: vi.fn(),
    }),
    getElementById: (id: string) => ({
      id,
      innerHTML: '',
      textContent: '',
      style: { display: 'block' },
      classList: {
        add: vi.fn(),
        remove: vi.fn(),
        contains: vi.fn(() => false),
        toggle: vi.fn(),
      },
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      click: vi.fn(),
      focus: vi.fn(),
      blur: vi.fn(),
      disabled: false,
      value: '',
    }),
    querySelector: vi.fn(),
    querySelectorAll: vi.fn(() => []),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    body: {
      appendChild: vi.fn(),
      removeChild: vi.fn(),
      classList: {
        add: vi.fn(),
        remove: vi.fn(),
      },
    },
  },
  writable: true,
});

// =============== WINDOW MOCKING ===============

// Enhanced window mock with wallet APIs
Object.defineProperty(global, 'window', {
  value: {
    // Phantom wallet mock
    solana: {
      isPhantom: true,
      connect: vi.fn().mockResolvedValue({
        publicKey: 'mock_public_key',
      }),
      disconnect: vi.fn().mockResolvedValue(undefined),
      signTransaction: vi.fn().mockResolvedValue('mock_signed_transaction'),
      signAllTransactions: vi.fn().mockResolvedValue(['mock_signed_transaction']),
      on: vi.fn(),
      off: vi.fn(),
      publicKey: null,
      isConnected: false,
    },
    
    // Other wallet providers
    ethereum: undefined, // No MetaMask in Solana context
    
    // Browser APIs
    localStorage: {
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      length: 0,
      key: vi.fn(),
    },
    
    sessionStorage: {
      getItem: vi.fn(() => null),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      length: 0,
      key: vi.fn(),
    },
    
    location: {
      href: 'http://localhost:3000',
      origin: 'http://localhost:3000',
      pathname: '/',
      search: '',
      hash: '',
      reload: vi.fn(),
    },
    
    navigator: {
      userAgent: 'test-agent',
      language: 'en-US',
      onLine: true,
      clipboard: {
        writeText: vi.fn().mockResolvedValue(undefined),
        readText: vi.fn().mockResolvedValue(''),
      },
    },
    
    // Console APIs
    alert: vi.fn(),
    confirm: vi.fn(() => true),
    prompt: vi.fn(() => 'test-input'),
    
    // Timing APIs
    setTimeout: vi.fn((fn, delay) => setTimeout(fn, delay)),
    clearTimeout: vi.fn(clearTimeout),
    setInterval: vi.fn((fn, delay) => setInterval(fn, delay)),
    clearInterval: vi.fn(clearInterval),
    
    // Request APIs
    requestAnimationFrame: vi.fn((fn) => setTimeout(fn, 16)),
    cancelAnimationFrame: vi.fn(),
  },
  writable: true,
});

// =============== CRYPTO MOCKING ===============

// Mock crypto APIs for secure random generation
Object.defineProperty(global, 'crypto', {
  value: {
    getRandomValues: vi.fn((arr: any) => {
      for (let i = 0; i < arr.length; i++) {
        arr[i] = Math.floor(Math.random() * 256);
      }
      return arr;
    }),
    randomUUID: vi.fn(() => 'test-uuid-1234-5678-9abc-def0'),
  },
  writable: true,
});

// =============== FETCH MOCKING ===============

// Mock fetch API
global.fetch = vi.fn().mockResolvedValue({
  ok: true,
  status: 200,
  statusText: 'OK',
  json: vi.fn().mockResolvedValue({}),
  text: vi.fn().mockResolvedValue(''),
  blob: vi.fn().mockResolvedValue(new Blob()),
  headers: new Map(),
});

// =============== ERROR HANDLING ===============

// Capture unhandled errors and rejections
const originalConsoleError = console.error;
let capturedErrors: Error[] = [];

beforeEach(() => {
  capturedErrors = [];
  
  // Override console.error to capture errors during tests
  console.error = (...args: any[]) => {
    // Only capture actual Error objects
    const error = args.find(arg => arg instanceof Error);
    if (error) {
      capturedErrors.push(error);
    }
    
    // Still log to original console in development
    if (process.env.NODE_ENV === 'development') {
      originalConsoleError(...args);
    }
  };
});

afterEach(() => {
  // Restore original console.error
  console.error = originalConsoleError;
  
  // Check for uncaught errors (optional - can be enabled for strict testing)
  if (process.env.STRICT_ERROR_TESTING === 'true' && capturedErrors.length > 0) {
    throw new Error(
      `Uncaught errors detected during test: ${capturedErrors
        .map(e => e.message)
        .join(', ')}`
    );
  }
});

// =============== TEST ENVIRONMENT SETUP ===============

beforeAll(() => {
  // Set test environment variables
  process.env.NODE_ENV = 'test';
  process.env.VITE_SOLANA_NETWORK = 'localhost';
  process.env.VITE_PROGRAM_ID = 'FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B';
  
  // Suppress console methods during tests (can be overridden per test)
  if (process.env.VERBOSE_TESTS !== 'true') {
    global.console = {
      ...console,
      log: vi.fn(),
      info: vi.fn(),
      warn: vi.fn(),
      debug: vi.fn(),
      // Keep error for important debugging
      error: console.error,
    };
  }
  
  // Set up fake timers if needed
  if (process.env.USE_FAKE_TIMERS === 'true') {
    vi.useFakeTimers();
  }
});

// =============== TEST UTILITIES ===============

// Global test utilities available in all tests
declare global {
  // eslint-disable-next-line no-var
  var testUtils: {
    waitForNextTick: () => Promise<void>;
    flushPromises: () => Promise<void>;
    mockConsole: () => void;
    restoreConsole: () => void;
    captureErrors: () => Error[];
  };
}

global.testUtils = {
  // Wait for next tick (useful for async operations)
  waitForNextTick: () => new Promise(resolve => process.nextTick(resolve)),
  
  // Flush all pending promises
  flushPromises: () => new Promise(resolve => setTimeout(resolve, 0)),
  
  // Mock console methods
  mockConsole: () => {
    global.console = {
      ...console,
      log: vi.fn(),
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
      debug: vi.fn(),
    };
  },
  
  // Restore original console
  restoreConsole: () => {
    global.console = {
      ...console,
      error: originalConsoleError,
    };
  },
  
  // Get captured errors from current test
  captureErrors: () => [...capturedErrors],
};

// =============== CLEANUP ===============

afterEach(() => {
  // Clear all mocks after each test
  vi.clearAllMocks();
  
  // Reset DOM state
  if (global.document && global.document.body) {
    global.document.body.innerHTML = '';
  }
  
  // Clear localStorage and sessionStorage
  if (global.window?.localStorage) {
    (global.window.localStorage.clear as any)();
  }
  if (global.window?.sessionStorage) {
    (global.window.sessionStorage.clear as any)();
  }
  
  // Reset wallet state
  if (global.window?.solana) {
    global.window.solana.publicKey = null;
    global.window.solana.isConnected = false;
  }
});
