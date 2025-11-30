# Forward - Polymarket Integration on Solana

A Solana smart contract system that provides forward contracts on binary outcomes from Polymarket, using a probabilistic Automated Market Maker (pAMM) for pricing.

## Overview

This system allows users to trade forward contracts on Polymarket binary outcomes with:
- **Fully collateralized positions** (no margin, no liquidation)
- **pAMM pricing** that adjusts based on pool exposure
- **Premium system** to incentivize balanced exposure
- **Oracle integration** for Polymarket prices and resolution

## Architecture

### Account Types

1. **GlobalConfig**: Platform-wide configuration (admin, collateral mint, curve parameters)
2. **MarketConfig**: Per-market configuration (Polymarket ID, oracles, risk limits)
3. **PoolState**: Tracks exposure and collateral for each market
4. **Position**: Individual user positions (long/short, size, forward price)

### Core Instructions

#### Admin Instructions
- `init_global_config`: Initialize the platform
- `create_market`: Create a new market linked to Polymarket
- `update_curve_params`: Update pAMM curve parameters
- `close_market_for_trading`: Close market before resolution

#### User Instructions
- `open_position`: Open a long or short position
- `settle_position`: Settle a position after market resolution

#### Keeper Instructions
- `update_market_resolution`: Update market resolution from oracle

## Math Model

### Forward Price (K)
```
K = p + α * (e / E_max)
```
Where:
- `p` = Polymarket spot price (0 to 1)
- `e` = net exposure (Q_long - Q_short)
- `E_max` = maximum allowed exposure
- `α` = curve slope parameter

### Premium
```
premium_rate = β * (e / E_max)
premium = premium_rate * Q
```
- Positive premium: user pays pool
- Negative premium: pool pays user

### Collateral
- **Long**: `K * Q`
- **Short**: `(1 - K) * Q`

### Settlement
- **YES outcome**:
  - Long receives: `Q` (total collateral)
  - Short receives: `0`
- **NO outcome**:
  - Long receives: `0`
  - Short receives: `Q` (total collateral)

## Building

```bash
# Install Anchor
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash

# Build
anchor build

# Run tests
anchor test
```

## Deployment

```bash
anchor deploy
```

## Oracle Integration

The system expects two types of oracle accounts:

1. **PriceOracle**: Provides current Polymarket price (0-1, stored as basis points)
2. **ResolutionOracle**: Provides final outcome (YES/NO) after resolution

In production, integrate with:
- Pyth Network
- Switchboard
- Custom Polymarket oracle relay

## Configuration

### Curve Parameters
- `alpha`: Curve slope (how strongly price moves with exposure)
- `beta`: Premium multiplier
- `max_exposure`: Maximum allowed absolute exposure
- `min_price`: Minimum forward price (basis points)
- `max_price`: Maximum forward price (basis points)

### Risk Limits (per market)
- `max_total_exposure`: Maximum total exposure
- `max_long_share`: Maximum long exposure as fraction (basis points)
- `max_short_share`: Maximum short exposure as fraction (basis points)

## Security Considerations

- All positions are fully collateralized
- Admin-only functions are protected
- Oracle validation should be added in production
- Slippage protection for position opening
- Math overflow checks throughout

## License

MIT

