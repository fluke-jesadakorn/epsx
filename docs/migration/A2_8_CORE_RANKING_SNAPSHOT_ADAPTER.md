# A2.8 core ranking snapshot adapter

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A2.8 adds a static, unwired repository adapter at the core database boundary.
The adapter owns one schema-qualified, read-only Diesel SQL statement that
returns the raw plan, assignment and permission facts needed by the A2.7 pure
resolver. The raw snapshot DTO, repository error and repository port live in
`epsx-contracts`; resolution policy remains in the identity crate.

The statement obtains one epoch-microsecond `observed_at` from PostgreSQL
`statement_timestamp()`, selects raw facts through `LEFT JOIN`s, and returns a
sentinel row when the requested wallet has no assignments. It deliberately does
not filter assignment activity, expiry, plan activity, permission activity or
ranking namespace in SQL. The pure A2.7 resolver remains the only place that
decides whether those facts contribute to ranking entitlement.

This is static evidence only. The identity runtime and its gRPC service remain
byte-identical and always return the Free Plan offset. No database is opened,
no statement is executed, no schema or migration is changed, and no production
surface is activated.

## Frozen source and target

The source baseline is `origin/development` at
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. The source ranking query and both
colliding core baseline migrations are pinned. The v5 and v6 baselines each
define the plan, assignment and permission tables, but they are not evidence
that any particular deployed database adopted either definition.

The generated Diesel schema is also pinned. It omits
`wallet_plan_assignments`, and the primary `diesel.toml` table filter does not
include that table. A2.8 therefore uses checked, schema-qualified SQL rather
than claiming the generated schema is complete. It does not regenerate schema
or modify a migration.

The immutable target base is
`a8469ff61a8782dc1d53b8dcae20ad7c1085d4a1`, the completed post-A2.7 snapshot.
The A2.7 resolver and its always-Free runtime boundary remain historical
authority for this slice.

## SQL and decoder boundary

The adapter statement is constrained to one read-only statement with a single
wallet bind parameter. Every table reference is explicitly in `public`; the
selected column order and aliases are frozen by the machine contract. A
database-sourced observation instant is repeated across the returned rows and
decoded strictly. Mixed observation instants, partial sentinel rows, missing
required assignment facts, and structurally impossible nullable combinations
fail closed as corrupt repository data.

The sentinel row distinguishes an authoritative empty snapshot from transport
or decoding failure without inventing a synthetic assignment. Raw joins retain
inactive, expired, unrelated and malformed candidate facts so the existing pure
resolver can apply its fixed policy consistently. No SQL-side entitlement
minimum, Free Plan fallback, expiry rule, grace rule or ranking-permission
parser is introduced.

The checked-in fixture ledger and exact hermetic tests exercise SQL-shape
inspection and the pure row decoder only. They do not execute PostgreSQL or
prove a query plan, lock behavior, isolation level, snapshot consistency under
concurrent writes, schema adoption or production data compatibility.

## Explicit performance and schema STOPs

Both source baselines provide ordinary and partial indexes on
`wallet_plan_assignments(wallet_address, ...)`, while source-compatible lookup
uses `LOWER(wallet_address)`. Neither pinned baseline defines a matching
functional index on `LOWER(wallet_address)`. A2.8 adds no migration, so the
missing functional index remains a STOP.

The raw `LEFT JOIN` statement intentionally has no row limit because truncating
owner facts could change entitlement. Consequently row fan-out, query cost,
memory use and latency are unbounded until a representative PostgreSQL query
plan and workload envelope are measured. Static SQL inspection cannot close
that STOP.

## Residual STOP conditions

- The SQL has never been executed by this evidence package, so database
  compatibility and runtime behavior remain unproved.
- Neither colliding baseline is certified as the schema adopted by any local,
  staging or production database.
- Generated Diesel schema and table-filter gaps remain; no schema regeneration
  or migration is part of A2.8.
- No `LOWER(wallet_address)` functional index is proven, and no migration adds
  one.
- No representative `EXPLAIN`, cardinality bound, latency budget or load test
  proves the intentionally unbounded raw join.
- Transaction isolation, MVCC behavior and consistency under concurrent writes
  have not been exercised.
- Populated upgrade, fixture-to-source parity and data reconciliation remain
  absent.
- The identity runtime remains byte-identical and always-Free; it does not
  construct or call the adapter.
- Identity RPC workload identity, exact caller authorization, TLS and owner
  binding are absent.
- Ranking revisions, transactional outbox, durable cursor, replay and gap repair
  are absent.
- No typed BFF or Dioxus UI consumes this path, and no browser state is proven.
- Route ownership, configuration, images, gateway, Kubernetes, canary and
  rollback remain unchanged.
- Passing integrity never authorizes database, network, service, deployment or
  production activity.

## Gate usage

```bash
./scripts/migration/verify-a2-8-core-ranking-snapshot-adapter.sh --mode integrity
./scripts/migration/verify-a2-8-core-ranking-snapshot-adapter.sh --mode report
./scripts/migration/verify-a2-8-core-ranking-snapshot-adapter.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-8-core-ranking-snapshot-adapter.sh
```

The verifier refuses database, network, live and production-looking
environments, uses Cargo offline, and runs only the frozen exact test inventory.
Readiness intentionally exits `3` while the residual STOPs remain.
