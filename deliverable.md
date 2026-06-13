# Wave 11 / Track C — EventPublisherPort + orphaned events + plan-tier read-side fix

## 1. Summary

Shipped the kernel-level `EventPublisherPort` (ROADMAP §5 R7) — a
trait + in-process adapter that replaces the 88 `Arc<dyn DomainEventBus>`
direct references in the application command handler layer with a
port seam, enabling a network event bus in wave-N+2. The 3 R8
orphaned events (`PlanDeletedEvent`, `WalletAssignedToPlanEvent`,
`WalletRemovedFromPlanEvent`) now flow through the new port. The
in-process adapter is intentionally a no-op stub per ROADMAP §6
trap 8 — it logs at `tracing::info!` and (optionally) forwards to
the legacy bus via `tokio::spawn`. The plan-tier read-side fix
was already done in wave 10 (R6: `WalletRankingOffsetQuery` port);
this track adds the port-call unit test that the wave-10 report
did not include.

3 commits on `wave11/track-c-event-port` (base
`origin/migration/dioxus-microservices` HEAD `1014d8c4`):

| # | Hash | Subject |
|---|------|---------|
| 1 | `3519c4f2` | `wave11(track-c): add EventPublisherPort + InProcessEventPublisher + OwnedEvent` |
| 2 | `e5275d1b` | `wave11(track-c): migrate 19 application command handlers + container to EventPublisherPort` |
| 3 | `41f8f0be` | `wave11(track-c): append §13 implementation report to ROADMAP` |

Final commit hash: **`41f8f0be`** (HEAD of
`origin/wave11/track-c-event-port`).

## 2. Changed files

### Additions (6 files, ~1,580 LOC)

- `shared/rust/epsx-contracts/src/domain_event.rs` (139 LOC) —
  top-level `DomainEvent` / `DomainEventBus` / `EventMetadata` /
  `InMemoryEventBus` / `OwnedEvent`. Lifted from
  `epsx_contracts::traits::domain_event` per the spec.
- `shared/rust/epsx-contracts/src/event_publisher_port.rs` (79
  LOC) — the kernel-level `EventPublisherPort` trait +
  object-safety + AppError-typed return assertions.
- `apps/backend/src/infrastructure/adapters/events/mod.rs` (26
  LOC) — module entry + re-exports.
- `apps/backend/src/infrastructure/adapters/events/in_process_event_publisher.rs`
  (415 LOC) — the in-process impl + 4 round-trip tests + 4
  orphan-event tests.
- `apps/backend/src/infrastructure/adapters/events/event_publisher_migration.rs`
  (159 LOC) — comments-only migration table (file:line
  before/after for every migrated call site).
- `apps/backend/src/infrastructure/adapters/events/test_helpers.rs`
  (64 LOC) — `CapturingEventPublisher` mock for the orphan-event
  tests. `#[cfg(test)]`-gated.

### Edits (25 files)

- `shared/rust/epsx-contracts/src/lib.rs` — re-export
  `domain_event` + `event_publisher_port` at the crate root.
- `shared/rust/epsx-contracts/src/traits.rs` — drop the orphaned
  `pub mod domain_event;` declaration; replace with `pub use
  crate::domain_event::*;` for backward compat.
- `shared/rust/epsx-contracts/src/traits/aggregate_root.rs` —
  import path updated.
- `apps/backend/src/domain/shared_kernel/domain_event.rs` —
  re-export shim updated to point at the new top-level path.
- `apps/backend/src/domain/shared_kernel/mod.rs` — comments
  updated.
- `apps/backend/src/infrastructure/adapters/mod.rs` — add `pub
  mod events;` + re-export.
- `apps/backend/src/infrastructure/container/simple_container.rs` —
  add `event_publisher: Option<Arc<dyn EventPublisherPort>>` field
  + wire it at `build()`.
- 19 application command handlers (5 permission_management + 3
  subscription_management + 5 market_analytics + 5 notification +
  1 payment) — `event_bus: Arc<dyn DomainEventBus>` →
  `event_publisher: Arc<dyn EventPublisherPort>`; the publish
  call goes through the new port.
- `apps/backend/src/web/analytics/eps/rankings.rs` — add
  port-call unit test for the wave-10 R6 migration.
- `docs/wave8-service-boundary/ROADMAP.md` — append §13
  implementation report (374 LOC).

## 3. Test results

```
$ cargo check --workspace
warning: `epsx` (lib) generated 7 warnings (pre-existing; no new warnings)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (pre-existing)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.98s

$ cargo test -p epsx --lib
test result: ok. 423 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.16s

$ cargo test -p epsx-contracts --lib
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Test count delta:** 414 → 423 (epsx), 47 → 47 (epsx-contracts).
9 new tests added in this track:

- 4 `in_process_event_publisher::tests` (round-trip, bus forward,
  no-bus, dyn-dispatch)
- 4 `in_process_event_publisher::tests::orphan_event_tests` (3
  per-event + 1 combined mock-capture)
- 1 `web::analytics::eps::rankings::tests::test_calculate_ranking_config_from_permissions_uses_port`

The 8 ignored tests are pre-existing.

## 4. 88-site migration summary

| Category | Migrated | Out of scope | Total |
|----------|---------:|-------------:|------:|
| Application command handlers (19 files) | 19 | 0 | 19 |
| Container (1 file) | 1 | 0 | 1 |
| Infrastructure / prelude / shim re-exports | 0 | 7 | 7 |
| **Total** | **20** | **7** | **27 files** |

The 88 references map to 19 handler files × ~4-5 references per
file (struct field + ctor arg + ctor body + publish call +
optional import). The migration replaces every `Arc<dyn
DomainEventBus>` with `Arc<dyn EventPublisherPort>`. The 7
"out of scope" entries are not call sites — they are the
trait definition, the `SimpleEventBus` impl, the module
re-exports, and the shim comments that intentionally remain
for the wave-12 cleanup.

The full migration table is at
`apps/backend/src/infrastructure/adapters/events/event_publisher_migration.rs`.

## 5. 3 orphan events disposition (R8)

The 3 events are now published via the new `EventPublisherPort`:

- `PlanDeletedEvent` — `apps/backend/src/application/permission_management/commands/handlers/delete_plan_handler.rs:50-58`
- `WalletAssignedToPlanEvent` — `apps/backend/src/application/permission_management/commands/handlers/assign_wallet_handler.rs:87-101`
- `WalletRemovedFromPlanEvent` — `apps/backend/src/application/permission_management/commands/handlers/remove_wallet_handler.rs:50-61`

The in-process adapter is a **no-op stub** per ROADMAP §6
trap 8. Each event reaches the `tracing::info!` log line and
(optionally) the legacy bus via `tokio::spawn`. The bus
remains a no-op (per the analytics audit §4a–§4e) so any
consumer that was quietly relying on the events' absence is
not surprised.

**Tests:**
- `in_process_event_publisher::tests::orphan_event_tests::plan_deleted_event_publishes_via_in_process_publisher`
- `in_process_event_publisher::tests::orphan_event_tests::wallet_assigned_event_publishes_via_in_process_publisher`
- `in_process_event_publisher::tests::orphan_event_tests::wallet_removed_event_publishes_via_in_process_publisher`
- `in_process_event_publisher::tests::orphan_event_tests::all_three_orphan_events_captured_by_mock_publisher`
  — captures all 3 via a `CapturingEventPublisher` mock and
  asserts the event type headers match.

## 6. Plan-tier read-side fix

The wave-10 R6 migration (`WalletRankingOffsetQuery` port) was
already in place in `web/analytics/eps/{rankings,cache}.rs`.
This track **verified** the migration by adding a unit test
that mocks the port, asserts the call goes through the port
(not a concrete `UnifiedPermissionService`), and exercises the
free-plan fallback path. No `// TODO(track-c):` annotations
were added — the port call was already in place.

## 7. Deviations from the spec

1. **Port signature is `Box<dyn DomainEvent>` (owned), not
   `&dyn DomainEvent` (borrowed).** An interim edit tried
   `&dyn DomainEvent` for ergonomics, but the in-process
   adapter's `tokio::spawn` forward to the bus requires
   owned events (`Send + 'static` bounds). Reverted to
   `Box<dyn DomainEvent>`. The 4 for-loop call sites use
   the `OwnedEvent` wrapper to bridge the gap.
2. **`DomainEvent` move from `epsx_contracts::traits::domain_event`
   to `epsx_contracts::domain_event`.** The trait was already
   at the `traits::` path (wave 9 prep). This track lifts it
   to the top-level `domain_event` path and keeps
   `traits::domain_event` as a `pub use` re-export shim.
3. **The 3 orphan event tests are at the `events` adapter
   module, not in the 3 handler modules.** The handlers need
   repository port mocks to instantiate; the publish path is
   verified by the events adapter tests + the in-process
   publisher tests. The handler-level integration tests are
   deferred to wave-12.
4. **The web layer's `Extension(event_bus)` pattern was
   never present.** The 88 references are struct-field +
   ctor-arg + ctor-body + publish-call in the 19
   application command handlers, not `Extension` in the
   web layer. The migration covers the actual locations.
5. **`MockEventBus` was renamed to `MockEventPublisher` in
   `create_payment_command.rs`.** The test mock is now a
   `#[async_trait]` impl of `EventPublisherPort`, not a
   `DomainEventBus` impl.
6. **`simple_event_bus.rs` is kept on disk** (per the
   in-process publisher's `with_bus` constructor). The
   wave-12 cleanup can delete it after the publisher
   migrates to the pure-log-line `new()` shape.

## 8. Open issues for the integration gate

1. **`DomainEventBus` shim removal** — wave-12 cleanup. The
   7 "out of scope" entries (trait def, `SimpleEventBus`
   impl, re-exports, shim comments) are the cleanup
   targets.
2. **Network `EventPublisherPort` impl** — wave-N+2. The
   DI wiring change is one line in
   `stateless_service_factory::create_auth_app_state`:
   replace `InProcessEventPublisher::with_bus(event_bus)`
   with `HttpEventPublisher::new(...)`.
3. **Defensive `Option<...>` on `event_publisher`** — the
   container's field is `Option<...>` but the call sites
   take it as `Arc<...>` (non-Optional). Wave-12 should
   decide.
4. **`OwnedEvent` JSON round-trip cost** — ~10µs per
   publish, negligible at the current event rate. A wave
   that needs higher throughput can implement `Clone` on
   `DomainEvent` and replace the wrapper.
5. **`create_payment_command.rs` test mock** — the mock
   captures events but the test doesn't assert on the
   capture. Wave-12 should add the assertion.
6. **`plan_expiration_service.rs` does not use the
   publisher** — the wave-10 R3 already routed its
   notifications through the `NotificationPort`. No
   `DomainEvent` publishing happens here. Track C does
   not need to touch it.
7. **The `EventPublisherPort` DI wiring in
   `UnifiedRouteBuilder`** — the 19 application command
   handlers are not called by the web layer (per the
   wave-8 audit; the web layer has its own raw-SQL
   paths). The port is wired at the `SimpleContainer`
   level, but the web layer's `AppState` does not pass
   it through. Wave-12 should either (a) wire it
   through for consistency or (b) document the bypass.

## 9. Branch + commit info

- **Branch:** `wave11/track-c-event-port`
- **Worktree:** `/Users/fluke/Desktop/Work/epsx/.worktrees/wave11-track-c-event-port`
- **Base commit:** `1014d8c4` (wave-10 integration final)
- **Final commit:** `41f8f0be` (HEAD of
  `origin/wave11/track-c-event-port`)
- **Pushed:** `git push -u origin wave11/track-c-event-port` —
  branch is now tracked on `origin`.

The orchestrator opens the MR after the integration gate.

## 10. Verifier checklist

- [x] `cargo check --workspace` clean (pre-existing warnings only)
- [x] `cargo test -p epsx --lib` → 423 passed / 0 failed / 8 ignored
- [x] `cargo test -p epsx-contracts --lib` → 47 passed / 0 failed
- [x] 19 application command handlers migrated
- [x] `SimpleContainer` wired with `event_publisher: Option<Arc<dyn EventPublisherPort>>`
- [x] 3 R8 orphan events publish through the new port
- [x] Plan-tier read-side fix verified via port-call test
- [x] 9 new tests added (4 in-process + 4 orphan + 1 rankings)
- [x] Migration table documented in
      `apps/backend/src/infrastructure/adapters/events/event_publisher_migration.rs`
- [x] ROADMAP §13 addendum appended
- [x] Branch pushed to `origin/wave11/track-c-event-port`
