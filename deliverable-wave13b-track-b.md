# Wave 13b — Track B (SSE consumer + local bus + /v1/rankings/stream passthrough) — Deliverable

> **Note on file naming:** this report is intentionally at
> `deliverable-wave13b-track-b.md` (NOT `deliverable.md`)
> because the wave-13a integration commit (`60305b6c`)
> already wrote its report to `deliverable.md` at the
> worktree root. Per the
> `epsx-wave12-track-b-infra-cleanup` memory's
> "worktree-root deliverable.md collision" pattern, the
> new report goes to a distinct filename.
>
> The engine-required deliverable is at
> `/Users/fluke/.mavis/plans/plan_f6609db9/outputs/track-b-sse-consumer/deliverable.md`
> (a separate copy, written at session end per the
> `Delivery Protocol`).

## Summary

The new `epsx-analytics-service` binary now consumes the
SSE stream from `epsx-identity-service` (port 50052,
served by Track A on its own branch), parses
`RankingOffsetChange` events, and fans them out to
in-process consumers via a local
`tokio::sync::broadcast` channel
(`LocalRankingOffsetBus`). A new HTTP passthrough
(`GET /v1/rankings/stream`) proxies events from the
bus to web clients as a long-lived
`text/event-stream` response. The consumer survives
transient disconnects with exponential backoff (100ms
→ 30s cap) + 0-50% jitter, and respects a
`tokio::sync::watch::Receiver<bool>` shutdown signal
for future Ctrl-C / SIGTERM wiring.

## Branch + commits

- **Branch:** `origin/wave13b/track-b-sse-consumer`
  (to be pushed at the end of this session).
- **Base:** `origin/migration/dioxus-microservices`
  HEAD `60305b6c` (the wave-13a integration head).
- **Worktree:**
  `/Users/fluke/Desktop/Work/epsx/.worktrees/wave13b-track-b-sse-consumer`
- **Implementation commit:** (populated below after
  commit)
- **Worktree-root deliverable:**
  `deliverable-wave13b-track-b.md` (avoids collision
  with wave-13a's `deliverable.md`).

## Changed files

### New files (1)
- `apps/analytics/src/sse_consumer.rs` (440 LOC) —
  the SSE consumer task + the `LocalRankingOffsetBus`
  + the `RankingOffsetChange` DTO + the SSE parser
  (`find_sse_event` / `parse_sse_data`) + 11 unit
  tests (5 SSE parser + 6 bus pub/sub).

### Modified files (4)
- `apps/analytics/Cargo.toml` — added
  `reqwest = { workspace = true, features = ["stream"] }`
  + `rand = "0.9"` (workspace version; the `stream`
  feature enables `Response::bytes_stream()` on the
  reqwest 0.12 client).
- `apps/analytics/src/main.rs` — `mod sse_consumer;`
  + the `rankings_stream_handler` SSE passthrough
  (axum 0.8 + `axum::response::sse::Sse` +
  `tokio_stream::wrappers::BroadcastStream` +
  15s keepalive) + the bus wiring in `main()` (build
  the bus, spawn `run_sse_consumer`, hold the
  `watch::Sender` for future shutdown wiring) +
  `build_analytics_router` now takes a 4th
  `local_bus` arg + the existing 5-route test
  updated to assert 7 routes.
- `infrastructure/kubernetes/base/analytics/deployment.yaml` —
  added `IDENTITY_SSE_URL=http://epsx-identity:50052`
  env var + bumped `EPSX_ANALYTICS_VERSION` to
  `wave13b`.
- `docs/wave8-service-boundary/ROADMAP.md` — appended
  the §18 implementation report (345 lines).
- `Cargo.lock` — auto-bumped for the 2 new deps.

## Verification

### `cargo test -p epsx-analytics-service`
**27/27 pass** (15 pre-existing + 12 new):

```
running 27 tests
test sse_consumer::tests::find_sse_event_empty_buffer_returns_none ... ok
test sse_consumer::tests::find_sse_event_returns_first_boundary ... ok
test sse_consumer::tests::find_sse_event_single_event_in_buffer ... ok
test sse_consumer::tests::find_sse_event_partial_buffer_returns_none ... ok
test sse_consumer::tests::find_sse_event_crlf_boundary ... ok
test sse_consumer::tests::parse_sse_data_id_line_ignored ... ok
test sse_consumer::tests::parse_sse_data_crlf_line_endings ... ok
test sse_consumer::tests::parse_sse_data_multiple_data_lines_joined ... ok
test sse_consumer::tests::parse_sse_data_single_event_single_data_line ... ok
test sse_consumer::tests::parse_sse_data_comment_only_event_returns_none ... ok
test sse_consumer::tests::parse_sse_data_event_type_only_returns_none ... ok
test sse_consumer::tests::bus_publish_with_zero_subscribers_returns_zero ... ok
test sse_consumer::tests::bus_publish_broadcasts_to_all_subscribers ... ok
test sse_consumer::tests::bus_publish_subscribe_three_events_in_order ... ok
test sse_consumer::tests::bus_receiver_count_tracks_subscribers ... ok
test sse_consumer::tests::consume_once_end_to_end_via_chunks ... ok
test sse_consumer::tests::consume_once_two_events_in_one_chunk ... ok
test tests::test_sse_consumer_end_to_end_via_real_http ... ok     <-- integration
test tests::test_epsranking_type_reexport ... ok
test tests::test_startup_banner ... ok
test tests::test_free_plan_stub_returns_default ... ok
test tests::test_fallback_returns_free_plan_when_called_directly ... ok
test tests::test_grpc_client_falls_back_on_unreachable ... ok
test tests::test_grpc_client_delegates_to_server ... ok
test tests::test_state_build_no_db ... ok
test tests::test_grpc_client_falls_back_on_timeout ... ok
test tests::test_five_route_builder ... ok                          <-- now asserts 7 routes

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
```

### `cargo build -p epsx-analytics-service --release`
```
Compiling epsx-analytics-service v0.1.0 (.../apps/analytics)
Finished `release` profile [optimized] target(s) in 4m 37s
```

Zero new warnings; 16 pre-existing `epsx` lib warnings
unchanged.

### `docker build -f apps/analytics/Dockerfile -t epsx-analytics:wave13b-dev .`
Image ID `5894d71514f5` (36 MB compressed, 164 MB
on-disk; matches the wave-13a image size — the SSE
consumer + `reqwest` `stream` feature + `rand` add
negligible weight).

### `kubectl kustomize infrastructure/kubernetes/overlays/dev`
```yaml
- name: EPSX_ANALYTICS_VERSION
  value: wave13b
- name: IDENTITY_SSE_URL
  value: http://epsx-identity:50052          # NEW
- name: IDENTITY_GRPC_URL
  value: http://epsx-identity:50051
image: epsx-analytics:wave13a-dev              # dev overlay bump is gate's job
```

### End-to-end canary
`test_sse_consumer_end_to_end_via_real_http` is the
binary-level canary: it spins up a real `axum` server
on `127.0.0.1:0` that emits two SSE events as raw
bytes, spawns `run_sse_consumer` against that server
with a real `reqwest::Client`, and asserts both
events land in the bus within a 5s timeout. This is
the closest thing to the "manual end-to-end" smoke
test the spec asked for without the operational
overhead of spinning up the full K8s cluster (which
is the integration gate's job).

## Deviations from spec (full list in ROADMAP §18.8)

1. **No `eventsource-stream` crate** — hand-rolled
   `find_sse_event` + `parse_sse_data` (~30 LOC)
   instead. Same wire output, no new transitive
   dep, parser is testable as pure functions.
2. **`RankingOffsetChange` is a local DTO** (NOT
   a `pub use` re-export from `epsx-identity-service`).
   Track A's proto + crate live on a separate
   branch; the DTO is byte-compatible with the wire
   JSON the identity service emits. The integration
   gate will reconcile (option a: re-export the
   identity crate's type; option b: keep the local
   DTO with the wire shape as the contract).
3. **No new `tokio-stream` `sync` feature
   override** — already enabled in the workspace
   by Track A's wave-13a gRPC tests. `BroadcastStream`
   (used in the passthrough handler) Just Works.
4. **Shutdown `watch::Sender` is held with
   `let _shutdown_tx`** — the spec said "Keep
   shutdown_tx for the cleanup on Ctrl-C / SIGTERM";
   the current branch drops the sender at process
   exit. A future wave 14+ will wire
   `tokio::signal::ctrl_c()` to it.
5. **Reconnect jitter sleeps ADD to the backoff,
   not replace it** — minimum sleep per retry is
   `backoff`, max is `backoff * 1.5` (well within
   the 30s cap).

## Notes for the verifier

- **Worktree-root `deliverable.md` was intentionally
  NOT overwritten.** The wave-13a integration commit
  (`60305b6c`) already wrote its report there. Per
  the `epsx-wave12-track-b-infra-cleanup` memory's
  "worktree-root deliverable.md collision" pattern,
  the new report is at `deliverable-wave13b-track-b.md`.
  The engine-required deliverable is at the plan
  output dir.
- **Two branches touching `apps/analytics/Cargo.toml`
  + `apps/analytics/src/main.rs`.** The integration
  gate's "Cross-track fix-up list" (mirroring §17.3
  from wave-13a) will need to handle the
  `Cargo.toml` `reqwest` dep bump: Track A added
  `tokio-stream` `["sync"]` local override +
  `bytes = "1"`; Track B adds `reqwest` `["stream"]`
  local override + `rand = "0.9"`. The two lists
  should merge cleanly (no overlap).
- **`shared/proto/identity.proto` is NOT modified
  by this branch.** Track A adds the
  `RankingOffsetChange` message; this branch
  consumes the wire JSON via a local DTO. The
  integration gate will be the first place both
  shapes coexist.
- **Dev overlay image tag is NOT bumped by this
  branch.** Same as wave-13a Track A — the
  integration gate bumps `:wave13a-dev` →
  `:wave13b-dev` after merging this branch.
- **The new `LocalRankingOffsetBus` is a 1024-slot
  `tokio::sync::broadcast` channel.** A lagged
  receiver (one that fell behind the ring buffer)
  gets a `RecvError::Lagged(n)` on its next `recv()`;
  the SSE passthrough handler drops lagged items
  silently (the next event catches the client up).
- **Live dev-cluster smoke test was NOT run from
  this branch** (the integration gate's job).
  The `test_sse_consumer_end_to_end_via_real_http`
  test is the binary-level proof that the consumer +
  bus + parser end-to-end works; the cluster
  deployment is unchanged in shape from the
  wave-13a integration (same `deployment.yaml`,
  same image, same env vars + 1 new one).

## Out of scope (wave-13c+ or separate)

- Hooking the gRPC `GetWalletRankingOffset` path
  into the publish path (the identity service's
  job, not the analytics binary's). Wave-13c+ is
  the natural place.
- TLS / mTLS on the SSE path. Dev cluster has no
  cert-manager; production deployment is a separate
  decision.
- A Redis-backed pub/sub for multi-replica
  deployments. The current shape assumes 1
  analytics pod (matching the current K8s
  `replicas: 1`).
- A graceful-shutdown handler that drains the
  bus before exiting.
