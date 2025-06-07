use anchor_lang::prelude::*;

/// Global configuration account
#[account]
pub struct Config {
    /// Base reward rate
    pub base_rate: u64,
    /// Halving interval (seconds)
    pub halving_interval: i64,
    /// Next halving timestamp
    pub next_halving_time: i64,
    /// Admin address
    pub admin: Pubkey,
    /// Reserved for future expansion
    pub reserve: [u8; 64],
}

/// User state account
#[account]
pub struct UserState {
    /// User's public key
    pub owner: Pubkey,
    /// Total Grow Power
    pub total_grow_power: u64,
    /// Last harvest timestamp
    pub last_harvest_time: i64,
    /// Whether the user has a facility
    pub has_facility: bool,
    /// Reserved for future expansion (referral system, etc.)
    pub reserve: [u8; 64],
}

/// Facility account
#[account]
pub struct Facility {
    /// Facility owner
    pub owner: Pubkey,
    /// Machine count
    pub machine_count: u32,
    /// Total Grow Power
    pub total_grow_power: u64,
    /// Reserved for future expansion (multiple machine types, etc.)
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