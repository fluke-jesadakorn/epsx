# A3.3 Runtime Rust DDL Triage

Status: **offline/static integrity complete; production readiness STOP**.

This package deterministically expands the runtime Rust DDL evidence already pinned by
[`migration-safety.json`](contracts/migration-safety.json). It does not inspect a database,
run a migration, create forward SQL, or claim that any runtime DDL gap is remediated. The
canonical remediation assignment remains A3.6 in the upstream risks.

## Fixed evidence boundary

- Upstream contract SHA-256:
  `6b4c3779cd5fcb8c9539bf071fad55c14eb31fdf7b9a567ae7a86ffaf25f58d1`
- Tracked Rust files: 1,124
- Scanner findings: 37
- Exact reviewed exceptions: 6
- Actionable findings: 31, all `blocked`
- Production ready: `false`
- Readiness result: `STOP`

The triage contract enumerates all 37 findings in the scanner's stable order with the exact
file, line, normalized DDL kind, classification, reviewed-exception ID (when present), service
group, boot-time risk group, and blocked status. The upstream scanner digest also pins the
normalized source text without duplicating that text here.

## Scanner parity

[`verify-a3-3-runtime-ddl-triage.ts`](../../scripts/migration/verify-a3-3-runtime-ddl-triage.ts)
reproduces the migration-safety runtime scanner rather than using a second interpretation:

1. Read sorted tracked `*.rs` paths from `git ls-files -z`.
2. Remove `//` and `/* ... */` comments with the same line-preserving routine.
3. Match the same case-insensitive `CREATE`, `ALTER`, `DROP`, or `TRUNCATE` object pattern.
4. Normalize whitespace, sort by file/line/text, and compute the same SHA-256 digest.
5. Resolve each reviewed exception by its exact file and unique anchor from the checksum-pinned
   upstream contract.
6. Compare every enumerated finding and every grouped count to the triage contract.

Any upstream byte change, scanner result change, exception remap, finding-status promotion,
group-count change, credential-shaped contract content, or symbolic-link contract path fails
closed.

## Exact grouped inventory

### Classification

| Classification | Findings | Readiness meaning |
|---|---:|---|
| Actionable | 31 | Remains blocked |
| Exact reviewed exception | 6 | Not runtime application DDL; does not prove remediation |

### DDL kind

| Normalized kind | Findings |
|---|---:|
| `CREATE DATABASE` | 3 |
| `CREATE INDEX` | 10 |
| `CREATE SCHEMA` | 4 |
| `CREATE TABLE` | 19 |
| `DROP TABLE` | 1 |

The six reviewed exceptions account for one `DROP TABLE`, four `CREATE SCHEMA`, and one
`CREATE TABLE` lexical match. The remaining 31 actionable findings account for three
`CREATE DATABASE`, ten `CREATE INDEX`, and eighteen `CREATE TABLE` matches.

### Boot-time risk triage

| Evidence-backed group | Findings | Interpretation |
|---|---:|---|
| `service-startup-schema-mutation` | 28 | SQL in the six services still named by upstream `runtime.service-schema-ddl` evidence |
| `runtime-database-bootstrap` | 1 | `CREATE DATABASE` in the tracked migration binary |
| `lexical-match-not-schema-ddl` | 2 | “create database pool” error strings; still actionable because they are not upstream exceptions |
| `reviewed-exception-not-runtime-ddl` | 6 | Exact test-only exceptions already reviewed upstream |

Lexical triage never silently creates an exception. The two pool-message matches therefore
remain in the 31 actionable findings until the canonical migration-safety contract is changed
through its explicit exception review process.

### Service group

| Service group | Findings | Actionable | Reviewed exceptions |
|---|---:|---:|---:|
| backend-blockchain-monitor | 1 | 1 | 0 |
| backend-main | 1 | 1 | 0 |
| backend-migrate | 1 | 1 | 0 |
| backend-security-test | 1 | 0 | 1 |
| backend-smoke-test | 5 | 0 | 5 |
| content | 4 | 4 | 0 |
| indexer | 5 | 5 | 0 |
| notification | 4 | 4 | 0 |
| pay | 10 | 10 | 0 |
| subscription | 2 | 2 | 0 |
| wallet | 3 | 3 | 0 |

### File group

| File | Findings |
|---|---:|
| `apps/backend/__test__/web3/web3_security_tests.rs` | 1 |
| `apps/backend/src/bin/blockchain_monitor.rs` | 1 |
| `apps/backend/src/bin/migrate.rs` | 1 |
| `apps/backend/src/main.rs` | 1 |
| `apps/backend/tests/wave11_smoke.rs` | 2 |
| `apps/backend/tests/wave12_smoke.rs` | 3 |
| `services/content/src/main.rs` | 4 |
| `services/indexer/src/main.rs` | 5 |
| `services/notification/src/main.rs` | 4 |
| `services/pay/src/db.rs` | 10 |
| `services/subscription/src/main.rs` | 2 |
| `services/wallet/src/main.rs` | 3 |

## Remediation requirements carried forward unchanged

Only the two blocked upstream A3 risks are attached to this inventory:

- `runtime.service-schema-ddl`: move runtime schema mutations into per-service versioned
  migrations and make readiness fail when required versions are absent. Required proof remains
  a zero runtime-DDL scan, fresh and source-version upgrade tests, and a concurrent-startup test.
- `runtime.missing-service-migrations`: establish immutable ordered service migration roots,
  adopt existing populated schemas without recreating data, and wire a pre-rollout migration
  job. Required proof remains a migration-root inventory, populated-schema baseline adoption
  test, and upgrade idempotency test.

These requirements are copied from the checksum-pinned upstream risk objects and verified byte
for byte at the field level. This triage adds no priority, dependency, database-state claim, or
forward SQL.

The canonical inventory now also records `services/analytics/migrations` as a partial manual
migration root with `runner: null`. That classification acknowledges the tracked additive SQL
without claiming runner wiring, deployed-baseline adoption, source-version upgrade, or
reconciliation proof. Removing the analytics and identity startup DDL findings therefore does
not align either upstream A3.6 risk.

## Offline verification

```bash
scripts/migration/verify-a3-3-runtime-ddl-triage.ts
scripts/migration/verify-a3-3-runtime-ddl-triage.ts --json
scripts/migration/test-a3-3-runtime-ddl-triage.sh
```

The synthetic self-test compares two generated JSON reports, asserts the readiness command
exits `2` with `STOP`, and exercises fail-closed tampering for production readiness, an
actionable status, an exception mapping, a grouped count, the upstream checksum, and a symlinked
contract.

```bash
scripts/migration/verify-a3-3-runtime-ddl-triage.ts --readiness
```

An integrity pass is expected. A readiness pass is not: `--readiness` always exits `2` while the
31 actionable findings are blocked and executable database proof is absent.
