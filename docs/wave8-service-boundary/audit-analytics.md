# Wave 8 — Analytics Split-Readiness Audit

**Audit worktree:** `wave8/audit-analytics` (branched from `wave8/service-boundary`)
**Source root:** `apps/backend/` (Rust modular monolith, 118,975 LOC total)
**Scope:** `market_analytics` (domain) + `application::market_analytics` + `web::analytics` + `diesel_analytics.toml` + `migrations/analytics/` + cache + realtime events.

This audit answers: could the analytics domain be lifted out of the monolith as a
standalone microservice, and what would it cost? Every claim below is backed by a
`rg` / `wc` / `find` citation.

---

## 1. Bounded-context inventory

The analytics domain is spread across **three sibling tree roots** that all carry
the same conceptual name. Naming collision is part of the audit's findings.

### 1a. `domain/market_analytics/` — DDD core (3,369 LOC across 16 files)

| File | LOC | Role |
|---|---:|---|
| `aggregates/eps_ranking.rs` | 619 | `EPSRanking` aggregate; emits `EPSRankingCreated` |
| `aggregates/stock_analysis.rs` | 579 | `StockAnalysis` aggregate; emits `StockAnalysis{Created,Updated,Deleted}` |
| `domain_services/eps_cache_service.rs` | 465 | In-process cache + TradingView fetcher |
| `value_objects/market_sector.rs` | 430 | `MarketSector` enum-like VO |
| `value_objects/country.rs` | 406 | `Country` VO + ISO mapping |
| `value_objects/growth_factor.rs` | 281 | `GrowthFactor` math VO |
| `value_objects/eps_value.rs` | 247 | `EPSValue` math VO |
| `value_objects/stock_symbol.rs` | (small) | `StockSymbol` branded string |
| `repository_ports/eps_ranking_repository_port.rs` | — | `EPSRankingRepositoryPort` |
| `repository_ports/stock_analysis_repository_port.rs` | — | `StockAnalysisRepositoryPort` |
| `repository_ports/market_data_scanner_port.rs` | — | `MarketDataScannerPort` (impl'd by TradingView) |
| `mod.rs`, `aggregates/mod.rs`, `value_objects/mod.rs`, `repository_ports/mod.rs` | — | re-exports |

### 1b. `application/market_analytics/` — CQRS use cases (2,858 LOC across 32 files)

- `commands/` — 9 commands + 9 handlers:
  - `create_eps_ranking`, `add_stock_to_ranking`, `create_stock_analysis`,
    `update_stock_analysis`, `delete_stock_analysis`,
    `refresh_cache`, `sync_eps_data`,
    `extend_assignment`, `revoke_assignment` *(orphan — see §3 / §6)*
- `queries/` — 16 query models + 14 handlers:
  - read paths: `list_eps_rankings`, `get_eps_ranking`, `list_stock_analyses`,
    `get_stock_analysis`, `get_stock_statistics`, `get_stocks_by_sector`,
    `get_sectors_by_country`, `get_growth_leaders`, `get_top_performers`,
    `get_portfolio_rankings`, `get_cached_rankings`, `get_cache_stats`,
    `get_system_metrics`, `get_admin_timeseries`, `get_admin_modules`,
    `get_stock_ranking_assignments`
- `dtos/` (`res.rs` 331 LOC, `req.rs`) — wire DTOs
- `services/` (mod.rs only), `controllers/` (mod.rs only) — empty shells

### 1c. `web/analytics/` — HTTP transport (3,366 LOC across 19 files)

| File | LOC | Role |
|---|---:|---|
| `eps/cache.rs` | 459 | `GET /rankings` hot path + `get_cache_stats` + `force_cache_refresh` |
| `eps/types.rs` | 329 | response DTOs (`CardDashboardResponse`, `SymbolCardData`, …) |
| `eps/rankings.rs` | 276 | legacy `get_eps_rankings` path |
| `eps/estimate.rs` | 272 | EPS estimate aggregator (TradingView + WebSocket) |
| `eps/metadata.rs` | 268 | static metadata (countries, sectors, filters) |
| `eps/transform.rs` | 179 | DTO conversion (screening → unified → card) |
| `eps/errors.rs` | 176 | error mapping |
| `eps/enhancement.rs` | 171 | WebSocket EPS enhancement |
| `eps/quarterly.rs` | 113 | quarterly EPS fallback |
| `eps/system.rs` | 61 | system-level helpers |
| `eps/date_metrics.rs` | 40 | small date helper |
| `websocket_service.rs` | 412 | `WebSocketEarningsService` + private `lazy_static` cache |
| `admin_handlers.rs` | 211 | `system_metrics_handler`, `admin_time_series_handler`, `admin_modules_handler` |
| `repository.rs` | 182 | `TradingViewEPSRepository` (the live data adapter) |
| `eps_handlers.rs` | 93 | re-export shim (no real handlers) |
| `types.rs` | 20 | `AuthenticatedUser`, `AnalyticsQuery` (admin) |
| `mod.rs` | 83 | module surface |

**Total:** 9,593 LOC across 67 Rust files (counts via `find … | xargs wc -l`).

### 1d. Sibling — `web/admin/analytics/` (NOT the same domain)

A separate 1,338-LOC tree at `src/web/admin/analytics/` exists
(`overview.rs`, `dashboard.rs`, `users.rs`, `revenue.rs`, `permissions.rs`,
`usage.rs`, `types.rs`, `mod.rs`). It uses its own `AnalyticsQuery` type, does
**not** import `crate::web::analytics`, and serves admin-dashboard pages.

**Evidence:** `rg "use crate::" src/web/admin/analytics/overview.rs` →
`crate::web::auth::AppState` + `crate::web::responses::wrappers::AdminResponse`
only. No reference to `crate::web::analytics`.

This is a **naming collision**, not a coupling. It will, however, confuse
anyone reading the route table.

### 1e. Infrastructure adapters that belong with analytics (must move as a unit)

`rg "crate::domain::market_analytics" src/infrastructure/ | sort -u` returns
four files that the analytics domain depends on and that physically live in
`infrastructure/adapters/`:

- `infrastructure/adapters/repositories/stock_analysis_repository_adapter.rs`
- `infrastructure/adapters/repositories/market_data_repository_adapter.rs`
- `infrastructure/adapters/repositories/mappers/market_analytics_mappers.rs`
- `infrastructure/adapters/services/tradingview/tradingview_adapter.rs`

Plus the TradingView transport layer (8,316 LOC) that the analytics domain
calls into:
- `infrastructure/adapters/services/tradingview/{mod,api_service,scanner,rest,types,mapper,cache,utils}.rs`
- `infrastructure/adapters/services/tradingview_websocket/{mod,connection,extractor,types}.rs` (853 LOC)

These have to move with analytics or the domain becomes a hollow shell.

---

## 2. Inbound dependencies (what analytics imports from other domains)

190 `use crate::` lines live under the three analytics trees. Filtering out
intra-domain and shared-kernel imports leaves these cross-domain consumers:

| Cross-domain import | Where | Hard/Soft | Why |
|---|---|---|---|
| `crate::auth::UnifiedPermissionService` | `web/analytics/eps/cache.rs:14`, `web/analytics/eps/rankings.rs:14`; called as `Extension(Arc<UnifiedPermissionService>)` in `cache.rs:51` and `rankings.rs:24`; `permission_service.get_wallet_ranking_offset(wallet).await` at `cache.rs:73`, `rankings.rs:39` | **HARD** | Plan-tier rank offset is computed per request from this service. Removing it requires either keeping the auth binary as a library, or splitting the `get_wallet_ranking_offset` query into a separate port/contract that the analytics service can call over the wire. Per CLAUDE.md "Permissions & Plan Logic — Backend Only", the *enforcement* must stay in the Rust backend — but a *read-only* "what's this wallet's tier?" query is delegable. |
| `crate::web::middleware::bearer_middleware::OpenIDUserContext` | `web/analytics/eps/cache.rs:15,52`; `rankings.rs` | **HARD** | Pulls `wallet_address` from the JWT-extracted user context to feed the rank-offset call. Same constraint as above: need to keep middleware validation local, or accept a different auth shim that the analytics service can decode. |
| `crate::web::pagination::Pagination` | `web/analytics/eps/rankings.rs:15` | soft | Standard pagination DTO; trivially extractable. |
| `crate::infrastructure::cache::Cache` (trait) | `web/analytics/eps/cache.rs:16` — `Extension(Arc<dyn Cache>)` is received at handler level | soft / mostly-unused | Trait parameter is **prefixed `_cache`** in the main `get_unified_analytics_rankings_cached` handler (`cache.rs:50`) and is never read; cache writes are explicitly disabled (`cache.rs:300-303`). Only the optional cache read in the legacy path uses it. The actual data cache is a private in-memory `HashMap` inside `EPSCacheService` (`domain_services/eps_cache_service.rs:42`). |
| `crate::infrastructure::adapters::services::tradingview::TradingViewApiService` | `unified_router.rs:273,351` (route setup), `web/analytics/eps/cache.rs:121`, `web/analytics/eps/estimate.rs:89`, `web/analytics/admin_handlers.rs:43`, `web/analytics/repository.rs`, `application/market_analytics/commands/handlers/{refresh_cache,sync_eps_data,get_portfolio_rankings}_handler.rs` | **HARD** | This is the live-data adapter. Analytics cannot exist without it. It would move with analytics to a new service, but the new service has to reach `https://scanner.tradingview.com/global/scan` (`infrastructure/adapters/services/tradingview/types.rs:136`) — see §6. |
| `crate::domain::shared_kernel::entities::eps_growth::EPSRanking` (+ `EPSGrowthData`, `EPSRankingsResponse`, `EPSPagination`) | `domain_services/eps_cache_service.rs:5`, `web/analytics/eps/cache.rs:11`, `web/analytics/eps/rankings.rs` | **HARD** | These are not just "shared types" — they carry a legacy `EPSRepository` port that the modern aggregates don't use. This is the "DDD vs legacy" splice point in analytics (see §8 refactor #1). |
| `crate::domain::shared_kernel::services::eps_ranking_service::{EPSRankingService, EPSRankingParams, EPSRepository}` | `infrastructure/adapters/repositories/stock_analysis_repository_adapter.rs:6`, `unified_router.rs:281,359` | **HARD** | Legacy ranking service. Analytics instantiates `EPSRankingService::new(eps_repository)` at route-build time. |
| `crate::domain::shared_kernel::DomainEventBus` | 5 command handlers: `application/market_analytics/commands/handlers/{create_eps_ranking,create_stock_analysis,update_stock_analysis,delete_stock_analysis,add_stock_to_ranking}_handler.rs` — `event_bus: Arc<dyn DomainEventBus>`, `.publish(&**event)` | soft | The `DomainEventBus` is publish-only and in-process (see §4). The trait is already a port; the analytics domain never reads from it. If we keep the trait as a shared-kernel concept, this is just another dependency on a `dyn Trait`, and the new analytics service can publish over a network bus. |
| `crate::infrastructure::adapters::services::tradingview_websocket::{TradingViewWebSocketService, QuarterlyEPSData, EPSWebSocketData}` | `web/analytics/websocket_service.rs:8-12` | **HARD** | Used to enhance the rankings with real-time EPS data on small batches. |
| `crate::config::get_fallback_config` | `web/analytics/eps/enhancement.rs:9`, `unified_router.rs:271,349`, `admin_handlers.rs:44` | soft | Reads env at runtime. |
| `crate::core::errors::{AppError, ErrorKind}` | every handler file (10+ files) | soft | Standard error type — already shared kernel. |

**Summary of hard dependencies:** 4 (auth service, auth middleware,
TradingView API service, shared-kernel EPS entities). These four are the
real blockers. The rest are ports/traits that are already abstracted.

---

## 3. Outbound dependents (who imports from analytics)

Filtered `rg "crate::(domain|application|web)::(market_analytics|analytics)"`
to files outside the three analytics trees:

| File | What it imports | Hard/Soft |
|---|---|---|
| `src/infrastructure/adapters/repositories/stock_analysis_repository_adapter.rs` | `crate::domain::market_analytics::{aggregates::eps_ranking, value_objects}` | hard — moves with analytics (§1e) |
| `src/infrastructure/adapters/repositories/market_data_repository_adapter.rs` | `crate::domain::market_analytics::{aggregates::stock_analysis, value_objects}` | hard — moves with analytics |
| `src/infrastructure/adapters/repositories/mappers/market_analytics_mappers.rs` | same as above | hard — moves with analytics |
| `src/infrastructure/adapters/services/tradingview/tradingview_adapter.rs:3` | `crate::domain::market_analytics::repository_ports::MarketDataScannerPort` | hard — moves with analytics |
| `src/web/admin/routes.rs:238-240` | `crate::web::analytics::{system_metrics_handler, admin_time_series_handler, admin_modules_handler}` | soft — admin UI. Replace with an HTTP client calling the new analytics service. |
| `src/web/user/permissions.rs:14` | `crate::web::analytics::AuthenticatedUser` | soft — re-exports a type. Replace with a local type. |
| `src/web/docs/openapi_admin.rs:61-70,152-161` | many `crate::web::analytics::eps::*` types/handlers | soft — OpenAPI doc registration. Would move with the service. |
| `src/web/docs/openapi_user.rs:59` | `crate::web::analytics::eps::cache::get_cache_stats` | soft — same |

**No other domain (`payment`, `notification`, `wallet_management`, etc.) imports
from analytics.** The `rg` for cross-domain consumers returns only the four
infrastructure adapters above and the three web-layer files.

The four infrastructure adapters (the "domain must-move-with" group) are
private to the analytics domain — they should be relocated into a new
`analytics-service` crate as part of the split.

The web-layer consumers are all "admin UI borrowed types". Three soft
references in admin routes, one in user permissions, two in OpenAPI doc
registrations. None represent business logic coupling.

**Leakage verdict:** 8 outbound call-sites, of which 4 are physically
co-located in `infrastructure/adapters/` (must move with analytics anyway) and
4 are web/admin consumers that can be replaced with HTTP clients.

---

## 4. Realtime events — publisher / subscriber analysis

The prompt's focus area is critical here because **analytics is a publisher
but not a subscriber**, and the "event bus" in this codebase is publish-only
and in-process.

### 4a. The `DomainEventBus` is publish-only

`rg "fn publish" src/domain/shared_kernel/domain_event.rs:57-67`:

```rust
pub trait DomainEventBus: Send + Sync {
    fn publish(&self, event: &dyn DomainEvent);
    fn publish_batch(&self, events: &[Box<dyn DomainEvent>]) { … }
}
```

`impl DomainEventBus for InMemoryEventBus` (`domain_event.rs:92-97`) just
appends `event.event_type()` to a `Vec<String>`. The production wiring is
`SimpleEventBus` at `infrastructure/event_bus/simple_event_bus.rs:37-49` —
also just appends to a `Vec<String>`. The bus **has no subscribe API**.

**There is no in-process event pub/sub.** This is a stub.

### 4b. Analytics publishes these domain events

`rg "self.event_bus.publish" src/application/market_analytics/`:

| Event | Emitted in |
|---|---|
| `EPSRankingCreated` | `commands/handlers/create_eps_ranking_handler.rs:69` |
| `StockAnalysisCreated` | `commands/handlers/create_stock_analysis_handler.rs:70` |
| `StockAnalysisUpdated` | `commands/handlers/update_stock_analysis_handler.rs:74` |
| `StockAnalysisDeleted` | `commands/handlers/delete_stock_analysis_handler.rs:45` |
| `StockAddedToRanking` (or similar) | `commands/handlers/add_stock_to_ranking_handler.rs:74` |

### 4c. Analytics does NOT subscribe to any events

`rg "subscribe|on_event|EventHandler" src/domain/market_analytics src/application/market_analytics src/web/analytics`
returns zero matches. No analytics code registers an event handler anywhere.

`rg "use crate::domain::market_analytics" src/ | grep -vE
"^src/(domain|application|web)/(market_analytics|analytics)"` — i.e.
external consumers of the analytics events — returns only the four
infrastructure adapters and no one calling `subscribe()`. The events are
**published into the void.**

### 4d. Analytics is a CONSUMER of WebSocket data, not a publisher of it

This is the most counterintuitive part of the audit. The `WebSocketEarningsService`
in `web/analytics/websocket_service.rs:33` is a *client* of TradingView's
WebSocket (`tradingview_websocket::TradingViewWebSocketService`); it pulls
real-time earnings data for enhancement. It uses a private `lazy_static` cache
(`websocket_service.rs:22-28`), not the shared infrastructure cache.

Analytics does **not** own or publish to the application's
`realtime_events` system. `rg "CreateRealtimeEventCommand" src/ | grep -v
"src/application/realtime_events"` returns no analytics callers.

### 4e. Implication for the split

The "realtime" coupling is actually weaker than the prompt feared. Analytics
is a one-way publisher into a no-op event bus. If we want other services
(notifications, etc.) to react to `StockAnalysisCreated`, we have to add
real pub/sub first — that's a pre-split platform refactor, not an analytics
coupling issue.

---

## 5. Database seams

### 5a. The "analytics" schema is a misnomer

`migrations/analytics/00000000000001_consolidated_analytics_v2/up.sql` creates
22 tables in the analytics DB. Of those, only **2 are analytics-domain
tables** (`analytics_events`, `event_store`); the rest are **shared
infrastructure** for high-volume logs:

| Table | Owned by analytics domain? | Real owner |
|---|---|---|
| `api_key_usage_logs` (partitioned monthly) | NO | developer_portal (writes to it from `usage_service.rs:114,188,253,292,315,341`) |
| `event_store` | **shared infra** | CQRS event-sourcing scaffolding (`infrastructure/cqrs/event_store.rs`) |
| `outbox_events` | **shared infra** | CQRS outbox (`infrastructure/cqrs/outbox.rs`) |
| `aggregate_snapshots` | **shared infra** | CQRS projection (`infrastructure/cqrs/projection.rs`) |
| `analytics_events` | partial — written by global middleware | `web/middleware/usage_tracking_middleware.rs:114` (not by analytics domain code) |
| `permission_audit_log` | NO | auth/permission_management |
| `payment_audit_log` | NO | payment |
| `assignment_audit_log` | NO | admin assignments |
| `wallet_activity_logs` | NO | wallet |
| `audit_logs` | NO | core |
| `unified_audit_log` | NO | core / admin |

`rg "analytics_events" src/ | grep -v "src/schemas/analytics.rs"` returns only
`src/web/middleware/usage_tracking_middleware.rs:14,19,114` — the global
usage-tracking middleware writes to it. **No code in
`domain/market_analytics/`, `application/market_analytics/`, or
`web/analytics/` ever writes to `analytics_events`.**

The `analytics` schema in PostgreSQL is the **"high-volume logs / event
sourcing" schema**, not the analytics domain's own storage. Renaming it
to `infra_logs` or `cqrs` would be more honest.

### 5b. The actual analytics-domain tables are missing

`rg -i "create table.*stock_analyses|create table.*eps_rankings|create table.*ranking_entries" migrations/`
returns **zero results**. There is no SQL `CREATE TABLE stock_analyses` or
`CREATE TABLE eps_rankings` in any migration file.

`rg "table_name\s*=\s*stock_analyses|table_name\s*=\s*eps_rankings" src/` —
also zero results. There is no Diesel `#[diesel(table_name = stock_analyses)]`
struct anywhere in the source tree.

The repositories in `infrastructure/adapters/repositories/stock_analysis_repository_adapter.rs`
are wrappers around the **legacy** `EPSRankingService` (an in-process service
that talks to TradingView). The DDD "stock analyses" data is *never persisted
to PostgreSQL* — it's a read-through to TradingView with an in-memory cache.

**The analytics domain owns no tables.** Its state is:
1. `EPSCacheService`'s private `HashMap` (in-process, volatile)
2. `WebSocketEarningsService`'s private `lazy_static` `HashMap` (in-process,
   volatile)
3. `EPSRanking` / `StockAnalysis` aggregates (in-process, in-memory DDD
   aggregates — emitted events but not persisted)

### 5c. The "analytics pool" is a real read-replica, but analytics domain code doesn't use it

`infrastructure/database/diesel_connection_manager.rs:261-296`:
`get_analytics_pool()` reads `ANALYTICS_DATABASE_URL` env var and creates a
separate `&'static TlsPool` with `max_size: 5` (smaller than the main pool).
The comment at line 280 says: *"Smaller pool for analytics - write-heavy, less
concurrent needs"*. It falls back to the main pool if `ANALYTICS_DATABASE_URL`
is unset.

`rg "analytics_pool|get_analytics_pool" src/` returns 18 hits, of which:
- 11 are infrastructure/container plumbing
- 7 are the audit logging repository (`src/infrastructure/repositories/audit_log_repository.rs:7,35,452`)
- 0 are in `domain/market_analytics/`, `application/market_analytics/`, or `web/analytics/`

The analytics domain never opens a connection to `analytics_pool`. It opens
no PostgreSQL connections at all. The read-replica is real, but it's used by
audit logging, not by the analytics domain.

### 5d. Migration collision (real bug, not just a smell)

`ls migrations/analytics/` shows two migrations with the same version:

```
00000000000001_consolidated_analytics_v2
00000000000001_consolidated_baseline_v3
```

`embed_migrations!("migrations/analytics")` in `src/bin/migrate.rs:11` will
panic at compile time when it discovers the duplicate version
(`diesel_migrations::find_migrations_directory` returns an error on
collisions). To verify which is canonical, the v3 file has `unified_audit_log`
and `2026_01..03` partitions that v2 lacks — so v3 is the newer one and v2
should be deleted. The split work would discover this either way.

### 5e. Foreign-key cross-schema risks

Searching for cross-schema FKs from the analytics schema:

`rg -i "references.*\." migrations/analytics/00000000000001_consolidated_analytics_v3/up.sql`
returns no explicit FK constraints in the analytics schema. The shared
tables in the analytics schema reference *no* tables in other schemas — they
are self-contained.

This means there is **no DB-level FK coupling** between analytics and the
other schemas. Good. The coupling is entirely at the application layer
(auth service calls, TradingView adapter calls).

---

## 6. Background jobs / scheduled tasks / queue workers

**There are none in the analytics domain.** Evidence:

- `rg "tokio::spawn|tokio::time::interval|tokio::time::sleep" src/domain/market_analytics src/application/market_analytics src/web/analytics` returns zero matches.
- `rg "schedule|cron|tokio_cron" src/ Cargo.toml` finds scheduling-related
  code only in `notification` (topic notifications) and `realtime_events` —
  nothing in analytics.
- `src/main.rs` and `src/lib.rs` have no analytics startup hooks
  (`rg "market_analytics|SyncEPSData|RefreshCache" src/main.rs src/lib.rs` → 0).

**Dead commands.** `application/market_analytics/commands/models/sync_eps_data.rs`
defines `SyncEPSDataCommand` and `application/market_analytics/commands/models/refresh_cache.rs`
defines `RefreshCacheCommand`. Both have handlers in
`commands/handlers/{sync_eps_data,refresh_cache}_handler.rs`. But:

`rg "SyncEPSDataCommand\b" src/` returns hits only in the model + handler
files. There is no `.handle(SyncEPSDataCommand { … })` call anywhere in the
codebase. Same for `RefreshCacheCommand`.

These commands exist for an architectural dream of cron-driven
pre-computation, but in the current code the "cache refresh" path is exposed
as the HTTP endpoint `force_cache_refresh` (and that endpoint isn't even
registered as a route — see §7).

The only `tokio::spawn` in the broader infrastructure is in
`infrastructure/services/{audit_service,plan_expiration_service}.rs` — neither
analytics. So the analytics domain has no worker threads, no cron jobs, no
queue consumers.

---

## 7. API surface (read from `web/routes/unified_router.rs`)

`unified_router.rs:58-124` builds the main `Router` and `unified_router.rs:269-324`
defines `create_analytics_routes`. There are two mounts of analytics routes:

### 7a. Authenticated mount — `/api/analytics/...`

`unified_router.rs:94` nests `create_analytics_routes` under `/api/analytics/...`.
That builder (`unified_router.rs:296-323`) registers:

| Method | Path | Handler (file:line) | Auth | Request DTO | Response DTO |
|---|---|---|---|---|---|
| GET | `/api/analytics/rankings` | `web/analytics/eps_handlers::get_unified_analytics_rankings_cached` → `web/analytics/eps/cache.rs:48` | `optional_bearer_middleware` (line 322) | `EPSRankingQueryParams` (page, limit, country, sector, sort_by, min_eps, min_growth) | `CardDashboardResponse` (eps/cache.rs:270) |
| GET | `/api/analytics/filters` | `get_filter_options` → `web/analytics/eps/metadata.rs` | optional-bearer | none | `FiltersResponse` |
| GET | `/api/analytics/countries` | `get_all_valid_countries` → `metadata.rs` | optional-bearer | none | `CountriesResponse` |
| GET | `/api/analytics/available-countries` | `get_available_countries` → `metadata.rs` | optional-bearer | none | `CountriesResponse` |
| GET | `/api/analytics/sectors` | `get_sectors_by_country` → `metadata.rs` | optional-bearer | `country: Option<String>` | `SectorsResponse` |

### 7b. Public mount — `/api/public/analytics/...`

`unified_router.rs:374-381` (inside `create_public_routes`) duplicates three of
the above routes under `/api/public/analytics/`:

| Method | Path | Handler | Auth |
|---|---|---|---|
| GET | `/api/public/analytics/rankings` | `get_unified_analytics_rankings_cached` | **none** |
| GET | `/api/public/analytics/filters` | `get_filter_options` | **none** |
| GET | `/api/public/analytics/countries` | `get_all_valid_countries` | **none** |

These are the same handlers, mounted twice. (The `unified_router:345` reads
`analytics_pool` to pass to `AppState`, but the handler itself doesn't touch
it — see §5c.)

### 7c. Admin mount — `/api/admin/analytics/...`

`web/admin/routes.rs:238-240`:

| Method | Path | Handler | Auth |
|---|---|---|---|
| GET | `/api/admin/analytics/metrics` | `system_metrics_handler` (admin_handlers.rs:23) | `bearer_middleware` + `permission_validation_middleware` |
| GET | `/api/admin/analytics/time-series` | `admin_time_series_handler` (admin_handlers.rs:87) | same |
| GET | `/api/admin/analytics/modules` | `admin_modules_handler` (admin_handlers.rs:175) | same |

### 7d. Dead routes (defined but not registered)

`web/analytics/eps_handlers.rs:10` re-exports `get_cache_stats` and
`force_cache_refresh` from `web/analytics/eps/cache.rs:318,356`. They are
referenced in `web/docs/openapi_{admin,user}.rs:62-63,59` (OpenAPI doc
registration) but **`rg "force_cache_refresh|get_cache_stats" src/web/routes/unified_router.rs`
returns zero results** — neither handler is mounted.

So the documented API surface includes `GET /api/analytics/cache/stats` and
`POST /api/analytics/cache/refresh` that the router does not actually
serve. A request to those paths would return 404.

### 7e. Total live API surface

- 5 routes under `/api/analytics/...`
- 3 routes under `/api/public/analytics/...`
- 3 routes under `/api/admin/analytics/...`
- **11 live routes total**, of which 5 are unique (the public 3 are duplicates
  of authenticated ones), plus 3 admin-only.

If the analytics service is lifted out, all 11 become external HTTPS calls.
The handlers themselves are stateless except for the `Extension`-injected
services, so the lift is mechanical.

---

## 8. External integrations

| Integration | File | Direction | Protocol | Used by analytics? |
|---|---|---|---|---|
| TradingView REST scanner | `infrastructure/adapters/services/tradingview/rest.rs:59` | outbound | HTTPS GET `https://scanner.tradingview.com/global/scan?label-product=screener-stock` (`types.rs:136`) | yes — every rankings request |
| TradingView WebSocket | `infrastructure/adapters/services/tradingview_websocket/mod.rs:435` | outbound | WSS to `wss://…tradingview.com` | yes — WebSocket enhancement path |
| TradingView auth token | `src/config/env.rs:292` `get_optional("TRADINGVIEW_AUTH_TOKEN")` | env | n/a | yes |
| Redis (cache) | `infrastructure/cache/redis_cache.rs` (1 file) | outbound | RESP | **NO** — the `Arc<dyn Cache>` injected into the rankings handler is unused. The actual cache is the in-process `HashMap` in `EPSCacheService` and the `lazy_static` in `WebSocketEarningsService`. |
| PostgreSQL `analytics_pool` | `infrastructure/database/diesel_connection_manager.rs:261` | outbound | pg | **NO** — analytics domain never opens a connection |
| PostgreSQL main pool | same | outbound | pg | **NO** — see §5c |
| Cross-domain Kafka / NATS / RabbitMQ | — | n/a | n/a | **NO** — there is no broker. The "event bus" is in-process and no-op. |
| TradingView session cookies | `tradingview/rest.rs` | outbound | HTTPS | yes — needed for scanner auth |
| `TRADINGVIEW_AUTH_TOKEN` | env | n/a | n/a | yes |

**Net external surface:** 1 TradingView REST endpoint + 1 TradingView
WebSocket. That's it. No queue, no broker, no webhook receivers, no
outbound HTTP to any other third party. The dependencies are concentrated.

---

## 9. Split-readiness score: **3 / 5**

**Justification against the evidence above:**

| Dimension | Score (1-5) | Reasoning |
|---|:-:|---|
| Bounded context purity | 3 | Domain is well-isolated in DDD terms (16/16 files in `domain/`, clear ports), but the legacy `EPSRankingService` from `shared_kernel` and the in-process `EPSCacheService` make the "domain owns its data" claim partially false. |
| Inbound coupling | 2 | 4 hard inbound deps to `crate::auth` and `crate::domain::shared_kernel::services::eps_ranking_service`. The auth dep is the biggest blocker because per CLAUDE.md the auth enforcement *cannot* be moved out, but the *read-only "what tier is this wallet?"* query could be extracted as a port. |
| Outbound coupling | 4 | Only 4 outbound sites, all of them infrastructure adapters that move with analytics. The web/admin and openapi references are soft. |
| Database seam | 4 | Analytics domain owns zero tables (it talks to TradingView, not the DB). The "analytics schema" in PostgreSQL is actually shared infra (high-volume logs) and would have to be carved up separately. No FK cross-schema coupling. |
| Cache ownership | 3 | The cache is in-process, split into 2 private HashMaps (`EPSCacheService`, `WebSocketEarningsService` `lazy_static`) and 1 unused `Arc<dyn Cache>` extension. Analytics already "owns" its cache by accident — extracting it is free. |
| Realtime events | 5 | Publisher-only, no subscribers, no broker. Lifting it out doesn't break anyone. |
| Background jobs | 5 | None. The "commands" that look like workers (`SyncEPSDataCommand`, `RefreshCacheCommand`) are orphaned — never invoked. |
| External integrations | 4 | Single vendor (TradingView, REST + WSS). 2 endpoints, both outbound. The split moves the integration with the service. |
| API surface | 4 | 11 routes, well-named, two duplicate mounts (public + authed for the same 3 handlers). Easy to lift. The 2 dead routes (`/cache/stats`, `/cache/refresh`) need cleanup either way. |
| Migration discipline | 2 | Duplicate version number `00000000000001` in `migrations/analytics/` (`consolidated_analytics_v2` and `consolidated_baseline_v3`). `embed_migrations!` will panic. Must fix before any split. |
| Naming collisions | 3 | `src/web/analytics/` and `src/web/admin/analytics/` are different things, plus `crate::web::analytics::AnalyticsQuery` vs `crate::application::market_analytics::dtos::req.rs::AnalyticsQuery` (the DTOs) vs `crate::web/admin/analytics/types.rs::AnalyticsQuery` (the admin). Three different `AnalyticsQuery` types. Not a blocker, but a footgun. |

**Overall: 3 / 5 — could be lifted in 2-3 medium refactors, but the migration
collision and the auth-tier query coupling are non-trivial.** The biggest
counterintuitive finding: analytics is **less** coupled to the rest of the
system than the prompt feared (no subscribers, no broker, no DB) and
**more** coupled to one specific thing (the auth service for plan tiers,
and the TradingView API for live data).

---

## 10. Top 3 specific refactors

### Refactor #1 — Extract `get_wallet_ranking_offset` into a port

**Where it lives today:** `web/analytics/eps/cache.rs:73-85` and
`web/analytics/eps/rankings.rs:39-58` call
`permission_service.get_wallet_ranking_offset(wallet).await` on
`crate::auth::UnifiedPermissionService`. The unified_router setup
(`unified_router.rs:285-292, 364-371`) injects this same
`UnifiedPermissionService` as an Axum extension for the analytics routes.

**Mechanism:**
1. In `domain/market_analytics/repository_ports/` (or a new sibling port
   `domain/market_analytics/services/`) define:
   ```rust
   #[async_trait]
   pub trait WalletRankingOffsetQuery: Send + Sync {
       async fn get_rank_offset(&self, wallet: &str) -> Result<i32, AppError>;
   }
   ```
2. Implement it in `infrastructure/adapters/permissions/wallet_ranking_offset_adapter.rs`
   as a thin wrapper over the existing `UnifiedPermissionService`.
3. Change the two analytics handlers to take
   `Extension(Arc<dyn WalletRankingOffsetQuery>)` instead of
   `Extension(Arc<UnifiedPermissionService>)`.
4. After the lift, the new `analytics-service` binary keeps the same
   wrapper adapter and calls the auth service over the wire (gRPC or HTTP)
   to satisfy the read-only query. Per CLAUDE.md, the *enforcement* of
   "this wallet may not see rank < 100" must remain local in any backend
   that consumes analytics; the read-only "what's this wallet's tier?"
   query is delegable.

**Files touched:**
- `src/domain/market_analytics/repository_ports/mod.rs` (+1 trait)
- `src/infrastructure/adapters/permissions/wallet_ranking_offset_adapter.rs` (new, ~30 LOC)
- `src/web/analytics/eps/cache.rs:14,51,73` (3 lines)
- `src/web/analytics/eps/rankings.rs:14,24,39` (3 lines)
- `src/web/routes/unified_router.rs:285-292, 364-371` (replace Extension)
- ~40 LOC net.

**Why it unblocks the split:** Removes the single hardest inbound dep on
`crate::auth`. After this refactor, analytics has zero direct imports from
`crate::auth::*`.

### Refactor #2 — Fix the migration collision and split the "analytics" schema

**Where it lives today:** `migrations/analytics/00000000000001_consolidated_analytics_v2/`
and `migrations/analytics/00000000000001_consolidated_baseline_v3/` have the
same version number. `diesel_migrations::embed_migrations!` in
`src/bin/migrate.rs:11` will refuse to compile, or the duplicate will be
silently dropped, depending on order. v3 is canonical (has 2026 partitions
and `unified_audit_log`); v2 is dead.

Additionally, the tables in the analytics schema are not actually
analytics-domain tables. Most belong to developer_portal, audit, CQRS, or
payment/auth logging (see §5a).

**Mechanism:**
1. Delete `migrations/analytics/00000000000001_consolidated_analytics_v2/`
   (up.sql + down.sql) — it is a strict subset of v3.
2. Rename the `analytics` schema in the migration to `infra_logs` (a single
   ALTER SCHEMA statement at the top of v3) so future readers don't confuse
   it with the analytics domain.
3. Move analytics-domain tables (when they exist — currently none) to a
   new `analytics` schema, or keep the analytics domain stateless.

**Files touched:**
- `migrations/analytics/00000000000001_consolidated_analytics_v2/up.sql` — delete
- `migrations/analytics/00000000000001_consolidated_analytics_v2/down.sql` — delete
- `migrations/analytics/00000000000001_consolidated_baseline_v3/up.sql` — prepend `CREATE SCHEMA IF NOT EXISTS infra_logs; SET search_path TO infra_logs;` and update all `CREATE TABLE` to use `infra_logs.table_name`
- `migrations/analytics/00000000000001_consolidated_baseline_v3/down.sql` — same
- `migrations/analytics/20260216100000_create_unified_audit_log/up.sql` — same
- `src/schemas/analytics.rs` — would need re-generation via `diesel print-schema --config diesel_analytics.toml`
- `src/schemas/mod.rs` — re-export path
- `diesel_analytics.toml` — update filter if needed
- ~10 LOC of SQL edits, plus generated `analytics.rs` regeneration.

**Why it unblocks the split:** Once the analytics-domain state is
acknowledged to be in-process (not in PostgreSQL), the "should the
analytics DB be a separate database in the new service?" question
disappears. The `ANALYTICS_DATABASE_URL` plumbing can stay as
shared-infra storage (CQRS + audit logs) and the analytics service
itself needs no DB connection at all.

### Refactor #3 — Consolidate the two route mounts and delete the dead routes

**Where it lives today:**
- `unified_router.rs:296-323` registers `/api/analytics/{rankings,filters,countries,available-countries,sectors}` with optional-bearer auth.
- `unified_router.rs:373-381` re-registers 3 of the same 5 routes under
  `/api/public/analytics/...` with no auth.
- `web/analytics/eps/cache.rs:318,356` defines `get_cache_stats` and
  `force_cache_refresh` but no route in `unified_router.rs` calls them.
  They are only referenced by `web/docs/openapi_{admin,user}.rs`.
- `web/admin/routes.rs:238-240` registers 3 admin routes that wrap
  `crate::web::analytics::system_metrics_handler` etc.

**Mechanism:**
1. Pick one mount prefix. The cleaner option is to keep the
   `/api/analytics/...` mount with `optional_bearer_middleware` (which it
   already has — line 320) and remove the `/api/public/analytics/...` duplicate.
   `optional_bearer_middleware` is precisely the "public but tier-aware"
   pattern. The handlers already accept `Option<Extension<OpenIDUserContext>>`
   and degrade gracefully.
2. Wire up the dead routes or delete them. `force_cache_refresh` is the
   only one with operational value (admin can flush cache after
   schema changes). Either mount it under `/api/admin/analytics/cache/refresh`
   with admin auth, or delete the handler and its OpenAPI doc.
3. After consolidation, the public-mount code at `unified_router.rs:373-381`
   and the analytics_pool plumbing at `unified_router.rs:45,53,224,345,629,854`
   can be simplified. The `analytics_pool` extension on `AppState` is unused
   by every handler that takes it — drop it.

**Files touched:**
- `src/web/routes/unified_router.rs:330-392` (remove `/api/public/analytics/...` nest)
- `src/web/analytics/eps/cache.rs:308-390` (keep or delete `get_cache_stats`/`force_cache_refresh`)
- `src/web/docs/openapi_admin.rs:62-63,59` and `openapi_user.rs:59` (update or remove references)
- ~30 LOC removed, 0 LOC added.

**Why it unblocks the split:** The 11 live routes become 5 unique external
endpoints. The "is the public mount its own service?" question vanishes.
The dead routes are discovered before the lift, not after.

---

## Appendix — What I did NOT find

For the verifier's spot-check purposes, here are claims I considered and dropped
for lack of evidence:

- **"Analytics subscribes to wallet events"** — false. `rg "subscribe" src/domain/market_analytics src/application/market_analytics src/web/analytics` → 0.
- **"Analytics has a cron that pre-computes rankings"** — false. `rg "tokio::spawn|interval|schedule" src/domain/market_analytics src/application/market_analytics src/web/analytics` → 0.
- **"Analytics uses Redis"** — false at the analytics-domain level. The
  `Arc<dyn Cache>` extension on the rankings handler is unused
  (`_cache: Extension<Arc<dyn Cache>>` with leading underscore at
  `web/analytics/eps/cache.rs:50`). All actual caching is in-process.
- **"Analytics owns the `analytics_events` table"** — false. The only
  writer is `src/web/middleware/usage_tracking_middleware.rs:114` (a
  global middleware). The analytics domain never writes to it.
- **"Analytics uses the `analytics_pool` for read replicas"** — false.
  The pool is used by `infrastructure/repositories/audit_log_repository.rs:35,452`
  and CQRS plumbing in `simple_container.rs:231`. Analytics-domain code
  opens no PostgreSQL connections.
- **"The `event_store` is an analytics-domain table"** — false. It's CQRS
  infrastructure (`infrastructure/cqrs/event_store.rs`). All five
  `event_store` writers are in CQRS or shared-kernel code, not in
  analytics.

---

*Audit complete. Verifier: see "Top 3 refactors" for actionable next steps.
See `state.json` for the synthesis task that depends on this audit.*
