import { defineConfig } from 'vite';
import { resolve } from 'path';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';

export default defineConfig({
  // 環境変数の設定
  envDir: '.',
  envPrefix: 'VITE_',

  // 開発サーバー設定
  server: {
    port: 3000,
    open: true,
  },

  // ビルド設定
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      plugins: [
        NodeGlobalsPolyfillPlugin({
          buffer: true,
        }),
      ],
    },
  },

  // 依存関係の最適化
  optimizeDeps: {
    include: ['@solana/web3.js', '@coral-xyz/anchor', 'buffer'],
    esbuildOptions: {
      plugins: [
        NodeGlobalsPolyfillPlugin({
          buffer: true,
        }),
      ],
    },
  },

  // Node.js polyfills for browser
  define: {
    global: 'globalThis',
    'process.env': {},
  },

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      buffer: 'buffer',
    },
  },

  // Vitest configuration
  test: {
    environment: 'node',
    globals: true,
    setupFiles: './src/test/setup.ts',
  },
});
