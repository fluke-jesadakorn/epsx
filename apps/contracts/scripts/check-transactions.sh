#!/bin/bash

# Transaction Checker Script for EPSX
# Shows all blockchain transactions and wallet status

echo "🔍 EPSX BLOCKCHAIN TRANSACTION VIEWER"
echo "====================================="
echo ""

# Wallet and contract addresses
USER_WALLET="0x71bE63f3384f5fb98995898A86B02Fb2426c5788"
PAYMENT_ESCROW="0x5fbdb2315678afecb367f032d93f642f64180aa3"
USDT_TOKEN="0xe7f1725e7734ce288f8367e1bb143e90bb3f0512"
USDC_TOKEN="0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"

echo "📍 KEY ADDRESSES:"
echo "Your Wallet:     $USER_WALLET"
echo "Payment Escrow:  $PAYMENT_ESCROW"
echo "USDT Token:      $USDT_TOKEN"
echo "USDC Token:      $USDC_TOKEN"
echo ""

# Check if Hardhat node is running
echo "🔧 CHECKING HARDHAT NODE..."
if curl -s http://127.0.0.1:8545 > /dev/null; then
    echo "✅ Hardhat node is running on http://127.0.0.1:8545"
else
    echo "❌ Hardhat node is NOT running on http://127.0.0.1:8545"
    echo "Start with: cd apps/contracts && npx hardhat node"
    exit 1
fi

echo ""

# Get current block number
echo "📊 BLOCKCHAIN STATUS:"
BLOCK_NUM_HEX=$(curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://127.0.0.1:8545 | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
BLOCK_NUM=$((16#${BLOCK_NUM_HEX#0x}))
echo "Current Block: $BLOCK_NUM"

# Check ETH balance
BALANCE_HEX=$(curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$USER_WALLET\",\"latest\"],\"id\":1}" \
    http://127.0.0.1:8545 | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
BALANCE_ETH=$(echo "scale=6; $((16#${BALANCE_HEX#0x}))/1000000000000000000" | bc -l)
echo "Your ETH Balance: $BALANCE_ETH ETH"
echo ""

# Show recent transactions from the Hardhat logs
echo "📜 RECENT TRANSACTIONS FROM LOGS:"
echo "================================="

# Check if we can extract transactions from Hardhat logs
if [ -f /tmp/hardhat-transactions.log ]; then
    echo "Found transaction log:"
    cat /tmp/hardhat-transactions.log | tail -20
else
    echo "Checking Hardhat node output..."

    # Look for transactions in recent blocks
    for ((i=$BLOCK_NUM; i>$(($BLOCK_NUM-5)) && i>=0; i--)); do
        echo "Checking Block #$i:"

        # Get block with transactions
        BLOCK_DATA=$(curl -s -X POST -H "Content-Type: application/json" \
            --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"0x$(printf "%x" $i)\",true],\"id\":1}" \
            http://127.0.0.1:8545)

        # Extract transactions
        TX_COUNT=$(echo "$BLOCK_DATA" | grep -o '"transactions":\[.*\]' | grep -o '{' | wc -l)
        if [ "$TX_COUNT" -gt 0 ]; then
            echo "  Found $TX_COUNT transaction(s):"

            # Extract transaction hashes
            echo "$BLOCK_DATA" | grep -o '"hash":"[^"]*"' | while read tx_hash; do
                HASH=$(echo "$tx_hash" | cut -d'"' -f4)
                echo "    - Transaction: $HASH"

                # Get transaction details
                TX_DATA=$(curl -s -X POST -H "Content-Type: application/json" \
                    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionByHash\",\"params\":[\"$HASH\"],\"id\":1}" \
                    http://127.0.0.1:8545)

                FROM=$(echo "$TX_DATA" | grep -o '"from":"[^"]*"' | cut -d'"' -f4)
                TO=$(echo "$TX_DATA" | grep -o '"to":"[^"]*"' | cut -d'"' -f4)
                VALUE=$(echo "$TX_DATA" | grep -o '"value":"[^"]*"' | cut -d'"' -f4)

                if [ -n "$FROM" ]; then
                    # Check if it's our transaction
                    if [[ "${FROM,,}" == "${USER_WALLET,,}" ]]; then
                        echo "      🎯 YOUR TRANSACTION!"
                    fi

                    # Convert value to ETH
                    if [ -n "$VALUE" ] && [ "$VALUE" != "0x0" ]; then
                        VALUE_ETH=$(echo "scale=6; $((16#${VALUE#0x}))/1000000000000000000" | bc -l)
                        echo "      From: $FROM"
                        echo "      To: $TO"
                        echo "      Value: $VALUE_ETH ETH"
                    else
                        echo "      From: $FROM"
                        echo "      To: $TO (Token transaction)"
                    fi
                fi
            done
        fi
    done
fi

echo ""
echo "🔗 USEFUL LINKS:"
echo "- Hardhat Node: http://127.0.0.1:8545"
echo "- Your Wallet: http://127.0.0.1:8545/address/$USER_WALLET"
echo "- Payment Contract: http://127.0.0.1:8545/address/$PAYMENT_ESCROW"
echo "- USDT Token: http://127.0.0.1:8545/address/$USDT_TOKEN"
echo "- USDC Token: http://127.0.0.1:8545/address/$USDC_TOKEN"

echo ""
echo "💡 TIPS TO SEE TRANSACTIONS:"
echo "1. Use MetaMask and add Hardhat Network:"
echo "   - Network Name: Hardhat Localhost"
echo "   - RPC URL: http://127.0.0.1:8545"
echo "   - Chain ID: 31337"
echo "   - Currency Symbol: ETH"
echo ""
echo "2. In MetaMask, import your account with private key:"
echo "   Private Key: 0x701b615bbdfb9de65240bc28bd21bbc0d996645a3dd57e7b12bc2bdf6f192c82"
echo ""
echo "3. Watch the Hardhat console for real-time transaction logs"
echo "4. Use this script to check balances and recent transactions"