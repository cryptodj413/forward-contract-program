use anchor_lang::prelude::*;
use crate::state::{GlobalConfig, CurveParams};

#[derive(Accounts)]
pub struct InitGlobalConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = GlobalConfig::LEN,
        seeds = [b"global_config"],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,
    
    /// CHECK: Collateral mint account (e.g., USDC)
    pub collateral_mint: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitGlobalConfig>,
    curve_params: CurveParams,
) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    
    global_config.admin = ctx.accounts.admin.key();
    global_config.collateral_mint = ctx.accounts.collateral_mint.key();
    global_config.curve_params = curve_params;
    global_config.bump = ctx.bumps.global_config;
    
    msg!("Global config initialized with admin: {}", global_config.admin);
    
    Ok(())
}

