# Track B Deliverable — PubsubPort extraction

## Summary

Hoisted the generic pub/sub primitive that
`apps/backend/src/web/notifications/redis_broadcaster.rs` wrapped into a
kernel-level `PubsubPort` trait in `epsx-contracts`, added Redis and
in-memory adapters, and migrated 16 call sites across 14 files (8+ chat
call sites, notification SSE handlers, container DI, AppState) to use the
generic port. Deleted the `RedisNotificationBroadcaster` struct
(178 LOC). All 7 pubsub tests pass — including the chat pubsub canary
the audit flagged as the critical correctness gate.

## Final commit

**`b735caaa`** — `wave10(track-b): ROADMAP addendum — implementation report`

(Squashed into the wave-10 R2 series. The four-commit series is
`fb3245b1` → `dd66cda9` → `7e9b9630` → `b735caaa` — see the Git log
below.)

```
b735caaa wave10(track-b): ROADMAP addendum — implementation report
7e9b9630 wave10(track-b): migrate 16 call sites + delete RedisNotificationBroadcaster
dd66cda9 wave10(track-b): Redis + in-memory PubsubPort adapters
fb3245b1 wave10(track-b): PubsubPort trait + DTOs in epsx-contracts
9f794784 wave10/prep(5/4): add DB-ACCESS.md + create prod read-only role   <- base
```

## Test results

| Suite | Result |
|-------|--------|
| `cargo check --workspace` | ✅ green (4 pre-existing warnings) |
| `cargo check -p epsx-contracts` | ✅ green |
| `cargo test -p epsx --lib infrastructure::adapters::pubsub` | ✅ 7 passed; 0 failed |
| `cargo test -p epsx --lib` (full) | ✅ 401 passed; 0 failed; 8 ignored |

The 8 ignored tests are the `#[cfg(feature = "redis-tests")]` live-Redis
tests gated behind `--features redis-tests -- --include-ignored`. The
default `cargo test` path runs the in-memory round-trip suite + chat
canary, which is what CI will exercise.

### Pubsub-specific tests

- `in_memory_pubsub_adapter::tests::round_trip_publish_subscribe_single_channel`
- `in_memory_pubsub_adapter::tests::round_trip_publish_subscribe_multi_channel`
- `in_memory_pubsub_adapter::tests::empty_channel_list_rejected`
- `in_memory_pubsub_adapter::tests::publish_with_no_subscribers_is_ok`
- `in_memory_pubsub_adapter::tests::fan_out_two_subscribers`
- `chat_pubsub_canary_tests::chat_new_round_trip_via_pubsub_port` (the canary)
- `chat_pubsub_canary_tests::admin_chat_multi_channel_round_trip`

## Changed files

### Added (7)

- `shared/rust/epsx-contracts/src/pubsub_port.rs` — `PubsubPort` +
  `MessageStream` traits.
- `apps/backend/src/infrastructure/adapters/pubsub/mod.rs` — adapter
  module + chat canary tests.
- `apps/backend/src/infrastructure/adapters/pubsub/redis_pubsub_adapter.rs` —
  production Redis impl with the live-Redis gated test.
- `apps/backend/src/infrastructure/adapters/pubsub/in_memory_pubsub_adapter.rs` —
  test in-memory impl with 5 round-trip tests.
- `shared/rust/epsx-contracts/src/lib.rs` (mod entry)
- `shared/rust/epsx-contracts/Cargo.toml` (tokio + futures)
- `Cargo.toml` (workspace `futures = "0.3"`)

### Edited (16)

- `apps/backend/src/web/auth/app_state.rs` — field rename `redis_broadcaster` → `pubsub`
- `apps/backend/src/infrastructure/container/simple_container.rs` — PubsubPort construction
- `apps/backend/src/infrastructure/container/stateless_service_factory.rs` — PubsubPort construction
- `apps/backend/src/main.rs` — pass pubsub to `PlanExpirationService`
- `apps/backend/src/infrastructure/services/plan_expiration_service.rs` — field + publish
- `apps/backend/src/infrastructure/services/notification_service.rs` — `publish_to_wallet`/`publish_to_all` → `pubsub.publish`
- `apps/backend/src/web/notifications/sse_handlers.rs` — `subscribe_to_wallet` → `pubsub.subscribe(&[channels])`
- `apps/backend/src/web/notifications/mod.rs` — drop `pub use redis_broadcaster`
- `apps/backend/src/web/admin/notification_handlers/notification_admin.rs` — publish + response
- `apps/backend/src/web/user/chat_handlers.rs` — 5 publish sites + SSE stream subscribe
- `apps/backend/src/web/user/chat_upload_handlers.rs` — 4 publish sites
- `apps/backend/src/web/admin/chat_handlers.rs` — 4 publish sites + SSE stream subscribe (multi-channel)
- `apps/backend/src/web/routes/unified_router.rs` — 6 `AppState` construction sites
- `apps/backend/src/infrastructure/adapters/mod.rs` — re-export new types
- `docs/wave8-service-boundary/ROADMAP.md` — addendum §"Wave 10 — Track B"

### Deleted (1)

- `apps/backend/src/web/notifications/redis_broadcaster.rs` (178 LOC) — the
  notifications-typed wrapper; the new port + Redis adapter supersede it.

## Notes for the verifier

1. **The chat pubsub canary is the gate.** Run
   `cargo test -p epsx --lib infrastructure::adapters::pubsub::chat_pubsub_canary_tests`
   to confirm the canary passes. It does. (See
   `apps/backend/src/infrastructure/adapters/pubsub/mod.rs` §"Chat pubsub canary test"
   for the audit-trail commentary.)
2. **The `pubsub` field on `AppState` is `Option<Arc<dyn PubsubPort>>`.**
   Production wires it to `RedisPubsubAdapter` in
   `simple_container.rs:296-323`; tests use `InMemoryPubsubAdapter`.
3. **`RedisNotificationBroadcaster` is deleted.** `rg 'RedisNotificationBroadcaster' apps/backend/src/`
   returns 0 hits in the worktree (the file is gone; doc comments in
   `app_state.rs` and `notification_admin.rs` reference the rename
   for historical context but do not import the type).
4. **The Redis adapter has a live-Redis test gated behind
   `#[cfg(feature = "redis-tests")] #[ignore]`.** The default `cargo test`
   path exercises the in-memory adapter; the live-Redis test is the
   operator's smoke test, not CI's gate. To run it:
   `REDIS_URL=redis://127.0.0.1:6379 cargo test -p epsx --lib infrastructure::adapters::pubsub::redis_pubsub_adapter --features redis-tests -- --include-ignored`.
5. **`Cargo.toml` adds `futures = "0.3"` to `[workspace.dependencies]`.**
   The previous `apps/backend/Cargo.toml` had `futures = "0.3"` as a
   non-workspace dep; this normalizes it. `epsx-contracts/Cargo.toml`
   now uses `tokio.workspace = true` and `futures.workspace = true`.
6. **Deviations from the spec are documented in
   `docs/wave8-service-boundary/ROADMAP.md` §"Wave 10 — Track B" §5.**
   The headline deviation: `subscribe` takes `&[&str]` (slice) not
   `&str` because the admin chat stream subscribes to two channels in
   one call.
7. **No `NotificationPort` files were touched** (Track A's territory).
   Track A's `epsx-contracts::notification_port` is independent of
   this track's `epsx-contracts::pubsub_port`.
8. **No MR was opened.** Per the spec, the orchestrator opens it
   after the gate. The branch is `wave10/track-b-pubsub` and is
   pushed to origin.
