# EPSX Pay merchant dashboard — dev

Open https://dev-pay.epsx.io/dashboard. Connect MetaMask and sign the SIWE message, then choose a shop name. No email or password is required. One wallet owns one shop and is its immutable receiving wallet.

## Using the dashboard

- **Overview**: confirmed customer payments, token balances ready for collection after the 0.5% fee, and processing fees. Fees on QR invoices are estimates until collection. Refunded direct payments retain their processing fee.
- **Payments**: payment state and collection state are independent. Open Details to collect or refund. Collection requires the shop wallet and a gas signature. Refunds require the merchant to fund the full original amount; token approval may be needed.
- **Packages**: name, description/service terms, a price for USDT and/or USDC, optional positive service duration in days, and availability. Prices entered here are decimal token amounts, validated and converted by the server. These are one-time purchases, not renewing subscriptions.
- **Payment links**: a custom item name, token price, optional usage limit, and expiry. New stablecoin direct links use QR checkout with optional wallet connection. Existing contract direct/escrow links retain their original behavior.
- **Webhooks**: enter a public HTTPS URL. An endpoint receives events for all packages and links in its merchant/environment. Save the signing secret when shown. Enter another URL and choose **Replace with URL above** on the old endpoint to create a new version. The old endpoint is disabled, its delivery history stays unchanged, and replay never redirects those deliveries to the replacement. Replaced endpoints cannot be re-enabled; ordinary disabled endpoints can.
- **Settings**: update shop name and manage server-only API keys. Changing the shop name affects new checkouts; old receipts keep the snapshot.

Share `/m/{merchant_id}?environment=test` for the storefront or `/packages/{product_id}?environment=test` for a package. Both QR transfer and Connect wallet use the same checkout receiver. A package edit or closing it cannot rewrite an existing checkout. A repeated create request with the same idempotency key returns the same checkout; a new purchase of a closed package is rejected.

The generic receipt confirms payment. The merchant provides the service through its webhook integration. EPSX Plan checkouts separately track payment and access grants through the existing EPSX backend; no catalog webhook directly grants EPSX permissions.

## API additions

All routes are below `/api/v1/pay`, with `X-Pay-Api-Version: 2026-09-08` and `X-Pay-Environment: test|live`. Management uses the owner's EPSX Pay SIWE session. A bearer API key cannot manage catalog, webhook settings or delivery retries.

| Method | Route | Purpose |
|---|---|---|
| GET | `/merchants/me` | Shop profile; existing server API key reads remain supported |
| POST | `/merchants/me/update` | Update `name` |
| GET/POST | `/products` | List/create owned packages |
| GET | `/products/{id}` | Read an owned package |
| POST | `/products/{id}/update` | Replace editable fields, increment revision |
| GET | `/catalog/{merchant_id}` | Public active packages for the environment |
| GET | `/catalog/products/{id}` | Public active package |
| POST | `/products/{id}/checkouts` | Public checkout, body `{"token":"USDT"}` |
| GET | `/overview` | Exact amounts in token base units |
| POST | `/webhook-endpoints/{id}/replace` | New URL/version; body `{"url":"https://..."}` |
| POST | `/webhook-endpoints/{id}/enable` | Enable an endpoint that has not been replaced |

Create/update package body (complete editable fields):

```json
{"name":"Research Starter","description":"Seven days of updates","prices":{"USDT":"5","USDC":"5"},"duration_days":7,"enabled":true}
```

Package writes and public checkout creation require `Idempotency-Key`. Public creation also requires `X-Pay-Guest-Id`, 32–128 alphanumeric/hyphen characters. Package response `prices` values use token **base units**, not the human decimal inputs. Read decimals from `/config`. Client-supplied recipient, price and duration are ignored when creating a package checkout; the server copies the catalog row under a lock.

Each payment/webhook contains `checkout_snapshot`: merchant name, item name, optional description, product ID/revision, duration, payee, token, amount, and `kind` (`merchant` or trusted `epsx_plan`). Order reference, payment ID, transaction hash, event ID and signature format remain unchanged. Existing events are immutable and are not rewritten by the migration.

## Webhook consumer example

Read the **raw request bytes**, not reserialized JSON. `EPSX-Pay-Signature` contains `t=<Unix seconds>,v1=<hex HMAC>`, possibly two `v1` values during the 24-hour rotation overlap. The signing input is `timestamp + "." + raw_body`, HMAC-SHA256 with the endpoint signing secret. Reject timestamps outside your five-minute replay window and compare MACs in constant time.

Rust verification using the same HMAC primitives as Pay:

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn valid(secret: &str, timestamp: i64, raw: &[u8], candidate: &str) -> bool {
    let Ok(signature) = hex::decode(candidate) else { return false };
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(raw);
    mac.verify_slice(&signature).is_ok()
}
```

After validating the timestamp and at least one signature, atomically insert the event ID into your database with a unique constraint and record the service action. Return 2xx after durable acceptance. A duplicate event ID returns 2xx without repeating fulfillment. Handle `payment.succeeded`, `refund.succeeded` and `payment.verification_required` separately, including reorg/review outcomes. Do not use arrival order as payment revision order. Pay retries failed deliveries with backoff for 72 hours; the dashboard supports retry within the retained 30-day event window.

## Dev operation and rollback

Runtime: native Rust, existing Anvil **31337**, fake USDT/USDC, RPC `127.0.0.1:8545`. It is not public; a mobile wallet cannot reach it. Merchant login uses MetaMask. WalletConnect checkout still requires a valid Reown project ID; the current configured value is a placeholder.

Private configs, test-wallet import material and backups: `~/.config/epsx/dev`. Never copy keys or database dumps into public docs or release directories. Existing wallet instructions are in [wallet-checkout-dev.md](wallet-checkout-dev.md).

```sh
python3 infrastructure/native/dev-control.py status
python3 infrastructure/native/dev-control.py restart pay-service
python3 infrastructure/native/dev-control.py restart bff-pay
python3 infrastructure/native/dev-control.py rollback
```

Rollback restores the previous native release/configuration snapshot and restarts dev services; **do not downgrade the database**. The additive migration `20260909010000_add_merchant_catalog.sql` preserves existing merchants, links, payments, events and deliveries. Dev Pay and core dumps were saved before migration in the private `backups/merchant-20260909-111235` directory. Production routes, contracts and databases were not changed.

## Validation evidence

See [merchant-proof.json](evidence/dev-20260909/merchant-proof.json) for actual dev Anvil USDT/USDC payment and collection hashes, independent wallet ownership rejection, and exact balances. These transactions were signed using the dedicated test-wallet CLI; they are not claimed as interactive MetaMask acceptance.

The isolated PostgreSQL/Anvil suite exercises package snapshots, disable/idempotency, two-merchant isolation, generic token transfer and collection, QR refund, legacy direct/escrow lifecycle, webhook failure/retry/signature rotation, endpoint replacement history and reorg recovery. Foundry: 33 tests passed. Native Rust tests, workspace Clippy, Wasm Clippy, formatting, assets verification and strict no-node audit passed during implementation.

Headless browser validation passed at 1440px and 390px in both themes, using a separate profile and a CLI-signed test-wallet session. Creating/editing/disabling a package, creating a link, QR/Wallet switching, checkout reload and generic receipt presentation were checked. Real dev USDT refund, EPSX Plan webhook delivery (HTTP 200), access grant, matching admin record, and preservation after service/Anvil restart also passed.

Interactive MetaMask merchant login/payment/collection is still pending: the Mac was locked when computer use attempted access. WalletConnect pairing/mobile payment is not part of this acceptance; a valid project ID is still needed.
