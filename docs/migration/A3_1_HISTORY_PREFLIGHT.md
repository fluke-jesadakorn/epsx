# A3.1a Diesel migration-history preflight

Status: **preflight only — not production-ready**.

This package inventories the four legacy Diesel roots and captures read-only
database catalog evidence. It does not repair migration history, run a
migration, execute DDL, update `__diesel_schema_migrations`, prove a backfill,
or authorize a deployment.

## Why the preflight stops

The backend is locked to Diesel 2.1. Diesel derives a file-based migration
version from the directory prefix before the first underscore and removes all
hyphens. Both of these names therefore have version `00000000000001`:

```text
00000000000001_consolidated_schema_v5
00000000000001_consolidated_baseline_v6
```

Diesel stores only `version` and `run_on`. It stores no directory name or SQL
checksum, so an applied version-1 row cannot identify which colliding baseline
ran. Duplicate versions are also loaded through a version-keyed map, making
fresh execution dependent on which directory wins the collision.

Never run `diesel migration revert`, `redo`, or `revert --all` against this
history. Never repair it by manually inserting or deleting Diesel history rows.

## Static history classifier

Run from the repository root or any other directory:

```bash
./scripts/migration/a3-1-history-preflight.sh > /tmp/epsx-a3-1-history.json

cd /tmp
/absolute/path/to/epsx/scripts/migration/a3-1-history-preflight.sh \
  --output /tmp/epsx-a3-1-history-exclusive.json
```

The tool:

- resolves and verifies the pinned `origin/development` commit;
- applies Diesel's exact normalized-version rule to every immediate migration
  directory in core, analytics, notifications, and payments;
- inventories duplicate versions in the source and worktree;
- SHA-256 fingerprints every `up.sql` and `down.sql` in the source/worktree
  union;
- compares shared paths byte-for-byte;
- verifies checksum-pinned canonical source files;
- detects deleted, rewritten, unreviewed-added, or incorrectly relocated SQL;
- stops on active alternate baselines, destructive forward SQL, session
  `search_path` changes, and direct Diesel-history mutation;
- emits deterministic JSON with `productionReady: false`.

Exit `2` means the report was successfully generated and contains stop reasons.
That is the expected result on the current branch. Exit `64` means the request
or output path was unsafe/invalid. A future exit `0` means only that static
history is clear; database evidence is still mandatory.

The source contract is
[`a3-1-history-preflight.json`](contracts/a3-1-history-preflight.json). Approved
archive relocations and new migrations must be explicit, checksum-locked
contract entries. An archive target must remain outside every active Diesel
root.

## Read-only database catalog capture

The database command has no defaults. Supply all four URLs explicitly through
the process environment:

```bash
export A3_1_CORE_DATABASE_URL='postgresql://...'
export A3_1_ANALYTICS_DATABASE_URL='postgresql://...'
export A3_1_NOTIFICATIONS_DATABASE_URL='postgresql://...'
export A3_1_PAYMENTS_DATABASE_URL='postgresql://...'

./scripts/migration/a3-1-database-preflight.sh
```

By default, artifacts are written to a new mode-0700 directory under `TMPDIR`.
To choose a location, pass an absolute, nonexistent directory:

```bash
./scripts/migration/a3-1-database-preflight.sh \
  --output-dir /secure/operator-selected/epsx-a3-1-preflight
```

The command refuses an existing directory, `/`, `$HOME`, and every path inside
the repository. URL values must use a PostgreSQL URI scheme and may not contain
literal whitespace, control characters, or a URI fragment. URLs and credentials
are never written to artifacts.

Every `psql` call uses:

```text
-X -v ON_ERROR_STOP=1
PGOPTIONS=-c default_transaction_read_only=on ...
BEGIN READ ONLY;
SELECT ...;
ROLLBACK;
```

The first query captures database identity and discovers migration tracking
tables. Inspection stops before the schema fingerprint query unless exactly one
regular `__diesel_schema_migrations` table exists. The second query captures:

- ordered migration versions and timestamps;
- user relations and estimated row counts;
- columns, types, nullability, and defaults;
- constraints and validation state;
- indexes, triggers, functions, partitions, and views.

Each artifact is canonicalized with `jq -S`; `manifest.json` records SHA-256
digests and still declares `productionReady: false`. Exact critical-table row
counts and bounded data checksums remain an operator follow-up because this
preflight intentionally avoids production-sized scans.

## Recovery classification matrix

| Domain | Canonical active source | Possible deployed evidence | Safe recovery direction |
|---|---|---|---|
| Core | `consolidated_schema_v5` | v5, v6, or known partial chain | One new unique forward reconciliation; never replay a colliding baseline. |
| Analytics | `consolidated_analytics_v2` | public v2, public v3, or `infra_logs` v3 | Schema-qualified forward object moves. Abort if both public and `infra_logs` contain the same required relation. Never `SET search_path` in migration SQL. |
| Notifications | `consolidated_baseline_v2` | subscriptions table present or recorded drop/table absent | Retain by default; forward-restore only after backup/log evidence. The two source baseline aliases are byte-identical. |
| Payments | `consolidated_payments_v3` | v3, v4, or known partial chain | One new unique forward reconciliation after validating balances, addresses, reference types, and transaction-hash uniqueness. |

The newer consolidated core, analytics, and payments baselines already include
objects that their later incremental migrations create. Keeping those newer
baselines active does not make a safe fresh chain; it creates duplicate-object
failures.

## Mandatory stop conditions

Do not proceed to a recovery agent or database mutation when any of these is
true:

- no restore-tested backup/PITR point exists;
- database URL-to-domain identity is not operator-confirmed;
- zero or multiple Diesel migration tables exist;
- a version-1 row exists but the schema does not match a recognized fingerprint;
- both old and target analytics schemas contain the same required relation;
- notification drop history is present without pre-drop row/backup evidence;
- payment/core data violates a constraint needed by the target schema;
- an unknown migration version or untracked application table is present;
- concurrent schema work, unexpected active writes, or unacceptable lock/replica
  conditions are observed;
- any plan proposes `DROP`/recreate, historical `down.sql`, or manual mutation of
  `__diesel_schema_migrations`.

## Evidence still required before implementation

An operator must provide the exact environment/database mapping, deployment
commit or image, migration-runner logs, migration-table exports, schema-only
dumps, critical row counts, confirmation of whether the rewritten analytics,
notification-drop, and payments-replica migrations ran, and a restore rehearsal.

Only after that evidence is classified may a separate package add new unique,
transactional, forward-only reconciliation migrations and disposable-database
upgrade fixtures. A3.1a by itself is deliberately a red gate, not a release
gate pass.
