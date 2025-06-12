use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("User already has a farm")]
    AlreadyHasFarm,
    
    #[msg("User does not have a farm")]
    NoFarm,
    
    #[msg("User already has a farm space")]
    AlreadyHasFarmSpace,
    
    #[msg("User does not have a farm space")]
    NoFarmSpace,
    
    #[msg("No reward to claim")]
    NoRewardToClaim,
    
    #[msg("Calculation overflow")]
    CalculationOverflow,
    
    #[msg("Invalid configuration")]
    InvalidConfig,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Invalid referrer")]
    InvalidReferrer,
    
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("Farm is at maximum capacity")]
    FarmAtMaxCapacity,
    
    #[msg("Farm space is at maximum capacity")]
    FarmSpaceAtMaxCapacity,
    
    #[msg("Already upgrading")]
    AlreadyUpgrading,
    
    #[msg("Maximum level reached")]
    MaxLevelReached,
    
    #[msg("No upgrade in progress")]
    NoUpgradeInProgress,
    
    #[msg("Upgrade still in progress")]
    UpgradeStillInProgress,
    
    #[msg("Seed not found")]
    SeedNotFound,
    
    #[msg("Seed already planted")]
    SeedAlreadyPlanted,
    
    #[msg("Seed not planted")]
    SeedNotPlanted,
    
    #[msg("Invalid quantity")]
    InvalidQuantity,
    
    #[msg("Invalid invite code")]
    InvalidInviteCode,
    
    #[msg("Invite code limit reached")]
    InviteCodeLimitReached,
    
    #[msg("Invalid inviter")]
    InvalidInviter,
    
    #[msg("Farm space capacity exceeded")]
    FarmSpaceCapacityExceeded,
    
    #[msg("Not seed owner")]
    NotSeedOwner,
    
    #[msg("Seed not in this farm space")]
    SeedNotInThisFarmSpace,
    
    #[msg("Seed pack already opened")]
    SeedPackAlreadyOpened,
    
    #[msg("No grow power")]
    NoGrowPower,
    
    #[msg("No global grow power")]
    NoGlobalGrowPower,
    
    #[msg("Seed storage not initialized")]
    SeedStorageNotInitialized,
    
    #[msg("Seed storage is full")]
    StorageFull,
    
    #[msg("Entropy sequence mismatch")]
    EntropySequenceMismatch,
    
    #[msg("Entropy result not ready yet")]
    EntropyNotReady,
    
    #[msg("Invalid entropy result account")]
    InvalidEntropyAccount,
    
    #[msg("Invalid user entropy seed")]
    InvalidUserEntropySeed,
    
    #[msg("Total supply cap exceeded - cannot mint more tokens")]
    SupplyCapExceeded,
    
    #[msg("Too many transfers in batch operation")]
    TooManyTransfers,
    
    #[msg("Invalid entropy value")]
    InvalidEntropyValue,
    
    #[msg("Low entropy quality")]
    LowEntropyQuality,
    
    #[msg("Invalid seed pack quantity")]
    InvalidSeedPackQuantity,
    
    #[msg("Cannot discard planted seed")]
    CannotDiscardPlantedSeed,
    
    #[msg("Invalid referral level - must be 1 or 2")]
    InvalidReferralLevel,
    
    #[msg("Insufficient liquidity in the pool")]
    InsufficientLiquidity,
    
    #[msg("Insufficient fees for operation")]
    InsufficientFees,
    
    #[msg("Invalid amount specified")]
    InvalidAmount,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageToleranceExceeded,
    
    // Hash-based invite system errors
    #[msg("Invite code has already been used")]
    InviteCodeAlreadyUsed,
    
    #[msg("Invite code is inactive")]
    InviteCodeInactive,
    
    #[msg("Seed type limit reached - maximum 100 seeds per type")]
    SeedTypeLimitReached,
    
    #[msg("Hash verification failed for invite code")]
    HashVerificationFailed,
    
    #[msg("Invalid salt for hash generation")]
    InvalidSalt,
}