use anchor_lang::prelude::*;
use crate::state::{MarketStatus, Outcome};
use crate::errors::ForwardError;

#[derive(Accounts)]
pub struct UpdateMarketResolution<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        seeds = [b"global_config"],
        bump = global_config.bump,
        has_one = admin @ ForwardError::Unauthorized
    )]
    pub global_config: Account<'info, crate::state::GlobalConfig>,
    
    #[account(
        mut,
        seeds = [b"market", market_config.polymarket_market_id.as_bytes()],
        bump = market_config.bump
    )]
    pub market_config: Account<'info, crate::state::MarketConfig>,
    
    #[account(
        mut,
        seeds = [b"resolution_oracle", market_config.key().as_ref()],
        bump
    )]
    pub resolution_oracle: Account<'info, crate::oracle::ResolutionOracle>,
}

pub fn handler(
    ctx: Context<UpdateMarketResolution>,
    outcome: Outcome,
) -> Result<()> {
    let market_config = &mut ctx.accounts.market_config;
    
    require!(
        market_config.status == MarketStatus::Active || market_config.status == MarketStatus::TradingClosed,
        ForwardError::InvalidMarketStatus
    );
    
    // Persist outcome in the per‑market resolution oracle PDA
    ctx.accounts.resolution_oracle.outcome = Some(outcome.as_u8());
    ctx.accounts.resolution_oracle.resolved_at = Some(Clock::get()?.unix_timestamp);

    market_config.status = MarketStatus::Resolved;
    
    msg!("Market resolved with outcome: {:?}", outcome);
    
    Ok(())
}

