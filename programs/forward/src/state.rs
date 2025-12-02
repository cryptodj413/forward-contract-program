use anchor_lang::prelude::*;

/// Global configuration for the platform
#[account]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub collateral_mint: Pubkey,
    pub curve_params: CurveParams,
    pub bump: u8,
}

impl GlobalConfig {
    pub const LEN: usize = 8 + 32 + 32 + CurveParams::LEN + 1;
}

/// Market configuration for each Polymarket market
#[account]
pub struct MarketConfig {
    pub polymarket_market_id: String, // Max 256 chars
    pub resolution_timestamp: i64,
    pub price_oracle: Pubkey,
    pub resolution_oracle: Pubkey,
    pub risk_limits: RiskLimits,
    pub status: MarketStatus,
    pub pool_state: Pubkey,
    pub collateral_vault: Pubkey,
    pub bump: u8,
}

impl MarketConfig {
    pub const MAX_MARKET_ID_LEN: usize = 256;
    pub const LEN: usize = 8 + 4 + Self::MAX_MARKET_ID_LEN + 8 + 32 + 32 + RiskLimits::LEN + 1 + 32 + 32 + 1;
}

/// Pool state tracking exposure for a market
#[account]
pub struct PoolState {
    pub market: Pubkey,
    pub total_long_exposure: u64,  // Q_long
    pub total_short_exposure: u64, // Q_short
    pub pool_collateral: u64,      // Total pool collateral locked
    pub position_counter: u64,     // Counter for unique position IDs
    pub bump: u8,
}

impl PoolState {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 8 + 1;

    pub fn net_exposure(&self) -> i64 {
        self.total_long_exposure as i64 - self.total_short_exposure as i64
    }
}

/// Individual user position
#[account]
pub struct Position {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub direction: Direction,
    pub size: u64,              // Q
    pub forward_price: u64,     // K (stored as basis points, e.g., 5000 = 0.5)
    pub collateral_locked: u64, // User collateral
    pub premium_paid: i64,      // Can be negative if user received premium
    pub status: PositionStatus,
    pub bump: u8,
}

impl Position {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 8 + 8 + 8 + 1 + 1;
}

/// Curve parameters for pAMM pricing
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CurveParams {
    pub alpha: u64,        // Curve slope parameter (basis points)
    pub beta: u64,         // Base premium multiplier (basis points)
    pub max_exposure: u64, // E_max: maximum allowed absolute exposure
    pub min_price: u64,    // p_min: minimum forward price (basis points, e.g., 500 = 0.05)
    pub max_price: u64,    // p_max: maximum forward price (basis points, e.g., 9500 = 0.95)
}

impl CurveParams {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8;
}

/// Risk limits per market
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RiskLimits {
    pub max_total_exposure: u64,
    pub max_long_share: u64,  // Maximum long exposure as fraction (basis points)
    pub max_short_share: u64, // Maximum short exposure as fraction (basis points)
}

impl RiskLimits {
    pub const LEN: usize = 8 + 8 + 8;
}

/// Market status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum MarketStatus {
    Active,
    TradingClosed,
    Resolved,
}

/// Position direction
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum Direction {
    Long,
    Short,
}

/// Position status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PositionStatus {
    Open,
    Settled,
    Cancelled,
}

/// Market outcome
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Yes, // 1
    No,  // 0
}

impl Outcome {
    pub fn as_u8(&self) -> u8 {
        match self {
            Outcome::Yes => 1,
            Outcome::No => 0,
        }
    }
}

