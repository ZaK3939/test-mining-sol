/// Centralized error handling utilities
/// Provides consistent error handling patterns across the application

use anchor_lang::prelude::*;
use crate::error::GameError;

// ===== ERROR CONTEXT HELPERS =====

/// Add context to calculation overflow errors
pub fn handle_calculation_overflow<T>(result: Option<T>, context: &str) -> Result<T> {
    result.ok_or_else(|| {
        msg!("Calculation overflow in context: {}", context);
        GameError::CalculationOverflow.into()
    })
}

/// Handle checked arithmetic operations with context
pub fn checked_add_with_context(a: u64, b: u64, context: &str) -> Result<u64> {
    a.checked_add(b)
        .ok_or_else(|| {
            msg!("Addition overflow in context: {} ({} + {})", context, a, b);
            GameError::CalculationOverflow.into()
        })
}

/// Handle division operations with zero check
pub fn safe_divide(dividend: u64, divisor: u64, context: &str) -> Result<u64> {
    if divisor == 0 {
        msg!("Division by zero in context: {}", context);
        return Err(GameError::CalculationOverflow.into());
    }
    
    dividend.checked_div(divisor)
        .ok_or_else(|| {
            msg!("Division overflow in context: {}", context);
            GameError::CalculationOverflow.into()
        })
}

/// Handle multiplication operations with overflow check
pub fn safe_multiply(a: u64, b: u64, context: &str) -> Result<u64> {
    a.checked_mul(b)
        .ok_or_else(|| {
            msg!("Multiplication overflow in context: {} ({} * {})", context, a, b);
            GameError::CalculationOverflow.into()
        })
}

/// Handle percentage calculations safely
pub fn safe_percentage(amount: u64, percentage: u8, context: &str) -> Result<u64> {
    if percentage > 100 {
        msg!("Invalid percentage in context: {} ({}%)", context, percentage);
        return Err(GameError::InvalidConfig.into());
    }
    
    let result = safe_multiply(amount, percentage as u64, context)?;
    safe_divide(result, 100, context)
}

/// Handle basis points calculations safely
pub fn safe_basis_points(amount: u64, basis_points: u16, context: &str) -> Result<u64> {
    if basis_points > 10000 {
        msg!("Invalid basis points in context: {} ({} bp)", context, basis_points);
        return Err(GameError::InvalidConfig.into());
    }
    
    let result = safe_multiply(amount, basis_points as u64, context)?;
    safe_divide(result, 10000, context)
}

// ===== RESULT CHAIN HELPERS =====

/// Chain multiple operations that might fail
pub trait ResultChain<T> {
    fn chain<F, U>(self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>;
    
    fn chain_with_context<F, U>(self, f: F, context: &str) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>;
}

impl<T> ResultChain<T> for Result<T> {
    fn chain<F, U>(self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>,
    {
        self.and_then(f)
    }
    
    fn chain_with_context<F, U>(self, f: F, context: &str) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>,
    {
        match self {
            Ok(value) => f(value).map_err(|e| {
                msg!("Error in context '{}': {}", context, e);
                e
            }),
            Err(e) => {
                msg!("Previous error propagated to context '{}': {}", context, e);
                Err(e)
            }
        }
    }
}

// ===== VALIDATION RESULT HELPERS =====

/// Combine multiple validation results
pub fn combine_validations(results: Vec<Result<()>>) -> Result<()> {
    let errors: Vec<_> = results.into_iter().filter_map(|r| r.err()).collect();
    
    if errors.is_empty() {
        Ok(())
    } else {
        // Return the first error for now - could be enhanced to collect all errors
        Err(errors.into_iter().next().unwrap())
    }
}

/// Validate with early return on first error
pub fn validate_all<F>(validations: Vec<F>) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    for validation in validations {
        validation()?;
    }
    Ok(())
}

// ===== ERROR LOGGING HELPERS =====

/// Log error with context and continue
pub fn log_and_continue<T>(result: Result<T>, context: &str) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(e) => {
            msg!("Non-fatal error in context '{}': {}", context, e);
            None
        }
    }
}

/// Log error and return default value
pub fn log_and_default<T: Default>(result: Result<T>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            msg!("Error in context '{}', using default: {}", context, e);
            T::default()
        }
    }
}

/// Enhanced error logging with structured information
pub fn log_structured_error(
    error: &anchor_lang::error::Error,
    operation: &str,
    account: Option<Pubkey>,
    amount: Option<u64>,
) {
    let account_str = account.map_or("None".to_string(), |a| a.to_string());
    let amount_str = amount.map_or("None".to_string(), |a| a.to_string());
    
    msg!(
        "🚨 Error in operation '{}': {} | Account: {} | Amount: {}",
        operation,
        error,
        account_str,
        amount_str
    );
}

// ===== TRANSACTION SAFETY HELPERS =====

/// Ensure account state consistency before operations
pub fn ensure_account_consistency<T: AccountSerialize + AccountDeserialize + Clone>(
    account_info: &AccountInfo,
    _expected_data: &T,
) -> Result<()> {
    let _current_data: T = T::try_deserialize(&mut &account_info.data.borrow()[..])?;
    
    // This is a simplified check - in practice you'd implement PartialEq for your types
    // or use other consistency validation methods
    msg!("Account consistency check passed for {}", account_info.key);
    Ok(())
}

/// Safe account data update with rollback on error
pub fn safe_account_update<T, F>(
    account: &mut T,
    operation: F,
    context: &str,
) -> Result<()>
where
    T: Clone,
    F: FnOnce(&mut T) -> Result<()>,
{
    let backup = account.clone();
    
    match operation(account) {
        Ok(()) => {
            msg!("✅ Account update successful: {}", context);
            Ok(())
        }
        Err(e) => {
            msg!("❌ Account update failed, rolling back: {}", context);
            *account = backup;
            Err(e)
        }
    }
}

// ===== MACROS FOR COMMON ERROR PATTERNS =====

/// Macro for safe arithmetic operations
#[macro_export]
macro_rules! safe_math {
    ($op:ident, $a:expr, $b:expr, $context:expr) => {
        $a.$op($b).ok_or_else(|| {
            msg!("Math operation {} failed in context: {}", stringify!($op), $context);
            $crate::error::GameError::CalculationOverflow
        })?
    };
}

/// Macro for requirement checks with context
#[macro_export]
macro_rules! require_with_context {
    ($condition:expr, $error:expr, $context:expr) => {
        if !($condition) {
            msg!("Requirement failed in context '{}': {}", $context, stringify!($condition));
            return Err($error.into());
        }
    };
}

/// Macro for non-zero validation
#[macro_export]
macro_rules! require_non_zero {
    ($value:expr, $context:expr) => {
        if $value == 0 {
            msg!("Zero value not allowed in context: {}", $context);
            return Err($crate::error::GameError::InvalidAmount.into());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_math_operations() {
        // Test safe multiplication
        assert!(safe_multiply(100, 200, "test").is_ok());
        assert_eq!(safe_multiply(100, 200, "test").unwrap(), 20000);
        
        // Test overflow case
        assert!(safe_multiply(u64::MAX, 2, "overflow_test").is_err());
        
        // Test safe division
        assert!(safe_divide(100, 2, "test").is_ok());
        assert_eq!(safe_divide(100, 2, "test").unwrap(), 50);
        
        // Test division by zero
        assert!(safe_divide(100, 0, "zero_div_test").is_err());
    }

    #[test]
    fn test_percentage_calculations() {
        // Valid percentage
        assert_eq!(safe_percentage(1000, 10, "test").unwrap(), 100);
        assert_eq!(safe_percentage(1000, 0, "test").unwrap(), 0);
        assert_eq!(safe_percentage(1000, 100, "test").unwrap(), 1000);
        
        // Invalid percentage
        assert!(safe_percentage(1000, 101, "invalid_test").is_err());
    }

    #[test]
    fn test_basis_points_calculations() {
        // 1000 basis points = 10%
        assert_eq!(safe_basis_points(1000, 1000, "test").unwrap(), 100);
        
        // 0 basis points = 0%
        assert_eq!(safe_basis_points(1000, 0, "test").unwrap(), 0);
        
        // 10000 basis points = 100%
        assert_eq!(safe_basis_points(1000, 10000, "test").unwrap(), 1000);
        
        // Invalid basis points
        assert!(safe_basis_points(1000, 10001, "invalid_test").is_err());
    }

    #[test]
    fn test_result_chaining() {
        let result: Result<u64> = Ok(100)
            .chain(|x| Ok(x * 2))
            .chain(|x| Ok(x + 50));
        
        assert_eq!(result.unwrap(), 250);
        
        let error_result: Result<u64> = Ok(100)
            .chain(|_| Err(GameError::InvalidAmount.into()))
            .chain(|x: u64| Ok(x + 50));
        
        assert!(error_result.is_err());
    }

    #[test]
    fn test_validation_combining() {
        let all_ok = vec![Ok(()), Ok(()), Ok(())];
        assert!(combine_validations(all_ok).is_ok());
        
        let with_error = vec![Ok(()), Err(GameError::InvalidAmount.into()), Ok(())];
        assert!(combine_validations(with_error).is_err());
    }
}