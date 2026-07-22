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

The manual root contains two ordered transaction-body-only files. The original projection migration
remains byte-for-byte unchanged:

`services/indexer/migrations/20260722050000_create_indexer_projection_tables.sql`

It is exactly 4,822 bytes with SHA-256
`5d0ec77a11d2abe1303c5f9b87e7da18eadee9d2e7fa4aeda1aeaf3d76549ff8`. It contains three guarded
`CREATE TABLE IF NOT EXISTS public.*` statements and five guarded indexes. It contains no
`ALTER`, `DROP`, `TRUNCATE`, data mutation, `CASCADE`, transaction control, schema/extension
creation, or backfill. A future reviewed runner owns transaction and version-ledger semantics.

The additive dormant fork-store migration is:

`services/indexer/migrations/20260722070000_create_indexer_fork_store.sql`

It is exactly 23,326 bytes with SHA-256
`60b82188c74c5de7463610ce4c5795150970a4b760d5a81c66981cd25d9e5f00`. It contains exactly eight
guarded `CREATE TABLE IF NOT EXISTS public.*` statements and two necessary guarded lookup indexes.
It contains no destructive statement, DML, transaction control, schema/extension creation,
function, trigger, payload cap, or reference to the three projection tables.

The original projection guards are additive for an empty database. They are deliberately not
baseline adoption: when an old runtime-created projection table already exists, its table guard is
a SQL no-op and startup rejects incompatible shape. This package never attempts an in-place
primary-key replacement or data rewrite.

The fork-store `IF NOT EXISTS` guards are repository-required but are not treated as collision
safety by themselves. Before the first `CREATE`, an exact PostgreSQL preflight scans `pg_class` in
`public` for all eight table names and both explicit index names, without filtering relation kind,
and raises with the complete collision list. Any collision aborts this fresh-create-only migration;
it never adopts or repairs an existing relation. A future reviewed transactional runner must
execute this preflight and the guarded creates atomically, then record the migration version only
after they succeed. The anonymous preflight uses literal SQLSTATE `42P07`; its sole procedural
`BEGIN` is confined before the first `CREATE` and is not migration transaction control. Top-level
`BEGIN TRANSACTION`, `BEGIN WORK`, `START TRANSACTION`, `COMMIT`, and `ROLLBACK` remain forbidden.

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
booleans to a one-row-per-`(chain,height)` table would not preserve competing ancestry.

## Dormant fork-store substrate

The separate additive substrate contains 74 exact columns across eight tables:

| Table | Purpose |
|---|---|
| `indexer_block_candidates` | Immutable candidate facts keyed by `(chain_id, block_hash)` |
| `indexer_transaction_inclusions` | Block-scoped inclusions; the same transaction hash may occur in competing candidates |
| `indexer_receipts` | Exact reverted/succeeded/post-state receipt outcomes and gas facts |
| `indexer_raw_logs` | Receipt-scoped raw logs with a strict contiguous four-topic shape |
| `indexer_selected_blocks` | Current internal selected block per chain and height |
| `indexer_chain_state` | Revision, selection references, and persisted fenced lease state |
| `indexer_mutation_journal` | Exact transition header and outcome facts without a serialization fingerprint |
| `indexer_mutation_blocks` | Ordered detach/attach block identities for each mutation |

The 28 structural constraints and 73 checks preserve block-scoped fork identities, exact candidate
references, signed-storage revision/fence bounds, nullable reference pairs, deferrable selected-head
and finalized-selection references, live-lease fencing, NULL-safe mutation-kind shapes, and exact
U256 value bounds. Two explicit indexes support parent and transaction-hash lookup. Candidate,
transaction, receipt, and raw-log facts contain no canonical/finalized flags. `input_data` and raw
log `data` have no fixed SQL cap because the public validation limits remain caller-selectable.

This remains a dormant boundary. No runner has parsed or executed the migration. A private,
default-off PostgreSQL substrate now statically targets the fork-store tables, but it is not
exported, implemented as the repository port, called from `main`, or activated by a provider,
worker, route, or startup hook. The runtime compatibility query deliberately remains pinned to the
original three projection tables. This compiled static substrate does not establish external
canonicality, consensus finality, durable replay, checkpointing, or production readiness. A12
still owns executable ingestion, repair, replay, and rollout proof.

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

The recursively pinned Rust inventory now contains twelve files: `lib.rs`, `main.rs`, the dormant
`ingestion/{domain,memory,mod,ports,selection}.rs` module, and the private
`ingestion/postgres/{candidates,codec,leases,mod,reads}.rs` substrate. The public ingestion module
defines offline checked block-batch and fork-preserving selected-chain transition/port contracts.
`memory.rs` is a `cfg(test)`-only in-memory conformance implementation; it is dormant and makes no
external canonicality, consensus-finality, or durability claim.

The PostgreSQL substrate is behind the default-disabled `dormant-postgres-adapter` feature. Its
module is private, has no public re-export or `main` callsite, and its repository type remains only
a `pub(super)` `PgPool` holder without a `SelectedChainRepository` implementation. Its reviewed
static behavior targets parent candidate conflict only, keeps child inserts strict, reloads all
candidate children, and revalidates the reconstructed batch. The codecs fail closed across exact
hash/address widths, unsigned decimal U256, whole-second nonnegative timestamps, and exact receipt
outcomes. Lease acquisition, renewal, and release use row locks, persisted monotonically advancing
fences, and PostgreSQL `clock_timestamp()` predicates; they use neither process `Utc::now()` nor
advisory locks.

The private read-side helpers statically define repeatable-read, read-only transactions around
candidate reconstruction, candidate-at-height lookup, selected-hash lookup, and full chain
snapshots. Candidate reconstruction and snapshot validation keep their multi-table reads inside one
PostgreSQL snapshot. A missing chain-state row is accepted only when both selected-block and
mutation-journal rows are absent. Present snapshots verify the highest selected-head mapping,
finalized mapping, and every selected revision against the chain revision. Selected-hash lookup
uses a `LEFT JOIN` to chain state and rejects missing, zero/stale, or future revision state. The
candidate readers never query the legacy `public.blocks`, `public.transactions`, or
`public.token_transfers` projections. These are compiled static helpers only: no provider, worker,
checkpoint, startup activation, route activation, trait implementation, migration execution,
database read, or database write occurred, so all ten STOP blockers remain unchanged.

Seven ordered boundary sources are cryptographically pinned before their semantic anchors are
checked. Exact byte counts and SHA-256 digests prevent comment-only, case, alias, path, or ordering
drift from passing merely because a few expected substrings remain:

| Source | Bytes | SHA-256 |
|---|---:|---|
| `services/indexer/Cargo.toml` | 771 | `9cd598ce3adeac3fde3ec021704ee5213b93622d6d6ff8e836e0b0c2b165a135` |
| `services/indexer/src/ingestion/mod.rs` | 1,170 | `395e589d5eb05c5d8577d9a15bf1c131f3d1c114ff3eb3289985b97424d6d547` |
| `services/indexer/src/ingestion/postgres/candidates.rs` | 19,643 | `9bccc08effb68e06593469f93d779cc2a12bad088b6698b3eded8d2de4128180` |
| `services/indexer/src/ingestion/postgres/codec.rs` | 5,891 | `693e1ddba5a8f8808251ed8be68f547b5a8da1122eec954a741ca8c0c95f9915` |
| `services/indexer/src/ingestion/postgres/leases.rs` | 11,531 | `20adcdc84b1fd970ed404d2ac9219b3de827ca01a84ece33813cfaf6ba690910` |
| `services/indexer/src/ingestion/postgres/mod.rs` | 568 | `521534817aafdb618ebe3528cebceb3206be4e5b145c0ef2ae933794ee026d10` |
| `services/indexer/src/ingestion/postgres/reads.rs` | 20,505 | `b87bf78f4773b8f63d619a058d262462200bcb606c2c2c909337fe1a52809cce` |

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
  services/indexer/src/ingestion/postgres/candidates.rs \
  services/indexer/src/ingestion/postgres/codec.rs \
  services/indexer/src/ingestion/postgres/leases.rs \
  services/indexer/src/ingestion/postgres/mod.rs \
  services/indexer/src/ingestion/postgres/reads.rs \
  services/indexer/src/ingestion/selection.rs \
  services/indexer/src/lib.rs \
  services/indexer/src/main.rs
cargo test --locked --offline -p epsx-indexer --lib
cargo test --locked --offline -p epsx-indexer --lib --features dormant-postgres-adapter
cargo test --locked --offline -p epsx-indexer --bin indexer
cargo check --locked --offline -p epsx-indexer --bin indexer

scripts/migration/verify-a3-12-indexer-schema-boundary.sh --mode integrity
scripts/migration/verify-a3-12-indexer-schema-boundary.sh --mode report
scripts/migration/test-a3-12-indexer-schema-boundary.sh
```

The default library suite passes 33/33, the feature-enabled library suite passes 50/50, and the
binary suite passes 4/4. The locked offline binary check passes. The verifier pins provenance,
removed runtime bytes, both migration digests, the unchanged runtime-query digest, runtime DDL
zero, the twelve-file recursive Rust inventory, public qualification, schema/constraint/index
catalog semantics, model/bind corrections, startup ordering, absent fake sync, the default-off
private module, seven ordered source byte/digest pins, PgPool-only holder, candidate/codec/lease/read
anchors, fail-closed readiness, and ten residual blockers. Its self-test adversarially tampers
readiness, source commit/path/blob, query digest/bytes, relation counts, Rust inventory, adapter
source-pin hash/bytes/path/order, feature/privacy/database-clock/strict-child policy, and read-module/
consistent-transaction/orphan/future/legacy/activation policy, both migration digests/guards,
fork-store collision and key policies, global transaction key, and schema descriptors. Same-length
column, structural-constraint,
weakened-check, and index substitutions are rejected, as are inheritance/RLS/opclass/collation/
partial-index policy and blocker tampering.

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
