#!/bin/bash
set -e

# Always run from the repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

DATE=$(date +%Y-%m-%d)
OUTPUT_DIR="benchmarks/results"
mkdir -p "$OUTPUT_DIR"

# ------------------------------------------------------------------
# 1. State microbenchmarks (Criterion)
# ------------------------------------------------------------------
if [ -f "agent-backend/benches/state_machine.rs" ]; then
    echo "📊 Running state microbenchmarks..."
    cd agent-backend
    cargo bench --bench state_machine -- --save-baseline "$DATE"
    cd "$ROOT_DIR"
else
    echo "⚠️  State microbenchmark not found, skipping."
fi

# ------------------------------------------------------------------
# 2. RPC load test (getBalance) – using oha
# ------------------------------------------------------------------
if command -v oha &> /dev/null; then
    echo "🌐 Running getBalance load test..."
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:8545 | grep -q "200\|400\|405"; then
        echo "Server is running. Starting oha..."
        oha -n 0 \
            -c 100 \
            -z 30s \
            -q 1000 \
            -m POST \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"agent_getBalance","params":["7Ec9tK1aP5A4pfnCbP1hJbX9qM7C2p8ZyzpXqgT5fLkK"],"id":1}' \
            --format json \
            "http://localhost:8545" > "$OUTPUT_DIR/oha_getBalance_$DATE.json" 2>&1

        if command -v jq &> /dev/null && [ -f "$OUTPUT_DIR/oha_getBalance_$DATE.json" ]; then
            SUCCESS_RATE=$(jq -r '.summary.success_rate // 0' "$OUTPUT_DIR/oha_getBalance_$DATE.json")
            AVG_LATENCY=$(jq -r '.summary.average_response_time_ms // 0' "$OUTPUT_DIR/oha_getBalance_$DATE.json")
            RPS=$(jq -r '.summary.requests_per_second // 0' "$OUTPUT_DIR/oha_getBalance_$DATE.json")
            echo "$DATE,getBalance,$SUCCESS_RATE,$AVG_LATENCY,$RPS" >> "$OUTPUT_DIR/history.csv"
            echo "✅ getBalance results recorded."
        else
            echo "⚠️  Failed to parse oha output."
        fi
    else
        echo "⚠️  RPC server not running on http://localhost:8545. Skipping getBalance load test."
    fi
else
    echo "⚠️  oha not installed. Run 'cargo install oha' to enable getBalance benchmarking."
fi

# ------------------------------------------------------------------
# 3. RPC load test (submitTransaction)
# ------------------------------------------------------------------
echo "💸 Running submitTransaction load test..."
cd agent-backend
cargo run --bin loadtest_submit -- --concurrency 50 --requests 1000
cd "$ROOT_DIR"

# The binary writes to benchmarks/results/latest_submit_load.json
# (relative to where it was run, which is agent-backend/).
# Check both possible locations.
SUBMIT_JSON=""
if [ -f "agent-backend/benchmarks/results/latest_submit_load.json" ]; then
    SUBMIT_JSON="agent-backend/benchmarks/results/latest_submit_load.json"
elif [ -f "benchmarks/results/latest_submit_load.json" ]; then
    SUBMIT_JSON="benchmarks/results/latest_submit_load.json"
fi

if [ -n "$SUBMIT_JSON" ]; then
    # Copy to dated file in the root results directory
    cp "$SUBMIT_JSON" "$OUTPUT_DIR/submit_$DATE.json"
    if command -v jq &> /dev/null; then
        RPS=$(jq -r '.throughput_rps // 0' "$SUBMIT_JSON")
        P50=$(jq -r '.latency_p50_ms // 0' "$SUBMIT_JSON")
        P95=$(jq -r '.latency_p95_ms // 0' "$SUBMIT_JSON")
        P99=$(jq -r '.latency_p99_ms // 0' "$SUBMIT_JSON")
        ERRORS=$(jq -r '.errors // 0' "$SUBMIT_JSON")
        echo "$DATE,submitTransaction,$RPS,$P50,$P95,$P99,$ERRORS" >> "$OUTPUT_DIR/history.csv"
        echo "✅ submitTransaction results recorded."
    else
        echo "⚠️  jq not installed; cannot parse submitTransaction results."
    fi
else
    echo "⚠️  submitTransaction results file not found."
fi

echo ""
echo "✅ Daily benchmarks complete! Results stored in $OUTPUT_DIR/history.csv"