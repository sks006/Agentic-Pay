# AgenticPay

**Autonomous, on-chain-guarded micropayments for AI agents.**
An agent needing real-time Pyth market data pays per query — no subscriptions, no API keys, no human in the loop — and it is *physically incapable* of overspending, because the spending limit lives in an on-chain program, not in the agent's own logic.

Built on Solana. Built for [Colosseum's Crypto World's Fair](https://colosseum.com/worldsfair) (Solana track).

---

## The problem

AI agents can reason, plan, and act — but the moment one needs a paid data feed, it hits infrastructure built for humans: sign up, get an API key, attach a credit card, wait for a monthly cycle to reset. That model doesn't fit software making decisions at machine speed, and it doesn't fit a bot that only needs *one* expensive data pull today.

`x402` — the open, Coinbase-originated, now Solana Foundation / Cloudflare / Stripe / Visa–backed HTTP 402 payment standard — solves the "how do I pay per request" half of this. It doesn't solve the other half: **what stops the agent's own logic from spending more than it should?**

## The solution

AgenticPay is an event-driven data-procurement loop for trading agents, with spending safety enforced in two independent places — one in the client, one on-chain — so a bug or a poisoned signal in the agent's decision logic can never translate into unbounded spend.

```
 Free baseline feed  →  Trigger fires  →  Payment settlement  →  Decision & execution
 (WebSocket, $0 cost)   (volatility,       (x402 + Solana Pay,    (Jupiter swap if
                         liquidation,       instant or deferred)   signal still holds)
                         divergence)               │
                                                    ├──► checked against on-chain
                                                    │    spend caps (Anchor program)
                                                    ▼
                                          Immutable audit log
                                     (every trigger, payment, trade)
```

The agent runs a free public feed (Binance / Pyth Hermes WebSocket) continuously and pays for nothing most of the time. It only opens its wallet when a detected signal — a volatility breakout, a liquidation cascade, a statistical divergence — justifies the cost of pulling deeper, priced data (a Pyth price feed pull, an L3 depth snapshot, a historical window). That's the whole economic model: **spend $0 by default, spend a fraction of a cent when the expected value of the data clears the bar.**

## Why this, not raw tick-by-tick payment

Solana's ~400ms finality plus an HTTP 402 round-trip is not fast enough to compete with real HFT arbitrage — paying per tick to front-run a spread is a losing pitch. AgenticPay doesn't claim that. It's positioned for the horizon where a few hundred milliseconds of settlement latency doesn't matter: confirming a signal before acting on it, not racing someone else to a price.

## Settlement: two modes, one guardrail

| | Instant (on-chain) | Deferred (session voucher) |
|---|---|---|
| **Flow** | 402 → Solana Pay transaction request → sign → confirm → data released | Agent signs a session-key-scoped voucher → resource server verifies locally → data released instantly → vouchers batched to on-chain settlement later |
| **Latency** | ~400ms (network finality) + HTTP round-trip | Sub-20ms (local signature check) |
| **Best for** | Low-urgency pulls: rebalancing, sentiment digests, periodic context | Time-sensitive triggers: liquidation confirmation, fast trade gating |
| **Trust model** | Zero counterparty risk — atomic exchange of value for data | Bounded credit — the session key's on-chain allowance caps total exposure |

`payment.rs` picks the mode per trigger. Both paths ultimately settle through the same Anchor program, so neither can exceed the account's spend ceiling.

## Guardrails: two independent rings

1. **Off-chain (`state.rs`)** — sliding-window rate limits, in-flight request caps, nonce/idempotency to kill duplicate payments before they touch the network.
2. **On-chain (`programs/agenticpay-guardrails`)** — hard daily/overall spend caps, max-per-transaction limits, session-key accounts with their own bounded allowance, emergency pause. This is the ring that holds even if the decision engine itself is compromised or poisoned — it cannot be bypassed by anything running off-chain.

Every trigger, payment, and trade decision is written to `audit.rs`, an append-only log, regardless of which settlement path was used.

## Tech stack

- **Agent (`agent-backend/`)** — Rust. Owns triggering, settlement-mode selection, the x402/pay.sh HTTP flow, Solana Pay transaction-request construction, and voucher signing.
- **On-chain program (`programs/agenticpay-guardrails/`)** — Anchor. Owns the hard spend ceiling, per-transaction limits, and session-key accounts. This is the part no amount of off-chain logic can override.
- **Resource server (`mock-resource-server/`)** — Node/TypeScript. Serves real Pyth Hermes price data behind an x402 paywall; issues Solana Pay transaction requests or verifies session vouchers depending on the request.
- **Frontend (`frontend/`)** — React/Vite dashboard for the demo: live payment log, agent budget remaining, price chart, and the agent's reasoning for each decision.

## Repo structure

```
agenticpay/
├── README.md
├── .env.example                       # + PYTH_API_KEY, SOLANA_RPC_URL, FACILITATOR_URL, FRONTEND_URL
├── docker-compose.yml
│
├── docs/
│   ├── architecture.md
│   ├── pitch.md
│   ├── demo-script.md
│   └── market.md
│
├── agent-backend/                     # Rust: the autonomous agent
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── wallet.rs
│   │   ├── decision.rs                # trigger logic: volatility, liquidation, divergence
│   │   ├── payment.rs                 # picks instant vs deferred settlement per trigger
│   │   ├── client.rs                  # x402 / pay.sh HTTP round-trip
│   │   ├── solana_pay.rs              # ★ NEW – builds/parses Solana Pay transaction requests
│   │   ├── voucher.rs                 # ★ NEW – signs session-key intent-to-pay vouchers, batches for settlement
│   │   ├── rpc.rs
│   │   ├── types.rs
│   │   ├── error.rs
│   │   ├── state.rs                   # rate limits, in-flight caps, nonce/idempotency
│   │   ├── audit.rs                   # immutable decision log
│   │   └── redact.rs                  # PII / metadata sanitization (stretch)
│   └── tests/
│       ├── state_tests.rs
│       ├── audit_tests.rs
│       ├── voucher_tests.rs           # ★ NEW
│       └── integration_test.rs
│
├── programs/                          # ★ NEW – the on-chain half of the guardrails
│   └── agenticpay-guardrails/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs                 # hard spend caps, per-tx limits, session-key accounts, batch settlement
│   ├── Anchor.toml
│   ├── migrations/deploy.ts
│   └── tests/guardrails.ts            # Anchor/TS integration tests
│
├── mock-resource-server/              # Serves real Pyth data behind the paywall
│   ├── package.json
│   ├── src/
│   │   ├── server.ts
│   │   ├── pyth.ts                    # Hermes client + caching
│   │   ├── paywall.ts                 # x402 middleware, builds Solana Pay requests
│   │   ├── routes/{price.ts,health.ts}
│   │   └── webhook.ts
│   ├── pay-demo.yml
│   └── README.md
│
├── frontend/                          # React/Vite demo dashboard
│   ├── package.json / vite.config.ts / index.html
│   └── src/
│       ├── main.tsx / App.tsx
│       ├── components/{AgentStatus,PaymentLog,PriceChart,DecisionPanel,LiveFeed,WalletConnect}.tsx
│       ├── hooks/{useAgentRpc,usePythProxy}.ts
│       └── lib/types.ts
│
└── scripts/
    ├── setup.sh
    ├── fund-agent.sh
    ├── run-demo.sh
    ├── get-pyth-key.sh
    └── deploy-guardrails.sh           # ★ NEW – builds & deploys the Anchor program to devnet
```

## Getting started

```bash
git clone https://github.com/sks006/Agentic-Pay.git
cd Agentic-Pay
cp .env.example .env          # add PYTH_API_KEY, SOLANA_RPC_URL, FACILITATOR_URL

./scripts/setup.sh            # installs Rust + Node deps, builds the Anchor program
./scripts/get-pyth-key.sh     # helper to fetch a Hermes API key if you don't have one
./scripts/deploy-guardrails.sh --network devnet
./scripts/fund-agent.sh        # airdrops devnet SOL + USDC to the agent's session key
./scripts/run-demo.sh          # starts agent-backend + mock-resource-server + frontend
```

Then open `http://localhost:5173` to watch the agent monitor the free feed, fire a trigger, pay for premium Pyth data, and decide.

## Demo flow

1. Baseline loop streams live Pyth Hermes prices — no payments yet.
2. A synthetic trigger fires (volatility spike).
3. Agent requests the gated deep-data endpoint → gets a 402 → builds and signs a Solana Pay transaction (or a session voucher, depending on urgency) → data is released.
4. `decision.rs` evaluates the paid-for data against the remaining on-chain allowance.
5. If the signal holds, a swap is routed through Jupiter.
6. Every step above appears in the dashboard's live feed and the immutable audit log.

## Roadmap

- [ ] Session-key scoping: per-strategy vs per-agent-instance (open design decision — affects the Anchor account structure)
- [ ] Batch settlement via netted claims against the escrow account
- [ ] Mainnet-beta deployment of `agenticpay-guardrails`
- [ ] Additional premium data sources beyond Pyth (order-book depth, liquidation heatmaps)

## Why Solana

x402 already processes the majority of its global transaction volume on Solana — sub-second finality and effectively-zero fees are what make sub-cent, per-query payments economically viable in the first place. AgenticPay leans into that rather than competing with it: the payment rail is standard infrastructure, the contribution here is the on-chain-enforced safety layer on top of it.

## License

Apache License, Version 2.0 