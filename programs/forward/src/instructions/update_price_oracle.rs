use anchor_lang::prelude::*;
use crate::state::MarketConfig;
use crate::errors::ForwardError;
use crate::math::BASIS_POINTS;

#[derive(Accounts)]
pub struct UpdatePriceOracle<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        seeds = [b"global_config"],
        bump = global_config.bump,
        has_one = admin @ ForwardError::Unauthorized
    )]
    pub global_config: Account<'info, crate::state::GlobalConfig>,
    
    #[account(
        seeds = [b"market", market_config.polymarket_market_id.as_bytes()],
        bump = market_config.bump
    )]
    pub market_config: Account<'info, MarketConfig>,
    
    #[account(
        mut,
        seeds = [b"price_oracle", market_config.key().as_ref()],
        bump
    )]
    pub price_oracle: Account<'info, crate::oracle::PriceOracle>,
}

pub fn handler(
    ctx: Context<UpdatePriceOracle>,
    price: u64,
    exponent: i8,
) -> Result<()> {
    // Validate price is within valid range [0, BASIS_POINTS]
    require!(
        price <= BASIS_POINTS,
        ForwardError::InvalidOracleData
    );
    
    // Get current timestamp
    let clock = Clock::get()?;
    
    // Update price oracle
    let price_oracle = &mut ctx.accounts.price_oracle;
    price_oracle.price = price;
    price_oracle.timestamp = clock.unix_timestamp;
    price_oracle.exponent = exponent;
    
    msg!(
        "Price oracle updated: price={} bps, timestamp={}, exponent={}",
        price,
        clock.unix_timestamp,
        exponent
    );
    
    Ok(())
}

