use crate::state::{CurveParams, Direction, PoolState};

/// Basis points constant (10000 = 100%)
pub const BASIS_POINTS: u64 = 10000;

/// Calculate forward price K based on pAMM curve
/// 
/// Formula: K = p + α * (e / E_max)
/// Where:
/// - p = Polymarket price (0 to 1, stored as basis points)
/// - e = net exposure = Q_long - Q_short
/// - E_max = maximum allowed exposure
/// - α = curve slope parameter
pub fn calculate_forward_price(
    polymarket_price: u64, // p in basis points (e.g., 5000 = 0.5)
    pool_state: &PoolState,
    curve_params: &CurveParams,
) -> u64 {
    let p = polymarket_price as i64;
    let e = pool_state.net_exposure();
    let e_max = curve_params.max_exposure as i64;
    let alpha = curve_params.alpha as i64;

    // Calculate exposure ratio: e / E_max (clamped to [-1, 1])
    let exposure_ratio = if e_max > 0 {
        (e * BASIS_POINTS as i64) / e_max
    } else {
        0
    };
    let exposure_ratio = exposure_ratio.max(-(BASIS_POINTS as i64)).min(BASIS_POINTS as i64);

    // K = p + α * (exposure_ratio / BASIS_POINTS)
    let k = p + (alpha * exposure_ratio) / (BASIS_POINTS as i64);

    // Clamp K between min_price and max_price
    let k = k.max(curve_params.min_price as i64).min(curve_params.max_price as i64);

    k as u64
}

/// Calculate premium rate based on exposure
/// 
/// Formula: premium_rate = β * (e / E_max)
/// For long: premium = premium_rate * Q
/// For short: premium = -premium_rate * Q
pub fn calculate_premium_rate(
    pool_state: &PoolState,
    curve_params: &CurveParams,
    direction: Direction,
) -> i64 {
    let e = pool_state.net_exposure();
    let e_max = curve_params.max_exposure as i64;
    let beta = curve_params.beta as i64;

    // Calculate exposure ratio
    let exposure_ratio = if e_max > 0 {
        (e * BASIS_POINTS as i64) / e_max
    } else {
        0
    };
    let exposure_ratio = exposure_ratio.max(-(BASIS_POINTS as i64)).min(BASIS_POINTS as i64);

    // premium_rate = β * exposure_ratio
    let premium_rate = (beta * exposure_ratio) / (BASIS_POINTS as i64);

    // For long: positive premium_rate means user pays
    // For short: negative premium_rate means user receives
    match direction {
        Direction::Long => premium_rate,
        Direction::Short => -premium_rate,
    }
}

/// Calculate total premium for a position
pub fn calculate_premium(
    premium_rate: i64,
    size: u64,
) -> i64 {
    // premium = premium_rate * Q / BASIS_POINTS
    // premium_rate is already in basis points, so we divide by BASIS_POINTS
    (premium_rate * size as i64) / (BASIS_POINTS as i64)
}

/// Calculate required collateral for a position
/// 
/// Long collateral: K * Q
/// Short collateral: (1 - K) * Q
pub fn calculate_collateral(
    forward_price: u64, // K in basis points
    size: u64,          // Q
    direction: Direction,
) -> u64 {
    match direction {
        Direction::Long => {
            // coll_long = K * Q
            (forward_price as u128 * size as u128 / BASIS_POINTS as u128) as u64
        }
        Direction::Short => {
            // coll_short = (1 - K) * Q
            let one_minus_k = BASIS_POINTS - forward_price;
            (one_minus_k as u128 * size as u128 / BASIS_POINTS as u128) as u64
        }
    }
}

/// Calculate maximum allowed size for a new position
pub fn calculate_max_size(
    pool_state: &PoolState,
    _curve_params: &CurveParams,
    risk_limits: &crate::state::RiskLimits,
    direction: Direction,
) -> u64 {
    let current_exposure = match direction {
        Direction::Long => pool_state.total_long_exposure,
        Direction::Short => pool_state.total_short_exposure,
    };

    // Check against max total exposure
    let max_by_total = risk_limits.max_total_exposure.saturating_sub(
        pool_state.total_long_exposure + pool_state.total_short_exposure
    );

    // Check against max share
    let max_by_share = match direction {
        Direction::Long => {
            let max_long = (risk_limits.max_total_exposure as u128 * risk_limits.max_long_share as u128 / BASIS_POINTS as u128) as u64;
            max_long.saturating_sub(current_exposure)
        }
        Direction::Short => {
            let max_short = (risk_limits.max_total_exposure as u128 * risk_limits.max_short_share as u128 / BASIS_POINTS as u128) as u64;
            max_short.saturating_sub(current_exposure)
        }
    };

    max_by_total.min(max_by_share)
}

/// Calculate settlement payout for a position
/// 
/// If outcome = YES:
///   Long receives: Q (total collateral)
///   Short receives: 0
/// 
/// If outcome = NO:
///   Long receives: 0
///   Short receives: Q (total collateral)
pub fn calculate_settlement_payout(
    size: u64,
    direction: Direction,
    outcome: crate::state::Outcome,
) -> u64 {
    match (direction, outcome) {
        (Direction::Long, crate::state::Outcome::Yes) => size,
        (Direction::Long, crate::state::Outcome::No) => 0,
        (Direction::Short, crate::state::Outcome::Yes) => 0,
        (Direction::Short, crate::state::Outcome::No) => size,
    }
}

