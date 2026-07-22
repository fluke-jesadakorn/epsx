# A3.11 Notification Schema Boundary

Status: **partial static remediation; production readiness STOP**.

The notification service no longer creates tables or indexes during startup and no longer writes
default templates or demo notifications before serving. Startup connects to `epsx_notification`,
runs one read-only exact-schema compatibility query, loads every active template without swallowing
query or registration failures, and only then initializes provider state and binds the listener.
All 19 application SQL relation references name `public.templates` or `public.notifications`
explicitly.

This package changes no route, authorization policy, delivery truth, provider/job behavior,
template strictness, BFF, user/admin UI, publisher adapter, Kubernetes resource, or production
state. It does not authorize database access, migration execution, Redis, SMTP, browser/network
access, deployment, cutover, or production use.

## Exact runtime delta

The immutable pre-change source is commit
`b624f320c2db3dc24944cc0414deae7bc2d42196`, blob
`64633151dae98bd7e5368d225f869936d3237a41`. It contains exactly four runtime DDL statements:
two tables and two indexes. It also calls both startup seed functions and binds a naive UTC value to
`sent_at`.

| Runtime boundary | Before | After | Delta |
|---|---:|---:|---:|
| Rust DDL findings | 4 | 0 | -4 |
| Startup seed calls | 2 | 0 | -2 |
| Startup seed write sites | 2 functions | 0 | removed |
| Swallowed startup template-load errors | query + registrations | 0 | fail closed |
| Public-qualified application relations | 0 | 19 | +19 |

The `sent_at` bind now uses `Some(chrono::Utc::now())`, matching the runtime
`Option<DateTime<Utc>>` model and PostgreSQL `TIMESTAMPTZ(6)`. This is a type-boundary correction,
not delivery-state remediation.

## Immutable source and migration provenance

`origin/development` is a provenance label only. The gate reads immutable commit
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`; it does not resolve that moving ref. The commit has no
`services/notification` crate. Its legacy notification authority is pinned by the exact Git blob
`db5ed1153a360559b07852842a132427535aa34c` for the consolidated
`wallet_notifications` baseline. A3.11 deliberately does not claim compatibility with, import,
backfill, or replace that model.

The existing notification migration root and runner remain:

- root: `apps/backend/migrations/notifications`;
- runner config: `apps/backend/diesel_notifications.toml`;
- configured directory: `migrations/notifications/`.

The same config's `print_schema` filter currently lists `notifications` and
`wallet_notifications` but omits `templates`. That does not redirect the migration runner and the
SQLx service does not consume Diesel's generated schema, so A3.11 does not change the config.
Nevertheless, it is recorded as a tooling-drift STOP until the owning schema-generation workflow
reviews the filter.

A3.11 adds one ordered migration directory,
`20260722040000_create_notification_service_tables`, without creating a second root or changing the
runner:

| File | Bytes | SHA-256 |
|---|---:|---|
| `up.sql` | 1,128 | `788fa9500df1759d7b224c739f90f4756c2397f28a42aca1ec9af197f27290f7` |
| `down.sql` | 191 | `5f47cf6f1c82416ac8c60bd3e691c78b4d58f4ee78bae3f778869206350b76cc` |

`up.sql` contains exactly two guarded `CREATE TABLE IF NOT EXISTS public.*` statements and two
guarded indexes. It contains no DML, `ALTER`, `DROP`, `TRUNCATE`, cascade, extension/schema creation,
transaction control, foreign key, or check constraint. The `down.sql` body raises a forward-only
exception instead of pretending a destructive table drop is safe. That refusal prevents accidental
data loss; it is not a recovery or rollback plan.

The pre-existing root remains a STOP. The renamed/consolidated baseline requires ledger-adoption
review, and `20260613000000_drop_notification_subscriptions/up.sql` still contains a destructive
`DROP TABLE ... CASCADE`. A3.11 neither edits nor excuses that history.

## Exact fresh-schema contract

The additive migration defines 26 columns. Fourteen are effectively non-null: twelve explicit
`NOT NULL` declarations plus the two primary-key columns.

| Table | Columns | Exact keys | Application indexes |
|---|---:|---|---|
| `public.templates` | 9 | `id` PK; `name` unique | constraint-backed PK and unique indexes |
| `public.notifications` | 17 | `id` PK | `idx_notif_user(user_id ASC, created_at DESC)`; `idx_notif_status(status ASC)` |

There are exactly three relation key constraints and five total indexes. There are no foreign keys
or check constraints. Inbound foreign keys from any schema are rejected too; absence is a complete
inbound-and-outbound inventory, not an outbound-only observation.

The compatibility query is exactly 20,887 bytes with SHA-256
`8733c2fd595ad6ea319dc83a5d9ece2adad0e78008a134b129faae6fcdea190e`. It reads only
`information_schema` and `pg_catalog`, returns one boolean through `query_scalar`, and verifies:

1. both names resolve in `public` to ordinary permanent, non-partitioned tables using default
   replica identity, with no inheritance, RLS, forced RLS, or policies;
2. the exact 9+17 ordered column inventory, PostgreSQL data type/UDT, varchar length,
   `TIMESTAMPTZ(6)` precision, nullability, semantic default, default collation declaration,
   non-identity, and non-generated state;
3. exactly the two primary keys and `templates.name` unique key, all validated, enforced when that
   catalog field is exposed, non-deferrable, initially immediate, local, non-inheritable, without
   PostgreSQL 18 `WITHOUT OVERLAPS` period semantics, and constraint-index-backed;
4. an empty complete FK inventory, empty CHECK inventory, and no exclusion or other relation
   constraints;
5. PostgreSQL 18-style NOT NULL constraint rows as either absent on older catalog versions or the
   exact fourteen-column set when exposed;
6. exactly five live, valid, ready, immediate B-tree indexes with no partial predicate, expression,
   included column, extra standalone uniqueness, clustering, replica-identity role, or unexpected
   index;
7. exact key order and direction, column collation OIDs, `text_ops` for varchar keys, and
   `timestamptz_ops` for the descending `created_at` key.

PostgreSQL 18 fresh-schema primary and unique constraints expose `connoinherit=true`. The key
probe therefore requires that value; the inverse condition would reject the migration's valid
`templates_pkey`, `templates_name_key`, and `notifications_pkey` rows. NOT NULL constraints retain
their separately versioned zero-row/pre-18 or exact-fourteen/18+ inventory semantics.

Evidence validation is dual-pinned. The contract records the query and migration bytes/digests,
while the verifier carries independent hardcoded canonical bytes/digests. Repinning a tampered
fixture contract therefore cannot authorize different SQL. The verifier also checks an exact
catalog-identifier occurrence inventory and the complete 191-byte down body, including matching
`$forward_only$` delimiters; same-length identifier or delimiter corruption fails integrity.

The exactness is intentional. A pre-existing table with extra columns, constraints, indexes, RLS,
inheritance, the wrong default, nullable model-required columns, a timestamp without the expected
precision, or a same-named but structurally different index fails startup before cache load and
listener binding.

## Fresh versus populated databases

This migration is a fresh-schema definition. `CREATE TABLE IF NOT EXISTS` is a complete no-op when
a relation already exists. It cannot repair the old runtime-created `notifications` table that
omitted `read_at`, `title`, `notification_type`, `priority`, and `action_url`, nor can it strengthen
old nullable defaults. Startup correctly rejects such a shape.

A populated additive upgrade must separately inventory rows, choose safe backfills, resolve
duplicates/nulls, adopt migration-ledger history, reconcile `wallet_notifications`, and prove a
single writer. None of those operations is authorized or implemented here.

## Residual STOPs

Seven A3.11 blockers remain: unsafe/ambiguous pre-existing migration history and print-schema
tooling drift, no empty-database
execution, no populated upgrade, no legacy mapping/backfill/reconciliation or single-writer proof,
no ledger-adoption exercise, no recovery/rollback procedure, and no deployment/cutover evidence.
The broader A11 notification gate therefore retains all 22 STOP blockers and readiness exit `3`.

## Offline verification

```sh
./scripts/migration/verify-a3-11-notification-schema-boundary.sh --mode integrity
./scripts/migration/verify-a3-11-notification-schema-boundary.sh --mode readiness  # expected exit 3
./scripts/migration/verify-a3-11-notification-schema-boundary.sh --mode report
./scripts/migration/test-a3-11-notification-schema-boundary.sh
```

The verifier refuses production-looking, database, Redis, SMTP, and proxy environment variables.
Its self-test covers readiness claims, source/query/migration digests, columns/defaults, keys,
PostgreSQL 18 `connoinherit=true` key behavior, indexes, FK/CHECK inventories, inheritance/RLS,
opclass/collation, mutation tokens, restored sample seeding, path safety, and environment refusal
without contacting live infrastructure.
