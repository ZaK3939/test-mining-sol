# 確率テーブル管理ガイド（運営者向け）

## 📋 概要

このドキュメントは、運営がゲーム途中で確率テーブルを動的に変更する方法を説明します。新しい動的確率テーブルシステムにより、コードをデプロイせずにシード生成確率をリアルタイムで調整できます。

## 🎯 現在の設定（確率テーブル1）

現在のシステムは**確率テーブル1**（6種類のシード）に設定されています：

| シード名 | Grow Power | 確率 | 累積閾値 |
|---------|------------|------|----------|
| OG Kush | 100 | 43% | 4300 |
| Blue Dream | 180 | 25% | 6800 |
| Sour Diesel | 420 | 14% | 8200 |
| Girl Scout Cookies | 720 | 9% | 9100 |
| Gorilla Glue | 1000 | 6% | 9700 |
| Skywalker Kush | 5000 | 3% | 10000 |

**期待値**: 421.6 Grow Power per pack

## 🔧 システム初期化

### 1. 確率テーブルアカウントの初期化

```bash
# Anchor CLIを使用した初期化
anchor invoke initialize_probability_table \
  --provider.cluster devnet \
  --program-id FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B
```

### 2. 初期設定確認

```bash
# 確率テーブルアカウントの状態確認
solana account <probability_table_address> \
  --output json-compact \
  --url devnet
```

## 📊 確率テーブル更新方法

### 1. 確率テーブル2への変更例

確率テーブル2（9種類のシード）に変更する場合：

```bash
anchor invoke update_probability_table \
  --provider.cluster devnet \
  --program-id FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B \
  --args '{
    "version": 2,
    "seedCount": 9,
    "growPowers": [100, 180, 420, 720, 1000, 5000, 15000, 30000, 60000],
    "probabilityThresholds": [4222, 6666, 7999, 8832, 9388, 9721, 9854, 9943, 10000],
    "probabilityPercentages": [42.23, 24.44, 13.33, 8.33, 5.56, 3.33, 1.33, 0.89, 0.56],
    "expectedValue": 1226,
    "name": "Table2_9Seeds"
  }'
```

### 2. カスタム確率テーブルの作成例

イベント用特別確率テーブルの例：

```bash
anchor invoke update_probability_table \
  --provider.cluster devnet \
  --program-id FA1xdxZNykyJaMsuSekWJrUzwY8PVh1Usn7mR8eWmw5B \
  --args '{
    "version": 3,
    "seedCount": 6,
    "growPowers": [100, 180, 420, 720, 1000, 5000],
    "probabilityThresholds": [2000, 4000, 6000, 7500, 9000, 10000],
    "probabilityPercentages": [20.0, 20.0, 20.0, 15.0, 15.0, 10.0],
    "expectedValue": 845,
    "name": "SpecialEvent_Balanced"
  }'
```

## 🎲 確率テーブル設計ガイド

### 1. パラメータ説明

- **version**: テーブルのバージョン番号（追跡用）
- **seedCount**: シード種類数（1-9）
- **growPowers**: 各シードのGrow Power値
- **probabilityThresholds**: 累積確率閾値（10,000分率、最後は必ず10000）
- **probabilityPercentages**: 表示用確率（パーセント）
- **expectedValue**: 期待値（Grow Power）
- **name**: テーブル名（最大32文字）

### 2. 確率設計の注意点

#### ✅ 正しい設定例
```json
{
  "probabilityThresholds": [4300, 6800, 8200, 9100, 9700, 10000],
  "probabilityPercentages": [43.0, 25.0, 14.0, 9.0, 6.0, 3.0]
}
```

#### ❌ 間違った設定例
```json
{
  // 昇順でない
  "probabilityThresholds": [4300, 6800, 8200, 9100, 9500, 9700],
  // 最後が10000でない
  "probabilityThresholds": [4300, 6800, 8200, 9100, 9700, 9999]
}
```

### 3. 期待値計算式

```
期待値 = Σ(確率 × Grow Power)

例：確率テーブル1
= 0.43 × 100 + 0.25 × 180 + 0.14 × 420 + 0.09 × 720 + 0.06 × 1000 + 0.03 × 5000
= 43 + 45 + 58.8 + 64.8 + 60 + 150
= 421.6 GP
```

## 🛡️ セキュリティ要件

### 1. 権限確認

- **管理者署名必須**: `config.admin`アカウントの署名が必要
- **アカウント検証**: 確率テーブルアカウントのPDA検証

### 2. 入力検証

システムは以下を自動的に検証します：
- シード数は1-9の範囲内
- 配列サイズの一致
- 確率閾値の昇順ソート
- 最終閾値が10000
- 名前の長さ制限（32文字以内）

## 📈 運用シナリオ例

### 1. 通常運用
- **確率テーブル1**: 日常的なバランス（6種類）
- **期待値**: 421.6 GP
- **用途**: 安定したゲーム体験

### 2. イベント時
- **確率テーブル2**: 高レアリティ追加（9種類）
- **期待値**: 1,226.79 GP
- **用途**: 特別イベント、プロモーション

### 3. バランス調整
- **カスタムテーブル**: 細かな調整
- **期待値**: 要件に応じて
- **用途**: 経済バランス微調整

## 🔄 更新手順の詳細

### 1. 事前準備
```bash
# 1. 現在の設定確認
anchor test --skip-build --skip-deploy

# 2. 新しい確率テーブル設計
# - エクセル/計算ツールで確率と期待値を計算
# - 閾値の正確性を確認
```

### 2. 更新実行
```bash
# 1. テストネットでテスト
anchor invoke update_probability_table \
  --provider.cluster devnet \
  --args '<新しい設定>'

# 2. 動作確認
anchor test --grep "probability"

# 3. メインネットに適用
anchor invoke update_probability_table \
  --provider.cluster mainnet-beta \
  --args '<新しい設定>'
```

### 3. 事後確認
```bash
# 1. 更新されたテーブル確認
solana account <probability_table_address>

# 2. シード生成テスト
# - 複数回のパック開封をテスト
# - 確率分布の確認
```

## 📊 モニタリング

### 1. 確率分布の監視

```javascript
// 確率分布チェックスクリプト例
const results = await testSeedGeneration(1000); // 1000回テスト
const distribution = calculateDistribution(results);
console.log('期待確率 vs 実際確率:', distribution);
```

### 2. 期待値の監視

```javascript
// 期待値チェック
const actualValue = calculateActualExpectedValue(results);
const theoreticalValue = 421.6; // テーブル1の場合
const deviation = Math.abs(actualValue - theoreticalValue) / theoreticalValue;
console.log('期待値偏差:', deviation * 100, '%');
```

## 🚨 緊急時対応

### 1. 問題のある確率テーブルの復旧

```bash
# 確率テーブル1（安全な設定）に戻す
anchor invoke update_probability_table \
  --provider.cluster mainnet-beta \
  --args '{
    "version": 999,
    "seedCount": 6,
    "growPowers": [100, 180, 420, 720, 1000, 5000],
    "probabilityThresholds": [4300, 6800, 8200, 9100, 9700, 10000],
    "probabilityPercentages": [43.0, 25.0, 14.0, 9.0, 6.0, 3.0],
    "expectedValue": 421,
    "name": "Emergency_Rollback"
  }'
```

### 2. 問題診断

```bash
# ログ確認
solana logs --url mainnet-beta | grep "probability"

# アカウント状態確認
solana account <probability_table_address> --output json
```

## 📝 ベストプラクティス

### 1. 変更管理
- バージョン番号で変更履歴を管理
- 変更前後の期待値を記録
- テストネットで必ず事前確認

### 2. リスク管理
- 段階的な変更（大幅な変更を避ける）
- ロールバック計画の準備
- 複数人での確認体制

### 3. ユーザー体験
- 確率変更の事前告知
- 変更理由の説明
- 影響範囲の明確化

## 🎯 まとめ

動的確率テーブルシステムにより、運営は以下が可能になりました：

1. **リアルタイム調整**: コード変更なしでの確率調整
2. **柔軟な運用**: イベントや調整に応じた即座の対応
3. **安全な変更**: 検証機能による設定ミスの防止
4. **追跡可能性**: バージョン管理による変更履歴の保持

このシステムを活用して、より良いゲーム体験とバランス調整を実現してください。