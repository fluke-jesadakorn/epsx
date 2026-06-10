# Fixing Web3 Connection Issues

## Problem
JSON-RPC Parse Error: `{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error: Unexpected end of JSON input","data":{"message":"Parse error: Unexpected end of JSON input"}}}`

## Root Cause
The Hardhat node is working perfectly. The error is caused by empty/invalid JSON requests being sent from the frontend Web3 client.

## Solutions

### 1. Quick Check: Browser Network Tab
1. Open browser dev tools (F12)
2. Go to Network tab
3. Filter by `127.0.0.1:8545`
4. Look for failed requests with empty bodies

### 2. MetaMask Configuration (Most Likely Fix)
```json
Network Name: Hardhat Local
RPC URL: http://127.0.0.1:8545
Chain ID: 31337
Currency Symbol: ETH
```

### 3. Frontend Troubleshooting

Check these files for RPC configuration issues:

**AuthProvider.tsx** - Wagmi configuration:
- Ensure RPC URLs are correct
- Check for duplicate provider initialization

**Payment Components** - Web3 calls:
- Verify all contract calls have proper parameters
- Check for undefined/null values in transactions

### 4. Debug Steps

1. **Clear browser cache and localStorage**
2. **Disable browser extensions temporarily**
3. **Restart frontend and Hardhat node**
4. **Test with a fresh browser profile**

### 5. Common Fixes

#### RPC Transport Configuration
```typescript
// In wagmi config, ensure proper transport
const config = getDefaultConfig({
  chains: [hardhatLocalhost],
  transports: {
    [hardhatLocalhost.id]: http('http://127.0.0.1:8545'),
  },
});
```

#### Contract Interaction
```typescript
// Ensure all contract calls have proper parameters
const { writeContract } = useWriteContract();

writeContract({
  address: '0x5fbdb2315678afecb367f032d93f642f64180aa3',
  abi: PaymentEscrowABI,
  functionName: 'payForPlan',
  args: [planId, tokenAddress, amount], // Ensure no undefined values
});
```

## Testing Your Connection

1. **Run the transaction checker**:
   ```bash
   cd apps/contracts
   ./scripts/check-transactions.sh
   ```

2. **Test Web3 connection**:
   ```bash
   node scripts/test-web3-connection.js
   ```

3. **Check your transactions in MetaMask**:
   - All successful transactions: ✅
   - Failed payment attempt: ❌ (due to contract validation, not blockchain)

## Status

- ✅ Hardhat node: Working perfectly
- ✅ Contract deployment: Successful
- ✅ Token minting: Successful
- ✅ Your wallet: 100 USDT + 100 USDC
- ❌ Web3 connection: Malformed requests (frontend issue)

The blockchain infrastructure is ready. The issue is purely on the frontend Web3 configuration side.