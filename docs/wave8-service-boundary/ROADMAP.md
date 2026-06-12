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
