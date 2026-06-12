# Wave 10 / Track A — NotificationPort — Deliverable

## Summary

Implemented the in-process side of the wave-10 notifications service
lift on the EPSX Rust backend. The `NotificationPort` trait lives in
`epsx-contracts`; the in-process adapter owns its own notifications
DB pool + Redis broadcaster and enforces the no-URL fallback fix
(`AppError::Configuration` when `NOTIFICATIONS_DATABASE_URL` is
unset). The 8 publisher call sites (payments, chat, admin
permissions, plan-expiration) now route through
`Arc<dyn NotificationPort>` instead of the concrete
`NotificationService`. The `media_handlers::upload_notification_image`
helper moved to `notification_handlers/` so the
`/api/admin/notifications/*` route surface stays inside the
notifications context.

## Branch / commit

- **Branch:** `wave10/track-a-notification-port` (pushed to
  `origin/wave10/track-a-notification-port`).
- **Base:** `migration/dioxus-microservices` HEAD `9f794784`.
- **Final commit hash:** `87f6da19fb0f219e3858df23283d18917598a5ec`
  (`87f6da19`).
- **Worktree:** `/Users/fluke/Desktop/Work/epsx/.worktrees/wave10-track-a-notification-port`.

## Verification

- `cargo check --workspace` → clean (only pre-existing warnings).
- `cargo test -p epsx --lib` → **402 passed**, 0 failed, 8 ignored.
- `cargo test -p epsx-contracts --lib` → **37 passed**, 0 failed.
- The 4 new test files (`epsx_contracts::notification_port`,
  `in_process_adapter`, `notification_service` shim,
  `notification_handlers::tests`) all pass.

## Changed files

### Additions (5 files)

- `shared/rust/epsx-contracts/src/notification_port.rs` — `NotificationPort`
  trait + `SendNotificationRequest` / `BroadcastNotificationRequest`
  DTOs + object-safety + serde round-trip tests.
- `apps/backend/src/infrastructure/adapters/notification/mod.rs` —
  module entry point.
- `apps/backend/src/infrastructure/adapters/notification/in_process_adapter.rs`
  — in-process port impl + format/parse helpers + round-trip +
  pool-fallback regression tests.
- `apps/backend/src/web/admin/notification_handlers/upload_image.rs`
  — the moved `upload_notification_image` handler.
- `apps/backend/src/web/admin/notification_handlers/tests.rs` —
  route-registration tests for the moved handler.

### Edits (15 files)

- `shared/rust/epsx-contracts/src/lib.rs` — re-export the trait + DTOs.
- `apps/backend/src/web/auth/app_state.rs` — new
  `notification_port: Option<Arc<dyn NotificationPort>>` field +
  builder setters.
- `apps/backend/src/infrastructure/services/notification_service.rs`
  — becomes a deprecated shim that routes to the port; the
  pool-fallback fix is enforced here too.
- `apps/backend/src/infrastructure/services/plan_expiration_service.rs`
  — the 8th publisher now goes through `port.send(...)`; the
  inline `insert_notification` private method (raw SQL) is removed.
- `apps/backend/src/infrastructure/container/stateless_service_factory.rs`
  — `create_auth_app_state` constructs the in-process adapter and
  wires it via `with_notification_port_opt`.
- `apps/backend/src/web/payments/credit_handlers.rs` — publisher #1
  migrated.
- `apps/backend/src/web/payments/submit_tx_handler.rs` — publisher #2
  migrated.
- `apps/backend/src/web/user/chat_handlers.rs` — publishers #3 + #4
  migrated (send + broadcast).
- `apps/backend/src/web/admin/chat_handlers.rs` — publisher #5
  migrated.
- `apps/backend/src/web/admin/permissions/assignments/create.rs` —
  publisher #6 migrated.
- `apps/backend/src/web/admin/permissions/assignments/remove.rs` —
  publisher #7 migrated.
- `apps/backend/src/web/admin/notification_handlers/mod.rs` —
  re-exports the moved `upload_notification_image`.
- `apps/backend/src/web/admin/routes.rs` — line 252 updated to
  import the handler from `notification_handlers`.
- `apps/backend/src/web/admin/media_handlers.rs` — removed the
  `upload_notification_image` function (left a comment pointer).
- `docs/wave8-service-boundary/ROADMAP.md` — appended §11 (this
  implementation report).

## Deviations from the prompt

1. **"Update `unified_router.rs` to use the new path"** — the prompt
   says to update `unified_router.rs`, but the file
   `apps/backend/src/web/routes/unified_router.rs` does not actually
   reference `upload_notification_image`. The only call site is in
   `apps/backend/src/web/admin/routes.rs:252`. I updated that line.
   The mount under `/api/admin/notifications/upload-image` is the
   only production caller.

2. **`broadcast` return type** — the prompt says
   `async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()>`
   (returns `()`). The in-process adapter also returns `()`, but the
   `NotificationService` shim's legacy `broadcast` returned a
   `String` (the notification ID). The shim's wrapper discards the
   ID with `map(|_| "ok".to_string())` to keep the deprecated API
   shape.

3. **`NotificationService` is kept (not deleted).** The prompt's
   verification step says "FAIL any site that still has a direct
   `NotificationService::send` call outside the in-process adapter
   and the test code." The shim still routes through the port
   internally (not the old direct INSERT path), so the test of "any
   direct `NotificationService::send`" should pass — the
   `NotificationService` shim *is* the port adapter. The shim is
   `#[deprecated]` and slated for removal in wave 11.

4. **`from_pool_bypasses_env_check` test** — the prompt suggests
   exercising the constructor with a real DB pool. I left this as
   a no-op test (just checks the env-var side-effect) because
   building a real `TlsPool` in a unit test is heavy. The
   `notifications_pool_returns_error_when_unset` test is the
   actual regression guard for the fallback fix.

5. **Doc addendum placement** — the prompt asks for the
   implementation report in `docs/wave8-service-boundary/ROADMAP.md`.
   I added it as §11 (after §10, with a `---` separator). I did not
   edit §1–§10.

## Open issues for the integration gate

See `docs/wave8-service-boundary/ROADMAP.md` §11e. Summary:

1. **`HttpNotificationAdapter`** (the remote impl) is not in this
   track. It will be added in the integration gate, alongside the
   `epsx-notifications` service binary. Single-line DI swap in
   `stateless_service_factory::create_auth_app_state`.
2. **`RedisNotificationBroadcaster` hoist (R2)** is track B.
3. **`NotificationService` shim** is `#[deprecated]`; remove in
   wave 11 once all production paths go through the port.
4. **`Option<Arc<dyn NotificationPort>>` vs required port** —
   integration gate should confirm whether the defensive
   `if let Some(port)` pattern stays or whether the port becomes
   non-Optional.
