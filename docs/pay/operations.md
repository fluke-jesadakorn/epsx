# Merchant Pay operation and release qualification

Native runtime remains governed by `infrastructure/native/README.md`. Nothing in this change deploys a contract, changes Tunnel/DNS, switches production traffic, or regenerates signing keys.

## Configuration

Keep `PAY_ESCROW_*` and the legacy package contract configuration unchanged. The new merchant contracts are separate deployments from `apps/contracts/contracts/MerchantPayments.sol`: `DirectPayments` version 1 at 50 bps; `MerchantEscrow` version 2 at 100 bps. Existing `DealEscrow` version 1 remains 30 bps. Always identify a contract by chain ID, address and version; version alone is not globally unique.

Set `PAY_MERCHANT_SECRET` in the private Pay-service configuration to a persistent random secret of at least 32 bytes. It derives checkout capabilities and versioned webhook endpoint secrets. Losing/changing it breaks recovery and webhook signatures. Back it up with existing RSA/refresh keys and configuration; do not place it in release assets, browser variables or logs. API keys themselves are hashed in PostgreSQL.

Set `PAY_MERCHANT_NETWORKS` to a JSON array. The following is a shape example; replace every symbolic value before use:

```json
[
  {
    "environment": "test",
    "chain_id": 97,
    "rpc_url": "https://YOUR_BSC_TESTNET_RPC",
    "admin": "EXISTING_ADMIN_WALLET",
    "treasury": "REVIEWED_TREASURY_WALLET",
    "direct": {"address": "REVIEWED_DIRECT_CONTRACT", "deployment_block": 123},
    "escrow": {"address": "REVIEWED_MERCHANT_ESCROW_CONTRACT", "deployment_block": 124},
    "tokens": {
      "BNB": {"address":"0x0000000000000000000000000000000000000000", "decimals":18},
      "USDT": {"address":"REVIEWED_ALLOWLISTED_USDT", "decimals":18},
      "USDC": {"address":"REVIEWED_ALLOWLISTED_USDC", "decimals":18}
    },
    "confirmations": 3
  }
]
```

Use the actual token decimals; startup reconciliation checks them against the token contract. BNB uses 18 decimals. Mainnet is environment `live`, chain 56, minimum 15 confirmations. Local Anvil is `test`, chain 31337, minimum 1. At most one network per environment is configured. Numeric loopback HTTP is allowed for local RPC; other RPCs require HTTPS. Invalid network configuration prevents startup. The reconciler verifies deployed chain, code, version, fee, mode, Admin, treasury and token allowlist before enabling checkouts. `/ready` fails if any configured merchant contract's checkpoint is unavailable, more than 60 seconds stale, or still catching up.

Networks may additionally specify `archive_rpc_url` and `scan_blocks` (default 10,
allowed 1..50). An archival endpoint is used only when the primary explicitly
reports pruned log history; it must return the configured chain ID. Native escrow
uses `PAY_ESCROW_ARCHIVE_RPC_URL` and `PAY_ESCROW_SCAN_BLOCKS` for the same policy.
All archive operations share a three-second rate limit. Qualify the selected
block range and recovery throughput before increasing it; recent receipt access
alone does not prove that a provider retains historical event logs. See the
native operations guide for failure handling and public-provider limits.
The opt-in merchant Anvil suite accepts `EPSX_MERCHANT_TEST_SCAN_BLOCKS` to
exercise a qualified larger range. Native escrow reads its usual
`PAY_ESCROW_SCAN_BLOCKS`; use isolated databases for either rehearsal.

The Foundry deployment script is `apps/contracts/script/DeployMerchantPayments.s.sol`. It reads explicit chain, deployer, Admin, treasury and token address settings. It uses the operator's selected Foundry wallet. No application process receives a fund-moving private key. Run simulation/review first. Broadcast on BSC testnet/mainnet only after an explicit deployment instruction and the relevant review gates.

## EPSX as first merchant

Register the EPSX settlement wallet through Pay using signed authentication. Create the appropriate test/live API key and an outgoing endpoint for `https://api.epsx.io/api/webhooks/epsx-pay`. Save that endpoint's secret. Do not reuse the incoming chain-hint webhook secret.

Configure only the EPSX backend with `EPSX_PAY_API_KEY`, `EPSX_PAY_MERCHANT_ID`, `EPSX_PAY_ENVIRONMENT`, `EPSX_PAY_RECIPIENT` and `EPSX_PAY_WEBHOOK_SECRET_OUTBOUND`. Its `PAYMENT_SERVICE_URL` remains numeric loopback; `PAY_FRONTEND_URL` remains the public HTTPS Pay origin.

Set each purchasable plan's `plan_metadata.pay_prices`, for example `{"USDT":"100.00","USDC":"100.00"}`, using reviewed prices explicitly denominated in those tokens. Existing USD plan prices are not silently converted. These are final token prices; legacy USD promotions do not automatically alter them. Only active, public, non-system subscription plans are purchasable through this path. Plan metadata/billing cycle determines the frozen entitlement duration in the Rust backend.

After configuring and validating EPSX fulfillment, set `EPSX_PAY_CHECKOUT_ENABLED=true` and `EPSX_PAY_CHECKOUT_TOKEN=USDT` in the Frontend BFF configuration. The browser sends plan/token identifiers only. The backend freezes the order and exact token price before contacting Pay. The browser redirect never grants access. Existing purchases continue through their original monitor. New Pay records are financially written only by Pay's reconciler.

Entitlement grants are recorded per purchase. A later manual absolute plan assignment supersedes the prior entitlement projection: correcting/refunding an old purchase preserves that Admin assignment, while a new purchase gets a new grant. Existing legacy monitor grants share this ledger once the plan has Pay purchases. Financial purchase/event history is never removed by a manual access override. JWT authorization for wallets with purchase grants reloads effective permissions from PostgreSQL, including after refunds and cache loss; Redis invalidation is not the only revocation mechanism.

## Recovery

Pay startup does not apply migrations. Explicitly run the packaged migration runner's `pending` then `up` with the core and Pay service database URLs. Both new schemas are additive. Never replay a baseline over populated storage, run destructive down migrations, change historical checksums or restore an old database as a binary rollback.

Events, revisions, outbox jobs, leases, operation parameters, chain proofs and checkpoints persist in PostgreSQL. Reorgs invalidate canonical evidence, emit corrective events and rebuild from each contract's deployment block. Financial webhook deliveries wait for healthy reconciliation; corrective `payment.verification_required` notifications can be sent immediately. An RPC outage returns unavailable instead of fabricating a refund or success. Webhook retries last 72 hours; manual replay works for 30 days. Retain financial/event history beyond that period for audit and reconciliation.

A rollback must preserve reconciliation and webhook delivery for new contracts until every outstanding merchant payment and escrow has been reconciled/settled. A pre-merchant binary cannot process these records: do not route new merchant checkout traffic to it, and keep the compatible Pay reconciler and merchant-aware API gateway/authorization running even if rolling back another app. Keep a qualified previous **merchant-compatible** release for normal whole-system rollback. Never erase new records to make an old binary appear compatible.

Include the Pay/core PostgreSQL databases, MinIO, existing signing/refresh keys, merchant master secret, EPSX API key and endpoint secrets in backup/restore rehearsals. Restore only into isolated databases/directories. Check reconciliation, pending deliveries and entitlements with new records present before accepting the release.

## Required qualification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --locked` (unfiltered)
- `cargo check -p epsx-browser-runtime --target wasm32-unknown-unknown`
- `cargo xtask audit no-node --strict` and `cargo xtask assets verify`
- `forge test` in `apps/contracts` (both legacy and merchant suites)
- Explicit real PostgreSQL/Anvil test: `merchant::tests::merchant_payments_guest_isolation_webhooks_reorg_and_recovery` in `epsx-pay-svc --bin pay-service`, using an isolated database named `epsx_merchant_check_*` via `EPSX_MERCHANT_TEST_DATABASE` and numeric-loopback RPC via `EPSX_MERCHANT_TEST_RPC`. This test deploys only to its local Anvil fixture.
- Explicit core integration tests: `merchant_checkout::tests` in `epsx --lib -- --include-ignored`, using a migrated isolated `EPSX_MERCHANT_CORE` database named `epsx_merchant_check_*`.
- Native release package launched outside the source tree, all app assets/deep links, authenticated refresh/logout and restart persistence.
- Real browser/wallet journeys for guest direct/escrow, full refunds, disputes and Admin resolution with each supported token; BSC testnet evidence; security review; backup/restore and compatible binary rollback.

7 October 2026 is the target, not a reason to waive a failed gate. Both direct and escrow flows must pass. Production cutover and new BSC contract deployment remain separate, explicitly instructed actions.
