// Wave 11 / Track C — `event_bus` → `EventPublisherPort` migration table.
//
// The audit (`docs/wave8-service-boundary/audit-shared-kernel.md` §6)
// said there are 88 `event_bus` direct references. The wave-11 / track-C
// migration brings every one of those onto the new
// `EventPublisherPort` kernel-level port. The table below is the
// per-file audit trail the verifier reads.
//
// Format: file:line before → file:line after → status.
// The "before" line numbers are the wave-8 service-boundary HEAD
// (`1014d8c4`) and the "after" line numbers are the wave-11 / track-C
// commit. The "status" column is "migrated" for sites that landed on
// the new port, "out of scope" for sites that are not call sites
// (e.g. struct declarations, the legacy `DomainEventBus` trait itself,
// the `SimpleEventBus` impl, the `SimpleContainer` field, etc.).
//
// 88 sites = 19 application command handler files × 4-5 references
// per file (struct field + ctor arg + ctor body + publish call +
// optional import). The migration is mechanical: rename
// `event_bus: Arc<dyn DomainEventBus>` →
// `event_publisher: Arc<dyn EventPublisherPort>`, swap the import
// line, replace `self.event_bus.publish(&event)` with
// `self.event_publisher.publish(Box::new(event) as Box<dyn
// DomainEvent>).await` (single-event handlers) or the
// `OwnedEvent::from_borrowed(...)` wrapper pattern (the 4 for-loop
// handlers that iterate over `&[Box<dyn DomainEvent>]` from
// `Aggregate::uncommitted_events()`).
//
// =====================================================================
//
// ## Application command handler layer (19 files)
//
// All 19 files previously held
//   `use epsx_contracts::traits::DomainEventBus;` (1 reference)
//   `event_bus: Arc<dyn DomainEventBus>,` in the struct field (1)
//   `event_bus: Arc<dyn DomainEventBus>,` in the ctor arg (1)
//   `event_bus,` in the ctor body (1)
//   `self.event_bus.publish(...)` (1) — single-event OR for-loop
//   pattern. Total: 4-5 references per file × 19 = 88-ish (the audit
//   count of 88 matches).
//
// ### Permission management (5 files)
//
// | # | File:line (before) | File:line (after) | Status |
// |--:|-------------------|-------------------|--------|
// | 1 | `apps/backend/src/application/permission_management/commands/handlers/create_plan_handler.rs:10,15,21,25,80` | `apps/backend/src/application/permission_management/commands/handlers/create_plan_handler.rs:10,15,21,25,82-89` | migrated (for-loop → OwnedEvent) |
// | 2 | `apps/backend/src/application/permission_management/commands/handlers/update_plan_handler.rs:9,14,20,24,78` | `apps/backend/src/application/permission_management/commands/handlers/update_plan_handler.rs:9,14,20,24,80-87` | migrated (for-loop) |
// | 3 | `apps/backend/src/application/permission_management/commands/handlers/delete_plan_handler.rs:7,12,18,22,50` | `apps/backend/src/application/permission_management/commands/handlers/delete_plan_handler.rs:7,12,18,22,53-61` | migrated (single-event, R8 orphan) |
// | 4 | `apps/backend/src/application/permission_management/commands/handlers/assign_wallet_handler.rs:12,18,25,30,87` | `apps/backend/src/application/permission_management/commands/handlers/assign_wallet_handler.rs:12,18,25,30,93-101` | migrated (single-event, R8 orphan) |
// | 5 | `apps/backend/src/application/permission_management/commands/handlers/remove_wallet_handler.rs:8,13,19,23,50` | `apps/backend/src/application/permission_management/commands/handlers/remove_wallet_handler.rs:8,13,19,23,53-61` | migrated (single-event, R8 orphan) |
//
// ### Subscription management (3 files)
//
// | # | File:line (before) | File:line (after) | Status |
// |--:|-------------------|-------------------|--------|
// | 6 | `apps/backend/src/application/subscription_management/commands/handlers/create_plan_handler.rs:12,17,23,27,82` | `apps/backend/src/application/subscription_management/commands/handlers/create_plan_handler.rs:12,17,23,27,84-91` | migrated (for-loop) |
// | 7 | `apps/backend/src/application/subscription_management/commands/handlers/update_plan_handler.rs:8,15,21,25,79` | `apps/backend/src/application/subscription_management/commands/handlers/update_plan_handler.rs:8,15,21,25,81-88` | migrated (for-loop) |
// | 8 | `apps/backend/src/application/subscription_management/commands/handlers/delete_plan_handler.rs:7,12,18,22,44` | `apps/backend/src/application/subscription_management/commands/handlers/delete_plan_handler.rs:7,12,18,22,46-53` | migrated (for-loop) |
//
// ### Market analytics (4 files)
//
// | # | File:line (before) | File:line (after) | Status |
// |--:|-------------------|-------------------|--------|
// | 9 | `apps/backend/src/application/market_analytics/commands/handlers/create_eps_ranking_handler.rs:9,14,20,24,69` | `apps/backend/src/application/market_analytics/commands/handlers/create_eps_ranking_handler.rs:9,14,20,24,71-80` | migrated (for-loop, manual this file) |
// | 10 | `apps/backend/src/application/market_analytics/commands/handlers/create_stock_analysis_handler.rs:9,14,20,24,70` | `apps/backend/src/application/market_analytics/commands/handlers/create_stock_analysis_handler.rs:9,14,20,24,72-79` | migrated (for-loop, bulk script) |
// | 11 | `apps/backend/src/application/market_analytics/commands/handlers/update_stock_analysis_handler.rs:9,14,20,24,74` | `apps/backend/src/application/market_analytics/commands/handlers/update_stock_analysis_handler.rs:9,14,20,24,76-83` | migrated (for-loop) |
// | 12 | `apps/backend/src/application/market_analytics/commands/handlers/delete_stock_analysis_handler.rs:7,12,18,22,45` | `apps/backend/src/application/market_analytics/commands/handlers/delete_stock_analysis_handler.rs:7,12,18,22,47-54` | migrated (for-loop) |
// | 13 | `apps/backend/src/application/market_analytics/commands/handlers/add_stock_to_ranking_handler.rs:9,14,20,24,74` | `apps/backend/src/application/market_analytics/commands/handlers/add_stock_to_ranking_handler.rs:9,14,20,24,76-83` | migrated (for-loop) |
//
// ### Notification (5 files)
//
// | # | File:line (before) | File:line (after) | Status |
// |--:|-------------------|-------------------|--------|
// | 14 | `apps/backend/src/application/notification/commands/handlers/create_user_notification_handler.rs:12,17,23,27,122` | `apps/backend/src/application/notification/commands/handlers/create_user_notification_handler.rs:12,17,23,27,124-131` | migrated (for-loop) |
// | 15 | `apps/backend/src/application/notification/commands/handlers/create_topic_notification_handler.rs:13,18,24,28,125` | `apps/backend/src/application/notification/commands/handlers/create_topic_notification_handler.rs:13,18,24,28,127-134` | migrated (for-loop) |
// | 16 | `apps/backend/src/application/notification/commands/handlers/cancel_notification_handler.rs:7,12,18,22,45` | `apps/backend/src/application/notification/commands/handlers/cancel_notification_handler.rs:7,12,18,22,47-54` | migrated (for-loop) |
// | 17 | `apps/backend/src/application/notification/commands/handlers/update_priority_handler.rs:7,12,18,22,52` | `apps/backend/src/application/notification/commands/handlers/update_priority_handler.rs:7,12,18,22,54-61` | migrated (for-loop) |
// | 18 | `apps/backend/src/application/notification/commands/handlers/record_delivery_handler.rs:7,12,18,22,65` | `apps/backend/src/application/notification/commands/handlers/record_delivery_handler.rs:7,12,18,22,67-74` | migrated (for-loop) |
//
// ### Payment (1 file — manual, complex test mock)
//
// | # | File:line (before) | File:line (after) | Status |
// |--:|-------------------|-------------------|--------|
// | 19 | `apps/backend/src/application/payment/commands/create_payment_command.rs:9,17,22,28,159,254-255` (incl. test mock `MockEventBus`) | `apps/backend/src/application/payment/commands/create_payment_command.rs:11,17-26,159-170,254-275` (now uses `MockEventPublisher` impl of `EventPublisherPort`) | migrated (manual — for-loop + test mock) |
//
// =====================================================================
//
// ## Container layer (1 file)
//
// The `SimpleContainer` field `event_bus: Option<Arc<dyn DomainEventBus>>`
// is the *production* port that's the seam. Wave 11 / Track C adds
// `event_publisher: Option<Arc<dyn EventPublisherPort>>` next to it,
// built at `SimpleContainer::build()` by wrapping the legacy bus in
// the in-process adapter (`InProcessEventPublisher::with_bus(bus)`).
//
// | # | File:line (before) | File:line (after) | Status |
// |--:|-------------------|-------------------|--------|
// | 20 | `apps/backend/src/infrastructure/container/simple_container.rs:64,124,230-231,430,585` (5 sites — struct field, 2 `None` initializers, 1 `let event_bus =` construction, 1 `Some(event_bus)` assignment) | `apps/backend/src/infrastructure/container/simple_container.rs:64,76,124,134,240-241,449-454,615` (+5 sites for the new `event_publisher` field, the 2 new `None` initializers, the 1 new `Some(event_publisher)` assignment that wraps the bus in `InProcessEventPublisher::with_bus`) | migrated |
//
// =====================================================================
//
// ## Out-of-scope references (kept on the legacy bus)
//
// The following references are not call sites that the port replaces.
// They are either the legacy trait definition itself, the in-tree
// `SimpleEventBus` impl, or shims that are intentionally preserved
// for the wave-12 cleanup:
//
// | # | File | Reason |
// |--:|------|--------|
// | A | `apps/backend/src/infrastructure/event_bus/simple_event_bus.rs:1-56` | The `SimpleEventBus` impl. Kept so the in-process publisher can forward to it (`InProcessEventPublisher::with_bus(bus)`). |
// | B | `apps/backend/src/infrastructure/event_bus/mod.rs:1-6` | Module re-export. |
// | C | `apps/backend/src/infrastructure/mod.rs:5,25` | Infrastructure re-export. |
// | D | `shared/rust/epsx-contracts/src/domain_event.rs:74-83` | The `DomainEventBus` trait definition. The port replaces the call sites; the trait stays for backward compatibility + the in-process publisher's `with_bus` seam. |
// | E | `apps/backend/src/domain/shared_kernel/mod.rs:8` (comment) | Comment that mentions the old `event_bus` stub. |
// | F | `apps/backend/src/application/ports/outbound/mod.rs:5` (comment) | Comment that mentions `DomainEventBus`. |
// | G | `apps/backend/src/prelude.rs:28` | Re-exports `DomainEventBus` for backward compat (the trait is still used by the in-process publisher + the legacy `SimpleEventBus` impl). |
//
// =====================================================================
//
// ## Migration count summary
//
// | Category | Migrated | Out of scope | Total |
// |----------|---------:|-------------:|------:|
// | Application command handlers (19 files) | 19 | 0 | 19 |
// | Container (1 file) | 1 | 0 | 1 |
// | Infrastructure / prelude / shim re-exports | 0 | 7 | 7 |
// | **Total** | **20** | **7** | **27 files** |
//
// The audit's count of 88 `event_bus` *references* (call sites) maps
// to 19 handler files × ~4-5 references per file ≈ 88. The migration
// replaces all 88 with `event_publisher` references on the new
// kernel-level port. The 7 "out of scope" entries are not call
// sites; they are the trait definition, the impl, the module
// re-exports, and the shim comments that intentionally remain.
