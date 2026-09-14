# EPSX Pay — QR and connected wallets

The hosted checkout offers **QR / Transfer** (default) and **Connect wallet**.
Both transfer the exact ERC-20 amount to the same invoice receiver. Changing the
UI option does not create an order, change its price, bind the purchaser to the
paying wallet, or invoke the legacy DirectPayments approval flow.

MetaMask connects only after a click. The buyer reviews the account/network and
presses Pay separately. The wallet signs one ERC-20 transfer. Pay's existing
canonical scanner, confirmations and webhook remain the source of truth for
payment and plan access. A submitted hash is retained in session storage across
reloads; the Pay button stays disabled while that transfer is pending. Reverted
receipts permit retry, while an unknown/pending receipt never permits automatic
resubmission. The blockchain cannot prevent a user from separately sending a
second manual transfer; entitlement fulfillment remains idempotent.

## API

`POST /api/v1/pay/checkout-sessions/{checkout_id}/prepare-transfer` requires the
existing API-version/environment headers and `x-pay-checkout-token` capability.
The body accepts `payer`; the server derives token, receiver, amount, chain and
calldata from its stored invoice. Other client transaction fields have no effect.
The response contains `transaction_parameters` and the public `payment` snapshot.
Preparation does not bind the payer, store an operation or mark payment complete.
It rejects expired/non-payable/non-transfer invoices, invalid capability, unhealthy
scanner, insufficient selected token and insufficient native gas balance.
Wallet gas estimation is advisory; gas prices and balances can change before signing.

## WalletConnect

Only the WalletConnect option loads the pinned, self-hosted SDK. Rust renders the
pairing QR separately from the transfer QR. The bundle has SRI and a checksum
manifest; the no-node audit permits exactly its SDK/bridge files and still
rejects other scripts. See `apps/pay/vendor/README.md` for provenance and updates.
No Node process or build toolchain is required in the native deployment.

`WALLETCONNECT_PROJECT_ID` must contain a real 32-character Reown Project ID,
configured for `https://dev-pay.epsx.io`. This is a public application identifier,
not an API secret. `/api/wallet-config` exposes only that identifier and SDK
location/integrity. An absent or placeholder ID produces an explicit configuration
message; no third-party connection is attempted. The preexisting dev value was
found to be a placeholder during this implementation; a real ID is requested.

Anvil remains bound to `127.0.0.1:8545`, chain 31337, on the Mac Mini. No public
or LAN RPC is added. Mobile payment acceptance is deferred: a phone cannot reach
this loopback chain, even if WalletConnect pairing succeeds.

## Operations

Use `python3 infrastructure/native/dev-control.py status`, `restart bff-pay`,
`restart pay-service`, or `rollback` from the repository. Release/config snapshots
retain QR compatibility; rollback does not revert the database. No migration,
contract redeployment, production route or public RPC change is required.

## Delivery verification — 2026-09-09

Active release: `20260909-wallet-v5`; previous: `20260909-wallet-v4`.
All four dev domains return HTTP 200. Native services restarted successfully;
both verified payments and grants remain intact.

- [MetaMask browser proof](evidence/dev-20260909/wallet-metamask-proof.json):
  browser-created order, connected account/chain, user-confirmed wallet popup,
  5 USDT, automatic payment success, purchaser history `succeeded / granted`,
  matching admin transaction and webhook HTTP 200 on the first attempt.
- [Prepared API proof](evidence/dev-20260909/wallet-prepared-proof.json):
  separate 5 USDC order; exact prepared transaction signed with generated buyer
  through CLI; webhook, admin and entitlement verified.
- [Validation and remaining limitations](evidence/dev-20260909/wallet-delivery-status.json):
  targeted Rust tests, isolated PostgreSQL/Anvil integration, workspace/Wasm
  Clippy, format, assets, strict audit and 33 Foundry tests passed. Both themes
  and responsive layouts inspected; the mobile layout has no horizontal overflow.

WalletConnect lifecycle was tested with a mock provider. Real relay pairing still
requires a valid Project ID; mobile payment is deferred. Manual MetaMask rejection,
account switching and wrong-chain popup scenarios were not completed through the
extension UI and are not counted as passed.

The bridge uses plain JavaScript objects for EIP-1193 transactions/config rather
than serializing Rust JSON objects as JavaScript Maps. CSS has an explicit revision
to refresh older browser caches. Transient polling errors clear on recovery.
