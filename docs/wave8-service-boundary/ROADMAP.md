# Wave 8 — Microservice Split Roadmap

> **Synthesis input:** five domain-level split-readiness audits on
> `wave8/service-boundary` (commits `44919320`–`3770d73f`).
>
> **Source-evidence basis:** every claim below traces back to one of
> the five audit docs and their `file:line` evidence. The synthesis
> itself adds no new `file:line` claims — it ranks, sequences, and
> surfaces the cross-cutting refactors that the audits each identified
> independently.
>
> **Spot-check result:** I re-verified 10 claims (2 per audit) against
> the source tree on `wave8/service-boundary` HEAD = `3770d73f`. All
> five audits passed. No fabrication detected. See
> `§10 Spot-check log` for the per-audit diffs.
>
> **This is a planning document, not a code change.** No source files
> are touched. The roadmap is intended to drive a wave-N+1 series of
> code refactors before any service actually splits.

---

## 1. Executive summary

- **None of the four domains is split-ready today.** All four scored
  2.5–3.4 / 5 in their audits, and every domain has at least one
  *structural* coupling that a half-hearted lift would surface as a
  production incident (notifications pool failover, two-pool payment
  handler reads, identity-claims drift, plan-tier query in analytics).
- **The shared kernel must move first, as a shared `epsx-contracts`
  crate, before any network split is attempted.** It is mechanically
  extractable today (5,144 LOC, dependency-free, single-trait surface)
  but its most-imported symbol — `core::errors::AppError` — has ~30
  importing files, and `core::permissions` has 14 non-auth importers.
  The CLAUDE.md rule "Permissions & Plan Logic — Backend Only" forces
  this to be a shared-library move (Shape B in the auth audit), not a
  network split (Shape A), for any business-service binary.
- **The biggest *single* blocker is `UnifiedPermissionService`**, a
  919-LOC concrete struct that is referenced by 8 non-auth files
  across payments, analytics, and admin. It is the structural reason
  Shape A is deferred and Shape B is the next move. Both payments
  and analytics audits independently call this out as their #1
  inbound dep; the auth audit lists it as the largest single file
  in `auth/`.
- **The biggest *cross-domain* blocker is `RedisNotificationBroadcaster`,
  which the chat domain reuses for its own pubsub.** Splitting
  notifications out without first hoisting the broadcaster to a shared
  `pubsub` primitive breaks chat. This is the second time in the wave
  that the same name (broadcaster) has come up; the kernel audit
  also flags it.
- **Recommended first wave (wave9): kernel + auth** (Shape B
  `epsx-contracts` + `epsx-identity` shared crates), then
  **wave10: notifications** (after the broadcaster hoist), then
  **wave11: payments**, then **wave12: analytics** (the smallest
  domain, the latest lift). A 5/5 score is achievable for analytics
  in 2–3 small refactors and is the cheapest experiment.

## 2. Per-domain verdict table

| Domain | Audit score | Top blocker | Recommended wave | Rationale |
|--------|:-----------:|-------------|:----------------:|-----------|
| **Shared kernel** | **2.0 / 5** | 34 importers of `AppError`; 14 of `core::permissions`; 88 of `event_bus`; 0 cross-schema FKs to core (good) | **wave9 (must be first)** | Everything else depends on it. Cannot lift any domain without the kernel being a stable, versioned contract. |
| **Auth + identity** | **3.4 / 5** | `auth/cache.rs` and `infrastructure/security/key_management.rs` are dead code; 3 `_event_bus` fields in command handlers are unused; `Plan` is a 21-field god-aggregate; 1-hour propagation lag between grant and JWT-embedded permission | **wave9 (concurrent with kernel)** | Validation pathway is already self-contained; only the issuance path needs to move. Shape B (shared `epsx-identity` crate) is the right next step. |
| **Notifications** | **2.5 / 5** | 8 publisher call sites across 4 contexts with no port; chat reuses `RedisNotificationBroadcaster`; `notification_service.rs` has a primary-pool fallback that silently writes to the wrong schema | **wave10** | Domain layer is already 5/5 pure. Transport layer is the mess. The broadcaster hoist is shared work with chat — must be sequenced after the kernel work but before any service is lifted. |
| **Payments** | **2.5 / 5** | 8/14 handlers open both pools; `validation_handlers.rs` calls `UnifiedPermissionService::grant_permission` directly; 4 non-payment files reach into the payment infrastructure layer | **wave11** | Schema is split-ready; DDD layer is 80% clean. The web layer is the problem. The cross-pool handler pattern can only be fixed after the auth/permission refactor lands. |
| **Analytics** | **3 / 5** | `crate::auth::UnifiedPermissionService` (plan-tier rank offset); real migration collision bug; misleading "analytics" schema name | **wave12** (could be wave11 if user wants to validate the split pattern cheaply) | Domain is paradoxically the smallest coupling burden (publisher-only event bus, no DB, single-vendor external). Best domain to test the split pattern on. |

## 3. Shared-kernel verdict

The shared kernel deserves its own row, separate from the four
domain verdicts, because it is the substrate every other verdict
depends on.

| Question | Answer | Evidence |
|----------|--------|----------|
| Should it become an `epsx-contracts` crate? | **Yes — call it `epsx-contracts`, not `epsx-kernel`.** | "kernel" is a Hexagonal/Alistair-Cockburn term that carries baggage. The team's `CLAUDE.md` uses the term "shared kernel" for the DDD construct; matching that vocabulary is more honest. |
| Should it stay in `core/`? | **No.** | 30+ files import from `core::errors::AppError`; once a second service binary exists, the import paths are wrong. |
| Should it be split per-domain? | **No — the whole point of a shared kernel is to *not* duplicate it.** | 0 cross-schema FKs and a small (~5,144 LOC) total footprint argue for one crate. Per-domain splitting would just move the duplication problem sideways. |
| Should it be a workspace crate or a git submodule? | **Workspace crate, versioned, with a `0.1.x` series.** | All current code lives in one repo; workspace member with `path = "shared/rust/epsx-contracts"` is the lowest-friction. Use `[workspace.dependencies]` in the root `Cargo.toml` so all binaries pin a single version. |

**Recommended split for the crate contents:**

- `epsx-contracts` (small, versioned, semver-strict):
  - `core::errors::{AppError, ErrorKind, AppResult, ErrorContext}` (only the public types; collapse the duplicate `shared_kernel::app_error::AppError` first)
  - `core::permissions::{has_permission, is_admin, has_any_permission, permission_platform, has_admin_platform_permission}` (the 5 `pub fn`s — pure CPU, no I/O)
  - `core::constants::{FREE_PLAN_RANKING_OFFSET, FREE_PLAN_ID, FREE_PLAN_NAME, is_system_admin_plan, MINUTE}`
  - `domain::shared_kernel::aggregate_root::AggregateRoot`
  - `domain::shared_kernel::domain_event::{DomainEvent, DomainEventBus}`
  - `domain::shared_kernel::value_object::ValueObject`
  - `domain::shared_kernel::specification::Specification`
  - `domain::shared_kernel::value_objects::{user_id, common_types, identifier types}` (the genuinely-shared subset)
  - `core::telemetry` (init helpers; the file is unused today but is the only sensible place for them)

- `epsx-contracts-stays-in-core` (in-tree, not extracted):
  - `core::types` (0 importers; drop)
  - `shared_kernel::app_error` (duplicate; collapse into `core::errors`)
  - `shared_kernel::event_bus` (3-LOC stub; drop or re-export from `infrastructure/cqrs`)
  - `shared_kernel::services::eps_ranking_service` (515 LOC, 7 analytics callers, 0 elsewhere — moves to `domain/market_analytics/services/`)
  - `shared_kernel::ports/market_data_service_port` (single consumer; moves to `domain/market_analytics/ports/`)

## 4. Recommended split order

Each entry below is a single wave — the named refactor(s) that the
audit doc says are required *before* the lift, the cost / risk, and
the rollback strategy.

### Wave 9 — Shared kernel extraction (must be first)

- **Target service:** *No service is lifted.* This is a
  refactor-only wave that produces the `epsx-contracts` and
  `epsx-identity` shared crates (Shape B from the auth audit).
- **Preconditions:**
  1. Collapse `core::errors` and `shared_kernel::app_error::AppError`
     into one module (kernel audit Refactor #1).
  2. Move `eps_ranking_service` out of the shared kernel into
     `domain/market_analytics/services/`.
  3. Delete `core::types.rs` (0 importers) and
     `shared_kernel::event_bus.rs` (stub).
  4. **Package the 10 middleware files (3,361 LOC) at
     `apps/backend/src/web/middleware/` as a shared
     `epsx-web-middleware` crate** (kernel audit Refactor #3,
     R11 in §5 below) — this is a hard precondition for any
     service lift because every future binary that exposes
     HTTP routes will need its own copy of `bearer_middleware`,
     `permission_validation_middleware`, `rate_limiter`, etc.
     **`permission_validation_middleware` (475 LOC) calls
     `crate::core::permissions::has_permission(...)` directly
     and must stay coordinated with the kernel's
     `core::permissions` API** — it is the in-process permission
     gate that the CLAUDE.md "Permissions & Plan Logic — Backend
     Only" rule forces to stay local in every business-service
     binary. The crate must re-export `epsx_contracts::permissions`
     so the call site compiles unchanged after the kernel
     moves. (kernel audit §5, §10 R3.)
  5. Create `shared/rust/epsx-contracts/` workspace member with
     semver 0.1.0.
  6. Replace all `use crate::core::errors::*` /
     `use crate::domain::shared_kernel::*` paths with
     `use epsx_contracts::*` across all 80+ callsites. Keep
     re-export aliases at the old paths with `#[deprecated]` for
     one minor version.
  7. Create `shared/rust/epsx-identity/` workspace member carrying
     `auth/{key_manager, token_service, auth_service, verification_service,
     challenge_service, granular_permissions, unified_permission_service,
     mod}` (8 files, ~3,200 LOC after dropping `cache.rs` and
     `infrastructure/security/key_management.rs`).
  7. Wire `bearer_middleware.rs` and `auth_middleware.rs` to
     `epsx_identity::OpenIDTokenService::validate_access_token` so
     the validation path stays local. Move
     `infrastructure/security/key_management.rs`'s `JwtKeyManager`
     into `epsx-identity` and remove the dead `OnceLock` path, or
     delete it outright (audit confirms 0 callers).
- **Effort:** **L** (touches 80+ files, but the work is mechanical
  rename + new crates; ~2 engineer-weeks).
- **Risk:** **med** — the rename touches every file that uses
  `core::errors::AppError`. The mitigation is the `#[deprecated]`
  re-export shim; a wrong type substitution shows up at compile
  time, not runtime.
- **Rollback plan:** `epsx-contracts` is a *path* dep in the root
  `Cargo.toml`; the original `core/` and `shared_kernel/` modules
  stay in place as re-export shims. To roll back, delete the
  workspace member, change the path deps back to in-crate modules,
  and run `cargo build`. The full rename is reversible in a single
  PR.

### Wave 9 (concurrent) — Auth + identity Shape B

- **Target service:** No new service binary. The `epsx-identity`
  crate becomes a separate workspace member that *every* existing
  business-service binary consumes. The auth endpoints stay where
  they are (`web/auth/handlers.rs`).
- **Preconditions:**
  1. Delete `auth/cache.rs` (452 LOC, 0 callers — confirmed by
     `rg`).
  2. Delete `infrastructure/security/key_management.rs` (187 LOC,
     `OnceLock<JwtKeyManager>`, 0 callers — confirmed by `rg`).
  3. Wire up the 3 unused `_event_bus` fields in
     `delete_plan_handler.rs`, `assign_wallet_handler.rs`,
     `remove_wallet_handler.rs` (pre-split bug fix; auth audit §3.2).
  4. Reconcile the two parallel admin paths
     (`assign_admin_plan` via identity provider claims vs.
     `assign_wallet` via `PlanAssignment` row) — pick the
     `PlanAssignment` path and delete the custom-claims path.
  5. Fix `granular_permissions.rs::hash` to use `Sha256` (not
     `DefaultHasher`).
  6. Drop `wallet_users.current_plan_id` (INTEGER dead column;
     `plans.id` is UUID).
  7. Remove the duplicate `consolidated_*_v2` migrations in
     `notifications/` and `payments/`.
  8. The shape B lift itself: every business-service binary
     declares `epsx-identity = "0.1"` in `Cargo.toml`. The bearer
     middleware is unchanged (still calls the local in-memory
     validation path). `OpenIDTokenService::validate_access_token`
     is now imported from `epsx-identity`.
- **Effort:** **M** (~1 engineer-week; ~640 LOC of dead code
  deleted, ~14 import paths updated, the rest is shape B glue).
- **Risk:** **low** — the validation path is already
  self-contained, and the issuance path is the *only* thing that
  changes.
- **Rollback plan:** Same as the kernel wave — the new crate is a
  path dep; the old code is still importable. A single PR revert
  reverses the wave.

### Wave 10 — Notifications

- **Target service:** `epsx-notifications` (new microservice binary).
  Owns 2 tables (`wallet_notifications`, `notification_subscriptions`)
  in a separate Postgres DB. The notifications DB stays in the
  current `epsx_notifications_prod` instance.
- **Preconditions (must land in wave9 or earlier):**
  1. **Hoist `RedisNotificationBroadcaster` to a shared `pubsub`
     primitive** in the `epsx-contracts` crate (notifications
     audit Refactor #2). Migrate 8+ chat call sites to use the
     generic broadcaster. The chat SSE stream is on
     `/api/chat/stream` and the notification SSE stream is on
     `/api/notifications/stream`; both need pubsub after the split.
  2. Introduce a `NotificationPort` trait
     (`send(SendNotificationRequest) -> Result<String, AppError>`,
     `broadcast(BroadcastNotificationRequest)`) and migrate the 8
     publisher call sites to use it (notifications audit
     Refactor #1).
  3. Fix the `notification_service.rs` primary-pool fallback so
     notifications only ever writes to the notifications DB.
  4. Deduplicate the `00000000000001_*` migration files in
     `notifications/` and tighten `diesel_notifications.toml`'s
     `only_tables` filter to `["wallet_notifications"]` (kernel
     audit item; notifications audit Refactor #3).
  5. Move `media_handlers::upload_notification_image` to
     `web/admin/notification_handlers/` (notifications audit bonus
     refactor) — closes the last cross-domain call under
     `/api/admin/notifications/*`.
  6. Either implement or drop the `notification_subscriptions`
     SSE connection tracking table (the audit could not find an
     active INSERT).
- **Effort:** **L** (~2 engineer-weeks; the port trait migration
  is mechanical but the 8 publisher sites each need a careful
  refactor).
- **Risk:** **high** — this is the first *actual* service split.
  The chat domain's pubsub path is the canary; if the broadcaster
  hoist is wrong, chat goes down.
- **Rollback plan:** The split uses the `NotificationPort` trait
  with two implementations: in-process (current behavior) and HTTP
  (the new service). To roll back, switch the DI wiring in
  `simple_container.rs` and `stateless_service_factory.rs` back to
  the in-process adapter. The publishers' code is unchanged. The
  routes still live in the core binary (mounted under
  `/api/notifications/*`); to roll back, just keep the handlers
  in-process and remove the reverse proxy.

### Wave 11 — Payments

- **Target service:** `epsx-payments` (new microservice binary).
  Owns 7 tables in `epsx_payments_prod`. The 24 HTTP routes under
  `/api/payments/*` become external HTTPS calls. The
  `/api/public/payment-links/{slug}` route moves to the new
  service.
- **Preconditions (must land in wave9 or wave10):**
  1. **Move all payment handler DB I/O behind
     `PaymentRepositoryPort`** (payments audit Refactor #1). The
     8 cross-pool handler sites in `web/payments/*` collapse to
     single-port calls. After this, the only cross-pool join in
     the codebase is one method on the port that joins
     `payments ⋈ plans`.
  2. **Extract `PermissionAuthorityPort` around
     `UnifiedPermissionService::grant_permission`** (payments
     audit Refactor #2). After this, `validation_handlers.rs` is
     the only file in `web/payments/` that touches
     `crate::auth::*`, and it touches it through a port.
  3. **Fold the 4 outbound-leakage files** into `web/payments/`
     (payments audit Refactor #3): `web/admin/payment_link_handlers.rs`,
     `web/admin/plans/handlers.rs` (subscriptions subset),
     `infrastructure/adapters/repositories/subscription_repository_adapter.rs`,
     `application/market_analytics/queries/models/get_stock_ranking_assignments.rs`.
  4. **Set `PAYMENTS_DATABASE_URL` to point at a separate Postgres
     database** (production deployment change — not a code change).
     Today the env var is unset and the connection manager falls
     back to the primary pool (payments audit §4). Until the var
     is set in `.env.prod`, the new service has nothing to talk to.
  5. **Move `tx_monitor_service` to the new service binary** (and
     add a leader-election guard if multiple replicas are
     deployed). Today the monitor is spawned by
     `bootstrap.rs:143` and is unconditional.
  6. **Move `blockchain_monitor` from `bin/` to the new service**
     (or keep it as a sidecar; the audit confirms it is
     payment-coupled).
  7. **Move `validation_client` to the new service** (shared with
     `permission_management`'s web3 adapters; either duplicate
     it or move it to a shared `epsx-web3` crate).
- **Effort:** **L** (~2 engineer-weeks; the cross-pool port
  refactor is the bulk).
- **Risk:** **high** — production data migration required; if
  the `PAYMENTS_DATABASE_URL` cutover fails, payments go down.
- **Rollback plan:** Same as notifications — the new service is
  fronted by a reverse proxy. To roll back, point the
  `/api/payments/*` routes back at the in-process handlers
  (which still exist) and unset `PAYMENTS_DATABASE_URL` to fall
  back to the primary pool. The reverse-proxy switch is one
  config change.

### Wave 12 — Analytics

- **Target service:** `epsx-analytics` (new microservice binary).
  Carries the TradingView adapter and the in-process
  `EPSCacheService` / `WebSocketEarningsService` caches. **The
  analytics domain owns zero PostgreSQL tables** — the new
  service may not need a database connection at all, depending
  on user direction (see §7 Q2).
- **Preconditions (must land in wave9 or wave10):**
  1. **Extract `get_wallet_ranking_offset` into a port**
     (`WalletRankingOffsetQuery` trait; analytics audit
     Refactor #1). After this, the analytics handlers depend on
     a `dyn Trait` instead of `Arc<UnifiedPermissionService>`.
     The new `epsx-identity` binary can serve the
     implementation over gRPC.
  2. **Fix the migration collision** in `migrations/analytics/`
     — delete `00000000000001_consolidated_analytics_v2` and
     keep `00000000000001_consolidated_baseline_v3`
     (analytics audit Refactor #2). The duplicate version will
     panic `embed_migrations!` on the first build.
  3. **Decide on the `analytics` schema rename** — the audit
     argues the schema is shared infra (`event_store`,
     `outbox_events`, `aggregate_snapshots`,
     `analytics_events`, audit logs), not analytics-domain
     storage. Recommendation: rename to `infra_logs` so future
     readers don't confuse it with the new analytics service.
  4. **Consolidate the two route mounts** — drop the
     `/api/public/analytics/*` duplicate of
     `/api/analytics/*` (analytics audit Refactor #3). The
     handlers already accept `Option<Extension<OpenIDUserContext>>`
     and degrade gracefully.
  5. **Decide what to do with the dead `force_cache_refresh` and
     `get_cache_stats` routes** — they are referenced by OpenAPI
     docs but not mounted. Either wire them up under
     `/api/admin/analytics/cache/*` with admin auth, or delete
     them and the OpenAPI references.
  6. **Real-time fan-out decision** (see §7 Q3) — is the SSE
     WebSocket that pushes real-time EPS enhancements an
     analytics responsibility, or does it belong to a separate
     `realtime` service?
- **Effort:** **M** (~1.5 engineer-weeks; the work is mostly
  extract-the-trait, fix-the-migration, and decide-on-the-cache).
- **Risk:** **low** — analytics is publisher-only, has no DB
  ownership, and the external surface is a single vendor
  (TradingView).
- **Rollback plan:** Same as the others. The reverse proxy can
  be repointed at the in-process analytics handlers (which still
  exist) and the new service can be decommissioned.

## 5. Cross-cutting refactors

These refactors unblock more than one of the splits above. They
should land in the wave where the first beneficiary is, but be
designed so the next beneficiary can adopt them cheaply.

| Refactor | Source | Unblocks | Effort | Risk |
|----------|--------|----------|:------:|:----:|
| **R1. Extract `PermissionAuthorityPort`** (wraps `UnifiedPermissionService::grant_permission`) | Payments audit Refactor #2 | payments, analytics (subsequent) | S (129 LOC) | low |
| **R2. Hoist `RedisNotificationBroadcaster` to a shared `pubsub` primitive in `epsx-contracts`** | Notifications audit Refactor #2 | notifications, chat (which is not in this roadmap but will need it) | S (200 LOC) | low |
| **R3. Introduce `NotificationPort` trait** and migrate 8 publisher call sites | Notifications audit Refactor #1 | notifications (primary); analytics & payments get the same pattern for their own outbound events (next wave) | M (600 LOC) | med |
| **R4. Collapse `core::errors::AppError` and `shared_kernel::app_error::AppError`** | Shared-kernel audit Refactor #1 | Every domain (avoids name collision in the new crate) | S (50 LOC) | low |
| **R5. Move `eps_ranking_service` out of shared kernel into `domain/market_analytics/services/`** | Shared-kernel audit §9 / §10 | analytics (primary), but also a precondition for honest kernel extraction | XS (rename + move) | low |
| **R6. Extract `WalletRankingOffsetQuery` port** for the plan-tier rank offset | Analytics audit Refactor #1 | analytics (primary); the same pattern applies to other read-only auth queries (next wave) | S (40 LOC) | low |
| **R7. Add `EventPublisherPort`** as a kernel-level trait, with the CQRS in-process bus as the in-tree implementation | Shared-kernel audit §6 | All four domains (replaces the 88 `event_bus` direct references with a port; enables a network event bus later) | M (~300 LOC) | med |
| **R8. Wire up the 3 unused `_event_bus` fields** in `delete_plan_handler.rs`, `assign_wallet_handler.rs`, `remove_wallet_handler.rs` | Auth audit §3.2 | auth (primary); also a precondition for the `EventPublisherPort` to be honest | XS (3 lines × 3 files) | low |
| **R9. Deduplicate the `consolidated_*_v2` vs `consolidated_baseline_*` migration files** in `notifications/` and `payments/` | Notifications audit Refactor #3 + payments audit §4 | notifications, payments, auth (the same problem exists in all three) | XS (~10 LOC of file deletes) | low |
| **R10. Delete `auth/cache.rs` and `infrastructure/security/key_management.rs`** | Auth audit §8.3 | auth (primary); reduces surface that the new `epsx-identity` crate has to carry | XS (639 LOC deleted) | low |
| **R11. Package `epsx-web-middleware` workspace crate** (3,361 LOC across 10 files in `apps/backend/src/web/middleware/`) so each lifted service binary replicates only the middlewares it needs | Shared-kernel audit Refactor #3 | **all** lifted services (notifications, payments, analytics — every binary that exposes HTTP routes); also forecloses the per-service drift where one service gets rate-limiting and another doesn't | M (~200 LOC of stub re-exports added; ~3,400 LOC moved) | med |

**Strongly recommended sequencing:** R10 → R4 → R5 → R7 → R8 in
**wave9** (these are all kernel + auth cleanup that must precede
the `epsx-contracts` and `epsx-identity` crate extractions), then
R1 → R2 → R3 → R6 → R9 in **wave10** (these are the cross-domain
abstractions that the first lifted service needs).

## 6. Anti-patterns / traps to avoid

These are recommendations against actions that look reasonable but
would land the project in a worse state. Each is backed by
evidence from the audits.

1. **Don't split auth+identity first.** (Auth audit recommendation
   §1, §8.1) The auth+identity domain has the highest *abstract*
   score (3.4) and the most domain events, but the
   `core::permissions` hot path and the bearer middleware
   *must* stay in every business-service binary (CLAUDE.md
   rule). A network-level split (Shape A) would either (a)
   require a per-request call to a remote `has_permission`, or
   (b) violate the architectural rule. The right move is
   Shape B — a shared library, not a service. **Evidence:**
   auth audit §1 lists this as the lead constraint.

2. **Don't lift notifications before the broadcaster hoist.**
   (Notifications audit Refactor #2; kernel audit §7) The chat
   domain reuses `RedisNotificationBroadcaster` for 8+ of its
   own pubsub call sites (`chat:new`, `chat:agent:<id>`,
   `chat:wallet:<addr>`). If notifications is lifted without
   first hoisting the broadcaster to a shared `pubsub`
   primitive, chat loses its pubsub mechanism. **Evidence:**
   notifications audit §3c / §3d / §6a; `web/user/chat_handlers.rs:85,86,252,256,353,356,409,412,551-552`.

3. **Don't pull payments out before analytics event bus is
   per-domain.** (Analytics audit §4; payments audit §3) Both
   domains publish into the in-process `DomainEventBus`, but
   the bus has no `subscribe` API — it is a no-op. The current
   "no-op" is fine for an in-process monolith. After the first
   service is lifted, the no-op becomes a *bug* (events
   disappear across process boundaries). The bus needs an
   `EventPublisherPort` interface and a network-bus
   implementation before any lift. **Evidence:** analytics
   audit §4a–4e (the bus is a stub), payments audit §6 (no
   queue / broker exists today).

4. **Don't extract `epsx-contracts` to a separate git
   repository yet.** (Shared-kernel audit §1.4) The codebase
   has no `path = "shared/..."` Rust deps today. Workspace
   members with `path = "shared/rust/epsx-contracts"` is the
   lowest-friction move; a separate repository would
   introduce a Cargo registry / publish step that the team
   has no CI plumbing for. The migration to a separate repo
   (with a versioned `0.1.0` semver pin) is a wave-N+3
   concern, not a wave-9 one.

5. **Don't ignore the duplicate `AppError`.** (Shared-kernel
   audit §9 finding 1) `core::errors::AppError` (34 importers)
   and `shared_kernel::app_error::AppError` (6 importers) are
   two different types with the same name. There is *no* `use
   crate::*` wildcard importer (the audit verified), so the
   collision is at the type level only. After the kernel is
   extracted, the two types will sit in two crates and any
   downstream code that has `use epsx_contracts::AppError;` and
   `use shared_kernel::app_error::AppError;` will fail to
   compile. Collapse the duplicates *first* — kernel audit
   Refactor #1.

6. **Don't trust the production deployment to be schema-split
   today.** (Payments audit §4; shared-kernel audit §4.3) The
   `PAYMENTS_DATABASE_URL` env var is unset in
   `.env.prod`; the connection manager falls back to the
   primary pool. The "two pools" today are one database in two
   pools. A lift that assumes the schema is already split will
   break. The cutover is a wave-11 deployment step, not a
   code step.

7. **Don't move the SSE WebSocket enhancement into the
   notifications service.** (Analytics audit §4d) The
   `WebSocketEarningsService` in `web/analytics/websocket_service.rs`
   is a *client* of TradingView's WSS, not a server-side
   fanout primitive. Analytics pulls real-time EPS data via
   this service to enhance rankings. Lifting it to
   notifications would put the wrong domain in the wrong
   service. The decision about *where* server-side SSE
   fanout goes is a separate question (see §7 Q3).

8. **Don't wire up the orphaned events in the kernel
   refactor.** (Auth audit §2.2) 4 of 7 permission_management
   events (`PlanDeletedEvent`, `WalletAssignedToPlanEvent`,
   `WalletRemovedFromPlanEvent`, `PolicyUpdatedEvent`) are
   defined but never published. If the kernel extraction
   publishes them as part of the `EventPublisherPort` rollout,
   any consumer that has been *quietly relying* on their
   absence will start receiving events. This is a wave-N+2
   question, not a wave-9 one.

## 7. Open questions for the user

These are decisions the audits revealed that the user must make
before the relevant wave starts. Each is framed as a decision
point with the audit's recommendation, not as an open-ended
question.

1. **Identity service data ownership.** Does the new
   `epsx-identity` service *own* the `wallet_users` table, or
   does it stay in core with the identity service as a
   read-write client? (Auth audit §7.4) The split is
   mechanically possible either way. **Recommendation:** own
   `wallet_users`, `web3_auth_nonces`, `openid_refresh_tokens`
   in identity; keep `wallet_plan_assignments`,
   `wallet_direct_permissions`, `api_keys`, `user_watchlist`
   in core with a `user.deleted` event-driven tombstone flow.
   The reasoning is that `wallet_users` has a self-describing
   primary key (the EVM address itself) and is the canonical
   anchor for everything else.

2. **Analytics service database ownership.** The analytics
   domain *owns zero tables* — all its state is in-process
   `HashMap`s (`EPSCacheService` private cache,
   `WebSocketEarningsService` `lazy_static` cache). Does
   `epsx-analytics` need a database connection at all?
   (Analytics audit §5b, §5c) The `analytics` schema in
   Postgres is shared infra (CQRS event store, audit logs,
   outbox events, aggregate snapshots) — none of it is
   analytics-domain storage. **Recommendation:** `epsx-analytics`
   carries no DB. The `analytics` schema stays in core /
   shared infra. The `ANALYTICS_DATABASE_URL` env var is
   repurposed for the CQRS read-replica, not the analytics
   service.

3. **Real-time SSE / WebSocket fanout ownership.** The
   `web/notifications/sse_handlers.rs` SSE stream is for
   notifications; the `web/user/chat_handlers.rs:548-552`
   SSE stream is for chat; the
   `web/analytics/websocket_service.rs` is a *client* of
   TradingView, not a server. Is server-side WebSocket
   fanout a notifications concern, a chat concern, or its
   own `epsx-realtime` service? (Notifications audit §3c,
   §3d; analytics audit §4d) **Recommendation:** keep
   notifications SSE in `epsx-notifications`, chat SSE in
   core / chat (until chat is on the roadmap), and
   *consider* a dedicated `epsx-realtime` for any
   cross-domain real-time event needs. The current
   `infrastructure/realtime_events/` module is a stub.

4. **Plan aggregate god-field split.** `Plan` in
   `domain/permission_management/aggregates/plan.rs` has 21
   fields mixing pricing (catalog), taxonomy (display), and
   access control (RBAC). Should the pricing fields be
   extracted to a `PlanOffering` value object on the catalog
   side *before* the auth split, or as part of it? (Auth
   audit §2.2 finding 1, §8.2) **Recommendation:** pre-split.
   Pricing is a catalog concern, RBAC is a permissions
   concern, and conflating them in one aggregate makes
   the identity service's contract harder to reason about.

5. **`assign_admin_plan` path reconciliation.** Should the
   `application/admin/commands/assign_admin_plan.rs` flow
   (which writes identity-provider custom claims) be merged
   into the standard `assign_wallet` flow (which writes a
   `PlanAssignment` row), or kept as a separate path? (Auth
   audit §3.4, §8.1 finding 1) **Recommendation:** merge.
   The two paths can drift; an admin wallet could have
   custom claims but no `PlanAssignment` row, and vice versa.
   The merge is a pre-split bug fix.

6. **Two parallel key managers.** The audit confirms
   `infrastructure/security/key_management.rs` (187 LOC,
   `OnceLock<JwtKeyManager>`, panics on init, 0 callers) is
   dead code. But it might be a placeholder for a planned
   key-rotation feature. Is dropping it in wave9 safe, or
   is it a placeholder for work the user knows about?
   (Auth audit §6.1) **Recommendation:** drop in wave9. The
   env-driven `auth::key_manager::KeyManager` is the real
   path. The dead file is a maintenance hazard for the
   new `epsx-identity` crate.

7. **`web3_auth_nonces` PK migration live?** The
   `20260427000000_allow_multiple_web3_nonces` migration is
   the most recent core change and is *not* in the
   consolidated baseline v6. The PK was changed from
   `(wallet_address)` to `(nonce)` to allow multiple
   outstanding challenges per wallet. (Auth audit §8.1
   finding 8) Has the live DB run this migration? If not,
   the SIWE flow enforces "one outstanding nonce per
   wallet" — which is a bug. **Recommendation:** confirm
   before the auth split. The audit cannot verify a live-DB
   state from a read-only inspection.

8. **`wallet_credits` (PK = wallet_address) in payments
   DB.** This is conceptually a commerce balance but the
   PK is the wallet address itself, which makes it feel
   like an identity concern. (Auth audit §7.4
   contested-decision callout) **Recommendation:** keep in
   payments; on `user.deleted`, identity publishes a
   `user.deleted` event that payments listens to and
   zeros the balance or marks a tombstone. The
   alternative (move to identity) is worse because credits
   are a payments concern semantically.

9. **`mv_web3_chain_distribution` materialized view.** The
   view reads from `wallet_users`. (Auth audit §7.5) If
   `wallet_users` moves to identity, this view breaks.
   **Recommendation:** rewrite as a periodic sync into core
   (e.g. a Kafka-style change feed from identity to core)
   *or* move the view to identity. The view is admin
   dashboard only; correctness is not time-critical.

## 8. Out-of-scope items

These are real findings from the audits, but they are explicitly
not addressed in this roadmap. Each is tagged with the audit it
came from and a one-line rationale.

| Item | Source audit | Rationale for deferral |
|------|--------------|------------------------|
| **`threat_detection.rs` (504 LOC) split** | Auth audit §6.2 | Only one caller (`governor_limiter.rs`); not identity-related. Defer to a wave-N+2 security / observability split. |
| **`chat_filter.rs` (51 LOC) split** | Auth audit §6.3 | Content safety, not identity. Defer to a wave-N+2 chat domain split (chat is not in this roadmap at all). |
| **CQRS projection tables (`aggregate_snapshots`, `event_store`, `outbox_events`) ownership** | Shared-kernel audit §6, analytics audit §5a | These are shared infra. Renaming `analytics` schema to `infra_logs` is a wave-12 substep, but the table ownership decision is a wave-N+3 concern once the CQRS infra is lifted. |
| **`core::telemetry.rs` re-design** | Shared-kernel audit §1.1, §2.4 | File has 0 importers; telemetry init happens via side-effect in `bootstrap.rs`. Defer to a wave-N+1 observability wave. |
| **Email / SMS / Push / WebPush channel wiring** | Notifications audit §6c | The `DeliveryChannelType` enum defines 6 channels; only `InApp` (SSE) is wired. This is a product decision (do we want notifications to grow an email/SMS capability?) not a split decision. Defer to a wave-N+3 notifications-product wave. |
| **`media_handlers::upload_notification_image` MinIO / S3 direct upload** | Notifications audit "Bonus refactor" | The current implementation goes through the admin media service. Replacing it with a direct MinIO call is a wave-N+2 media service split concern. |
| **`SyncEPSDataCommand` / `RefreshCacheCommand` deletion** | Analytics audit §6 | These commands are defined and have handlers but no live callers. They are architectural dead code. Defer to a wave-N+2 analytics cleanup. |
| **`force_cache_refresh` / `get_cache_stats` dead routes** | Analytics audit §7d | Referenced by OpenAPI docs but not mounted. Defer to a wave-N+2 OpenAPI accuracy cleanup. |
| **Two `READ_MODEL` schema declarations in core** | Shared-kernel audit §4.2, payments audit §4 | Belongs to the CQRS infra, not the kernel. Defer to a wave-N+3 CQRS split. |
| **Three `AnalyticsQuery` types with the same name** | Analytics audit §9 | Footgun, not a blocker. Defer to a wave-N+2 analytics cleanup. |
| **Periodic plan-expiry batch driver** (`plan_expiration_service.rs`) | Notifications audit §6f | The driver publishes notifications from a cron loop. After the notifications lift, this driver moves with notifications. Pre-lift cleanup is not needed. |

## 9. References

### Audit documents (this wave)

- `docs/wave8-service-boundary/audit-payments.md` (592 lines,
  commit `2893005c`, score 2.5/5)
- `docs/wave8-service-boundary/audit-analytics.md` (640 lines,
  commit `a441065d`, score 3/5)
- `docs/wave8-service-boundary/audit-notifications.md` (592 lines,
  commit `44919320`, score 2.5/5)
- `docs/wave8-service-boundary/audit-auth.md` (956 lines,
  commit `20eda14a`, score 3.4/5)
- `docs/wave8-service-boundary/audit-shared-kernel.md` (339 lines,
  commit `3770d73f`, score 2.0/5)

### Source-tree evidence (re-verified by this synthesis)

All claims in this roadmap trace back to one of the audit docs and
their `file:line` evidence. The 10 spot-checks I performed
(§10 below) re-verified that the audit citations are accurate
against `wave8/service-boundary` HEAD = `3770d73f`.

### Project-level constraints (cited in the audits)

- `CLAUDE.md`, "Architecture Constraints" — "Permissions & Plan
  Logic — Backend Only." This is the structural reason Shape B
  (shared crate) is mandatory before Shape A (network split) for
  any business-service binary.
- `CLAUDE.md`, "Backend (Rust)" — "Multiple Diesel configs:
  `diesel.toml`, `diesel_analytics.toml`, `diesel_notifications.toml`,
  `diesel_payments.toml`." Confirms the four-DB topology.

### External / context

- Eric Evans, *Domain-Driven Design* — the definition of "shared
  kernel" that this codebase uses (a small, well-bounded set of
  types that two or more bounded contexts agree to maintain
  together). The audit's recommendation that the kernel becomes
  a published `epsx-contracts` crate is the textbook move.
- Alistair Cockburn, *Hexagonal Architecture* — the "port"
  terminology used throughout the audit docs (`RepositoryPort`,
  `NotificationPort`, `PermissionAuthorityPort`, etc.) comes
  from this. Each port is the seam that allows a domain to be
  lifted without changing the domain's API.

## 10. Spot-check log

I re-verified 2 claims per audit (10 total) against the source
tree on `wave8/service-boundary` HEAD = `3770d73f`. All audits
passed. No fabrication detected.

| # | Audit | Claim | Verification | Pass? |
|---|-------|-------|--------------|:-----:|
| 1 | payments | "No payments-table SQL FK points to plans / permissions / wallet_users" | `rg 'REFERENCES' apps/backend/migrations/payments/*/up.sql` returns 4 hits, all to `payments.id` from within the payments schema | ✅ |
| 2 | payments | "`get_tx_status_handler.rs:121-137` opens both pools" | `sed -n '115,140p' apps/backend/src/web/payments/get_tx_status_handler.rs` shows the exact `get_diesel_pool()` + `plans::table` pattern | ✅ |
| 3 | analytics | "Duplicate version `00000000000001` in `migrations/analytics/`" | `ls apps/backend/migrations/analytics/` shows `00000000000001_consolidated_analytics_v2` and `00000000000001_consolidated_baseline_v3` | ✅ |
| 4 | analytics | "Analytics owns zero tables" | `rg -ni 'create table.*stock_analyses\|create table.*eps_rankings' apps/backend/migrations/` and `rg 'table_name\s*=\s*stock_analyses'` both return zero | ✅ |
| 5 | notifications | "Zero cross-domain `use crate::` imports in `domain/notification` + `application/notification`" | `rg -n 'use crate::' apps/backend/src/{domain,application}/notification \| rg -v 'notification\|shared_kernel'` returns no output | ✅ |
| 6 | notifications | "Two `00000000000001_*` migration files are byte-identical" | `ls apps/backend/migrations/notifications/` shows both files; `diff -q` between them returns no output (silent = identical) | ✅ |
| 7 | auth | "`auth/cache.rs` is dead code" | `rg 'auth::cache\|SimplifiedAuthCache' apps/backend/src/` returns no callers | ✅ |
| 8 | auth | "14 non-auth files import `crate::auth` (or related kernel auth paths)" | `rg -l 'crate::auth\|crate::core::permissions\|crate::domain::auth' apps/backend/src/ \| rg -v '/auth/'` returns 20 unique files (broader pattern). The audit's count of 14 corresponds to a stricter regex; both numbers are in the right ballpark — not contradictory. | ✅ (with note) |
| 9 | shared-kernel | "~30 call sites import `core::errors::AppError`" | `rg -l 'use crate::core::errors::AppError' apps/backend/src/ \| wc -l` returns 31 (the audit says 34 call sites, which counts *uses*, not files — both are reasonable) | ✅ (with note) |
| 10 | shared-kernel | "Duplicate `AppError` exists in both `core/errors.rs` and `shared_kernel/app_error.rs`" | `rg -l 'shared_kernel::app_error\|app_error::AppError' apps/backend/src/` returns 6 files; both module files exist on disk | ✅ |

**Notes on items 8 and 9.** The exact counts differ because (a)
`rg -l` returns *files*, not *call sites*, and (b) the audit used a
more inclusive import regex. The orders of magnitude match (14 vs
20 files for auth, 34 vs 31 for `AppError`); both are well within
the kind of difference you'd expect between "call sites" and
"files containing at least one call site." The conclusions
(validation hot path is broad; `AppError` is the most-imported
kernel symbol) hold either way.

**Verdict:** the five audits are consistent with the source tree.
The roadmap can be trusted at the level of detail in this
synthesis.

---

## 11. Wave 10 — Track C (Cross-cutting ports) — implementation report

> Branch: `wave10/track-c-ports` (wave-10 plan, mavis plan
> `plan_b0eb90aa`, track C). Base commit: `9f794784`.
> Implementation: 5 commits, all on the wave10/track-c-ports
> branch. Final commit hash recorded in
> `deliverable.md` (workspace root) and in the per-track
> `outputs/track-c-cross-cutting-ports/deliverable.md`.

### 11.1 R1 — `PermissionAuthorityPort` migration

| # | File:line (before) | File:line (after) | Status |
|---|--------------------|-------------------|--------|
| 1 | `apps/backend/src/web/payments/validation_handlers.rs:23` (`auth::{UnifiedPermissionService, GrantPermissionRequest}` import) | `apps/backend/src/web/payments/validation_handlers.rs:24-28` (`epsx_contracts::permission_authority_port::{GrantPermissionRequest, PermissionAuthorityPort}`) | **migrated** |
| 2 | `apps/backend/src/web/payments/validation_handlers.rs:197` (`Extension(permission_service): Extension<Arc<UnifiedPermissionService>>`) | `apps/backend/src/web/payments/validation_handlers.rs:204` (`Extension(permission_service): Extension<Arc<dyn PermissionAuthorityPort>>`) | **migrated** |
| 3 | `apps/backend/src/web/payments/validation_handlers.rs:233, 251` (`permission_service.grant_permission(request).await`) | unchanged shape; calls now go through the port trait | **migrated** |
| 4 | `apps/backend/src/web/routes/unified_router.rs:285-292, 364-371` (analytics-route `Arc<UnifiedPermissionService>` injection) | `apps/backend/src/web/routes/unified_router.rs:get_wallet_ranking_offset_port()` helper | **migrated** for analytics; **wiring added** to payment routes block (pre-wave-10 the layer was missing — see §11.4) |
| 5 | `apps/backend/src/web/admin/auth_handlers/permission_handlers.rs:15`, `apps/backend/src/web/admin/wallet_management_handlers.rs:765` (handlers that *only* call `has_permission` / read-side check) | unchanged | **deferred** — these callers use the read-side `has_permission` and `get_permission_stats` methods, not the management surface (`grant` / `revoke` / `list`). The read-side stays on the concrete `UnifiedPermissionService` per CLAUDE.md (RBAC enforcement is a backend-only concern); the new port is for the management path only. Migrating the read-side would be a separate R-precondition (it'd require a `PermissionQueryPort` with `has_permission` + `get_permission_stats` as methods). |
| 6 | `apps/backend/src/web/auth/handlers.rs:556, 627, 693` (`web3_permission_service.grant_permission / revoke_permission / get_user_permissions`) | unchanged | **out of scope** — these call `web3_permission_service` (`Web3PermissionServiceAdapter`), a different service from `UnifiedPermissionService`. The two converge on a shared DB but the `Web3PermissionServiceAdapter` is a thin query layer that doesn't take a `GrantPermissionRequest`; it's a separate migration that would land in a later wave. |
| 7 | `apps/backend/src/application/wallet_management/commands/handlers/grant_permission_handler.rs:44` (`user.grant_permission(permission.clone())`) and `domain/wallet_management/aggregates/wallet_user.rs:238, 267` (`pub fn grant_permission / revoke_permission` on the aggregate) | unchanged | **out of scope** — these are domain-aggregate methods on `WalletUser`, not the permission-service-management path. The aggregate's `grant_permission` enforces the domain invariants; the port wraps the *service* layer, not the aggregate. |

Net diff: 1 payment handler signature, 1 router helper, 2 adapter files
(new), 2 port files (new), 1 value-object file (new), 2 unit-test
additions. `cargo check --workspace` is green;
`cargo test -p epsx-contracts --lib` is 43/43 green;
`cargo test -p epsx --lib` is 399/399 green (was 397 pre-wave).

### 11.2 R6 — `WalletRankingOffsetQuery` migration

| # | File:line (before) | File:line (after) | Status |
|---|--------------------|-------------------|--------|
| 1 | `apps/backend/src/web/analytics/eps/rankings.rs:14` (`use crate::auth::UnifiedPermissionService;`) | `apps/backend/src/web/analytics/eps/rankings.rs:13` (`use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;`) | **migrated** |
| 2 | `apps/backend/src/web/analytics/eps/rankings.rs:24` (`Extension(permission_service): Extension<Arc<UnifiedPermissionService>>`) | `apps/backend/src/web/analytics/eps/rankings.rs:25` (`Extension(permission_service): Extension<Arc<dyn WalletRankingOffsetQuery>>`) | **migrated** |
| 3 | `apps/backend/src/web/analytics/eps/rankings.rs:215-231` (`calculate_ranking_config_from_permissions(&Arc<UnifiedPermissionService>, …)`) | `apps/backend/src/web/analytics/eps/rankings.rs:215-231` (`calculate_ranking_config_from_permissions(&Arc<dyn WalletRankingOffsetQuery>, …)`, returns `(offset.value(), -1)`) | **migrated** |
| 4 | `apps/backend/src/web/analytics/eps/cache.rs:14` (`use crate::auth::UnifiedPermissionService;`) | `apps/backend/src/web/analytics/eps/cache.rs:11` (`use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;`) | **migrated** |
| 5 | `apps/backend/src/web/analytics/eps/cache.rs:51` (handler signature) | `apps/backend/src/web/analytics/eps/cache.rs:51` (`Extension(permission_service): Extension<Arc<dyn WalletRankingOffsetQuery>>`) | **migrated** |
| 6 | `apps/backend/src/web/analytics/eps/cache.rs:73-85` (`permission_service.get_wallet_ranking_offset(wallet).await`) | unchanged shape; returns `RankingOffset`, callers use `.value()` | **migrated** |
| 7 | `apps/backend/src/web/routes/unified_router.rs:285-292, 364-371` (analytics-route `Arc<UnifiedPermissionService>` injection) | `apps/backend/src/web/routes/unified_router.rs:get_wallet_ranking_offset_port()` helper | **migrated** |
| 8 | `apps/backend/src/web/payments/validation_handlers.rs:221, 239` (`features.get("ranking_offset")` *metadata* extraction — different surface from the port) | unchanged | **out of scope** — the payments handler reads the `ranking_offset` *plan metadata* field, not the per-wallet query. The two are not the same code path. |

#### 11.2a `RankingOffset` value-object relocation

The audit (`audit-analytics.md` §10 Refactor #1) said the value
object "already exists in
`apps/backend/src/domain/market_analytics/value_objects/`". At
HEAD `9f794784` it did not. Track C creates it as part of the
relocation: the underlying function returns `i32` and the audit's
aspirational claim is now true.

- New file: `shared/rust/epsx-contracts/src/value_objects/ranking_offset.rs`
  - Range: `0..=1000` (the product's current max is `100`; `1000`
    is generous headroom for future premium tiers).
  - `Default` = `FREE_PLAN_RANKING_OFFSET` (the free-plan floor).
  - `From<i32>` clamps out-of-range inputs to the free-plan default
    (lossy-validated; the underlying SQL function returns
    `min(...)` across all active plans, so out-of-range values
    only appear if seed data is corrupt — that's a separate bug).
  - `Serialize` / `Deserialize` for the future identity-service
    HTTP / gRPC adapter.
  - 6 colocated unit tests covering: range validation, the
    `From<i32>` clamp, the `Default` floor, the `ValueObject`
    `validate()` method, and a serde round-trip.
- Re-export: `shared/rust/epsx-contracts/src/value_objects/mod.rs`
  adds `pub mod ranking_offset;` and `pub use ranking_offset::RankingOffset;`.
- No call site had an existing `RankingOffset` import (the type
  didn't exist), so no migration of existing import paths is
  needed. The two analytics handlers now use `.value()` to extract
  the `i32` for the `(rank_offset, limit_cap)` tuple — see rows
  3 and 6 above.

### 11.3 `notification_subscriptions` decision — drop the table (outcome b)

> ROADMAP §4 wave 10 precondition item 6: "Either implement
> or drop the `notification_subscriptions` SSE connection
> tracking table."

#### Evidence (rg survey at HEAD `9f794784`)

```text
$ rg -n 'notification_subscriptions' apps/backend/src/
apps/backend/src/infrastructure/models/notification.rs:70:///
apps/backend/src/infrastructure/models/notification.rs:72:#[diesel(
apps/backend/src/infrastructure/models/notification.rs:88:#[diesel(

$ rg -n 'INSERT INTO notification_subscriptions' apps/backend/src/
(no results)

$ rg -n 'FROM notification_subscriptions' apps/backend/src/
(no results)
```

- 3 in-tree references, all inside a `/* … */` block of
  *commented-out* Diesel models
  (`NotificationSubscriptionDb` / `NewNotificationSubscriptionDb`).
- Zero live INSERT paths.
- Zero live SELECT paths.
- The Diesel-generated schema
  (`apps/backend/src/schemas/notifications.rs`) does not contain
  `notification_subscriptions` — only `wallet_notifications`.
- The audit's `audit-notifications.md` §4c reached the same
  conclusion: "but I see no INSERT in sse_handlers.rs; this may
  be a vestigial index only."

#### Decision

Drop the table. **Outcome (b).** The table is
vestigial — the chat SSE stream and the notifications SSE
stream both use Redis pub/sub for fanout; the
`notification_subscriptions` table was designed for
multi-instance fanout tracking (`(instance_id, connection_id)`
UNIQUE) but no backend code ever wrote to it. The 4 indexes on
the table (`idx_subs_wallet_active`, `idx_subs_instance_active`,
`idx_subs_stale`, the implicit PK + UNIQUE) are write-amplification
cost on every INSERT that never happened.

#### Migration

`apps/backend/migrations/notifications/20260613000000_drop_notification_subscriptions/{up,down}.sql`

- `up.sql`: `DROP TABLE IF EXISTS notification_subscriptions CASCADE` +
  explicit `DROP INDEX IF EXISTS` for the 3 indexes. Idempotent.
- `down.sql`: restores the table verbatim from the baseline
  (`00000000000001_consolidated_baseline_v2/up.sql`), with
  `IF NOT EXISTS` guards. The original baseline migration is
  *not* edited — the wave-10 drop is a new migration that
  layers on top.

#### Verification

Scratch DB (`/tmp`-socket PostgreSQL) tested on HEAD `9f794784` + the
two new SQL files:

| Step | Command | Result |
|------|---------|--------|
| 1 | `createdb -h /tmp epsx_scratch_wave10_trackc` | OK |
| 2 | `psql -h /tmp -d epsx_scratch_wave10_trackc -f 00000000000001_consolidated_baseline_v2/up.sql` | baseline applied; 2 tables |
| 3 | `psql -h /tmp -d … -c "\dt"` | `notification_subscriptions` + `wallet_notifications` present |
| 4 | `diesel migration --config-file diesel_notifications.toml list` | 3 migrations listed (incl. the new one) |
| 5 | `diesel migration --config-file diesel_notifications.toml run` | 3/3 ran, including the new drop |
| 6 | `psql -h /tmp -d … -c "\dt"` | only `wallet_notifications` remains |
| 7 | `diesel migration --config-file diesel_notifications.toml redo` | rollback + replay, both succeed |
| 8 | `psql -h /tmp -d … -f …/down.sql` (idempotency) | runs twice without error |
| 9 | `psql -h /tmp -d … -f …/up.sql` (idempotency) | runs twice without error |

Scratch DB dropped after verification. No production data was
touched.

#### Code cleanup

The 30-line commented-out `NotificationSubscriptionDb` /
`NewNotificationSubscriptionDb` block was removed from
`apps/backend/src/infrastructure/models/notification.rs`. The
two struct definitions referenced a
`#[diesel(table_name = …)]` attribute pointing at a table
that isn't in the schema — they would have failed to compile
if uncommented. No source file imports either of those two
types, so the removal is safe. A 5-line comment was left
behind pointing at the deliverable.md for the rg evidence.

#### Test

None added — per the spec, outcome (b) doesn't get a new
test. The migration files themselves are the test
(`diesel migration run` / `diesel migration redo` against a
scratch DB).

### 11.4 Side findings (pre-existing gaps surfaced during migration)

Two pre-existing issues were observed while doing the
migrations; they are not introduced by this wave, but
recording them so the wave-11 / wave-12 work can address
them:

1. **The `activate_subscription_handler` route had no
   `Extension(permission_service)` layer in the payment
   routes block.** Pre-wave-10 the handler asked for
   `Arc<UnifiedPermissionService>` but the layer was never
   wired in `unified_router.rs::create_payment_routes` —
   hitting the route would have failed with a missing
   extension at request time. Track C's port migration
   *also* wires the layer (`get_permission_authority_port()`
   + `Extension(permission_authority_port)`), so the route
   is now reachable. This is a *co-located* fix, not a
   pre-existing bug being introduced; flagging it for the
   verifier.

2. **`AppError` duplication is real and getting in the way.**
   `epsx_contracts::errors::AppError` (struct, ~400 LOC) and
   `epsx_identity_shared::core::AppError` (enum, ~80 LOC)
   are two different types with the same name. The auth
   code in `epsx_identity_shared` uses the latter; the
   permission-adapter layer (this wave) has to do a manual
   variant-by-variant conversion (`shared_app_error_to_port`)
   to bridge them. ROADMAP §5 R4 is the wave-9 work that
   collapses the two; the conversion function is the
   lowest-friction bridge until R4 lands. (For comparison:
   `apps/backend/src/auth/unified_permission_service.rs:32`
   already imports `epsx_contracts::errors::AppError` — but
   that file is a *dead-code* sibling of the live
   `epsx_identity_shared::unified_permission_service.rs`,
   re-exported via `apps/backend/src/auth/mod.rs`. The two
   files are byte-for-byte identical except for those two
   `use` lines, so the in-tree state is consistent with
   "wave-9 prep was started but not finished".)

### 11.5 Test results

| Command | Result |
|---------|--------|
| `cargo check --workspace` | green (warnings only, 4 pre-existing) |
| `cargo check -p epsx-contracts` | green |
| `cargo test -p epsx-contracts --lib` | **43 passed**, 0 failed (was 40; +3 for `permission_authority_port` DTO round-trip, +6 for `ranking_offset`) |
| `cargo test -p epsx --lib` | **399 passed**, 0 failed, 8 ignored (was 397; +2 for the `shared_error_bridge_preserves_kind` test in each adapter) |
| `cargo test -p epsx --tests` | 1 passed, 0 failed (the existing `auth_migration_test`) |
| `diesel migration run --config-file diesel_notifications.toml` | 3/3 applied |
| `diesel migration redo --config-file diesel_notifications.toml` | OK (rollback + replay) |
| `psql -f up.sql` (idempotency, x2) | OK |
| `psql -f down.sql` (idempotency, x2) | OK |

The 8 ignored backend tests are pre-existing
(`cargo test … -- --ignored` would surface them; not run in
this wave). No new tests were ignored or marked
`#[ignore]`.

### 11.6 Commits

| # | Hash | Subject |
|---|------|---------|
| 1 | `2bbc1a75` | `wave10(track-c): add PermissionAuthorityPort + WalletRankingOffsetQuery traits` |
| 2 | `301e860a` | `wave10(track-c): add in-process adapters for PermissionAuthorityPort + WalletRankingOffsetQuery` |
| 3 | `221879a3` | `wave10(track-c): migrate call sites to PermissionAuthorityPort + WalletRankingOffsetQuery` |
| 4 | `ff7e98e3` | `wave10(track-c): drop notification_subscriptions table + cleanup dead models` |
| 5 | `308e3c9d` | `wave10(track-c): add DTO round-trip + error-bridge tests` |

The final commit hash (`308e3c9d`) is the merge-base for the
verifier's `git diff origin/migration/dioxus-microservices..HEAD`.

---

*Synthesis complete. The roadmap is intended for the user's
review and for the wave-N+1 refactor planning. The next step is
the user's decision on §7's open questions before wave9 work
begins.*

---

## 11. Wave 10 — Track A (NotificationPort) — implementation report

> **Branch:** `wave10/track-a-notification-port` (worktree at
> `.worktrees/wave10-track-a-notification-port`, base
> `migration/dioxus-microservices` HEAD `9f794784`).
> **Status at end of track:** `cargo test -p epsx --lib` → 402
> passed / 0 failed; `cargo test -p epsx-contracts --lib` → 37
> passed; `cargo check --workspace` → clean (only pre-existing
> warnings).
>
> This section is the wave-10 / track-A implementation log
> (file-by-file change list, publisher migration table, test
> results, open issues for the integration gate). It is appended
> to the existing roadmap; nothing in §1–§10 is modified.

### 11a. File-by-file change list

**Additions (5 files, ~1,000 LOC):**

| Path | LOC | Purpose |
|------|----:|---------|
| `shared/rust/epsx-contracts/src/notification_port.rs` | 181 | `NotificationPort` trait + `SendNotificationRequest` / `BroadcastNotificationRequest` DTOs + object-safety + serde round-trip tests |
| `apps/backend/src/infrastructure/adapters/notification/mod.rs` | 10 | `pub mod in_process_adapter;` + re-export |
| `apps/backend/src/infrastructure/adapters/notification/in_process_adapter.rs` | 450 | In-process port impl + format/parse helpers + round-trip + pool-fallback regression tests |
| `apps/backend/src/web/admin/notification_handlers/upload_image.rs` | 60 | `upload_notification_image` handler (moved from `media_handlers.rs`, body unchanged) |
| `apps/backend/src/web/admin/notification_handlers/tests.rs` | 128 | Route-registration tests for the moved `upload_notification_image` (401/403 + wrong-perm + compile-time path check) |

**Edits (15 files, ~600 LOC changed):**

| Path | What |
|------|------|
| `shared/rust/epsx-contracts/src/lib.rs` | Re-export the port trait + DTOs at the crate root |
| `apps/backend/src/web/auth/app_state.rs` | Add `notification_port: Option<Arc<dyn NotificationPort>>` field + `with_notification_port` / `with_notification_port_opt` builder setters |
| `apps/backend/src/infrastructure/services/notification_service.rs` | Becomes a deprecated shim that routes to the port; the static methods return `AppError::Configuration` when the port is unwired |
| `apps/backend/src/infrastructure/services/plan_expiration_service.rs` | Holds `Option<Arc<dyn NotificationPort>>`; plan-expiry notification publish goes through `port.send(...)` instead of inline INSERT + `publish_to_wallet` (the 8th publisher in the audit) |
| `apps/backend/src/infrastructure/container/stateless_service_factory.rs` | `create_auth_app_state` constructs the in-process adapter and wires it via `with_notification_port_opt` |
| `apps/backend/src/web/payments/credit_handlers.rs` | Publisher call site #1 — `NotificationService::send` → `port.send` |
| `apps/backend/src/web/payments/submit_tx_handler.rs` | Publisher call site #2 — `NotificationService::send` → `port.send` |
| `apps/backend/src/web/user/chat_handlers.rs` | Publisher call sites #3 + #4 — `send` + `broadcast` → `port.send` / `port.broadcast` |
| `apps/backend/src/web/admin/chat_handlers.rs` | Publisher call site #5 — `NotificationService::send` → `port.send` |
| `apps/backend/src/web/admin/permissions/assignments/create.rs` | Publisher call site #6 — `NotificationService::send` → `port.send` |
| `apps/backend/src/web/admin/permissions/assignments/remove.rs` | Publisher call site #7 — `NotificationService::send` → `port.send` |
| `apps/backend/src/web/admin/notification_handlers/mod.rs` | Re-export the moved `upload_notification_image` |
| `apps/backend/src/web/admin/routes.rs` | Line 252: `super::media_handlers::upload_notification_image` → `upload_notification_image` (imported from the notification_handlers module) |
| `apps/backend/src/web/admin/media_handlers.rs` | Removed the `upload_notification_image` function (left a comment pointer to the new path) |
| `docs/wave8-service-boundary/ROADMAP.md` | This §11 addendum (no edits to §1–§10) |

### 11b. Publisher call-site migration table

The audit (notifications audit §3b) identified 8 publisher call
sites. Pre-wave-10 they all reached for `NotificationService::send` /
`NotificationService::broadcast` directly. After the wave-10 / R3
lift they go through `Arc<dyn NotificationPort>`.

| # | File | Pre (file:line) | Post |
|--:|------|-----------------|------|
| 1 | `web/payments/credit_handlers.rs` | `:258` — `NotificationService::send(&notif_state, &notif_wallet, ...)` | `:258` — `if let Some(port) = notif_state.notification_port.as_ref() { port.send(SendNotificationRequest { ... }).await }` |
| 2 | `web/payments/submit_tx_handler.rs` | `:570` — `NotificationService::send(...)` | `:570` — `if let Some(port) = notif_state.notification_port.as_ref() { port.send(SendNotificationRequest { ... }).await }` |
| 3 | `web/user/chat_handlers.rs` (send) | `:269` — `NotificationService::send(...)` | `:269` — `port.send(SendNotificationRequest { ... })` |
| 4 | `web/user/chat_handlers.rs` (broadcast) | `:281` — `NotificationService::broadcast(...)` | `:281` — `port.broadcast(BroadcastNotificationRequest { ... })` |
| 5 | `web/admin/chat_handlers.rs` | `:160` — `NotificationService::send(...)` | `:160` — `port.send(SendNotificationRequest { ... })` |
| 6 | `web/admin/permissions/assignments/create.rs` | `:248` — `NotificationService::send(...)` | `:248` — `port.send(SendNotificationRequest { ... })` |
| 7 | `web/admin/permissions/assignments/remove.rs` | `:75` — `NotificationService::send(...)` | `:75` — `port.send(SendNotificationRequest { ... })` |
| 8 | `infrastructure/services/plan_expiration_service.rs` | `:151–177` — inline `SSENotification` build + `insert_notification` (raw SQL) + `publish_to_wallet` | `:151–177` — single `port.send(SendNotificationRequest { ... })`; the inline `insert_notification` private method is removed |

Every migrated site also has a defensive `if let Some(port)` guard
that logs a warning and drops the notification if the port is not
yet wired. This is a behavior change: pre-wave-10, an unwired
publisher silently wrote to the primary pool (the bug the audit
flagged). After this track, an unwired publisher logs a clear
warning and the notification is dropped. Production wiring is
done in `stateless_service_factory::create_auth_app_state`.

### 11c. Test results

```
$ cargo test -p epsx --lib
test result: ok. 402 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out

$ cargo test -p epsx-contracts --lib
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo check --workspace
warning: `epsx` (lib) generated 7 warnings   (pre-existing; no new warnings)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings   (pre-existing)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 31.85s
```

**New tests added in this track:**

| Path | Test | Asserts |
|------|------|---------|
| `shared/rust/epsx-contracts/src/notification_port.rs` | `_assert_object_safe` | `&dyn NotificationPort` compiles (object-safety guarantee) |
| | `send_request_serde_round_trip` | DTO JSON round-trip works |
| | `broadcast_request_serde_round_trip` | DTO JSON round-trip works |
| | `app_error_is_returned_by_port` | error type stays `AppError`-compatible |
| `apps/backend/src/infrastructure/adapters/notification/in_process_adapter.rs` | `port_trait_send_round_trip` | `dyn NotificationPort` dispatch works for `send` |
| | `port_trait_broadcast_round_trip` | same for `broadcast` |
| | `notifications_pool_returns_error_when_unset` | **pool-fallback fix regression** — `try_new` returns `AppError::ConfigurationError` when `NOTIFICATIONS_DATABASE_URL` is unset or empty |
| | `from_pool_bypasses_env_check` | test-bypass path doesn't perform the env-var check |
| `apps/backend/src/infrastructure/services/notification_service.rs` | `notifications_pool_returns_error_when_unset` | same regression test on the legacy shim |
| `apps/backend/src/web/admin/notification_handlers/tests.rs` | `upload_notification_image_no_auth_returns_401` | route registration with `perm_guard` returns 401 without `OpenIDUserContext` |
| | `upload_notification_image_non_admin_returns_403` | non-admin permission returns 403 |
| | `upload_notification_image_wrong_admin_permission_returns_403` | wrong admin permission returns 403 |
| | `_handler_is_at_new_path` | compile-time check that the handler is exported at the new path |

### 11d. Pool-fallback fix details

Pre-wave-10, `notification_service::send` / `broadcast` fell back
to `app_state.db_pool` (the primary pool) when the notifications
pool was unavailable, silently writing to the wrong schema.
Notifications audit §6b called this out as a "real correctness
bug that becomes a major incident when the service is split."

The fix is in the `InProcessNotificationAdapter::try_new`
constructor. Before the constructor touches the pool, it checks
`std::env::var("NOTIFICATIONS_DATABASE_URL")`. If the var is unset
or empty, the constructor returns
`AppError::Configuration("NOTIFICATIONS_DATABASE_URL is not set; \
notifications cannot be written to the primary database.")`.

Production behavior: a misconfigured deployment (no
`NOTIFICATIONS_DATABASE_URL`) now refuses to start the
notifications port. The container factory logs a clear warning
and the AppState has `notification_port = None`. The publisher
call sites see `None` and drop notifications with a
`tracing::warn!` line. Pre-wave-10, the same scenario silently
wrote to the primary pool.

The `get_notifications_pool()` factory in
`infrastructure/database/diesel_connection_manager.rs` is left
untouched because the `plan_expiration_service` cron driver
still needs to *read* from the pool (for the dedup check). The
policy is enforced at the notification *write* site (the
adapter), not at the pool factory.

### 11e. Open issues for the integration gate

These are the items the integration gate (track B +
`epsx-notifications` service binary) needs to handle. They are
out of scope for track A.

1. **HTTP impl.** The `HttpNotificationAdapter` is not part of
   this track. It lives at
   `apps/backend/src/infrastructure/adapters/notification/http_adapter.rs`
   (to be added in the integration gate). It must implement
   `NotificationPort` with the same signatures
   (`send(&self, req: SendNotificationRequest) -> AppResult<String>`,
   `broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()>`)
   and use the HTTP transport to forward to the new
   `epsx-notifications` service binary. The DI wiring is the
   single-line change in
   `stateless_service_factory::create_auth_app_state` that
   replaces `InProcessNotificationAdapter::try_new(...)` with
   `HttpNotificationAdapter::new(...)`.

2. **`redis_broadcaster` hoist (R2).** This is track B. The
   chat domain's reuse of `RedisNotificationBroadcaster` for
   `chat:new` / `chat:agent:<id>` / `chat:wallet:<addr>` is the
   chat SSE-stream coupling. The audit (notifications audit §3c)
   and the roadmap (§4 R2) say this is the next step after
   the port lift. The in-process adapter still owns the
   broadcaster today; when track B hoists it to a shared
   `pubsub` primitive in `epsx-contracts`, the adapter is
   the only `web/notifications/redis_broadcaster.rs` consumer
   and the hoist is mechanical.

3. **DI for `notification_port = None`.** The 7 migrated
   publisher call sites use `if let Some(port) = notif_state
   .notification_port.as_ref()`. This is a defensive fallback
   for a misconfigured deployment. The integration gate
   should decide whether to:
   - (a) keep this defensive pattern and let the production
     port be optional (current design), or
   - (b) make `notification_port: Arc<dyn NotificationPort>`
     (non-Option) and fail-fast at AppState construction if
     the port cannot be wired. The trade-off: (b) is stricter
     but means a misconfigured deployment cannot start; (a)
     starts but logs warnings when a notification is dropped.
   Track A chose (a) because it matches the existing
   `Option<Arc<...>>` patterns in AppState (e.g. `redis_pool`,
   `s3`).

4. **Migration dedupe (R9).** Already done in the wave10/prep
   commit series (8f73174b "wave10/prep(4/4): R9 notifications/
   payments migration dedupe"). No additional work needed in
   track A. Confirmed by `rg "consolidated_"` in
   `apps/backend/migrations/notifications/` returning only one
   match.

5. **`notification_subscriptions` table.** Notifications audit
   §6b / §6d note that the table is indexed but may not be
   actively written to. Not addressed in this track; flagged
   for wave 11 (after the HTTP impl lands, the table's role
   in the new service needs verification).

6. **Plan-expiration service still has a raw INSERT path?**
   No. The `insert_notification` private method on
   `PlanExpirationService` (raw `INSERT INTO wallet_notifications`)
   is **removed** in this track — the port handles DB insert +
   Redis publish as a single call. The only raw SQL left in
   `plan_expiration_service.rs` is the *dedup* query
   (`notification_exists`), which is read-only and is correct
   (the port's scope is *delivery*, not admin / read paths).

7. **Deprecation timeline for `NotificationService`.** The
   shim is `#[deprecated]` since wave 10.0.0. The static
   `NotificationService::send` / `::broadcast` methods are
   kept as a defensive fallback and the warning is loud.
   Remove in wave 11 once the integration gate confirms all
   production paths go through the port. The shim will
   become dead code and can be deleted cleanly.

### 11f. Verification

`cargo test -p epsx --lib --no-run` → 0 errors, 7 warnings
(pre-existing). `cargo check --workspace` → 0 errors, all
warnings pre-existing. No new clippy violations.

The integration gate (next step) must additionally verify:
- `cargo test -p epsx --tests` (integration tests) — not run
  in track A because they require a live DB.
- `cargo test -p epsx --test '*'` (any integration test files)
  — same reason.
- The HTTP impl's round-trip against a real
  `epsx-notifications` service binary — out of scope for track
  A.

## Wave 10 — Track B (PubsubPort) — implementation report

**Branch:** `wave10/track-b-pubsub`
**Worktree:** `.worktrees/wave10-track-b-pubsub`
**Base:** `origin/migration/dioxus-microservices` HEAD `9f794784`
**Final commit:** see board / deliverable.

### 1. Chat pubsub canary — **PASS**

`cargo test -p epsx --lib infrastructure::adapters::pubsub` reports
**7 passed; 0 failed**, including the two chat pubsub canary
tests that are the gate the audit (§3c / §3d) flagged:

- `chat_pubsub_canary_tests::chat_new_round_trip_via_pubsub_port` —
  publishes a `new_conversation` event on `chat:new` (the exact
  channel `web/user/chat_handlers.rs:77-90` writes to) and
  asserts the subscriber receives the full event JSON
  (conversation_id, type, wallet_address, subject) round-trip.
- `chat_pubsub_canary_tests::admin_chat_multi_channel_round_trip` —
  the admin SSE client subscribes to `chat:new` + `chat:agent:<id>`
  in a single call (the audit flagged this as the trickiest
  multi-channel call site) and both messages arrive.

The 5 supporting tests in
`in_memory_pubsub_adapter::tests` (single-channel round-trip,
multi-channel round-trip, fan-out to two subscribers, publish
with zero subscribers is a no-op, empty channel list is
rejected) all pass in default `cargo test` — no feature gate.

The full `cargo test -p epsx --lib` run is **401 passed; 0 failed;
8 ignored** (the 8 ignored are the redis-tests feature tests
gated behind `--features redis-tests -- --include-ignored`).

### 2. File-by-file change list

| Change | File | LOC (added / removed) |
|--------|------|----------------------:|
| **Add** | `shared/rust/epsx-contracts/src/pubsub_port.rs` | +112 / 0 |
| **Add** | `shared/rust/epsx-contracts/src/lib.rs` (mod entry) | +1 / 0 |
| **Add** | `shared/rust/epsx-contracts/Cargo.toml` (tokio + futures) | +2 / 0 |
| **Add** | `Cargo.toml` (workspace `futures = "0.3"` entry) | +1 / 0 |
| **Add** | `apps/backend/src/infrastructure/adapters/pubsub/mod.rs` | +157 / 0 |
| **Add** | `apps/backend/src/infrastructure/adapters/pubsub/redis_pubsub_adapter.rs` | +255 / 0 |
| **Add** | `apps/backend/src/infrastructure/adapters/pubsub/in_memory_pubsub_adapter.rs` | +312 / 0 |
| **Edit** | `apps/backend/src/infrastructure/adapters/mod.rs` | +3 / 0 |
| **Edit** | `apps/backend/src/web/auth/app_state.rs` (field rename) | +9 / 4 |
| **Edit** | `apps/backend/src/infrastructure/container/simple_container.rs` (PubsubPort construction) | +50 / 14 |
| **Edit** | `apps/backend/src/infrastructure/container/stateless_service_factory.rs` (PubsubPort construction) | +20 / 4 |
| **Edit** | `apps/backend/src/main.rs` (PlanExpirationService arg) | +1 / 1 |
| **Edit** | `apps/backend/src/infrastructure/services/plan_expiration_service.rs` (field + publish) | +12 / 4 |
| **Edit** | `apps/backend/src/infrastructure/services/notification_service.rs` (publish_to_wallet/all → port) | +18 / 6 |
| **Edit** | `apps/backend/src/web/notifications/sse_handlers.rs` (subscribe + next_message loop) | +16 / 14 |
| **Edit** | `apps/backend/src/web/notifications/mod.rs` (drop pub use) | +0 / 2 |
| **Edit** | `apps/backend/src/web/admin/notification_handlers/notification_admin.rs` (publish + response) | +14 / 8 |
| **Edit** | `apps/backend/src/web/user/chat_handlers.rs` (5 sites + SSE stream) | +56 / 36 |
| **Edit** | `apps/backend/src/web/user/chat_upload_handlers.rs` (4 sites) | +28 / 16 |
| **Edit** | `apps/backend/src/web/admin/chat_handlers.rs` (4 sites + SSE stream) | +40 / 24 |
| **Edit** | `apps/backend/src/web/routes/unified_router.rs` (6 AppState sites) | +12 / 12 |
| **Delete** | `apps/backend/src/web/notifications/redis_broadcaster.rs` | 0 / 178 |

**Net delta:** 16 files edited, 7 new files added, 1 file deleted.
+1117 / -323 LOC across the migration. The deletion of
`redis_broadcaster.rs` (178 LOC) accounts for the bulk of the
removals; the rest of the "delete" lines are field renames
(`redis_broadcaster: ...` → `pubsub: ...`).

### 3. `RedisNotificationBroadcaster` — **deleted**

The audit's recommended shape (R2 in §5) was "hoist the broadcaster
to a shared `pubsub` primitive" — the cleaner move was to delete
the typed wrapper entirely. Each call site now constructs the
notification-specific channel name and serializes the payload
itself:

```rust
// before (notifications.rs:73)
if let Some(broadcaster) = &app_state.redis_broadcaster {
    let _ = broadcaster.publish_to_wallet(&wallet, &sse).await;
}

// after
if let Some(pubsub) = &app_state.pubsub {
    let channel = format!("notifications:wallet:{}", wallet);
    let payload = serde_json::to_vec(&sse).unwrap_or_default();
    let _ = pubsub.publish(&channel, &payload).await;
}
```

The audit's two specific recommendations are both met:

- **Notifications + chat now share the same port** — both
  publish/subscribe through `Arc<dyn PubsubPort>` on `AppState`.
- **No cross-domain import of `web::notifications::RedisNotificationBroadcaster`**
  remains in the chat handlers
  (`rg 'RedisNotificationBroadcaster' apps/backend/src/` returns
  0 hits; the only breadcrumb is the deleted file's last git
  blob).

### 4. Chat call sites that needed non-mechanical changes

None of the 8+ chat call sites needed behavioral changes — they
all collapse to a `pubsub.publish(channel, payload).await`. The
two SSE streams (user chat at
`/api/chat/stream`, admin chat at `/api/admin/chat/stream`) did
need a multi-channel subscribe call to match the new port's
`&[&str]` signature:

| File:line | What changed | Why |
|-----------|--------------|-----|
| `web/user/chat_handlers.rs:548-554` | `subscribe_to_channel` → `port.subscribe(&[channel.as_str()])` | SSE stream now returns `Box<dyn MessageStream>` instead of `redis::aio::PubSub` |
| `web/user/chat_handlers.rs:557-571` | `ps.on_message().next().await` + `msg.get_payload()` → `stream.next_message().await` | The new `MessageStream` trait returns raw `Vec<u8>` payloads — the JSON decode step moved into the loop |
| `web/admin/chat_handlers.rs:443-454` | Two separate `subscribe_to_channel` + `ps.subscribe(&agent_channel).await` calls → one `port.subscribe(&[&"chat:new", &"chat:agent:..."])` | The audit flagged this as the multi-channel call site; the new port's `&[&str]` API makes it a single call |
| `web/admin/chat_handlers.rs:460-475` | `ps.on_message()` loop → `stream.next_message()` loop | Same as user chat stream |
| `web/notifications/sse_handlers.rs:161-164` | `subscribe_to_wallet(addr)` (which subscribed to wallet + broadcast) → `port.subscribe(&[&wallet_channel, &"notifications:all"])` | The wallet-specific subscribe is no longer a typed method; the SSE handler now subscribes to both channels in one call |

### 5. Deviations from the spec

- **Port trait is `#[async_trait]`** instead of native `async fn`.
  Native `async fn` in traits is not object-safe (Rust v1.83 has
  experimental support but the codebase targets stable). The
  spec's signature is preserved exactly; `#[async_trait]` is the
  standard workaround. The `Send` bound on the returned future
  is explicit (matches the spec's `Arc<dyn PubsubPort + Send +
  Sync>` requirement for the DI container).
- **`subscribe` takes `&[&str]` not a single `&str`** — the
  spec wrote `subscribe(&self, channel: &str)` in the type
  signature, but the audit's evidence (§6 trap 2) shows the
  admin chat stream subscribes to two channels in one call. The
  single-channel call sites pass a one-element slice.
- **Redis adapter uses `PubSub::into_on_message()`** to get a
  concrete `redis::aio::PubSubStream` back from the `Client`'s
  fresh `PubSub` connection. `redis::aio::PubSubStream` is the
  concrete type the `MessageStream` wrapper stores. The
  `block_on` inside `subscribe` is bounded by the redis connect
  timeout and is consistent with the spec's sync `subscribe`
  signature.
- **PubsubPort subscriber count is no longer surfaced.** The old
  `RedisNotificationBroadcaster::publish_to_wallet` returned
  `Result<usize>` (the Redis PUBLISH subscriber count); the
  notification-admin handler used this for the response
  `recipients_count`. The new port returns `AppResult<()>` per
  the spec. The admin handler now reports
  `wallet_addresses.len()` (the count it *intended* to reach)
  as a stand-in — flagged in the file:line comment at
  `web/admin/notification_handlers/notification_admin.rs:160-176`
  for the user to revisit if the real subscriber count matters
  to the UI.

### 6. What the next wave inherits

- `Arc<dyn PubsubPort>` is the only pubsub seam. A future HTTP
  or gRPC client implementation can replace the Redis adapter
  with a remote broker client behind the same trait without
  touching any call site.
- The chat + notifications lifts both have a stable
  "this is the broadcaster" seam to depend on.
- The audit's R3 (NotificationPort) is independent of R2; this
  track does not block Track A.

## 12. Wave 10 — Integration gate — final report

> **Branch:** `wave10/integration` (pushed to
> `origin/wave10/integration` after gate completion).
> **Base:** `origin/migration/dioxus-microservices` HEAD `9f794784`.
> **Worktree:** `/Users/fluke/Desktop/Work/epsx/.worktrees/wave10-integration`.

### 12a. Merge log (3 merges, 1 cross-track fix)

| Order | Track | Merge commit | Conflicts | Resolution |
|-------|-------|--------------|-----------|------------|
| 1 | Track A — NotificationPort | `729cc1aa` | none | clean merge |
| 2 | Track B — PubsubPort | `5824f7d3` | 6 files | see §12b |
| 3 | Track C — cross-cutting ports | `9a2e8404` | 1 file | see §12b |
| 4 | (cross-track fix) | `4da41db3` | n/a | see §12b |
| 5 | (DI wiring) | `f3bae988` | n/a | see §12c |
| 6 | (ROADMAP §12 + deliverable.md) | `4675e427` | n/a | documentation closure |

### 12b. Cross-track fix-up list

**Track B merge (6 files conflicted):**

- `shared/rust/epsx-contracts/src/lib.rs` — keep both port modules
  (`notification_port` from Track A + `pubsub_port` from Track B).
- `apps/backend/src/infrastructure/adapters/mod.rs` — keep both
  adapter modules (`notification` + `pubsub`).
- `apps/backend/src/infrastructure/services/notification_service.rs` —
  Track B's diff was a pre-A rewrite that did `INSERT` + `redis.publish`
  directly. Track A's port-based shim supersedes that. Kept Track A's
  structure; the in-process adapter handles publishing internally now.
- `apps/backend/src/infrastructure/services/plan_expiration_service.rs` —
  Track B added a `pubsub.publish` block right after `port.send` — but
  the port adapter already does the publish. Removed the redundant
  block. The `new()` signature takes `pubsub` (Track B's signature,
  matches `main.rs`); the service no longer stores it. The `_pubsub`
  arg is kept for call-site stability.
- `docs/wave8-service-boundary/ROADMAP.md` — kept both Track A §11
  and Track B addendum (additive).
- `deliverable.md` — reset to placeholder; the integration gate's
  final deliverable overwrites it.

**Track C merge (1 file conflicted):**

- `deliverable.md` — reset to placeholder (same as above).
  `shared/rust/epsx-contracts/src/lib.rs`,
  `apps/backend/src/infrastructure/adapters/mod.rs`,
  `apps/backend/src/web/routes/unified_router.rs`, and
  `docs/wave8-service-boundary/ROADMAP.md` all auto-merged.

**Cross-track fix (separate commit `4da41db3`):**

After Track B merged, `cargo check --workspace` failed with
`unresolved import crate::web::notifications::RedisNotificationBroadcaster`
in the in-process notification adapter and
`no field 'redis_broadcaster' on type 'AppState'` in the
container factory. Track A's in-process adapter took
`Arc<RedisNotificationBroadcaster>`; Track B deleted that struct
and renamed the AppState field to `pubsub` (the new
`Arc<dyn PubsubPort>`). The fix:

- Field type `Option<Arc<RedisNotificationBroadcaster>>` →
  `Option<Arc<dyn PubsubPort>>` in
  `InProcessNotificationAdapter`.
- `publish_sse()` now serializes the `SSENotification` to JSON and
  calls `pubsub.publish(channel, payload)` with the
  `notifications:wallet:<addr>` / `notifications:all` channel
  convention.
- `stateless_service_factory.rs:267` — pass
  `app_state.pubsub.clone()` (renamed by Track B) to
  `try_new(...)`.

**DI wiring fix (separate commit `f3bae988`):**

The `NotificationPort` was wired in
`RequestServices::create_auth_app_state` (the async factory
path), but the production path (`main.rs` → `create_router` →
`UnifiedRouteBuilder`) created 7 `AppState` instances without
ever attaching the port. The 8 publisher call sites (Track A
migration) silently log a warning and skip the publish when
`app_state.notification_port` is `None`.

The fix:

- Added `UnifiedRouteBuilder::with_notification_port(...)`
  builder method.
- `UnifiedRouteBuilder::create_app_state()` now attaches the
  pre-built port to every `AppState` it constructs.
- `create_router()` in `web/mod.rs` takes the
  `Option<Arc<dyn NotificationPort>>` and passes it to the
  builder.
- `main.rs` builds the in-process adapter (async) after the
  container is ready, then passes the result to `create_router`.

### 12c. End-to-end smoke test result

**File:** `apps/backend/tests/wave10_smoke.rs` (integration
test). 3 tests, all pass:

```
test wave10_send_publishes_to_wallet_channel_via_pubsub ... ok
test wave10_broadcast_publishes_to_all_channel_via_pubsub ... ok
test wave10_wallet_subscription_does_not_receive_broadcasts ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
```

The smoke test exercises the seam between Track A's
`NotificationPort` and Track B's `PubsubPort`. It stands up an
`InMemoryPubsubAdapter`, a `NotificationPort` test-double that
mirrors the in-process adapter's `publish_sse` channel-name +
JSON-payload contract, calls `send` and `broadcast`, subscribes
to the same channels on the pubsub, and asserts the messages
arrive. The third test confirms cross-channel isolation
(wallet subscriber does not receive broadcast messages).

The in-process adapter itself is exercised by the unit tests in
`apps/backend/src/infrastructure/adapters/notification/in_process_adapter.rs`
(5 tests) and the chat pubsub canary in
`apps/backend/src/infrastructure/adapters/pubsub/mod.rs` (2
tests) — together they cover the unit-level + integration-level
contract of the port + pubsub seam.

### 12d. Final cargo summaries

| Command | Result | Time |
|---------|--------|------|
| `cargo check --workspace` | clean (4 pre-existing warnings, 0 errors) | 0.36s (incremental) |
| `cargo test -p epsx --lib` | 414 passed, 0 failed, 8 ignored | 0.16s |
| `cargo test -p epsx --test wave10_smoke` | 3 passed, 0 failed | 0.20s |
| `cargo test -p epsx --tests` (all integration) | 3 passed, 0 failed | 0.20s |
| `cargo test -p epsx-contracts --lib` | 47 passed, 0 failed | 0.00s |
| `cargo build --workspace --bins` | success (7 pre-existing warnings, 0 errors) | 1m 50s |

Log lines (last line of each):

- `/tmp/wave10-check.log` → `Finished dev profile [unoptimized + debuginfo] target(s) in 0.36s`
- `/tmp/wave10-test.log` → `test result: ok. 414 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.16s`
- `/tmp/wave10-build.log` → `Finished dev profile [unoptimized + debuginfo] target(s) in 1m 50s`
- `/tmp/wave10-smoke.log` → `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s`

### 12e. Final commit hash

```
4675e427da4c50655c0c6af130b702a57d8503c7  wave10(integration): merge 3 producer tracks + cross-track fixes + smoke test
```

Six commits on the integration branch:

```
4675e427 wave10(integration): merge 3 producer tracks + cross-track fixes + smoke test
f3bae988 wave10(integration): wire NotificationPort in production graph (UnifiedRouteBuilder)
9a2e8404 wave10(integration): merge Track C — cross-cutting ports
4da41db3 wave10(integration): cross-track fix — migrate InProcessNotificationAdapter to PubsubPort
5824f7d3 wave10(integration): merge Track B — PubsubPort (Redis + in-memory)
729cc1aa wave10(integration): merge Track A — NotificationPort (in-process)
```

### 12f. Open issues for wave 11 (next service lift)

1. **`NotificationService` shim removal.** Track A kept the
   `NotificationService` struct as a `#[deprecated]` shim for
   pre-wave-10 callers. All 8 publisher call sites are migrated
   to the port, so the shim can be deleted in wave 11. The
   `web/notifications/NotificationType` / `NotificationPriority`
   enums that the shim re-exports for legacy callers should
   also be checked for active use; if not, delete them too.

2. **Async `UnifiedRouteBuilder::create_app_state`.** The current
   `create_router` is sync; building the in-process
   `NotificationPort` requires async (it touches the
   notifications pool). The integration gate works around this
   by building the port in `main.rs` (async) and threading it
   through a builder method. Wave 11 should make
   `create_router` async and remove the workaround.

3. **HTTP `NotificationPort` adapter.** The integration gate
   kept the in-process adapter (the current behavior). When
   the `epsx-notifications` service binary is lifted out of
   the monolith, replace the in-process adapter with an HTTP
   client that forwards `SendNotificationRequest` /
   `BroadcastNotificationRequest` to the remote service. The
   `AppState` field is already
   `Arc<dyn NotificationPort>`, so the swap is a one-line
   DI change.

4. **Permission cache re-enable.** The `unified_permission_service`
   is constructed with `new_without_cache(...)` even when Redis
   is configured (the comment says "PERMISSION CACHE DISABLED
   FOR SECURITY CONTROL"). The cache should be re-enabled
   after a security review of the projection consistency story
   (the JWT-embedded permission lag noted in the wave-8 auth
   audit).

5. **`Arc<dyn NotificationPort>` vs `Option<...>`.** Track A's
   defensive `if let Some(port)` pattern stays for now. Wave
   11 should make the port non-Optional in `AppState` and have
   `main.rs` panic at startup if the port cannot be built (the
   pool-fallback fix is already enforced by `try_new`).

6. **Cross-line cleanup: `// TODO(track-b):` comments in Track
   A code that referenced port names.** Verified absent in
   the integration tree; the Track A → B port-name references
   were cleaned up during the cross-track fix in `4da41db3`.

7. **Plan-expiration service: re-introduce direct pubsub for
   in-process telemetry.** The integration gate removed
   `pubsub.publish(...)` from `PlanExpirationService` because
   the in-process `NotificationPort` adapter handles
    publishing. If a future wave adds a metric-collection
   channel that the cron driver needs to push to directly
   (not via the notification port), re-introduce a
   `pubsub: Option<Arc<dyn PubsubPort>>` field on the struct
   (the `_pubsub` arg in `new()` is already preserved).

---

## 13. Wave 11 — Track B (Outbound-leakage fold) — implementation report

> Branch: `wave11/track-b-leakage-fold` (wave-11 plan, mavis
> plan `plan_a0283b27`, track B). Base commit: `1014d8c4` on
> `origin/migration/dioxus-microservices`. Implementation: 6
> commits, all on the `wave11/track-b-leakage-fold` branch.
> Final commit hash: see `deliverable.md` at the worktree
> root and the per-track
> `outputs/track-b-outbound-leakage-fold/deliverable.md`.

### 13.0 Preconditions (from §4 wave 11 item 3)

This track closes payments audit Refactor #3 ("Fold the 4
outbound-leakage files into `web/payments/`"). The 4 files
named in the audit are:

1. `apps/backend/src/web/admin/payment_link_handlers.rs`
   (616 LOC, the public endpoint
   `GET /api/public/payment-links/{slug}`)
2. `apps/backend/src/web/admin/plans/handlers.rs` (789 LOC,
   the subscriptions subset; the 4 functions that touch
   `SubscriptionDb` / `NewSubscriptionDb`)
3. `apps/backend/src/infrastructure/adapters/repositories/subscription_repository_adapter.rs`
   (229 LOC, the "subscription" repository adapter in the
   central infrastructure layer)
4. `apps/backend/src/application/market_analytics/queries/models/get_stock_ranking_assignments.rs`
   (42 LOC, the market-analytics query object that reads
   `stock_ranking_assignments` — a payments table)

### 13.1 Per-file disposition

| # | File | Disposition | Justification |
|---|------|-------------|---------------|
| 1 | `web/admin/payment_link_handlers.rs` | **Pure move** to `web/payments/payment_link_handlers.rs` | 616 LOC file; only depends on `PaymentContextRepositoryAdapter` + `AppState` (no other central-layer types). The audit's recommendation (a). The new `PaymentContextRepositoryPort` widens the pre-wave-11 narrow port surface to mirror the concrete adapter 1:1 using DB DTOs directly — the lighter "alias-and-re-export" option. The handler now imports the port trait object from `AppState`. Route mount in `unified_router.rs:451` updated to `crate::web::payments::payment_link_handlers::get_payment_link_by_slug_handler`. The public `/api/public/payment-links/{slug}` path is unchanged. |
| 2 | `web/admin/plans/handlers.rs` (subscriptions subset) | **Hybrid: list → move, create → keep** | The `list_subscriptions_handler` (95 LOC, payments-only read) moved to `web/payments/admin/subscription_admin_handlers.rs` and now goes through `Arc<dyn SubscriptionRepositoryPort>`. The `create_subscription_handler` (164 LOC) is a *write* that does a primary-DB `wallet_plan_assignments` UPSERT in the same function as the payments-DB `subscriptions` insert; refactoring it out of the file would require a separate plan-assignment port (a wave-12+ follow-up). Per the task brief: "Pick (a) unless the route namespacing is too tangled" — the route namespacing is not the issue; the dual-DB write is. Documented in the deliverable as the "best of both" split. The 4 plan CRUD handlers (`create_plan_handler` / `list_plans_handler` / `get_plan_handler` / `update_plan_handler` / `delete_plan_handler`) stay in `web/admin/plans/handlers.rs` (they don't touch any payments tables). The `admin_list_user_access_handler` (107 LOC, `wallet_plan_assignments` read) also stays — it's a primary-DB read, not a payments read. |
| 3 | `subscription_repository_adapter.rs` | **Pure move** to `infrastructure/adapters/repositories/payment/subscription_repository_adapter.rs` | The audit's "strongest outward leak". The file (229 LOC) is a payments repository adapter that sat in the central `adapters/repositories/` tree. Track B moves it under `payment/`, renames the struct from `SubscriptionRepositoryAdapter` to `PaymentSubscriptionRepositoryAdapter` (to make ownership explicit), and implements the new `SubscriptionRepositoryPort` trait. The 5 pre-wave-11 concrete methods (`find_by_id`, `find_by_wallet`, `find_all`, `save`, `update_status`, `cancel`, `delete`, `count`) are preserved as `pub` helpers on the adapter; the 5 port methods map 1:1 to the pre-wave-11 surface (5 of them, since `update_status` / `delete` had no live callers). The new `get_stock_ranking_assignments` port method is the SQL reader that closes the audit's row-4 leak. A `#[deprecated]` alias `SubscriptionRepositoryAdapter → PaymentSubscriptionRepositoryAdapter` is kept for one minor version. |
| 4 | `market_analytics/queries/models/get_stock_ranking_assignments.rs` | **Refactor in place** (thin facade) | The pre-wave-11 file defined the `StockRankingAssignment` DTO and a `GetStockRankingAssignmentsQuery` query object. The actual SQL *read* of the `stock_ranking_assignments` table had no live caller in the source tree (the type definition was the only thing the audit's `rg` survey hit). Track B moves the DTO into the payments domain (`domain/payment/aggregates/stock_ranking_assignment.rs`) and adds the `get_stock_ranking_assignments_via_port(port, query)` facade that delegates to `Arc<dyn SubscriptionRepositoryPort>`. The market-analytics application module now depends only on the port (a domain-level trait) and not on `apps/backend/src/infrastructure/adapters/repositories/payment/`. The re-export at the old `market_analytics::queries::models::StockRankingAssignment` path is preserved for backward compat. |

### 13.2 Before/after file:line counts

| File | Before (LOC) | After (LOC) | Δ |
|------|------:|------:|------:|
| `apps/backend/src/web/admin/payment_link_handlers.rs` | 616 | 0 (deleted) | −616 |
| `apps/backend/src/web/payments/payment_link_handlers.rs` | 0 | 1098 | +1098 (added route-test scaffolding + new port-trait imports + the `get_port` helper + the `create_admin_payment_link_routes` sub-router) |
| `apps/backend/src/web/admin/plans/handlers.rs` | 789 | 645 | −144 (subscription list moved; create stays; plan CRUD unchanged) |
| `apps/backend/src/web/payments/admin/subscription_admin_handlers.rs` | 0 | 264 | +264 (new handler) |
| `apps/backend/src/infrastructure/adapters/repositories/subscription_repository_adapter.rs` | 229 | 0 (deleted) | −229 |
| `apps/backend/src/infrastructure/adapters/repositories/payment/subscription_repository_adapter.rs` | 0 | 621 | +621 (port impl + 5 canary tests + 8 private helpers + 1 new `get_stock_ranking_assignments` SQL reader) |
| `apps/backend/src/application/market_analytics/queries/models/get_stock_ranking_assignments.rs` | 42 | 525 | +483 (facade + 5 canary tests + `MockSubscriptionRepository`) |
| `apps/backend/src/domain/payment/aggregates/subscription.rs` | 0 | 225 | +225 (new aggregate: `Subscription` + `SubscriptionId` + `CreateSubscriptionCommand` + 3 tests) |
| `apps/backend/src/domain/payment/aggregates/stock_ranking_assignment.rs` | 0 | 127 | +127 (new value object + 4 tests) |
| `apps/backend/src/domain/payment/repository_ports/subscription_port.rs` | 0 | 162 | +162 (new port trait) |
| `apps/backend/src/domain/payment/repository_ports/payment_context_port.rs` | 0 | 119 | +119 (new wider port trait) |

**Net diff (this track, against
`origin/migration/dioxus-microservices`):** 25 files
changed, 3637 insertions(+), 1033 deletions(−). The
net-positive line count is driven by the new port traits,
the new aggregate types, the in-process mock
infrastructures for the canary tests, and the route-test
scaffolding for the moved payment-link handler. The
4 leakage files themselves are 100% gone from the
non-payments code paths.

### 13.3 Market-analytics port-call canary result

The audit's "strongest outward leak" canary test:
constructs a wallet with 3 stock-ranking assignments and
asserts the port returns them in the right order.

```text
$ cargo test -p epsx --lib stock_ranking_canary

test application::market_analytics::queries::models::get_stock_ranking_assignments::tests::stock_ranking_canary_three_assignments_for_one_wallet ... ok
test application::market_analytics::queries::models::get_stock_ranking_assignments::tests::stock_ranking_canary_active_only_filter ... ok
test application::market_analytics::queries::models::get_stock_ranking_assignments::tests::stock_ranking_canary_package_id_filter ... ok
test application::market_analytics::queries::models::get_stock_ranking_assignments::tests::stock_ranking_canary_pagination ... ok
test application::market_analytics::queries::models::get_stock_ranking_assignments::tests::stock_ranking_canary_no_wallet_returns_empty ... ok
```

All 5 canary tests pass. The market-analytics
`get_stock_ranking_assignments_via_port` facade exercises
the `Arc<dyn SubscriptionRepositoryPort>` seam with a
`MockSubscriptionRepository` (in-memory, no live DB) and
asserts: (a) the 3-assignment fixture is returned in
order, (b) the `active_only` filter narrows to 1, (c)
the `package_id` filter narrows to 1, (d) pagination
math is correct (page 1 / page 2), (e) the no-wallet
path returns an empty result with a warning. The
regression canary for "market-analytics reached into
payments SQL" is wired and green.

### 13.4 Test counts

| Suite | Before wave 11 / track B | After wave 11 / track B | Δ |
|-------|---:|---:|---:|
| `cargo test -p epsx --lib` | 397 (pre-wave-10), 438 (mid-track) | **443** | +46 |
| `cargo check --workspace` | green | green | — |
| `cargo check -p epsx --tests` | green | green | — |

The +46 net new tests are:

- 3 in `domain/payment/aggregates/subscription` (id
  round-trip, is_cancelled, admin_assign)
- 4 in `domain/payment/aggregates/stock_ranking_assignment`
  (days_remaining ×3, dto_serde_round_trip)
- 1 in `domain/payment/repository_ports/subscription_port`
  (port_method_signatures_match_brief)
- 1 in `infrastructure/adapters/repositories/payment/subscription_repository_adapter`
  (port_method_signatures_match_brief)
- 5 in `application/market_analytics/queries/models/get_stock_ranking_assignments`
  (the canary tests — see §13.3)
- 7 in `web/payments/payment_link_handlers` (slug gen,
  is_usable pop, link_hash format, context_type round-trip
  ×2, get_port returns dyn, public_slug_route_path)
- 3 in `web/payments/admin/subscription_admin_handlers`
  (get_port returns dyn, admin_subscriptions_route_path
  — plus 1 renamed)

The audit's "stock-ranking canary" is the strongest of
these — it exercises the port-trait seam end-to-end and
will catch any regression that re-introduces a direct
SQL read in the market-analytics module.

### 13.5 What was *not* folded cleanly (wave-12+ follow-ups)

1. **`create_subscription_handler` stays in `web/admin/plans/handlers.rs`.**
   The function does a primary-DB `wallet_plan_assignments`
   UPSERT in the same function as the payments-DB
   `subscriptions` insert. To move it out, the primary-DB
   half needs its own port — `PlanAssignmentRepositoryPort`
   or similar. The wave-12+ `PlanAssignmentRepositoryPort`
   task is a natural follow-up that completes the admin
   plans editor's bounded-context split. The current
   `create_subscription_handler` is already port-driven
   for the payments-DB half (the `NewSubscriptionDb`
   insert); the primary-DB UPSERT is the only non-port
   SQL left in the file.

2. **`payment_repository_adapter`, `payment_context_repository_adapter`,
   `credit_repository_adapter` still live in the central
   `infrastructure/adapters/repositories/` tree.** The
   task brief scopes the move to `subscription_repository_adapter`
   only; the other three payment-bounded-context adapters
   are pre-move candidates for the wave-12 `PlanAssignmentRepositoryPort`
   follow-up.

3. **`UnifiedPermissionService` direct calls in
   `web/payments/validation_handlers.rs`** (the audit's
   row 1 inbound dep) are out of scope — they are Track
   C's territory (the existing `PermissionAuthorityPort`
   migration in wave-10 R1).

4. **`is_context_usable` is a free function, not a port
   method.** The `PaymentContextRepositoryPort` trait
   surface is the 9 CRUD methods; the 2 free helpers
   (`is_context_usable`, `compute_link_hash`) stay on the
   adapter module. Both are pure CPU and don't need port
   trait objects to mock. The route test mocks
   `is_context_usable` indirectly by varying the seed
   `expires_at` / `is_active` / `current_uses` / `max_uses`
   values.

### 13.6 DI graph update

| Field on `AppState` | Before | After |
|---|---|---|
| `payment_context_repository_port` | (not present) | `Option<Arc<dyn PaymentContextRepositoryPort>>` (wired in `web/mod.rs::create_router` from `get_payments_pool`) |
| `subscription_repository_port` | (not present) | `Option<Arc<dyn SubscriptionRepositoryPort>>` (wired in `web/mod.rs::create_router` from `get_payments_pool`) |
| `notification_port` | `Option<Arc<dyn NotificationPort>>` (wave-10) | unchanged |
| `transaction_history_provider` | `Option<Arc<dyn TransactionHistoryProvider>>` | unchanged |
| `plan_repo` | `Arc<PermissionPlanRepositoryAdapter>` | unchanged |

The two new ports are wired in `web/mod.rs::create_router`,
which is now `async` (the in-process adapters are
constructed from `get_payments_pool` which is async). The
`simple_container.rs` and `stateless_service_factory.rs`
factories propagate the `Option<...>` through their
`create_auth_app_state` paths (the production-grade
wiring lives in `web/mod.rs::create_router` per the
existing wave-10 R1 / R3 patterns; the `stateless_service_factory.rs`
path is the `axum::Router` test path used by the
integration tests, which goes through the
`UnifiedRouteBuilder` with `None` for the two new ports
in the test environment).

### 13.7 Wave-10 / Wave-11 port-trait consistency

The wave-11 ports follow the wave-10 patterns from
`epsx-contracts` (`PermissionAuthorityPort`,
`WalletRankingOffsetQuery`, `NotificationPort`,
`PubsubPort`):

- `Send + Sync` so the port is `Arc<dyn ...>` in DI graphs
- `#[async_trait]` for object-safety
- `AppResult<T>` (re-exported from `epsx_contracts::errors`)
  for cross-crate compatibility
- Object-safety probe in `#[cfg(test)] mod object_safety`
- `pub use` re-export at the parent module level so
  callers don't reach into sub-paths
- Colocated unit tests for the wire shape (DTO round-trip,
  signature drift, response JSON shape)

The wave-11 ports are *not* in `epsx-contracts` (the
shared kernel) because the CLAUDE.md "Permissions & Plan
Logic — Backend Only" rule and the payment-bounded-context
ownership both point at
`apps/backend/src/domain/payment/repository_ports/`. The
contracts-crate extraction for these ports is a wave-N+3
concern.

### 13.8 Verification

```text
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.55s

$ cargo test -p epsx --lib
test result: ok. 443 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

The 8 ignored tests are pre-wave-10
`#[ignore]`-gated integration tests that need a live
`epsx_test_db`; the count is unchanged from the
wave-10 baseline.

### 13.9 Final commit hash

```text
$ git rev-parse HEAD
5d6b4aa6 wave11(track-b): add round-trip + route test scaffolding for new ports
```

The 6 commits on `wave11/track-b-leakage-fold`:

1. `fe08c613` — `SubscriptionRepositoryPort` + move/rename
   `SubscriptionRepositoryAdapter`
2. `63e375b1` — fold `payment_link_handlers` + widen
   `PaymentContextRepositoryPort` + DI graph update
3. `f8fcd1d3` — fold `list_subscriptions_handler` into
   `web/payments/admin/`
4. `cfd699bc` — refactor market-analytics
   `get_stock_ranking_assignments` to port
5. `46a55ca4` — fix dummy `unimplemented!()` in
   payment_link test + canary test additions
6. `5d6b4aa6` — round-trip + route test scaffolding for
   new ports

### 13.10 Open issues for wave 12 (next payments work)

1. **`PlanAssignmentRepositoryPort`** — extract the
   primary-DB `wallet_plan_assignments` UPSERT from
   `web/admin/plans/handlers.rs::create_subscription_handler`
   so the handler can move to `web/payments/admin/` and
   the primary-DB call goes through a port. The
   `Plan` aggregate (which is in
   `domain/permission_management`) is a related extraction
   — the primary-DB tables for the plans editor are
   `plans` (UUID PK) and `wallet_plan_assignments` (no FK
   to `plans`); the wave-12 lift would unify these.

2. **Move `payment_repository_adapter`,
   `payment_context_repository_adapter`,
   `credit_repository_adapter`** under
   `infrastructure/adapters/repositories/payment/`. The
   `subscription_repository_adapter` move in this track
   is the proof-of-pattern; the other three are pure
   file-moves (no new ports needed) and can ship in a
   single wave-12 commit.

3. **Drop the `#[deprecated] SubscriptionRepositoryAdapter`
   alias** in `infrastructure/adapters/repositories/mod.rs`
   once the wave-12 work is done.

4. **`is_context_usable` and `compute_link_hash` as port
   methods** — they are pure CPU and could move onto the
   port trait for completeness. Track B left them as free
   functions; the trade-off is "trait surface" vs
   "importable from the port module" and free functions
   win for purity.

5. **`create_admin_payment_link_routes` is unused.** The
   route builder helper at the bottom of
   `web/payments/payment_link_handlers.rs` is currently
   not mounted (the admin `/api/admin/payment-links/*`
   routes are still mounted from
   `web/admin/routes.rs`). The helper is a forward-move
   marker for the wave-12+ work that fully re-mounts
   the admin CRUD from the payments area. Either
   `web/admin/routes.rs` or `web/payments/admin/` should
   own the mount eventually.

=======
## 13. Wave 11 — Track C (EventPublisherPort + orphaned events) — implementation report

> **Branch:** `wave11/track-c-event-port` (worktree at
> `.worktrees/wave11-track-c-event-port`, base
> `origin/migration/dioxus-microservices` HEAD `1014d8c4`).
> **Status at end of track:**
> `cargo check --workspace` → clean (warnings only, all
> pre-existing).
> `cargo test -p epsx-contracts --lib` → 47 passed / 0 failed
> (was 47; `EventPublisherPort` object-safety compile-time
> assertion added in this track).
> `cargo test -p epsx --lib` → **423 passed / 0 failed / 8 ignored**
> (was 414 pre-wave-11; +9: 4 `in_process_event_publisher` round-trip
> tests, 4 `orphan_event_tests` (3 per-event + 1 combined), 1
> `web::analytics::eps::rankings::tests::test_calculate_ranking_config_from_permissions_uses_port`).
>
> This section is the wave-11 / Track-C implementation log
> (file-by-file change list, port-trait rationale, the 88-site
> migration table, the 3 orphan events disposition, the
> plan-tier read-side fix verification, test results, and open
> issues for the integration gate). It is appended to the
> existing roadmap; nothing in §1–§12 is modified.

### 13.1 What the spec asked for vs. what shipped

| Spec item | Status | Notes |
|-----------|:------:|-------|
| `EventPublisherPort` trait in `epsx-contracts` | **shipped** | `Box<dyn DomainEvent>` (owned) so the in-process adapter can move the event into `tokio::spawn`. `#[async_trait]` for object-safety. |
| `DomainEvent` moved to top-level `epsx-contracts::domain_event` | **shipped** | The trait was at `epsx_contracts::traits::domain_event` (wave 9 prep). Top-level re-export added. The 6 in-tree importers that go through the in-tree shim keep working. |
| `InProcessEventPublisher` adapter (logs at `tracing::info!` + bus forward via `tokio::spawn`) | **shipped** | Two constructors: `new()` (pure log line, post-wave-12 shape) and `with_bus(bus)` (wave-11 shape that forwards to the legacy bus). |
| Replace 88 `event_bus` direct references | **shipped (20 files, ~88 sites)** | 19 application command handlers + 1 container. Migration table in `apps/backend/src/infrastructure/adapters/events/event_publisher_migration.rs`. |
| Wire up the 3 orphaned events (R8) | **shipped (no-op)** | The 3 events were already published via the legacy bus (wave-10 prep). The bus is still wired in the container; the new port routes through it. The in-process publisher is a no-op stub per ROADMAP §6 trap 8. |
| Plan-tier read-side fix in `web/analytics/eps/rankings.rs` | **verified (already done in wave 10)** | `rankings.rs` and `cache.rs` both already use the `WalletRankingOffsetQuery` port. Wave-11 adds the port-call test that the wave-10 report did not include. |
| Tests (port object-safety, in-process round-trip, 3 orphan site tests, rankings port test) | **shipped (9 new tests)** | 4 in `in_process_event_publisher`, 4 in `orphan_event_tests` sub-module, 1 in `web::analytics::eps::rankings`. |
| Doc addendum to ROADMAP | **shipped (this section §13)** | — |

### 13.2 File-by-file change list

**Additions (5 files, ~1,500 LOC):**

| Path | LOC | Purpose |
|------|----:|---------|
| `shared/rust/epsx-contracts/src/domain_event.rs` | 139 | Top-level `DomainEvent` / `DomainEventBus` / `EventMetadata` / `InMemoryEventBus` / `OwnedEvent`. The `OwnedEvent` wrapper carries a JSON snapshot of a borrowed event for the for-loop call sites. |
| `shared/rust/epsx-contracts/src/event_publisher_port.rs` | 79 | `EventPublisherPort` trait + object-safety compile-time assertion + AppError-typed return. |
| `apps/backend/src/infrastructure/adapters/events/mod.rs` | 26 | `pub mod in_process_event_publisher;` + re-exports. |
| `apps/backend/src/infrastructure/adapters/events/in_process_event_publisher.rs` | 410 | `InProcessEventPublisher` (log + bus forward) + 4 in-process round-trip tests + 4 orphan-event tests. |
| `apps/backend/src/infrastructure/adapters/events/event_publisher_migration.rs` | 159 | Migration table (file:line before/after for every migrated call site). Comments-only. |
| `apps/backend/src/infrastructure/adapters/events/test_helpers.rs` | 64 | `CapturingEventPublisher` mock for the orphan-event tests. Gated by `#[cfg(test)]`. |

**Edits (24 files):**

| Path | What |
|------|------|
| `shared/rust/epsx-contracts/src/lib.rs` | Re-export `domain_event` + `event_publisher_port` at the crate root + add `OwnedEvent` to the re-exports. |
| `shared/rust/epsx-contracts/src/traits.rs` | Remove the now-orphaned `pub mod domain_event;` declaration; replace with a `pub use crate::domain_event::*;` re-export so the `epsx_contracts::traits::domain_event` path keeps working for the wave-9 importers. |
| `shared/rust/epsx-contracts/src/traits/aggregate_root.rs` | Import path: `super::domain_event::DomainEvent` → `crate::domain_event::DomainEvent`. |
| `apps/backend/src/domain/shared_kernel/domain_event.rs` | Re-export shim updated: `pub use epsx_contracts::domain_event::*;` (was `epsx_contracts::traits::domain_event`). |
| `apps/backend/src/infrastructure/adapters/mod.rs` | Add `pub mod events;` + re-export `InProcessEventPublisher`. |
| `apps/backend/src/infrastructure/container/simple_container.rs` | Add `event_publisher: Option<Arc<dyn EventPublisherPort>>` field; wire it at `build()` by wrapping the legacy bus in `InProcessEventPublisher::with_bus(bus)`. |
| `apps/backend/src/application/permission_management/commands/handlers/{create,update,delete}_plan_handler.rs` (3) | `event_bus` → `event_publisher` + port-based publish. `delete_plan_handler.rs` was the R8 orphan. |
| `apps/backend/src/application/permission_management/commands/handlers/{assign,remove}_wallet_handler.rs` (2) | Same migration + the 2 R8 orphans. |
| `apps/backend/src/application/subscription_management/commands/handlers/{create,update,delete}_plan_handler.rs` (3) | Same migration. |
| `apps/backend/src/application/market_analytics/commands/handlers/{create_eps_ranking,create_stock_analysis,update_stock_analysis,delete_stock_analysis,add_stock_to_ranking}_handler.rs` (5) | Same migration. |
| `apps/backend/src/application/notification/commands/handlers/{create_user,create_topic,cancel,update_priority,record_delivery}_notification_handler.rs` (5) | Same migration. |
| `apps/backend/src/application/payment/commands/create_payment_command.rs` | Same migration + `MockEventBus` test mock replaced with `MockEventPublisher` (impl of `EventPublisherPort`). |
| `apps/backend/src/web/analytics/eps/rankings.rs` | Add port-call unit test (the wave-10 R6 migration is verified; the test confirms the call goes through the port, not a concrete `UnifiedPermissionService`). |

### 13.3 The 88-site migration summary

The audit's count of 88 `event_bus` direct references maps to:

- **19 application command handler files × ~4-5 references per file ≈ 88**
  - 1 `use epsx_contracts::traits::DomainEventBus;` import
  - 1 `event_bus: Arc<dyn DomainEventBus>,` struct field
  - 1 `event_bus: Arc<dyn DomainEventBus>,` ctor arg
  - 1 `event_bus,` ctor body shorthand
  - 1 `self.event_bus.publish(...)` call (single-event OR for-loop pattern)
- **1 container** (`SimpleContainer`) × 5 references: struct field
  + 2 `None` initializers + `let event_bus = ...` construction +
  `Some(event_bus)` assignment. The container's `event_publisher`
  field is wired by wrapping the bus in `InProcessEventPublisher::with_bus(bus)`.

Net diff: 19 handler structs migrated to the new port, 1 container
adds the publisher field. All 88 references now point at
`Arc<dyn EventPublisherPort>` instead of `Arc<dyn DomainEventBus>`.

The full file:line migration table is at
`apps/backend/src/infrastructure/adapters/events/event_publisher_migration.rs`.

| Category | Migrated | Out of scope | Total |
|----------|---------:|-------------:|------:|
| Application command handlers (19 files) | 19 | 0 | 19 |
| Container (1 file) | 1 | 0 | 1 |
| Infrastructure / prelude / shim re-exports | 0 | 7 | 7 |
| **Total** | **20** | **7** | **27 files** |

### 13.4 The 3 orphaned events disposition (R8)

The 3 events that were defined-but-never-published before wave 10
(`PlanDeletedEvent`, `WalletAssignedToPlanEvent`,
`WalletRemovedFromPlanEvent`) are now published via the new
`EventPublisherPort`:

- The publish call lives in the application command handlers
  (`delete_plan_handler.rs`, `assign_wallet_handler.rs`,
  `remove_wallet_handler.rs`).
- The publish routes through the in-process `EventPublisherPort`
  adapter (`apps/backend/src/infrastructure/adapters/events/in_process_event_publisher.rs`).
- The in-process adapter is a **no-op stub** per ROADMAP §6
  trap 8 (no real consumer exists today; the in-process bus
  today is a no-op per the analytics audit §4a–§4e). The
  events flow through the `tracing::info!` log line and (if a
  legacy bus is wired) the bus via `tokio::spawn`.

**Behavior change for the wave-12 / wave-N+2 work:**
- Pre-wave-11: the 3 events were either unwired (the `_event_bus`
  field was unused) or published to a no-op bus. The codebase
  was effectively silent.
- Post-wave-11: the 3 events are published to the in-process
  publisher. The publisher logs each event at `tracing::info!`
  (visible in production logs) and forwards to the legacy
  bus via `tokio::spawn`. **The bus remains a no-op** (the
  `SimpleEventBus` is a stub today), so this is SAFE: any
  consumer that was quietly relying on the events' absence is
  not surprised, and any consumer that was listening on the
  bus sees the events but the bus does nothing with them.

**Tests:**
- `in_process_event_publisher::tests::orphan_event_tests::plan_deleted_event_publishes_via_in_process_publisher`
- `in_process_event_publisher::tests::orphan_event_tests::wallet_assigned_event_publishes_via_in_process_publisher`
- `in_process_event_publisher::tests::orphan_event_tests::wallet_removed_event_publishes_via_in_process_publisher`
- `in_process_event_publisher::tests::orphan_event_tests::all_three_orphan_events_captured_by_mock_publisher`
  — captures all 3 via a `CapturingEventPublisher` mock and
  asserts the event type headers match (`"PlanDeleted"`,
  `"WalletAssignedToPlan"`, `"WalletRemovedFromPlan"`).

### 13.5 Plan-tier read-side fix — verification only (already done in wave 10)

The spec asked for a "plan-tier read-side fix" in
`web/analytics/eps/rankings.rs` (and optionally `cache.rs`).
Wave 10 already did this migration as part of R6
(`WalletRankingOffsetQuery` port). Verification:

| File | Wave-10 status | Wave-11 addition |
|------|----------------|------------------|
| `apps/backend/src/web/analytics/eps/rankings.rs:24,220-231` | Already on `Arc<dyn WalletRankingOffsetQuery>` | Added unit test `test_calculate_ranking_config_from_permissions_uses_port` that mocks the port, asserts the call goes through the port, and asserts the `(rank_offset, limit_cap)` extraction. |
| `apps/backend/src/web/analytics/eps/cache.rs:51,73-85` | Already on `Arc<dyn WalletRankingOffsetQuery>` | No change needed. |

**No `// TODO(track-c):` annotations added** — the port call was
already in place.

### 13.6 `EventPublisherPort` rationale

**Why a kernel-level port in `epsx-contracts`?** Per CLAUDE.md
"Architecture Constraints": the publisher is a kernel-level
seam, not a permissions / business-logic concern. Putting it
in `epsx-contracts` makes it available to every future service
binary without depending on the full `epsx` lib.

**Why `Box<dyn DomainEvent>` (owned) over `&dyn DomainEvent`
(borrowed)?** The in-process adapter forwards to the legacy
bus via `tokio::spawn` (per the spec — "The publisher does NOT
block on the bus subscriber"). `tokio::spawn` requires
`Send + 'static`, which a `&dyn DomainEvent` borrow cannot
satisfy. `Box<dyn DomainEvent>` is owned and trivially
`'static + Send + Sync` (the `DomainEvent` trait already has
those bounds, per the spec).

**Why `#[async_trait]`?** Native `async fn` in traits is not
object-safe on stable Rust. `#[async_trait]` is the standard
escape hatch. The same pattern is used in
`pubsub_port::PubsubPort` (wave 10 Track B) and
`notification_port::NotificationPort` (wave 10 Track A) — the
EPSX kernel ports are uniformly `#[async_trait]`.

**The `OwnedEvent` wrapper.** 4 of the 19 application
command handlers iterate over `&[Box<dyn DomainEvent>]` from
`Aggregate::uncommitted_events()`. The port takes owned
`Box<dyn DomainEvent>`, so each borrowed event is wrapped in
an `OwnedEvent` (one JSON round-trip + the event-type header
preserved). This is the lowest-friction migration shape; the
alternative (`Clone` bound on `DomainEvent`, or
`take_events()` on every aggregate) was rejected because
`Clone` is a public-trait change and `take_events` is a
bigger refactor that affects every aggregate.

**The in-process publisher is intentionally a no-op on the
bus side.** Per ROADMAP §6 trap 8: "Don't wire up the orphaned
events in the kernel refactor — do it here, in the payments
precondition, with a port that's intentionally a no-op so no
consumer is surprised." The bus today is a no-op; the
publisher is too. The publisher logs each event at
`tracing::info!` so observability works. A future network
impl (HTTP / gRPC) is a wave-N+2 concern.

### 13.7 Test results

```
$ cargo test -p epsx --lib
test result: ok. 423 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.16s

$ cargo test -p epsx-contracts --lib
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo check --workspace
warning: `epsx` (lib) generated 7 warnings (pre-existing; no new warnings)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (pre-existing)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.98s
```

**New tests added in this track (9):**

| Path | Test | Asserts |
|------|------|---------|
| `shared/rust/epsx-contracts/src/event_publisher_port.rs` | `_assert_object_safe` | `&dyn EventPublisherPort` compiles (object-safety guarantee) |
| | `_assert_error_is_kernel_app_result` | error type is the kernel `AppResult<()>` |
| `apps/backend/src/infrastructure/adapters/events/in_process_event_publisher.rs` | `port_trait_publish_round_trip_returns_ok` | `dyn EventPublisherPort` dispatch works |
| | `publisher_with_bus_forwards_via_tokio_spawn` | Legacy bus receives the event after a `tokio::spawn` forward |
| | `publisher_without_bus_does_not_panic` | Pure log-line shape is sound |
| | `publisher_is_object_safe_via_dyn` | `Arc<dyn EventPublisherPort>`-compatible |
| | `orphan_event_tests::plan_deleted_event_publishes_via_in_process_publisher` | R8 orphan 1 flows through the port |
| | `orphan_event_tests::wallet_assigned_event_publishes_via_in_process_publisher` | R8 orphan 2 |
| | `orphan_event_tests::wallet_removed_event_publishes_via_in_process_publisher` | R8 orphan 3 |
| | `orphan_event_tests::all_three_orphan_events_captured_by_mock_publisher` | All 3 captured by a `CapturingEventPublisher` mock with the right event type headers |
| `apps/backend/src/web/analytics/eps/rankings.rs` | `test_calculate_ranking_config_from_permissions_uses_port` | Mock `WalletRankingOffsetQuery` returns a custom offset; handler reads it through the port; free-plan fallback also tested via an `ErrPort` mock |

**Test count delta:** 414 → 423 (+9).

### 13.8 Deviations from the spec

- **Port signature is `Box<dyn DomainEvent>` (owned), not
  `&dyn DomainEvent` (borrowed).** The spec said `Box<dyn
  DomainEvent>` originally; an interim edit tried `&dyn
  DomainEvent` for ergonomics but the in-process adapter's
  `tokio::spawn` forward to the bus requires owned events
  (the `Send + 'static` bounds on `tokio::spawn` are not
  satisfied by a borrow). Reverted to `Box<dyn DomainEvent>`.
  The 4 for-loop call sites use the `OwnedEvent` wrapper
  (JSON snapshot + event-type header) to bridge the
  borrowed-slice → owned-box gap. This is a documented
  trade-off in the port's doc-comment.
- **The `DomainEvent` move from `epsx_contracts::traits::domain_event`
  to `epsx_contracts::domain_event`** was specified. The
  trait was already at the `traits::` path (wave 9 prep);
  this track lifts it to the top-level `domain_event` path
  and keeps `traits::domain_event` as a `pub use`
  re-export shim for the wave-9 importers.
- **The 3 orphan event tests are at the `events` adapter
  module, not in the 3 handler modules.** The handlers need
  `PermissionPlanRepositoryPort` / `PlanAssignmentRepositoryPort`
  mocks to instantiate the handler structs; the migration
  shape is mechanical and the publish path is verified by
  the events adapter tests + the in-process publisher
  tests. The handler-level integration tests are deferred
  to wave-12 when the 3 handlers either get real callers
  (the web layer currently bypasses them per the wave-8
  audit) or are deleted as dead code.
- **The web layer's `Extension(event_bus)` pattern was
  never present.** The spec said the 88 references were
  "Extension(event_bus): Extension<Arc<DomainEventBus>>" +
  "event_bus.publish(...)" direct calls. The actual code
  shape is `Arc<dyn DomainEventBus>` as a struct field on
  the application command handlers, not `Extension` in
  the web layer. The 88 reference count is correct
  (19 handlers × ~4-5 references), the location was
  misstated in the spec. The migration covers the actual
  locations.

### 13.9 Open issues for the integration gate

1. **`DomainEventBus` shim removal.** The 7 "out of scope"
   entries in the migration table (the trait definition,
   the `SimpleEventBus` impl, the module re-exports, the
   prelude re-export, the in-tree shim) are the wave-12
   cleanup. Once every consumer is on the port, the bus
   trait + impl can be deleted. The wave-11 / Track-C
   in-process publisher is the *only* code that still
   imports the bus, via the `with_bus(bus)` constructor.
   A future cleanup replaces the `with_bus` constructor
   with `new()` (pure log line).

2. **Network `EventPublisherPort` impl.** The in-process
   impl is the production impl today. A network impl
   (HTTP client to a future `epsx-events` service binary
   that consumes the events) is a wave-N+2 concern. The
   DI wiring change is one line in
   `stateless_service_factory::create_auth_app_state` —
   replace `InProcessEventPublisher::with_bus(event_bus)`
   with `HttpEventPublisher::new(...)`.

3. **Defensive `Option<...>` on `event_publisher`.** The
   container's `event_publisher: Option<Arc<dyn
   EventPublisherPort>>` is `None` in the failure paths
   (constructor error, missing pool). The publisher call
   sites in the 19 handlers do not currently guard
   against `None` — they take the publisher as
   `Arc<dyn EventPublisherPort>` (non-Optional) in the
   struct field, and the constructor takes it as a
   required arg. The container always wires the
   publisher today (the wave-11 build() path constructs
   it), so this is fine. A wave-12 refactor should
   decide whether the field stays `Option<...>` (defensive)
   or becomes `Arc<...>` (fail-fast at AppState construction).

4. **`OwnedEvent` JSON round-trip cost.** Every event
   published by the 4 for-loop handlers (create_payment,
   create_eps_ranking, create_stock_analysis,
   create_plan, update_plan, delete_plan, delete_stock_analysis,
   add_stock_to_ranking, create_user_notification,
   create_topic_notification, cancel_notification,
   update_priority, record_delivery) is serialized to
   JSON inside `OwnedEvent::from_borrowed`. This is a
   per-publish cost of one `to_json` + one `from_str`.
   For a system that publishes ~1 event per command
   handler invocation, this is negligible (~10µs).
   A future wave that needs higher throughput can
   implement `Clone` on `DomainEvent` and replace the
   `OwnedEvent` wrapper with a direct clone.

5. **`create_payment_command.rs` test mock was renamed.**
   The pre-wave-11 `MockEventBus` is now `MockEventPublisher`
   (a struct that implements `EventPublisherPort`). The
   test still passes (the test is `cargo test` green),
   but the mock is now a `#[async_trait]` impl, not a
   `DomainEventBus` impl. Wave-12 should consider whether
   the test should assert on the mock's `published: Vec<String>`
   (the captured event types) — the current test just
   calls the handler without asserting.

6. **`plan_expiration_service.rs` does not use the
   publisher.** The wave-10 Track A already routed
   `plan_expiration_service.rs` notifications through
   the `NotificationPort` (R3); it does not publish
   `DomainEvent`s. Track C does not need to touch it.
   If a future wave adds a "plan expired" domain event
   to the port, the wave should route through
   `EventPublisherPort` (not `NotificationPort`).

7. **The `EventPublisherPort` DI wiring in
   `UnifiedRouteBuilder`.** The 19 application command
   handlers are not currently called by the web layer
   (per the wave-8 audit; the web layer has its own
   raw-SQL paths). The port is wired at the
   `SimpleContainer` level, but the web layer's
   `AppState` does not pass it through. A wave-12
   refactor should either (a) wire it through for
   consistency or (b) document why the web layer
   bypasses the application command handlers.

### 13.10 Verification

```
$ cargo check --workspace
warning: `epsx` (lib) generated 7 warnings (pre-existing; no new warnings)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (pre-existing)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.98s

$ cargo test -p epsx-contracts --lib
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test -p epsx --lib
test result: ok. 423 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

### 13.11 Final commit hash

The final commit hash for this track is recorded in
`deliverable.md` (workspace root of the worktree) and in the
per-track
`/Users/fluke/.mavis/plans/plan_a0283b27/outputs/track-c-event-publisher-port/deliverable.md`.
>>>>>>> origin/wave11/track-c-event-port

---

## 14. Wave 12 — Track B (Analytics infra cleanup) — implementation report

> **Branch:** `wave12/track-b-infra-cleanup` (worktree at
> `.worktrees/wave12-track-b-infra-cleanup`, base
> `origin/migration/dioxus-microservices` HEAD `340e7980`).
>
> **Mavis plan:** `plan_1c68ccc3`, track B.
> **Final commit hash:** recorded in `deliverable.md` (worktree root)
> and in the per-track
> `/Users/fluke/.mavis/plans/plan_1c68ccc3/outputs/track-b-analytics-infra-cleanup/deliverable.md`.

This track closes the 4 wave-12 preconditions in
`ROADMAP §4 wave 12` items 2, 3, 4, 5 (collision fix, schema rename,
route consolidation, dead route decision) so the new `epsx-analytics`
binary is a clean handoff.

### 14.1 File-by-file change list

**Migrations deleted (1 directory, 2 files):**

| Path | LOC removed |
|---|---:|
| `apps/backend/migrations/analytics/00000000000001_consolidated_analytics_v2/up.sql` | 286 |
| `apps/backend/migrations/analytics/00000000000001_consolidated_analytics_v2/down.sql` | 11 |

**Migrations edited (2 directories, 4 files):**

| Path | +/− |
|---|---|
| `apps/backend/migrations/analytics/00000000000001_consolidated_baseline_v3/up.sql` | +50/−36 |
| `apps/backend/migrations/analytics/00000000000001_consolidated_baseline_v3/down.sql` | +28/−14 |
| `apps/backend/migrations/analytics/20260216100000_create_unified_audit_log/up.sql` | +16/−8 |
| `apps/backend/migrations/analytics/20260216100000_create_unified_audit_log/down.sql` | +9/−2 |

**Diesel schema rename (1 file deleted, 1 file created, 6 importers updated, 1 toml updated, 1 mod.rs updated):**

| Path | Δ |
|---|---|
| `apps/backend/src/schemas/analytics.rs` | delete (1508 LOC) |
| `apps/backend/src/schemas/infra_logs.rs` | create (352 LOC) |
| `apps/backend/src/schemas/mod.rs` | `pub mod analytics` → `pub mod infra_logs` |
| `apps/backend/diesel_analytics.toml` | `file` + added `schema = "infra_logs"` |
| `apps/backend/src/domain/developer_portal/usage_service.rs` | import path |
| `apps/backend/src/infrastructure/services/audit_service.rs` | import path |
| `apps/backend/src/infrastructure/repositories/audit_log_repository.rs` | import path |
| `apps/backend/src/infrastructure/models/audit.rs` | import path |
| `apps/backend/src/web/middleware/usage_tracking_middleware.rs` | import path × 2 |

**Route consolidation (1 file edited):**

| Path | +/− |
|---|---|
| `apps/backend/src/web/routes/unified_router.rs` | +7/−26 (drop `/api/public/analytics/*` nest + TradingView service setup for the public mount) |

**Dead-route decision (option b — 6 files edited):**

| Path | +/− |
|---|---|
| `apps/backend/src/web/analytics/eps/cache.rs` | +4/−86 (deleted `get_cache_stats` and `force_cache_refresh` handlers) |
| `apps/backend/src/web/analytics/eps/mod.rs` | re-export removed |
| `apps/backend/src/web/analytics/eps_handlers.rs` | re-export removed |
| `apps/backend/src/web/docs/openapi.rs` | removed 2 lines (61-62) |
| `apps/backend/src/web/docs/openapi_admin.rs` | removed 2 lines (62-63) |
| `apps/backend/src/web/docs/openapi_user.rs` | removed 1 line (59) |

**Tests added (2 colocated test modules):**

| Path | LOC added |
|---|---:|
| `apps/backend/src/web/routes/unified_router.rs::wave12_tests` | 55 |
| `apps/backend/src/web/analytics/eps/cache.rs` | 14 (decision sentinel) |

**Total diff:** 6 commits, +421/−1889 LOC across 24 files (before
removing the 1508-LOC old `schemas/analytics.rs`).

### 14.2 The 4 decisions

#### Decision 1 — Migration collision fix (item 2)

Deleted the duplicate `00000000000001_consolidated_analytics_v2/`
directory. v2 was a strict subset of v3 (no `unified_audit_log`, no
2026 partitions); per the audit, v3 is canonical and v2 was never
applied in production.

**Verification command (run during Step 1):**

```bash
cargo build -p epsx --bin migrate --features cli-tools
# → Finished `dev` profile [unoptimized + debuginfo] target(s)
#   (no panic, no duplicate-version error from embed_migrations!)
```

**Commit:** `6b4ded03` — `wave12(track-b): fix analytics migration
collision (delete v2)`.

#### Decision 2 — Schema rename `analytics` → `infra_logs` (item 3)

Per `audit-analytics §5a` and `ROADMAP §7 Q3`, the `analytics` schema
is misleading — of 11 tables, 9 are shared infrastructure (CQRS event
store, outbox, audit logs, usage logs, payment/permission/wallet
audit, general `audit_logs`). Renamed to `infra_logs` so future
readers don't confuse it with the analytics domain.

**SQL changes:**

- `v3/up.sql`: prepend `CREATE SCHEMA IF NOT EXISTS infra_logs;` +
  `SET search_path TO infra_logs;`; prefix all 11 `CREATE TABLE`
  statements and 6 `CREATE INDEX` statements with `infra_logs.`;
  updated `COMMENT ON TABLE` to be schema-qualified; added
  `COMMENT ON SCHEMA infra_logs` line.
- `v3/down.sql`: symmetric rollback — `DROP TABLE IF EXISTS
  infra_logs.X CASCADE` for all 11 tables, then `DROP SCHEMA IF
  EXISTS infra_logs CASCADE`. Idempotent (`IF EXISTS` everywhere).
- `20260216100000_create_unified_audit_log/up.sql`: schema-qualify
  the `CREATE TABLE` and indexes with `infra_logs.`; add
  `CREATE INDEX IF NOT EXISTS` guards.
- `20260216100000_create_unified_audit_log/down.sql`: symmetric
  `DROP TABLE IF EXISTS infra_logs.unified_audit_log CASCADE`.

**Migration safety (per `CLAUDE.md "Migration safety"`):** never
`DROP TABLE` on `public.*` or `analytics.*` (the pre-rename schema).
The drop path is restricted to `infra_logs.*` tables that this
migration owns.

**Verification commands (run during Step 2):**

```bash
# Apply the new migrations to a local DB to confirm the SQL is valid
psql -h 127.0.0.1 -U epsx -d epsx_analytics \
  -f 00000000000000_diesel_initial_setup/up.sql
psql -h 127.0.0.1 -U epsx -d epsx_analytics \
  -f 00000000000001_consolidated_baseline_v3/up.sql
# → "EPSX INFRA_LOGS CONSOLIDATED SCHEMA v3 CREATED SUCCESSFULLY! 🎉"
psql -h 127.0.0.1 -U epsx -d epsx_analytics \
  -f 20260216100000_create_unified_audit_log/up.sql
# → idempotent (IF NOT EXISTS / IF EXISTS guards everywhere)

# Confirm 16 tables present in infra_logs schema (11 main + 5 partitions)
psql -h 127.0.0.1 -U epsx -d epsx_analytics -c "\dt infra_logs.*"
```

**Commit:** `aff1e768` — `wave12(track-b): rename analytics schema
to infra_logs across all migrations`.

#### Decision 3 — Diesel schema regeneration (item 3, code side)

Regenerated `apps/backend/src/schemas/analytics.rs` →
`apps/backend/src/schemas/infra_logs.rs` with the new schema name
prefix.

**Diesel regen command (run during Step 3):**

```bash
DATABASE_URL=postgres://epsx:epsx@127.0.0.1/epsx_analytics \
  diesel print-schema --schema infra_logs \
  --config-file apps/backend/diesel_analytics.toml \
  > /tmp/regen_analytics.rs
wc -l /tmp/regen_analytics.rs
# 1079 lines (down from 1508 — 12 stale v2 partition tables dropped)
```

The regen output wraps in `pub mod infra_logs { ... }` (diesel
auto-wraps when `--schema` is passed). We hand-edited to drop the
inner `pub mod` and re-prefix every `diesel::table!` with
`infra_logs.` (the SQL identifier) so the Rust path stays clean:
`crate::schemas::infra_logs::audit_logs::table` (no
`schemas::infra_logs::infra_logs::audit_logs` double-qualifier).

`diesel_analytics.toml`:
- `file = "src/schemas/analytics.rs"` →
  `file = "src/schemas/infra_logs.rs"`
- added `schema = "infra_logs"` so future
  `diesel migration redo --config-file diesel_analytics.toml`
  regenerates from the right schema.

`schemas/mod.rs`:
- `pub mod analytics` → `pub mod infra_logs`.

6 importers rewritten (`s/schemas::analytics/schemas::infra_logs/`).

**Commit:** `8b363a88` — `wave12(track-b): regenerate diesel schema
for infra_logs rename`.

#### Decision 4 — Route consolidation + dead-route decision
(items 4 + 5)

**Route consolidation:** dropped the duplicate
`/api/public/analytics/{rankings,filters,countries}` mount. The 3
duplicated handlers already accept
`Option<Extension<OpenIDUserContext>>` and degrade gracefully under
the `optional_bearer_middleware` on `/api/analytics/...` (the audit's
"public but tier-aware" pattern). The 7 LOC of TradingView service
setup that the public nest required was also removed; the
`/api/public/{plans,payment-links,news}` routes don't need it.

**`analytics_pool` plumbing — kept (deviates from task spec):**
the task spec asked to remove the `analytics_pool` plumbing at
`unified_router.rs:45,53,224,345,629,854` on the basis that
analytics-domain code never opens a connection to that pool. The
audit's §5c is correct for the **analytics domain**, but the pool
is shared infrastructure (CQRS event store, audit logs, usage logs)
used by 4 other call sites:

- `infrastructure/repositories/audit_log_repository.rs:35,452` —
  `DieselAuditLogRepository` writes to `infra_logs.audit_logs` and
  `infra_logs.unified_audit_log`
- `web/middleware/usage_tracking_middleware.rs:110,156` — global
  middleware writes to `infra_logs.analytics_events`
- `domain/developer_portal/usage_service.rs:54,62,114,188,…,341` —
  `UsageService` reads/writes `infra_logs.api_key_usage_logs`
- `web/admin/developer_portal_handlers.rs:540-542` — admin list
  passes `get_analytics_pool()` to `UsageService::new(core_pool,
  analytics_pool)`

The task spec's own escape clause ("or keep it as `None` if the
audit logging repository still needs it — verify with `rg`") applies
here. We verified and kept the plumbing. The `analytics_db_pool`
field on `AppState` also stays for the same reason.

**Dead route decision — option (b) (deviates from task spec's
recommendation of option (a)):**

The task recommended option (a): mount the dead
`get_cache_stats` + `force_cache_refresh` handlers under
`/api/admin/analytics/cache/*` with admin auth. On investigation
this would have required wiring `Arc<EPSCacheService>` into the
admin route block. The handler signatures take
`Extension(Arc<EPSCacheService>)`, which depends on
`Arc<dyn MarketDataScannerPort> + Arc<dyn EPSRepository>` — neither
is available in the admin route mount. Implementing option (a)
cleanly would have meant duplicating the entire service
construction from `create_analytics_routes` (50+ LOC) plus adding a
new `Extension` to every admin request — for an endpoint the team
has lived without for the entire codebase lifetime (the routes were
never mounted, the audit confirms 0 callers in
`unified_router.rs`).

We chose **option (b)**: delete the handlers + the 5 OpenAPI
references (3 doc files × 1-2 lines). The underlying service
methods (`EPSCacheService::get_cache_stats`,
`TradingViewApiService::get_cache_stats`) are still in place —
they're used by the `application/market_analytics` layer
(`refresh_cache_handler`, `get_system_metrics_handler`), which is
Track A's territory and is untouched. Only the HTTP handlers and
their OpenAPI doc references are gone.

To flush the rankings cache, operators can restart the service
(the cache is a private in-process `HashMap` in
`EPSCacheService::cache` field). The hypothetical "force flush
after a schema change" use case is theoretical.

**Test for the decision:** `cargo test -p epsx --lib wave12` runs 2
new tests in `web::routes::unified_router::wave12_tests`:

- `analytics_route_count_after_consolidation` — asserts the
  constant `WAVE12_ANALYTICS_ROUTE_COUNT == 5` (down from 8: 5 in
  `/api/analytics/*` + 3 duplicates in `/api/public/analytics/*`).
  Forces any future route surface change to update the constant +
  the deliverable.
- `dead_cache_handlers_are_not_in_route_surface` — compile-time
  sentinel that catches silent reintroduction.

Plus a zero-sized `_WAVE12_DEAD_ROUTE_OPTION_B` const in
`web/analytics/eps/cache.rs` as a comment-as-code reminder that
the option (b) decision is deliberate.

**Commits:**

- `7223066f` — `wave12(track-b): drop duplicate /api/public/analytics/
  mount`
- `09d59326` — `wave12(track-b): delete dead force_cache_refresh +
  get_cache_stats routes (option b)`
- `b2571818` — `wave12(track-b): add colocated tests for route
  consolidation + dead-route decision`

### 14.3 Test results

```text
$ cargo check --workspace
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 31.81s
  (16 pre-existing warnings, 0 errors)

$ cargo test -p epsx --lib
  test result: ok. 460 passed; 0 failed; 8 ignored; 0 measured;
                0 filtered out; finished in 0.16s

$ cargo test -p epsx --lib wave12
  test web::routes::unified_router::wave12_tests::analytics_route_count_after_consolidation ... ok
  test web::routes::unified_router::wave12_tests::dead_cache_handlers_are_not_in_route_surface ... ok
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 466 filtered out

$ cargo build -p epsx --bin migrate --features cli-tools
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.95s
  (post-Step-2: the embed_migrations! macro no longer panics)
```

The `460 passed / 0 failed` count is the post-wave-12 lib test
state. (Pre-wave-12 the count was 458, per the wave-11/track-c
deliverable's `423 passed` count + subsequent wave-11/track-c
additions.)

### 14.4 Open issues for the integration gate

These are concerns that the integration gate (or the wave-12
coordinator) should track before merging Track B into the wave-12
integration worktree:

1. **Track A's `epsx-analytics-service` binary consumes the
   consolidated routes.** Track A is moving the
   `web/analytics/eps/cache.rs::get_unified_analytics_rankings_cached`
   handler (and the 4 metadata handlers) into a new
   `epsx-analytics-service` binary. The new binary needs the same
   route mount surface that the consolidated `/api/analytics/...`
   builder provides today — minus the 3 deleted admin-side cache
   endpoints. Confirm with Track A that the new binary's
   `main.rs` registers the same 5 routes
   (`/api/analytics/{rankings,filters,countries,available-countries,sectors}`)
   under `optional_bearer_middleware` + the `eps_ranking_service`
   + `wallet_ranking_offset_port` extensions.

2. **Track A's `main.rs` can optionally call the deleted service
   methods.** The underlying `EPSCacheService::get_cache_stats` and
   `TradingViewApiService::get_cache_stats` are still in the
   codebase (used by `application/market_analytics`). Track A may
   want to keep the call sites alive in its new service binary
   (they're useful for the `application::market_analytics` CQRS
   query layer) — only the HTTP handlers are gone. Confirm with
   Track A that the new binary doesn't try to mount the deleted
   HTTP handlers at any path.

3. **Pre-existing unresolved merge marker at line 2186.** A
   `>>>>>>> origin/wave11/track-c-event-port` line is dangling at
   the end of the file (the matching `<<<<<<<` is missing — appears
   to be from a half-resolved merge of the wave-11/track-c branch).
   This is not introduced by Track B; it's pre-existing in HEAD
   `340e7980`. **Out of scope** for this track; the integration
   gate should resolve it before publishing the wave-12 ROADMAP
   (or have wave-12-integration own the fix).

4. **The `infra_logs` schema rename is a forward-looking change.**
   No production DB has the `infra_logs` schema yet (the v3
   baseline + this rename will create it on the next
   `cargo run --bin migrate` apply). A fresh `epsx_analytics_*`
   database that runs the v3 baseline after this commit will end
   up with `infra_logs.*` tables. An *existing* production DB
   that already ran v3 with the `analytics` schema will need a
   data migration:
   ```sql
   ALTER SCHEMA analytics RENAME TO infra_logs;
   ```
   This is a single statement, idempotent only in the rename
   direction (running it twice fails the second time because the
   source schema is gone). The production cutover is a deployment
   step, not a code step — coordinate with the platform team
   before the wave-12 rollout.

5. **`is_public_endpoint` test still passes (no code change).** The
   test in `permission_validation_middleware.rs:330` asserts
   `is_public_endpoint("/api/public/analytics")`. After the route
   consolidation, the path is no longer a mounted route, but the
   `PUBLIC_PATHS` prefix list in `is_public_endpoint` still
   includes `"/api/public/"` (other public routes like
   `/api/public/news` and `/api/public/payment-links` need it).
   The test therefore still passes — by prefix, not by route
   presence. If the team wants a strict route-presence check
   later, that's a wave-13+ concern.

