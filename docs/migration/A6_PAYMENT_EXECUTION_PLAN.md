# A6.0 payment execution audit and stop contract

Status: **design/audit only; production readiness is STOP**. This document does not authorize deployment, database access, chain access, or runtime changes. The deterministic contract is [`contracts/payment-execution.json`](contracts/payment-execution.json).

## What must remain compatible

`origin/development` is pinned at `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. Its user checkout submits an included wallet transaction to `POST /api/payments/submit`, polls `GET /api/payments/status/{tx_hash}`, and expects `pending|confirming|confirmed|failed|expired`. It also uses validation, history, plan access, upgrade preview, and plan switch routes. The admin uses `/api/payments/admin/list`, payment-link APIs, plan-access APIs, and admin subscription APIs. The source client contracts and exact blob IDs are locked in the JSON contract.

The canonical backend already has the closest production-shaped flow: authenticated wallet ownership, server-side plan/price checks, atomic submission bookkeeping, unique transaction hash protection, owner-scoped status reads, receipt and ERC-20 `Transfer` verification, and a configurable 15-mainnet/3-non-mainnet confirmation gate. That is evidence, not proof that the extracted microservices are safe or that both write models may run together.

This reconciliation does **not** select that backend, the historical prototype, `services/pay`, or `services/subscription` as the future production write authority. The locked decision remains `unresolved-do-not-cut-over-or-dual-write`, and `productionWriteAuthority` remains `null`.

## Exact schema, route, and database crosswalk

| Model | Pinned schema tables | Route prefix | Database evidence | Reachability evidence |
|---|---|---|---|---|
| Canonical backend | `payments`, `subscriptions`, `stock_ranking_assignments`, `payment_contexts`, `wallet_credits`, `credit_transactions`, `payment_audit_log` | `/api/payments` | Backend runtime compose uses `epsx_payments_dev`, while its migrator uses `epsx_pay_dev`; environment provisioning also names `epsx_payments_{dev,staging,prod}`; unset `PAYMENTS_DATABASE_URL` falls back to primary `DATABASE_URL` | Existing production-shaped writer; this audit does not declare it the future authority |
| Historical `development` prototype at `6fe4d5bb…` | `payment_intents`, `escrows` | `/api/v1/payment` | `epsx_payment` | Historical unauthenticated, runtime-DDL, DB-only writer; provenance only |
| Current pay candidate | `pay_intents`, `escrows`, `pay_links`, `pay_webhook_events` | `/api/v1/pay`, `/api/v1/admin/pay` | `epsx_pay` | Owner/admin reads are principal scoped; owner/internal mutations are `404`; admin mutations are `401`/`403` unless authorized with manage permission, then handler-unavailable `404` |
| Current subscription candidate | `subscription_plans`, `subscriptions` | `/api/v1/subscription` | `epsx_subscription` | Protected router: plan reads/manage require admin audience plus their exact permission; owner lifecycle and vault remain `404` |

The environment provisioning path still emits `epsx_payments_{dev,staging,prod}`. The pay deployment manifest names `epsx_pay`; backend compose points the canonical backend runtime at `epsx_payments_dev` but its migrator at `epsx_pay_dev`. These are evidence of unresolved naming and authority, not aliases proven to contain equivalent data. In particular, renaming `epsx_payment` to `epsx_pay` would not rename `payment_intents` to `pay_intents`. The guarded A3.13 fresh-schema migration has no `ALTER TABLE`, backfill, or adoption ledger, and the old `escrows` shape is not proven compatible with the exact current candidate shape.

## Observed Rust drift

| Surface | Observed contract | Stop reason |
|---|---|---|
| Frontend BFF | `/api/v1/plans`, canned payment/subscription JSON | Missing source-compatible submit/status/validate/history/plan lifecycle |
| `/payment` UI | Redirects to `pay.epsx.io` | Old wizard is unreachable; no verified end-to-end checkout |
| Pay hostname | Cloudflare `4747 -> NodePort 30082 -> pay service` | Bypasses the pay BFF even though the config comment says BFF |
| Pay BFF | singular `/api/v1/pay/intent*`; calls service `/execute` | Service exposes plural `/intents*` and `/confirm`, not `/execute`; no auth context forwarding |
| Gateway | `/api/v1/payment/*` proxy; financial policy blocked | No proven `/payment/* -> /pay/*` rewrite or ownership boundary |
| Pay service | exact read-only A3.13 schema boundary; owner/admin reads are verified-principal scoped | Owner/internal mutations remain `404`; admin mutations are auth-gated `401`/`403` and return handler-unavailable `404` only after manage authorization; inactive bodies still trust caller coordinates and lack execution proof |
| Admin BFF | `/api/v1/payment/*`, empty confirm/release bodies | Downstream authorization is enforced, but valid manage authority still reaches only handler-unavailable `404`; inactive handler DTOs require fields the BFF omits |
| Subscription service | A3.7 schema boundary behind OIDC; plan reads need `admin:plans:read`, plan creates need `admin:plans:manage`; owner lifecycle/vault remain `404` | Unreachable subscription/vault bodies contain caller `user_id`, active-on-insert, and zero-vault behavior; no owner/funding lifecycle, uniqueness proof, reviewed runner/adoption, or deployment |
| Durable data | canonical `payments.*` plus separate pay/subscription tables | No declared system of record, cutover, reconciliation, or outbox |

## Corrected runtime-DDL residual

A3.13 reduced pay-service Rust scanner findings from `10` to `0`. The central A3.3 inventory now has exactly three remaining actionable records, all outside pay:

| Finding | Exact location | Correct meaning |
|---|---|---|
| `finding.002` | `apps/backend/src/bin/blockchain_monitor.rs:84` | Lexical match in “Failed to create database pool”; not schema DDL |
| `finding.003` | `apps/backend/src/bin/migrate.rs:74` | Executable runtime `CREATE DATABASE` bootstrap |
| `finding.004` | `apps/backend/src/main.rs:38` | Lexical match in “Failed to create database pool”; not schema DDL |

The contract pins each file, line, and exact anchor. This is a correction to evidence only; it neither clears A3.3 nor authorizes changing those runtime files.

## Required execution order

1. Declare one write authority and a reversible cutover: retain the canonical backend or move it behind a compatibility adapter; do not dual-write without an outbox, reconciliation ledger, and rollback procedure.
2. Lock the browser/API compatibility layer for all routes in the JSON contract, including method, body, envelope, and status semantics. Choose one canonical prefix; rewrites must be explicit and tested.
3. Enforce identity at gateway and service: the verified JWT wallet is the payer/owner key; server configuration owns receiver, token, chain, amount, and plan price. Hide foreign resources with a uniform `404`.
4. Build on the A3.7/A3.13 candidate migration roots: declare authority, add reviewed runner/adoption/upgrade evidence, durable financial constraints, compare-and-set transitions, idempotency records, inbox/outbox records, and atomic transactions. Removing runtime DDL alone is not financial durability.
5. Verify on chain before activation: configured chain, successful receipt, supported token, exact `Transfer` sender/receiver/amount, block hash/number/log index, and chain-specific finality. Handle reorgs before terminal activation.
6. Lock escrow permissions: participant-scoped reads; payer-or-admin release, payee-or-admin refund, either participant dispute, admin/arbitrator resolution. Admin mutations require admin audience plus `admin:payments:manage`; plan mutation requires `admin:plans:manage`.
7. Make webhooks internal-only with service identity, key ID, signed timestamp, raw-body digest, replay window, and an atomic inbox-transition-outbox transaction keyed by chain event identity.
8. Implement the source-compatible UX against the locked contract. Success pages may render success only from a verified terminal backend state.
9. Add contract, unit, integration, reorg/replay, recovery, reconciliation, and browser tests plus operational metrics/alerts. Only then change blocker states and remove the readiness stop through reviewed evidence.

## Locked status and replay semantics

- Create: `201`; transaction accepted for monitoring: `202`; reads: `200`.
- Authentication: `401`; authenticated but unauthorized: `403`; foreign owner-scoped object: `404`; transition/idempotency conflict: `409`; expired intent: `410`; validation: one documented `400`/`422` policy.
- `Idempotency-Key` is scoped to `(verified principal, operation)` and stores a request hash plus the original response. Same key/body replays the response; same key/different body returns `409`.
- Transaction submission is unique by `(chain_id, tx_hash)`; individual chain events are unique by `(chain_id, tx_hash, log_index)`. A hash already owned by another principal returns a non-leaking `409`.
- State changes use compare-and-set predicates and verify one affected row. Payment/escrow/subscription activation and plan assignment occur only after finality.

## Gate usage

```sh
./scripts/migration/verify-payment-execution.sh --mode integrity
./scripts/migration/verify-payment-execution.sh --mode readiness  # expected exit 3
./scripts/migration/test-payment-execution.sh
```

Integrity validates JSON shape, source ref/commit/blob/anchor pins, local target anchors, path safety, blocker references, and deterministic reporting without network or database access. Integrity passing means only that this audit is intact. Readiness intentionally exits `3` while any stop blocker remains.
