# A3.10 Content Schema Boundary

Status: **partial static remediation; production readiness STOP**.

The content service no longer creates `pages`, `themes`, `block_types`, or `edit_sessions`
during startup. Startup now connects to `epsx_content`, runs one read-only compatibility query,
and fails before filesystem-to-database synchronization or listener binding when the exact public
schema is absent or incompatible. All 19 runtime SQL relation references are explicitly qualified
with `public`, so a different `search_path` cannot redirect reads or writes after the public schema
probe succeeds.

This is a schema-boundary package, not content-lifecycle parity. It changes no route, authorization
policy, public-news fallback, unknown-slug behavior, plan/ranking/portfolio response, watcher policy,
or sample data. In particular, editor routes still fail closed with HTTP `404` in the existing
authorization layer before their handlers run.

## Exact runtime finding delta

The current canonical A3.3 inventory contains four actionable content findings,
`finding.010` through `finding.013`, all in `services/content/src/main.rs`. They are the four
startup `CREATE TABLE` statements removed by this package. The same scanner has zero findings
under `services/content/**/*.rs` after the change.

Against the 35-finding canonical inventory present when A3.10 was authored, its isolated
projection is:

| Inventory | Before | Projected after | Delta |
|---|---:|---:|---:|
| Runtime Rust DDL findings | 35 | 31 | -4 |
| Actionable runtime findings | 29 | 25 | -4 |
| Content findings | 4 | 0 | -4 |
| Reviewed runtime exceptions | 6 | 6 | 0 |

The global worktree can contain concurrent remediations. A3.10 does not edit or silently rebaseline
the canonical migration-safety or A3.3 contracts; their owner must recompute the combined inventory.

## Pinned development news boundary

The provenance label is `origin/development`, but integrity never resolves that moving ref. It
reads only immutable commit `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` via
`git cat-file`/`git show <commit>:<path>`. Seven exact Git blobs pin the legacy
`news_articles` table, performance indexes, pin columns, Rust model/repository, and public/admin
APIs. That audit establishes a deliberate boundary: development news is a separate durable
authority with publication, pagination, featured ordering, and admin CRUD semantics. This A3.10
migration does not import, replace, backfill, or claim parity with it. The current file-backed
content-service news endpoints and their unknown-slug fallback remain unchanged.

## Immutable migration root

The new transaction-body-only root contains one ordered file:

`services/content/migrations/20260722030000_create_content_tables.sql`

The file is exactly 1,656 bytes with SHA-256
`b4eaf9ec57b1a823e0dad8a2a5fdb1b574488d6c7ebceb1187784cd505bba24d`.
It contains four guarded `CREATE TABLE IF NOT EXISTS public.*` statements and no `DROP`,
`TRUNCATE`, `DELETE FROM`, `ALTER`, data mutation, backfill, rewrite, extension creation, schema
creation, or transaction control. A future reviewed runner owns transaction and version-ledger
semantics.

The exact fresh-schema inventory is 34 columns:

| Table | Columns | Required keys |
|---|---:|---|
| `public.themes` | 8 | `id` PK; `name` unique |
| `public.pages` | 11 | `id` PK; `slug` unique |
| `public.block_types` | 9 | `id` PK; `block_type` unique |
| `public.edit_sessions` | 6 | `id` PK; `page_id` FK to `public.pages.id` |

The unique `themes.name` key is required by the existing
`ON CONFLICT (name) DO UPDATE` synchronization statement. Without a matching unique or exclusion
constraint PostgreSQL rejects that statement. The migration and compatibility probe validate the
key semantically rather than trusting an index or constraint name.

The edit-session foreign key preserves the legacy `ON DELETE CASCADE` action. Consequently, the
canonical migration-safety lexical scanner gains one explicit `CASCADE` finding. At the isolated
authoring snapshot, the projected SQL inventory was 169→170 files and 510→511 findings, with
projected digest `cda0fbb7411db38cc02a4c4d7ec97d26b15aaff5a5faa9281ff96e3e763e9132`.
That digest must be recomputed when concurrent migrations are combined. A3.10 classifies this as a
reviewed lexical safety STOP; it does not hide it, weaken the scanner, or claim any delete ran.

## Intentional fresh-schema drift

The removed runtime DDL is pinned at commit
`c0339d663123cb26ecd682aeea28e9917cf05b7f`, blob
`9d623fdd83780e3a13f18e439383e4fcba72b601`. Relative to that immutable snapshot, the new
fresh-schema definition intentionally adds exactly 18 structural requirements:

- 17 `NOT NULL` additions: five on `pages` (`locale`, `status`, `blocks_json`, `created_at`,
  `updated_at`); five on `themes` (`colors_json`, `fonts_json`, `spacing_json`,
  `breakpoints_json`, `is_default`); four on `block_types` (`schema_json`,
  `default_props_json`, `admin_only`, `updated_at`); and three on `edit_sessions` (`page_id`,
  `status`, `started_at`);
- one unique-key addition, `themes.name`, required by its existing `ON CONFLICT (name)` write.

These are fresh-schema definitions, not an upgrade. `CREATE TABLE IF NOT EXISTS` is a complete
no-op for a pre-existing table, so it cannot add any of the 17 nullability requirements or the
unique key to a table created by the old runtime DDL. Startup deliberately rejects that old shape.
Designing a populated additive upgrade, resolving null rows/duplicate theme names, adopting its
version, and reconciling results remain the blocked baseline-adoption and populated-upgrade work;
A3.10 does not run an `ALTER`, rewrite, or backfill.

## Fail-closed compatibility contract

The 15,196-byte compatibility query is pinned by SHA-256
`65a6e45346adc594b4a87f9090a346a924fe666a89c06715be145e46886ced61`.
It reads only `information_schema` and `pg_catalog` and verifies:

1. all four relations resolve in `public`, are ordinary permanent tables, and have exactly 34
   columns in the required ordinal order;
2. every column has the exact PostgreSQL data type, UDT, length, nullability, and default required
   by the runtime Rust model;
3. all four single-column `id` primary keys exist exactly once;
4. the distinct unique-key set is exactly `themes.name`, `pages.slug`, and
   `block_types.block_type`, one validated constraint per pair; each is non-deferrable and
   initially immediate, backed by a unique, non-primary, valid, ready, immediate, single-key,
   non-partial, expression-free index;
5. the complete inbound-and-outbound FK inventory involving any of the four content tables contains
   only the validated, non-deferrable
   `edit_sessions.page_id -> public.pages.id` reference with update `NO ACTION`, delete `CASCADE`,
   and simple match semantics; this also rejects an unexpected FK from another schema into a
   content table;
6. the complete `pg_index` inventory for `indisunique` indexes on the four tables contains exactly
   seven rows. Each row must be owned by its expected PK/UQ constraint and column, use `btree`,
   match the indexed column collation, use `uuid_ops` for UUID IDs or `text_ops` for the three
   varchar keys, and be unique, valid, ready, immediate, single-key, non-partial, and
   expression-free. Any extra standalone, partial, or expression unique index fails the boundary,
   even when it is not referenced by `pg_constraint`.

Missing and extra columns fail closed. Unexpected PK, unique, or FK constraints and unexpected
unique indexes fail closed. The
guarded migration is therefore not name-only adoption: a pre-existing wrong table is a SQL no-op at
migration time and is then rejected by startup. Default comparisons explicitly wrap the `CASE`
expression in `COALESCE(..., false)`, so `bool_and` cannot ignore a SQL `NULL` comparison and turn
an unknown default into a pass.

## PostgreSQL/Rust type audit

The dynamic SQL previously had four runtime incompatibility classes even if the tables existed:

- PostgreSQL `jsonb` fields were decoded into Rust `String`. Runtime projections now use
  `jsonb::text AS ...` for 38 response-field occurrences, preserving the legacy JSON string wire
  shape used by the preview/protobuf contracts.
- Filesystem theme/block JSON was bound as `String` to `jsonb` columns. It is now bound as
  `serde_json::Value`, matching SQLx/PostgreSQL JSONB typing without changing the stored JSON.
- `block_types.id` and all edit-session UUID fields were modeled as `String`. They now use
  `uuid::Uuid`; request strings are parsed before binding. UUIDs still serialize as JSON strings.
- `TIMESTAMPTZ` edit-session fields were modeled as `chrono::NaiveDateTime`. They now use
  `chrono::DateTime<chrono::Utc>` and retain RFC 3339 JSON strings.

Every JSON/UUID/timestamp projection names its columns explicitly; all seven former `RETURNING *`
queries now list the exact typed response columns. Nullable PostgreSQL values map only to Rust
`Option`: page SEO/theme/publication, theme radius, block description, and edit-session end time.
Required Rust fields correspond to `NOT NULL` columns in the new schema. The locked/offline model
tests freeze UUID/timestamp string serialization and page/block JSON string/null shapes.

## Offline evidence

```bash
cargo test --locked --offline -p epsx-content --lib
cargo test --locked --offline -p epsx-content --bin content
cargo check --locked --offline -p epsx-content --bin content
scripts/migration/verify-a3-10-content-schema-boundary.sh --mode integrity
scripts/migration/verify-a3-10-content-schema-boundary.sh --mode report
scripts/migration/test-a3-10-content-schema-boundary.sh
```

The verifier pins the immutable development commit and seven blobs, the immutable removed-runtime
snapshot and all 18 fresh-schema drift items, migration bytes/order, compatibility-query
bytes, all table/column definitions, constraints/index properties, model/bind anchors, startup
sequence, zero content runtime DDL, 19 qualified relation references, 38 JSONB text projections,
zero `RETURNING *`, the explicit cascade finding, and six blocked readiness categories. The
self-test proves deterministic reporting and detects readiness, migration, query, qualified-relation,
duplicate/missing/deferrable unique-key, standalone/partial/expression unique-index, FK-boundary,
cascade classification, stale source-commit, source-blob, blocker, production-environment, and
database-environment tampering.

`--mode readiness` intentionally exits `3`. No database, network, migration, service, container,
or deployment execution is performed.

## Residual A3.10 STOP blockers

- No migration runner or version ledger is wired.
- Safe baseline adoption of already matching deployed tables is unproven.
- No populated source-version upgrade has preserved real rows.
- No page/theme/block/session/news pre/post reconciliation has run.
- Concurrent migration/content-service startup ordering is untested.
- Neither the migration nor compatibility query has run against a live database.

Until those six blockers have executable evidence, `productionReady` remains `false`.
