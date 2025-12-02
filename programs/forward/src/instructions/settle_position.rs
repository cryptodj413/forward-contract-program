use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{MarketStatus, PositionStatus};
use crate::math;
use crate::errors::ForwardError;

#[derive(Accounts)]
pub struct SettlePosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"market", market_config.polymarket_market_id.as_bytes()],
        bump = market_config.bump
    )]
    pub market_config: Account<'info, crate::state::MarketConfig>,
    
    #[account(
        mut,
        seeds = [b"pool_state", market_config.key().as_ref()],
        bump = pool_state.bump
    )]
    pub pool_state: Account<'info, crate::state::PoolState>,
    
    #[account(
        mut,
        seeds = [b"resolution_oracle", market_config.key().as_ref()],
        bump
    )]
    pub resolution_oracle: Account<'info, crate::oracle::ResolutionOracle>,
    
    #[account(
        mut,
        constraint = position.owner == user.key(),
        constraint = position.market == market_config.key(),
        constraint = position.status == PositionStatus::Open @ ForwardError::PositionAlreadySettled
    )]
    pub position: Account<'info, crate::state::Position>,
    
    #[account(
        mut,
        seeds = [b"collateral_vault", market_config.key().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key(),
        constraint = user_collateral_account.mint == collateral_vault.mint @ ForwardError::InvalidMint
    )]
    pub user_collateral_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<SettlePosition>) -> Result<()> {
    require!(
        ctx.accounts.market_config.status == MarketStatus::Resolved,
        ForwardError::MarketNotResolved
    );
    
    // Read outcome from oracle
    let outcome_opt = crate::oracle::read_resolution(&ctx.accounts.resolution_oracle)?;
    let outcome = outcome_opt.ok_or(ForwardError::InvalidOracleData)?;
    
    // Calculate payout
    let payout = math::calculate_settlement_payout(
        ctx.accounts.position.size,
        ctx.accounts.position.direction,
        outcome,
    );
    
    // Calculate total collateral locked for this position
    // User collateral + pool collateral = size (Q)
    let total_collateral = ctx.accounts.position.size;
    
    // Calculate pool collateral for this position
    let pool_collateral = total_collateral
        .checked_sub(ctx.accounts.position.collateral_locked)
        .ok_or(ForwardError::MathOverflow)?;
    
    // Transfer payout to user
    if payout > 0 {
        require!(
            ctx.accounts.collateral_vault.amount >= payout,
            ForwardError::InsufficientCollateral
        );
        
        let market_config_key = ctx.accounts.market_config.key();
        let seeds = &[
            b"collateral_vault",
            market_config_key.as_ref(),
            &[ctx.bumps.collateral_vault],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.collateral_vault.to_account_info(),
            to: ctx.accounts.user_collateral_account.to_account_info(),
            authority: ctx.accounts.collateral_vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, payout)?;
    }
    
    // Update pool state - reduce exposure
    let pool_state = &mut ctx.accounts.pool_state;
    match ctx.accounts.position.direction {
        crate::state::Direction::Long => {
            pool_state.total_long_exposure = pool_state
                .total_long_exposure
                .checked_sub(ctx.accounts.position.size)
                .ok_or(ForwardError::MathOverflow)?;
        }
        crate::state::Direction::Short => {
            pool_state.total_short_exposure = pool_state
                .total_short_exposure
                .checked_sub(ctx.accounts.position.size)
                .ok_or(ForwardError::MathOverflow)?;
        }
    }
    
    // Reduce pool collateral
    pool_state.pool_collateral = pool_state
        .pool_collateral
        .checked_sub(pool_collateral)
        .ok_or(ForwardError::MathOverflow)?;
    
    // Mark position as settled
    ctx.accounts.position.status = PositionStatus::Settled;
    
    msg!(
        "Position settled: outcome={:?}, payout={}, size={}",
        outcome,
        payout,
        ctx.accounts.position.size
    );
    
    Ok(())
}

