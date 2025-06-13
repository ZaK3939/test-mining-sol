// ===== モジュールの概要 =====
// このファイルは各命令（instruction）で必要となるアカウントの集合を定義します。
// Solanaでは、プログラムが使用するすべてのアカウントを事前に宣言する必要があり、
// それを「コンテキスト」として管理します。

// 機能別にモジュールを分割
pub mod admin;
pub mod user;
pub mod farm; // Farm operations + level management
pub mod referral; // 紹介料蓄積・請求システム
pub mod seeds;
pub mod invite; // Hash-based invite system

// すべての構造体と関数を再エクスポート
pub use admin::*;
pub use user::*;
pub use farm::*;
pub use referral::*;
pub use seeds::*;
pub use invite::*;

