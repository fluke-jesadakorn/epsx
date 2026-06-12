# Shared-Kernel & Core Dependencies — Split-Readiness Audit

> **Wave 8 — Service-Boundary Planning**
> **Author:** Mavis (orchestrator, owner takeover from stuck `coder` session at minute 27 — worker had completed the full 6-step inventory in context but never wrote the report)
> **Branch:** `wave8/service-boundary` (commit pending — see §10)
> **Source evidence:** all claims backed by `rg`/`wc`/file reads at worktree HEAD = `20eda14a`
> **Scope:** `apps/backend/src/core/`, `apps/backend/src/domain/shared_kernel/`, process-global state in `main.rs`/`lib.rs`/`application/mod.rs`, process-wide middleware in `web/middleware/`, cross-schema FKs in `migrations/core/`, and the (non-existent) cross-language Rust shared crate surface.

## Executive summary

The "shared kernel" in this codebase is **physically small and operationally critical**, but its public surface is **wide-open**: there are no `pub(crate)` or feature gates anywhere. Almost every domain imports directly from `core::errors::AppError` / `core::errors::ErrorKind` and from `shared_kernel::value_objects::*` / `shared_kernel::entities::*`. The cost of "sharing" today is hidden because the process is a single binary. The cost of "splitting" will be: defining a contract for which types each downstream service may import.

**Split-readiness score: 2.0/5** — the kernel is mechanically extractable (small, well-bounded, dependency-free) but the consumer fan-out is so wide that an honest extraction requires either (a) the kernel becomes a published crate with a stable API, or (b) we accept that all future services keep depending on it as a shared library, and the "split" is purely about removing domains' cross-imports of each other (Shape B from the auth audit, not Shape A).

**Top blocker:** `core::errors::AppError` and `core::errors::ErrorKind` are imported by 34 and 27 distinct call sites respectively (the most-shared symbols in the whole tree, by an order of magnitude). These will be the first thing to formalize when the kernel is extracted.

**Good news:** there is **no cross-language Rust shared crate** to worry about. `shared/` is TypeScript-only (api, auth, components, config, env, hooks, state, stubs, types, utils, validators). The backend has no `path = "shared/..."` deps in `Cargo.toml`. Any "Rust shared" work in the future would be net-new, not a migration.

---

## 1. Shared-kernel inventory

### 1.1 `apps/backend/src/core/` — 1,408 LOC across 6 files

| File | LOC | Role |
|------|-----|------|
| `core/mod.rs` | 16 | Re-exports `constants`, `errors`, `permissions`, `telemetry`, `types` |
| `core/constants.rs` | 206 | `FREE_PLAN_RANKING_OFFSET`, `FREE_PLAN_ID`, `FREE_PLAN_NAME`, `is_system_admin_plan`, `MINUTE` |
| `core/errors.rs` | 736 | `AppError`, `ErrorKind`, `AppResult`, `ErrorContext` |
| `core/permissions.rs` | 217 | `has_permission(...)` — the **canonical** rule check (per CLAUDE.md "Permissions & Plan Logic — Backend Only") |
| `core/telemetry.rs` | 143 | Tracer/meter init helpers |
| `core/types.rs` | 90 | Common type aliases |

### 1.2 `apps/backend/src/domain/shared_kernel/` — 3,736 LOC across 33 files

Top-level modules (counts are `*.rs` files / total LOC):

| Subtree | Files | LOC | Role |
|---------|------:|----:|------|
| `value_objects/` | 10 | 1,041 | `common_types`, `email`, `identifiers`, `market`, `payments`, `quarterly_eps_data`, `session_id`, `symbol`, `user_id`, `user_limits` |
| `entities/` | 6 | 759 | `audit`, `auth`, `eps_growth`, `market_data`, `stock`, `user` |
| `services/` | 2 | 520 | `eps_ranking_service` (515 LOC — biggest single file), `mod.rs` |
| `app_error.rs` | 1 | 821 | The shared `AppError`/`ErrorKind` live here **too** — duplicate of `core/errors.rs` (see §9 finding) |
| `aggregate_root.rs` | 1 | 136 | `pub trait AggregateRoot` |
| `domain_event.rs` | 1 | 102 | `pub trait DomainEvent`, `pub trait DomainEventBus` |
| `specification.rs` | 1 | 96 | `pub trait Specification<T>` |
| `value_object.rs` | 1 | 33 | `pub trait ValueObject` |
| `ports/` | 2 | 51 | `market_data_service_port` (one port, single consumer: `domain::market_analytics`) |
| `event_bus.rs` | 1 | 3 | Stub re-export (real bus lives in `infrastructure/cqrs/`) |
| `mod.rs` | 1 | 24 | Re-exports |

### 1.3 Process-global state (`main.rs` / `lib.rs`)

`main.rs` is **76 lines total**, all entry-point wiring. It delegates everything to `epsx::bootstrap::build_runtime(BackendBootstrapOptions::stateful_server())`. The actual "globals" live one level down in `bootstrap.rs` (out of scope here, but worth flagging: the `AppState` struct in `web/auth/app_state.rs` holds 13 `Arc`s — `db_pool`, `cache`, `domain_container`, `redis_pool`, `redis_broadcaster`, `plan_repo`, `transaction_history_provider`, `identity_provider`, `analytics_db_pool`, `audit`, `s3`, plus two backwards-compat stubs). Every one of those `Arc`s is process-global and would need to be reconstituted per-service.

### 1.4 Cross-language dependencies

**There is no `shared/rust/`.** The `shared/` directory contains 13 TypeScript subdirs only (`api`, `auth`, `components`, `config`, `env`, `hooks`, `state`, `stubs`, `types`, `utils`, `validators`, plus root `package.json` / `tsconfig.json` / `eslint.config.cjs`). `apps/backend/Cargo.toml` has **zero `path = "..."` deps** pointing at any shared crate. The only `path =` lines are backend's own bins (`migrate.rs`, `grant_wallet_permission.rs`, `migrate_media.rs`, `epsx_vercel.rs`). **This is a meaningful finding: any "Rust shared crate" extraction is net-new, not a migration.**

---

## 2. Inbound dependencies — who consumes the shared kernel

Per-symbol fan-out (greps in `apps/backend/src`):

### 2.1 `core::constants` importers

| Symbol | Call sites |
|--------|-----------:|
| `FREE_PLAN_RANKING_OFFSET` | 8 |
| `FREE_PLAN_ID` | 4 |
| `FREE_PLAN_NAME` | 2 |
| `is_system_admin_plan` | 1 |
| `MINUTE` | 1 |

Total: 16 imports. **Concentrated in `domain/payment` and `application/wallet_management`.** Not really shared — it's a payments/wallets constant that's accidentally in the kernel.

### 2.2 `core::errors` importers (the real hot path)

| Symbol | Call sites |
|--------|-----------:|
| `AppError` | **34** |
| `ErrorKind` | **27** |
| `AppResult` | 3 |
| `ErrorContext` | 1 |

**This is the single most-imported surface in the entire backend.** If the kernel becomes a published crate, this is the first module to lock down.

### 2.3 `core::types` importers

**0 importers** as of HEAD. The file exists but is unused. Dead code candidate for kernel extraction.

### 2.4 `core::telemetry` importers

**0 importers** in `apps/backend/src`. Telemetry init likely happens via side-effect in `bootstrap.rs` / `infrastructure/logger.rs` — not by typed import.

### 2.5 `core::permissions` importers

See the auth audit (§2.2 in `audit-auth.md`): 14 non-auth files import this. **The whole permission check is a kernel-level function call**, not a port. This is the constraint that forces Shape B (shared crate) before Shape A (network split) — the rule has to be in the deployed business-service binary.

### 2.6 `domain::shared_kernel::*` importers

| Symbol | Call sites | Notes |
|--------|-----------:|-------|
| `shared_kernel::entities` | 52 | Mostly `use shared_kernel::entities::{User, Audit, ...}` |
| `shared_kernel::value_objects` | 22 | Specific value-object types |
| `shared_kernel::DomainEventBus` | 18 | (lives in `infrastructure/cqrs` but re-exported via shared_kernel) |
| `shared_kernel::value_object` | 14 | The trait |
| `shared_kernel::aggregate_root` | 11 | The trait |
| `shared_kernel::services` | 9 | `eps_ranking_service` only — but it's used by 7 files across `web/analytics`, `infrastructure/adapters/repositories`, `domain/market_analytics` |
| `shared_kernel::domain_event` | 7 | The trait |
| `shared_kernel::app_error` | 6 | The duplicate `AppError` (see §9) |
| `shared_kernel::ValueObject` | 4 | The trait |
| `shared_kernel::DomainEvent` | 3 | The trait |
| `shared_kernel::Specification` | 1 | The trait |

**No `use crate::domain::shared_kernel::*` (wildcard) importers.** That was a feared failure mode that doesn't exist.

### 2.7 Importer layers for `shared_kernel::value_objects` and `entities`

**`value_objects` importers by layer** (8 distinct consumers):

| Layer | Sites |
|-------|------:|
| `domain/payment` | 5 |
| `application/wallet_management` | 4 |
| `domain/shared_kernel` (self) | 2 |
| `web/middleware` | 1 |
| `web/admin` | 1 |
| `infrastructure/repositories` | 1 |
| `infrastructure/adapters` | 1 |
| `domain/wallet_management` | 1 |
| `application/payment` | 1 |

**`entities` importers by layer** (8 distinct consumers, dominated by analytics):

| Layer | Sites |
|-------|------:|
| `infrastructure/adapters` | 16 |
| `web/analytics` | 10 |
| `domain/shared_kernel` (self) | 2 |
| `domain/market_analytics` | 2 |
| `web/admin` | 1 |
| `infrastructure/repositories` | 1 |
| `domain/audit` | 1 |
| `application/market_analytics` | 1 |

**Read:** `entities` are mostly used by **analytics-shaped code paths** (infrastructure adapters, web/analytics). The value_objects are mostly used by **payments-shaped code paths**. If we ever extract a per-service `entities.rs` / `value_objects.rs`, the natural split is by which future service uses them.

### 2.8 `shared_kernel::services::eps_ranking_service` callers (concrete)

7 call sites, all of them in analytics:
- `domain/market_analytics/domain_services/eps_cache_service.rs` — `EPSRepository`
- `web/analytics/eps/metadata.rs` — `EPSRankingService`
- `web/analytics/eps/rankings.rs` — `EPSRankingService`, `EPSRankingParams`
- `web/analytics/eps/cache.rs` — only a `// REMOVED` comment referencing the import
- `web/analytics/repository.rs` — `EPSRepository`
- `infrastructure/adapters/repositories/stock_analysis_repository_adapter.rs` — both
- `infrastructure/adapters/repositories/tradingview_eps_repository.rs` — `EPSRepository`

**Conclusion:** the largest single file in the shared kernel (515 LOC `eps_ranking_service.rs`) is **only used by analytics**. It should arguably move into `domain/market_analytics/services/` as part of any kernel extraction. Keeping it in `shared_kernel` is a misnomer that the analytics audit's "Score 3/5" verdict already flagged.

---

## 3. Outbound dependencies — what the kernel depends on

The shared kernel is **clean outbound**: only `std`, `chrono`, `serde`, `uuid`, `tracing`, plus the in-tree `crate::core::*` re-exports. No domain imports. No application imports. No web imports. The only "external" dependency of note is `app_error.rs` referencing some types from `value_object.rs` (the `ValueObject` trait is the error context).

This is good news: the kernel can move to a workspace crate without dragging any domain code with it.

---

## 4. Database seams

### 4.1 Cross-schema FKs (the single most important question for any split)

`rg "REFERENCES" apps/backend/migrations/core/` returns **32 FK declarations** in core migrations, across 5 files:
- `core/00000000000001_consolidated_baseline_v6/up.sql`
- `core/00000000000001_consolidated_schema_v5/up.sql`
- `core/20260218000000_create_support_chat/up.sql`
- `core/20260214000000_plan_features_and_categories/up.sql`
- `core/20260216000000_create_user_watchlist/up.sql`

**Crucially:** `rg "REFERENCES" apps/backend/migrations/{payments,analytics,notifications}/` returns **0 cross-schema FKs** to core (or anywhere else). The sibling schemas' migrations are clean. Verified.

This means: **if any service needs the user/auth/plan tables, it has to either keep its own copy (synchronization problem) or query the core schema over the network**. There is no FK-based hard link forcing coupling.

### 4.2 `read_model` schema (mentioned in payments audit)

`migrations/core/00000000000001_consolidated_schema_v5/up.sql` contains 2 `CREATE SCHEMA read_model` statements (the read-side projection tables for CQRS). This is **in the core schema**, not a separate one. The payments audit's §4 mentioned this. It belongs to the kernel's database story.

### 4.3 Diesel schema registration

There are 4 separate `diesel*.toml` configs: `diesel.toml`, `diesel_payments.toml`, `diesel_analytics.toml`, `diesel_notifications.toml`. They produce 4 separate `src/schemas/*.rs` files. The `apps/backend/src/infrastructure/database/diesel_connection_manager.rs:339-373` keeps a **single process-wide pool** that multiplexes the 4 schemas. After a split, each service would own one of these configs and one pool.

---

## 5. Process-wide middleware (must replicate per service)

`apps/backend/src/web/middleware/` is **3,361 LOC across 10 files**. The unified router (`web/routes/unified_router.rs:707-833` etc.) layers all of them on every request:

| File | LOC | Purpose | Per-service copy needed? |
|------|----:|---------|:------------------------:|
| `rate_limiter.rs` | 643 | Sliding window + threat-aware limits | ✅ |
| `permission_validation_middleware.rs` | 475 | Calls `core::permissions::has_permission` | ✅ (must stay in sync with kernel) |
| `bearer_middleware.rs` | 502 | JWT parse → `OpenIDUserContext` | ✅ (split already: validation path must stay local per auth audit) |
| `auth_middleware.rs` | 380 | Session-based auth (alternative to bearer) | ✅ |
| `rate_limit_middleware.rs` | 226 | Token-bucket fallback | ✅ |
| `usage_tracking_middleware.rs` | 194 | Per-request usage counters → Postgres | ✅ |
| `security_headers.rs` | 131 | CORS / CSP / HSTS | ✅ |
| `multi_level_rate_limiter.rs` | 647 | Per-tier limits (admin/user/public) | ✅ |
| `governor_limiter.rs` | 79 | Threat-aware governor (in-memory) | ✅ |
| `mod.rs` | 84 | Re-exports | n/a |

**`permission_validation_middleware.rs:228` (and 326) calls `crate::core::permissions::has_permission(...)` directly** — this is the canonical rule check, in-process. Per the auth audit's recommendation, this must remain a kernel-level function call (Shape B), not a network call.

**Every service that exposes HTTP routes will need its own copy of all 10 middleware files** (or a published `epsx-web-middleware` crate). This is the single biggest "replicate per service" surface in the codebase.

---

## 6. CQRS / event-bus state

`apps/backend/src/infrastructure/cqrs/` is 6 files (`event_dispatcher.rs`, `event_store.rs`, `outbox.rs`, `projection.rs`, `projections/`, `mod.rs` + `STATUS.md` + `USAGE.md`).

- `rg "event_bus"` across `apps/backend/src` returns **88 hits**, 12 importers across `application/*` (9 sites) and `infrastructure/*` (3 sites).
- `rg "DomainEventBus"` returns **67 hits** — the bus is referenced heavily even though the producer in `apps/backend/src/infrastructure/cqrs/` has **no `subscribe` API** (per the analytics audit's "Realtime coupling is one-way publisher into a no-op DomainEventBus" finding).

**The CQRS infrastructure is a single-bus design** (one in-process bus, one Postgres outbox, one set of projections). If we split into N services, each gets its own outbox, but **events that span services (payment → notification, auth → analytics) would have to go over the network**. The kernel would gain an `EventPublisherPort` trait that the CQRS infrastructure implements, and each domain would inject the port instead of the concrete bus. That's a refactor, not a split blocker.

---

## 7. External integrations (kernel-relevant)

- **Redis** (`apps/backend/src/infrastructure/redis/`): one client, 2 keys namespaces (cache + SSE broadcaster). Per the notifications audit, the SSE broadcaster leaks 4 chat-side call sites — hoisting it to a shared `pubsub` module is the cleanest fix and is a kernel concern.
- **WebSocket fanout** (`apps/backend/src/infrastructure/realtime_events/`): single process, no replication today. After a split, this either moves to a dedicated service or each service owns its own.
- **Blockchain adapters** (`apps/backend/src/infrastructure/blockchain/`): payments-only, not a kernel concern.
- **HTTP clients** (reqwest via `infrastructure/adapters/`): per-domain, not a kernel concern.

---

## 8. Cross-cutting concerns for the synthesis

The synthesis task in `docs/wave8-service-boundary/ROADMAP.md` should treat the shared kernel as a **separate, addressable row** in the per-domain verdict table, not a footnote. Key items the synthesis must surface:

1. The kernel is mechanically extractable today (small, dependency-free, single-trait surface). The cost is **API stability**: anything in `core::errors` or `core::permissions` becomes a published contract.
2. The duplicate `app_error` situation (see §9) must be resolved **before** the kernel is extracted, or the crate will have two error types.
3. `core::types` is dead (0 importers) — drop it from the kernel.
4. `eps_ranking_service` (515 LOC) belongs in `domain/market_analytics`, not in the shared kernel.
5. The 10 middleware files (~3,361 LOC) are the biggest "replicate per service" cost. Packaging them as `epsx-web-middleware` is a high-leverage move.
6. The `permission_validation_middleware` → `core::permissions` call is the structural reason Shape B (shared crate) is mandatory before Shape A (network split).

---

## 9. Real bugs / smells surfaced

Each of these is a pre-split cleanup item the synthesis should flag as a wave-N+1 candidate.

1. **Duplicate `AppError`** — `core::errors::AppError` and `shared_kernel::app_error::AppError` are both imported (34 + 6 sites respectively). The two files are 736 + 821 LOC. They are NOT the same type (different module paths), so any code that has both imports has a name collision. This is a maintenance footgun and a real bug source — there are likely code paths where one is used by mistake. **Action: collapse into a single `core::errors` module before kernel extraction.**
2. **`core::types.rs` is dead** — 0 importers, 90 LOC. Safe to delete.
3. **`shared_kernel::event_bus.rs` is a 3-LOC stub** — the real bus is in `infrastructure/cqrs/`. Confusing.
4. **`eps_ranking_service` is in the wrong place** — used by 7 analytics call sites, 0 elsewhere. Should be in `domain/market_analytics/services/`.
5. **`shared_kernel::ports/` has one port** — `market_data_service_port.rs` (45 LOC), used by `domain/market_analytics`. Either expand into a real ports layer or remove.
6. **`infrastructure/security/key_management.rs`** (per the auth audit) is a dead second key manager. Independent concern; cross-references for the synthesis.
7. **2 `READ_MODEL` schemas in core migrations** (the payments audit's §4 already noted this). Belongs to the CQRS infra, not the kernel.

---

## 10. Top 3 refactors for a 4+ score (with file paths, LOC, mechanism)

### Refactor 1 — Collapse `core::errors` and `shared_kernel::app_error` into one

- **Files touched:** `apps/backend/src/core/errors.rs` (move-target), `apps/backend/src/domain/shared_kernel/app_error.rs` (delete), `apps/backend/src/core/mod.rs` (re-export), `apps/backend/src/domain/shared_kernel/mod.rs` (remove re-export).
- **Estimated LOC:** ~50 LOC net (821 + 736 → 736 + 50 of re-exports).
- **Mechanism:** pick one as canonical (recommend `core::errors` because it has 34 importers vs. 6, and it's the top-level module), re-export from the old path with `#[deprecated]`, fix the 6 callers in one sweep, delete `shared_kernel::app_error.rs`. Per-callsite rename is mechanical (`sed -i 's/shared_kernel::app_error::/core::errors::/g'`).
- **Score effect:** removes the most confusing maintenance footgun; 2.0 → 2.5.

### Refactor 2 — Extract `epsx-contracts` (or `epsx-kernel`) workspace crate, versioned

- **Files moved:**
  - `apps/backend/src/core/{constants,errors,permissions,telemetry}.rs` → `shared/rust/epsx-kernel/src/`
  - `apps/backend/src/domain/shared_kernel/{aggregate_root,domain_event,specification,value_object,value_objects/*,entities/*}.rs` → `shared/rust/epsx-kernel/src/`
  - `apps/backend/src/domain/shared_kernel/ports/` → `shared/rust/epsx-kernel/src/ports/`
- **Files kept in `apps/backend`:** `core/mod.rs` becomes a 1-line `pub use epsx_kernel::*;`. `domain/shared_kernel/mod.rs` likewise.
- **Estimated LOC:** ~2,500 LOC moved (1,408 core + ~1,400 of shared_kernel that is genuinely shared), ~30 LOC of stub re-exports added.
- **Mechanism:** create `shared/rust/epsx-kernel/Cargo.toml` with the kernel's deps, move files, replace `use crate::core::` with `use epsx_kernel::` in 16 callsites for constants + 65 for errors + 14 for permissions, and `use crate::domain::shared_kernel::` → `use epsx_kernel::` in 90+ callsites. Bump edition 2021, set up CI to publish to internal registry.
- **Score effect:** this is the score-3-to-4 step. After this, every service can declare `epsx-kernel = "0.1"` in its Cargo.toml and stop recompiling the kernel.
- **Risk:** breaking change surface. Mitigation: start with re-exports at the old paths, deprecate in 0.2, remove in 0.3.

### Refactor 3 — Package `epsx-web-middleware` crate, replicate per service

- **Files moved:** `apps/backend/src/web/middleware/*.rs` (3,361 LOC across 10 files) → `shared/rust/epsx-web-middleware/src/`.
- **Files kept in `apps/backend`:** the unified router's `.layer(...)` calls become `epsx_web_middleware::*` calls.
- **Estimated LOC:** ~3,400 LOC moved, ~200 LOC of stub re-exports.
- **Mechanism:** same as Refactor 2 — workspace crate, versioned, with the same `epsx-kernel` re-export so `permission_validation_middleware` can still call `epsx_kernel::permissions::has_permission`. Each future service gets `epsx-web-middleware = "0.1"` and is forced to opt-in to specific middlewares (the threat-aware governor is in-memory and probably doesn't need to ship to every service).
- **Score effect:** this is the score-4-to-5 step. It's the last piece of "shared kernel + shared web layer" before services can be truly independent.
- **Risk:** the rate-limiter and threat-aware governor carry in-process state (sliding windows per IP, threat scores). After this refactor, each service has its own counters — which is actually the right behavior (per-service rate limits), but it's a behavior change.

### Combined score trajectory

- Current: 2.0
- After Refactor 1: 2.5
- After Refactor 2: 3.5
- After Refactor 3: 4.5

---

## 11. Summary table for the synthesis

| Aspect | Status | Notes |
|--------|:------:|-------|
| `core/` + `shared_kernel/` LOC | 5,144 | small, manageable |
| Most-imported symbol | `AppError` (34 sites) | locks down the kernel's #1 API |
| Cross-domain fan-out | 8+8 layers (value_objects, entities) | wide but clean |
| Cross-language Rust shared | **none** | only TypeScript in `shared/` |
| Cross-schema FKs to core | **0** | payments/analytics/notifications migrations are clean |
| `eps_ranking_service` misplacement | 515 LOC, 7 analytics callers | should move out of kernel |
| `core::types` dead code | 90 LOC, 0 importers | safe to delete |
| Duplicate `AppError` | 736 + 821 LOC, two definitions | real bug source |
| Process-wide middleware | 3,361 LOC, 10 files | biggest replicate-per-service cost |
| Event bus (CQRS) | 88 refs, single in-process bus | needs `EventPublisherPort` per service after split |
| `core::permissions` callers | 14 non-auth files | forces Shape B (shared crate) |
| Mid-split-blocking bugs | 7 items | all listed in §9 |

---

## 12. References

- `apps/backend/src/core/{constants,errors,permissions,telemetry,types,mod}.rs`
- `apps/backend/src/domain/shared_kernel/` (33 files, see §1.2)
- `apps/backend/src/web/middleware/` (10 files, see §5)
- `apps/backend/src/infrastructure/cqrs/` (single-bus design, see §6)
- `apps/backend/src/main.rs` (76 LOC, all entry-point wiring)
- `apps/backend/src/web/auth/app_state.rs` (13 `Arc` fields — `AppState`)
- `apps/backend/migrations/core/00000000000001_consolidated_{baseline_v6,schema_v5}/up.sql` (32 FKs, all intra-core)
- `docs/wave8-service-boundary/audit-payments.md` (sister audit — references the duplicate `read_model` schema)
- `docs/wave8-service-boundary/audit-auth.md` (sister audit — establishes the Shape B requirement)
- `docs/wave8-service-boundary/audit-analytics.md` (sister audit — established that `eps_ranking_service` is analytics-only)
- `docs/wave8-service-boundary/audit-notifications.md` (sister audit — referenced for the Redis pubsub hoist)
- `CLAUDE.md` ("Permissions & Plan Logic — Backend Only" — the structural reason this audit's recommendation is Shape B)
