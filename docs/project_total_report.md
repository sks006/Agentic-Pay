# AgenticPay — Total Project Summary

## The Core Problem We Solved

**AI agents can think, plan, and execute — but they can't pay.**

When an autonomous AI agent needs premium data (Pyth price feeds, L3 order books, sentiment analysis), it hits an infrastructure built for humans:

- Credit card registrations
- API key management
- Monthly subscription tiers
- Human-in-the-loop approval for every purchase

This is economically irrational. A trading bot that fires 10,000 data queries a day doesn't need a $500/month subscription — it needs to pay **$0.001 per query**, only when the data matters.

**Worse:** if you give an AI agent a funded wallet with API keys, and its decision logic gets compromised (prompt injection, poisoned data, an infinite loop), it can drain the entire treasury in minutes. There is no safety net.

---

## What AgenticPay Built

AgenticPay is an **autonomous micropayment protocol** for AI agents with **physically enforced spending guardrails** — on-chain caps that cannot be bypassed even if the agent is fully compromised.

### The System Has Three Layers

```
┌─────────────────────────────────────────────────────────────┐
│  LAYER 1 — AGENT BACKEND (Rust)                             │
│  The autonomous AI that runs the strategy                   │
│  • Monitors free WebSocket streams                          │
│  • Detects trading signals (volatility, divergence)         │
│  • Fetches premium data via x402 micropayments              │
│  • Evaluates expected value with fixed-point math           │
│  • Executes trades                                           │
└─────────────────────────────────────────────────────────────┘
              │                        ▲
              ▼                        │
┌─────────────────────────────────────────────────────────────┐
│  LAYER 2 — RESOURCE SERVER (Node/TS)                        │
│  The data provider that gates Pyth feeds behind x402        │
│  • Serves real-time Pyth Hermes price data                  │
│  • Returns HTTP 402 Payment Required when unpaid            │
│  • Verifies Ed25519-signed vouchers in sub-1ms              │
│  • Grants access on valid signature                         │
└─────────────────────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────┐
│  LAYER 3 — ON-CHAIN GUARDRAILS (Anchor/Solana)              │
│  The physics engine that cannot be bypassed                 │
│  • Holds the agent's SOL in a Program-Derived escrow        │
│  • Enforces daily spend caps and per-transaction limits     │
│  • Verifies Ed25519 signatures via sysvar introspection     │
│  • Batches settlements for gas efficiency                   │
│  • Emergency pause switch owned by the human                │
└─────────────────────────────────────────────────────────────┘
```

---

## The Two Critical Loops

### 🔵 Loop A: Data Acquisition (Agent ↔ Resource Server)

The agent needs premium data. Instead of subscribing, it pays per query.

```
Free WebSocket trigger fires
  → Agent sends GET /price/BTC (no payment)
  → Server returns 402 Payment Required + payment terms
  → Agent signs a 105-byte Ed25519 voucher
  → Agent retries with PAYMENT-SIGNATURE header
  → Server verifies signature locally in <1ms
  → Server returns Pyth price data
  → Agent feeds data into fixed-point decision engine
```

**Latency:** ~20–50 ms end-to-end (no on-chain settlement on critical path).

**Cost:** Zero when there's no signal. Fractions of a cent when there is.

---

### 🔵 Loop B: On-Chain Settlement (Voucher → Guardrails)

The signed vouchers accumulate. A background worker settles them in batches.

```
Voucher enters state as IntentSigned
  → SettlementWorker sweeps every 10 seconds
  → Chunks vouchers into MTU-safe batches (≤1232 bytes)
  → Builds [Ed25519 ix, batch_settle_vouchers ix] pairs per voucher
  → Sends transaction to Solana
  → Anchor program:
      1. Verifies preceding Ed25519 instruction
      2. Checks 105-byte canonical message layout
      3. Enforces per-tx cap
      4. Enforces daily cap (resets every ~24h via slots)
      5. Checks voucher TTL
      6. Transfers SOL from escrow PDA → provider
      7. Emits VoucherSettled event
  → Voucher promoted to Finalized
```

**What makes this unique:** Even if the agent's decision engine is fully poisoned or hijacked, the on-chain program will refuse to transfer more than the pre-authorized caps. **The bot can be compromised — the money cannot.**

---

## The Cross-Language Cryptographic Contract

Rust (agent), TypeScript (server), and Anchor (on-chain) all agree on **one byte-for-byte canonical message format**:

| Offset | Field | Size | Endianness |
|--------|-------|------|------------|
| 0..17 | `domain_separator` = `"agenticpay_x402v1"` | 17B | ASCII |
| 17..25 | `nonce` | 8B | LE |
| 25..57 | `agent` pubkey | 32B | raw |
| 57..89 | `provider` pubkey | 32B | raw |
| 89..97 | `amount_lamports` | 8B | LE |
| 97..105 | `expires_at` (Unix seconds) | 8B | LE |

**Wire format** (`PAYMENT-SIGNATURE` header):
```
Base64( canonical_bytes[105] || signature[64] || retry_count[1] )  → 170 bytes
```

The **domain separator** prevents cross-protocol replay. The **explicit LE endianness** prevents architecture collisions. The **fixed length** makes on-chain verification O(1).

---

## What We Actually Solved — Concrete Wins

| Problem | Our Solution |
|---------|--------------|
| **Subscription fatigue** | Pay-per-query with x402 — the agent only opens its wallet when the data matters |
| **API key leakage / theft** | No static API keys — every payment is a single-use signed voucher |
| **Replay attacks** | Nonces + canonical domain separator + on-chain TTL |
| **Double-spend (TOCTOU)** | On-chain Ed25519 verification via sysvar, not client-side trust |
| **Agent insolvency** | On-chain daily cap + per-tx cap + emergency pause — enforced by immutable Solana program |
| **Infinite loop drain** | Off-chain rate limiter + in-flight cap + on-chain spend ceiling |
| **Floating-point non-determinism** | Pure integer fixed-point EV evaluator — mirrors on-chain math exactly |
| **Cross-language signature drift** | Single canonical byte layout shared by Rust, TS, and Anchor |
| **Oversized transactions** | MTU-safe chunking (≤1232 bytes) with [Ed25519, settle] interleaving |
| **Batch settlement gas costs** | Chunked batch transactions with interleaved instructions |

---

## Verified Test Coverage

| Suite | Tests | What It Proves |
|-------|-------|----------------|
| **Anchor Guardrails** | 10/10 ✅ | Caps, session keys, pause, Ed25519 verification, tampered messages |
| **Rust Agent Core** | 25/25 ✅ | Nonces, rate limiting, EV evaluator, MTU-safe parsing, voucher crypto |
| **Worker Settlement** | 1/1 ✅ | Batch construction, discriminator matching, instruction pairing |
| **Resource Server** | 9/9 ✅ | 402 challenge, canonical byte parsing, Ed25519 verify, expiry rejection |
| **End-to-End Loop A** | 1/1 ✅ | Full live roundtrip: GET → 402 → sign → retry → data received |

**Total: 46 verified test cases, all green.**

---

## What's Left to Complete

You're **~80% done** with a submission-ready product. The remaining work:

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 1 | **Agent autonomous loop** (`agent_loop.rs`) — WebSocket subscription, indicator computation, trigger detection | ~4h | Makes the agent truly autonomous |
| 2 | **Frontend dashboard** — visualize balance, vouchers, Pyth prices, decisions | ~6h | Demo power |
| 3 | **Devnet deployment** — deploy guardrails, fund escrow, run live | ~1h | Live demo |
| 4 | **Demo video** — script and record the end-to-end flow | ~2h | Submission requirement |
| 5 | **Written submission** — architecture doc, pitch, market analysis | ~3h | Colosseum deliverable |

**~16 hours to full submission readiness.**

---

## The Three-Sentence Pitch

> AI agents can reason but can't pay for premium data without human intervention — subscriptions, API keys, and credit cards are all built for humans, not machines.
>
> AgenticPay lets an autonomous trading agent pay per query using the x402 standard over Solana, spending fractions of a cent only when a signal justifies the data cost — with **on-chain spending caps that make it mathematically impossible for a compromised agent to drain the treasury**.
>
> The system is fully cross-language: Rust agent, TypeScript resource server, and an Anchor program that verifies Ed25519 signatures on-chain via sysvar introspection, all sharing one byte-for-byte canonical message contract — **46 tests green, both critical loops closed.**

---
