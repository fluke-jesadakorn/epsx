# A2.6 ranking authority failure boundary

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A2.6 closes one narrow failure-mode gap after A2.5. Anonymous rankings retain
the Free Plan input and do not contact identity. A verified wallet must obtain a
successful ranking-offset decision before the canonical market provider is
called. Identity status, transport, timeout and invalid-wire failures are
sanitized as service unavailability and return before provider work.

The standalone analytics client now parses its identity endpoint and constructs
a lazy tonic channel. Startup and anonymous requests therefore do not dial the
identity service. An authenticated lookup performs one RPC attempt under one
100 ms deadline. There is no retry and no in-process Free Plan fallback.

This is deliberately not a claim that identity is authoritative. The current
identity server still returns Free Plan for every successful RPC and has no
database-backed plan lookup.

## Pinned evidence

The source baseline is `origin/development` at
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. It records the monolith plan query
and the prior authenticated error-to-Free downgrade as evidence, not as a safe
production target.

The immutable target base is
`a7f7ed0c0d0d3b07cb43414b1e3cd2a5f64bd5d1`, the completed post-A2.5 snapshot.
A2.6 does not modify or reinterpret the historical A2.5 evidence package.

## Hermetic proof boundary

The focused tests use an injected in-process RPC fake and counters. They prove
valid success, single-call status failure, timeout, strict wire validation,
lazy construction with an unreachable URI, and opaque malformed-URI rejection.
Handler structure and focused backend tests prove anonymous bypass and
authenticated denial before provider work. No test binds a listener or contacts
identity, PostgreSQL, Redis, a market provider, RPC, browser or deployment.

The machine-readable contract is
`docs/migration/contracts/a2-6-ranking-authority-failure-boundary.json`.

## Residual STOP conditions

- The identity success path is still always-Free and has no database authority.
- There is no single-statement/transactional plan-assignment snapshot, schema
  probe, migration-adoption proof, populated reconciliation or shadow parity.
- The identity RPC has no workload identity, owner binding or TLS policy.
- Identity SSE and emit remain unauthenticated, global, ephemeral and unrelated
  to an atomic entitlement change.
- The monolith remains the route owner; no runtime configuration, image,
  gateway, Cloudflare, Kubernetes, canary or rollback change is authorized.
- No live or production action is authorized by this package.

## Gate usage

```bash
./scripts/migration/verify-a2-6-ranking-authority-failure-boundary.sh --mode integrity
./scripts/migration/verify-a2-6-ranking-authority-failure-boundary.sh --mode report
./scripts/migration/verify-a2-6-ranking-authority-failure-boundary.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-6-ranking-authority-failure-boundary.sh
```

Integrity is offline and refuses live or production-looking configuration.
Readiness intentionally exits `3` while the eight recorded STOPs remain.
