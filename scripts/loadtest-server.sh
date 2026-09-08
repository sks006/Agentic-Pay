#!/bin/bash
set -e

# Configuration
HOST="http://localhost:8545"
CONNECTIONS=100
DURATION="30s"
RATE=1000  # requests per second (adjust to avoid overwhelming your machine)

# Test payload for getBalance (lightweight, no state mutation)
PAYLOAD='{"jsonrpc":"2.0","method":"agent_getBalance","params":["7Ec9tK1aP5A4pfnCbP1hJbX9qM7C2p8ZyzpXqgT5fLkK"],"id":1}'

# Ensure results directory exists
mkdir -p benchmarks/results

echo "🚀 Starting load test against $HOST"
echo "Connections: $CONNECTIONS, Duration: $DURATION, Rate: $RATE req/s"

# Run oha and save results
oha -n 0 \
     -c $CONNECTIONS \
     -z $DURATION \
     -q $RATE \
     -m POST \
     -H "Content-Type: application/json" \
     -d "$PAYLOAD" \
     --json \
     "$HOST" > benchmarks/results/latest_load_test.json

echo "✅ Load test complete. Results saved to benchmarks/results/latest_load_test.json"

# Print summary
if command -v jq &> /dev/null; then
    cat benchmarks/results/latest_load_test.json | jq '.summary'
else
    cat benchmarks/results/latest_load_test.json
fi
