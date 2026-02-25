# USDT Refund Manual

## Wallet Addresses

| Role | Address |
|------|---------|
| System Receiver | `0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF` |
| Escrow Contract | `0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0` |
| BSC USDT Token | `0x55d398326f99059fF775485246999027B3197955` |

## How Payment Works

User pays plan -> ERC20 `transfer()` -> System Receiver wallet (`0xea64...`)

This is a direct token transfer, NOT through the escrow contract.

## Refund Steps (via BscScan)

### Prerequisites
- MetaMask with the System Receiver private key (`0xea64...`) imported
- Small BNB balance in `0xea64...` for gas (~0.001 BNB)

### Steps

1. Open MetaMask, switch to account `0xea64...`
2. Make sure network is **BSC Mainnet** (Chain ID: 56)
3. Go to: `https://bscscan.com/token/0x55d398326f99059fF775485246999027B3197955#writeContract`
4. Click **"Connect to Web3"** -> connect MetaMask (`0xea64...`)
5. Find function **`transfer`** and fill:
   - **recipient (address):** the user's wallet address to refund
   - **amount (uint256):** amount in 18 decimals (see table below)
6. Click **Write** -> confirm in MetaMask

### Amount Reference

| USDT | uint256 value |
|------|---------------|
| 1 | `1000000000000000000` |
| 5 | `5000000000000000000` |
| 10 | `10000000000000000000` |
| 50 | `50000000000000000000` |
| 100 | `100000000000000000000` |

Formula: `amount * 10^18`

## Check Balances via RPC

### Check USDT balance of any wallet
```bash
curl -s -X POST https://bsc-dataseed1.binance.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_call","params":[{"to":"0x55d398326f99059fF775485246999027B3197955","data":"0x70a08231000000000000000000000000<ADDRESS_WITHOUT_0x>"},"latest"],"id":1}'
```

### Check BNB balance (for gas)
```bash
curl -s -X POST https://bsc-dataseed1.binance.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["<FULL_ADDRESS>","latest"],"id":1}'
```

### Decode hex result to decimal
```bash
python3 -c "print(int('<HEX_RESULT>', 16) / 10**18)"
```

## Verify Transaction

After refund, check tx on: `https://bscscan.com/tx/<TX_HASH>`
