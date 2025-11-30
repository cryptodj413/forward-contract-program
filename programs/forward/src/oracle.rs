use anchor_lang::prelude::*;
use crate::errors::ForwardError;

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
    // In production, add validation for timestamp freshness
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

