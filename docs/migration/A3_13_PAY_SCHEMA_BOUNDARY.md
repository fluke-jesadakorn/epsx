# A3.13 Pay Candidate Schema Boundary

Status: **partial remediation with disposable PostgreSQL 18 proof; database authority and production readiness STOP**.

The pay service no longer creates four tables and six indexes during startup. Startup connects to
its configured database, executes one exact read-only `pg_catalog` compatibility query, and fails
before provider construction, router construction, or listener binding when the required public
schema is absent or incompatible. All 54 runtime SQL relation references are explicitly qualified
with `public`, so a hostile or accidental `search_path` cannot redirect handler reads or writes.

This package is only a candidate schema boundary. It does not decide whether the canonical backend
payment schema, `epsx_payment`, `epsx_pay`, or an `epsx_payments_*` environment database is the
payment write authority. It does not adopt or upgrade a populated database, and it does not enable
payment, escrow, link-redemption, webhook, deposit-confirmation, or admin mutations. Those routes
retain the existing uniform fail-closed `404` authorization boundary required by A6.

## Exact runtime finding delta

The canonical A3.3 inventory assigns the ten pay findings `finding.019` through `finding.028`:

| Finding | Removed startup statement |
|---|---|
| `finding.019` | `CREATE TABLE pay_intents` |
| `finding.020` | `CREATE TABLE escrows` |
| `finding.021` | `CREATE TABLE pay_links` |
| `finding.022` | `CREATE TABLE pay_webhook_events` |
| `finding.023` | payer/status intent index |
| `finding.024` | payee/status intent index |
| `finding.025` | escrow status index |
| `finding.026` | pay-link slug index |
| `finding.027` | pay-link intent index |
| `finding.028` | webhook intent index |

The isolated pay delta is `10 → 0`. A3.13 does not edit or silently rebaseline central A3.3,
migration-safety, or readiness contracts; their owners must recompute combined worktree state.

## Immutable provenance and unresolved authority

The development label is `origin/development`, but A3.13 integrity never resolves that moving ref.
It reads immutable commit `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` directly with
`git cat-file` and `git show <commit>:<path>`. Seven exact Git blobs pin the development payment
baseline, replay index, route, owner-scoped submit/status behavior, finality configuration, and
browser submit contract.

The removed candidate runtime DDL is independently pinned at commit
`526c3850fd4b1af336cb29a1a86f86b68be6c59f`, blob
`df3979ce0d92a0b8bbb7374d873b0cd75df71d26`. The six handler blobs from the same commit are also
pinned. After removing only `public.` before the four owned relation names and ignoring rustfmt
whitespace/trailing commas, each handler must remain token-equivalent to that snapshot. This proves
the handler packet changed relation qualification, not financial logic or state transitions.

These pins are evidence of competing durable shapes, not an authority decision. Cutover, dual
write, backfill, or retirement remains prohibited.

## Transaction-body-only migration root

The new root contains one ordered file:

`services/pay/migrations/20260722060000_create_pay_store.sql`

It is exactly 2,150 bytes with SHA-256
`b048fdefebb1c091a0d86ddcf9876a9531519f8ec4d959e863b51067826be83b`. It contains exactly four
guarded `CREATE TABLE IF NOT EXISTS public.*` statements followed by six guarded indexes on public
relations. It contains no `BEGIN`, `COMMIT`, rollback, savepoint, `ALTER`, `DROP`, `TRUNCATE`, row
mutation, schema/extension/type/view creation, backfill, rewrite, or version-ledger operation. A
future reviewed runner owns the transaction and migration ledger.

The exact fresh-schema inventory is 39 columns:

| Table | Columns | Primary/unique keys | Standalone indexes |
|---|---:|---|---:|
| `public.pay_intents` | 13 | `id` PK | 2 |
| `public.escrows` | 13 | `id` PK | 1 |
| `public.pay_links` | 7 | `id` PK; `slug` UQ | 2 |
| `public.pay_webhook_events` | 6 | `event_id` PK | 1 |

The complete structural inventory is five constraints and eleven indexes: four primary-key indexes,
one unique-key index, and six exact standalone non-unique indexes. PostgreSQL 18 additionally
materializes every `NOT NULL` declaration in `pg_constraint` as `contype = 'n'`; the fresh schema
therefore has 29 exact NOT NULL constraints and 34 total catalog constraint rows. Earlier supported
PostgreSQL versions must have zero `contype = 'n'` rows while retaining the same 29
`pg_attribute.attnotnull` values. The candidate intentionally has zero foreign keys. That absence is
checked exactly; it is not represented as production financial durability. A6 continues to require
reviewed ownership, idempotency, state-machine, chain-event, audit, inbox/outbox, and finality
constraints before any write route can become reachable.

## Intentional fresh-schema drift

The migration aligns non-optional Rust fields with `NOT NULL` definitions and lets every decimal
`u64` chain ID fit in its string column. Relative to the removed runtime DDL, a fresh schema adds:

- eleven `NOT NULL` requirements: intent status/timestamps; escrow fee/status/timestamps; link use
  counters/timestamp; and webhook receipt timestamp;
- two `VARCHAR(10) → VARCHAR(20)` chain-ID widenings, one each on intents and escrows.

These thirteen items are fresh-schema definitions, not an upgrade. Guarded table creation is a
complete no-op for an existing table with the same name, so startup deliberately rejects the old
nullable/narrow shape. Designing and proving an additive populated upgrade remains a STOP.

No other money or lifecycle semantics were added. Amount and fee remain strings, the legacy fee
formula and state transitions are unchanged, and no FK/idempotency/outbox/finality schema is
claimed.

## Fail-closed catalog compatibility contract

The 19,212-byte compatibility query has SHA-256
`a4ee6c4ad87e81e1a272d22ed22d3cd7d771f958e899d2b06e040a096f0abca7`. It reads only
`pg_catalog` and verifies:

1. exactly four ordinary permanent public tables, with no partitioning, inheritance, RLS, or
   `pg_policy` rows—even when RLS itself is disabled;
2. exactly 39 columns in ordinal order with exact formatted types, lengths, nullability, and
   default expressions;
3. exactly four single-column primary keys and one immediate single-column unique constraint;
4. the complete inbound/outbound structural boundary contains exactly those five PK/UQ constraints,
   zero FKs/checks/exclusions, and no other non-NOT-NULL constraint;
5. PostgreSQL 18 has exactly 29 one-column, validated, immediate `contype = 'n'` rows matching the
   29 `attnotnull` columns; earlier versions have zero such catalog rows;
6. exactly eleven indexes with the required key signatures and uniqueness/constraint ownership;
7. every index is valid, ready, immediate, non-clustered, non-replica-identity, uses ordinary null
   distinctness, has no INCLUDE column, predicate, or expression, and uses `btree`;
8. all 28 varchar/text columns use their PostgreSQL type's default collation; every index key also
   matches its column collation and uses default index options;
9. every key opclass is exactly `pg_catalog.text_ops`, rejecting a same-named opclass in another
   namespace;
10. all four relations resolve explicitly through `to_regclass('public.*')`.

Missing and extra columns, constraints, and indexes fail closed. An incoming FK from another schema
also fails. A same-named shadow table cannot satisfy the explicit-public probe, and all handler SQL
continues to address `public.*` after the probe.

## Rust model and bind boundary

The migration makes every non-optional `PayIntent`, `EscrowRecord`, and `PayLink` field non-null in
SQL. The eight nullable response fields remain Rust `Option`. `chain_id` remains a decimal `String`
bound from `u64`, now backed by `VARCHAR(20)`; amounts, IDs, addresses, statuses, timestamps, optional
text, integer counters, and webhook JSON retain their existing SQLx bind types. Eleven exact bind
anchors are frozen by the contract.

This is type compatibility only. It does not validate raw amount syntax, positivity, token decimals,
address ownership, status transitions, fee precision, receipt identity, or webhook replay safety.

## Authorization invariant

The schema check executes before provider and router construction. After router construction,
`protect_router` remains installed before listener binding. `UnsafePaymentsManage` validates an
admin credential and still returns `404`; `UnsafeFinancialMutation`,
`InternalIdentityUnavailable`, and blocked paths still return `404` without entering handlers.
Existing service authorization tests remain the executable route-level proof.

## Hermetic evidence

```sh
rustfmt --edition 2021 --check \
  services/pay/src/db.rs \
  services/pay/src/main.rs \
  services/pay/src/handlers/intents.rs \
  services/pay/src/handlers/escrows.rs \
  services/pay/src/handlers/pay_admin.rs \
  services/pay/src/handlers/pay_history.rs \
  services/pay/src/handlers/pay_links.rs \
  services/pay/src/handlers/pay_webhooks.rs
cargo test --locked --offline -p epsx-pay-svc --lib
cargo test --locked --offline -p epsx-pay-svc --bin pay-service
cargo check --locked --offline -p epsx-pay-svc --bin pay-service
./scripts/migration/verify-a3-13-pay-schema-boundary.sh --mode integrity
./scripts/migration/verify-a3-13-pay-schema-boundary.sh --mode readiness  # expected exit 3
./scripts/migration/test-a3-13-pay-schema-boundary.sh
./scripts/migration/verify-payment-execution.sh --mode integrity
./scripts/migration/verify-payment-execution.sh --mode readiness          # expected exit 3
./scripts/migration/test-payment-execution.sh
```

The A3.13 self-test covers deterministic reporting and readiness, full top-level/safety shape,
authority, source commit/blob, removed-source, handler-source, migration/query digest, mutation
policy, hostile `search_path`, unqualified runtime relation, bind/model arrays, fresh-schema drift,
type/nullability/default, PK/UQ/FK/index/PG18 NOT NULL inventories, policy rows,
partial/expression/INCLUDE, inheritance, RLS, default type collation, opclass name/namespace, unsafe
route, exact blocker text, isolated-evidence claims, production-environment, database-environment, and
chain-environment tampering.

The hermetic verifier and self-test contact no database, provider, chain, network, container,
migration runner, or deployment.

## Disposable PostgreSQL 18 evidence

An isolated PostgreSQL 18.4 Homebrew cluster was created with a fresh `initdb` under a uniquely
generated `/tmp/epsx-pay-a3-13.*` directory. It contained no repository, candidate, deployed, or
production data. The exact migration and extracted compatibility query produced:

- clean fresh schema: `true`;
- catalog constraints: `34 total | 29 contype=n | 5 structural`;
- policy present while RLS remained disabled: `false`, then `true` after policy removal;
- non-indexed `TEXT COLLATE "C"`: `false`, then `true` after restoring `COLLATE "default"`;
- an alternate-schema operator class also named `text_ops`: `false`, then `true` after restoring
  the `pg_catalog.text_ops` index;
- cluster stopped and its unique temporary directory removed, with absence confirmed.

This proves fresh-schema PostgreSQL 18 catalog behavior only. It is not baseline adoption,
populated upgrade, authority, deployed-database, concurrency, or readiness evidence.

## Residual A3.13 STOP blockers

- No migration runner or version ledger is wired.
- Payment database/write authority remains unresolved.
- Safe baseline adoption of an already matching schema is unproven.
- No populated upgrade has preserved rows through the thirteen fresh-schema drift items.
- No cross-authority row, constraint, or index reconciliation has run.
- Concurrent migration/pay-service startup ordering is untested.
- The migration/probe have run only in an empty disposable PostgreSQL 18 cluster; no candidate,
  deployed, populated, adoption, or upgrade database proof exists.
- A6 financial durability and execution requirements remain blocked.

Until those eight blockers have reviewed executable evidence, `productionReady` remains `false` and
readiness intentionally exits `3`.
