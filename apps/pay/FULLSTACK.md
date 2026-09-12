# Pay Dioxus Fullstack

The native `bff-pay` binary mounts the same `PayApp` component tree hydrated by
`epsx-pay` WebAssembly. Page reads use `use_server_future`; subsequent updates use
signals and typed server functions. The BFF retains the existing verified Pay
session and merchant API contracts.

## Route and interaction inventory

| Routes | Dioxus interactions |
| --- | --- |
| `/` | Public service landing, persistent theme, responsive navigation and environment-preserving workspace/documentation links; no payment provider read |
| `/dashboard` | Overview, payment rows, environment selection, refresh, internal navigation |
| `/payments`, `/payments/:id` | Payment/collection state, service-authorized collect/refund/escrow operations |
| `/packages`, `/packages/:id/edit` | Typed create/edit form, currency prices, duration, availability, validation and retry |
| `/packages/:id`, `/m/:merchant` | Public product cards, token selection, idempotent checkout creation |
| `/payment-links` | Link form, exact decimal input, direct/escrow selection, expiration and use count, disable |
| `/r/plink_:id` | Account-free checkout creation with retained idempotency key |
| `/checkout/cs_:id` | SSR shell, capability recovery from fragment/session storage, QR/transfer and wallet method selection, copy, WalletConnect pairing, pending/terminal status |
| `/settings` | Profile form, API key create/revoke, one-time secret reveal/dismiss |
| `/webhooks` | Create/replace/enable/disable endpoints, rotate secret, delivery attempts and replay |
| `/docs`, `/docs/merchant` | Bundled merchant documentation; static Markdown body, Dioxus page and navigation |
| `/escrow` | Legacy native escrow link creation and history |
| `/checkout/:legacy-id`, `/intent/:legacy-id`, `/r/:legacy-slug` | Original native Pay protocol, participant-only reads, redeem, deposit/release/refund/dispute, approval/transaction confirmation |
| All routes | Persistent theme, login/logout/session refresh, scoped refresh tasks, loading/error state |

Unknown routes return HTTP 404. Auth, Pay REST proxy, wallet configuration and
static assets stay on their existing paths. The only new server functions are
`POST /_server/pay/read` and `POST /_server/pay/action`; no arbitrary upstream
path or method is accepted. Other app server functions are never registered.

## Public home

The root route renders the public merchant landing component directly, including
its scoped stylesheet and sample checkout illustration. It does not call
`read_pay`, load merchant data, or require authentication; `/dashboard` continues
to own workspace onboarding and payment reads. Signed-in visitors also see the
public home at `/`.

Home, workspace branding, and documentation return links preserve
`?environment=test|live` (default `test`). The home theme toggle shares the
existing persistent `PayTheme` state. The sample checkout is decorative and has
no payment action. Pricing copy reflects the existing 0.5% direct / 1% released
escrow rates, with gas separate.

The Pay SSR test renders home with an unreachable upstream to protect this
boundary. Shared UI tests cover route recognition and environment selection.

## Browser capabilities

`wallet_adapter.js` only bridges browser capabilities: EIP-1193 wallet requests,
WalletConnect transport, clipboard, persisted theme, checkout capability/request
recovery. Authentication uses typed Pay server functions which call the existing
verified native session handlers and forward HttpOnly cookies. It never selects elements,
replaces HTML, or handles UI events. Pairing QR, forms, status, navigation and
secret visibility belong to Dioxus.

The service supplies available actions, prices and authoritative payment status.
A wallet hash is shown as submitted, never as successful payment. Prepared
operations retain their request keys, approval hashes and transaction hashes for
retry. Checkout creation keys remain until the checkout reaches a terminal state.

## Build and verify

Build matching client and server features (do not pass `web` to the server):

```sh
dx build --fullstack true --web --no-default-features --locked --package epsx-pay-bff --bin epsx-pay
cargo build -p epsx-pay-bff --bin bff-pay --features server --locked
cargo test -p epsx-pay-bff --lib fullstack::tests --locked
```

Package the entire generated `target/dx/epsx-pay/<profile>/web/public` directory.
Set `DIOXUS_PUBLIC_PATH` to that directory. The BFF serves both `/assets` and
`/wasm`; debug Dioxus output uses `/wasm/epsx-pay.js`. Startup refuses a missing
hydration index or WASM binary. The old browser runtime is not mounted.

An ignored loopback-only browser fixture provides synthetic merchant and checkout
data, with no connection to production or an actual chain:

```sh
DIOXUS_PUBLIC_PATH="$PWD/target/dx/epsx-pay/debug/web/public" \
  cargo test -p epsx-pay-bff --lib pay_browser_fixture -- --ignored --nocapture
```

Open `http://127.0.0.1:43127`. Stop the fixture with Ctrl-C. Wallet tests must use
an injected mock EIP-1193 provider; no real transactions are needed.

Historical `worker.rs`, `worker.js`, Docker and Wrangler files are inactive
rollback references, not part of the native Fullstack release path.

The browser audit is reproducible with `python3 e2e/enterprise/dioxus_pay_audit.py`
from the repository root after starting the ignored fixture. It checks actual
hydrated events, history, forms, QR decoding, responsive overflow and a mock
EIP-1193 provider (rejection, network mismatch and repeated transaction safety).
Screenshots and the 20-check report are written to `target/pay-fullstack-evidence`.

Both native entrypoints call `epsx_pay_bff::run()`. The DX server has the same
providers, verified session middleware and REST compatibility routes as `bff-pay`.

Historical singular REST intent URLs remain transport aliases to the native plural service: `/api/v1/pay/intent`, `/:id`, `/:id/execute`, and `/:id/status`. The old create fields are converted to the native intent DTO; plural merchant APIs are unchanged. These aliases use the same verified Pay session and same-origin mutation checks as the canonical proxy. They do not mount a legacy browser UI.
