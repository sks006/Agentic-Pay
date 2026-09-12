# Walkthrough: Mock Resource Server (B1) & x402 HTTP Client (B2)

We have implemented both the **Mock Resource Server** (TypeScript) and the **x402 HTTP Client** (Rust), completing **Loop A (Data Acquisition)** end-to-end. The agent autonomously queries protected data, challenges an HTTP 402, signs an off-chain voucher with its wallet, and receives Pyth oracle data.

---

## 1. Resource Server (Node/TypeScript)

### Cryptographic Interoperability & Voucher Engine
- [**`mock-resource-server/src/voucher.ts`**](file:///home/seam/Desktop/project/Agentic-Pay/mock-resource-server/src/voucher.ts):
  - Canonical byte layout serializer matching Rust's `agent-backend/src/voucher.rs` and Anchor's `programs/agenticpay-guardrails` exactly:
    - `0..17`: Domain separator (`agenticpay_x402v1`)
    - `17..25`: Nonce (`u64` little-endian)
    - `25..57`: Agent Solana public key (32 bytes)
    - `57..89`: Provider Solana public key (32 bytes)
    - `89..97`: Amount in lamports (`u64` little-endian)
    - `97..105`: Expiry timestamp (`i64` little-endian)
  - Full 170-byte header format (`105 bytes canonical + 64 bytes Ed25519 signature + 1 byte retry_count`), Base64 encoded for `PAYMENT-SIGNATURE`.
  - Signature verification using `tweetnacl.sign.detached.verify` and expiry validation.

### x402 Paywall Middleware
- [**`mock-resource-server/src/paywall.ts`**](file:///home/seam/Desktop/project/Agentic-Pay/mock-resource-server/src/paywall.ts):
  - Emits `HTTP 402 Payment Required` with `PAYMENT-REQUIRED` header containing payment parameters (recipient pubkey, price, network, TTL, scheme).
  - Inspects `PAYMENT-SIGNATURE` header, verifies recipient match, lamport amount threshold, expiry time, and Ed25519 signature.
  - Injects verified `voucher` and `agentPubkey` into Express request context upon successful authorization.

### Pyth Hermes Service with Caching
- [**`mock-resource-server/src/pyth.ts`**](file:///home/seam/Desktop/project/Agentic-Pay/mock-resource-server/src/pyth.ts):
  - Integrates `@pythnetwork/hermes-client` for real-time price feeds (`BTC/USD`, `ETH/USD`, `SOL/USD`).
  - LRU caching with configurable TTL (default 5s) to eliminate redundant requests.
  - Resilient simulated fallback for offline testing or rate-limited environments.

---

## 2. Agent Wallet & x402 HTTP Client (Rust)

### Agent Cryptographic Wallet
- [**`agent-backend/src/wallet.rs`**](file:///home/seam/Desktop/project/Agentic-Pay/agent-backend/src/wallet.rs):
  - `AgentWallet` with Ed25519 key management using `ed25519-dalek` 2.x and Solana SDK.
  - `from_hex_secret`: supports 32/64-byte hex and Base58 secret keys (e.g. `AGENT_SECRET_KEY`).
  - `signing_key()`: exposes `&SigningKey` for canonical voucher signing.
  - `pubkey_bytes()`, `pubkey()`, and `keypair()` helpers.

### x402 HTTP Client
- [**`agent-backend/src/client.rs`**](file:///home/seam/Desktop/project/Agentic-Pay/agent-backend/src/client.rs):
  - `X402Client::new(base_url, wallet)`: initializes HTTP client with atomic nonce generation.
  - `fetch_price(symbol)`:
    1. Sends unauthenticated `GET /price/:symbol`.
    2. Intercepts `402 Payment Required` and parses `payment_required` terms.
    3. Signs a canonical voucher for provider terms using `AgentWallet`.
    4. Retries request with `PAYMENT-SIGNATURE` (Base64 voucher) and `PAYMENT-SCHEME`.
    5. Parses response into `PythPriceFeed` for `decision.rs`.

---

## 3. Verification & Test Results

### Unit & Integration Tests (Agent Backend)
```
running 25 tests
test decision::tests::test_confidence_bps_overflow_safe ... ok
test decision::tests::test_evaluate_confidence_too_wide ... ok
test decision::tests::test_evaluate_not_profitable ... ok
test decision::tests::test_evaluate_fee_too_high ... ok
test decision::tests::test_evaluate_profitable ... ok
test decision::tests::test_fee_bps_overflow_safe ... ok
test solana_pay::tests::test_decode_ok ... ok
test solana_pay::tests::test_size_exceeded ... ok
test solana_pay::tests::test_validate_ok ... ok
test solana_pay::tests::test_validate_trailing_bytes ... ok
test voucher::tests::test_canonical_bytes_deterministic ... ok
test client::tests::test_encode_voucher_layout ... ok
test voucher::tests::test_expiry ... ok
test voucher::tests::test_sign_and_verify_ok ... ok
test voucher::tests::test_header_roundtrip ... ok
test worker::tests::test_batch_settle_instruction_data_layout ... ok
test worker::tests::test_chunking_respects_mtu ... ok
test worker::tests::test_discriminator_is_correct ... ok
test worker::tests::test_recovery_failure_revert_and_terminal_failed ... ok
test worker::tests::test_recovery_success_finalized ... ok
test worker::tests::test_extraction_lock_prevents_duplicate_sweep ... ok
test voucher::tests::test_verify_tampered_payload ... ok
test worker::tests::test_instruction_pairing ... ok
test state::tests::test_guard_drop_on_panic ... ok
test state::tests::test_acquire_and_commit ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Full Loop A Cross-Language End-to-End Test
With `mock-resource-server` running on port 8080:
```bash
cargo test --test client_test test_fetch_price_with_x402 -- --ignored --nocapture
```
Output:
```
running 1 test
test test_fetch_price_with_x402 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.95s
```

Log trace during execution:
1. `GET /price/BTC` -> `402 Payment Required` (`PAYMENT-REQUIRED: {"price":1000,...}`)
2. Rust agent parses terms, signs 105-byte canonical voucher, generates 170-byte Base64 header.
3. `GET /price/BTC` with `PAYMENT-SIGNATURE` -> Resource server verifies Ed25519 signature and returns `200 OK` with Pyth price data.
4. Rust client parses `PythPriceFeed` with `price > 0`.
