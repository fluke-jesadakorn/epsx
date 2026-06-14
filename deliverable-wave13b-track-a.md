# Wave 13b — Track A (SSE server + pub-sub + admin emit hook) — Deliverable

> **NOTE on filename:** the wave-13a integration commit
> already wrote its deliverable to
> `deliverable.md` at the worktree root. To avoid
> clobbering the prior wave's report (per the
> `epsx-wave12-track-b-infra-cleanup` memory's
> "worktree-root deliverable.md collision" pattern),
> this wave's report is at
> `deliverable-wave13b-track-a.md`. The engine-required
> deliverable is at
> `/Users/fluke/.mavis/plans/plan_f6609db9/outputs/track-a-sse-server/deliverable.md`.

## What shipped

Extended the `epsx-identity-service` binary (wave 13a
Track A) with a real-time Server-Sent Events endpoint
that broadcasts `RankingOffsetChange` events to
subscribers, plus a `POST /v1/emit` admin hook that the
integration gate (and future tier-aware impls) can use
to publish changes. Track B will consume the SSE stream
from the analytics binary; the integration gate will
exercise the full round-trip end-to-end on the dev
cluster.

- **Branch:** `wave13b/track-a-sse-server` (pushed).
  **Base:** `origin/migration/dioxus-microservices` HEAD
  `60305b6c` (the wave-13a integration head).
- **Final commit hash:** `<populated at end of run>`.

## Smoke output

### `cargo check --workspace`
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 46s
```
16 pre-existing warnings in `epsx` (apps/backend) —
unchanged from wave 13a. 0 new warnings.

### `cargo test -p epsx-identity-service`
```
running 10 tests
test tests::test_proto_file_included ... ok
test tests::test_stub_is_arc_dyn_compatible ... ok
test tests::test_identity_server_accepts_grpc_service ... ok
test tests::wave13b_test_event_bus_publish_with_zero_subscribers ... ok
test tests::wave13b_test_event_bus_broadcast_fan_out ... ok
test tests::wave13b_test_event_bus_publish_then_subscribe_in_order ... ok
test tests::test_free_plan_stub_returns_default ... ok
test tests::test_identity_client_constructible ... ok
test tests::wave13b_test_emit_with_zero_subscribers ... ok
test tests::wave13b_test_sse_round_trip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/main.rs

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo build -p epsx-identity-service --release`
```
Finished `release` profile [optimized] target(s) in 37.45s
```
33.3 MB release binary (matches wave-13a).

### `docker build -f shared/rust/epsx-identity-service/Dockerfile -t epsx-identity:wave13b-dev .`
```
View build details: docker-desktop://dashboard/build/default/default/...
```
Image built, ID `f5f9f2a59241`:
```
IMAGE                          ID             DISK USAGE   CONTENT SIZE   EXTRA
epsx-identity:wave13a-dev      5f6d9159152c        155MB         33.1MB   U
epsx-identity:wave13a-verify   fbbbc834eac8        155MB         33.1MB   U
epsx-identity:wave13b-dev      f5f9f2a59241        156MB         33.3MB
```

### `kubectl kustomize infrastructure/kubernetes/overlays/dev`
Renders the identity Service with `nodePort: 30105` +
`image: epsx-identity:wave13b-dev` (full 11-resource
output: 5 services + 5 deployments + 1 namespace).

### `kubectl apply -k infrastructure/kubernetes/overlays/dev`
```
namespace/epsx-dev unchanged
service/epsx-admin unchanged
service/epsx-analytics unchanged
service/epsx-backend unchanged
service/epsx-frontend unchanged
service/epsx-identity configured
deployment.apps/epsx-admin configured
deployment.apps/epsx-analytics configured
deployment.apps/epsx-backend configured
deployment.apps/epsx-frontend configured
deployment.apps/epsx-identity configured
```

### `kubectl -n epsx-dev rollout status deployment/epsx-identity`
```
Waiting for deployment "epsx-identity" rollout to finish: 1 old replicas are pending termination...
Waiting for deployment "epsx-identity" rollout to finish: 1 old replicas are pending termination...
deployment "epsx-identity" successfully rolled out
```

### Pod state
```
NAME                             READY   STATUS        RESTARTS   AGE
epsx-identity-859cf67c49-gllrh   1/1     Terminating   0          66m
epsx-identity-9f7989ffd-jhfrg    1/1     Running       0          16s
```

New pod `epsx-identity-9f7989ffd-jhfrg` Running with
image `f5f9f2a59241` (the wave-13b build).

### Pod logs (confirms both ports bound)
```
INFO  ============================================================
INFO    epsx-identity-service v0.1.0
INFO    Wave 13a — Track A (gRPC, tonic) +
INFO    Wave 13b — Track A (SSE + admin emit, axum)
INFO    gRPC:        0.0.0.0:50051
INFO    HTTP/1.1 SSE:0.0.0.0:50052
INFO    Event bus:   broadcast channel, capacity = 1024
INFO    gRPC methods (1):
INFO      rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)
INFO          returns GetWalletRankingOffsetResponse
INFO    HTTP/1.1 routes (2):
INFO      GET  /v1/stream/ranking-offsets  (SSE)
INFO      POST /v1/emit                    (JSON, admin)
INFO  ============================================================
INFO  epsx-identity-service: tonic gRPC server listening addr=0.0.0.0:50051
INFO  epsx-identity-service: axum HTTP/1.1 server listening addr=0.0.0.0:50052 routes="/v1/stream/ranking-offsets (GET, SSE), /v1/emit (POST, JSON)"
```

### Round-trip smoke test (the canary)
```
$ curl -N -m 5 -H 'Accept: text/event-stream' http://127.0.0.1:30105/v1/stream/ranking-offsets &
[1] 78642
$ curl -s -X POST -H 'Content-Type: application/json' -d '{"wallet":"0x1234","offset":100}' http://127.0.0.1:30105/v1/emit
{"delivered_to":1}
data: {"wallet":"0x1234","offset":100,"changed_at_ms":1781423064449}
```

`delivered_to: 1` confirms one SSE subscriber
received the event, and the `data:` line on the SSE
stream confirms the round-trip end-to-end
(emit → bus.publish → bus.subscribe → SSE handler →
`Event::default().data(json)` → reqwest bytes_stream →
JSON parse). Latency: ~10ms.

## Changed files (worktree-relative)

### New files
- `shared/rust/epsx-identity-service/src/event_bus.rs` —
  `RankingOffsetEventBus` (1024-slot
  `tokio::sync::broadcast::Sender`).
- `shared/rust/epsx-identity-service/src/sse_handler.rs` —
  `stream_ranking_offsets` axum handler (the SSE
  endpoint) + `RankingOffsetChangeDto` (the JSON
  envelope).
- `shared/rust/epsx-identity-service/src/emit_handler.rs` —
  `emit_ranking_offset` axum handler (the admin hook) +
  `EmitRequest` + `EmitResponse` DTOs.

### Modified files
- `shared/proto/identity.proto` — added
  `RankingOffsetChange` message (wire schema for the
  SSE event; future gRPC `UpdateRankingOffset` will
  reuse the same struct).
- `shared/rust/epsx-identity-service/Cargo.toml` —
  added `axum.workspace`, `tokio-stream` with
  `["sync"]` feature (local override),
  `serde.workspace`, `serde_json.workspace`,
  `tower.workspace`, `bytes = "1"` (for the
  `bytes::Bytes` SSE test stream). No
  `tonic-web` dep (see §"Deviations from spec" in
  ROADMAP §17.2).
- `shared/rust/epsx-identity-service/src/lib.rs` —
  exposed `event_bus`, `sse_handler`,
  `emit_handler` modules + 5 new tests
  (3 unit, 2 integration) in the test submodule.
- `shared/rust/epsx-identity-service/src/main.rs` —
  dual-port binding (gRPC on 50051 + axum on
  50052), new `BIND_ADDR_SSE` env var, updated
  startup banner.
- `infrastructure/kubernetes/base/identity/deployment.yaml` —
  added `containerPort: 50052`, `BIND_ADDR_SSE` env
  var, bumped `EPSX_IDENTITY_VERSION` to `wave13b`.
- `infrastructure/kubernetes/base/identity/service.yaml` —
  added the `sse` port (`50052`).
- `infrastructure/kubernetes/overlays/dev/patches/services-identity.yaml` —
  added `nodePort: 30105` for the SSE port.
- `infrastructure/kubernetes/overlays/dev/patches/services-nodeport.yaml` —
  documented the 30105 entry in the NodePort plan.
- `infrastructure/kubernetes/overlays/dev/kustomization.yaml` —
  bumped `epsx-identity` image tag to
  `:wave13b-dev`.
- `docs/wave8-service-boundary/ROADMAP.md` — appended
  the §17.2 implementation report (Track A
  sub-section).
- `Cargo.lock` — auto-bumped for the new deps.

## Deviations from spec (full list in ROADMAP §17.2)

1. **`tonic-web` was a wrong fit — replaced with plain
   `axum 0.8` on a separate port.** `tonic-web` is a
   gRPC-Web protocol translator, not an HTTP/1.1 router
   — no `resource()` / `add_routes()` API in any
   version. The right primitive for hosting arbitrary
   HTTP/1.1 routes alongside tonic is plain `axum` on a
   separate port.
2. **`tokio-stream` `sync` feature added locally** (the
   workspace default is `["net"]`; `BroadcastStream` is
   gated on `sync`).
3. **`bytes` added as a direct dep** for the SSE test
   stream's element type.
4. **`serde` added as a direct dep** (workspace-level
   serde doesn't propagate through transitive deps; the
   binary uses `serde::Serialize` directly in
   `sse_handler.rs` + `emit_handler.rs`).

## Out of scope (Track B + wave-13c+)

- Consuming the SSE stream from the analytics binary
  (Track B's deliverable).
- Reconnect logic on the consumer side (Track B).
- Exposing a stream endpoint on the analytics binary
  (Track B).
- Hooking the gRPC `GetWalletRankingOffset` path (or a
  new `UpdateRankingOffset` gRPC) into the publish path
  (wave-13c+).
- TLS / mTLS on the HTTP/1.1 path (dev cluster has no
  cert-manager; production deployment is a separate
  decision).
- Replacing the in-process admin emit hook with a
  gRPC-only `UpdateRankingOffset` RPC. The HTTP/1.1
  hook is a dev-cluster convenience for the integration
  gate; the gRPC seam is the long-term public surface.
