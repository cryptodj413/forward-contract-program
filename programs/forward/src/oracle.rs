use anchor_lang::prelude::*;
use crate::errors::ForwardError;
use crate::math::BASIS_POINTS;

/// Oracle price feed account structure
/// This is a simplified structure - in production, you'd integrate with
/// Pyth, Switchboard, or a custom Polymarket oracle
#[account]
pub struct PriceOracle {
    pub price: u64,        // Price in basis points (e.g., 5000 = 0.5)
    pub timestamp: i64,
    pub exponent: i8,      // Price exponent for precision
}

impl PriceOracle {
    pub const LEN: usize = 8 + 8 + 8 + 1;
}

/// Oracle resolution feed account structure
#[account]
pub struct ResolutionOracle {
    pub outcome: Option<u8>, // 1 = YES, 0 = NO, None = not resolved
    pub resolved_at: Option<i64>,
}

impl ResolutionOracle {
    pub const LEN: usize = 8 + 1 + 1 + 8 + 1;
}

/// Read price from oracle account
pub fn read_price(oracle_account: &Account<PriceOracle>) -> Result<u64> {
    // Validate price is within valid range [0, BASIS_POINTS]
    require!(
        oracle_account.price <= BASIS_POINTS,
        ForwardError::InvalidOracleData
    );
    
    // Validate timestamp freshness (price must be < 5 minutes old and not in future)
    let clock = Clock::get()?;
    let max_age: i64 = 300; // 5 minutes in seconds
    
    // Reject if timestamp is in the future
    require!(
        oracle_account.timestamp <= clock.unix_timestamp,
        ForwardError::InvalidOracleData
    );
    
    // Reject if timestamp is too old
    require!(
        clock.unix_timestamp.saturating_sub(oracle_account.timestamp) <= max_age,
        ForwardError::InvalidOracleData
    );
    
    Ok(oracle_account.price)
}

/// Read resolution from oracle account
pub fn read_resolution(oracle_account: &Account<ResolutionOracle>) -> Result<Option<crate::state::Outcome>> {
    match oracle_account.outcome {
        Some(1) => Ok(Some(crate::state::Outcome::Yes)),
        Some(0) => Ok(Some(crate::state::Outcome::No)),
        Some(_) => Err(ForwardError::InvalidOracleData.into()),
        None => Ok(None),
    }
}

