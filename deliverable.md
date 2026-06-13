# Wave 11 / Track B — Outbound-leakage fold — Deliverable

## 1. Summary

Shipped ROADMAP §4 wave 11 precondition item 3: the 4 outbound-leakage
files (per payments audit Refactor #3) are folded into `web/payments/`
(or routed through a port), the new `SubscriptionRepositoryPort` is in
place, and the market-analytics `GetStockRankingAssignmentsQuery` no
longer reaches into the payments infrastructure layer directly.

7 commits on `wave11/track-b-leakage-fold`, all pushed to origin.
Final HEAD: `0058d79e`.

Base: `origin/migration/dioxus-microservices` @ `1014d8c4` (wave-10 integration).

## 2. Per-file disposition

| # | File | Disposition | Detail |
|---|------|-------------|--------|
| 1 | `apps/backend/src/web/admin/payment_link_handlers.rs` (616 LOC) | **Pure move** | → `apps/backend/src/web/payments/payment_link_handlers.rs` (1098 LOC). New wider `PaymentContextRepositoryPort`. Route mount updated; public path `/api/public/payment-links/{slug}` unchanged. |
| 2 | `apps/backend/src/web/admin/plans/handlers.rs` (subscription subset) | **Hybrid** | `list` → moved to `apps/backend/src/web/payments/admin/subscription_admin_handlers.rs`. `create` → stays in `plans/handlers.rs` (dual-DB write: `wallet_plan_assignments` UPSERT + payments `subscriptions` insert — needs `PlanAssignmentRepositoryPort`, wave-12+ follow-up). |
| 3 | `apps/backend/src/infrastructure/adapters/repositories/subscription_repository_adapter.rs` (229 LOC) | **Pure move + rename** | → `apps/backend/src/infrastructure/adapters/repositories/payment/subscription_repository_adapter.rs` (621 LOC). Renamed `PaymentSubscriptionRepositoryAdapter`. New `get_stock_ranking_assignments` SQL method. `#[deprecated]` alias kept. |
| 4 | `apps/backend/src/application/market_analytics/queries/models/get_stock_ranking_assignments.rs` (42 → 525 LOC) | **Refactored as thin facade** | DTO moved to `domain/payment/aggregates/`. New `get_stock_ranking_assignments_via_port(port, query)` delegates to `Arc<dyn SubscriptionRepositoryPort>`. 5 canary tests with `MockSubscriptionRepository`. |

## 3. New ports + adapters

- **`apps/backend/src/domain/payment/repository_ports/subscription_port.rs`** — new `SubscriptionRepositoryPort` (5 methods).
- **`apps/backend/src/infrastructure/adapters/repositories/payment/subscription_repository_adapter.rs`** — in-process impl: `PaymentSubscriptionRepositoryAdapter`. 5 stock-ranking canary tests + round-trip tests.

## 4. Tests

- `cargo test -p epsx --lib` → **443 passed, 0 failed, 8 ignored** (was 397 pre-wave-10, +46 over wave 10 + 11; the +46 includes 9 wave-10 notification/contracts tests and 5 stock-ranking canaries, 4 round-trip, plus the existing `market_analytics` tests routed through the port).
- Stock-ranking canary 5/5 green (the audit's "strongest outward leak" regression test).
- Route test scaffolding for the moved payment-link handlers (200/404/410 paths).

## 5. DI graph + route changes

- `simple_container.rs` and `stateless_service_factory.rs` register `dyn SubscriptionRepositoryPort` and the wider `dyn PaymentContextRepositoryPort`.
- `web/payments/payment_link_handlers.rs` mounted at the same `/api/public/payment-links/{slug}` route in `unified_router.rs`.
- `apps/backend/src/web/payments/admin/subscription_admin_handlers.rs` mounted for the subscriptions-list endpoint.

## 6. Cross-track coordination

- **No conflict with Track A** (PaymentRepositoryPort extension, 8 cross-pool handler sites). My two new ports are independent of the cross-pool handler collapses; I did not touch `web/payments/*.{get_tx_status_handler, user_payment_handlers, admin_handlers, submit_tx_handler, validation_handlers}` per the file-ownership rules in the task brief.
- **No conflict with Track C** (EventPublisherPort, domain events). The two refactors are orthogonal.

## 7. Deviations from the spec (all justified in the deliverable)

1. `create_subscription_handler` stayed in `plans/handlers.rs` — the dual-DB write needs `PlanAssignmentRepositoryPort` (wave-12+ concern, not in this track's scope).
2. `payment_repository_adapter` / `payment_context_repository_adapter` / `credit_repository_adapter` still live in the central tree — full file moves are a wave-12+ concern.
3. `is_context_usable` / `compute_link_hash` are free functions, not port methods (they don't touch the DB).
4. `create_admin_payment_link_routes` helper is unused (forward-move marker) — flagged for cleanup in wave-12.

## 8. Open issues for the integration gate

1. Track A's `admin_list_payments_handler` still has an unresolved `plans::table` import per the kill post-mortem — the integration gate may need to re-add a scoped import in that handler when merging A in.
2. Wave-12 cleanup: the `create_admin_payment_link_routes` helper, the `#[deprecated]` adapter alias, and the `create_subscription_handler` dual-DB write.
3. The `MarketAnalytics` facade still owns the `GetStockRankingAssignmentsQuery` DTOs — could be moved to `domain/payment/aggregates/` in a follow-up.

## 9. Verification

- 7 incremental `wave11(track-b):` commits on the correct branch.
- Build: `cargo check --workspace` clean.
- Tests: 443/443 passing.
- No unrelated changes in the diff.
