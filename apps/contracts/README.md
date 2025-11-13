# EPSX Payment Escrow Smart Contracts

BSC smart contracts for handling subscription payments on the EPSX platform.

## 📦 Contracts

### PaymentEscrow.sol
Main payment escrow contract that:
- Accepts USDT/USDC payments for subscription plans
- Emits events for backend verification
- Manages plan pricing
- Enables admin fund withdrawal
- Provides emergency pause functionality

## 🚀 Quick Start

### Installation

```bash
cd apps/contracts
npm install
```

### Configuration

```bash
# Create environment file
cp .env.example .env

# Edit .env with your values:
# - PRIVATE_KEY: Deployer wallet private key
# - BSCSCAN_API_KEY: BSCScan API key for verification
# - ADMIN_WALLET_ADDRESS: Admin wallet address
```

### Compile Contracts

```bash
npm run compile
```

### Run Tests

```bash
npm run test
```

## 🌐 Deployment

### BSC Testnet

```bash
# Deploy to BSC Testnet
npm run deploy:testnet

# Verify on BSCScan
npm run verify:testnet
```

### BSC Mainnet

```bash
# Deploy to BSC Mainnet
npm run deploy:mainnet

# Verify on BSCScan
npm run verify:mainnet
```

## 📋 Token Addresses

### BSC Mainnet (ChainID: 56)
- **USDT**: `0x55d398326f99059fF775485246999027B3197955`
- **USDC**: `0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d`

### BSC Testnet (ChainID: 97)
- **USDT**: `0x337610d27c682E347C9cD60BD4b3b107C9d34dDD`
- **USDC**: `0x64544969ed7EBf5f083679233325356EbE738930`

## 💵 Plan Pricing

| Plan ID | Plan Name | Price (USD) |
|---------|-----------|-------------|
| 1 | Starter | $29 |
| 2 | Professional | $59 |
| 3 | Enterprise | $99 |

## 🔧 Contract Functions

### User Functions

#### `payForPlan(uint256 planId, address token, uint256 amount)`
Pay for a subscription plan.

**Parameters:**
- `planId`: Plan ID (1, 2, or 3)
- `token`: Token address (USDT or USDC)
- `amount`: Payment amount (must match plan price)

**Events:**
- `PaymentReceived(user, planId, token, amount, timestamp, paymentId)`

### Admin Functions

#### `setPlanPrice(uint256 planId, uint256 price)`
Update plan pricing.

#### `setTokenEnabled(address token, bool enabled)`
Enable or disable payment tokens.

#### `withdrawFunds(address token, uint256 amount, address recipient)`
Withdraw collected funds.

#### `pause()` / `unpause()`
Emergency pause/unpause contract.

### View Functions

#### `getPlanPrice(uint256 planId)`
Get plan price.

#### `isTokenSupported(address token)`
Check if token is supported.

#### `getTokenBalance(address token)`
Get contract's token balance.

#### `getTotalPayments()`
Get total payment count.

## 🔐 Security Features

- ✅ **Access Control**: Owner-only admin functions
- ✅ **Reentrancy Protection**: SafeERC20 and ReentrancyGuard
- ✅ **Pausable**: Emergency stop mechanism
- ✅ **Input Validation**: Comprehensive validation on all functions
- ✅ **Duplicate Prevention**: Payment hash tracking

## 📊 Testing

The test suite covers:
- ✅ Deployment and initialization
- ✅ Plan configuration
- ✅ Token management
- ✅ Payment processing (USDT & USDC)
- ✅ Fund withdrawal
- ✅ Emergency pause
- ✅ Access control
- ✅ Error cases

Run tests with:
```bash
npm run test
```

Generate coverage report:
```bash
REPORT_GAS=true npm run test
```

## 📁 Deployment Artifacts

After deployment, the following files are created in `deployments/`:

- `mainnet.json`: BSC Mainnet deployment info
- `testnet.json`: BSC Testnet deployment info
- `PaymentEscrow.json`: Contract ABI

## 🔗 Integration

### Frontend Integration

```typescript
import { PaymentEscrow } from '@/lib/contracts/PaymentEscrow';
import { PAYMENT_ESCROW_ADDRESS } from '@/lib/contracts/addresses';

// Approve token spending
await usdtContract.approve(PAYMENT_ESCROW_ADDRESS, amount);

// Pay for plan
await paymentEscrow.payForPlan(planId, usdtAddress, amount);
```

### Backend Integration

Listen to `PaymentReceived` events:

```typescript
contract.on('PaymentReceived', (user, planId, token, amount, timestamp, paymentId) => {
  // Verify payment
  // Activate subscription
});
```

## 📚 Resources

- [Hardhat Documentation](https://hardhat.org/docs)
- [OpenZeppelin Contracts](https://docs.openzeppelin.com/contracts/)
- [BSC Documentation](https://docs.bnbchain.org/)
- [BSCScan API](https://docs.bscscan.com/)

## ⚠️ Important Notes

1. **Never commit `.env`**: Keep private keys secure
2. **Test thoroughly**: Always test on testnet first
3. **Verify contracts**: Always verify contracts on BSCScan
4. **Monitor events**: Set up proper event monitoring
5. **Secure admin keys**: Use hardware wallet for admin operations

## 🆘 Support

For issues or questions:
- Check the [test suite](./test/PaymentEscrow.test.ts)
- Review [deployment logs](./deployments/)
- Contact the development team
