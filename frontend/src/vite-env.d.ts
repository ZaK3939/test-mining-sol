/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_HELIUS_API_KEY_DEVNET: string;
  readonly VITE_HELIUS_API_KEY_MAINNET: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
