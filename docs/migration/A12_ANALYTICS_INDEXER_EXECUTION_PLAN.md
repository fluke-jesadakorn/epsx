# A12.0 analytics and indexer execution audit

Status: **audit/design only; production readiness is STOP**. This plan does not authorize runtime edits, database/Redis/RPC/market-provider access, backfills, deployment, or production traffic. Its machine-enforced source of truth is [`contracts/analytics-indexer-execution.json`](contracts/analytics-indexer-execution.json).

## Four boundaries, not one analytics bucket

The Rust target currently contains four independent contracts. They must not share authority merely because several use the words analytics or ranking.

| Domain | Code owner | Data authority | Current production blocker |
|---|---|---|---|
| Market analytics | `apps/analytics` | External market observations normalized into ranked/filterable results | The deployed binary exposes root-level routes and live TradingView wiring, but it has no proven direct auth boundary, source compatibility, freshness contract, or reliable entitlement lookup. |
| Event analytics | `services/analytics` | First-party product events and derived operational/product aggregates | The gateway points to this different analytics API; it uses runtime DDL, discards subject attribution, has open-ended event semantics, and estimates revenue from event counts. |
| Chain indexer | `services/indexer` | Canonical blocks, transactions, receipts and decoded logs | Its non-health handlers now fail closed at a direct shared-JWT boundary, but it still writes placeholder block records, keeps the sync cursor in memory, and has no reorg/finality/backfill model. |
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

The service creates `events` at startup, stores `NULL` for canonical subjects because its column is UUID while subjects are wallets, accepts arbitrary event names/properties, and has no producer event ID or deduplication contract. Its revenue endpoint counts `subscription.created`; it explicitly notes that payment integration is required for exact values. The frontend tracking adapter returns `{"ok":true}` even when the upstream fails, and SSR asks for `/api/v1/analytics/summary`, which the service does not expose.

### Indexer

`services/indexer` opens its database and performs DDL at startup. `transactions.hash` is globally primary rather than chain-scoped. The process starts at an in-memory `last_block = 0`; it does not recover that cursor from the durable head. Its ingestion loop inserts number-derived placeholder hashes, zero gas and no transactions or transfers. `ON CONFLICT ... DO NOTHING` cannot repair changed canonical ancestry.

The direct service now narrows `/sync` to POST, verifies the admin audience plus `admin:indexer:manage`, and still returns 404 before the unsafe sync handler. Status, block, transaction and transfer handlers also return 404 at the boundary because their truth/finality contracts are not proven. This closes anonymous direct execution but does not make the indexer usable: status still labels any lag under 100 as healthy without chain-specific finality or a degraded reason, and production manifests do not prove a separately deployed indexer workload.

### Identity ranking projection and UI truth

The identity HTTP side publicly mounts `POST /v1/emit` beside the ranking-offset SSE stream. The stream is an in-memory broadcast: lagged events can be dropped and there is no revision/cursor/replay repair. Neither this stream nor the gRPC query is backed by active plan assignments.

The Dioxus analytics page uses sample rankings, sample events and generated charts while rendering a `Live` label. Portfolio uses six static stocks. Those fixtures are appropriate for isolated visual tests only; they are a release blocker when presented as production data.

## Locked execution sequence

1. **Name and route the four domains.** Use distinct service/workload/upstream names and write one route-ownership table. Preserve legacy market paths with an explicit adapter; give event analytics and indexer distinct prefixes. Unknown paths and methods return `404` before upstream/data access.
2. **Lock typed contracts.** For each of the 16 surface contracts, fix method, path, query/body, envelope, statuses, pagination/cursor, freshness and error behavior. Preserve the source's public/auth distinction and public rank cap.
3. **Close every direct-service boundary.** Health alone may be public. Market authenticated routes derive wallet from a verified token. Event admin reads keep exact audience/permission rules. The indexer now fails non-health handlers closed and verifies admin sync credentials; decide whether address history is deliberately public or owner/admin restricted before enabling truthful reads or sync.
4. **Move all DDL into migrations.** Add guarded, additive event and indexer migrations. Use chain-scoped keys, canonical/finalized flags, event IDs, indexes, check constraints, migration ledger and forward-only rollback procedures. Startup only validates schema compatibility.
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
