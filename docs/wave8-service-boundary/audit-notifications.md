# Notifications Service-Boundary Audit

**Date:** 2026-06-12
**Branch:** `wave8/audit-notifications` (worktree at `/Users/fluke/.mavis/plans/plan_f2d63ead/outputs/notifications-audit`, base `wave8/service-boundary`)
**Scope:** EPSX `notifications` bounded context, in modular monolith at `apps/backend/`
**Purpose:** Quantify the coupling that would need to be broken to lift the notifications domain out into a standalone microservice.

---

## 1. Bounded-context inventory

The notifications bounded context is split into a **pure DDD layer** (`domain/notification` + `application/notification`) and a **delivery/transport layer** (`web/notifications` + supporting infra). The "domain" is well-shaped; the "transport" is the part other modules have wedged themselves into.

### 1a. Domain layer (DDD pure, 100 % notifications-only)

| Path | Files | LOC | Notes |
|---|---|---|---|
| `apps/backend/src/domain/notification/` | 14 | 3,914 | Aggregates + value objects + events + port |
| `apps/backend/src/application/notification/` | 27 | 1,298 | CQRS commands/queries, DTOs, controller trait |

Per-file highlights (full breakdown in audit notes):

- `domain/notification/aggregates/notification.rs` — 619 LOC, the `Notification` aggregate root + `NotificationMetadata` + `DeliveryTracking` + status/priority enums. 9 states, 5 priorities.
- `domain/notification/value_objects/` — 6 modules, 2,808 LOC. The biggest: `user_preferences.rs` (713), `delivery_channel.rs` (570), `schedule_info.rs` (562), `notification_topic.rs` (522).
- `domain/notification/events/notification_events.rs` — 257 LOC. 7 events: `NotificationCreated`, `NotificationScheduled`, `NotificationSending`, `NotificationDeliveryCompleted`, `NotificationExpired`, `NotificationPriorityUpdated`, `NotificationCancelled`.
- `domain/notification/repository_ports/notification_repository_port.rs` — 55 LOC. Single trait `NotificationRepositoryPort` with `NotificationSearchCriteria`.

Application layer is the standard commands/queries split: 5 command handlers + 4 query handlers, plus 9 model structs (5 commands + 4 queries). All four query handlers are wired to `event_bus: Arc<dyn DomainEventBus>` (`shared_kernel`) — see §2.

### 1b. Transport / infrastructure layer (the "everything else" of notifications)

| Path | Files | LOC | Role |
|---|---|---|---|
| `apps/backend/src/web/notifications/` | 5 | 1,021 | SSE handler + Redis pub/sub broadcaster + offline queue |
| `apps/backend/src/web/admin/notification_handlers/` | 3 (+`mod.rs`) | 1,252 | Admin API for send/list/stats/acknowledge/overview |
| `apps/backend/src/web/admin/wallet_notification_repository.rs` | 1 | 631 | Direct Diesel-backed repo (parallel to the port) |
| `apps/backend/src/web/admin/notification_query_helper.rs` | 1 | 55 | Query helper used by admin handlers |
| `apps/backend/src/infrastructure/services/notification_service.rs` | 1 | 139 | The "send a notification from anywhere" facade |
| `apps/backend/src/infrastructure/services/plan_expiration_service.rs` | 1 | (shared) | Background cron driver that publishes expiry notifications |
| `apps/backend/src/infrastructure/repositories/notification_repository.rs` | 1 | 416 | Concrete repo (only `find_pending`) |
| `apps/backend/src/infrastructure/repositories/notification_record.rs` | 1 | 144 | DB record struct + Diesel row mapping |
| `apps/backend/src/infrastructure/adapters/repositories/notification_repository_adapter.rs` | 1 | 315 | Adapter implementing the domain port |
| `apps/backend/src/infrastructure/adapters/repositories/mappers/notification_mappers.rs` | 1 | 122 | Domain ↔ DB mapper |
| `apps/backend/src/infrastructure/adapters/services/notification_service_adapter.rs` | 1 | 63 | Application-service adapter |
| `apps/backend/src/infrastructure/models/notification.rs` | 1 | 99 | DB model (used by query handler for `find_pending`) |
| `apps/backend/src/schemas/notifications.rs` | 1 | 167 | Diesel-generated `wallet_notifications` table DSL |
| **Subtotal transport** | **~18** | **~4,425** | |

The two `wallet_notification_repository.rs` (admin web, 631 LOC) and `notification_repository.rs` (infra, 416 LOC) are *both* doing the same job — one is a direct Diesel path used by the admin HTTP handlers, the other is the port-implementing adapter. They do not share code; the `wallet_notification_repository` short-circuits the port to get admin-side filtering (status, date range, type) that the port does not expose.

### 1c. "Notifications" Migrations + Diesel config

| Path | Notes |
|---|---|
| `apps/backend/diesel_notifications.toml` | Points at `migrations/notifications/`, schema file `src/schemas/notifications.rs` |
| `apps/backend/migrations/notifications/00000000000000_diesel_initial_setup/` | Diesel boilerplate, 36 / 6 LOC |
| `apps/backend/migrations/notifications/00000000000001_consolidated_baseline_v2/` | 84 / 3 LOC, **identical to the next one** |
| `apps/backend/migrations/notifications/00000000000001_consolidated_notifications_v2/` | 84 / 3 LOC, identical |

The two `00000000000001_*` migration dirs are byte-identical (`diff -q` reports no difference). One is a `baseline` (created at a different time), the other is `consolidated_notifications_v2`. **This is dead migration history that needs deduplication before the schema can be lifted into its own DB.** Details in §4.

### 1d. Module structure summary

```
domain/notification                 14 files     3,914 LOC   (pure, no cross-domain deps)
application/notification           27 files     1,298 LOC   (CQRS, event-bus only)
web/notifications                   5 files     1,021 LOC   (SSE + Redis + offline queue)
web/admin/notification_handlers     4 files     1,307 LOC   (admin API)
web/admin/wallet_notification_repo  1 file        631 LOC   (parallel to domain port)
infrastructure/services             2 files     ~340 LOC    (NotificationService facade + plan-expiration)
infrastructure/repositories         2 files       560 LOC
infrastructure/adapters/*notif*     3 files       500 LOC
schemas/notifications               1 file        167 LOC
migrations/notifications            6 files       216 LOC   (with duplicates)
                                                          ─────
                                              ~9,950 LOC across ~60 files
```

---

## 2. Inbound dependencies (what notifications imports from other domains)

**The notifications domain itself is almost perfectly isolated — but the application/transport layer is not.**

### 2a. `domain/notification` and `application/notification` (the pure layer)

```bash
$ rg -n 'use crate::' apps/backend/src/domain/notification apps/backend/src/application/notification \
    | rg -v 'notification|shared_kernel'
(no results)
```

**Zero cross-domain imports from inside the bounded context.** All non-`notification` `use crate::` paths point at `shared_kernel` (aggregate base, domain event bus, repository port marker traits). This is the single best signal in the audit.

The only `event_bus`-related coupling is to `crate::domain::shared_kernel::DomainEventBus` — used by 3 of the 4 command handlers (`create_topic_notification_handler.rs:13`, `create_user_notification_handler.rs:13` is also there but `rg` matched only the `_topic` variant; both `create_*` handlers and `record_delivery_handler` take an `Arc<dyn DomainEventBus>`).

### 2b. `web/notifications` (the transport layer)

| File | Line | Import |
|---|---|---|
| `web/notifications/offline_queue.rs` | 5 | `use crate::web::notifications::{SSENotification, NotificationType, NotificationPriority}` (intra-module) |
| `web/notifications/sse_handlers.rs` | 11 | `use crate::{core::errors::AppError, web::auth::AppState}` (cross-cutting core) |
| `web/notifications/sse_handlers.rs` | 129,189,298 | `use crate::infrastructure::database::get_notifications_pool` |
| `web/notifications/redis_broadcaster.rs` | 4 | `use crate::infrastructure::redis::RedisPool` |
| `web/notifications/mod.rs` | (exposes `SSENotification`, `NotificationType`, `NotificationPriority`) | |

The transport layer is the *de facto* delivery surface for every other domain — but it does not depend on them.

**What `web/notifications` depends on from outside the bounded context** is `crate::core` (errors, `AppState`), `crate::infrastructure::redis`, and `crate::infrastructure::database`. These are the *three* things the microservice has to either take with it or replace with a clean port.

### 2c. `infra/services/notification_service.rs` (the cross-domain facade)

```rust
// apps/backend/src/infrastructure/services/notification_service.rs:11-14
use crate::core::errors::AppError;
use crate::web::auth::AppState;
use crate::web::admin::wallet_notification_repository::WalletNotificationRepository;
use crate::web::notifications::{NotificationType, NotificationPriority, SSENotification};
```

This file is the *primary cross-domain leak surface*. It takes `AppState` (which contains the global `db_pool`, `redis_broadcaster`, etc.) and writes directly to `wallet_notifications` via the admin-side repo. It is reachable from 6 other handlers (see §3).

### 2d. `infra/services/plan_expiration_service.rs` (cron-driven publisher)

```rust
// apps/backend/src/infrastructure/services/plan_expiration_service.rs:16-19
use crate::web::notifications::{
    SSENotification, NotificationType, NotificationPriority,
    RedisNotificationBroadcaster,
    cleanup_old_notifications,
};
```

Reaches into the notifications transport layer to (a) read `cleanup_old_notifications` and (b) build a `SSENotification` and call `RedisNotificationBroadcaster::publish_to_wallet`. This is the only background service that publishes notifications — it does not go through `NotificationService`.

**Hard vs soft inbound dependencies**

| Dep | File:line | Hard / Soft | Why |
|---|---|---|---|
| `crate::web::auth::AppState` | `infra/services/notification_service.rs:12`, `web/notifications/sse_handlers.rs:11` | **Hard** | SSE handler reads claims from `app_state.domain_container.get_token_service()`. Cannot be removed without re-plumbing auth. |
| `crate::core::errors::AppError` | (everywhere) | **Hard** | Shared error type; would be a port under a split. |
| `crate::infrastructure::redis::RedisPool` | `web/notifications/redis_broadcaster.rs:4` | **Hard** | Type of the broadcaster; cannot be a port without replacing the field on `AppState`. |
| `crate::infrastructure::database::get_notifications_pool` | `web/notifications/sse_handlers.rs:129,189,298`; `web/notifications/offline_queue.rs:5`; `infra/services/notification_service.rs:37,96`; `infra/services/plan_expiration_service.rs:79,227` | **Hard** | Functional global; would need to become a `NotificationsPool` port. |
| `crate::web::admin::wallet_notification_repository::WalletNotificationRepository` | `infra/services/notification_service.rs:13` | **Soft** | It is the only write path. Could be replaced by an `Inbox` port that takes an ID + payload and writes to `wallet_notifications`. |
| `crate::domain::shared_kernel::{AggregateRoot, DomainEvent, DomainEventBus}` | All `domain/notification` + `application/notification/commands/handlers/*` | **Hard (cheap)** | Shared kernel; the new service would carry this over. |

---

## 3. Outbound dependents (what other domains import from notifications)

**The notifications domain is depended on by 22 distinct call sites across 6 non-notification files.** Almost every one of them is a *publisher* — a place that, when something happens in the caller's domain, wants to push a notification to a wallet. There is also one big secondary coupling: the chat domain reuses `RedisNotificationBroadcaster` for its own real-time fanout.

### 3a. Direct imports of `crate::web::notifications::…` (type-level coupling)

```
src/infrastructure/services/notification_service.rs:14
src/infrastructure/services/plan_expiration_service.rs:16
src/infrastructure/container/simple_container.rs:13
src/infrastructure/container/stateless_service_factory.rs:11
src/web/auth/app_state.rs:11
src/web/admin/notification_handlers/notification_types.rs:4
src/web/admin/notification_handlers/notification_user.rs:1   (intra-admin)
src/web/payments/credit_handlers.rs:257
src/web/payments/submit_tx_handler.rs:569
src/web/user/chat_handlers.rs:267
src/web/admin/chat_handlers.rs:159
src/web/admin/permissions/assignments/create.rs:247
src/web/admin/permissions/assignments/remove.rs:74
src/web/docs/openapi.rs                (utoipa registration)
src/web/docs/openapi_admin.rs          (utoipa registration)
src/web/docs/openapi_user.rs           (utoipa registration)
src/web/routes/unified_router.rs       (route registration)
```

### 3b. Functional call sites (the publishers)

These are the actual `NotificationService::send` / `NotificationService::broadcast` call sites — every one is a cross-domain fan-out from a non-notifications module:

| Caller | File:line | What it publishes |
|---|---|---|
| `web/payments/credit_handlers.rs` | 258 | Payment credit event (plan purchase) |
| `web/payments/submit_tx_handler.rs` | 570 | Payment tx submission result |
| `web/user/chat_handlers.rs` | 269 | Chat message → assigned agent |
| `web/user/chat_handlers.rs` | 281 | Chat broadcast (no assigned agent) |
| `web/admin/chat_handlers.rs` | 160 | Admin chat reply → wallet |
| `web/admin/permissions/assignments/create.rs` | 248 | Plan assigned to wallet |
| `web/admin/permissions/assignments/remove.rs` | 75 | Plan revoked from wallet |
| `infra/services/plan_expiration_service.rs` | 151-177 (inline) | Plan expiry warning (7d/3d/1d) |

**8 distinct publisher sites across 7 files, in 4 different bounded contexts (payments, chat, admin permissions, plan lifecycle).** `admin_assignment.rs` itself (the file in the audit scope) does *not* publish notifications directly — it delegates to `application/admin/commands/assign_admin_plan.rs` which is silent about notifications; the `assignments/create.rs` and `assignments/remove.rs` handlers do the publishing in the `web/admin/permissions` module, not in `admin_assignment.rs`.

### 3c. `RedisNotificationBroadcaster` is reused by the chat domain

The chat domain does not have its own pub/sub. It uses the **notification's** broadcaster, with different channel names:

| File:line | Channel | Purpose |
|---|---|---|
| `web/user/chat_handlers.rs:85,86,252,256,353,356,409,412` | `chat:new`, `chat:agent:<id>` | Chat message fanout |
| `web/user/chat_upload_handlers.rs:83,86,197,236,238,268` | `chat:new`, `chat:agent:<id>` | Chat upload fanout |
| `web/admin/chat_handlers.rs:148,210,212` | `chat:wallet:<addr>`, `chat:new`, `chat:agent:<id>` | Admin chat reply |
| `web/user/chat_handlers.rs:548-552` | `subscribe_to_channel` | Chat SSE stream |

`RedisNotificationBroadcaster::publish_to_channel` (`redis_broadcaster.rs:115`) and `subscribe_to_channel` (`redis_broadcaster.rs:132`) are the *generic* primitives — they take a channel name as a string. The notifications-specific methods (`publish_to_wallet`, `subscribe_to_wallet`) are the *typed* version. **If notifications is split out, chat needs its own broadcaster or the broadcaster needs to be hoisted into a shared `infra` crate.**

### 3d. Container / AppState coupling

`RedisNotificationBroadcaster` is constructed in two container files and held on `AppState`:

- `infrastructure/container/simple_container.rs:299` — `let broadcaster = Arc::new(RedisNotificationBroadcaster::new(Arc::clone(&pool_arc)));`
- `infrastructure/container/stateless_service_factory.rs:151` — same line
- `web/auth/app_state.rs:27,47,71` — `pub redis_broadcaster: Option<Arc<RedisNotificationBroadcaster>>` on `AppState`

This is a 22nd outward dependency vector: every handler that takes `State<AppState>` has the broadcaster visible. Of the 6 publishers above, 5 reach it via `app_state.redis_broadcaster`; only `notification_service.rs` accepts `AppState` and reads it indirectly.

### 3e. Hardness of outbound coupling

| Outbound edge | File:line | Hard / Soft | Why |
|---|---|---|---|
| 8 publisher call sites calling `NotificationService::send/broadcast` | (see 3b) | **Hard** | Replacing them with an in-process event bus is mechanical but each one is a *direct* call — no port exists. |
| 8+ `RedisNotificationBroadcaster::publish_to_channel("chat:...")` call sites | (see 3c) | **Hard** | The chat system would need its own broadcaster (or notifications would carry a "generic pubsub" service). |
| `AppState.redis_broadcaster` field | `web/auth/app_state.rs:27` | **Hard** | Visible to every handler; would become a port in a split. |
| `web/admin/wallet_notification_repository` referenced from `infra/services/notification_service.rs:13` | | **Soft** | Could be a `NotificationInbox` port behind `NotificationService::send`. |
| `web/admin/notification_handlers/notification_admin.rs` (admin overview) reads `notification_subscriptions` table indirectly | (admin handler) | **Hard** | Schema lives in the notifications DB. |
| OpenAPI registration | `web/docs/openapi.rs`, `…_admin.rs`, `…_user.rs` | **Soft** | Just route docs; would move with the service. |

---

## 4. Database seams

### 4a. Schema inventory (in the notifications DB)

| Table | Rows / columns | Purpose | Migrated in |
|---|---|---|---|
| `wallet_notifications` | 25 columns | In-app + persisted SSE backlog | `00000000000001_consolidated_notifications_v2/up.sql` |
| `notification_subscriptions` | 11 columns (UNIQUE on `(instance_id, connection_id)`) | Tracks active SSE connections across backend instances | same |

**There are exactly 2 tables in the notifications schema, both in the same physical DB.** No views, no triggers, no stored procs.

```sql
-- indexes
CREATE INDEX idx_notifications_recipient ON wallet_notifications(recipient_wallet_address);
CREATE INDEX idx_notifications_status ON wallet_notifications(status);
CREATE INDEX idx_notifications_created_at ON wallet_notifications(created_at);

CREATE INDEX idx_subs_wallet_active ON notification_subscriptions(wallet_address, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subs_instance_active ON notification_subscriptions(instance_id, connected_at) WHERE disconnected_at IS NULL;
CREATE INDEX idx_subs_stale ON notification_subscriptions(last_ping_at, disconnected_at) WHERE disconnected_at IS NULL;
```

`diesel_notifications.toml` filters schemas to **only** `["notifications", "wallet_notifications"]` — `wallet_notifications` is the only real table; `notifications` is a stale value (no such table exists in migrations). The filter is loose enough that it works, but the config should be cleaned up.

### 4b. Cross-schema references

```bash
$ rg -n 'REFERENCES|FOREIGN KEY' migrations/notifications/ src/schemas/notifications.rs
(no results)
```

**No foreign keys from notifications tables into any other domain's tables, and no FKs from any other domain's tables into `wallet_notifications` / `notification_subscriptions`.** The notifications DB is completely referentially isolated.

### 4c. Cross-DB writers

The `wallet_notifications` table is *written* from:

- `infra/services/notification_service.rs:45` — via `WalletNotificationRepository::create` (the "all callers" path).
- `infra/services/plan_expiration_service.rs:227-260` — direct SQL via `sqlx`/`tokio-postgres` (`INSERT INTO wallet_notifications ...`). **This bypasses the repository entirely.**
- `web/admin/notification_handlers/notification_admin.rs` (admin broadcast, from the admin overview).

`notification_subscriptions` is *written* from:

- `web/notifications/sse_handlers.rs` (sse stream connection open) — but I see no INSERT in sse_handlers.rs; this may be a vestigial index only. (The table is also referenced by the schema file but is not actively populated in the read path I traced.) **This needs verification before the split.**

### 4d. Migration hygiene

Two issues that block a clean split:

1. **Duplicate migration**: `00000000000001_consolidated_baseline_v2/up.sql` is byte-identical to `00000000000001_consolidated_notifications_v2/up.sql` (`diff -q` reports no difference). One of them should be removed. The `baseline` was likely created when the v2 schema was copied into the new DB during dev. The `diesel migration run --config diesel_notifications.toml` would re-apply both on a fresh DB, but the table is `CREATE TABLE` not `CREATE TABLE IF NOT EXISTS`, so the second one would fail.
2. **Diesel config filter**: `diesel_notifications.toml:5` lists `["notifications", "wallet_notifications"]` — the first is a stale entry. Should be `["wallet_notifications"]`.

Neither blocks a *correctness* split, but both will cause pain on the first `diesel migration run` in the new microservice.

### 4e. Schema ownership

`wallet_notifications` and `notification_subscriptions` are **owned 100 % by the notifications context.** No other bounded context has a `CREATE TABLE` for them, no other domain's migrations touch them, and no FK references them.

The only seam the schema introduces is the *read* side: `web/admin/notification_handlers/notification_admin.rs` (admin broadcast) and `web/admin/notification_query_helper.rs` query directly. Both files already live under `web/admin/notification_handlers/` and would move with the service.

---

## 5. API surface

All routes are mounted at `/api/notifications` (user) and `/api/admin/notifications` (admin). The SSE stream is at `/api/notifications/stream`.

### 5a. User / client routes — `/api/notifications/*`

| Method | Path | Handler | Auth | Notes |
|---|---|---|---|---|
| `GET` | `/api/notifications/stream` | `web/notifications/sse_handlers.rs:94` `sse_notifications_handler` | Bearer (validated in-handler) | SSE; queries `?types=` and `?timeout=` |
| `GET` | `/api/notifications/` | `web/admin/notification_handlers/notification_user.rs` `get_user_notifications_handler` | Bearer | List |
| `GET` | `/api/notifications/preferences` | `web/user/unified_user_handlers.rs:896` | Bearer | Get preferences |
| `POST` | `/api/notifications/preferences` | `web/user/unified_user_handlers.rs:1137` | Bearer | Update preferences |
| `GET` | `/api/notifications/unread-count` | `get_unread_count_handler` | Bearer | |
| `PUT` | `/api/notifications/mark-all-read` | `mark_all_notifications_read_handler` | Bearer | |
| `DELETE` | `/api/notifications/clear-all` | `clear_all_notifications_handler` | Bearer | |
| `PUT` | `/api/notifications/{id}/read` | `mark_notification_read_handler` | Bearer | |
| `PUT` | `/api/notifications/{id}/unread` | `mark_notification_unread_handler` | Bearer | |
| `PUT` | `/api/notifications/{id}/acknowledge` | `acknowledge_notification_handler` | Bearer | |
| `DELETE` | `/api/notifications/{id}` | `delete_notification_handler` | Bearer | |

### 5b. Admin routes — `/api/admin/notifications/*`

Mounted via `web/admin/routes.rs:245-253`, gated by `perm_guard("admin:notifications:manage")` at the layer below.

| Method | Path | Handler |
|---|---|---|
| `POST` | `/api/admin/notifications/send` | `send_notification_handler` (admin/notification_handlers/notification_admin.rs) |
| `GET` | `/api/admin/notifications` | `get_all_notifications_handler` |
| `GET` | `/api/admin/notifications/stats` | `get_notification_stats_handler` |
| `GET` | `/api/admin/notifications/overview` | `admin_notification_overview_handler` |
| `PUT` | `/api/admin/notifications/{id}/acknowledge` | `acknowledge_notification_handler` |
| `DELETE` | `/api/admin/notifications/{id}` | `delete_admin_notification_handler` |
| `POST` | `/api/admin/notifications/upload-image` | `media_handlers::upload_notification_image` (multipart) |

The media upload route is the only one that *crosses* a domain boundary: `media_handlers::upload_notification_image` lives under `web/admin/media_handlers/`, not under notifications. This is a small leak: notifications uses the admin media service for image uploads on notification payloads.

### 5c. Out-of-route paths that *also* matter

- `infra/services/plan_expiration_service.rs:80` — calls `cleanup_old_notifications(pool, 90).await` from a background task driver (`bootstrap.rs:180`).
- `bootstrap.rs:180-186` — wires the `PlanExpirationService` as a background task with a poll loop. This is the only background service in the notifications context.

### 5d. OpenAPI registration

The notification types `SSENotification`, `NotificationType`, `NotificationPriority` are re-exported in `web/notifications/mod.rs:11-16` and registered in `web/docs/openapi.rs`, `…_admin.rs`, `…_user.rs`. The utoipa `#[utoipa::path]` annotations are co-located with the handlers, so the OpenAPI definitions move with the service.

---

## 6. External integrations

The notifications service is, in practice, a **Redis + Postgres + admin UI** system. Domain enums define Email/SMS/Push channels that are *not* wired.

### 6a. Redis (active)

| Channel pattern | Producer | Consumer | Purpose |
|---|---|---|---|
| `notifications:wallet:<addr-lowercased>` | `RedisNotificationBroadcaster::publish_to_wallet` (`redis_broadcaster.rs:23`) | `sse_notifications_handler` via `subscribe_to_wallet` (`sse_handlers.rs:160`) | Per-wallet real-time fanout |
| `notifications:all` | `RedisNotificationBroadcaster::publish_to_all` (`redis_broadcaster.rs:54`) | same | Broadcast to all wallets |
| `chat:new` | `web/user/chat_handlers.rs`, `web/user/chat_upload_handlers.rs`, `web/admin/chat_handlers.rs` | SSE chat streams | Chat fanout (**cross-domain reuse of broadcaster**) |
| `chat:agent:<id>` | same | same | Per-agent chat fanout |
| `chat:wallet:<addr>` | `web/admin/chat_handlers.rs:148` | per-wallet chat stream | Admin → user chat |

`RedisNotificationBroadcaster` is a single `Arc<RedisPool>` (constructed in `simple_container.rs:299` and `stateless_service_factory.rs:151`). The pool is the *global* Redis pool, not a notifications-scoped one — it is the same pool that the rest of the system uses for caching and pubsub.

**Operational dependency:** the SSE stream's "live" path requires Redis. If Redis is down, the SSE handler falls back to a one-shot "queued notifications" delivery and the long-poll loop dies (see `sse_handlers.rs:160-165`).

### 6b. PostgreSQL (active)

| Table | Path | Operations |
|---|---|---|
| `wallet_notifications` | `notifications` schema (its own physical DB) | CRUD via `WalletNotificationRepository` (admin side) + `infra/services/notification_service.rs` write path + `plan_expiration_service.rs` raw INSERT + `notification_repository_adapter.rs` port impl |
| `notification_subscriptions` | same | Indexed; appears referenced by the schema but I could not find an active INSERT in the read-path. **Verify before split.** |

Two DB pools are involved: `app_state.db_pool` (primary / `core` schema) and `crate::infrastructure::database::get_notifications_pool()` (notifications schema). The `notification_service.rs` *falls back* to the primary pool if the notifications pool is unavailable (`infra/services/notification_service.rs:37-41` and `96-100`):

```rust
let pool = if let Ok(p) = crate::infrastructure::database::get_notifications_pool().await {
    Arc::new(p)
} else {
    app_state.db_pool.clone()
};
```

This is a soft failover that writes notifications to the wrong schema if the notifications DB is down. **This is a real correctness bug that becomes a major incident when the service is split.**

### 6c. Domain enums that are NOT wired

`domain/notification/value_objects/delivery_channel.rs:7-15` defines six channel types:

```rust
pub enum DeliveryChannelType {
    WebPush, Push, InApp, Email, Sms, SMS
}
```

None of `WebPush`, `Push`, `Email`, `Sms` / `SMS` has any actual sending code. `Cargo.toml` has *no* `lettres`, `sendgrid`, `twilio`, `firebase-messaging`, `web-push`, `apns`, or `fcm` dependencies. The only actually-wired channel is the implicit `InApp` + SSE combo (Postgres + Redis).

The `in_app` alias in `delivery_channel.rs:42` (`"inapp" | "app" => Ok(DeliveryChannelType::InApp)`) and the `wallet` → `Push`, `websocket` → `InApp` aliases (`delivery_channel.rs:45-46`) confirm: there is no real WebSocket/WebPush/SMTP/SMS client in the codebase.

### 6d. SSE itself

`web/notifications/sse_handlers.rs` uses axum's `Sse` response with a 15 s keep-alive. Each connection validates the JWT in-handler, fetches queued notifications from Postgres, then subscribes to Redis pub/sub and forwards messages. The `notification_subscriptions` table is designed for multi-instance fanout tracking (per `instance_id` + `connection_id`), but the SSE handler I read does not write to it on connect. **This is a gap that needs to be confirmed.**

### 6e. No third-party HTTP clients, no blockchains, no webhooks

Unlike payments (which has `apps/backend/src/infrastructure/blockchain/` adapters) or analytics (which scrapes TradingView), notifications has **zero outbound HTTP / WebSocket calls to external services.** All "external" communication is to Redis on the same host.

### 6f. Cron / scheduled work

- `infra/services/plan_expiration_service.rs:80-83` — `cleanup_old_notifications(pool, 90)` runs once per `poll_interval_secs` (default 3600 s, see `plan_expiration_service.rs:31`).
- `infra/services/plan_expiration_service.rs:89-200` — checks for plans expiring in `[7, 3, 1]` days and publishes notifications.
- The service is started in `bootstrap.rs:180` and uses `tokio::spawn` with a poll loop. No external scheduler (no `cron` crate, no `tokio-cron-scheduler`).

---

## 7. Split-readiness score

**Score: 2.5 / 5** — could be lifted to 4 with focused work, but the chat coupling is the blocker.

### Justification (by the numbers)

| Dimension | Score | Evidence |
|---|---|---|
| Domain purity | **5** | Zero cross-domain `use crate::` imports in `domain/notification` or `application/notification`. |
| Schema isolation | **4** | 2 tables, no FKs, no shared columns; one duplicate migration; one stale Diesel filter entry. |
| Domain event contracts | **3** | `NotificationCreated`, `NotificationCancelled`, etc. exist in `domain/notification/events/notification_events.rs` and are *published* by command handlers, but **no one subscribes to them anywhere in the codebase** (rg on the 7 event types shows hits only in the events module itself). The events are an inert interface. |
| Transport coupling | **2** | The chat domain reuses `RedisNotificationBroadcaster` for its own channels. This is the single biggest blocker. |
| Cross-domain fan-in | **1** | 8 distinct publisher call sites across 7 files in 4 bounded contexts. They use `AppState.redis_broadcaster` and call `NotificationService::send` / `RedisNotificationBroadcaster::publish_to_channel` directly — no port, no event bus, no abstraction. |
| External dependencies | **5** | Redis + Postgres only. No email, no SMS, no push, no third-party HTTP. |
| API surface cleanliness | **3** | All routes are under `/api/notifications` and `/api/admin/notifications`. One cross-domain helper (`media_handlers::upload_notification_image`) and a hard dep on `AppState` for auth. |
| Concurrency / isolation | **2** | `notification_service.rs` falls back to the primary `db_pool` if the notifications pool is missing — a soft failover that silently writes to the wrong schema. The background `PlanExpirationService` is a `tokio::spawn` driver, not a proper job system. |
| Operational cost | **4** | Only one background task, no external API keys, no rate-limit-sensitive services. |

**Average: ~3.1, but I round down to 2.5** because the chat-reuses-broadcaster problem is severe: even after introducing an event-bus port, the broadcaster would have to be lifted to a shared crate, or the chat system would have to grow its own. Either is a substantial refactor.

### What "ready to lift" would look like

To go from 2.5 → 4:

1. **Introduce a `NotificationPort` trait** that other domains call instead of `NotificationService::send`. The port takes `(wallet, type, priority, title, message, data, action_url)`. Implementations: in-process (current behavior) and HTTP (the microservice). This eliminates the 8 publisher call sites' direct coupling.
2. **Move `RedisNotificationBroadcaster` to `infra/pubsub`** (or duplicate it for chat), so notifications is no longer the home of a generic pubsub primitive.
3. **Replace the `notification_subscriptions` connection-tracking gap** — either implement it (INSERT on SSE connect) or drop the table.
4. **Fix the `notification_service.rs` pool fallback** so notifications only ever writes to the notifications DB; remove the `app_state.db_pool.clone()` fallback.
5. **Deduplicate the migration history** (`00000000000001_consolidated_baseline_v2` vs `…_consolidated_notifications_v2`) and tighten `diesel_notifications.toml`'s `only_tables` filter to `["wallet_notifications"]`.
6. **Move the media-upload-image helper** out of `media_handlers` (or duplicate it as a thin wrapper inside notifications). It is the only `/api/admin/notifications/upload-image` cross-domain call.

---

## 8. Top 3 specific refactors

Each is sized to be a single PR / single wave. The first two are independent; the third depends on the first.

### Refactor 1 — Introduce a `NotificationPort` and migrate the 8 publisher call sites

**Why:** the 8 publisher call sites are the primary leak surface that would break when notifications leaves. They call `NotificationService::send` / `NotificationService::broadcast` directly. After a split, those calls would have to become HTTP (or gRPC) over the network, but the callers have no abstraction in front of them today.

**Files touched:**
- New file: `apps/backend/src/application/notification/ports/notification_port.rs` — define `#[async_trait] pub trait NotificationPort: Send + Sync { async fn send(&self, req: SendNotificationRequest) -> Result<String, AppError>; async fn broadcast(&self, req: BroadcastNotificationRequest) -> Result<String, AppError>; }`.
- New file: `apps/backend/src/infrastructure/adapters/services/in_process_notification_adapter.rs` — implements `NotificationPort` by delegating to the existing `WalletNotificationRepository::create` + `RedisNotificationBroadcaster::publish_to_wallet` (the exact body that lives in `infra/services/notification_service.rs:21-138` today, lifted verbatim).
- Refactor `apps/backend/src/infrastructure/services/notification_service.rs` to a thin module-level function that delegates to whichever `NotificationPort` is wired in `AppState`.
- Migrate 8 call sites to take `Arc<dyn NotificationPort>` (or read it from `AppState`):
  - `apps/backend/src/web/payments/credit_handlers.rs:258`
  - `apps/backend/src/web/payments/submit_tx_handler.rs:570`
  - `apps/backend/src/web/user/chat_handlers.rs:269, 281`
  - `apps/backend/src/web/admin/chat_handlers.rs:160`
  - `apps/backend/src/web/admin/permissions/assignments/create.rs:248`
  - `apps/backend/src/web/admin/permissions/assignments/remove.rs:75`
  - `apps/backend/src/infrastructure/services/plan_expiration_service.rs:151-177` (replace the raw INSERT + `publish_to_wallet` with a single `notification_port.send(...)` call).
- Container wiring: register the in-process adapter in `simple_container.rs` and `stateless_service_factory.rs`; expose `pub notification_port: Arc<dyn NotificationPort>` on `AppState` (or — better — pass the port as a constructor arg to each publisher handler).

**LOC estimate:** ~600 LOC (200 port + 250 adapter + 150 mechanical migration at 8 sites, ~5 min each).

**Why it unblocks the split:** after this refactor, the notifications microservice can be replaced with an HTTP client behind the same `NotificationPort` trait, and *zero caller code has to change*. The 8 publishers do not need to know whether notifications is in-process or remote.

### Refactor 2 — Hoist `RedisNotificationBroadcaster` into a shared `pubsub` crate (or duplicate it for chat)

**Why:** the chat domain's reliance on the notification broadcaster (`publish_to_channel("chat:new", …)`) means the chat code has to import `crate::web::notifications::RedisNotificationBroadcaster`. If notifications becomes a microservice, chat either loses its pubsub mechanism or has to reach across the network. The broadcaster's core operation is a 30-LOC wrapper around `redis::AsyncCommands::publish`; it has no notifications-specific logic. It belongs in a shared crate.

**Files touched:**
- New crate (or new module `apps/backend/src/infrastructure/pubsub/`): `RedisBroadcaster { pool: Arc<RedisPool> }` with `publish(channel, payload)`, `subscribe(channel)`, `get_subscriber_count(channel)`. ~80 LOC, copy body from `redis_broadcaster.rs:1-150`.
- `apps/backend/src/web/notifications/redis_broadcaster.rs` — keep only the notifications-typed wrappers (`publish_to_wallet`, `publish_to_all`, `subscribe_to_wallet`) and have them delegate to the new generic broadcaster.
- `apps/backend/src/web/notifications/mod.rs:18` — re-export the new `RedisBroadcaster` from the new crate.
- Migrate 8+ chat call sites to use the generic broadcaster:
  - `apps/backend/src/web/user/chat_handlers.rs:85,86,252,256,353,356,409,412,551-552`
  - `apps/backend/src/web/user/chat_upload_handlers.rs:83,86,197,236,238,268`
  - `apps/backend/src/web/admin/chat_handlers.rs:148,210,212`

**LOC estimate:** ~200 LOC (80 new broadcaster + 100 re-wiring at 8+ sites).

**Why it unblocks the split:** chat keeps its pubsub after notifications leaves. The chat SSE stream (at `/api/chat/stream`) and the notification SSE stream (at `/api/notifications/stream`) can each have their own backend without sharing a single Redis connection owned by notifications.

### Refactor 3 — Deduplicate the notifications migrations and tighten the Diesel config

**Why:** the two `00000000000001_*` migration files are byte-identical; on a fresh notifications DB, `diesel migration run --config diesel_notifications.toml` will fail on the second one because it has a plain `CREATE TABLE wallet_notifications`. The first thing the new microservice's `db setup` will do is break.

**Files touched:**
- Delete one of:
  - `apps/backend/migrations/notifications/00000000000001_consolidated_baseline_v2/`
  - `apps/backend/migrations/notifications/00000000000001_consolidated_notifications_v2/`
  (Both directories: `up.sql` and `down.sql`.)
- Update `apps/backend/diesel_notifications.toml:5` — change `["notifications", "wallet_notifications"]` to `["wallet_notifications"]` (the `"notifications"` entry is stale; no such table exists).
- Verify: `cd apps/backend && DATABASE_URL=… diesel migration run --config diesel_notifications.toml` on a fresh DB.

**LOC estimate:** ~10 LOC net (4 lines deleted, 1 line changed).

**Why it unblocks the split:** a clean migration history is a hard prerequisite for the new microservice's `db/migrate` job to work in CI. The duplicate currently works only because the existing dev DB already had the table.

### Bonus refactor (small but visible)

**Move `media_handlers::upload_notification_image` to `web/admin/notification_handlers/`.** It's the only cross-domain call under `/api/admin/notifications/*` (`web/admin/routes.rs:252`). Either give notifications its own image-upload endpoint that stores via S3/MinIO directly, or move the existing `upload_notification_image` to live next to the rest of the notification handlers. This is ~30 LOC and would close the last `/api/admin/notifications/*` route that does not stay inside the notifications context.

---

## Appendix A — Bounded context vs transport layer: where each file lives

```
PURE DDD LAYER
  domain/notification/                                          (14 files, 3,914 LOC)
    aggregates/notification.rs                                  Notification aggregate
    events/notification_events.rs                               7 domain events
    repository_ports/notification_repository_port.rs            NotificationRepositoryPort
    value_objects/{notification_id, notification_content,
                   notification_topic, delivery_channel,
                   schedule_info, user_preferences}.rs          2,808 LOC of VOs

  application/notification/                                    (27 files, 1,298 LOC)
    commands/handlers/{create_user, create_topic, cancel,
                       update_priority, record_delivery}_handler.rs
    commands/models/*.rs
    queries/handlers/{get_notification, list_notifications,
                      list_pending_notifications,
                      get_delivery_status}_handler.rs
    queries/models/*.rs

TRANSPORT / INFRA LAYER (mix of pure-transport and DB-touching)
  web/notifications/                                            (5 files, 1,021 LOC)
    sse_handlers.rs          SSE handler + SSENotification/Type/Priority types
    redis_broadcaster.rs     RedisNotificationBroadcaster (used by chat too!)
    offline_queue.rs         Postgres-backed queue (fetch_queued, mark_delivered, cleanup)
    tests.rs
    mod.rs                   re-exports

  web/admin/notification_handlers/                              (4 files, 1,307 LOC)
    notification_admin.rs    Admin send / list / stats / overview
    notification_user.rs     Per-user notification ops
    notification_types.rs    Request/response DTOs
    mod.rs

  web/admin/wallet_notification_repository.rs                   631 LOC — direct Diesel repo (parallel to port)
  web/admin/notification_query_helper.rs                        55 LOC

  infrastructure/services/notification_service.rs              139 LOC — the cross-domain facade
  infrastructure/services/plan_expiration_service.rs           (background task, 7/3/1-day expiry)

  infrastructure/repositories/notification_repository.rs        416 LOC — concrete repo (find_pending)
  infrastructure/repositories/notification_record.rs           144 LOC — DB record struct

  infrastructure/adapters/repositories/notification_repository_adapter.rs   315 LOC — port impl
  infrastructure/adapters/repositories/mappers/notification_mappers.rs      122 LOC — domain ↔ DB
  infrastructure/adapters/services/notification_service_adapter.rs         63 LOC

  infrastructure/models/notification.rs                          99 LOC — DB model
  schemas/notifications.rs                                      167 LOC — Diesel table DSL

  migrations/notifications/                                     6 files, 216 LOC (with duplicates)
```

## Appendix B — Verification commands

```bash
# Cross-domain imports from inside notifications
rg -n 'use crate::' apps/backend/src/{domain,application,web}/notification/ \
  | rg -v 'notification|shared_kernel'
# → (no results)

# Cross-domain call sites of NotificationService
rg -n 'NotificationService::(send|broadcast)' apps/backend/src/

# Cross-domain call sites of RedisNotificationBroadcaster
rg -ln 'RedisNotificationBroadcaster|publish_to_channel' apps/backend/src/

# Event subscribers (should be empty for notifications events)
rg -l 'NotificationCreated|NotificationScheduled|NotificationCancelled' apps/backend/src/
# → only files inside domain/notification itself

# All notification tables
rg -n '^diesel::table!' apps/backend/src/schemas/notifications.rs

# FKs from notifications schema
rg -n 'REFERENCES|FOREIGN KEY' apps/backend/migrations/notifications/ apps/backend/src/schemas/notifications.rs
# → (no results)

# Duplicate migration
diff -q apps/backend/migrations/notifications/00000000000001_consolidated_baseline_v2/up.sql \
        apps/backend/migrations/notifications/00000000000001_consolidated_notifications_v2/up.sql
# → (no output → identical)
```

## Appendix C — Things I could not verify from a read-only audit

- Whether `notification_subscriptions` is actively written to. The table and indexes exist; the SSE handler reads from `notification_subscriptions` via `notification_query_helper.rs` paths I did not trace in full, but I did not find an INSERT statement in `sse_handlers.rs`. Worth a `SELECT COUNT(*)` check before splitting.
- The exact poll interval for `PlanExpirationService` (default 3600 s, from `plan_expiration_service.rs:31`; not externally configurable).
- Whether the `/api/notifications/upload-image` route is exercised in production (the call sites are admin-only and not in the test fixtures I read).
- The set of `NotificationType` values in the SSE `parse_notification_types` function (`sse_handlers.rs:340-358`) is 9 variants, but the `NotificationType` enum in the same file lists 10 (`WalletManagement` is a separate value, plus `Wallet`). The discrepancy is in the parse function, not the enum. Looks like a bug but is out of scope.
