# Payments Split-Readiness Audit (Wave 8)

> **Scope.** EPSX Rust backend — payments bounded context. Read-only audit; no
> refactors performed. All claims are backed by `file:line` evidence measured
> against `wave8/service-boundary` at `373bd231`.
>
> **Worktree.** Findings were assembled in
> `/Users/fluke/.mavis/plans/plan_f2d63ead/outputs/payments-audit` (branch
> `wave8/audit-payments`). The report itself lives on the base worktree at
> `docs/wave8-service-boundary/audit-payments.md` per task instructions.

---

## 1. Bounded-context inventory

The payments bounded context is reasonably well isolated at the DDD layer
(`domain/payment/*`, `application/payment/*`), but the **web and infrastructure
layers leak in both directions** because handlers were written directly against
`diesel` + `schemas::payments`/`schemas::primary` instead of going through the
`PaymentRepositoryPort` adapter.

### File counts and LOC

| Area | Files | LOC | Notes |
|------|------:|----:|-------|
| `apps/backend/src/domain/payment/` | 19 | 4,769 | aggregates (6) + value_objects (11) + repository_ports (1) + mod.rs (1) + payment_tests (1) |
| `apps/backend/src/application/payment/` | 4 | 365 | commands module: `create_payment_command`, `activate_subscription_command`, `mod.rs` (and one commented-out `handlers`) |
| `apps/backend/src/web/payments/` | 16 | 4,751 | top-level handlers (9) + `admin_handlers/` (4) + `admin/dtos.rs` + `mod.rs` |
| `apps/backend/src/infrastructure/adapters/repositories/payment_*_adapter.rs` | 2 | 929 | `payment_repository_adapter` (506), `payment_context_repository_adapter` (423) |
| `apps/backend/src/infrastructure/adapters/repositories/credit_repository_adapter.rs` | 1 | 377 | |
| `apps/backend/src/infrastructure/models/payment.rs` | 1 | 211 | `PaymentDb`, `NewPaymentDb`, `SubscriptionDb`, `NewSubscriptionDb` |
| `apps/backend/src/infrastructure/models/credit.rs` | 1 | 182 | `CreditBalanceResponse`, `CreditTransactionResponse`, `CreditStatsResponse`, request DTOs |
| `apps/backend/src/infrastructure/blockchain/` (payment-coupled modules) | 6 | 2,526 | `payment_verifier` (204), `bsc_listener` (196), `tx_monitor_service` (1,122), `event_parser` (219), `validation_client` (363), `contract_subscriber` (422) — `rpc_history_provider` and `scanner_history_provider` are shared with permission web3 and not counted in the payment total |
| `apps/backend/src/infrastructure/services/blockchain_monitor.rs` | 1 | ~250 (from earlier listing) | Background service that calls `BscEventListener` and writes to `payments`/`subscriptions` |
| `apps/backend/migrations/payments/` | 7 SQL files | — | `00000000000000_diesel_initial_setup`, `00000000000001_consolidated_baseline_v4`, `00000000000001_consolidated_payments_v3`, `20260212100000_add_credit_wallet`, `20260220100000_add_unique_tx_hash_and_expiry`, `20260303100000_add_audit_log_tx_hash`, `20260304100000_add_confirming_status` |
| `apps/backend/diesel_payments.toml` | 1 | 11 | Pins the Diesel schema generator to payments tables only |

**Payment-only subtotal: 58 files, ~14,360 LOC** (44 Rust files + 7 SQL
migrations + 1 Diesel config + 6 blockchain modules that are entirely
payment-domain).

### Payment-owned database tables (logical "payments" schema)

Defined in `apps/backend/diesel_payments.toml:7` filter and the migrations
listed above. All live in the **public** Postgres schema today — there is no
`CREATE SCHEMA payments` statement in any migration (see §4).

| Table | Source migration | Purpose |
|-------|------------------|---------|
| `payments` | `consolidated_payments_v3/up.sql` (v3 only) | Core payment record |
| `subscriptions` | same | Active subscriptions (FK to `payments.id` only, **no** FK to `plans`) |
| `stock_ranking_assignments` | same | Stock-ranking package assignments |
| `payment_contexts` | same | V2 dynamic payment links |
| `payment_audit_log` | same | Status-change audit trail (FK to `payments.id` ON DELETE CASCADE) |
| `wallet_credits` | `20260212100000_add_credit_wallet` | Per-wallet credit balance |
| `credit_transactions` | same | Credit movement history |

The SQL functions `add_credit_transaction`, `update_wallet_credits_timestamp`,
`update_payment_updated_at`, `update_payment_context_updated_at`,
`log_payment_status_changes`, and triggers
`payment_updated_at`, `payment_context_updated_at`, `payment_status_audit`
are owned by these migrations.

### Domain entities, value objects, and ports (DDD layer)

- **Aggregates** (`apps/backend/src/domain/payment/aggregates/`):
  `Payment` (`payment.rs`, 385 LOC), `PaymentContext`
  (`payment_context.rs`, 373), `PaymentStatus`, `PaymentMetadata`,
  `CryptoPaymentDetails`/`FiatPaymentDetails` (`payment_details.rs`), and a
  near-empty `payment_tests.rs` stub.
- **Value objects** (`apps/backend/src/domain/payment/value_objects/`):
  `PaymentId`, `PaymentReference`, `PaymentAmount`, `CryptoAddress`,
  `PaymentAddress`, `PaymentMethod`, `PaymentMethodId`, `CryptoNetwork`,
  `TransactionHash`, `Credit`, and 4 supporting modules (`mod.rs`).
- **Repository ports** (`apps/backend/src/domain/payment/repository_ports/mod.rs`):
  `PaymentRepositoryPort`, `TransactionRepositoryPort`,
  `CryptoAddressRepositoryPort`, `PaymentMethodRepositoryPort`,
  `PaymentContextRepositoryPort`, plus the support types `PaymentStats` and
  `TransactionRecord`.
- **Domain events** (declared in
  `apps/backend/src/domain/payment/aggregates/payment.rs` and re-exported
  via `mod.rs:45-55`): `PaymentCreated`, `PaymentAddressAssigned`,
  `PaymentConfirmed`, `PaymentCompleted`, `PaymentFailed`,
  `PaymentCancelled`, `PaymentRefundInitiated`, `PaymentRefundCompleted`.

---

## 2. Inbound dependencies (what payments depends on from other domains)

| File:line | Import | Hard / Soft | Notes |
|-----------|--------|-------------|-------|
| `apps/backend/src/domain/payment/aggregates/payment.rs:11` | `use crate::domain::wallet_management::value_objects::WalletAddress;` | **Hard** | The `Payment` aggregate's primary identity field is a `WalletAddress`. Cannot extract payments without taking `wallet_management` (or splitting `WalletAddress` into `shared_kernel`). |
| `apps/backend/src/domain/payment/repository_ports/mod.rs:6` | `use crate::domain::wallet_management::value_objects::WalletAddress;` | **Hard** | `PaymentRepositoryPort::find_by_user(&self, wallet: &WalletAddress)` — also a real domain dependency. |
| `apps/backend/src/domain/payment/value_objects/payment_id.rs` (and 3 other VOs) | `use crate::domain::shared_kernel::{ValueObject, AggregateBase, Identity, ...}` | **Soft** | `shared_kernel` is the DDD shared kernel; by design it lives across bounded contexts. The `AggregateBase`, `ValueObject`, `DomainEvent` traits are meant to be shared — moving them is a *placement* question, not a coupling question. |
| `apps/backend/src/application/payment/commands/create_payment_command.rs:11` | `use crate::domain::wallet_management::value_objects::WalletAddress;` | **Hard** | `CreatePaymentCommand.wallet_address: WalletAddress` — same `WalletAddress` VO leaks into the application command. |
| `apps/backend/src/application/payment/commands/activate_subscription_command.rs:4` | `use crate::domain::shared_kernel::value_objects::UserId;` | **Soft** | `UserId` is in `shared_kernel`; field is `wallet_address: UserId` (the comment in the struct says wallet, the type is `UserId` — these are aliases, see `domain/shared_kernel/value_objects/user_id.rs`). |
| `apps/backend/src/infrastructure/adapters/repositories/payment_repository_adapter.rs:18` | `use crate::domain::wallet_management::value_objects::WalletAddress;` | **Hard** | The Postgres adapter implements `PaymentRepositoryPort` which takes `WalletAddress`. |
| `apps/backend/src/web/payments/validation_handlers.rs:23` | `use crate::auth::{UnifiedPermissionService, GrantPermissionRequest};` | **Hard** | `activate_subscription_handler` (`validation_handlers.rs:194`) calls `permission_service.grant_permission(...)` directly. `UnifiedPermissionService` is a concrete struct (not a port) that lives in `crate::auth` (top-level, not in any bounded context). The handler also takes `Extension(permission_service): Extension<Arc<UnifiedPermissionService>>` — extraction is by concrete type, so this is structural, not interface. |

**No other cross-domain imports** in `domain/payment/`, `application/payment/`,
or `web/payments/`. The `web/payments/admin_handlers/payment_handlers.rs:16`,
`web/payments/admin_handlers/subscription_handlers.rs:16`,
`web/payments/admin_handlers/analytics_handlers.rs:13`,
`web/payments/user_payment_handlers.rs:25-26`, and
`web/payments/get_tx_status_handler.rs:20-21` all import
`schemas::primary::plans` — this is **NOT** a domain import (schemas is the
Diesel-generated code in the same crate), but it is a hard data-coupling on
the `plans` table (see §4 and §3).

**Summary:** 3 hard inbound imports (all `WalletAddress` or
`UnifiedPermissionService`), 4 soft `shared_kernel` imports that are
intentional.

---

## 3. Outbound dependents (what other domains import from payments)

| File:line | Import | Hard / Soft | Notes |
|-----------|--------|-------------|-------|
| `apps/backend/src/web/admin/payment_link_handlers.rs:20,23` | `PaymentContextRepositoryAdapter`, `NewPaymentContextDb`, `PaymentContextDb`, `UpdatePaymentContextDb`, `is_context_usable` from `payment_context_repository_adapter` | **Hard** | The admin public endpoint `GET /api/public/payment-links/{slug}` (registered in `unified_router.rs:385`) lives in `web/admin/` (NOT `web/payments/`) and reaches directly into the payments infrastructure layer. The handler is 460+ LOC and the public `/api/public/payment-links/{slug}` route would break if the `payment_context_repository_adapter` is moved. |
| `apps/backend/src/web/admin/plans/handlers.rs:6,30` | `SubscriptionDb`, `NewSubscriptionDb` from `infrastructure::models::payment` | **Hard** | The admin plans editor imports the payment-bounded `SubscriptionDb` model to insert/update `subscriptions` rows. Lifting payments to a separate service would require an HTTP/port call here. |
| `apps/backend/src/infrastructure/adapters/repositories/subscription_repository_adapter.rs:1-50` | `use crate::infrastructure::models::payment::{SubscriptionDb, NewSubscriptionDb};` | **Hard** | The "subscription" repository adapter (lives in `adapters/repositories/`, not in the payment area) writes to the `subscriptions` table using the payment's `SubscriptionDb` struct. This is the **strongest** outward leak — code that *should* live in payments sits in the central infrastructure layer. |
| `apps/backend/src/infrastructure/blockchain/rpc_history_provider.rs:11,12` | `use crate::domain::payment::repository_ports::{TransactionHistoryProvider, TransactionHistoryInfo};` and `use crate::infrastructure::blockchain::event_parser::parse_payment_event;` | **Soft** | The RPC history provider implements the payment port (`TransactionHistoryProvider`). Splitting payments would let this stay if the port moves with it, OR a payment client would replace the in-process call. |
| `apps/backend/src/infrastructure/blockchain/scanner_history_provider.rs:11,12` | Same as above for scanner | **Soft** | Same situation as RPC. |
| `apps/backend/src/infrastructure/container/simple_container.rs:18,31,37` | `PaymentRepositoryAdapter`, `PaymentRepositoryPort`, `TransactionHistoryProvider`, `ContractSubscriber`, `PaymentEvent` | **Soft / Hard (mixed)** | DI wiring. Soft for ports; hard for the concrete `PaymentRepositoryAdapter` (constructor takes the concrete pool). |
| `apps/backend/src/web/auth/app_state.rs:1` | `// use crate::infrastructure::adapters::repositories::payment_repository_adapter::PaymentRepositoryAdapter; // Temporarily disabled` | **Soft** | Commented-out wiring for a future "payment lookup from auth state" feature. |
| `apps/backend/src/web/auth/app_state.rs:1` | `use crate::domain::payment::repository_ports::{TransactionHistoryProvider};` | **Soft** | The auth `AppState` exposes `payment_history_provider` so handlers can look up tx history. |
| `apps/backend/src/web/auth/integration_tests.rs:1` | `use crate::infrastructure::adapters::repositories::payment_repository_adapter::tests::MockPaymentRepository;` | **Hard (test-only)** | Integration test uses the payment adapter's mock. Would need to move/re-export in a split. |
| `apps/backend/src/infrastructure/services/blockchain_monitor.rs:7-8` | `BscEventListener`, `PaymentEvent` from `infrastructure::blockchain` | **Hard** | Background service consumes payment events. Could stay inside the payments service or be a separate worker. |
| `apps/backend/src/application/market_analytics/queries/models/mod.rs:20,40` | Re-exports `GetStockRankingAssignmentsQuery`/`Response` and `StockRankingAssignment` which read the `stock_ranking_assignments` table (a payments table) | **Hard** | The market-analytics application module owns the query object that reads payment data. |
| `apps/backend/src/lib.rs` (via `create_router`) | The whole `web::payments` module | N/A | Route registration. |

**No outbound imports** from `domain/subscription_management/`,
`domain/notification/`, `domain/audit/`, `domain/wallet_management/`,
`domain/auth/`, or `domain/permission_management/` into the payments domain
(via grep over `domain/*/`.). The leakage is entirely on the **infrastructure
and web sides**.

**Summary:** 4 files with hard, non-port-based outbound coupling on the
payments infrastructure models/ports (the two big ones are
`web/admin/payment_link_handlers.rs` and
`web/admin/plans/handlers.rs`).

---

## 4. Database seams

### Schema-level split

There is **no Postgres `CREATE SCHEMA payments`** in any migration under
`migrations/payments/` (verified: `grep -E "CREATE SCHEMA|SET search_path"
apps/backend/migrations/payments/*/up.sql` returns zero matches). All 7
payments tables and the supporting SQL functions live in the default
`public` schema. The only `CREATE SCHEMA` in the codebase is
`migrations/core/00000000000001_consolidated_baseline_v6/up.sql` and
`migrations/core/00000000000001_consolidated_schema_v5/up.sql` — both create
`read_model` and are not payment-related.

The "split" today is a **Diesel-side split**:
- `diesel.toml` filters on 19 core tables (lines 15), output `schemas/primary.rs`.
- `diesel_payments.toml` filters on 7 payment tables (line 7), output
  `schemas/payments.rs`.
- `diesel_connection_manager.rs:339-373` keeps a *separate* connection pool
  driven by `PAYMENTS_DATABASE_URL`. **It falls back to the primary pool if
  that env var is unset** (`diesel_connection_manager.rs:341-348`), which is
  the current production mode (the wave8 worktree has no `PAYMENTS_DATABASE_URL`
  set in `infrastructure/docker/.env.prod`).

### Foreign-key seams

Verified with `grep -E "REFERENCES" apps/backend/migrations/payments/*/up.sql`:

- **Payments → core: none.** No payments table declares an FK to
  `wallet_users`, `plans`, `permissions`, or any other core table.
  `subscriptions.plan_id` is a `UUID NOT NULL` (line in
  `consolidated_payments_v3/up.sql:75`) with no `REFERENCES plans(id)` clause.
  The same is true for `payments.plan_id` (line 33). Both rely on
  application-level integrity checks.
- **Core → payments: none.** `grep -E "REFERENCES payments|REFERENCES
  subscriptions" apps/backend/migrations/core/*/up.sql` returns zero matches.
- **Payments → payments: 2.** `subscriptions.payment_id REFERENCES payments(id)`
  (line 79) and `payment_audit_log.payment_id REFERENCES payments(id) ON DELETE
  CASCADE` (line 153, both migrations).

So at the SQL level, the payments tables form a **closed subgraph** that does
not need to migrate with anything else. The application, however, does NOT
respect that — see below.

### Application-level cross-pool joins (the real coupling)

These are the leaks that a split must close, because today they require both
pools to be open in the same process:

| Handler | Lines | What it does |
|---------|------:|--------------|
| `apps/backend/src/web/payments/get_tx_status_handler.rs` | 19-21 (imports) + 121-137 (cross-pool lookup) | Open `payments_pool`, fetch payment; then open `get_diesel_pool()` and `select(plans::name)` to render `plan_name`. |
| `apps/backend/src/web/payments/user_payment_handlers.rs` | 25-26 (imports) + 144-166 (cross-pool lookup, per-row) | For each payment in the page, open `primary_pool` and `select(plans::name)` by `plans.id == payment.plan_id`. Runs in a loop — N+1 against the primary pool. |
| `apps/backend/src/web/payments/admin_handlers/payment_handlers.rs` (`admin_get_payment_details_handler`) | 16, 249-270 | Open both pools, query payment from `payments_conn`, then `plans::name` from `primary_conn`. |
| `apps/backend/src/web/payments/admin_handlers/subscription_handlers.rs` (`admin_list_subscriptions_handler`) | 16, 32-43, 99-101 (and following) | Two-conn pattern: filters/pagination on `subscriptions` (payments pool) + plan-name enrichment from `plans` (primary pool). |
| `apps/backend/src/web/payments/admin_handlers/analytics_handlers.rs` | 13, 39-44 | Same two-conn pattern for analytics rollups. |
| `apps/backend/src/web/payments/submit_tx_handler.rs` | 23 (import), 158-184 (price + plan_type validation), 285-294 (read `wallet_credits.balance`) | Read `plans.price` / `plans.is_active` / `plans.plan_type` / `plans.plan_metadata` from primary pool; read `wallet_credits.balance` from payments pool. |
| `apps/backend/src/web/payments/validation_handlers.rs` (`validate_payment_handler`) | 408-434 (`fetch_plan_info`) | Uses `app_state.plan_repo.get_subscription_plans()` (a port, GOOD) — this one is the only handler that already goes through `PlanRepositoryPort` (the `permission_management` domain port). |
| `apps/backend/src/web/payments/validation_handlers.rs` (`activate_subscription_handler`) | 23, 194-254 | Imports `UnifiedPermissionService` and `GrantPermissionRequest` from `crate::auth`; calls `permission_service.grant_permission(...)` on the **primary** pool. |

**8 of 14 handler functions** open both pools (or call into auth/permission
code that does). Only `validation_handlers.rs::fetch_plan_info` goes through
a proper port. Everything else reaches around the port layer with raw Diesel.

### What this means

- The **schema-level split is clean**: no SQL FKs cross the boundary, all 7
  tables are owned by payments migrations, and no core migration references
  them.
- The **application-level split is messy**: 8/14 handlers make hard-coded
  assumptions that the `plans` table and `wallet_users` table (via
  `UnifiedPermissionService`) are reachable in the same process.
- The **two pools are also not actually two databases** in the current
  deployment — they fall back to the same Postgres instance and schema. A
  future split that *actually* moves payments to a separate database would
  also have to solve the `plans` lookup problem (probably by snapshotting plan
  data into a `payment_plan_cache` table on the payments side, or by an HTTP
  call to the core service).

---

## 5. API surface

All payment routes are mounted under `/api/payments` via
`apps/backend/src/web/routes/unified_router.rs:98`
(`.nest("/payments", payment_routes)`) inside the `/api` super-nest. The
`create_payment_routes()` builder lives at `unified_router.rs:707-833` and
imports handlers from `crate::web::payments::{...}`.

### Public routes (no auth)

| Method | Path | Handler | File:line | Auth | DTOs (request → response) |
|--------|------|---------|-----------|------|---------------------------|
| GET | `/api/public/payment-links/{slug}` | `get_payment_link_by_slug_handler` | `web/admin/payment_link_handlers.rs:1+` (route registered in `unified_router.rs:385`) | none | `PaymentLinkBySlugResponse` (struct in same file) |

### Authenticated user routes (`bearer_middleware` only)

| Method | Path | Handler | File:line | Auth | DTOs |
|--------|------|---------|-----------|------|------|
| POST | `/api/payments/validate` | `validate_payment_handler` | `web/payments/validation_handlers.rs:139` | bearer | `ValidatePaymentRequest` → `ValidatePaymentResponse` |
| POST | `/api/payments/activate` | `activate_subscription_handler` | `web/payments/validation_handlers.rs:194` | bearer | `ActivateSubscriptionRequest` → `ActivateSubscriptionResponse` |
| POST | `/api/payments/submit` | `submit_transaction_handler` | `web/payments/submit_tx_handler.rs:76` | bearer | `SubmitTransactionRequest` → `SubmitTransactionResponse` |
| GET | `/api/payments/status/{tx_hash}` | `get_transaction_status_handler` | `web/payments/get_tx_status_handler.rs:76` | bearer | path-only → `TransactionStatusResponse` |
| GET | `/api/payments/details` | `get_payment_details_handler` | `web/payments/validation_handlers.rs:286` | bearer | `PaymentLookupParams` (query) → `PaymentDetailsResponse` |
| GET | `/api/payments/history` | `get_user_payment_history` | `web/payments/user_payment_handlers.rs:74` | bearer | `PaymentHistoryQuery` (query) → `PaymentHistoryResponse` |
| GET | `/api/payments/plans` | `get_user_plans_handler` | `web/payments/subscription_handlers.rs:121` | bearer | none → `UserPlansResponse` |
| GET | `/api/payments/plans/my-plan-access` | same handler (aliased) | `web/payments/subscription_handlers.rs:121` | bearer | none → `UserPlansResponse` |
| GET | `/api/payments/plans/expiry` | `get_plan_expiry_status_handler` | `web/payments/subscription_handlers.rs:305` | bearer | none → `PlanExpiryResponse` |
| POST | `/api/payments/plans/cancel/{id}` | `cancel_plan_handler` | `web/payments/subscription_handlers.rs:404` | bearer | `CancelPlanRequest` (optional) → `CancelPlanResponse` |
| GET | `/api/payments/plans/upgrade_preview` | `get_upgrade_preview_handler` | `web/payments/subscription_handlers.rs:683` | bearer | `UpgradePreviewQuery` (query) → `UpgradePreviewResponse` |
| POST | `/api/payments/plans/switch` | `execute_plan_switch_handler` | `web/payments/subscription_handlers.rs:758+` | bearer | `ExecutePlanSwitchRequest` → `ExecutePlanSwitchResponse` |
| GET | `/api/payments/credits/balance` | `get_credit_balance` | `web/payments/credit_handlers.rs:48` | bearer | none → `CreditBalanceResponse` |
| GET | `/api/payments/credits/history` | `get_credit_history` | `web/payments/credit_handlers.rs:90+` | bearer | `CreditTransactionFilters` (query) → `CreditTransactionResponse[]` |

### Admin routes (`permission_validation_middleware` + `bearer_middleware`)

| Method | Path | Handler | File:line | Auth | DTOs |
|--------|------|---------|-----------|------|------|
| GET | `/api/payments/admin/list` | `admin_list_payments_handler` | `web/payments/admin_handlers/payment_handlers.rs:27` | bearer + `admin:payments:*` | `AdminPaymentListParams` (query) → `AdminPaymentListResponse` |
| GET | `/api/payments/admin/{id}` | `admin_get_payment_details_handler` | `web/payments/admin_handlers/payment_handlers.rs:226` | bearer + admin | path → `AdminPaymentDetailsResponse` |
| PUT | `/api/payments/admin/{id}/status` | `admin_update_payment_status_handler` | `web/payments/admin_handlers/payment_handlers.rs:325` | bearer + admin | `UpdatePaymentStatusRequest` → `UpdatePaymentStatusResponse` |
| POST | `/api/payments/admin/{id}/refund` | `admin_process_refund_handler` | `web/payments/admin_handlers/payment_handlers.rs:430` | bearer + `admin:payments:refund` (re-checked at handler line 442) | `RefundPaymentRequest` → `RefundPaymentResponse` |
| GET | `/api/payments/admin/subscriptions` | `admin_list_subscriptions_handler` | `web/payments/admin_handlers/subscription_handlers.rs:21` | bearer + admin | `AdminPaymentListParams` (query) → `AdminSubscriptionListResponse` |
| GET | `/api/payments/admin/analytics` | `admin_get_payment_analytics_handler` | `web/payments/admin_handlers/analytics_handlers.rs:22` | bearer + admin | none → `PaymentAnalyticsResponse` |
| POST | `/api/payments/admin/tx/{tx_hash}/reprocess` | `admin_reprocess_payment_handler` | `web/payments/admin_reprocess_handler.rs` | bearer + admin | `ReprocessRequest` → `ReprocessResponse` |
| GET | `/api/payments/admin/tx/{tx_hash}/events` | `admin_payment_events_handler` | `web/payments/admin_reprocess_handler.rs` | bearer + admin | path → events list |
| GET | `/api/payments/admin/credits/{wallet}` | `admin_get_user_credits` | `web/payments/credit_handlers.rs` | bearer + admin | path → `CreditBalanceResponse` |
| POST | `/api/payments/admin/credits/grant` | `admin_grant_credits` | `web/payments/credit_handlers.rs:188+` | bearer + admin | `GrantCreditsRequest` → ack |
| POST | `/api/payments/admin/credits/revoke` | `admin_revoke_credits` | `web/payments/credit_handlers.rs:282+` | bearer + admin | `RevokeCreditsRequest` → ack |
| GET | `/api/payments/admin/credits/stats` | `admin_get_credit_stats` | `web/payments/credit_handlers.rs` | bearer + admin | none → `CreditStatsResponse` |

### Cross-cutting handlers that touch payments from outside `/api/payments`

| Method | Path | Handler | File:line | Notes |
|--------|------|---------|-----------|-------|
| GET | `/api/public/payment-links/{slug}` | `get_payment_link_by_slug_handler` | `web/admin/payment_link_handlers.rs` | Public lookup, mounted under `/api/public` in `unified_router.rs:385`. |
| (no API) | n/a | `admin_payment_events_handler` | `web/payments/admin_reprocess_handler.rs` | SSE stream of payment events for an admin UI; the route is registered at `/api/payments/admin/tx/{tx_hash}/events` (above). |
| (background) | n/a | `tx_monitor_service` | `infrastructure/blockchain/tx_monitor_service.rs` (spawned by `bootstrap.rs:143`) | Polls RPC for pending transactions; runs **as a background task inside the backend process**, not an HTTP service. |

**Total: 24 HTTP routes owned by payments** (1 public, 14 user, 9 admin),
plus 1 cross-domain public route that imports `payment_context_repository_adapter`
from `web/admin/`.

---

## 6. External integrations

### Blockchain / on-chain

Six Rust modules in `apps/backend/src/infrastructure/blockchain/` are entirely
payment-domain. Together they total **2,526 LOC** (see §1).

- **`payment_verifier.rs` (204 LOC).** Ethers-based `PaymentVerifier` that
  reads tx receipts, calls RPC, verifies ERC20 transfers. Used by the
  `validate_payment_handler`.
- **`bsc_listener.rs` (196 LOC).** Polling BSC RPC for `Transfer` events on
  configured receiver addresses. Constructs `PaymentEvent` values. Used by
  `BlockchainMonitor` (background service).
- **`tx_monitor_service.rs` (1,122 LOC).** The largest single payment
  file. Spawns a `tokio` task that periodically polls pending payments,
  reaches into `payments`/`subscriptions`/`wallet_credits` tables in
  `diesel`, and writes confirmations. Started from
  `apps/backend/src/bootstrap.rs:143`
  (`crate::infrastructure::blockchain::spawn_transaction_monitor();`).
- **`contract_subscriber.rs` (422 LOC).** WebSocket-based contract event
  subscriber. Initialized per-chain in
  `apps/backend/src/infrastructure/container/simple_container.rs:430-540`
  for BSC, Ethereum, Polygon, Arbitrum, Optimism, Base. The callback in
  `simple_container.rs:507-518` is a no-op that just logs
  (`tracing::info!("Processing payment: ...");`) — actual DB writes happen
  via the polling service. Status: **wired but not yet driving writes**;
  the contract-subscriber path is a future-looking integration.
- **`event_parser.rs` (219 LOC).** Decodes raw `Transfer` logs into
  `PaymentEvent` values (`parse_payment_event`). Imported by both
  `bsc_listener.rs` and the RPC/scanner history providers.
- **`validation_client.rs` (363 LOC).** Wraps the ethers provider for
  `web3::eth_call`-style lookups. **This one is shared with the
  permission-management web3 adapter** (`nft.rs`, `token.rs`, `dao.rs`) —
  splitting payments would require either (a) keeping `validation_client`
  in a shared crate or (b) duplicating it.

The blockchain monitor's separate binary lives at
`apps/backend/src/bin/blockchain_monitor.rs` and is run as its own
process in production. It uses `epsx::infrastructure::BlockchainMonitor`.

### RPC providers

Configured via env (`config::env::Config`) — `BSC_RPC_URL`, `ETHEREUM_RPC_URL`,
`POLYGON_RPC_URL`, `ARBITRUM_RPC_URL`, `OPTIMISM_RPC_URL`, `BASE_RPC_URL`,
plus per-chain WS URLs and WS backups (see
`infrastructure/container/simple_container.rs:443-484`).

### Redis

- **Notification fan-out.** `submit_tx_handler.rs:567` and
  `credit_handlers.rs:255` `tokio::spawn` a `NotificationService::send(...)`
  call that publishes to `RedisNotificationBroadcaster`. The broadcaster
  fan-out lives in `web::notifications` (not payments), so the
  payments-side dep is one import, not a Redis-client instantiation.
- **No payment-owned Redis keys.** A grep for `redis`, `XADD`, `xadd`,
  `publish`, `stream` inside `web/payments/`,
  `infrastructure/blockchain/{tx_monitor,bsc_listener,contract_subscriber,event_parser,payment_verifier}.rs`
  returns only the WS-stream split in `contract_subscriber.rs:178-189`
  (that's a `tokio::time::timeout` for the `ws_stream` future, not a Redis
  stream). The CQRS outbox (`infrastructure/cqrs/outbox.rs`) is **defined
  but unused** — no live caller in the source tree (only docs/USAGE.md).
- **Plan-expiration service** (`infrastructure/services/plan_expiration_service.rs`)
  publishes via `RedisNotificationBroadcaster` for "plan expiring" events;
  it's a *core* service that uses the *notifications* broadcaster, so it
  is not a payment-owned Redis key.

### Queues / cron / webhooks

- **No queue / cron** in the payment code. The closest things are:
  - `PlanExpirationService` background loop (hourly) — core, not payment.
  - `tx_monitor_service` poll loop (5s default, `tx_monitor_service.rs:86`) —
    payment.
  - `contract_subscriber` WS loop (5s reconnect backoff) — payment.
- **No webhooks.** No `Webhook`/`webhook` references in `web/payments/` or
  `infrastructure/blockchain/`. The integration is pull-based (poll RPC).
- **`bootstrap.rs:143`** is the only place that *starts* the
  `tx_monitor_service`. It is NOT conditional on `PAYMENTS_DATABASE_URL` —
  it always starts in the backend binary. This is a deployment-time
  concern: if payments is split to a separate process, the monitor needs
  to move with it (or both processes need to coordinate via a leader
  election, since today only one replica should be polling).

---

## 7. Split-readiness score

### Score: **2.5 / 5** (could lift in 2-3 weeks, not tomorrow)

**Why not higher than 2.5:**

- **Schema-level data is clean (§4).** All 7 payments tables form a closed
  subgraph, no SQL FKs cross the boundary, all migrations live under
  `migrations/payments/`. Two Diesel pools already exist in
  `diesel_connection_manager.rs:339-373`. This is a real
  almost-there foundation.
- **DDD layer is genuinely isolated.** 3 hard inbound imports
  (`WalletAddress` ×2, `UnifiedPermissionService` ×1) and zero outbound
  domain imports is **good** for a system of this size.
- **The web layer is the problem.** 8/14 handlers bypass the
  `PaymentRepositoryPort` and reach into `schemas::payments::*` + raw
  `diesel::sql_query` blocks. 8/14 handlers open **two** pools in the
  same request. 4 non-payment files (`web/admin/payment_link_handlers.rs`,
  `web/admin/plans/handlers.rs`,
  `infrastructure/adapters/repositories/subscription_repository_adapter.rs`,
  `application/market_analytics/queries/models/get_stock_ranking_assignments.rs`)
  reach directly into the payment infrastructure layer to read/write
  payments data.
- **One process-coupled integration.** `activate_subscription_handler` calls
  `UnifiedPermissionService::grant_permission` directly. The service
  constructor takes `&'static TlsPool` (the *primary* pool — line 116 of
  `auth/unified_permission_service.rs`). A standalone payments service
  would have to either (a) call a new HTTP endpoint on the core service
  or (b) replicate permission-grant logic on its side. This is a real
  re-architecture, not a rename.
- **The two pools are one database in production today.** The "split" runs
  on a single Postgres instance with a fallback in
  `diesel_connection_manager.rs:341-348`. Lifting payments to a
  separate microservice will require (a) actually moving the schema, and
  (b) dealing with the 8 cross-pool handler call sites.
- **Background workers are coupled.** `tx_monitor_service` is spawned by
  `bootstrap.rs:143` and writes to both `payments` (payments pool) and
  `wallet_credits` (payments pool, good) and *might* need to write to
  `permissions` / `wallet_users` (primary pool, bad). Moving it to a
  payments process removes the second write capability.
- **Permission string is hard-coded.** `validation_handlers.rs:222,240`
  builds `epsx:rankings:offset:N` and `epsx:rankings:limit:N` permission
  strings inline. If the format ever changes, the payment code must
  change. Soft coupling, but a smell.

**Why not lower than 2.5:**

- The DDD domain layer is already 80% split-ready.
- The schema split is already 100% done.
- The DI container (`simple_container.rs:430-540`) already isolates
  payment init into a single `initialize_contract_subscribers` call, so
  a feature-flag ("payments_on") could disable it cleanly.
- Only 3 inbound cross-domain imports in the entire DDD+application
  stack — that is competitive with the other bounded contexts in this
  codebase (see sibling `audit-notifications.md` for comparison).
- The number of "must change at the API level" call sites is small
  (12 handlers, 4 external files).

**Concrete numbers supporting the score:**

| Coupling metric | Count | Source |
|-----------------|------:|--------|
| Inbound cross-domain `use` statements (domain + application) | 3 | grep on `domain/payment` and `application/payment` |
| Inbound cross-domain `use` statements (web) | 1 | `validation_handlers.rs:23` only |
| Outbound domain imports FROM payment | 0 | grep on `domain/*/` excluding payment/ |
| Outbound infrastructure imports FROM payment | 4 | `web/admin/payment_link_handlers.rs`, `web/admin/plans/handlers.rs`, `infrastructure/adapters/repositories/subscription_repository_adapter.rs`, `application/market_analytics/queries/models/get_stock_ranking_assignments.rs` |
| Cross-pool join sites in handlers | 8 | listed in §4 table |
| SQL FKs crossing the boundary | 0 | grep on migrations |
| Background tasks that must move with payments | 2 | `tx_monitor_service` (start: `bootstrap.rs:143`), `blockchain_monitor.rs` (start: `bin/blockchain_monitor.rs:144`) |
| HTTP routes that would break with a hard split | 25 | 1 public + 14 user + 9 admin + 1 cross-domain public |
| `pub use` / `pub fn` exposure from `domain::payment::*` | 30+ | `mod.rs:36-62` re-exports all value objects, aggregates, ports |

A 4+ score would require (a) zero two-pool handler sites, (b) the
`UnifiedPermissionService` call to go through a port, and (c) the 4
outbound files to be deleted (their code absorbed into the payment
service or re-routed through HTTP/port).

---

## 8. Top 3 specific refactors that would move the score to 4+

Each refactor below is sized with file paths, LOC, and the exact
replacement code shape. No hand-waving.

### Refactor 1 — Move all payment handler DB I/O behind `PaymentRepositoryPort`

**Score impact:** 2.5 → 3.5

**Files to touch (11):**
- `apps/backend/src/infrastructure/adapters/repositories/payment_repository_adapter.rs` (+400 LOC) — add: `async fn find_payment_with_plan_name(&self, payment_id: Uuid) -> Result<Option<(Payment, String)>, AppError>`; `async fn list_with_plan_names(&self, criteria: ListCriteria) -> Result<Vec<(Payment, String)>, AppError>`; `async fn list_subscriptions_with_plan_names(...)`; `async fn analytics_rollup(...) -> AnalyticsSummary`; `async fn upsert_subscription(...)`. The current adapter already implements `PaymentRepositoryPort` (`payment_repository_adapter.rs:1+`) but exposes only the domain-level CRUD — these new methods encapsulate the cross-pool lookup.
- `apps/backend/src/web/payments/get_tx_status_handler.rs` (-30 LOC, +5 LOC) — replace the inline `get_diesel_pool` + `plans::table` lookup at lines 121-137 with `let payment_plan = self.payment_repo.find_payment_with_plan_name(payment_id).await?;`. Drop the `use schemas::primary::plans` and `use infrastructure::database::get_diesel_pool` imports.
- `apps/backend/src/web/payments/user_payment_handlers.rs` (-25 LOC, +3 LOC) — replace the N+1 lookup at lines 144-166 with `let items = self.payment_repo.list_with_plan_names(Pagination{...}).await?;`. Drop the same two imports.
- `apps/backend/src/web/payments/admin_handlers/payment_handlers.rs` (-22 LOC, +3 LOC) — replace lines 249-270 with the same port call. Drop the primary-conn block.
- `apps/backend/src/web/payments/admin_handlers/subscription_handlers.rs` (-30 LOC, +3 LOC) — replace the two-conn block (lines 32-43 + plan-name enrichment) with one port call. **This is also the place where the inline `plans::table` import at line 16 should be deleted.**
- `apps/backend/src/web/payments/admin_handlers/analytics_handlers.rs` (-25 LOC, +3 LOC) — same pattern. Delete the `use schemas::primary::plans` import at line 13.
- `apps/backend/src/web/payments/submit_tx_handler.rs` (-25 LOC, +3 LOC) — replace the `get_diesel_pool` + raw `diesel::sql_query` against `plans` (lines 158-189) with a `plan_repo.get_plan_for_purchase(plan_id)` call. (Already has a port — `app_state.plan_repo` — that goes through `permission_management::PlanRepositoryPort`.) Drop the `get_diesel_pool` import.
- `apps/backend/src/web/payments/admin_handlers/types.rs` and `dtos.rs` — unchanged, no touch needed.

**LOC delta:** +400 (new port methods, including tests) -157 (handlers simplified) = **+243 LOC net**, but the new methods move the "two pools" coupling from 8 handler sites to 1 port implementation.

**Why it unblocks the split:** After this, the 8 handler functions no longer import `schemas::primary::plans` or call `get_diesel_pool()`. They depend only on `Arc<dyn PaymentRepositoryPort>` (already injected via `AppState`). A standalone payments service can implement the port with a single pool — the secondary lookups become a single `LEFT JOIN plans` in the new port method (still cross-DB, but localised to ONE method that's easy to replace with a remote call later).

### Refactor 2 — Extract a `PermissionAuthority` port around `UnifiedPermissionService.grant_permission`

**Score impact:** 3.5 → 4.0

**Files to touch (5):**
- `apps/backend/src/domain/permission_management/repository_ports/` (NEW FILE) — add `permission_authority_port.rs` (~40 LOC) with:
  ```rust
  #[async_trait]
  pub trait PermissionAuthorityPort: Send + Sync {
      async fn grant_permission(&self, req: GrantPermissionRequest) -> AppResult<()>;
      async fn revoke_permission(&self, req: RevokePermissionRequest) -> AppResult<()>;
  }
  ```
  Plus a small `GrantPermissionRequest` / `RevokePermissionRequest` newtype (these already exist in `auth/unified_permission_service.rs:73-107` and can be moved verbatim — ~35 LOC).
- `apps/backend/src/infrastructure/adapters/services/permission_authority_adapter.rs` (NEW FILE, ~80 LOC) — implement the port by delegating to `UnifiedPermissionService`. This is a thin pass-through: `impl PermissionAuthorityPort for PermissionAuthorityAdapter { async fn grant_permission(&self, req) -> AppResult<()> { self.inner.grant_permission(req).await.map_err(AppError::from) } }`.
- `apps/backend/src/web/auth/app_state.rs` (+12 LOC) — add `pub permission_authority: Arc<dyn PermissionAuthorityPort>` next to the existing `pub plan_repo`. Wire from `domain_container` in `with_dependencies`.
- `apps/backend/src/web/payments/validation_handlers.rs` (-3 LOC, +2 LOC) — change the `Extension(permission_service): Extension<Arc<UnifiedPermissionService>>` extractor at line 197 to `Extension(authority): Extension<Arc<dyn PermissionAuthorityPort>>`; replace the two `permission_service.grant_permission(request).await` calls at lines 233, 251 with `authority.grant_permission(request).await`. Delete the `use crate::auth::{UnifiedPermissionService, GrantPermissionRequest};` import at line 23.
- `apps/backend/src/auth/unified_permission_service.rs` — unchanged, becomes the *adapter's* dependency, no longer directly imported by web/payments.

**LOC delta:** +40 (port) + 80 (adapter) + 12 (app_state wiring) -3 (handler change) = **+129 LOC net**. Most of the new code is the port + adapter boilerplate; the actual logic moves 4 lines.

**Why it unblocks the split:** `validation_handlers.rs:23` is the **only**
non-DI, hard cross-context import in `web/payments/`. After this refactor,
the handler's only outbound dependency is `Arc<dyn PermissionAuthorityPort>`
— which a standalone payments service can satisfy by HTTP-call into a
core-side `POST /api/permissions/grant` endpoint. The `UnifiedPermissionService`
itself stays in the auth module of the core service, and the payment
service holds a thin client.

### Refactor 3 — Fold the 4 outbound-leakage files into `web/payments/`

**Score impact:** 4.0 → 4.5

**Files to touch (4 moves + 2 deletions):**
- **Move** `apps/backend/src/web/admin/payment_link_handlers.rs` → `apps/backend/src/web/payments/payment_link_handlers.rs` (~460 LOC unchanged). Then change the route registration in `unified_router.rs:385` from `crate::web::admin::payment_link_handlers::get_payment_link_by_slug_handler` to `crate::web::payments::payment_link_handlers::get_payment_link_by_slug_handler`. The handler now has *zero* `crate::web::admin::` imports.
- **Move** `apps/backend/src/web/admin/plans/handlers.rs` — the only payment-specific subset is the 4 functions that touch `SubscriptionDb` (lines 6, 30, 250-340 from a quick grep). Move those ~250 LOC into `web/payments/admin_handlers/subscription_admin_handlers.rs` and re-register their routes under the existing `/api/payments/admin/subscriptions` group in `unified_router.rs:780-782`. The remaining `plans/handlers.rs` keeps only the actual plans admin (CRUD plans, no subscriptions).
- **Move** `apps/backend/src/infrastructure/adapters/repositories/subscription_repository_adapter.rs` → `apps/backend/src/infrastructure/adapters/repositories/payments/subscription_repository_adapter.rs` (or, better, into the payment repo adapter file). The whole adapter writes to the `subscriptions` table using `SubscriptionDb` from `models::payment` — it belongs with the payment adapter, not the central infrastructure layer. Rename the type to `PaymentSubscriptionRepositoryAdapter` to make the ownership explicit. ~250 LOC, but only ~20 lines change (the import path).
- **Move** `apps/backend/src/application/market_analytics/queries/models/get_stock_ranking_assignments.rs` — this is a *query* object that reads `stock_ranking_assignments`. Move to `apps/backend/src/application/payment/queries/get_stock_ranking_assignments.rs` (~150 LOC). The market_analytics module re-exports it as a type alias so the public API is unchanged.
- **Delete** the `// use crate::infrastructure::adapters::repositories::payment_repository_adapter::PaymentRepositoryAdapter; // Temporarily disabled` line in `web/auth/app_state.rs:1` (dead code).
- **Delete** the test-only `use crate::infrastructure::adapters::repositories::payment_repository_adapter::tests::MockPaymentRepository;` in `web/auth/integration_tests.rs:1` and replace it with a local mock in the test file (the test is gated on a payment flow that has its own integration test now).

**LOC delta:** ~1,110 LOC moved (no new code written), 2 lines deleted. The
4 files now sit inside the payments area; the 2 lines of dead code go
away. After this refactor, the ONLY way to read or write payments data is
through `crate::domain::payment::*` or `crate::web::payments::*`. Any
re-import of `crate::infrastructure::adapters::repositories::payment_*` or
`crate::infrastructure::models::payment::SubscriptionDb` from outside the
payment area becomes a code-review red flag.

**Why it unblocks the split:** A standalone payments service can take
these 4 files with it. No one in the core service is touching the
`stock_ranking_assignments` table, the `subscriptions` table, or the
`payment_contexts` table any more. The `web/admin/payment_link_handlers`
public route can be re-registered as
`/api/payments/public/payment-links/{slug}` on the payments side without
a backwards-compatibility shim.

### Combined score projection

| After | Score | Why |
|-------|------:|-----|
| Today | 2.5 | Two-pool coupling in 8 handlers, 4 outbound leaks, 1 direct auth-module call |
| + Refactor 1 | 3.5 | All handler DB I/O behind a port; 2-pool coupling lives in one method |
| + Refactor 2 | 4.0 | `UnifiedPermissionService` is behind a port; payments has zero auth-module calls |
| + Refactor 3 | 4.5 | Zero outbound leakage; all payment data only touched via `web/payments/*` or `domain::payment::*` |

A 5/5 would additionally require the `credit_transactions` ↔
`permission_management::Permission` correlation (which today is just a
`granted_by VARCHAR(42)` string and a reason text) to become a structured
link, plus moving the `tx_monitor_service` polling loop to a leader-election
setup so multiple payments-service replicas don't double-process. Those
are real but smaller-scope work; the 3 refactors above cover the
blocking coupling for a 4+ score.

---

## Appendix: verification commands

Every claim in this document can be re-verified with the following
one-liners (run from `apps/backend/`):

```bash
# §1 file count + LOC
find src/domain/payment -name '*.rs' | xargs wc -l
find src/application/payment -name '*.rs' | xargs wc -l
find src/web/payments -name '*.rs' | xargs wc -l
wc -l src/infrastructure/adapters/repositories/payment_*_adapter.rs \
      src/infrastructure/adapters/repositories/credit_repository_adapter.rs \
      src/infrastructure/models/payment.rs \
      src/infrastructure/models/credit.rs

# §1 blockchain LOC
wc -l src/infrastructure/blockchain/{payment_verifier,bsc_listener,tx_monitor_service,event_parser,validation_client,contract_subscriber}.rs

# §2 inbound cross-domain imports
grep -rE '^use crate::domain::(auth|notification|wallet_management|subscription_management|market_analytics|audit|developer_portal|permission_management)' \
    src/domain/payment src/application/payment

# §3 outbound dependents
grep -rE 'use crate::(domain::payment|application::payment|infrastructure::(adapters::repositories::(payment|credit)|models::(payment|credit)|blockchain::(payment_verifier|bsc_listener|tx_monitor_service|contract_subscriber)|services::(plan_expiration_service|blockchain_monitor))|web::payments|schemas::payments)' \
    src/ | grep -vE 'src/(domain/payment|application/payment|web/payments|infrastructure/(adapters/repositories/(payment|credit)|models/(payment|credit)|blockchain/(payment|bsc|tx_monitor|contract_subscriber))|schemas/payments)'

# §4 FK crossings
grep -E 'REFERENCES' migrations/payments/*/up.sql
grep -E 'REFERENCES (payments|subscriptions|payment_contexts|wallet_credits|credit_transactions|stock_ranking_assignments)' \
    migrations/core/*/up.sql

# §4 cross-pool handlers
grep -nE 'get_diesel_pool|schemas::primary::plans' src/web/payments/ -r

# §5 routes
sed -n '707,833p' src/web/routes/unified_router.rs

# §6 background workers
grep -nE 'spawn_transaction_monitor' src/bootstrap.rs
grep -nE 'BlockchainMonitor' src/bin/blockchain_monitor.rs
```
