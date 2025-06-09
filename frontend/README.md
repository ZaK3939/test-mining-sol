# Facility Game Frontend

TypeScript frontend for the Solana Facility Game using Vite and Bun.

## Prerequisites

- [Bun](https://bun.sh/) - Fast JavaScript runtime and package manager
- A Solana wallet (e.g., Phantom)

## Getting Started

### Prerequisites for Local Development

First, make sure the Solana program is deployed to your local validator:

1. Start a local Solana validator (in project root):
```bash
solana-test-validator
```

2. Configure Solana CLI for local development:
```bash
solana config set --url localhost
```

3. Build and deploy the Solana program (in project root):
```bash
anchor build
anchor deploy --provider.cluster localnet
```

### Frontend Development

1. Install dependencies:
```bash
bun install
```

2. Start the development server:
```bash
bun run dev
```

3. Open your browser at http://localhost:3000

4. Build for production:
```bash
bun run build
```

5. Preview production build:
```bash
bun run preview
```

6. Type check:
```bash
bun run type-check
```

7. Run tests:
```bash
bun test:run
```

## Project Structure

```
frontend/
├── src/
│   ├── config.ts      # Configuration (RPC endpoints, program ID)
│   ├── logger.ts      # Logging utilities
│   ├── main.ts        # Application entry point
│   ├── solana.ts      # Solana interaction logic
│   ├── types.ts       # TypeScript type definitions
│   └── vite-env.d.ts  # Vite environment types
├── index.html         # Main HTML file
├── package.json       # Dependencies and scripts
├── tsconfig.json      # TypeScript configuration
└── vite.config.ts     # Vite configuration
```

## How to Use

### Game Flow

1. **Connect Wallet**: Click "ウォレット接続" to connect your Phantom wallet
2. **Get SOL**: Use "SOLエアドロップ (開発用)" to get test SOL for transactions
3. **Initialize User**: Click "ユーザー初期化" to create your game account
4. **Buy Facility**: Click "施設購入" to buy your first facility (costs SOL, gives 100 Grow Power)
5. **Claim Rewards**: Click "報酬請求" to claim time-based rewards (SPL tokens)
6. **Refresh Data**: Click "データ更新" to see updated game state

### Game Mechanics

- **Grow Power**: Determines reward generation rate
- **Time-based Rewards**: Rewards accumulate over time based on Grow Power
- **Halving Mechanism**: Base reward rate halves periodically (default: 1 year)
- **SPL Tokens**: Rewards are minted as custom SPL tokens

## Features

- TypeScript for type safety
- Vite for fast development and optimized builds
- Bun for fast package management and script execution
- Solana Web3.js and Anchor integration
- Wallet adapter support
- Polyfills for browser compatibility
- Real-time game state updates
- Error handling and logging