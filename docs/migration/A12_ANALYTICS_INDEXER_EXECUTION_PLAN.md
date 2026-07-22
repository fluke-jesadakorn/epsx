# A12.0 analytics and indexer execution audit

Status: **audit/design only; production readiness is STOP**. This plan does not authorize runtime edits, database/Redis/RPC/market-provider access, backfills, deployment, or production traffic. Its machine-enforced source of truth is [`contracts/analytics-indexer-execution.json`](contracts/analytics-indexer-execution.json).

## Four boundaries, not one analytics bucket

The Rust target currently contains four independent contracts. They must not share authority merely because several use the words analytics or ranking.

| Domain | Code owner | Data authority | Current production blocker |
|---|---|---|---|
| Market analytics | `apps/analytics` | External market observations normalized into ranked/filterable results | The deployed binary exposes root-level routes and live TradingView wiring, but it has no proven direct auth boundary, source compatibility, freshness contract, or reliable entitlement lookup. |
| Event analytics | `services/analytics` | First-party product events and derived operational/product aggregates | A guarded migration and exact read-only compatibility probe replace startup DDL, but no runner/adoption/live-database proof exists; subject attribution, event semantics, and revenue authority remain blocked. |
| Chain indexer | `services/indexer` | Canonical blocks, transactions, receipts and decoded logs | A chain-scoped projection migration/probe replaces startup DDL and fake sync, but no runner/adoption proof, ingestion worker, durable checkpoint, or reorg/finality/backfill model exists. |
| Identity ranking offset | `shared/rust/epsx-identity-service` | Backend-owned wallet plan/permission projection | Its query always returns free-plan offset and its unauthenticated emit plus ephemeral SSE path is not a durable entitlement projection. |

This separation is an architecture constraint: plan/ranking entitlement decisions remain backend-only. Market analytics consumes a ranking offset but never computes plan access. Event analytics never becomes revenue authority. The indexer projects chain facts but never decides payment or subscription state.

## Pinned compatibility baseline

`origin/development` is fixed at `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. Fourteen blob-and-anchor records lock the source behavior:

- public rankings are capped at 10 and remove advanced financial filters;
- authenticated rankings and filters have separate paths and use plan-aware server state;
- the analytics page loads plan access/watchlist and supports filter/search/pagination behavior;
- portfolio access is authenticated;
- admin analytics has separate permission/system/revenue contracts;
- ranking offset is resolved by the backend permission service before the TradingView query;
- the source schema and source browser-flow evidence are pinned independently.

The source commit has no standalone indexer route contract. A12 therefore treats indexer routes as new Rust contracts; it does not invent legacy parity. Newness does not weaken production requirements for canonical chain correctness, authorization, durability, backfill, reconciliation, and rollback.

## Observed target drift

### Market analytics

`apps/analytics` is deployed under the `epsx-analytics` workload and serves `/rankings`, `/filters`, `/countries`, `/available-countries`, `/sectors`, and `/v1/rankings/stream`. The legacy paths are `/api/public/analytics/*` and `/api/analytics/*`. Meanwhile the gateway's `/api/v1/analytics/*` proxy targets the event service default on port 8107. There is no demonstrated path map selecting the intended binary.

The market service retains the monolith's TradingView implementation through re-exports. Its identity gRPC call has a 100 ms timeout and falls back to a free-plan adapter. The identity server itself is also a free-plan stub, so successful and failed calls currently converge on the same entitlement result. Health proves only that the listener responds; it does not prove provider, identity projection, SSE freshness, or usable ranking data.

### Event analytics

`services/analytics` now has a direct JWT boundary: tracking requires a verified frontend/admin audience and reads require admin audience plus `admin:analytics:view`. That is useful evidence, not end-to-end readiness.

The service now uses a guarded additive `public.events` migration and exact read-only startup compatibility probe. That static boundary has no reviewed runner/version ledger, populated-baseline adoption, source-version upgrade, concurrent-startup, or live-database proof. It still stores `NULL` for canonical subjects because its column is UUID while subjects are wallets, accepts arbitrary event names/properties, and has no producer event ID or deduplication contract. Its revenue endpoint counts `subscription.created`; it explicitly notes that payment integration is required for exact values. The frontend tracking adapter returns `{"ok":true}` even when the upstream fails, and SSR asks for `/api/v1/analytics/summary`, which the service does not expose.

### Indexer

`services/indexer` now opens its database and runs an exact read-only compatibility probe against a guarded additive projection migration. Transactions use the chain-scoped key `(chain_id, hash)`. The former default-on provider, in-memory cursor, autonomous loop, number-derived placeholder writes, and conflict-skipping fake sync are removed. This is an inert fail-closed shell, not an indexer: no runner/version ledger or populated adoption is proven, and no worker ingests canonical blocks, transactions, receipts, raw logs, or decoded transfers.

The direct service narrows `/sync` to POST, verifies the admin audience plus `admin:indexer:manage`, and returns 404; the explicit handler is also unavailable. Status, block, transaction, and transfer handlers return 404 at the boundary because their truth/finality contracts are not proven. This closes anonymous direct execution but does not make the indexer usable: the dormant status projection has no canonical head, durable checkpoint, chain-specific finality, or ingestion freshness, and production manifests do not prove a separately deployed indexer workload.

### Identity ranking projection and UI truth

The identity HTTP side publicly mounts `POST /v1/emit` beside the ranking-offset SSE stream. The stream is an in-memory broadcast: lagged events can be dropped and there is no revision/cursor/replay repair. Neither this stream nor the gRPC query is backed by active plan assignments.

The Dioxus analytics page uses sample rankings, sample events and generated charts while rendering a `Live` label. Portfolio uses six static stocks. Those fixtures are appropriate for isolated visual tests only; they are a release blocker when presented as production data.

## Locked execution sequence

1. **Name and route the four domains.** Use distinct service/workload/upstream names and write one route-ownership table. Preserve legacy market paths with an explicit adapter; give event analytics and indexer distinct prefixes. Unknown paths and methods return `404` before upstream/data access.
2. **Lock typed contracts.** For each of the 16 surface contracts, fix method, path, query/body, envelope, statuses, pagination/cursor, freshness and error behavior. Preserve the source's public/auth distinction and public rank cap.
3. **Close every direct-service boundary.** Health alone may be public. Market authenticated routes derive wallet from a verified token. Event admin reads keep exact audience/permission rules. The indexer now fails non-health handlers closed and verifies admin sync credentials; decide whether address history is deliberately public or owner/admin restricted before enabling truthful reads or sync.
4. **Complete the migration boundaries.** Guarded additive analytics and chain-scoped indexer projection migrations now exist and startup only validates their exact shapes. Wire reviewed runners/version ledgers, populated-baseline adoption, source-version upgrades, reconciliation, and concurrent-startup proof; add canonical/finalized/fork/receipt/raw-log/checkpoint and event-contract structures only through later reviewed migrations.
5. **Make ranking entitlement authoritative.** Identity resolves the minimum effective offset from active backend plan assignments, including overlap, scheduled start/expiry, cancellation and downgrade. Publish a monotonic projection revision through a transactional outbox; authenticate publishers; persist consumer cursor; reconcile gaps. A fallback must be explicitly safe and visible, not silently equivalent to success.
6. **Build the real indexer.** Fetch block, transaction, receipt and log data from the configured chain. Validate returned chain/hash/parent continuity. Atomically persist a block and its dependent records, then advance a leased durable checkpoint. Track canonical and finalized heads, detect divergence, orphan/replay affected ranges, decode allowlisted events, and test multi-block reorgs.
7. **Build bounded backfill.** A job declares chain, inclusive range, confirmation policy and provider budget. It has a unique idempotency key, lease, adaptive rate limit, retry budget, poison-record outcome, progress, pause/cancel and resume. Never let a synchronous public request launch an unbounded scan.
8. **Version event analytics.** Maintain an allowlisted event catalog with property schemas, producer/service identity, event ID, occurred/received timestamps and deduplication. Store canonical wallet subject in a compatible typed column or mark an event anonymous. Enforce size, privacy, redaction, retention, deletion and export rules. Revenue reads a reconciled financial projection rather than counting events.
9. **Wire truthful UX.** The BFF returns typed market/event payloads with provenance and `observed_at`/watermark. UI provides loading, empty, error, stale and offline states. Search/filter/sort/pagination/export/watchlist and plan range are tested on mobile and desktop against deterministic local fixtures shaped exactly like real contracts. Never display `Live` for static or stale data.
10. **Observe and reconcile.** Market alerts cover provider errors/throttling/cache age; event alerts cover rejects/dedup/watermark; indexer alerts cover canonical/finalized lag/reorg/backfill; identity alerts cover fallback/gaps/projection mismatch. Reconciliation emits counts, checksums, mismatch samples and repair outcomes per domain.
11. **Shadow and cut over one domain at a time.** Shadow reads and isolated/idempotent ingestion must not create dual write authority. Promotion requires a recorded parity window, zero unresolved reconciliation drift, accepted error budget and a tested rollback checkpoint.
12. **Request deployment authorization separately.** Passing A12 later would mean code/data/operational evidence is complete; it would not authorize production deployment.

## Acceptance evidence by domain

### Market analytics

- exact compatibility tests for public/auth rankings, filters, countries and sectors;
- direct-service auth/owner tests, including spoofed headers and unknown route/method drift;
- deterministic provider adapter tests plus bounded local fault injection for timeout, quota and malformed payload;
- freshness/provenance/cache policy tests and plan-offset revision tests;
- strict browser coverage for anonymous/free/paid, filter/search/sort/pagination/export/watchlist, stale/error/empty states.

### Event analytics

- additive migration and rollback-forward verification on disposable local databases only;
- event catalog/schema/idempotency/property-bound tests;
- canonical/anonymous attribution and admin access tests;
- late/out-of-order event aggregation, retention/redaction/deletion tests;
- revenue reconciliation against payment/subscription fixture authority.

### Indexer

- deterministic RPC fixture tests for block/transaction/receipt/log persistence;
- restart/checkpoint lease, concurrent worker and idempotent replay tests;
- single/multi-block reorg, finality and orphan-repair tests;
- bounded backfill pause/resume/cancel/rate-limit/poison-record tests;
- reconciliation against independent RPC fixtures and cursor pagination contract tests.

### Identity ranking offset

- active-plan overlap/start/expiry/cancel/downgrade resolution tests;
- exact internal publisher authorization and replay rejection;
- monotonic revision, durable outbox/cursor, disconnect/replay and gap-repair tests;
- reconciliation from plan assignments to projection and market-consumer cache state.

## Gate usage

```sh
./scripts/migration/verify-analytics-indexer-execution.sh --mode integrity
./scripts/migration/verify-analytics-indexer-execution.sh --mode report
./scripts/migration/verify-analytics-indexer-execution.sh --mode readiness  # expected exit 3
./scripts/migration/test-analytics-indexer-execution.sh
```

The verifier reads only local Git objects and local files. It refuses database, Redis, chain/RPC, live-market-data and production-looking environment variables, removes proxy variables, and never starts a service. Integrity passing proves only that the 14 source pins, target anchors, domain separation and 24-blocker STOP contract are internally consistent.
