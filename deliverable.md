# Wave 10 — Integration Gate — Deliverable

## Summary

Merged the three wave-10 producer branches (`wave10/track-a-notification-port`,
`wave10/track-b-pubsub`, `wave10/track-c-ports`) in sequence on
`wave10/integration`, resolved the cross-track conflicts (most
importantly, the in-process `NotificationPort` adapter still referenced
the deleted `RedisNotificationBroadcaster`), wired the production
`NotificationPort` in `UnifiedRouteBuilder` (the production router
path was creating 7 `AppState` instances without ever attaching the
port), and added an end-to-end smoke test (`apps/backend/tests/wave10_smoke.rs`)
that confirms Track A's `NotificationPort` + Track B's `PubsubPort`
interface contracts line up at runtime — 3/3 tests pass.

## Branch / final commit

- **Branch:** `wave10/integration` (pushed to `origin/wave10/integration`).
- **Base:** `origin/migration/dioxus-microservices` HEAD `9f794784`.
- **Final commit hash:** `6f7f00bcc5e1e1464d454c72a4fac3315fdeb589`
  (the "wire NotificationPort in production graph" commit).
- **Worktree:** `/Users/fluke/Desktop/Work/epsx/.worktrees/wave10-integration`.

6 commits on the integration branch (final commit first):

```
6f7f00bc wave10(integration): merge 3 producer tracks + cross-track fixes + smoke test
f3bae988 wave10(integration): wire NotificationPort in production graph (UnifiedRouteBuilder)
9a2e8404 wave10(integration): merge Track C — cross-cutting ports
4da41db3 wave10(integration): cross-track fix — migrate InProcessNotificationAdapter to PubsubPort
5824f7d3 wave10(integration): merge Track B — PubsubPort (Redis + in-memory)
729cc1aa wave10(integration): merge Track A — NotificationPort (in-process)
```

## Verification

- `cargo check --workspace` → clean (4 pre-existing warnings,
  0 errors). Incremental 0.36s; full build from clean was not
  re-measured (the prior Track A + B + C builds were all clean).
- `cargo test -p epsx --lib` → **414 passed**, 0 failed, 8
  ignored. (Was 397 pre-wave; +17 across all 3 tracks' tests.)
- `cargo test -p epsx --test wave10_smoke` → **3 passed**, 0
  failed (the new integration smoke test).
- `cargo test -p epsx --tests` (all integration tests) → 3
  passed (the `auth_migration_test` is now reached but does not
  need a DB to compile; the smoke test runs cleanly).
- `cargo test -p epsx-contracts --lib` → **47 passed**, 0
  failed. (Was 37 pre-wave; +10 across the 3 tracks' DTO
  round-trip tests.)
- `cargo build --workspace --bins` → success (7 pre-existing
  warnings, 0 errors) in 1m 50s — well under the 30-min cap,
  no substitution to `cargo check` needed.

The end-to-end smoke test (3/3 pass) is the integration truth:
it confirms Track A's `NotificationPort.send/broadcast` and
Track B's `PubsubPort.publish/subscribe` line up at runtime
with the correct channel naming convention and JSON payload
format. Track C's two ports (`PermissionAuthorityPort` and
`WalletRankingOffsetQuery`) are tested at the unit level in
`epsx-contracts` (3 DTO round-trip tests) and via the existing
permission-service call sites in `validation_handlers.rs` and
`analytics/{rankings,cache}.rs`.

## Merge log

| Order | Track | Merge commit | Conflicts | Resolution |
|-------|-------|--------------|-----------|------------|
| 1 | Track A — NotificationPort | `729cc1aa` | none | clean merge |
| 2 | Track B — PubsubPort | `5824f7d3` | 6 files | see below |
| 3 | Track C — cross-cutting ports | `9a2e8404` | 1 file | see below |
| 4 | (cross-track fix) | `4da41db3` | n/a | see below |
| 5 | (DI wiring) | `f3bae988` | n/a | see below |
| 6 | (ROADMAP §12 + deliverable.md) | `6f7f00bc` | n/a | documentation closure |

**Track B conflict resolution (6 files):**

- `shared/rust/epsx-contracts/src/lib.rs` — keep both port
  modules (`notification_port` + `pubsub_port`).
- `apps/backend/src/infrastructure/adapters/mod.rs` — keep
  both adapter modules (`notification` + `pubsub`).
- `apps/backend/src/infrastructure/services/notification_service.rs` —
  Track B's diff was a pre-A rewrite that did `INSERT` +
  `redis.publish` directly. Track A's port-based shim
  supersedes that. Kept Track A's structure.
- `apps/backend/src/infrastructure/services/plan_expiration_service.rs` —
  Track B added a `pubsub.publish` block right after
  `port.send`; the port adapter already does the publish.
  Removed the redundant block. The `new()` signature keeps
  Track B's shape (matches `main.rs`); the service no longer
  stores the pubsub arg (uses `_pubsub` for call-site
  stability).
- `docs/wave8-service-boundary/ROADMAP.md` — kept both Track
  A §11 and Track B addendum (additive).
- `deliverable.md` — reset to placeholder; the integration
  gate's final deliverable overwrites it.

**Track C conflict resolution (1 file):**

- `deliverable.md` — reset to placeholder (same as above).
  The other 4 auto-merged files: `epsx-contracts/src/lib.rs`
  (3 ports now exported), `infrastructure/adapters/mod.rs`
  (3 adapter modules), `web/routes/unified_router.rs` (2 new
  helpers `get_permission_authority_port` /
  `get_wallet_ranking_offset_port`), and
  `docs/wave8-service-boundary/ROADMAP.md` (Track A §11 +
  Track B addendum + Track C §11, all additive).

**Cross-track fix (commit `4da41db3`):**

`cargo check --workspace` after Track B failed with
`unresolved import crate::web::notifications::RedisNotificationBroadcaster`
(in the in-process adapter) and `no field 'redis_broadcaster' on
type 'AppState'` (in the container factory). Track A's
in-process adapter took `Arc<RedisNotificationBroadcaster>`;
Track B deleted that struct and renamed the AppState field to
`pubsub`. The fix migrated the adapter to use the new
`Arc<dyn PubsubPort>` and updated the `publish_sse` to use the
`pubsub.publish(channel, payload)` API with the channel
convention `notifications:wallet:<addr>` /
`notifications:all`.

**DI wiring fix (commit `f3bae988`):**

The `NotificationPort` was wired in
`RequestServices::create_auth_app_state` (the async factory
path), but the production path (`main.rs` → `create_router` →
`UnifiedRouteBuilder`) created 7 `AppState` instances without
ever attaching the port. The 8 publisher call sites
silently log a warning and skip the publish when the port is
`None`. The fix added
`UnifiedRouteBuilder::with_notification_port(...)` and the
`create_app_state()` now attaches the pre-built port to
every `AppState`. `main.rs` builds the in-process adapter
(async) and passes the result to `create_router`.

## Cargo summaries

| Command | Result | Time |
|---------|--------|------|
| `cargo check --workspace` | clean (4 pre-existing warnings, 0 errors) | 0.36s |
| `cargo test -p epsx --lib` | 414 passed, 0 failed, 8 ignored | 0.16s |
| `cargo test -p epsx --test wave10_smoke` | 3 passed, 0 failed | 0.20s |
| `cargo test -p epsx-contracts --lib` | 47 passed, 0 failed | 0.00s |
| `cargo build --workspace --bins` | success (7 warnings, 0 errors) | 1m 50s |

Last lines of each log:

- `/tmp/wave10-check.log` → `Finished dev profile [unoptimized + debuginfo] target(s) in 0.36s`
- `/tmp/wave10-test.log` → `test result: ok. 414 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.16s`
- `/tmp/wave10-build.log` → `Finished dev profile [unoptimized + debuginfo] target(s) in 1m 50s`
- `/tmp/wave10-smoke.log` → `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s`

## Changed files (cumulative across the 5 integration commits)

### Additions (2)

- `apps/backend/tests/wave10_smoke.rs` — 3 end-to-end smoke
  tests covering the `NotificationPort` + `PubsubPort` seam
  (send / broadcast / cross-channel isolation).

### Edits (integration-only — 6 files)

- `shared/rust/epsx-contracts/src/lib.rs` — both port modules
  exported.
- `apps/backend/src/infrastructure/adapters/mod.rs` — both
  adapter modules registered.
- `apps/backend/src/infrastructure/services/notification_service.rs`
  — conflict resolution (kept Track A's port-based shim).
- `apps/backend/src/infrastructure/services/plan_expiration_service.rs`
  — conflict resolution (removed Track B's redundant
  `pubsub.publish` block; `new()` keeps the Track B signature).
- `apps/backend/src/infrastructure/adapters/notification/in_process_adapter.rs`
  — cross-track fix; field type and `publish_sse` migrated
  from `Arc<RedisNotificationBroadcaster>` to
  `Arc<dyn PubsubPort>`.
- `apps/backend/src/infrastructure/container/stateless_service_factory.rs`
  — `try_new(app_state.pubsub.clone())` (renamed from
  `app_state.redis_broadcaster`).
- `apps/backend/src/web/routes/unified_router.rs` — added
  `UnifiedRouteBuilder::with_notification_port(...)`; the
  `create_app_state()` attaches the pre-built port to every
  AppState.
- `apps/backend/src/web/mod.rs` — `create_router()` signature
  now takes
  `Option<Arc<dyn NotificationPort>>` and passes it to the
  builder.
- `apps/backend/src/main.rs` — builds the in-process
  `NotificationPort` (async) and passes it to
  `create_router`.
- `docs/wave8-service-boundary/ROADMAP.md` — appended §12
  integration report.
- `deliverable.md` — this file (overwrites the placeholder
  set during Track B + C conflict resolution).

## Notes for the verifier

1. **The smoke test is the integration truth.** It is
   `apps/backend/tests/wave10_smoke.rs` and is run via
   `cargo test -p epsx --test wave10_smoke`. 3/3 pass.

2. **The smoke test does not use `InProcessNotificationAdapter`
   directly** because the constructor is async and requires a
   real `NOTIFICATIONS_DATABASE_URL`. Instead it uses a
   `SmokeNotificationPort` test-double that mirrors the
   in-process adapter's `publish_sse` channel-name +
   JSON-payload contract. This is sufficient to confirm the
   runtime contract at the port + pubsub seam; the
   in-process adapter itself is exercised by the 5 unit
   tests in its own module (which all pass).

3. **The chat pubsub canary is the other half of the
   integration truth.** It is the test
   `chat_pubsub_canary_tests::chat_new_round_trip_via_pubsub_port`
   in `apps/backend/src/infrastructure/adapters/pubsub/mod.rs`
   and runs as part of `cargo test -p epsx --lib`. It
   confirms Track B's pubsub port works against the chat
   call sites (which is the most rigorous pre-existing
   subscriber).

4. **No MR was opened.** Per the task spec, the orchestrator
   opens it after the gate. The branch is `wave10/integration`
   and is pushed to `origin/wave10/integration`.

5. **No production deploy was performed.** Per CLAUDE.md
   ("Never deploy to production unless explicitly instructed").

6. **Track C's DROP migration (`notification_subscriptions`)
   was applied by the producer on a scratch DB; not applied
   on the production DB** — that is a wave-11 ops task.

7. **Open issues for wave 11** are listed in
   `docs/wave8-service-boundary/ROADMAP.md` §12f (7 items).
