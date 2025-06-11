// ===== モジュールの概要 =====
// このファイルは各命令（instruction）で必要となるアカウントの集合を定義します。
// Solanaでは、プログラムが使用するすべてのアカウントを事前に宣言する必要があり、
// それを「コンテキスト」として管理します。

// 機能別にモジュールを分割
pub mod admin;
pub mod user;
pub mod farm;
pub mod rewards;
pub mod transfer;
// pub mod mystery; // Deprecated - functionality moved to seeds.rs
pub mod seeds;
pub mod invite;
pub mod meteora;

// すべての構造体と関数を再エクスポート
pub use admin::*;
pub use user::*;
pub use farm::*;
pub use rewards::*;
pub use transfer::*;
// pub use mystery::*; // Deprecated
pub use seeds::*;
pub use invite::*;
pub use meteora::*;

