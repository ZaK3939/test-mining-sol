# Solana Facility Game MVP

Solanaブロックチェーン上で動作する施設経営ゲームのMVP（Minimum Viable Product）です。Anchorフレームワークを使用して実装されています。

## 主な機能

### Phase 1: MVP機能
1. **施設購入システム** - 各ユーザー1施設まで、初期マシン自動配置
2. **Grow Power & 遅延計算** - 時間経過による報酬蓄積システム
3. **報酬請求（Claim）** - SPLトークンとしての報酬受け取り
4. **半減期システム** - 定期的な報酬レート減少メカニズム
5. **PDA設計** - 将来拡張に対応した拡張可能なアカウント構造

## アーキテクチャ

### アカウント構造
- **Config**: グローバル設定（基本レート、半減期設定）
- **UserState**: ユーザー状態（PDA: `["user", user_pubkey]`）
- **Facility**: 施設情報（PDA: `["facility", user_pubkey]`）
- **RewardMint**: 報酬トークンMint（PDA: `["reward_mint"]`）

### 命令（Instructions）
1. `initialize_config` - システム設定初期化
2. `create_reward_mint` - 報酬トークンMint作成
3. `init_user` - ユーザーアカウント初期化
4. `buy_facility` - 施設購入 + 初期マシン配置
5. `claim_reward` - 報酬請求（時間計算 + トークンMint）

## セットアップ手順

### 1. 必要なツールのインストール

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"

# Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Node.js dependencies
npm install -g yarn
```

### 2. プロジェクトの初期化

```bash
# 新しいAnchorプロジェクトを作成
anchor init facility-game --no-git
cd facility-game

# 提供されたファイルを対応する場所に配置
```

### 3. ファイル構成

```
facility-game/
├── programs/facility-game/src/
│   ├── lib.rs              # メインプログラム
│   ├── state.rs            # アカウント構造体
│   ├── instructions/
│   │   └── mod.rs          # 命令コンテキスト
│   └── error.rs            # エラー定義
├── tests/
│   └── facility-game.ts    # テストファイル
├── Anchor.toml             # Anchor設定
├── Cargo.toml              # Rust依存関係
├── package.json            # Node.js設定
└── tsconfig.json           # TypeScript設定
```

### 4. ビルドとテスト

```bash
# 依存関係インストール
yarn install

# プログラムビルド
anchor build

# ローカルバリデータ起動（別ターミナル）
solana-test-validator

# テスト実行
anchor test --skip-local-validator
```

### 5. デプロイ

```bash
# Devnetに設定変更
solana config set --url devnet

# エアドロップ（Devnet）
solana airdrop 2

# Devnetにデプロイ
anchor deploy --provider.cluster devnet
```

## 使用例

### 1. システム初期化（管理者）

```bash
# Config初期化（基本レート: 10, 半減期: 1年）
anchor run initialize-config
```

### 2. ユーザーの利用フロー

```typescript
// 1. ユーザー初期化
await program.methods.initUser().accounts({...}).rpc();

// 2. 施設購入
await program.methods.buyFacility().accounts({...}).rpc();

// 3. 時間経過後、報酬請求
await program.methods.claimReward().accounts({...}).rpc();
```

## 技術仕様

### 報酬計算式
```
報酬 = (経過時間[秒] × Grow Power × base_rate) / 1000
```

### 半減期
- 設定間隔ごとに `base_rate` が半減
- デフォルト: 1年間隔

### PDA Seeds
- Config: `["config"]`
- UserState: `["user", user_pubkey]`
- Facility: `["facility", user_pubkey]`
- RewardMint: `["reward_mint"]`
- MintAuthority: `["mint_authority"]`

## 将来の拡張計画

### Phase 2予定機能
- 複数マシンタイプ
- マシンアップグレードシステム
- 紹介報酬システム
- 複数施設所有

### 拡張可能設計
- 各アカウントに64バイトの `reserve` フィールド
- PDA構造による安全なアカウント管理
- モジュラー命令設計

## トラブルシューティング

### よくある問題

1. **Build Error**: Anchorのバージョン確認
2. **Test Failure**: Solana Test Validatorが起動しているか確認
3. **Deploy Error**: 十分なSOL残高があるか確認

### デバッグ方法

```bash
# ログ確認
solana logs

# アカウント状態確認
solana account <PDA_ADDRESS>

# プログラムログ
anchor test --skip-deploy -- --grep "test_name"
```

## ライセンス

MIT License# test-mining-sol
