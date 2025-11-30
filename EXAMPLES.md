# Usage Examples

## TypeScript/JavaScript Client Examples

### Initialize Global Config

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { forward } from "./target/types/forward";

// Initialize program
const program = anchor.workspace.forward as Program<forward>;

// Initialize global config
const curveParams = {
  alpha: new anchor.BN(1000),      // 10% slope (in basis points)
  beta: new anchor.BN(500),        // 5% premium multiplier
  maxExposure: new anchor.BN(1000000), // Max exposure
  minPrice: new anchor.BN(500),     // 5% min price
  maxPrice: new anchor.BN(9500),    // 95% max price
};

await program.methods
  .initGlobalConfig(curveParams)
  .accounts({
    admin: adminKeypair.publicKey,
    globalConfig: globalConfigPda,
    collateralMint: usdcMint,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();
```

### Create Market

```typescript
const polymarketMarketId = "0x1234..."; // Polymarket market ID
const resolutionTimestamp = new anchor.BN(Math.floor(Date.now() / 1000) + 86400); // 24h from now

const riskLimits = {
  maxTotalExposure: new anchor.BN(10000000),
  maxLongShare: new anchor.BN(6000),  // 60% max long
  maxShortShare: new anchor.BN(6000), // 60% max short
};

const [marketConfig] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("market"), Buffer.from(polymarketMarketId)],
  program.programId
);

const [poolState] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("pool_state"), marketConfig.toBuffer()],
  program.programId
);

const [collateralVault] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("collateral_vault"), marketConfig.toBuffer()],
  program.programId
);

await program.methods
  .createMarket(
    polymarketMarketId,
    resolutionTimestamp,
    riskLimits
  )
  .accounts({
    admin: adminKeypair.publicKey,
    globalConfig: globalConfigPda,
    marketConfig: marketConfig,
    poolState: poolState,
    priceOracle: priceOraclePda,
    resolutionOracle: resolutionOraclePda,
    collateralVault: collateralVault,
    tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();
```

### Open Position

```typescript
// Quote a position first (simulate)
const poolStateAccount = await program.account.poolState.fetch(poolState);
const priceOracleAccount = await program.account.priceOracle.fetch(priceOraclePda);

// Calculate forward price and premium client-side
const polymarketPrice = priceOracleAccount.price.toNumber();
const netExposure = poolStateAccount.totalLongExposure.sub(poolStateAccount.total_shortExposure).toNumber();
const exposureRatio = netExposure / curveParams.maxExposure.toNumber();
const forwardPrice = Math.max(
  curveParams.minPrice.toNumber(),
  Math.min(
    curveParams.maxPrice.toNumber(),
    polymarketPrice + (curveParams.alpha.toNumber() * exposureRatio) / 10000
  )
);

const direction = { long: {} }; // or { short: {} }
const size = new anchor.BN(1000); // Position size
const slippageTolerance = new anchor.BN(100); // 1% slippage

// Calculate required collateral
const userCollateral = direction.long !== undefined
  ? (forwardPrice * size.toNumber()) / 10000
  : ((10000 - forwardPrice) * size.toNumber()) / 10000;

const [positionPda] = anchor.web3.PublicKey.findProgramAddressSync(
  [
    Buffer.from("position"),
    marketConfig.toBuffer(),
    poolStateAccount.positionCounter.toArrayLike(Buffer, "le", 8),
  ],
  program.programId
);

await program.methods
  .openPosition(direction, size, slippageTolerance)
  .accounts({
    user: userKeypair.publicKey,
    globalConfig: globalConfigPda,
    marketConfig: marketConfig,
    poolState: poolState,
    priceOracle: priceOraclePda,
    collateralVault: collateralVault,
    userCollateralAccount: userUsdcAccount,
    position: positionPda,
    tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([userKeypair])
  .rpc();
```

### Settle Position

```typescript
const positionAccount = await program.account.position.fetch(positionPda);

await program.methods
  .settlePosition()
  .accounts({
    user: userKeypair.publicKey,
    marketConfig: marketConfig,
    poolState: poolState,
    resolutionOracle: resolutionOraclePda,
    position: positionPda,
    collateralVault: collateralVault,
    userCollateralAccount: userUsdcAccount,
    tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
  })
  .signers([userKeypair])
  .rpc();
```

### Update Market Resolution (Keeper)

```typescript
const outcome = { yes: {} }; // or { no: {} }

await program.methods
  .updateMarketResolution(outcome)
  .accounts({
    keeper: keeperKeypair.publicKey,
    marketConfig: marketConfig,
    resolutionOracle: resolutionOraclePda,
  })
  .signers([keeperKeypair])
  .rpc();
```

## Rust Client Example

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

// Initialize global config
let curve_params = CurveParams {
    alpha: 1000,        // 10% in basis points
    beta: 500,          // 5% in basis points
    max_exposure: 1000000,
    min_price: 500,     // 5% in basis points
    max_price: 9500,     // 95% in basis points
};

let accounts = InitGlobalConfig {
    admin: admin_keypair,
    global_config: global_config_pda,
    collateral_mint: usdc_mint,
    system_program: system_program::ID,
};

program
    .request()
    .accounts(accounts)
    .args(init_global_config::Args { curve_params })
    .send()?;

// Open position
let direction = Direction::Long;
let size = 1000u64;
let slippage_tolerance = Some(100u64); // 1%

let accounts = OpenPosition {
    user: user_keypair,
    global_config: global_config_pda,
    market_config: market_config_pda,
    pool_state: pool_state_pda,
    price_oracle: price_oracle_account,
    collateral_vault: collateral_vault_pda,
    user_collateral_account: user_usdc_account,
    position: position_pda,
    token_program: token::ID,
    system_program: system_program::ID,
};

program
    .request()
    .accounts(accounts)
    .args(open_position::Args {
        direction,
        size,
        slippage_tolerance,
    })
    .send()?;
```

## Price Quoting (Client-Side)

Before opening a position, users should quote the price:

```typescript
async function quotePosition(
  program: Program<forward>,
  marketConfig: PublicKey,
  direction: "long" | "short",
  size: number
) {
  // Fetch current state
  const [poolStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool_state"), marketConfig.toBuffer()],
    program.programId
  );
  
  const poolState = await program.account.poolState.fetch(poolStatePda);
  const marketConfigAccount = await program.account.marketConfig.fetch(marketConfig);
  const globalConfig = await program.account.globalConfig.fetch(globalConfigPda);
  const priceOracle = await program.account.priceOracle.fetch(marketConfigAccount.priceOracle);
  
  // Calculate forward price
  const p = priceOracle.price.toNumber();
  const e = poolState.totalLongExposure.toNumber() - poolState.totalShortExposure.toNumber();
  const eMax = globalConfig.curveParams.maxExposure.toNumber();
  const alpha = globalConfig.curveParams.alpha.toNumber();
  
  const exposureRatio = (e * 10000) / eMax;
  const k = Math.max(
    globalConfig.curveParams.minPrice.toNumber(),
    Math.min(
      globalConfig.curveParams.maxPrice.toNumber(),
      p + (alpha * exposureRatio) / 10000
    )
  );
  
  // Calculate premium
  const beta = globalConfig.curveParams.beta.toNumber();
  const premiumRate = (beta * exposureRatio) / 10000;
  const premium = direction === "long" 
    ? (premiumRate * size) / 10000
    : (-premiumRate * size) / 10000;
  
  // Calculate collateral
  const collateral = direction === "long"
    ? (k * size) / 10000
    : ((10000 - k) * size) / 10000;
  
  return {
    forwardPrice: k / 10000,
    premium: premium / 10000,
    collateral: collateral,
    totalRequired: collateral + (premium > 0 ? premium : 0),
  };
}
```

