# A3.2 offline deployed-history classification

Status: **offline evidence gate only — not production-ready**.

A3.2 consumes the JSON directory written by a successful A3.1a read-only
database preflight. It verifies and classifies that captured evidence. It does
not connect to PostgreSQL, inspect a live environment, execute Diesel, repair
`__diesel_schema_migrations`, generate reconciliation SQL, run DDL, access the
network, deploy, or authorize any later operation.

The machine contract is
[`a3-2-history-classification.json`](contracts/a3-2-history-classification.json).
It checksum-pins the A3.1 source contract, so the class vocabulary cannot drift
silently from A3.1's recovery matrix.

## Invocation

Use the canonical absolute path printed by resolving the A3.1 artifact
directory. The directory must be outside the repository and must contain
exactly the eight domain artifacts plus `manifest.json`:

```bash
artifact_dir="$(cd /operator/secure/epsx-a3-1-preflight && pwd -P)"

./scripts/migration/a3-2-history-classification.sh \
  --input-dir "$artifact_dir" \
  --output /operator/secure/epsx-a3-2-classification.json
```

Omit `--output` to write the deterministic report to stdout. An explicit
output must be a canonical absolute new file outside the repository. The tool
never overwrites an existing path.

Exit `0` means all four domains matched exactly one recognized evidence class.
The report still declares `productionReady: false`. Exit `2` is a
machine-readable evidence STOP. Exit `64` is an unsafe invocation, including a
symlink, path traversal, repository path, or overwrite attempt.

## Integrity and redaction boundary

Before classification, the tool requires all of the following:

- the exact domain set `core`, `analytics`, `notifications`, and `payments`;
- the exact filenames `<domain>.discovery.json` and
  `<domain>.inspection.json`, plus `manifest.json`, with no extra entry;
- the A3.1 package, purpose, non-production status, empty stop list, and exact
  eight-entry artifact inventory;
- a matching SHA-256 digest for every artifact;
- valid JSON with no PostgreSQL URI, private-key PEM, or credential-shaped
  field;
- the read-only discovery marker and exactly one regular
  `__diesel_schema_migrations` table;
- matching discovery and inspection migration-table identities;
- ordered, unique, digit-only migration versions, including the Diesel setup
  and version-1 rows;
- only checksum-contract-known migration versions and domain relations;
- unique relation and column fingerprints, with every column attached to a
  captured relation.

The emitted report contains only the manifest SHA-256, domain, evidence class,
and a SHA-256 of the normalized migration/relation/column fingerprint. It does
not reproduce database names, users, addresses, timestamps, row estimates,
relations, columns, constraints, indexes, functions, triggers, views, or any
artifact path. Reports contain no clock value and are byte-for-byte
deterministic for identical evidence.

## Exact supported evidence classes

Every class requires the full baseline relation landmarks and required column
landmarks pinned in the contract. A relation outside the per-domain allowlist
is not ignored; it is a STOP.

| Domain | Supported class | Exclusive evidence discriminator |
|---|---|---|
| Core | `v5` | Only base history; complete v5 landmarks; `plans.display_order` present and `plans.plan_category` absent. |
| Core | `v6` | Only base history; complete v6 landmarks; category/group/system/grace columns present and `display_order` absent. |
| Core | `known-partial-v5` | One or more known follow-up versions plus the v5 column profile. |
| Core | `known-partial-v6` | One or more known follow-up versions plus the complete v6 profile. |
| Analytics | `public-v2` | Complete public v2 relations, including the 2025-01 partition; no `infra_logs.api_key_usage_logs`. |
| Analytics | `public-v3` | Complete public v3 relations through the 2026-03 partition and unified audit log; no 2025-01 or infra_logs root table. |
| Analytics | `infra-logs-v3` | Complete v3 relations in `infra_logs`; no public root usage-log table. |
| Notifications | `baseline-v2-table-present` | Both notification tables exist and the recorded-drop version is absent. |
| Notifications | `recorded-drop-table-absent` | The recorded-drop version exists, `wallet_notifications` exists, and `notification_subscriptions` is absent. |
| Payments | `v3` | Only base history; complete v3 relations; no credit tables or audit `tx_hash`. |
| Payments | `v4` | Only base history; complete v4 relations, both credit tables, and audit `tx_hash`. |
| Payments | `known-partial-v3` | One or more known follow-up versions plus the v3 schema profile. |
| Payments | `known-partial-v4` | One or more known follow-up versions plus the v4 schema profile. |

"Known partial" means only that every recorded version is from the pinned
known set and the captured schema satisfies that profile. It is not proof that
a migration completed, data reconciles, constraints are valid for future
writes, or the database can be upgraded safely.

## Mandatory STOP cases

The classifier stops rather than guesses when evidence is:

- missing, extra, renamed, invalid JSON, or SHA-256 tampered;
- accessed through a symlink, a noncanonical `..` path, or a path in the
  repository;
- credential-bearing or contains a PostgreSQL connection URI/private key;
- missing or ambiguous about the Diesel migration table;
- missing either required base migration version, out of order, duplicated, or
  carrying an unknown version;
- carrying an unknown application relation or an orphan/duplicate fingerprint;
- incomplete for every recognized class;
- hybrid, including simultaneous public and `infra_logs` analytics roots;
- matching zero classes or, defensively, more than one class.

A classified report is still only a bounded input to later operator review.
Backup/PITR proof, environment-to-database identity confirmation, exact
critical row counts/checksums, migration-runner logs, restore rehearsal,
concurrent-write controls, lock/replica budgets, and a separately reviewed
forward-only reconciliation package remain mandatory before any mutation.

## Synthetic offline proof

Run:

```bash
./scripts/migration/test-a3-2-history-classification.sh
```

The self-test creates temporary synthetic JSON only. It proves all thirteen
positive classes, deterministic redaction, digest tamper failure, unknown
history/relation failure, incomplete and hybrid failure, credential removal,
exact filename enforcement, traversal/symlink refusal, and no-overwrite output.
It places failing stand-ins for `psql`, network clients, Docker, and `kubectl`
ahead of `PATH`; any accidental invocation fails the test. No database URL,
`psql`, network, Docker, Kubernetes, live filesystem artifact, or deployment is
used.
