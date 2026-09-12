# AgenticPay Mock Resource Server (x402 + Pyth)

A lightweight, high-performance Node/TypeScript resource server that serves real-time Pyth Hermes price feeds behind an **x402 micropayment paywall**.

It acts as the counterparty for autonomous trading agents: when an agent requests a premium data feed, the server returns an **HTTP 402 Payment Required** challenge, verifies the agent's Ed25519-signed voucher locally with sub-millisecond latency, and releases the Pyth price data.

---

## 🚀 Key Features

1. **x402 Paywall Middleware (`src/paywall.ts`)**
   - Emits standardized `402 Payment Required` with `PAYMENT-REQUIRED` header.
   - Decodes `PAYMENT-SIGNATURE` headers containing deferred payment vouchers.
   - Enforces provider identity, minimum payment amount, expiry TTL, and cryptographic validity.

2. **Cross-Language Canonical Voucher Verifier (`src/voucher.ts`)**
   - Matches `agent-backend/src/voucher.rs` and `programs/agenticpay-guardrails` byte-for-byte.
   - Domain separator: `agenticpay_x402v1` (17 bytes).
   - Fixed 105-byte canonical payload verified via Ed25519 (`tweetnacl`).

3. **Pyth Hermes Client with Caching (`src/pyth.ts`)**
   - Connects to Pyth Network Hermes service (`https://hermes.pyth.network`).
   - LRU cache with configurable TTL (default 5s) to avoid redundant network round-trips.
   - Resilient fallback for offline testing or development environments.

---

## 📐 Voucher Layout Specification

```
0                 17     25                 57                 89         97         105        169        170
├─────────────────┼──────┼──────────────────┼──────────────────┼──────────┼──────────┼──────────┼──────────┤
│ DOMAIN_SEP (17B)│ Nonce│ Agent Pubkey     │ Provider Pubkey  │ Amount   │ Expires  │Signature │ Retry    │
│agenticpay_x402v1│ (8B) │ (32B)            │ (32B)            │ (8B LE)  │ (8B LE)  │ (64B)    │ (1B)     │
└─────────────────┴──────┴──────────────────┴──────────────────┴──────────┴──────────┴──────────┴──────────┘
 <────────────── Canonical Message (105B) ───────────────>  <────── Signature (64B) ──────>
```

- **Header name**: `PAYMENT-SIGNATURE`
- **Encoding**: Standard Base64 of the full 170-byte structure.

---

## 🛠️ Installation & Testing

```bash
cd mock-resource-server

# Install dependencies
npm install

# Run Jest unit & integration tests
npm test

# Build TypeScript to dist/
npm run build

# Start dev server (runs on port 8080)
npm start
```

---

## 🌐 Endpoints

### 1. `GET /health` (Public)
Health check, service info, and provider public key.

```bash
curl http://127.0.0.1:8080/health
```

### 2. `GET /price/:symbol` (x402-Gated)
Supported symbols: `BTC`, `ETH`, `SOL` (or full Pyth feed ID).

#### Unauthenticated Request (returns 402):
```bash
curl -i http://127.0.0.1:8080/price/BTC
```

Response:
```http
HTTP/1.1 402 Payment Required
PAYMENT-REQUIRED: {"price":1000,"network":"solana","recipient":"9V...","ttl":60,"scheme":"agenticpay_x402v1"}
Content-Type: application/json

{
  "error": "Payment Required",
  "payment_required": {
    "price": 1000,
    "network": "solana",
    "recipient": "9V...",
    "ttl": 60,
    "scheme": "agenticpay_x402v1"
  }
}
```

#### Authenticated Request with Voucher:
```bash
curl -i http://127.0.0.1:8080/price/BTC \
  -H "PAYMENT-SIGNATURE: <base64-encoded-voucher>"
```

Response:
```json
{
  "symbol": "BTC",
  "price": 64230.5,
  "confidence": 12.3,
  "expo": -8,
  "publish_time": 1726140000,
  "formatted_price": "64230.50",
  "source": "hermes-live",
  "feed_id": "0xe62df6e110f78005f12b4fce2f688b04f32d5ffb5f5da89453feed78dd595a4d",
  "payment": {
    "status": "verified",
    "agent": "4xK...",
    "provider": "9V...",
    "nonce": "1",
    "amount_lamports": "1000",
    "expires_at": "1726140060"
  }
}
```
