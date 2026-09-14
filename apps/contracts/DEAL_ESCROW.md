# Pay DealEscrow v1

This is a separate, non-proxy deployment. Keep the existing PaymentEscrow
subscription deployment and its history unchanged.

Build and test with Foundry:

```
forge test --match-contract DealEscrowTest
```

The deployment script requires explicit public configuration:
`PAY_ESCROW_EXPECTED_CHAIN_ID` (31337, 97 or 56), `PAY_ESCROW_DEPLOYER`,
`PAY_ESCROW_ADMIN`, `PAY_ESCROW_TREASURY`, and `PAY_ESCROW_TOKEN_ADDRESSES`
(comma-separated reviewed ERC-20 addresses; empty for native BNB only).
BNB is always supported and must not be included as a zero token address.
Use the existing Admin wallet. The arbiter, treasury and token allowlist are
immutable after deployment; verify them before funding any deal.

Simulate without publishing a transaction:

```
forge script script/DeployDealEscrow.s.sol:DeployDealEscrow --rpc-url "$PAY_ESCROW_RPC_URL"
```

After a separate explicit deployment instruction, an operator supplies a
Foundry wallet or hardware signer and adds `--broadcast`. The backend must never
receive that signer's private key. Record chain ID, deployed address, creation
transaction and block, runtime bytecode hash, source revision/compiler settings,
arbiter, treasury and allowlist in the deployment review. Set the Pay service's
deployment block to the creation block and use the matching token decimals.

Run the accepted browser/wallet and failure/recovery gates on BSC testnet before
requesting a mainnet deployment. Contract tests and a local Anvil rehearsal are
not a substitute for a security review or testnet acceptance.
