# Frontend Overview

## Architecture

### Core Structure
```
frontend/
├── src/                    # Main application code
│   ├── App.tsx            # Root React component
│   ├── main.tsx           # Application entry point
│   ├── anchor-client.ts   # Full-featured Anchor client
│   ├── simple-client.ts   # Simplified client for basic operations
│   ├── solana.ts          # Solana connection utilities
│   ├── config.ts          # Configuration constants
│   ├── logger.ts          # Logging utilities
│   ├── types.ts           # Common type definitions
│   ├── components/        # React components
│   ├── types/             # TypeScript type definitions
│   └── idl/               # Anchor IDL definitions
├── scripts/               # Initialization and demo scripts
├── dist/                  # Build output
└── package.json           # Dependencies and scripts
```

## Key Components

### Client Libraries
- **AnchorClient** (`anchor-client.ts`) - Full-featured client with comprehensive functionality
- **SimpleClient** (`simple-client.ts`) - Streamlined client for core operations
- **Solana utilities** (`solana.ts`) - Connection management and wallet integration

### React Components
Located in `src/components/`:
- **App.tsx** - Main application component
- **ConnectionStatus.tsx** - Network connection status display
- **EconomicsFormulas.tsx** - Game economics visualization
- **GameControls.tsx** - Game interaction controls
- **InstructionSet.tsx** - Available game instructions display
- **InviteCodeSection.tsx** - Invite system interface
- **SystemLogs.tsx** - System event logging
- **WalletSection.tsx** - Wallet connection and management

### Configuration
- **config.ts** - Application configuration constants
- **Program ID**: `EDzDNN1v64dKgbmHc917kBiDThMV8ZrC7cLDDyGTyu89`
- **Network endpoints** and **cluster configuration**

## Scripts Directory

### Initialization Scripts
- **initialize.ts** - Basic system initialization
- **initialize-config.ts** - Configuration setup
- **example.ts** - Usage examples
- **live-runner.ts** - Live on-chain demonstration

## Development Workflow

### Build Commands
```bash
# Install dependencies
bun install

# Development server
bun run dev

# Build for production
bun run build

# Lint code
bun run lint
```

### Testing
- Integration tests located in project root `/tests/`
- Frontend-specific tests removed during cleanup for better organization

## Integration with Solana Program

### Core Instructions
1. **initialize_config** - System setup
2. **create_reward_mint** - Token mint creation
3. **init_user** - User account initialization
4. **buy_facility** - Facility purchase
5. **claim_reward** - Reward claiming

### Account Structure
- **Config** - Global system settings
- **UserState** - User data and facility ownership
- **Facility** - Facility details and grow power
- **RewardMint** - SPL token mint for rewards

### PDA Patterns
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- Facility: `["facility", user_pubkey]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`

## Technology Stack

- **React** - UI framework
- **TypeScript** - Type safety
- **Anchor** - Solana program framework client
- **@solana/web3.js** - Solana JavaScript SDK
- **@solana/spl-token** - SPL token operations
- **Vite** - Build tool and dev server
- **Bun** - Package manager and runtime

## Key Features

### Game Mechanics
- **Facility System** - Purchase and manage facilities
- **Reward System** - Time-based reward calculation and claiming
- **Economics** - Dynamic reward rates with halving mechanism
- **Invite System** - User referral and onboarding

### User Interface
- **Wallet Integration** - Connect with Solana wallets
- **Real-time Updates** - Live connection status and data
- **Interactive Controls** - Game action buttons and forms
- **System Monitoring** - Logs and status displays

## Security Considerations

- **PDA Validation** - Deterministic account addresses
- **Signature Requirements** - All user actions require signatures
- **Overflow Protection** - Checked arithmetic operations
- **Account Constraints** - Validation in instruction contexts