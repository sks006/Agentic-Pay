# Agentic-Pay
AgenticPay is a Solana-powered autonomous payment system that lets AI agents discover, decide, and pay for real-time financial data (via Pyth Network) on a per-query basis using the x402 / pay.sh protocol.


```
agentpay/
├── README.md
├── .env.example                       # + PYTH_API_KEY, FRONTEND_URL, etc.
├── docker-compose.yml
│
├── docs/
│   ├── architecture.md
│   ├── pitch.md
│   ├── demo-script.md
│   └── market.md
│
agent-backend/
│    ├── Cargo.toml
│    ├── src/
│    │   ├── main.rs
│    │   ├── config.rs
│    │   ├── wallet.rs
│    │   ├── decision.rs
│    │   ├── payment.rs
│    │   ├── client.rs                      # HTTP client that talks to pay.sh / 402 flow
│    │   ├── rpc.rs                         # JSON-RPC / WebSocket API for CLI + frontend
│    │   ├── types.rs
│    │   ├── error.rs
│    │   │
│    │   ├── state.rs                       # ★ NEW – Idempotency & Nonce Management
│    │   ├── audit.rs                       # ★ NEW – Immutable Decision Logging
│    │   └── redact.rs                      # ★ NEW – PII / Metadata Sanitization (stretch)
│    │
│    └── tests/
│        ├── state_tests.rs
│        ├── audit_tests.rs
│        └── integration_test.rs
│
├── mock-resource-server/              # Now serves REAL Pyth data + paywall
│   ├── package.json                   # Recommended: Node + Express or Fastify
│   ├── src/
│   │   ├── server.ts
│   │   ├── pyth.ts                    # Hermes client + caching
│   │   ├── paywall.ts                 # pay.sh / x402 middleware
│   │   ├── routes/
│   │   │   ├── price.ts               # GET /price/:symbol  (gated)
│   │   │   └── health.ts
│   │   └── webhook.ts
│   ├── pay-demo.yml                   # or pay server config
│   └── README.md
│
├── frontend/                          # ★ New – React/Vite or Next.js
│   ├── package.json
│   ├── vite.config.ts (or next.config)
│   ├── index.html
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── AgentStatus.tsx        # Budget remaining, wallet address
│   │   │   ├── PaymentLog.tsx         # Live list of micropayments
│   │   │   ├── PriceChart.tsx         # Real-time price from agent decisions
│   │   │   ├── DecisionPanel.tsx      # "Should I buy X?" + reasoning
│   │   │   ├── LiveFeed.tsx           # Streaming activity
│   │   │   └── WalletConnect.tsx      # Optional (or just show agent pubkey)
│   │   ├── hooks/
│   │   │   ├── useAgentRpc.ts         # Connect to Rust backend
│   │   │   └── usePythProxy.ts
│   │   ├── lib/
│   │   │   └── types.ts
│   │   └── styles/
│   ├── public/
│   └── README.md
│
└── scripts/
    ├── setup.sh
    ├── fund-agent.sh
    ├── run-demo.sh                    # Starts backend + mock + frontend
    └── get-pyth-key.sh                # Helper for API key

```