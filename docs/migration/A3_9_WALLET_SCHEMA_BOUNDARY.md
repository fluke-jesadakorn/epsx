# A3.9 Wallet Schema Boundary

Status: **partial static remediation; production readiness STOP**.

The wallet service no longer creates tables during service startup. Startup now connects to the
wallet database, runs one read-only compatibility query, and fails before listener bind when the
accepted legacy schema is absent or incompatible. All five runtime relation references are
schema-qualified (`public.accounts` three times, `public.nonces` once, and
`public.signed_transactions` once), so a caller-controlled or deployment-specific `search_path`
cannot redirect runtime wallet reads or writes after the probe checks the public relations.

## Development mapping and finding identity

The immutable locally available development commit
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` contains no `services/wallet` candidate. The service
and its three-table store are migration-branch additions; there is no development service schema
to import or reconcile statically.

The assigned package referenced findings 035–037, but the canonical A3.3 fixture had already been
refreshed before implementation. At inspection time the three wallet `CREATE TABLE` findings were
`finding.033`, `finding.034`, and `finding.035`, all in `services/wallet/src/main.rs`. The exact
wallet-local scanner now observes zero runtime Rust DDL findings.

| Inventory | Before | Isolated projection after | Delta |
|---|---:|---:|---:|
| Runtime Rust DDL findings | 35 | 32 | -3 |
| Actionable runtime findings | 29 | 26 | -3 |
| Wallet findings | 3 | 0 | -3 |
| Reviewed exceptions | 6 | 6 | 0 |

The after values are an isolated projection from the inspected A3.3 fixture. This package does
not edit or rebaseline the canonical migration-safety or A3.3 contracts, which are owned by the
global reconciliation package.

## Immutable additive migration root

The new ordered root contains one transaction-body-only migration:

`services/wallet/migrations/20260722020000_create_wallet_store.sql`

Its 775 bytes are pinned by SHA-256
`cf79bdb4e999d4cfb54648ba8d82e845af7c5feaccd20d5ca2143ff673ca1731`. It contains exactly three
guarded `CREATE TABLE IF NOT EXISTS public.*` statements and preserves the existing 17-column
schema. It contains no `DROP`, `TRUNCATE`, `DELETE`, `INSERT`, `UPDATE`, `ALTER`, backfill,
rewrite, extension/schema creation, or transaction-control statement. Transaction ownership is
left to a future reviewed runner.

The guard is additive for an empty database and a SQL no-op when each named table exists. It is
not a baseline-adoption protocol: an existing but incompatible table is not altered or recreated;
the service instead stops at the compatibility boundary.

## Exact accepted schema

The 22,561-byte compatibility query is pinned by SHA-256
`a46ba81e71d77d13f35c40437e79ff0f45e3365efe6898908e5b18177082c71d`. It checks:

- `public.accounts`: six ordered columns; composite `(address, chain_id)` primary key; one valid,
  ready, live, unique primary btree index with the expected key order and `text_ops` classes.
- `public.nonces`: four ordered columns; composite `(address, chain_id)` primary key; the same
  exact one-index boundary.
- `public.signed_transactions`: seven ordered columns; integer `SERIAL` `id`; single-column
  primary key and `int4_ops` btree index; the owned public sequence with integer range, start,
  increment, cache, and non-cycling properties.
- Exact data types, varchar lengths, nullability, defaults, column order, non-identity and
  non-generated status across all 17 columns. Timestamps require default microsecond precision;
  text/varchar columns require database-default collation, and each index collation OID must match
  its key column.
- Ordinary persistent, non-partitioned public tables with default replica identity, no row
  security, and no `pg_inherits` row naming a target as either inheritance child or parent.
- Exactly one structural relation constraint and exactly one relation index per table. PostgreSQL
  versions that expose `NOT NULL` as `pg_constraint.contype = 'n'` are normalized through the
  already-exact column-nullability check. Each primary key must be non-deferrable, initially
  immediate, validated, and bound by `conindid` to the exact immediate index being checked. The
  constraint inventory uses a left join to that index, so extra CHECK/FK rows with `conindid = 0`
  remain visible and make the exact-one check fail. Index operator classes must come from
  `pg_catalog`. Other extra constraints, indexes, or columns fail closed, as do missing catalog
  rows, expression/partial indexes, included columns, wrong key order, invalid/not-ready indexes,
  and sequence ownership drift.

The relation compatibility CTE contains exactly one `NOT EXISTS` inheritance guard and exactly one
`pg_inherits` scan. Both the Rust unit suite and the hermetic verifier reject a duplicated opener,
so malformed nested `AND NOT EXISTS (` syntax cannot be hidden by updating only the query digest.

The `signed_transactions.id` default must deparse to exactly one `nextval` call, allowing only the
justified qualified or unqualified rendering of `signed_transactions_id_seq`, and must also have
an exact `pg_attrdef`/`pg_depend` reference to the owned public sequence OID. Wildcard matching,
double-`nextval` expressions, and defaults targeting another sequence are rejected.

PostgreSQL 18 may expose column NOT NULL state as `pg_constraint.contype = 'n'`. When that catalog
shape is exposed, the probe requires exactly eight such constraints—one for every expected
non-null column—and rejects constraints on any of the nine nullable columns. Every exposed NOT
NULL constraint must be single-column, validated, enforced, non-deferrable, and initially
immediate. Enforcement is read through `to_jsonb(pg_constraint)` so the query remains parseable on
pre-PG18 catalogs where `conenforced` is not a physical column. A pre-PG18 server with no `n` rows
takes the explicit version-tolerant no-row path; an unexpected backported `n` row activates the
strict inventory instead of being ignored.

Every aggregate and default comparison explicitly coalesces SQL `NULL` to `false`, so a missing
catalog row or indeterminate default cannot be ignored by `bool_and`. The query is a CTE/catalog
read only; it contains no mutation or command token. This is static inspection evidence only—the
query has not been parsed or executed by PostgreSQL in this package.

## Rust model and bind audit

The accepted legacy schema contains no UUID fields. The runtime model/bind audit keeps database
and Rust types aligned:

- `AccountResponse.address` and `chain_id` remain `String`; nullable `label` and nullable `role`
  are `Option<String>`. Correcting `role` from `String` prevents legacy `NULL` rows from failing
  `sqlx::FromRow` decoding.
- `public.nonces.nonce BIGINT` is decoded as `i64`. Conversion to the unsigned response field uses
  `u64::try_from` so a corrupt negative legacy value cannot wrap.
- Numeric request chain IDs are converted to decimal strings and rejected above the accepted
  `VARCHAR(10)` boundary. Role input is bounded to 50 characters.
- Every database-bound address is Alloy-parsed and normalized to an exact lowercase,
  `0x`-prefixed 42-character string; merely length-shaped non-hex account input is rejected.
- Transaction values are parsed as decimal or explicitly prefixed hexadecimal `U256`. Empty,
  signed, fractional, non-numeric, and overflowing inputs are rejected; stored values are canonical
  decimal and fit `VARCHAR(78)`.
- Transaction data must be valid hex and at most 32 bytes before its normalized `0x` form is bound
  to `VARCHAR(66)`. Invalid hex is rejected instead of silently becoming empty data.
- EVM address parsing continues to bound stored sender/recipient/address values to their accepted
  42-character representation. No request or path UUID conversion exists in this schema.
- Nonce allocation, checked conversion, and signed-transaction insertion share one SQLx
  transaction, so a conversion or insert error returns before commit and rolls back the nonce.

These checks do not enable the wallet routes already disabled by the fail-closed authorization
boundary; they prevent a future route-enablement slice from reintroducing model/schema mismatch.

## Offline evidence

```bash
cargo test --locked --offline -p epsx-wallet --lib
cargo test --locked --offline -p epsx-wallet --bin wallet
cargo check --locked --offline -p epsx-wallet --bin wallet
scripts/migration/verify-a3-9-wallet-schema-boundary.sh --mode integrity
scripts/migration/verify-a3-9-wallet-schema-boundary.sh --mode report
scripts/migration/test-a3-9-wallet-schema-boundary.sh
```

The Rust library suite passes 11/11, including read-only/NULL-safe query guards and exact public
constraint/index/sequence anchors. The binary suite passes 4/4 for chain/data/address/value bind boundaries, and
the locked offline binary check passes. The verifier pins migration/query/model bytes, reproduces
the wallet runtime DDL scanner, proves runtime relation qualification and startup ordering, and
self-tests readiness, migration/query/catalog/model/development/blocker tampering.

`--mode readiness` intentionally exits `3`. No database, migration, network, listener, container,
or production action ran.

## Residual A3.9 STOP blockers

- No reviewed migration runner or durable version ledger is wired.
- Safe baseline adoption for already matching deployed wallet tables is unspecified and untested.
- No populated source-version wallet database has preserved real rows through an upgrade.
- No pre/post row, key, default, constraint, index, or sequence reconciliation has run.
- Concurrent migration/startup ordering across wallet replicas is untested.
- Neither the migration nor the compatibility query has run against live PostgreSQL.

Until all six blockers have executable evidence, this package is not production-ready and must
not be used as deployment approval.
