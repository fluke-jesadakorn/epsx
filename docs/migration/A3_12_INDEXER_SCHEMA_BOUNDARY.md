# A3.12 Indexer Schema Boundary

Status: **partial static remediation; production readiness STOP**.

The indexer no longer creates `blocks`, `transactions`, or `token_transfers`, or their indexes,
at service startup. Startup connects to `epsx_indexer`, runs one exact read-only PostgreSQL catalog
probe, and fails before listener binding if the reviewed public schema is absent or incompatible.
The package also removes the default-on autonomous provider/sync worker and every fabricated block
write. It does not enable any non-health route.

This is a schema boundary, not the canonical A12 indexer. No database, migration, provider, RPC,
chain, service, container, or deployment operation was performed.

## Immutable provenance

`origin/development` is only a provenance label. The verifier reads immutable commit
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` and proves that neither `services/indexer` nor
`migrated/services/indexer` exists there. Development therefore supplies no standalone indexer
route or schema contract to import.

The removed runtime boundary is pinned independently at commit
`b624f320c2db3dc24944cc0414deae7bc2d42196`, file
`services/indexer/src/main.rs`, Git blob
`3bb4779628eb888be9cc0a832bcf249828b2b345`. Its three table and two index anchors are the exact
five actionable A3.3 findings removed by this package.

## Runtime finding delta

| Inventory | Before | Isolated projection after | Delta |
|---|---:|---:|---:|
| Indexer runtime Rust DDL | 5 | 0 | -5 |
| Global runtime Rust DDL | 28 | 23 | -5 |
| Global actionable findings | 22 | 17 | -5 |
| Service-startup schema mutations | 19 | 14 | -5 |
| Reviewed exceptions | 6 | 6 | 0 |

The global values are an isolated projection from the inspected A3.3 fixture. A3.12 does not edit
or rebaseline migration-safety, A3.3, A12, or production-readiness evidence; their central owner
must recompute the combined worktree after concurrent packages are complete.

## Immutable additive migration

The new manual root contains one ordered transaction-body-only file:

`services/indexer/migrations/20260722050000_create_indexer_projection_tables.sql`

It is exactly 4,822 bytes with SHA-256
`5d0ec77a11d2abe1303c5f9b87e7da18eadee9d2e7fa4aeda1aeaf3d76549ff8`. It contains three guarded
`CREATE TABLE IF NOT EXISTS public.*` statements and five guarded indexes. It contains no
`ALTER`, `DROP`, `TRUNCATE`, data mutation, `CASCADE`, transaction control, schema/extension
creation, or backfill. A future reviewed runner owns transaction and version-ledger semantics.

The guard is additive for an empty database. It is deliberately not baseline adoption: when an old
runtime-created table already exists, the table guard is a SQL no-op and startup rejects its old
global transaction key, nullability, constraints, or index inventory. This package never attempts
an in-place primary-key replacement or data rewrite.

## Exact fresh-schema contract

The accepted schema contains 27 ordered columns:

| Table | Columns | Structural contract |
|---|---:|---|
| `public.blocks` | 9 | PK `(chain_id, number)`; unique `(chain_id, hash)` |
| `public.transactions` | 9 | PK `(chain_id, hash)`; block FK; chain/hash/block unique key |
| `public.token_transfers` | 9 | PK `(chain_id, tx_hash, log_index)`; chain/hash/block transaction FK |

The transaction primary key is intentionally chain-scoped. A global `PRIMARY KEY (hash)` is never
compatible even though EIP-155 commonly changes signed hashes across chains; chain remains part of
the logical identity and every query already supplies it.

All 24 checks are validated and enforced. They bound canonical decimal chain IDs, nonnegative
block/log/gas/count fields, lowercase full-width EVM hashes and addresses, receipt status, and block
gas usage. Transaction and transfer values must be canonical decimal strings and no greater than
`115792089237316195423570985008687907853269984665640564039457584007913129639935`, the exact
U256 maximum. Both foreign keys use `NO ACTION`; no delete cascade is introduced.

Ten exact btree indexes are accepted: five constraint-backed PK/unique indexes and five explicit
history indexes. Address history uses separate `from` and `to` indexes and the stable order
`block_number DESC, tx_hash DESC, log_index DESC`. The current HTTP transfer result remains capped
and blocked; a later route slice still needs a typed cursor contract.

This projection schema does not pretend to be a fork store. Adding `canonical` or `finalized`
booleans to a one-row-per-`(chain,height)` table would not preserve competing ancestry. A12 still
owns the canonical block inclusion, receipt/raw-log, finality, fork, repair-journal, and durable
checkpoint design.

## Exact read-only compatibility boundary

`INDEXER_SCHEMA_COMPATIBILITY_QUERY` is 17,490 bytes with SHA-256
`17238dc074b4975ea8e5af6ce54ac6ccf57683b84b37e1264123d20da6d38b1e`.
It uses `information_schema` and `pg_catalog` only and verifies:

- three exact ordinary, permanent, nonpartitioned `public` tables;
- exactly 27 columns in ordinal order with exact PostgreSQL types, typmods, nullability, timestamp
  precision, defaults, identity/generated state, and type-default collation;
- seven exact PK/unique/FK structures with exact key order and referenced keys; both catalog
  attribute-name arrays are explicitly converted from PostgreSQL `name[]` to contract `text[]`;
- exactly 24 named validated, enforced, immediate, nondeferrable checks whose complete normalized
  catalog definitions are pinned; substring matches and weakened forms such as `(number >= 0) OR
  true` are incompatible;
- the complete inbound/outbound FK boundary, containing only the two reviewed FKs;
- exactly ten live, ready, valid, immediate, expression-free, nonpartial, no-include btree indexes;
- exact index definitions/directions, only `pg_catalog` operator classes, and index/key collation
  equality, including PostgreSQL 18's `"timestamp"` quoting in `pg_get_indexdef`;
- default replica identity, disabled RLS, and no inheritance as either parent or child;
- no extra columns, structural constraints, checks, or indexes.

Every aggregate/default decision is NULL-coalesced to false. The query contains no mutation or
command token, and the compatibility function uses `query_scalar(...).fetch_one`, never
`execute`.

## Rust and startup corrections

- Every `TIMESTAMPTZ` projection now uses `chrono::DateTime<chrono::Utc>`.
- Nullable PostgreSQL values map only to `Option`: block miner, transaction recipient, and receipt
  status. Transaction sender and fresh-schema-required values are nonnullable.
- Database `BIGINT` to response `u64` conversion uses `u64::try_from`; negative values never wrap.
- Chain IDs must be canonical positive decimal strings within `VARCHAR(10)`.
- Transaction hashes and addresses are Alloy-parsed and emitted as canonical lowercase hex;
  lowercase-only arbitrary strings are rejected.
- All four surviving runtime relation occurrences are explicitly public-qualified: two blocks,
  one transactions, and one token-transfers reference.
- Transfer ordering is deterministic across equal block numbers.

The startup order is auth configuration, database connection, exact compatibility probe, inert
database-only state construction, router construction, then listener binding. There is no chain
provider, block-number fetch, in-memory cursor, polling option, autonomous task, placeholder hash,
or conflict-skipping insert.

The recursively pinned Rust inventory now contains six files: `lib.rs`, `main.rs`, and the dormant
`ingestion/{domain,memory,mod,ports}.rs` module. The module defines offline checked block-batch and
port contracts; `memory.rs` is test-only. It adds no provider adapter, repository adapter, worker,
checkpoint, startup hook, route, SQL, RPC, canonicality, or finality claim, so the ingestion,
checkpoint, and fork/reorg STOP blockers remain unchanged.

The existing direct boundary remains authoritative: only GET/HEAD `/health` reaches a handler.
All reads return `404` before SQL, while POST `/sync` still requires exact admin audience plus
`admin:indexer:manage` and then returns `404`. The explicit sync handler also returns `404` as a
second fail-closed layer.

## Offline evidence

```sh
rustfmt --edition 2021 --check \
  services/indexer/src/ingestion/domain.rs \
  services/indexer/src/ingestion/memory.rs \
  services/indexer/src/ingestion/mod.rs \
  services/indexer/src/ingestion/ports.rs \
  services/indexer/src/lib.rs \
  services/indexer/src/main.rs
cargo test --locked --offline -p epsx-indexer --lib
cargo test --locked --offline -p epsx-indexer --bin indexer
cargo check --locked --offline -p epsx-indexer --bin indexer

scripts/migration/verify-a3-12-indexer-schema-boundary.sh --mode integrity
scripts/migration/verify-a3-12-indexer-schema-boundary.sh --mode report
scripts/migration/test-a3-12-indexer-schema-boundary.sh
```

The library suite passes 29/29 and the binary suite passes 4/4. The locked offline binary check
passes. The verifier pins provenance, removed runtime bytes, migration/query digests, runtime DDL
zero, the six-file recursive Rust inventory, public qualification, schema/constraint/index catalog semantics, model/bind corrections,
startup ordering, absent fake sync, fail-closed readiness, and ten residual blockers. Its self-test
adversarially tampers readiness, source commit/path/blob, query digest/bytes, relation counts, fake
sync policy, migration digest/guards, global transaction key, and schema descriptors. Same-length
column, structural-constraint, weakened-check, and index substitutions are rejected, as are
inventory, inheritance/RLS/opclass/collation/partial-index policy, and blocker tampering.

`--mode readiness` intentionally exits `3`.

## Residual A3.12 STOP blockers

1. No reviewed migration runner or durable version ledger exists.
2. No populated legacy-table baseline adoption or upgrade has preserved data.
3. PostgreSQL has not parsed or executed the migration or compatibility query.
4. Concurrent migration and replica startup ordering is untested.
5. Canonical blocks, transactions, receipts, and raw/decoded logs are not ingested.
6. No durable checkpoint lease or atomic gap-free advancement exists.
7. Fork, reorg, finality, orphan repair, and replay semantics remain absent.
8. Bounded backfill, reconciliation, pause/resume, and poison-record recovery remain absent.
9. Address-history privacy and typed route/error/cursor contracts remain undecided.
10. No indexer workload, secret/provider wiring, truthful browser UX, rollout evidence, or explicit
    production authorization exists.

Until every blocker has executable evidence, `productionReady` remains `false`.
