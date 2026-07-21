# A2.3e indexer direct-service authorization

Status: **fail-closed service boundary implemented; indexer production readiness remains STOP**.

This package changes only the `services/indexer` HTTP boundary and its local evidence. It does not run or change database migrations, query a chain/RPC endpoint, repair placeholder ingestion, backfill data, alter Kubernetes, or authorize deployment.

## Locked route decisions

| Method and path | Boundary decision | Reason |
|---|---|---|
| `GET`/`HEAD /health` | anonymous | The exact health surface strips bearer and spoofable identity headers. |
| `GET /api/v1/indexer/status/{chain}` | `404` before DB/RPC | The disabled handler mixes DB `MAX` with provider fallback and lacks canonical/finalized/degraded state. |
| `GET /api/v1/indexer/block/{chain}/{number}` | `404` before SQL | Existing sync writes number-derived placeholder block fields and has no canonical/reorg/finality model. |
| `GET /api/v1/indexer/tx/{chain}/{hash}` | `404` before SQL | Canonical chain-scoped transaction, receipt, finality, and reorg ingestion is absent. |
| `GET /api/v1/indexer/transfers/{chain}/{address}` | `404` before SQL | Transfer logs are not ingested and public-versus-owner address-history policy is undecided. |
| `POST /api/v1/indexer/sync` | verify exact admin audience plus `admin:indexer:manage`, then `404` before handler | Authentication is proven, but the current handler writes placeholder data from an in-memory cursor. |
| every other method/path/arity, encoded/ambiguous path, or reserved dynamic segment | `404` before verifier/handler | The boundary is a strict normalized-path allowlist. |

The sync mount is narrowed from `ANY` to `POST`. Exact, resource-wildcard (`admin:indexer:*`) and domain-wildcard (`admin:*:*`) grants follow the shared permission matcher; wrong audiences, missing/invalid credentials, and invalid wildcard shapes are denied. No caller body, query value, or identity-looking header is trusted.

## Verification

Run only local, locked evidence:

```bash
cargo test --locked -p epsx-indexer
cargo test --locked -p epsx-service-auth --lib
./scripts/migration/verify-service-authorization.sh
./scripts/migration/verify-permission-grammar.sh --mode integrity
./scripts/migration/test-permission-grammar.sh
./scripts/migration/verify-analytics-indexer-execution.sh --mode integrity
./scripts/migration/test-analytics-indexer-execution.sh
git diff --check
```

The indexer suite contains nine focused boundary tests. Passing them proves exact local classification, shared verifier use, audience/permission denial, credential/header stripping, runtime method-mismatch `404`, and pre-handler fail closure. It does not prove DB contents, RPC truth, durable checkpoints, finality, reorg handling, backfill, observability, production workload wiring, or public address-history privacy.

## Residual STOP blockers

- Runtime DDL and the globally keyed transaction hash must move to additive, chain-scoped migrations.
- Placeholder block writes must be replaced with canonical block, transaction, receipt, and decoded-log ingestion.
- The in-memory cursor must become a leased durable checkpoint advanced atomically with indexed data.
- Parent continuity, canonical/finalized heads, reorg rollback/replay, bounded backfill, retries, cancellation, and reconciliation remain absent.
- Address history needs an explicit public or owner/admin policy plus cursor and finality semantics.
- Dependency-aware readiness, internal metrics, lag/reorg alerts, workload manifests, shadow comparison, rollback proof, and separate deployment authorization remain required.
