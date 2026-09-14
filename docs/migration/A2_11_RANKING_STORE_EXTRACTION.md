# A2.11 ranking-store extraction

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A2.11 moves the existing A2.8 PostgreSQL ranking-entitlement snapshot adapter
from the backend monolith into the library-only `epsx-ranking-store` package.
The backend keeps a compatibility re-export, while the actual repository port,
result type, SQL shape, strict row decoder and twelve exact A2.8 tests remain
unchanged in meaning.

The extracted adapter still implements
`RankingEntitlementSnapshotRepository` and returns
`Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError>`. It does
not invent a second query DTO, binary, service, listener or runtime. The only
constructor adjustment is ownership of the existing cloneable `TlsPool`
instead of an `Arc<&'static TlsPool>` borrowed from the monolith.

This is a physical ownership boundary only. Identity does not depend on or
construct the new package. Production identity still serves the unauthenticated
always-Free implementation, and the A2.10 authenticated composition remains
exported but unwired.

## Frozen source and extraction base

The compatibility source remains `origin/development` at
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. The source ranking query, both
colliding core baselines, the generated-schema omission and the Diesel
table-filter omission remain pinned. They are compatibility evidence, not
proof of an adopted schema.

The immutable extraction base is
`005a604542271050279a6190fc00eada00f32137`, the completed post-A2.10 snapshot.
At that base the complete adapter and its twelve tests live under the backend,
the workspace and lockfile have no `epsx-ranking-store` package, and identity
has no ranking-store dependency. A2.11 freezes those base blobs before moving
the adapter.

## Package and compatibility boundary

`shared/rust/epsx-ranking-store` is one library package with only the
dependencies needed by the moved adapter: the shared repository contract,
shared TLS pool, async trait support, Diesel/Diesel Async and JSON decoding.
Serde remains test-only. There is no binary target, feature-controlled runtime,
configuration reader, environment lookup, network client or migration runner.

The workspace adds the package once. The backend adds one path dependency and
its former adapter file becomes only a public compatibility re-export of
`PostgresRankingEntitlementSnapshotRepository` and
`RANKING_ENTITLEMENT_SNAPSHOT_SQL`. Existing backend import paths therefore
continue to name the same extracted implementation without duplicating policy.

Identity Cargo, `main.rs`, the always-Free service, authenticated A2.10
composition and identity protobuf remain byte-identical to the extraction
base. In particular, identity does not gain an `epsx-ranking-store`
dependency.

## Preserved SQL, decoder and tests

The one schema-qualified, semicolon-free read-only statement retains one `$1`
wallet bind, one `statement_timestamp()`, four raw `LEFT JOIN`s, no entitlement
filter and the same selected-column order. Its extracted SQL digest is frozen
against the A2.8 digest.

The row decoder still returns the actual shared
`RankingEntitlementSnapshot`/`RankingEntitlementSnapshotError` contract. It
keeps sentinel-empty handling, fixed observation/wallet consistency,
deterministic assignment and permission grouping, idempotent equal duplicates,
and fail-closed corrupt-row handling. It does not resolve plan access or
ranking policy.

All twelve exact `a2_8_*` tests move to
`epsx_ranking_store::tests::<name>`. They continue to use the checked-in
twenty-one-case row fixture and inspect SQL/decoder behavior in-process. The
verifier enumerates the test binary, requires every fully qualified name once,
and runs each test separately with `--exact`.

No PostgreSQL instance is opened. These tests do not prove schema adoption,
query planning, data compatibility, isolation, populated reconciliation or
runtime composition.

## Hermetic evidence

The verifier reads only local files and Git objects, freezes the source/base
blobs and implementation digests, rejects production-looking and live
dependency variables, removes proxies, and forces Cargo offline. It checks the
workspace/package/backend compatibility surfaces, the actual repository result
contract, SQL/decoder semantics, protected identity/proto files and the exact
test inventory.

The self-test copies only declared evidence. It updates both copied digest pins
and copied files before mutating the backend re-export, repository result type,
SQL qualification and identity dependency boundary. Each mutation must pass
hash validation and fail a named semantic assertion. Readiness remains exit
`3`.

## Residual STOP conditions

- Neither colliding source baseline is certified as the schema adopted by a
  populated environment.
- Generated Diesel schema and table-filter omissions remain unresolved.
- No functional index matching `LOWER(wallet_address)` is proven.
- The SQL and decoder have no live PostgreSQL execution or compatibility
  evidence.
- No representative `EXPLAIN`, query bound, workload latency or concurrency
  evidence exists.
- Transaction isolation and MVCC consistency remain unproved.
- Fixture rows are not populated source-to-target reconciliation.
- Identity has no dependency on, construction of or wiring to the extracted
  store.
- A2.10 has no concrete workload verifier, issuer/rotation mechanism,
  analytics client metadata, TLS policy or owner-delegation binding.
- Production identity `main.rs` still selects the unauthenticated always-Free
  service.
- Ranking revisions, transactional outbox, durable cursor, replay and repair
  remain absent.
- No BFF or Dioxus UI consumes an authoritative paid ranking result.
- No runtime, listener, network, schema, migration, infrastructure or
  deployment surface is activated.
- No staging, canary, rollback, live parity or operational proof exists.
- Passing integrity never authorizes database, service, network, deployment or
  production action.

## Gate usage

```sh
./scripts/migration/verify-a2-11-ranking-store-extraction.sh --mode integrity
./scripts/migration/verify-a2-11-ranking-store-extraction.sh --mode report
./scripts/migration/verify-a2-11-ranking-store-extraction.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-11-ranking-store-extraction.sh
```

Passing integrity proves only the bounded package extraction and compatibility
surface. It is not schema, query-plan, runtime, identity-composition or
production-readiness evidence.
