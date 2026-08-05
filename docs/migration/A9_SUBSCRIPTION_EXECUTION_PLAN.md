# A9.0 Subscription Lifecycle Execution Plan

Status: **audit complete; execution blocked**
Readiness verdict: **STOP (reserved exit 3)**
Canonical comparison: `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`
Machine contract: `docs/migration/contracts/subscription-execution.json`

This gate turns the subscription migration into an executable, evidence-pinned plan. It does not modify runtime code, schemas, fixtures, infrastructure, databases, Redis, or a chain. Passing integrity proves only that the audit still matches the pinned source and the present target. It does not make the target production-ready.

## Outcome

The current Rust subscription slice is not safe for production traffic. A3.7 removed both startup DDL findings into one additive candidate migration and added a read-only startup compatibility probe, but there is no reviewed runner or ledger and no baseline adoption, populated upgrade, reconciliation, concurrent-startup, or live-database proof. The runtime still treats a caller-supplied subscription as immediately active, trusts caller owner fields, lists and mutates records without an owner predicate, has no verified-payment consumer, has no renewal/expiry worker, does not grant or revoke entitlements, and is disconnected from the ranking projection. The public `/plans` path now removes its top-level compatibility producer, fallback loader, static catalog, and checkout mutation and fails closed; `/api/v1/subscription/plans` remains canned, and the admin sample surface plus backend/service contract drift remain production blockers.

There are **20 stop blockers**. The implementation must retain the source's externally relied-on behavior while hardening source weaknesses: non-persistent manual activation, swallowed permission errors, non-atomic plan switches, cross-database monitor writes, and replay-prone extension behavior are evidence of required remediation, not behavior to reproduce.

## Evidence boundary

The contract pins every source record by Git blob plus a literal anchor. The verifier also checks live target paths and anchors. Important source facts are:

- `shared/api/plans.ts` defines `PublicPlan`, `SubscriptionResponse`, source admin CRUD, and `GET /api/payments/plans/my-plan-access`.
- `shared/api/credits.ts` posts `{new_plan_id}` to `/api/payments/plans/switch`.
- `subscription_handlers.rs` derives the owner from `OpenIDUserContext`, reads `wallet_plan_assignments`, accounts for grace periods, picks the lowest ranking offset, and explicitly uses manual repayment instead of auto-renewal.
- The source plan switch deactivates the old assignment and inserts the new one as separate statements without visible transaction proof.
- `validation_handlers.rs` returns a generated subscription identifier for manual activation but does not persist the subscription; permission-grant failures are logged and ignored.
- `tx_monitor_service.rs` waits for confirmation depth, then changes plan assignments in the primary database and payment state in the payments database. Those cross-store writes are not atomic and assignment errors are sometimes discarded.
- The pinned subscription migration uses `UNIQUE(wallet_address, plan_id)`. That permits different plans to remain active concurrently and also mixes history with effective-state uniqueness; it is not proof of one effective subscription plan.
- Source frontend access helpers convert dependency failure into a default free tier. That is useful UX fallback behavior but unsafe as an authorization or entitlement assertion.

The present target facts are:

- A3.7 reduced subscription runtime DDL from **2 to 0**, pinned one **844-byte** additive migration (`20f38597d2d64bad3589036c2fe20aab2be89e5d240c540d401b46713c701349`), and makes startup run `verify_schema_compatibility(&db)` before binding the listener.
- That boundary is static and partial: the migration root has no reviewed runner or version ledger, and no baseline-adoption, populated-upgrade, reconciliation, concurrent-startup, or live-database proof exists. The two-table candidate is not a proven mapping from the development wallet-plan source of truth.
- `POST /api/v1/subscription/subscriptions` accepts `user_id`, `account_id`, and `payment_token` from the caller. The database default is `active`, and period boundaries are unset.
- Subscription list, detail, and cancel SQL use no verified owner predicate. Gateway authentication cannot compensate for missing service-level ownership.
- The service has no current-access, preview, switch, renew, expiry, entitlement, reconciliation, or lifecycle-event endpoint/worker.
- The vault response contains a zero address and canned token/rate data.
- The admin BFF exposes list/detail/cancel and plan create/read only. It forces `201` around a service response and posts an empty cancel body; the admin plan UI renders `sample_plans()`.
- The top-level `/api/v1/plans` compatibility producer and multi-endpoint SSR fallback are removed, and the Dioxus plans page renders an explicit unavailable state with no catalog, eligibility, or payment mutation. The mounted `/api/v1/subscription/plans` producer still returns canned `sub_1`/`sub_2` records, and a separate non-persistent subscribe echo is not a usable lifecycle flow.
- Checkout creates a pay intent without an immutable `plan_id` or plan price version. Pay confirmation does not activate subscription or entitlement state.
- `FreePlanRankingOffsetService` returns free-plan ranking access for every wallet, including paid users.

The exact 18 source and 25 target anchors are intentionally kept in the machine contract so stale evidence fails before any implementation is accepted.

## Contract inventory

| Capability | Pinned source contract | Current Rust target | Required production contract |
|---|---|---|---|
| Public plan read | `GET /api/public/plans` → `ApiResponse<PublicPlan[]>` | Frontend compatibility subscription producers are absent; top-level route/loader/UI fail closed | One versioned projection of canonical backend plans; live loading/empty/error/retry UI |
| Current access | `GET /api/payments/plans/my-plan-access` → `ApiResponse<PlanAccessData>` | Absent; ranking always free | JWT-owner read of effective plan, period, grace, permissions, and ranking offset |
| Owner create/activation | Payment submission followed by confirmed-chain monitor | Direct caller create becomes active | Pending payment or trusted finalized-payment event; never public direct-active creation |
| Owner list | Authenticated wallet assignment projection | Returns all subscription rows | Owner derived from JWT and present in the SQL predicate |
| Owner detail/expiry | Owner-scoped expiry projection | Select by subscription id only | Owner+id lookup; foreign and missing both return 404 |
| Owner cancel | Owner wallet deactivation | Update by id only | Owner or authorized admin, explicit immediate/period-end semantics, audit and outbox |
| Preview/switch | Owner preview and `{new_plan_id}` switch | Absent | Canonical price snapshot, deterministic proration, one transaction, concurrency proof |
| Renewal/expiry | Manual new payment extends; grace is read | Absent | One extension per verified payment; leased expiry/grace/revoke worker |
| Admin subscriptions | Source client expects create/list/detail/update/cancel | BFF supports list/detail/cancel only | `admin:payments:view/manage`, live CRUD, exact DTO/envelope/status adapter |
| Admin plans | Backend plan CRUD | Second service-owned plan store; partial BFF | Backend-only business authority, `admin:plans:read/manage`, projected reads |
| Entitlements | Assignment/permission data feeds access | No grant/revoke path | Transactional lifecycle outbox and idempotent entitlement projection |
| Ranking offset | Effective plan chooses best offset | Free-plan stub | Authoritative projection; dependency failure distinct from free/no-plan |

### Prefix, body, envelope, and status rule

Migration may introduce `/api/v1/...` internally, but source consumers need a versioned compatibility adapter until every caller moves. The adapter owns translation; handlers must not guess among multiple bodies or silently change an envelope.

The locked semantics are:

- `202` for payment-backed activation accepted for monitoring, not active success.
- `201` for an authorized administrative create.
- `200` for successful reads, cancel, preview, and switch.
- `400` for malformed/invalid input, `401` for missing authentication, `403` for valid principals without admin permission, `404` for missing or foreign owner resources, `409` for lifecycle conflicts or idempotency key/request-hash mismatch, and `503` for an unavailable required dependency.
- Owner mutations use an `Idempotency-Key` and canonical request hash. Same key/same body returns the stored response; same key/different body returns `409`.

## Authority and invariants

### Plan authority

Per repository architecture, Rust backend business logic remains the only authority for plan eligibility, permission mapping, price/billing cycle, promotion, active flag, ranking offset, and subscription rules. The subscription service may keep an immutable, versioned projection for lifecycle processing. It may not introduce an independently editable merchant catalog. Content files and former frontend compatibility producers are fixtures or display projections, never authority; the public `/plans` page and admin wallet-plan list/detail now present none of them and instead fail closed until typed backend adapters exist.

### Owner and service authorization

- Public access is limited to active public plan projections.
- User current-access/list/detail/cancel/preview/switch derive the owner from verified JWT claims. Caller-provided `user_id`, wallet, or account is ignored for authorization.
- Every owner SQL statement includes the normalized owner key. A foreign identifier is indistinguishable from a missing identifier (`404`).
- Admin subscription reads require `admin:payments:view`; mutations require `admin:payments:manage`.
- Admin plan reads require `admin:plans:read`; mutations require `admin:plans:manage`.
- Payment activation is internal-only and authenticates the payment service. The event wallet, plan, amount, token, receiver, chain, and price version must match the immutable verified payment record.

### Effective subscription uniqueness

A wallet has at most one effective active plan in the subscription category. Historical rows are retained. Enterprise, manual, API-developer, or system grants may overlap only under a written category policy, and best-access selection must remain deterministic.

Database proof must include:

1. a normalized owner key;
2. valid status/period checks;
3. a partial unique or exclusion constraint covering the effective subscription category;
4. transactional create/switch/cancel behavior; and
5. concurrent create and switch integration tests.

`UNIQUE(wallet_address, plan_id)` alone is insufficient: it permits two active plans with different IDs and complicates historical reuse.

### Lifecycle

The required states are `pending_payment`, `active`, `grace_period`, `cancel_at_period_end`, `cancelled`, `expired`, and `suspended`.

- Only a verified, finalized payment can move `pending_payment` to `active` for a purchase.
- A verified renewal extends exactly once from `max(current_period_end, verified_at)` using the canonical billing period.
- Auto-renew remains false until a separate payment-mandate design is approved. The pinned source is manual repayment.
- Owner cancellation may set `cancel_at_period_end` when access was paid through the period. Authorized risk/admin cancellation may be immediate, but requires reason and audit.
- A leased worker transitions due records through grace/expiry. It appends an outbox event in the same local transaction.
- Cancelled or expired state cannot reactivate without a new verified cause. A switch cannot report success while leaving two effective plans or no effective plan.

### Idempotency, outbox, and reconciliation

The immutable finalized payment key is `(chain_id, transaction_hash, log_index)`. It is unique at the subscription activation boundary. A retry cannot add another period.

Lifecycle row mutation, audit entry, and outbox append commit together. Entitlement consumers record `event_id` before applying grant/revoke effects. Scheduler and consumer crashes are covered by leases/inbox records and bounded retry. A dead-letter entry never becomes a silent free-plan or active result.

Reconciliation walks payment → subscription → lifecycle events → entitlement/ranking projection. It reports deterministic discrepancy classes and repairs missing projections from cause history without changing billing periods. Ambiguous legacy records are quarantined, not assigned invented payment proof.

## Execution order and agent handoff

Work packages are dependency-ordered. Agents may implement independent tests or adapters within a package, but no later package can claim acceptance while its dependency is blocked.

### 1. Freeze contract and authority

Owner: backend/API contract agent.
Deliver exact versioned DTOs, source compatibility routes, owner keys, status/envelope table, manual-renewal semantics, plan category overlap rules, and the single backend plan authority.

Acceptance: contract fixtures cover every row in the inventory; no body field is simultaneously an authorization source; architecture review confirms plan logic remains backend-only.

### 2. Add durable schema and audit

Owner: subscription persistence agent.
Build on the narrow A3.7 candidate migration with reviewed additive migrations for plan projections, lifecycle state/period, immutable payment cause, idempotency response, outbox/inbox, audit, scheduler lease, dead letter, and reconciliation results. Add a reviewed runner and version ledger; prove baseline adoption and populated upgrade before any migration execution is authorized.

Acceptance: migration up/down safety is reviewed, the runner/ledger and adoption flow are proven, no destructive data rewrite exists, invalid statuses/periods and duplicate effective subscriptions are rejected, and startup remains free of schema mutation.

### 3. Inventory, backfill, and reconcile

Owner: data migration agent.
Choose the subscription system of record across `subscriptions` and `wallet_plan_assignments`; build restartable dry-run/apply batches for normalized owners, categories, plan versions, periods, payment causes, lifecycle history, permissions, and ranking.

Acceptance: before/after counts are reproducible, ambiguous rows are quarantined, rerun is a no-op, and payment/subscription/entitlement reconciliation has zero unexplained discrepancies.

### 4. Secure owner, admin, and service boundaries

Owner: auth/gateway/subscription agent.
Propagate verified request context, enforce owner predicates inside the service, authenticate payment-service events, and apply canonical admin permissions.

Acceptance: negative tests prove caller owner fields cannot override claims, user A cannot list/read/cancel/switch user B, foreign IDs return 404, missing auth returns 401, and insufficient admin permission returns 403.

### 5. Integrate verified payment activation

Owner: payment/subscription integration agent; depends on A6 payment readiness.
Add immutable plan/price version to pay intent, consume only finalized verified events, compare amount/token/receiver/chain/wallet, and apply the unique payment identity.

Acceptance: unverified, wrong-recipient, wrong-amount, wrong-token, wrong-chain, reorged, duplicate, reordered, and crash/retry cases cannot grant or extend access; accepted monitoring is `202` and active appears only after finality.

### 6. Implement owner/admin lifecycle

Owner: subscription domain agent.
Implement current access, owner list/detail/cancel, admin create/update/cancel, preview/switch, renewal period math, grace, expiry scheduler, audit, and compatibility adapters.

Acceptance: the locked state machine and HTTP matrix pass unit/integration/concurrency tests; a switch is one transaction; renewals extend once; scheduler reruns are no-ops; source client fixtures pass.

### 7. Project entitlement and ranking state

Owner: identity/analytics agent.
Consume lifecycle events, grant/revoke plan permissions, publish the effective ranking offset, and add replay-safe reconciliation.

Acceptance: activation, switch, cancellation, grace, expiry, suspension, event duplication, out-of-order delivery, dependency outage, and recovery yield the expected permissions and ranking. Protected access fails truthfully; service failure is not interpreted as free tier.

### 8. Complete BFF and UI behavior

Owner: frontend/admin UX agent.
Replace canned/static plan/subscription data, add exact adapters, link checkout to plan/price version, and render explicit state machines.

Acceptance includes:

- public plan loading, non-empty, honest empty, error, and retry;
- current-access loading, no-plan, active, expiring, grace, cancelled/expired, error, and retry;
- checkout pending finality, confirmed activation, replay-safe retry, rejected payment, and terminal failure;
- owner/admin list loading, empty, error, retry, pagination, forbidden, conflict, and mutation-in-progress;
- no production code calls `default_plans()`, `sample_plans()`, canned subscribe success, or empty-intent navigation as a successful fallback.

### 9. Shadow, canary, cut over, and roll back

Owner: release/observability agent.
Deploy behind routing flags only after packages 1–8. Shadow source reads, compare effective plan/period/status/ranking/permissions, canary mutations, observe lifecycle/outbox/reconciliation SLOs, and drill rollback.

Acceptance: route/contract fixtures, full lifecycle integration, shadow comparison, canary, alerting, reconciliation, and rollback drill all pass. Source write authority remains enabled until the cutover decision is recorded.

## Required production test matrix

At minimum, implementation evidence must cover:

- public and authenticated route prefixes, request bodies, response DTOs, envelopes, and every locked status;
- missing/invalid JWT, wrong audience, expired session, caller owner spoof, foreign identifier, and each admin permission boundary;
- concurrent purchase and switch, duplicate idempotency key with same/different request, duplicate finalized event, worker crash before/after commit, and consumer replay;
- exact amount/token/receiver/chain, finality threshold, reorg, and payment-to-plan price-version mismatch;
- period boundary, daylight-saving-independent UTC math, grace disabled/enabled, cancellation timing, suspension/recovery, and manual renewal;
- entitlement grant/revoke/ranking selection after activation, switch, cancellation, grace, expiry, and reconciliation;
- migration dry run, interrupted batch restart, ambiguous legacy quarantine, constraint validation, and rerun no-op;
- UI live loading/empty/error/retry/forbidden/conflict/pending/success states with no canned-data success path.

## Observability and stop triggers

Production dashboards and alerts must expose activation latency from payment finality, duplicate/replay conflicts, lifecycle transition failures, due-expiry lag, outbox oldest age/retry/dead-letter count, entitlement projection lag, reconciliation discrepancies, owner-denial counts, and source-versus-target shadow mismatches. Logs and traces carry payment cause, lifecycle event, and subscription IDs but never secrets or bearer tokens.

Cutover stops immediately for owner leakage, duplicate activation/extension, an unexplained effective-plan/expiry/ranking/permission mismatch, outbox or entitlement lag above the agreed SLO, or an unexplained reconciliation discrepancy.

Rollback routes mutations back to the source backend, stops new Rust subscription mutations, quarantines undelivered events without replaying billing effects, preserves additive schema and audit history, and reverses only incorrect entitlement projections using immutable cause records. Destructive schema or history rollback is forbidden.

## Gate commands

These commands are deterministic and local. They refuse database, Redis, chain, network, deploy, and production-looking environment inputs.

```bash
scripts/migration/verify-subscription-execution.sh --mode integrity
scripts/migration/verify-subscription-execution.sh --mode report
scripts/migration/test-subscription-execution.sh
scripts/migration/verify-route-inventory.sh
scripts/migration/verify-contract-fixtures.sh
```

`--mode integrity` must exit `0`. `--mode report` must emit byte-stable JSON for unchanged inputs. `--mode readiness` must exit `3` while any of the 20 blockers remains. Exit `3` is not an error in the verifier; it is the reserved truthful readiness stop.

Readiness may change only through a new reviewed contract revision that replaces blocker assertions with concrete migration, test, runtime, shadow, canary, observability, and rollback evidence. Editing `productionReady` or blocker status alone is a tamper failure, not a readiness transition.
