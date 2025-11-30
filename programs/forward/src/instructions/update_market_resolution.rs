use anchor_lang::prelude::*;
use crate::state::{MarketStatus, Outcome};
use crate::errors::ForwardError;

#[derive(Accounts)]
pub struct UpdateMarketResolution<'info> {
    /// CHECK: Keeper or admin (can be same as admin or separate)
    pub keeper: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market", market_config.polymarket_market_id.as_bytes()],
        bump = market_config.bump
    )]
    pub market_config: Account<'info, crate::state::MarketConfig>,
    
    /// CHECK: Resolution oracle account
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
    
    // In production, you'd verify the outcome matches the oracle
    // For now, we accept the keeper's input but could add validation
    
    market_config.status = MarketStatus::Resolved;
    
    msg!("Market resolved with outcome: {:?}", outcome);
    
    Ok(())
}

