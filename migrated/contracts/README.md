# EPSX Smart Contracts

Foundry-based smart contracts for the EPSX platform, deployed to BSC mainnet and testnet.

## Contracts

### PaymentEscrow
Holds stablecoin (USDC/USDT) in escrow between buyer and seller. Supports:
- `createEscrow()` — buyer deposits tokens
- `releaseEscrow()` — buyer or admin releases to seller (0.3% fee)
- `refundEscrow()` — seller or admin refunds to buyer
- `disputeEscrow()` — buyer or seller flags a dispute
- `resolveDispute()` — owner resolves in either direction

### SubscriptionVault
Per-merchant subscription management:
- `createPlan()` — merchant defines a plan (amount, period, grace periods)
- `subscribe()` — user subscribes to a plan
- `charge()` — user pays for N periods in advance
- `cancel()` — user cancels
- `withdraw()` — merchant withdraws accumulated earnings

### Paymaster
ERC-4337 style paymaster for gas sponsorship:
- Users deposit USDC/USDT
- Sponsor pays native gas on user's behalf
- Charges user in stablecoin at gas price + 1% markup

### TokenRegistry
On-chain registry of accepted payment tokens per chain.

## Build

```bash
forge build
```

## Test

```bash
forge test
```

## Deploy (Local)

```bash
anvil &
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 forge script script/Deploy.s.sol:Deploy --rpc-url local --broadcast
```

## Deploy (BSC Testnet)

```bash
forge script script/Deploy.s.sol:Deploy --rpc-url bsc_testnet --broadcast
```

## Deploy (BSC Mainnet)

```bash
forge script script/Deploy.s.sol:Deploy --rpc-url bsc --broadcast --verify
```

## Architecture

- **OpenZeppelin** for ERC20, Ownable, ReentrancyGuard, Pausable
- **Solidity 0.8.24** with optimizer enabled
- **Foundry** for build/test/deploy
- **via_ir** in production profile for gas optimization
