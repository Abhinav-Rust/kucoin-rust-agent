# KuCoin Futures Scalp Agent — Automated Order Execution (Rust)

> An asynchronous, zero-latency automated trading agent built in Rust for the KuCoin Futures API. Implements HMAC-SHA256 double-signing authentication, dynamic position sizing from live contract metadata, and atomic market-entry → limit-exit order sequencing.

---

## What This Is

A **fire-and-forget scalp trading engine** that executes a complete trade lifecycle in a single run:

1. **Authenticate** against KuCoin's V2 Futures API using HMAC-SHA256 with double-signed passphrases
2. **Fetch** live contract metadata (price, multiplier, tick size) for precise sizing
3. **Calculate** position size, leverage allocation, and take-profit target — all derived dynamically from real-time data
4. **Execute** a market entry order immediately followed by a limit take-profit exit

The entire flow runs in ~200ms end-to-end.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          KuCoin Futures API                            │
│                    api-futures.kucoin.com (HTTPS)                      │
└────────────┬──────────────────────┬──────────────────────┬─────────────┘
             │                      │                      │
        GET /contracts         POST /orders            POST /orders
        (metadata)            (market entry)          (limit TP exit)
             │                      │                      │
             ▼                      ▼                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        Authentication Layer                            │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │  HMAC-SHA256 Signature:  sign(timestamp + method + path + body) │  │
│  │  Passphrase Signing:     sign(raw_passphrase) → Base64          │  │
│  │  Headers:  KC-API-KEY | KC-API-SIGN | KC-API-TIMESTAMP          │  │
│  │            KC-API-PASSPHRASE | KC-API-KEY-VERSION (v2)          │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
             │                      │                      │
             ▼                      ▼                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Calculation Engine                              │
│                                                                         │
│   Position Value  =  Margin (USDT) × Leverage                          │
│   Contract Lots   =  (Position Value / Price) / Multiplier → round()   │
│   TP Price Diff   =  Profit Target / (Lots × Multiplier)              │
│   TP Price        =  Entry + Diff → round to tick_size                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Key Engineering Patterns

### 1. HMAC-SHA256 Double-Signing (KuCoin V2 Auth)

KuCoin's V2 API requires **two separate HMAC operations** per request — one for the request signature, and a second to encrypt the passphrase itself:

```rust
// Request signature: HMAC(secret, timestamp + method + endpoint + body) → Base64
let str_to_sign = format!("{}{}{}{}", timestamp, method, endpoint, body);
let signature = BASE64.encode(hmac_sha256(secret, str_to_sign));

// Passphrase signing: HMAC(secret, raw_passphrase) → Base64
let pass_signature = BASE64.encode(hmac_sha256(secret, passphrase));
```

Both are injected into custom headers (`KC-API-SIGN`, `KC-API-PASSPHRASE`) alongside the millisecond-precision timestamp and API key version marker.

### 2. Dynamic Contract Sizing

Position size is **never hardcoded** — it's derived from three live parameters fetched from the contract endpoint:

| Parameter | Source | Purpose |
|-----------|--------|---------|
| `lastTradePrice` | `/api/v1/contracts/SOLUSDTM` | Current market price for lot calculation |
| `multiplier` | Contract metadata | Converts between contract lots and underlying asset units |
| `tickSize` | Contract metadata | Minimum price increment for order price rounding |

```
lots = round((margin × leverage / price) / multiplier)
tp_price = round_to_tick(entry + profit_target / (lots × multiplier))
```

### 3. Atomic Entry → Exit Sequencing

The agent places two orders in rapid succession:

| Order | Type | Side | Purpose |
|-------|------|------|---------|
| **Entry** | Market | Buy (Long) | Immediate fill at best available price |
| **Exit** | Limit | Sell | Take-profit at calculated target, `closeOrder: true` |

The limit exit uses `closeOrder: true` to bind it to the open position, ensuring it auto-cancels if the position is closed manually.

### 4. Isolated Margin Mode

All orders use `marginMode: "ISOLATED"` — risk is contained to the allocated margin per trade. A liquidation on this position cannot cascade to your account balance.

---

## Execution Flow

```
1.  Load API credentials from .env
2.  GET  /api/v1/contracts/SOLUSDTM  →  fetch price, multiplier, tick size
3.  Calculate lot size from margin ($13) × leverage (20×) ÷ price ÷ multiplier
4.  Calculate take profit price for $1.50 profit target
5.  POST /api/v1/orders  →  Market BUY (entry)
6.  POST /api/v1/orders  →  Limit SELL at TP price (exit)
7.  Print execution telemetry and exit
```

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust |
| Async Runtime | Tokio |
| HTTP Client | Reqwest (JSON feature) |
| Cryptography | `hmac` + `sha2` (HMAC-SHA256) |
| Encoding | `base64` (standard encoding) |
| Serialization | `serde_json` |
| Secret Management | `dotenv` (`.env` file, gitignored) |
| Order IDs | `uuid` v4 (cryptographically random) |

---

## Security Model

| Concern | Mitigation |
|---------|-----------|
| API keys in source | ❌ Never. Loaded from `.env` via `dotenv`, file is gitignored |
| Passphrase transmission | HMAC-signed before sending — raw passphrase never leaves the process |
| Order deduplication | UUID v4 `clientOid` per order prevents duplicate execution |
| Position isolation | `ISOLATED` margin mode caps max loss to allocated margin |

---

## Configuration

```rust
const MARGIN_USDT: f64 = 13.0;        // Capital allocated per trade
const LEVERAGE: f64 = 20.0;            // Position multiplier
const PROFIT_TARGET_USDT: f64 = 1.5;   // USD profit to capture
const SYMBOL: &str = "SOLUSDTM";       // KuCoin Futures contract
```

---

## Running

```bash
# Create .env with your KuCoin Futures API credentials
echo KUCOIN_API_KEY=your-key > .env
echo KUCOIN_API_SECRET=your-secret >> .env
echo KUCOIN_API_PASSPHRASE=your-passphrase >> .env

# Build and execute
cargo run
```

---

## Disclaimer

This agent was built and battle-tested against KuCoin's live Futures API. It is published for educational and portfolio purposes. Use at your own risk — automated trading involves significant financial risk. Always verify order parameters before deploying against real capital.
