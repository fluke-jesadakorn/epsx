# Wave 13a Track A — Deliverable

## What shipped

`epsx-identity-service` binary + tonic gRPC server + K8s base/dev overlay
on `origin/wave13a/track-a-identity`. The gRPC seam is now in place
for Track B (the analytics binary's gRPC client + fallback).

- `shared/proto/identity.proto` — `epsx.identity.v1.Identity` with
  one RPC `GetWalletRankingOffset(Request{wallet}) -> Response{offset}`
- `shared/rust/epsx-identity-service/` — new workspace crate
  (cargo name `epsx-identity-service`, binary name
  `epsx-identity-service`). `tonic-build` on the proto in build.rs,
  `identity_service.rs` implements the gRPC server delegating to a
  `FreePlanRankingOffsetService` that always returns
  `RankingOffset::free_plan()`. 5/5 lib tests pass.
- `Dockerfile` — single-build pattern (wave-13 retro), builds the
  binary inside the container so colima-BuildKit doesn't strip it
  to a 332 KB stub.
- K8s base — `infrastructure/kubernetes/base/identity/{deployment,
  service}.yaml`. Service is `ClusterIP` on port 50051 with gRPC
  readiness/liveness probes.
- Dev overlay — `infrastructure/kubernetes/overlays/dev/patches/
  services-identity.yaml` patches Service to `NodePort` 30104. The
  per-wave image tag `:wave13a-dev` is applied via the overlay's
  `images:` block.

## Deviation from spec

Crate renamed `epsx-identity` → `epsx-identity-service` to avoid
collision with `services/identity/` (the wave-10 extraction target
binary on port 8101, SIWE/Postgres/Redis-backed). Same suffix
pattern as wave-12's `epsx-analytics-service` vs `services/analytics`
collision. The K8s `metadata.name` stays `epsx-identity` (no
collision with the Cargo crate name) — the service DNS name
`epsx-identity:50051` is what Track B's `IDENTITY_GRPC_URL` env
var points at.

## Verification (track A scope)

- `cargo check --workspace` — clean, 16 pre-existing warnings
  unchanged. 2 cosmetic warnings on unused imports in the new
  crate, deferred.
- `cargo test -p epsx-identity-service` — 5/5 pass.
- `kubectl kustomize infrastructure/kubernetes/base` — identity
  Service rendered as `ClusterIP` on 50051.
- `kubectl kustomize infrastructure/kubernetes/overlays/dev` —
  identity Service rendered as `NodePort` 30104, image tag
  `epsx-identity:wave13a-dev`. All 5 services + 5 deployments
  + namespace = 11 resources present.
- `colima docker build` produced a real `epsx-identity-service`
  binary, not a stub. Image tag `epsx-identity:wave13a-dev`
  exists locally.
- `kubectl apply -k overlays/dev` — `service/epsx-identity
  created`, `deployment.apps/epsx-identity created`. Pod
  `epsx-identity-859cf67c49-zgrrt` reaches `1/1 Running` within
  13 seconds. gRPC readiness probe passes.
- gRPC round-trip — `grpcurl -plaintext 127.0.0.1:30104
  epsx.identity.v1.Identity/GetWalletRankingOffset
  -d '{"wallet": "0xdeadbeef"}'` returns `{"offset": 100}` which
  is `FREE_PLAN_RANKING_OFFSET`. Stub implementation works.

## Out of scope (deferred to Track B / wave-13a follow-up)

- Track B replaces the in-process
  `FreePlanWalletRankingOffsetQuery` stub in
  `apps/analytics/src/main.rs` with `GrpcWalletRankingOffsetQuery`
  (100 ms timeout + in-process fallback on tonic error).
- Track B adds `IDENTITY_GRPC_URL` env var to
  `infrastructure/kubernetes/base/analytics/deployment.yaml` with
  the dev overlay value `http://epsx-identity:50051`.
- The fallback contract test (kill `epsx-identity`, hit
  `/rankings`, expect 200 from the in-process fallback) is the
  integration-gate check; not exercised in track A.
- The "real" `services/identity/` binary on port 8101 lives in
  the wave-10 extraction roadmap; wave 13a just establishes the
  seam it will plug into.
- TLS / mTLS. Dev cluster has no cert-manager; production
  deployment is a separate decision.

## Branch

- `origin/wave13a/track-a-identity` pushed. **Not** fast-forwarded
  to `migration/dioxus-microservices` per user's "don't merge to
  dev" rule.
- Wave 13a integration branch will be `origin/wave13a/integration`
  (created by the integration gate after Track A verifier passes
  + Track B completes).
