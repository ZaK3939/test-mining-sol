use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::error::*;
use crate::utils::*;

// This file contains only the essential reward functions after refactoring
// Removed redundant functions: claim_reward, distribute_referral_reward, 
// distribute_referral_on_claim, distribute_complete_referral, claim_referral_rewards

// All reward functionality is now consolidated into:
// 1. claim_reward_with_referral_rewards (main claim function)
// 2. accumulate_referral_reward (for referral accumulation)
// 3. view_pending_referral_rewards (for checking pending rewards)

// These functions are implemented in referral_rewards.rs and are the only
// reward-related functions that should be used going forward.

// This file exists temporarily to ensure no functionality is lost during refactoring.
// It should be removed once the transition is complete.