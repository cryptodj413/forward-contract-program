use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Direction, MarketStatus, Position, PositionStatus};
use crate::math::{self, BASIS_POINTS};
use crate::errors::ForwardError;
use crate::oracle;

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"global_config"],
        bump = global_config.bump
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
        seeds = [b"pool_state", market_config.key().as_ref()],
        bump = pool_state.bump
    )]
    pub pool_state: Account<'info, crate::state::PoolState>,
    
    #[account(
        mut,
        seeds = [b"price_oracle", market_config.key().as_ref()],
        bump
    )]
    pub price_oracle: Account<'info, crate::oracle::PriceOracle>,
    
    #[account(
        mut,
        seeds = [b"collateral_vault", market_config.key().as_ref()],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_collateral_account.owner == user.key()
    )]
    pub user_collateral_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = user,
        space = Position::LEN,
        seeds = [b"position", market_config.key().as_ref(), &pool_state.position_counter.to_le_bytes()],
        bump
    )]
    pub position: Account<'info, Position>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<OpenPosition>,
    direction: Direction,
    size: u64,
    slippage_tolerance: Option<u64>,
) -> Result<()> {
    require!(
        ctx.accounts.market_config.status == MarketStatus::Active,
        ForwardError::MarketNotActive
    );
    
    require!(size > 0, ForwardError::PositionSizeExceedsLimit);
    
    // Read Polymarket price from oracle
    let polymarket_price = oracle::read_price(&ctx.accounts.price_oracle)?;
    require!(polymarket_price > 0, ForwardError::InvalidOracleData);
    
    // Calculate forward price K using pAMM curve
    let forward_price = math::calculate_forward_price(
        polymarket_price,
        &ctx.accounts.pool_state,
        &ctx.accounts.global_config.curve_params,
    );
    
    // Check slippage if provided
    if let Some(slippage) = slippage_tolerance {
        let price_diff = if forward_price > polymarket_price {
            forward_price - polymarket_price
        } else {
            polymarket_price - forward_price
        };
        let slippage_bps = (price_diff * BASIS_POINTS) / polymarket_price;
        require!(
            slippage_bps <= slippage,
            ForwardError::SlippageExceeded
        );
    }
    
    // Calculate premium
    let premium_rate = math::calculate_premium_rate(
        &ctx.accounts.pool_state,
        &ctx.accounts.global_config.curve_params,
        direction,
    );
    let premium = math::calculate_premium(premium_rate, size);
    
    // Calculate required collateral
    let user_collateral = math::calculate_collateral(
        forward_price,
        size,
        direction,
    );
    
    // Calculate pool collateral (opposite side)
    let pool_collateral = math::calculate_collateral(
        forward_price,
        size,
        match direction {
            Direction::Long => Direction::Short,
            Direction::Short => Direction::Long,
        },
    );
    
    // Check max size
    let max_size = math::calculate_max_size(
        &ctx.accounts.pool_state,
        &ctx.accounts.global_config.curve_params,
        &ctx.accounts.market_config.risk_limits,
        direction,
    );
    
    require!(
        size <= max_size,
        ForwardError::PositionSizeExceedsLimit
    );
    
    // Calculate total amount user needs to transfer
    // User collateral + premium (if positive) or -premium (if negative, user receives)
    let user_transfer_amount = if premium > 0 {
        user_collateral
            .checked_add(premium as u64)
            .ok_or(ForwardError::MathOverflow)?
    } else {
        // Premium is negative, user receives it, but still needs to lock collateral
        user_collateral
    };
    
    // Check user has enough balance
    require!(
        ctx.accounts.user_collateral_account.amount >= user_transfer_amount,
        ForwardError::InsufficientCollateral
    );
    
    // Transfer user collateral + premium to vault
    if user_transfer_amount > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_collateral_account.to_account_info(),
            to: ctx.accounts.collateral_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, user_transfer_amount)?;
    }
    
    // If premium is negative, transfer it back to user (user receives premium)
    if premium < 0 {
        let premium_amount = premium.unsigned_abs();
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
        token::transfer(cpi_ctx, premium_amount)?;
    }
    
    // Update pool state
    let pool_state = &mut ctx.accounts.pool_state;
    match direction {
        Direction::Long => {
            pool_state.total_long_exposure = pool_state
                .total_long_exposure
                .checked_add(size)
                .ok_or(ForwardError::MathOverflow)?;
        }
        Direction::Short => {
            pool_state.total_short_exposure = pool_state
                .total_short_exposure
                .checked_add(size)
                .ok_or(ForwardError::MathOverflow)?;
        }
    }
    pool_state.pool_collateral = pool_state
        .pool_collateral
        .checked_add(pool_collateral)
        .ok_or(ForwardError::MathOverflow)?;
    pool_state.position_counter = pool_state
        .position_counter
        .checked_add(1)
        .ok_or(ForwardError::MathOverflow)?;
    
    // Create position account
    let position = &mut ctx.accounts.position;
    position.owner = ctx.accounts.user.key();
    position.market = ctx.accounts.market_config.key();
    position.direction = direction;
    position.size = size;
    position.forward_price = forward_price;
    position.collateral_locked = user_collateral;
    position.premium_paid = premium;
    position.status = PositionStatus::Open;
    position.bump = ctx.bumps.position;
    
    let direction_str = match direction {
        crate::state::Direction::Long => "Long",
        crate::state::Direction::Short => "Short",
    };
    
    msg!(
        "Position opened: {} {} at K={}, premium={}, collateral={}",
        direction_str,
        size,
        forward_price,
        premium,
        user_collateral
    );
    
    Ok(())
}

