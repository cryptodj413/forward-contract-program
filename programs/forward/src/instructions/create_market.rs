use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{MarketConfig, PoolState, RiskLimits, MarketStatus};
use crate::oracle::{PriceOracle, ResolutionOracle};

#[derive(Accounts)]
#[instruction(polymarket_market_id: String, resolution_timestamp: i64)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        seeds = [b"global_config"],
        bump = global_config.bump,
        has_one = admin @ crate::errors::ForwardError::Unauthorized
    )]
    pub global_config: Account<'info, crate::state::GlobalConfig>,
    
    #[account(
        init,
        payer = admin,
        space = MarketConfig::LEN,
        seeds = [b"market", polymarket_market_id.as_bytes()],
        bump
    )]
    pub market_config: Account<'info, MarketConfig>,
    
    #[account(
        init,
        payer = admin,
        space = PoolState::LEN,
        seeds = [b"pool_state", market_config.key().as_ref()],
        bump
    )]
    pub pool_state: Account<'info, PoolState>,
    
    #[account(
        init,
        payer = admin,
        space = PriceOracle::LEN,
        seeds = [b"price_oracle", market_config.key().as_ref()],
        bump
    )]
    pub price_oracle: Account<'info, PriceOracle>,
    
    #[account(
        init,
        payer = admin,
        space = ResolutionOracle::LEN,
        seeds = [b"resolution_oracle", market_config.key().as_ref()],
        bump
    )]
    pub resolution_oracle: Account<'info, ResolutionOracle>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = admin,
        token::mint = mint,
        token::authority = collateral_vault,
        seeds = [b"collateral_vault", market_config.key().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateMarket>,
    polymarket_market_id: String,
    resolution_timestamp: i64,
    risk_limits: RiskLimits,
) -> Result<()> {
    require!(
        polymarket_market_id.len() <= MarketConfig::MAX_MARKET_ID_LEN,
        crate::errors::ForwardError::InvalidOracleData
    );
    
    require!(
        ctx.accounts.mint.key() == ctx.accounts.global_config.collateral_mint,
        crate::errors::ForwardError::InvalidMint
    );
    
    // Validate resolution timestamp is in the future but not too far
    let clock = Clock::get()?;
    require!(
        resolution_timestamp > clock.unix_timestamp,
        crate::errors::ForwardError::InvalidOracleData
    );
    // Max 10 years in the future
    require!(
        resolution_timestamp <= clock.unix_timestamp.saturating_add(315360000),
        crate::errors::ForwardError::InvalidOracleData
    );

    // Validate per‑market risk limits against ARCHITECTURE.md:
    // max_total_exposure > 0 and shares are fractions in basis points (0..=BASIS_POINTS).
    require!(
        risk_limits.max_total_exposure > 0,
        crate::errors::ForwardError::InvalidOracleData
    );
    require!(
        risk_limits.max_long_share <= crate::math::BASIS_POINTS,
        crate::errors::ForwardError::InvalidOracleData
    );
    require!(
        risk_limits.max_short_share <= crate::math::BASIS_POINTS,
        crate::errors::ForwardError::InvalidOracleData
    );
    
    let market_config = &mut ctx.accounts.market_config;
    let pool_state = &mut ctx.accounts.pool_state;
    
    market_config.polymarket_market_id = polymarket_market_id.clone();
    market_config.resolution_timestamp = resolution_timestamp;
    // Store the PDAs of the per‑market oracle accounts
    market_config.price_oracle = ctx.accounts.price_oracle.key();
    market_config.resolution_oracle = ctx.accounts.resolution_oracle.key();
    market_config.risk_limits = risk_limits;
    market_config.status = MarketStatus::Active;
    market_config.pool_state = pool_state.key();
    market_config.collateral_vault = ctx.accounts.collateral_vault.key();
    market_config.bump = ctx.bumps.market_config;
    
    pool_state.market = market_config.key();
    pool_state.total_long_exposure = 0;
    pool_state.total_short_exposure = 0;
    pool_state.pool_collateral = 0;
    pool_state.position_counter = 0;
    pool_state.bump = ctx.bumps.pool_state;
    
    msg!(
        "Market created: {} with resolution at {}",
        polymarket_market_id,
        resolution_timestamp
    );
    
    Ok(())
}

