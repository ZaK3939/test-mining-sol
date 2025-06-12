use anchor_lang::prelude::*;

/// Meteora統合用の設定アカウント
/// Config構造体の予備バイトが不足のため、独立したアカウントとして作成

#[account]
pub struct MeteoraConfig {
    /// Meteora DLMM pool address for WEED/SOL
    pub dlmm_pool: Pubkey,
    
    /// Pool's WEED token reserve account
    pub pool_weed_reserve: Pubkey,
    
    /// Pool's SOL reserve account  
    pub pool_sol_reserve: Pubkey,
    
    /// Pool authority for signing transactions
    pub pool_authority: Pubkey,
    
    /// Oracle account for price feeds (if required)
    pub oracle_account: Option<Pubkey>,
    
    /// Auto-conversion enabled flag
    pub auto_conversion_enabled: bool,
    
    /// Minimum WEED amount to trigger conversion
    pub min_conversion_amount: u64,
    
    /// Default slippage tolerance in basis points (100 = 1%)
    pub default_slippage_bps: u16,
    
    /// Maximum slippage tolerance allowed (500 = 5%)
    pub max_slippage_bps: u16,
    
    /// Minimum time interval between conversions (seconds)
    pub min_conversion_interval: i64,
    
    /// Last conversion timestamp
    pub last_conversion_time: i64,
    
    /// Total conversions completed
    pub total_conversions: u64,
    
    /// Total WEED converted to SOL
    pub total_weed_converted: u64,
    
    /// Total SOL received from conversions
    pub total_sol_received: u64,
    
    /// Emergency pause flag (admin can pause conversions)
    pub emergency_pause: bool,
    
    /// Reserve bytes for future expansion
    pub reserve: [u8; 32],
}

impl MeteoraConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // dlmm_pool
        32 + // pool_weed_reserve
        32 + // pool_sol_reserve
        32 + // pool_authority
        1 + 32 + // oracle_account (Option<Pubkey>)
        1 + // auto_conversion_enabled
        8 + // min_conversion_amount
        2 + // default_slippage_bps
        2 + // max_slippage_bps
        8 + // min_conversion_interval
        8 + // last_conversion_time
        8 + // total_conversions
        8 + // total_weed_converted
        8 + // total_sol_received
        1 + // emergency_pause
        32; // reserve
    
    /// デフォルト設定での初期化
    pub fn initialize_default(&mut self, _admin: Pubkey) -> Result<()> {
        self.dlmm_pool = Pubkey::default();
        self.pool_weed_reserve = Pubkey::default();
        self.pool_sol_reserve = Pubkey::default();
        self.pool_authority = Pubkey::default();
        self.oracle_account = None;
        self.auto_conversion_enabled = false; // 手動で有効化する必要がある
        self.min_conversion_amount = 5000 * 1_000_000; // 5000 WEED
        self.default_slippage_bps = 100; // 1%
        self.max_slippage_bps = 500; // 5%
        self.min_conversion_interval = 3600; // 1時間
        self.last_conversion_time = 0;
        self.total_conversions = 0;
        self.total_weed_converted = 0;
        self.total_sol_received = 0;
        self.emergency_pause = false;
        self.reserve = [0; 32];
        
        Ok(())
    }
    
    /// 設定検証
    pub fn validate_settings(&self) -> Result<()> {
        require!(
            self.default_slippage_bps <= self.max_slippage_bps,
            crate::error::GameError::InvalidConfig
        );
        
        require!(
            self.max_slippage_bps <= 1000, // 最大10%
            crate::error::GameError::InvalidConfig
        );
        
        require!(
            self.min_conversion_amount >= 100 * 1_000_000, // 最小100 WEED
            crate::error::GameError::InvalidConfig
        );
        
        require!(
            self.min_conversion_interval >= 600, // 最小10分
            crate::error::GameError::InvalidConfig
        );
        
        Ok(())
    }
    
    /// 変換可能かチェック
    pub fn can_convert(&self, current_time: i64, weed_amount: u64) -> Result<bool> {
        // 緊急停止チェック
        if self.emergency_pause {
            return Ok(false);
        }
        
        // 自動変換有効チェック
        if !self.auto_conversion_enabled {
            return Ok(false);
        }
        
        // 最小金額チェック
        if weed_amount < self.min_conversion_amount {
            return Ok(false);
        }
        
        // 時間間隔チェック
        let time_since_last = current_time - self.last_conversion_time;
        if time_since_last < self.min_conversion_interval {
            return Ok(false);
        }
        
        // プール設定チェック
        if self.dlmm_pool == Pubkey::default() {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// 変換実行後の統計更新
    pub fn update_conversion_stats(
        &mut self, 
        weed_amount: u64, 
        sol_received: u64, 
        current_time: i64
    ) -> Result<()> {
        self.total_conversions = self.total_conversions
            .checked_add(1)
            .ok_or(crate::error::GameError::CalculationOverflow)?;
        
        self.total_weed_converted = self.total_weed_converted
            .checked_add(weed_amount)
            .ok_or(crate::error::GameError::CalculationOverflow)?;
        
        self.total_sol_received = self.total_sol_received
            .checked_add(sol_received)
            .ok_or(crate::error::GameError::CalculationOverflow)?;
        
        self.last_conversion_time = current_time;
        
        Ok(())
    }
}

/// 拡張されたFeePool (統計追跡機能付き)
#[account]
pub struct FeePoolExtended {
    /// 基本FeePool情報
    pub accumulated_fees: u64,
    pub treasury_address: Pubkey,
    pub last_collection_time: i64,
    
    /// 拡張統計情報
    pub total_fees_collected: u64,
    pub total_transfers_processed: u64,
    pub daily_fee_average: u64,
    pub peak_daily_fees: u64,
    pub last_stats_update: i64,
    
    /// Meteora変換統計
    pub pending_conversion_amount: u64,
    pub last_conversion_attempt: i64,
    pub failed_conversion_count: u64,
    
    /// パフォーマンス追跡
    pub average_conversion_rate: u64, // WEED per SOL (scaled by 1e9)
    pub best_conversion_rate: u64,
    pub worst_conversion_rate: u64,
    
    /// 予備バイト
    pub reserve: [u8; 32],
}

impl FeePoolExtended {
    pub const LEN: usize = 8 + // discriminator
        8 + // accumulated_fees
        32 + // treasury_address
        8 + // last_collection_time
        8 + // total_fees_collected
        8 + // total_transfers_processed
        8 + // daily_fee_average
        8 + // peak_daily_fees
        8 + // last_stats_update
        8 + // pending_conversion_amount
        8 + // last_conversion_attempt
        8 + // failed_conversion_count
        8 + // average_conversion_rate
        8 + // best_conversion_rate
        8 + // worst_conversion_rate
        32; // reserve
    
    /// 基本FeePoolから拡張版への移行
    pub fn migrate_from_basic(&mut self, basic_fee_pool: &crate::state::FeePool) -> Result<()> {
        self.accumulated_fees = basic_fee_pool.accumulated_fees;
        self.treasury_address = basic_fee_pool.treasury_address;
        self.last_collection_time = basic_fee_pool.last_collection_time;
        
        // 拡張フィールドの初期化
        self.total_fees_collected = basic_fee_pool.accumulated_fees;
        self.total_transfers_processed = 0;
        self.daily_fee_average = 0;
        self.peak_daily_fees = 0;
        self.last_stats_update = Clock::get()?.unix_timestamp;
        self.pending_conversion_amount = 0;
        self.last_conversion_attempt = 0;
        self.failed_conversion_count = 0;
        self.average_conversion_rate = 0;
        self.best_conversion_rate = 0;
        self.worst_conversion_rate = u64::MAX;
        self.reserve = [0; 32];
        
        Ok(())
    }
    
    /// 手数料統計更新
    pub fn update_fee_stats(&mut self, fee_amount: u64, current_time: i64) -> Result<()> {
        self.accumulated_fees = self.accumulated_fees
            .checked_add(fee_amount)
            .ok_or(crate::error::GameError::CalculationOverflow)?;
        
        self.total_fees_collected = self.total_fees_collected
            .checked_add(fee_amount)
            .ok_or(crate::error::GameError::CalculationOverflow)?;
        
        self.total_transfers_processed = self.total_transfers_processed
            .checked_add(1)
            .ok_or(crate::error::GameError::CalculationOverflow)?;
        
        self.last_collection_time = current_time;
        
        // 日次統計更新 (24時間ベース)
        self.update_daily_stats(fee_amount, current_time)?;
        
        Ok(())
    }
    
    /// 日次統計更新
    fn update_daily_stats(&mut self, fee_amount: u64, current_time: i64) -> Result<()> {
        let day_seconds = 24 * 60 * 60;
        let days_since_update = (current_time - self.last_stats_update) / day_seconds;
        
        if days_since_update >= 1 {
            // 新しい日の統計リセット
            self.daily_fee_average = fee_amount;
            self.last_stats_update = current_time;
        } else {
            // 当日の平均更新 (移動平均)
            self.daily_fee_average = (self.daily_fee_average + fee_amount) / 2;
        }
        
        // ピーク値更新
        if fee_amount > self.peak_daily_fees {
            self.peak_daily_fees = fee_amount;
        }
        
        Ok(())
    }
    
    /// 変換レート統計更新
    pub fn update_conversion_rate_stats(&mut self, weed_amount: u64, sol_received: u64) -> Result<()> {
        // レート計算 (WEED per SOL, scaled by 1e9)
        let conversion_rate = if sol_received > 0 {
            weed_amount.checked_mul(1_000_000_000)
                .ok_or(crate::error::GameError::CalculationOverflow)?
                .checked_div(sol_received)
                .unwrap_or(0)
        } else {
            0
        };
        
        // 平均レート更新
        if self.average_conversion_rate == 0 {
            self.average_conversion_rate = conversion_rate;
        } else {
            self.average_conversion_rate = (self.average_conversion_rate + conversion_rate) / 2;
        }
        
        // 最高・最低レート更新
        if conversion_rate > self.best_conversion_rate {
            self.best_conversion_rate = conversion_rate;
        }
        
        if conversion_rate < self.worst_conversion_rate && conversion_rate > 0 {
            self.worst_conversion_rate = conversion_rate;
        }
        
        Ok(())
    }
}