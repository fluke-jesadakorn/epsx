#!/bin/bash
set -e

# Configuration
RPC_URL="http://127.0.0.1:8545"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80" # Account 0

# Ensure we are in the contracts directory for forge commands
cd apps/contracts

MAINNET_USDT="0x55d398326f99059fF775485246999027B3197955"
MAINNET_USDC="0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d"

# Accounts to fund
ACCOUNT_0="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
ACCOUNT_1="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

echo "=== Etching Mainnet Tokens on Local Anvil ==="

# 1. Get Bytecode for Mock Contracts
echo "Compiling mocks..."
forge build --force

echo "Fetching runtime bytecode for MockMainnetUSDT..."
# Note: Use deployedBytecode (runtime bytecode)
USDT_CODE=$(forge inspect contracts/MockReplicas.sol:MockMainnetUSDT deployedBytecode)
USDC_CODE=$(forge inspect contracts/MockReplicas.sol:MockMainnetUSDC deployedBytecode)

# 2. Etch USDT
echo "Etching USDT code at $MAINNET_USDT..."
cast rpc anvil_setCode "$MAINNET_USDT" "$USDT_CODE" --rpc-url $RPC_URL

# 3. Etch USDC
echo "Etching USDC code at $MAINNET_USDC..."
cast rpc anvil_setCode "$MAINNET_USDC" "$USDC_CODE" --rpc-url $RPC_URL

# 4. Mint Tokens (Initialize State)
echo "Minting tokens..."
MINT_AMOUNT="100000000000000000000000" # 100,000 * 1e18

# Mint USDT
cast send $MAINNET_USDT "mint(address,uint256)" $ACCOUNT_0 $MINT_AMOUNT --private-key $PRIVATE_KEY --rpc-url $RPC_URL
cast send $MAINNET_USDT "mint(address,uint256)" $ACCOUNT_1 $MINT_AMOUNT --private-key $PRIVATE_KEY --rpc-url $RPC_URL

# Mint USDC
cast send $MAINNET_USDC "mint(address,uint256)" $ACCOUNT_0 $MINT_AMOUNT --private-key $PRIVATE_KEY --rpc-url $RPC_URL
cast send $MAINNET_USDC "mint(address,uint256)" $ACCOUNT_1 $MINT_AMOUNT --private-key $PRIVATE_KEY --rpc-url $RPC_URL

echo ""
echo "=== Token Etch Complete ==="
echo "USDT: $MAINNET_USDT"
echo "USDC: $MAINNET_USDC"
echo "Funded Accounts: $ACCOUNT_0, $ACCOUNT_1"
