# A3.3 Runtime Rust DDL Triage

Status: **offline/static integrity complete; production readiness STOP**.

This package deterministically expands the runtime Rust DDL evidence already pinned by
[`migration-safety.json`](contracts/migration-safety.json). It records the combined static
removal of the analytics, subscription, wallet, content, notification, indexer, and pay
service-startup findings, but does not inspect a database, run a migration, or claim the broader
runner/upgrade/reconciliation risks are complete. The canonical remediation assignment remains
A3.6 in the upstream risks.

## Fixed evidence boundary

- Upstream contract SHA-256:
  `53db624ba569b6789d0c661f63489d3aee758ea44ca6b788829ee6f9371bd79f`
- Tracked Rust files: 1,124
- Scanner findings: 9
- Exact reviewed exceptions: 6
- Actionable findings: 3, all `blocked`
- Service-startup schema mutations: 0
- Tracked migration SQL files: 175
- Registered migration roots: 15
- Destructive SQL findings: 511 (digest
  `cda0fbb7411db38cc02a4c4d7ec97d26b15aaff5a5faa9281ff96e3e763e9132`)
- Production ready: `false`
- Readiness result: `STOP`

The triage contract enumerates all 9 findings in the scanner's stable order with the exact
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
| Actionable | 3 | Remains blocked |
| Exact reviewed exception | 6 | Not runtime application DDL; does not prove remediation |

### DDL kind

| Normalized kind | Findings |
|---|---:|
| `CREATE DATABASE` | 3 |
| `CREATE SCHEMA` | 4 |
| `CREATE TABLE` | 1 |
| `DROP TABLE` | 1 |

The six reviewed exceptions account for one `DROP TABLE`, four `CREATE SCHEMA`, and one
`CREATE TABLE` lexical match. The remaining three actionable findings are all `CREATE DATABASE`
matches: two error-message lexical matches and one backend migration-binary bootstrap statement.

### Boot-time risk triage

| Evidence-backed group | Findings | Interpretation |
|---|---:|---|
| `service-startup-schema-mutation` | 0 | Analytics, subscription, wallet, content, notification, indexer, and pay startup DDL findings have been removed statically |
| `runtime-database-bootstrap` | 1 | `CREATE DATABASE` in the tracked migration binary |
| `lexical-match-not-schema-ddl` | 2 | “create database pool” error strings; still actionable because they are not upstream exceptions |
| `reviewed-exception-not-runtime-ddl` | 6 | Exact test-only exceptions already reviewed upstream |

Lexical triage never silently creates an exception. The two pool-message matches therefore
remain in the 3 actionable findings until the canonical migration-safety contract is changed
through its explicit exception review process.

### Service group

| Service group | Findings | Actionable | Reviewed exceptions |
|---|---:|---:|---:|
| backend-blockchain-monitor | 1 | 1 | 0 |
| backend-main | 1 | 1 | 0 |
| backend-migrate | 1 | 1 | 0 |
| backend-security-test | 1 | 0 | 1 |
| backend-smoke-test | 5 | 0 | 5 |

### File group

| File | Findings |
|---|---:|
| `apps/backend/__test__/web3/web3_security_tests.rs` | 1 |
| `apps/backend/src/bin/blockchain_monitor.rs` | 1 |
| `apps/backend/src/bin/migrate.rs` | 1 |
| `apps/backend/src/main.rs` | 1 |
| `apps/backend/tests/wave11_smoke.rs` | 2 |
| `apps/backend/tests/wave12_smoke.rs` | 3 |

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

The canonical inventory records `services/analytics/migrations`,
`services/identity/migrations`, `services/subscription/migrations`,
`services/wallet/migrations`, `services/content/migrations`, `services/indexer/migrations`, and
`services/pay/migrations` as partial manual migration roots with `runner: null`; notification
uses the active backend notifications root. Those classifications acknowledge tracked additive
SQL without claiming runner/version-ledger wiring, deployed-baseline adoption, source-version
upgrade, backfill, or reconciliation proof. Reaching zero service-startup mutations does not
align either upstream A3.6 risk because the required executable database and rollout evidence is
absent.

## Offline verification

```bash
scripts/migration/verify-a3-3-runtime-ddl-triage.ts
scripts/migration/verify-a3-3-runtime-ddl-triage.ts --json
scripts/migration/test-a3-3-runtime-ddl-triage.sh
```

The synthetic self-test compares two generated JSON reports, asserts the readiness command
exits `2` with `STOP`, and exercises fail-closed tampering for production readiness, an
actionable status, an exception mapping, a grouped count, the upstream checksum, and a symlinked
contract, plus deletion of a remaining enumerated finding.

```bash
scripts/migration/verify-a3-3-runtime-ddl-triage.ts --readiness
```

An integrity pass is expected. A readiness pass is not: `--readiness` always exits `2` while the
3 actionable backend findings are blocked and executable database proof is absent.
