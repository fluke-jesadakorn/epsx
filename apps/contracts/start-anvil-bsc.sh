#!/bin/bash

# BSC RPC Nodes to try
# Use the official Ankr node as primary because it usually has better archival support for free tiers
RPC_URLS=(
    "https://bsc.publicnode.com"
    "https://binance.ankr.com/bsc"
    "https://bscrpc.com"
    "https://bsc-dataseed1.binance.org/"
)

# Check for cast
if ! command -v cast &> /dev/null;
then
    echo "Error: 'cast' is not installed. Please install Foundry."
    exit 1
fi

# Check if port 8545 is in use and kill it
if lsof -i :8545 -t >/dev/null 2>&1; then
    echo "⚠️  Port 8545 is in use. Killing existing Anvil process..."
    lsof -i :8545 -t | xargs kill -9
    sleep 1
    echo "✅ Port 8545 freed."
fi

# Find a working RPC and latest block
SELECTED_RPC=""
LATEST_BLOCK=""

echo "🔍 Finding best BSC RPC node..."

for URL in "${RPC_URLS[@]}"; do
    echo "Testing $URL..."
    
    # Use cast first, but show error if it fails
    BLOCK=$(cast block-number --rpc-url "$URL" --timeout 5000 2>/dev/null)
    
    if [ ! -z "$BLOCK" ] && [ "$BLOCK" -gt 0 ]; then
        SELECTED_RPC="$URL"
        LATEST_BLOCK="$BLOCK"
        echo "✅ Found valid block $LATEST_BLOCK on $SELECTED_RPC"
        break
    else
        # Fallback to curl
        echo "   (cast failed, trying curl...)"
        CURL_RES=$(curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' "$URL")
        
        if [[ $CURL_RES == *"result"* ]]; then
            HEX_BLOCK=$(echo $CURL_RES | sed -n 's/.*"result":"\([^"].*\)".*/\1/p')
            if [ ! -z "$HEX_BLOCK" ]; then
                BLOCK=$((HEX_BLOCK))
                if [ "$BLOCK" -gt 0 ]; then
                    SELECTED_RPC="$URL"
                    LATEST_BLOCK="$BLOCK"
                    echo "✅ Found valid block $LATEST_BLOCK on $SELECTED_RPC (via curl)"
                    break
                fi
            fi
        fi
    fi
done

if [ -z "$SELECTED_RPC" ]; then
    echo "❌ Could not connect to any BSC RPC node."
    exit 1
fi

# Use a very recent block (15 blocks behind) for official Binance nodes
# But deeper for public nodes if possible.
# Given the errors, staying close to tip might actually be safer for non-archival nodes
FORK_BLOCK=$((LATEST_BLOCK - 15))

echo "🚀 Starting Anvil..."
echo "   Network: BSC Mainnet (Chain ID 56)"
echo "   RPC: $SELECTED_RPC"
echo "   Fork Block: $FORK_BLOCK"

# We add --compute-units-per-second to slow down requests and avoid rate limits
# We REMOVE --no-rate-limit to be nicer to the public RPC
anvil \
    --fork-url "$SELECTED_RPC" \
    --fork-block-number "$FORK_BLOCK" \
    --chain-id 56 \
    --host 0.0.0.0 \
    --gas-price 3000000000 \
    --code-size-limit 50000 \
    --compute-units-per-second 100 \
    --timeout 30000 \
    --retries 5