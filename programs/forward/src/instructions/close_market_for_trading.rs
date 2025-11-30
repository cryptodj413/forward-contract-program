use anchor_lang::prelude::*;
use crate::state::MarketStatus;
use crate::errors::ForwardError;

#[derive(Accounts)]
pub struct CloseMarketForTrading<'info> {
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
}

pub fn handler(ctx: Context<CloseMarketForTrading>) -> Result<()> {
    let market_config = &mut ctx.accounts.market_config;
    
    require!(
        market_config.status == MarketStatus::Active,
        ForwardError::InvalidMarketStatus
    );
    
    market_config.status = MarketStatus::TradingClosed;
    
    msg!("Market closed for trading");
    
    Ok(())
}

