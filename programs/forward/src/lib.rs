#![allow(unexpected_cfgs)] // Suppress warnings from Anchor's internal macros (custom-heap, custom-panic, anchor-debug)

use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod math;
pub mod oracle;
pub mod state;

use instructions::*;

declare_id!("BwMMU9rwkXmLdTL5VLBRwkinri2wW1J6pnwTsVv8PvY9");

#[program]
pub mod forward {
    use super::*;

    /// Initialize the global configuration
    pub fn init_global_config(
        ctx: Context<InitGlobalConfig>,
        curve_params: state::CurveParams,
    ) -> Result<()> {
        instructions::init_global_config::handler(ctx, curve_params)
    }

    /// Create a new market linked to a Polymarket market
    pub fn create_market(
        ctx: Context<CreateMarket>,
        polymarket_market_id: String,
        resolution_timestamp: i64,
        risk_limits: state::RiskLimits,
    ) -> Result<()> {
        instructions::create_market::handler(ctx, polymarket_market_id, resolution_timestamp, risk_limits)
    }

    /// Update curve parameters for a market
    pub fn update_curve_params(
        ctx: Context<UpdateCurveParams>,
        curve_params: state::CurveParams,
    ) -> Result<()> {
        instructions::update_curve_params::handler(ctx, curve_params)
    }

    /// Close market for trading (before resolution)
    pub fn close_market_for_trading(ctx: Context<CloseMarketForTrading>) -> Result<()> {
        instructions::close_market_for_trading::handler(ctx)
    }

    /// Open a position (long or short)
    pub fn open_position(
        ctx: Context<OpenPosition>,
        direction: state::Direction,
        size: u64,
        slippage_tolerance: Option<u64>, // in basis points (10000 = 100%)
    ) -> Result<()> {
        instructions::open_position::handler(ctx, direction, size, slippage_tolerance)
    }

    /// Settle a position after market resolution
    pub fn settle_position(ctx: Context<SettlePosition>) -> Result<()> {
        instructions::settle_position::handler(ctx)
    }

    /// Update market resolution from oracle (keeper function)
    pub fn update_market_resolution(
        ctx: Context<UpdateMarketResolution>,
        outcome: state::Outcome,
    ) -> Result<()> {
        instructions::update_market_resolution::handler(ctx, outcome)
    }

    /// Update price oracle with current Polymarket price (keeper function)
    pub fn update_price_oracle(
        ctx: Context<UpdatePriceOracle>,
        price: u64,
        exponent: i8,
    ) -> Result<()> {
        instructions::update_price_oracle::handler(ctx, price, exponent)
    }
}

