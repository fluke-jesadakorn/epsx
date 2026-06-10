#!/bin/bash
# Fund test account with USDT by directly setting storage
# Works even when the RPC fork has indexing issues

# USDT contract on BSC  
USDT_ADDRESS="0x55d398326f99059fF775485246999027B3197955"

# Your test account (Anvil account #0)
TEST_ACCOUNT="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

# Amount: 10000 USDT (18 decimals) in hex
# 10000 * 10^18 = 10000000000000000000000 = 0x21e19e0c9bab2400000
AMOUNT="0x00000000000000000000000000000000000000000021e19e0c9bab2400000000"

echo "🏦 Funding test account with USDT..."
echo "   Account: $TEST_ACCOUNT"
echo "   Amount: 10,000 USDT"
echo ""

# For BSC USDT (standard ERC20), balances are stored at:
# keccak256(abi.encode(address, uint256(1))) where 1 is the balanceOf mapping slot
# Pre-computed slot for account #0:
# keccak256(abi.encodePacked(bytes32(address), bytes32(1)))

# Calculate storage slot for balanceOf[TEST_ACCOUNT]
# The balanceOf mapping is at slot 0 for BSC USDT
SLOT=$(cast index address $TEST_ACCOUNT 0)

echo "Storage slot: $SLOT"

# Set the storage directly
RESULT=$(curl -s -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\":\"2.0\",
    \"method\":\"anvil_setStorageAt\",
    \"params\":[\"$USDT_ADDRESS\", \"$SLOT\", \"$AMOUNT\"],
    \"id\":1
  }")

echo "Result: $RESULT"

# Verify the balance
echo ""
echo "Verifying balance..."
BALANCE=$(cast call $USDT_ADDRESS "balanceOf(address)(uint256)" $TEST_ACCOUNT --rpc-url http://127.0.0.1:8545 2>/dev/null)
echo "Balance: $BALANCE"

echo ""
echo "✅ Done! Your test account should now have 10,000 USDT."
echo "   Refresh MetaMask to see the balance."
