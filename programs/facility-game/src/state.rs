use anchor_lang::prelude::*;

/// グローバル設定アカウント
#[account]
pub struct Config {
    /// 基本報酬レート
    pub base_rate: u64,
    /// 半減期間隔（秒）
    pub halving_interval: i64,
    /// 次回半減期の時刻
    pub next_halving_time: i64,
    /// 管理者アドレス
    pub admin: Pubkey,
    /// 将来の拡張用
    pub reserve: [u8; 64],
}

/// ユーザー状態アカウント
#[account]
pub struct UserState {
    /// ユーザーの公開鍵
    pub owner: Pubkey,
    /// 総Grow Power
    pub total_grow_power: u64,
    /// 最後の収穫時刻
    pub last_harvest_time: i64,
    /// 施設を持っているかどうか
    pub has_facility: bool,
    /// 将来の拡張用（紹介機能など）
    pub reserve: [u8; 64],
}

/// 施設アカウント
#[account]
pub struct Facility {
    /// 施設の所有者
    pub owner: Pubkey,
    /// マシン数
    pub machine_count: u32,
    /// 総Grow Power
    pub total_grow_power: u64,
    /// 将来の拡張用（複数マシンタイプなど）
    pub reserve: [u8; 64],
}

impl Config {
    pub const LEN: usize = 8 + // discriminator
        8 + // base_rate
        8 + // halving_interval
        8 + // next_halving_time
        32 + // admin
        64; // reserve
}

impl UserState {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        8 + // total_grow_power
        8 + // last_harvest_time
        1 + // has_facility
        64; // reserve
}

impl Facility {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        4 + // machine_count
        8 + // total_grow_power
        64; // reserve
}