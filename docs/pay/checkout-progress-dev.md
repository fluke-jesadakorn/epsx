# Hosted checkout progress and EPSX return

Hosted `/checkout/cs_*` uses the shared Tailwind pipeline and a compact checkout header. Wallet progress is separate from payment proof and EPSX fulfillment. A submitted hash is never treated as a successful payment.

The browser stores `epsx.checkout.session.<checkout_id>` in session storage. Pending operations resume with the same hash after refresh; status checks run every four seconds without overlapping page reads. A failed operation allows a fresh attempt. Long waits and temporary service failures preserve the pending transaction and explorer link.

## Completion API

`GET /api/payments/checkout-completion/{checkout_id}` requires `x-pay-checkout-token`. The core backend validates the capability with Pay before matching the configured EPSX merchant, environment, payment intent, chain, contract, payee, token and amount to a purchase order. This read does not create grants.

The `completion` projection contains only `order_id`, `payment_status`, `fulfillment_status` and `return_url`. Unrelated merchant checkouts return `completion: null`. Legacy `kind: merchant` snapshots remain supported through the verified order match. `PAY_EPSX_MERCHANT_ID` in Pay must match core's `EPSX_PAY_MERCHANT_ID` for new EPSX snapshots.

The return URL is built from the environment's configured `FRONTEND_URL` and the verified order UUID. The BFF and browser validate that it stays on that origin and exact purchase path. Query parameters and referrers do not control this destination.

An active EPSX checkout displays success only after payment succeeds and the actual grant and assignment are active. It counts down three seconds, then uses `location.replace`. An already completed receipt opened later remains a receipt with a manual return button. The existing frontend authentication gate preserves the purchase path if sign-in is required.

## Development verification

No production deployment or schema migration is part of this change.

- Pay dev and its live CSS worker use `apps/frontend/public`, matching the shared Tailwind output.
- Reuse the running dev watcher: `python3 infrastructure/native/dev-control.py rebuild bff-pay`.
- Pay BFF tests: `cargo test -p epsx-pay-bff --all-features --lib fullstack::tests`.
- Checkout state tests: `cargo test -p epsx-dioxus-ui --features 'server pay-ui' fullstack::pay`.
- Core integration tests use an isolated `epsx_merchant_check_*` database and `EPSX_MERCHANT_CORE`; never point them at application databases.
- The ignored `fullstack_fixture::pay_browser_fixture` test (compile with default/server features, without the native `web` feature) serves real Dioxus assets at loopback port 43127. Set `DIOXUS_PUBLIC_PATH` to the built Pay `public` directory. It provides a simulated wallet via fixture-only HTML and has no RPC, keys or real-fund transfers.
- `e2e/enterprise/dioxus_pay_audit.py` exercises wallet approval/rejection, refresh, outages, delayed fulfillment, failure, receipt reopening, merchant receipts and responsive layouts against that fixture. Its `/fixture/scenario/{scenario}` controls are compiled only for tests.
- Verify dev checkout HTTP 200 without a capability; error details must load inside the shell after hydration. Verify the private frontend purchase path returns a login redirect containing the same path when signed out.

## Verification recorded for this change

Core (5), Pay BFF (9), shared UI (7), and dev orchestration (11) tests passed. The isolated Anvil guest/payment/webhook/reorg/recovery integration also passed. Relevant Clippy checks, formatting, no-node audit and assets verification passed. Browser verification covered ready/wallet/approval/confirming/activation/automatic return, refresh without a second send, outages, rejection, failed transactions, old and merchant receipts, expiration, copy feedback and explorer links, keyboard focus, reduced motion, and light/dark layouts at 320/390/1440px. The dev checkout and stylesheet both returned HTTP 200.
