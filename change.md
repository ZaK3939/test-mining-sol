### Seeds の購入

- Mystery Seed Pack:
  - Silk Road Marketplace から Mystery Seed Pack を購入することで新しい Seeds を取得可能。取得する Seed タイプはランダム。レアリティーが高ければ高いほどその Seed の Grow Power も高い。
  - 費用：300 $WEED (理想はバイラルする前で$10 くらい)
  - 最大購入数は一回につき 100 個
  - 取得した Seeds は全てストレージに送られる
- Seeds の種類：
  (Seed の名前の変更、Grow Power or 確率の変更は今後入ります）
  (Seed の Sell は不可） - Seed 1 - Grow Power：100 - 確率：42.23% - Seed ２ - Grow Power：180 - 確率：24.44% - Seed ３ - Grow Power：420 - 確率：13.33% - Seed ４ - Grow Power：720 - 確率：8.33% - Seed ５ - Grow Power：1000 - 確率：5.56% - Seed ６ - Grow Power：5000 - 確率：3.33% - Seed ７ - Grow Power：15000 - 確率：1.33% - Seed ８ - Grow Power：30000 - 確率：0.89% - Seed 9 - Grow Power：60000 - 確率：0.56%

### Seeds の配置

- **配置制限**: 各 Seed は 1 マス消費し、施設の容量内でのみ配置可能

### Farm Space のアップグレード

- Farm Space の種類：
  - Level 1:
    - 費用：0.5 SOL
    - Capacity： 4 Seeds
  - Level 2:
    - 費用：3500 $WEED
    - Capacity：８ Seeds
  - Level 3:
    - 費用：18000 $WEED
    - Capacity：12 Seeds
  - Level 4:
    - 費用：20000 $WEED
    - Capacity：16 Seeds
  - Level 5:
    - 費用：25000 $WEED
    - Capacity：20 Seeds
- **アップグレード条件**:
  - 支払い後 24 時間のクールダウンタイムを経て Farm Space がアップグレード
  - アップグレード進行中も旧容量で稼働
- **効果**: 施設のマス容量が上昇、より多くの Seeds を配置可能

初期設定（アカウント作成と Farm 購入）
Invite Code 管理

PDA で招待コードを管理（Invite PDA を作成）

招待コードの検証と使用上限を設定

招待コードエラーのハンドリング（Invalid invite code / Invite code limit reached）

Farm Space の購入

PDA（Farm Space PDA）を作成

0.5 SOL 支払い後に容量=4 の Farm Space (level 1)を初期化

購入時、初回 Seed（Seed 1）を自動的にプレイヤーに付与（Seed PDA として管理）

② Seeds 購入と配置システム
Mystery Seed Pack

Mystery Seed Pack の購入機能を追加（費用：300 $WEED）

購入時に指定個数（最大 100）をランダム抽選（確率テーブル参照）

抽選された Seeds をユーザーごとの Storage PDA に格納

各 Seed は PDA アカウントで管理（Seed ごとの Grow Power 属性）

Seeds 配置

Farm Space 容量チェックを実施（容量を超えるとエラー）

Seed 配置時に Farm PDA に紐づけて Grow Power を反映

③ Farm Space アップグレードシステム
Farm Space レベルアップの実装

レベルごとのアップグレードコストを実装：

Level 2：3500 $WEED

Level 3：18000 $WEED

Level 4：20000 $WEED

Level 5：25000 $WEED

コスト支払い後、24 時間後に容量が増える（クールダウン期間を記録する）

クールダウン中も旧容量で Farm 稼働継続可能とする。

④ 収穫システム
収穫量計算

全プレイヤー総 Grow Power を記録する Global PDA を管理

各プレイヤーの収穫量は以下の式で随時計算：

rust
Copy
user_reward = (user_total_grow_power / global_total_grow_power) × 100 WEED per sec
収穫報酬の分配

PDA 経由でプレイヤー報酬を一時保管（Claim 可能）

Invite 報酬を自動分配（Claim 時に第 1 階層 10%、第 2 階層 5%を送金）

⑤ Invite reward システム
Invite code の仕組み

初期招待数 5 人、その後拡張可能（設定を Config PDA に保持）

招待コードの使用状況を Invite PDA で管理し、上限到達チェックを実施

Referral 報酬受け取り

Referral 報酬 PDA に各招待ユーザーごとの報酬を記録

プレイヤーは任意タイミングで Claim 命令で報酬受取可能

⑥ トークノミクス（Tokenomics）
$WEED トークン供給量管理

初期供給量 1 Billion WEED、Supply 固定

1 秒ごとの報酬量を Global PDA で記録管理（100 WEED/秒）

Halving 機構

5〜7 日毎に報酬半減ロジックを Config PDA に実装

時間経過をブロックタイムから検知し、半減処理を自動的に適用

Burn 機構

Mystery Seed Pack 購入時の支払額（$WEED）を 100% Burn（Mint から Burn 命令を実行）

⑦ トレーディングフィーと SOL 変換・送金
2%手数料徴収

全ての取引（buy/sell）に 2%手数料を徴収

徴収した$WEED を PDA（FeePool）に一時保管

SOL への変換と送金

Jupiter Aggregator を利用した CPI 経由の$WEED→SOL 変換を実装

変換後の SOL を指定のマルチシグ or 単一アドレスに送金

📐 実装アーキテクチャ（PDA 設計）
以下のような PDA 設計を推奨します：

PDA 役割
Config PDA ゲーム全体の設定を保持（Invite 数上限・アップグレードコスト・Halving 周期）
Invite PDA 招待コード管理（使用者数カウント、上限管理）
Farm Space PDA 各ユーザーの Farm Space データ（容量、レベル、配置 Seeds）
Seed PDA 各 Seed のデータ（所有者、Grow Power、配置状況）
FeePool PDA 徴収した手数料の$WEED 残高を管理（SOL 変換前の一時保管）
Reward PDA ユーザー報酬の累積管理（Claim 可能額を保持）
Global PDA 全ユーザーの Grow Power 総量を管理し、報酬計算に利用
