# A3.6 Analytics Schema Boundary

Status: **partial static remediation; production readiness STOP**.

The analytics service no longer mutates its schema during startup. Startup now connects to its
database, runs one read-only compatibility query for the legacy `public.events` shape, and fails
before binding a listener when the required table, columns, defaults, or `id` primary key are
incompatible. Every runtime event read and write also names `public.events` explicitly, so
runtime SQL cannot resolve a different relation through `search_path` after the compatibility
probe validates the public relation.

## Exact runtime finding delta

The A3.3 scanner recorded one analytics finding at
`services/analytics/src/lib.rs`: a runtime `CREATE TABLE`. The same scanner has zero findings
under `services/analytics/**/*.rs` after this slice. Relative to the pinned 39-finding A3.3
baseline, this slice's isolated projection is:

| Inventory | Before | Expected after refresh | Delta |
|---|---:|---:|---:|
| Runtime Rust DDL findings | 39 | 38 | -1 |
| Actionable runtime findings | 33 | 32 | -1 |
| Analytics findings | 1 | 0 | -1 |
| Reviewed exceptions | 6 | 6 | 0 |

The global worktree can include other concurrent remediations, so 38 is not asserted as its live
total. This package contributes exactly `-1` finding and does not edit or silently rebaseline the
canonical migration-safety fixture.

## Immutable migration root

The new ordered root contains one transaction-body-only migration:

`services/analytics/migrations/20260722000000_create_events.sql`

Its 260 bytes are pinned by SHA-256 in the contract. It contains exactly one guarded,
additive `CREATE TABLE IF NOT EXISTS public.events` statement with the existing six-column
shape. It contains no destructive statement, data mutation, `ALTER`, schema creation, or
extension creation. Transaction ownership is intentionally left to a future reviewed runner;
embedding transaction control without a runner contract would make atomicity assumptions that
the repository cannot currently prove.

The guard is appropriate for a fresh database and is a harmless SQL no-op when a matching table
already exists. It is not a baseline-adoption protocol: there is no migration-version ledger or
runner to determine whether an existing deployed table may safely adopt this version. An
incompatible existing table remains fail-closed at service startup rather than being rewritten.

## Compatibility contract

The read-only query verifies these existing requirements without adding domain columns:

1. `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`
2. nullable `user_id UUID`
3. `event_name VARCHAR(100) NOT NULL`
4. nullable `properties_json JSONB DEFAULT '{}'`
5. nullable `chain_id VARCHAR(10)`
6. nullable `created_at TIMESTAMPTZ DEFAULT NOW()`

The compatibility boundary is intentionally exact: unknown additional columns fail the check,
as do changes to the types, lengths, nullability, or defaults above; the primary key must be
exactly `id`. A future additive schema change therefore requires an explicit service-contract
update instead of being silently accepted. The query uses only a CTE and catalog reads from
`information_schema`/`pg_catalog`; it does not create, alter, delete, or update database state.
Each required default comparison explicitly coalesces SQL `NULL` to `false`, preventing
`bool_and` from ignoring an indeterminate default comparison.

This deliberately preserves two existing limitations: `user_id` is not canonical wallet
attribution, and counts of `subscription.created` events are not authoritative payment revenue.

## Offline evidence

```bash
scripts/migration/verify-a3-6-analytics-schema-boundary.sh --mode integrity
scripts/migration/verify-a3-6-analytics-schema-boundary.sh --mode report
scripts/migration/test-a3-6-analytics-schema-boundary.sh
```

The verifier pins migration bytes and order, pins the complete 2,743-byte compatibility query,
checks the exact six-column SQL, rejects destructive or unguarded migration text, reproduces the
runtime Rust DDL scanner at zero analytics findings, rejects mutation tokens from the
compatibility query, requires NULL-safe default comparison, proves all seven runtime event SQL
relations are exactly `public.events`, and proves the main-function call occurs after pool
connection and before listener binding and serving.

`--mode readiness` intentionally exits `3`. No database, network, service, container, or migration
execution is performed.

## Residual A3.6 STOP blockers

- No migration runner or version ledger is wired.
- Safe baseline adoption of an already matching deployed table is unproven.
- No populated source-version upgrade has preserved real rows.
- No pre/post reconciliation has run.
- Concurrent migration/startup ordering is untested.
- Neither the migration nor the compatibility check has run against a live database.

Until those six blockers have executable evidence, `productionReady` remains `false`.
