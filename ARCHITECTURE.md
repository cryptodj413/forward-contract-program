# Architecture Documentation

## System Overview

This Solana program implements a forward contract system for Polymarket binary outcomes using a probabilistic Automated Market Maker (pAMM) for pricing.

## Account Structure

### GlobalConfig
- **Purpose**: Platform-wide configuration
- **Fields**:
  - `admin`: Admin authority
  - `collateral_mint`: Token mint for collateral (e.g., USDC)
  - `curve_params`: pAMM curve parameters
  - `bump`: PDA bump seed
- **PDA**: `[b"global_config"]`

### MarketConfig
- **Purpose**: Per-market configuration
- **Fields**:
  - `polymarket_market_id`: String identifier for Polymarket market
  - `resolution_timestamp`: Unix timestamp when market resolves
  - `price_oracle`: Pubkey of price oracle account
  - `resolution_oracle`: Pubkey of resolution oracle account
  - `risk_limits`: Risk limits for this market
  - `status`: Market status (Active, TradingClosed, Resolved, Cancelled)
  - `pool_state`: Pubkey of associated PoolState
  - `collateral_vault`: Pubkey of collateral token account
  - `bump`: PDA bump seed
- **PDA**: `[b"market", polymarket_market_id.as_bytes()]`

### PoolState
- **Purpose**: Tracks exposure and collateral for a market
- **Fields**:
  - `market`: Pubkey of MarketConfig
  - `total_long_exposure`: Total long positions (Q_long)
  - `total_short_exposure`: Total short positions (Q_short)
  - `pool_collateral`: Total pool collateral locked
  - `position_counter`: Counter for unique position IDs
  - `bump`: PDA bump seed
- **PDA**: `[b"pool_state", market_config.key()]`
- **Methods**:
  - `net_exposure()`: Returns Q_long - Q_short

### Position
- **Purpose**: Individual user position
- **Fields**:
  - `owner`: User's public key
  - `market`: Pubkey of MarketConfig
  - `direction`: Long or Short
  - `size`: Position size (Q)
  - `forward_price`: Forward price at entry (K, in basis points)
  - `collateral_locked`: User's locked collateral
  - `premium_paid`: Premium paid/received (can be negative)
  - `status`: Position status (Open, Settled, Cancelled)
  - `bump`: PDA bump seed
- **PDA**: `[b"position", market_config.key(), position_counter]`

### Oracle Accounts

#### PriceOracle
- **Purpose**: Provides current Polymarket price
- **Fields**:
  - `price`: Price in basis points (0-10000, where 10000 = 1.0)
  - `timestamp`: Last update timestamp
  - `exponent`: Price precision exponent

#### ResolutionOracle
- **Purpose**: Provides final market outcome
- **Fields**:
  - `outcome`: Option<u8> (1 = YES, 0 = NO, None = unresolved)
  - `resolved_at`: Option<i64> timestamp

## Instruction Flow

### 1. Initialize System

```
init_global_config
├── Creates GlobalConfig PDA
├── Sets admin authority
├── Sets collateral mint
└── Sets initial curve parameters
```

### 2. Create Market

```
create_market
├── Creates MarketConfig PDA
├── Creates PoolState PDA
├── Creates CollateralVault PDA (token account)
├── Links to Polymarket market ID
├── Sets oracle addresses
└── Sets risk limits
```

### 3. Open Position

```
open_position
├── Validates market is active
├── Reads Polymarket price from oracle
├── Calculates forward price K using pAMM curve
├── Calculates premium based on exposure
├── Validates position size against limits
├── Calculates required collateral
├── Transfers user collateral + premium to vault
├── Updates pool state (exposure, collateral)
└── Creates Position account
```

### 4. Settle Position

```
settle_position
├── Validates market is resolved
├── Reads outcome from resolution oracle
├── Calculates payout based on outcome
├── Transfers payout to user
├── Updates pool state (reduces exposure)
└── Marks position as settled
```

### 5. Update Resolution (Keeper)

```
update_market_resolution
├── Validates market status
├── Sets outcome
└── Changes market status to Resolved
```

## Pricing Model

### Forward Price Calculation

```
K = clamp(p + α * (e / E_max), p_min, p_max)
```

Where:
- `p` = Polymarket spot price (0-1, in basis points)
- `e` = net exposure = Q_long - Q_short
- `E_max` = maximum allowed exposure
- `α` = curve slope parameter
- `p_min`, `p_max` = price bounds

### Premium Calculation

```
premium_rate = β * (e / E_max)
premium = premium_rate * Q
```

For long positions: user pays if premium > 0, receives if premium < 0
For short positions: opposite sign

### Collateral Calculation

- **Long**: `collateral = K * Q`
- **Short**: `collateral = (1 - K) * Q`

Pool locks the opposite side's collateral.

### Settlement

- **YES outcome**:
  - Long receives: `Q` (total collateral)
  - Short receives: `0`
- **NO outcome**:
  - Long receives: `0`
  - Short receives: `Q` (total collateral)

## Security Considerations

1. **Fully Collateralized**: All positions are fully collateralized at opening
2. **Admin Protection**: Critical functions require admin authority
3. **Slippage Protection**: Users can set slippage tolerance
4. **Math Overflow**: All arithmetic operations use checked math
5. **Oracle Validation**: Oracle accounts should be validated in production
6. **Position Uniqueness**: Position counter ensures unique position IDs

## Oracle Integration

The system expects oracle accounts that provide:

1. **Price Oracle**: Current Polymarket price (updated regularly)
2. **Resolution Oracle**: Final outcome after Polymarket resolves

In production, integrate with:
- Pyth Network
- Switchboard
- Custom Polymarket oracle relay service

## Risk Management

### Per-Market Limits

- `max_total_exposure`: Maximum total exposure (long + short)
- `max_long_share`: Maximum long exposure as fraction
- `max_short_share`: Maximum short exposure as fraction

### Curve Parameters

- `alpha`: Controls how strongly price moves with exposure
- `beta`: Controls premium magnitude
- `max_exposure`: Maximum allowed absolute net exposure
- `min_price`, `max_price`: Price bounds to prevent extreme values

## Future Enhancements

1. **Early Position Closure**: Mark-to-market pricing for early exit
2. **Liquidity Pools**: Allow LPs to provide liquidity
3. **Governance**: DAO governance for parameter updates
4. **Multi-Collateral**: Support multiple collateral types
5. **Advanced Curves**: More sophisticated pricing curves
6. **Oracle Aggregation**: Multiple oracle sources with consensus

