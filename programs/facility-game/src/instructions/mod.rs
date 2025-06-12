// ===== モジュールの概要 =====
// このファイルは各命令（instruction）で必要となるアカウントの集合を定義します。
// Solanaでは、プログラムが使用するすべてのアカウントを事前に宣言する必要があり、
// それを「コンテキスト」として管理します。

// 機能別にモジュールを分割
pub mod admin;
pub mod user;
pub mod farm;
// pub mod rewards; // Deprecated - functionality moved to referral_rewards
pub mod referral_rewards; // 紹介料蓄積・請求システム
// pub mod transfer; // Deprecated - use transfer_improved instead
pub mod transfer_improved; // 改善されたTransferシステム
// pub mod mystery; // Deprecated - functionality moved to seeds.rs
pub mod seeds;
pub mod invite; // Hash-based invite system
pub mod meteora;
// pub mod meteora_minimal; // Minimal Meteora integration - Disabled
// pub mod meteora_admin; // Meteora管理者制御 - Temporarily disabled

// すべての構造体と関数を再エクスポート
pub use admin::*;
pub use user::*;
pub use farm::*;
// pub use rewards::*; // Deprecated - functionality moved to referral_rewards
pub use referral_rewards::*;
// pub use transfer::*; // Deprecated
pub use transfer_improved::*;
// pub use mystery::*; // Deprecated
pub use seeds::*;
pub use invite::*;
pub use meteora::*;
// pub use meteora_minimal::{SwapWeedToSolViaDlmm, swap_weed_to_sol_via_dlmm, check_auto_conversion_trigger};
// pub use meteora_admin::{
//     InitializeMeteoraConfig, 
//     AdminConfigureDlmmPool,
//     configure_dlmm_pool,
//     UpdateConversionSettings,
//     update_conversion_settings,
//     initialize_meteora_config,
//     DlmmPoolConfig,
//     ConversionSettings,
//     EmergencyControl,
//     MonitorPoolHealth,
//     ViewMeteoraStats,
//     emergency_pause_toggle,
//     monitor_pool_health,
//     view_meteora_stats
// };

