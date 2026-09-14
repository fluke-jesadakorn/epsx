# EPSX Pay integration guide

API version: **2026-09-08**. Base URL: `https://api.epsx.io/api/v1/pay`.
The same API serves EPSX package purchases and external merchants.

## Start accepting payments

1. Open Pay and sign in with your wallet. Create a merchant account using a business/display name. There is no merchant approval step.
2. Select Test or Live. Create a payment link, or create a server API key for that environment. Keys are displayed once. Keep them in your backend, never in browser code.
3. Add a public HTTPS webhook endpoint and save its signing secret. Create separate test and live endpoints.
4. Share the hosted link or the `pay_url` returned when your backend creates a checkout.
5. Your buyer connects a wallet, switches to BSC if needed, approves the exact token amount if needed, and signs the payment. Buyers do not create an account or sign a SIWE message.
6. Fulfill only after a verified `payment.succeeded`. An escrow deposit is `escrow.funded`; funds are still held.

Live availability depends on the platform's configured, qualified mainnet contracts. Test uses BSC testnet (or Anvil in local development). Query `GET /config` for the exact chain, confirmations, contracts, tokens and decimals. Symbols alone never identify a token contract. There is no currency conversion.

## Authentication and requests

Send `Authorization: Bearer YOUR_SECRET_KEY` from your server. Send `X-Pay-API-Version: 2026-09-08`. `X-Pay-Environment: test` or `live` must match the key. A key with no environment header uses its own environment.

Signed merchant sessions use the `epsx-pay` audience and HttpOnly BFF cookies. API-key creation, revocation and webhook secret management require that session. Admin dispute resolution requires an `epsx-admin` session, the payment-management permission and the configured Admin wallet.

Use `Idempotency-Key` for link creation, intent/session creation and every transaction preparation. Choose a random key per logical action, save it before the request, and reuse it on network errors. A reused key with a different payload returns HTTP 409. Idempotency is scoped to the merchant and environment (or the specific guest checkout). An order reference cannot create a second payment under a different key. JSON amounts are positive **integer strings in token base units**; never use floating point.

Example request body for `POST /intents` (also accepted by `POST /checkout-sessions`):

```json
{
  "mode": "direct",
  "token": "USDT",
  "amount": "100000000000000000000",
  "order_reference": "order-1042",
  "description": "Order 1042",
  "expires_in": 1800,
  "metadata": {"cart_id": "cart-72"}
}
```

The amount above is 100 tokens **only for a configured token with 18 decimals**. Read `/config` first. The merchant's registered wallet is the immutable recipient. Choose `mode: "escrow"` for escrow. The response contains `intent`, `checkout_id` and `pay_url`. Save the intent ID with your order, then redirect the buyer to the URL. Do not strip its capability fragment.

Create a reusable link with `POST /links`, using `mode`, `token`, `amount`, optional `description`, `expires_in` (60 seconds to 30 days), and optional positive `max_uses` (at most 100,000). The response contains its ID and shareable URL. A guest opens a link without authentication; the hosted checkout creates a separate intent for that attempt.

Limited links reserve capacity transactionally. Paid, funded, disputed and refunded checkouts consume a use. Unpaid reservations remain occupied until their deadline has passed on the confirmed chain and reconciliation proves that they did not pay. A timeout, tab close or RPC failure alone cannot release capacity. Disabling a link stops new checkouts; already issued checkouts remain bound to their original deadline.

## API resources

All paths below are relative to `/api/v1/pay`. List endpoints return `{"items": [...]}` with up to the newest 100 records. Financial records remain in storage; this first API version does not expose arbitrary historical pagination.

| Method and path | Purpose |
|---|---|
| POST `/merchants` | Register with signed wallet session; body `{"name":"Business"}` |
| GET `/merchants/me` | Authenticated merchant and environment |
| GET `/config` | Public allowlisted network and token configuration |
| POST / GET `/intents` | Create checkout / merchant payment history |
| GET `/intents/{pi_id}` | Current verified payment, fees, proof and revision |
| POST `/checkout-sessions` | Alias for merchant intent creation |
| POST / GET `/links` | Create/list merchant links |
| GET `/links/{plink_id}` | Public checkout terms for an enabled link |
| POST `/links/{plink_id}/disable` | Disable a merchant's link |
| POST `/links/{plink_id}/checkouts` | Create a guest checkout; idempotency key and random `X-Pay-Guest-Id` required |
| GET `/checkout-sessions/{cs_id}` | Read only that guest checkout using its capability |
| GET `/escrows` or `/escrows/{pi_id}` | Merchant escrow records; authorized Admin can inspect disputes |
| POST `/intents/{pi_id}/refunds` | Prepare full merchant-funded refund; no amount override |
| POST `/intents/{pi_id}/dispute` | Merchant opens an escrow dispute |
| POST `/escrows/{pi_id}/resolve-release` or `/resolve-refund` | Admin prepares dispute resolution |
| POST / GET `/api-keys` | Create once-displayed key / list metadata |
| POST `/api-keys/{key_id}/revoke` | Revoke key |
| POST / GET `/webhook-endpoints` | Register HTTPS endpoint / list endpoints |
| POST `/webhook-endpoints/{we_id}/rotate` or `/disable` | Rotate signing secret / stop delivery |
| GET `/events` or `/events/{evt_id}` | Immutable merchant event payloads |
| GET `/deliveries` or `/deliveries/{del_id}` | Delivery status and attempt log |
| POST `/deliveries/{del_id}/replay` | Replay an enabled endpoint's event within 30 days |

## Guest capabilities and transaction preparation

The hosted URL is `/checkout/cs_...#token=...`. Its random capability grants access only to that checkout. It is not wallet authentication and does not reveal wallet history, order metadata, merchant keys or deliveries. Store the complete link privately to revisit a payment, release funds or open a dispute. The browser keeps recovery data in session storage.

Guest requests send `X-Pay-Checkout-Token` and need no Authorization header. Prepare `POST /checkout-sessions/{cs_id}/pay` for direct, or `/deposit`, `/release`, `/dispute` for escrow, with `{"payer":"0x..."}` and an idempotency key. The address is unverified input; the contract and chain verifier prove who signed. The first preparation binds that checkout to its payer. Switching wallets requires a new checkout.

Preparation returns an operation ID, `status: "awaiting_signature"`, `transaction_parameters` and, when needed, `approval_transaction`. The wallet sends those transactions. Then POST `{"tx_hash":"0x..."}` to `/operations/{mop_id}/confirm` and poll GET `/operations/{mop_id}`. The operation stays pending until the reconciler verifies the receipt, exact calldata, event, chain, contract, parties, token, integer amounts, fee and block hash with enough confirmations. Posting a transaction hash is never proof of success. Repeated transactions cannot pay or refund the same on-chain identifier twice.

Native/token settlement and refunds are atomic. Direct refunds require the merchant to provide the **full original amount**, including the original fee, and sign with its wallet. The original direct fee remains earned. Escrow payees can refund an active or disputed escrow; payers can release active escrows; either party can dispute; only the configured Admin wallet can resolve a dispute. Escrow never releases automatically.

Admin POST `/contracts/direct/pause` or `/contracts/escrow/pause` with `{"paused":true}` prepares a wallet-signed control operation. Use `false` to resume. Submit and poll `/contract-controls/{mctl_id}/confirm` and `/contract-controls/{mctl_id}`. Pausing affects new payments/deposits only. GET `/contracts` reports current chain pause state.

## Webhook delivery and signatures

| Event | Meaning |
|---|---|
| `payment.succeeded` | Direct payment or escrow release confirmed |
| `escrow.funded` | Confirmed deposit, not merchant settlement |
| `escrow.disputed` | Confirmed dispute |
| `refund.succeeded` | Full refund confirmed |
| `checkout.expired` | Confirmed chain reconciliation released an unpaid reservation |
| `payment.verification_required` | Previously recognized evidence must be reconciled |

The event includes `id`, `api_version`, `type`, `created_at`, `merchant_id`, `environment`, `revision`, and a payment object under `data`. The payment includes its order reference, chain/contract version, token address/decimals, integer amount, earned fee and verified transaction/block proof. Revisions increase per payment, including corrective events after a reorg. The event and delivery jobs commit in the same PostgreSQL transaction as the payment transition.

Verify `EPSX-Pay-Signature: t=UNIX_SECONDS,v1=HEX_SIGNATURE` using HMAC-SHA256 with your full `whsec_...` secret over **ASCII timestamp + `.` + the exact raw request bytes**. Reject timestamps outside a five-minute window, duplicate timestamps, invalid signatures and wrong merchant/environment. Secret rotation sends both old and new `v1` signatures for 24 hours; accept any signature matching your configured secret. The new secret is shown only at rotation. Disabling an endpoint stops pending delivery.

Python verification example (standard library):

```python
import hashlib, hmac, json, time

def verify_event(raw_body: bytes, signature: str, secret: str):
    parts = [part.split("=", 1) for part in signature.split(",") if "=" in part]
    timestamps = [value for key, value in parts if key == "t"]
    if len(timestamps) != 1:
        raise ValueError("Invalid timestamp")
    timestamp = int(timestamps[0])
    if abs(time.time() - timestamp) > 300:
        raise ValueError("Expired signature")
    expected = hmac.new(secret.encode(), str(timestamp).encode() + b"." + raw_body,
                        hashlib.sha256).hexdigest()
    if not any(hmac.compare_digest(expected, value)
               for key, value in parts if key == "v1"):
        raise ValueError("Invalid signature")
    return json.loads(raw_body)
```

Delivery is at least once. Return HTTP 2xx only after your backend has durably accepted the event. Failures retry exponentially for 72 hours; logs/manual replay are available for 30 days. A webhook failure never changes a settled payment to failed. Endpoints must be public HTTPS on port 443; private/reserved IPs, redirects and internal hosts are rejected. DNS is revalidated and pinned for each connection.

Consumers must tolerate duplicates and reordered events, as described in [Stripe's webhook guidance](https://docs.stripe.com/webhooks). Use an event inbox and payment revision, not delivery time, to avoid repeat fulfillment:

```text
verify signature against raw bytes, merchant and environment
GET /intents/{event.data.id} with your server API key
begin database transaction
lock your order; match merchant, order, recipient, chain, contract, token and exact amount
record event ID with a unique constraint
if retrieved payment revision is newer than your stored revision:
    succeeded + verified chain proof -> activate the entitlement for this purchase once
    refunded or verification_required -> revoke/suspend only this purchase's entitlement
    persist the new payment revision
commit; return 2xx
```

A duplicate old event may retrieve a newer revision; apply that newer revision. On a Pay API/RPC outage, retry instead of acknowledging unverified fulfillment. A redirect, browser success screen or incoming chain-hint webhook never grants access.

## Prices

Signup, dashboard, links, API and webhooks have no subscription fee. Direct payments charge 0.5% at settlement. New merchant escrow charges 1% on release. Escrow refunds charge no fee; direct refunds have no additional service fee and retain the original processing fee. Each fee is `floor(amount * fee_bps / 10000)` in token base units. Network gas is separate, paid by the submitting wallet.

For 100 USDT the merchant receives 99.50 USDT directly, or 99 USDT on escrow release. The earlier 0.3% escrow contract and existing package purchases retain their original terms and records.

## Merchant catalog and dashboard

The wallet-owned dashboard, package API, public storefront, immutable checkout snapshot and webhook endpoint replacement are documented in [Merchant dashboard guide](/docs/merchant). In the repository, see `docs/pay/merchant-dashboard-dev.md`. Catalog and webhook management require the owner SIWE session; delivery retry now also requires that session. Existing payment API keys and contract links remain supported.
