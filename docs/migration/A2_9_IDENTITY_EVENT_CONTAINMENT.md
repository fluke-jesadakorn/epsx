# A2.9 identity event surface containment

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A2.9 removes the identity service's HTTP/SSE code path from the production
binary. Its production prefix no longer defines or parses `BIND_ADDR_SSE`,
binds port `50052`, creates an Axum listener, mounts `POST /v1/emit`, or mounts
`GET /v1/stream/ranking-offsets`. Historical emit-handler, event-bus and SSE
modules remain source-only behind `cfg(test)` so predecessor unit evidence can
still compile without making those modules part of a normal library or binary
build.

This is containment, not a replacement event design. A2.9 adds no publisher,
outbox, revision, cursor, replay or gap-repair mechanism. The identity gRPC
service remains on its existing `50051` default and still injects
`FreePlanRankingOffsetService`; every successful query therefore continues to
return the Free Plan offset. A2.7's resolver and A2.8's core adapter remain
unwired.

## Frozen boundary

The immutable target base is
`fd780ff257f0bc15910053704c5a59e5b3da4a3e`, the completed post-A2.8
snapshot. Its identity main, library, always-Free service and proto blobs pin
the removed exposure and preserved gRPC behavior.

Only `shared/rust/epsx-identity-service/src/main.rs` and
`shared/rust/epsx-identity-service/src/lib.rs` are implementation files in this
slice. The identity Cargo manifest, workspace lockfile, proto, always-Free
service, dormant handler/bus sources, migrations, generated schema, Diesel
configuration and Kubernetes files remain unchanged.

## Deliberately stale deployment configuration

The checked-in identity deployment and Service still advertise
`BIND_ADDR_SSE` and port `50052`; the analytics deployment still carries
`IDENTITY_SSE_URL`. A2.9 does not edit, render or apply those files. External
reachability and deployed image/configuration state were not exercised. Those
assumptions must be reconciled only inside a later reviewed
image/configuration/cutover packet. Integrity passing must not be read as
deployment readiness or proof that any externally reachable surface changed.

## Hermetic evidence

The verifier reads local Git objects and local files, validates frozen
SHA-256 digests, rejects production-looking and live dependency environment
variables, removes proxy variables and forces Cargo offline. Static inspection
proves that the production prefix of `main.rs` contains no second listener,
50052 configuration, event bus, emit mount or stream mount; the library exposes
the three historical modules only through `cfg(test)`. Three exact binary tests
and a package-scoped offline check preserve the gRPC-only code boundary without
starting a listener.

The deterministic self-test copies only the declared evidence files and
mutates readiness, pins, hashes, route tokens, `cfg(test)` guards, stale
manifest anchors, STOP inventory, paths and environment sentinels. Readiness
continues to exit `3`.

## Residual STOP conditions

- Identity still returns only the Free Plan offset; paid authority is unwired.
- The ranking SQL is unexecuted against a certified adopted schema and
  populated data.
- The missing functional-index decision, query plan, fan-out, latency,
  concurrency and reconciliation remain unproved.
- Identity gRPC lacks workload identity, exact caller authorization, owner
  binding and TLS policy.
- No authenticated publisher or durable outbox/revision/cursor/replay/repair
  replacement exists.
- Checked-in `50052` and SSE deployment configuration remains stale.
- No service, network, database, migration, image, manifest rendering,
  deployment, shadow, canary, cutover or rollback operation is authorized or
  proven.

## Gate usage

```sh
./scripts/migration/verify-a2-9-identity-event-containment.sh --mode integrity
./scripts/migration/verify-a2-9-identity-event-containment.sh --mode report
./scripts/migration/verify-a2-9-identity-event-containment.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-9-identity-event-containment.sh
```

Passing integrity proves only that the old event mounts are absent from the
production binary code path and the pre-existing gRPC behavior is preserved.
It is not evidence that external exposure or deployed configuration changed,
nor evidence of authoritative paid entitlement, durable events or production
readiness.
