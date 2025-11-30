use anchor_lang::prelude::*;

#[error_code]
pub enum ForwardError {
    #[msg("Market is not active for trading")]
    MarketNotActive,
    
    #[msg("Market has already been resolved")]
    MarketAlreadyResolved,
    
    #[msg("Position size exceeds maximum allowed")]
    PositionSizeExceedsLimit,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
    
    #[msg("Position already settled")]
    PositionAlreadySettled,
    
    #[msg("Market not yet resolved")]
    MarketNotResolved,
    
    #[msg("Invalid oracle data")]
    InvalidOracleData,
    
    #[msg("Unauthorized: admin only")]
    Unauthorized,
    
    #[msg("Invalid market status")]
    InvalidMarketStatus,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Invalid direction")]
    InvalidDirection,
    
    #[msg("Invalid mint")]
    InvalidMint,
}

