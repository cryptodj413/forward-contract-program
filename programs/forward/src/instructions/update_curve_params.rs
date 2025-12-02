use anchor_lang::prelude::*;
use crate::state::CurveParams;
use crate::errors::ForwardError;
use crate::math::BASIS_POINTS;

#[derive(Accounts)]
pub struct UpdateCurveParams<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"global_config"],
        bump = global_config.bump,
        has_one = admin @ ForwardError::Unauthorized
    )]
    pub global_config: Account<'info, crate::state::GlobalConfig>,
}

pub fn handler(
    ctx: Context<UpdateCurveParams>,
    curve_params: CurveParams,
) -> Result<()> {
    // Validate updated curve parameters with same invariants as init_global_config
    require!(
        curve_params.min_price <= curve_params.max_price,
        ForwardError::InvalidOracleData
    );
    require!(
        curve_params.max_price <= BASIS_POINTS,
        ForwardError::InvalidOracleData
    );
    require!(
        curve_params.max_exposure > 0,
        ForwardError::InvalidOracleData
    );

    let global_config = &mut ctx.accounts.global_config;
    global_config.curve_params = curve_params;
    
    msg!("Curve parameters updated");
    
    Ok(())
}

