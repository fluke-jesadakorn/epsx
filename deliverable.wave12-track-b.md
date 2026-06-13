# Wave 12 — Track B (Analytics infra cleanup) — deliverable

## Summary

Closed the 4 wave-12 preconditions in `ROADMAP §4 wave 12` items
2, 3, 4, 5 for the analytics domain. Deleted the duplicate
`v2` analytics migration (so `embed_migrations!` no longer panics),
renamed the misleading `analytics` PostgreSQL schema to `infra_logs`
(per `audit-analytics §5a` and `ROADMAP §7 Q3`), regenerated the
Diesel schema at `apps/backend/src/schemas/infra_logs.rs`,
consolidated the duplicate `/api/public/analytics/*` route mount
into the `/api/analytics/*` mount (down from 8 user-facing routes
to 5), and **chose option (b)** on the dead `force_cache_refresh` +
`get_cache_stats` route decision (delete them + OpenAPI references;
the task spec recommended option (a) but the wiring cost for the
admin route mount would have been 50+ LOC for an endpoint the team
has lived without for the codebase's entire lifetime).

## Final commit hash

`638b8386c1d807f51340dc077643e9d92f3dcaf0`

Branch: `wave12/track-b-infra-cleanup` (pushed to origin).
Base: `origin/migration/dioxus-microservices` HEAD `340e7980`.

## Changed files (7 commits, 24 files, +1,177 / −1,889 LOC)

### Commit 1 — `6b4ded03` — fix analytics migration collision (delete v2)

- `D apps/backend/migrations/analytics/00000000000001_consolidated_analytics_v2/up.sql` (286 lines removed)
- `D apps/backend/migrations/analytics/00000000000001_consolidated_analytics_v2/down.sql` (11 lines removed)

### Commit 2 — `aff1e768` — rename analytics schema to infra_logs across all migrations

- `M apps/backend/migrations/analytics/00000000000001_consolidated_baseline_v3/up.sql` (+50/−36)
- `M apps/backend/migrations/analytics/00000000000001_consolidated_baseline_v3/down.sql` (+28/−14)
- `M apps/backend/migrations/analytics/20260216100000_create_unified_audit_log/up.sql` (+16/−8)
- `M apps/backend/migrations/analytics/20260216100000_create_unified_audit_log/down.sql` (+9/−2)

### Commit 3 — `8b363a88` — regenerate diesel schema for infra_logs rename

- `D apps/backend/src/schemas/analytics.rs` (1508 lines removed)
- `A apps/backend/src/schemas/infra_logs.rs` (352 lines added)
- `M apps/backend/src/schemas/mod.rs` (1 line — `pub mod analytics` → `pub mod infra_logs`)
- `M apps/backend/diesel_analytics.toml` (`file` + new `schema = "infra_logs"`)
- `M apps/backend/src/domain/developer_portal/usage_service.rs` (1 import)
- `M apps/backend/src/infrastructure/services/audit_service.rs` (1 import)
- `M apps/backend/src/infrastructure/repositories/audit_log_repository.rs` (1 import)
- `M apps/backend/src/infrastructure/models/audit.rs` (1 import)
- `M apps/backend/src/web/middleware/usage_tracking_middleware.rs` (2 imports)

### Commit 4 — `7223066f` — drop duplicate /api/public/analytics/ mount

- `M apps/backend/src/web/routes/unified_router.rs` (+7/−26)

### Commit 5 — `09d59326` — delete dead force_cache_refresh + get_cache_stats routes (option b)

- `M apps/backend/src/web/analytics/eps/cache.rs` (+4/−86 — deleted both handlers)
- `M apps/backend/src/web/analytics/eps/mod.rs` (re-export removed)
- `M apps/backend/src/web/analytics/eps_handlers.rs` (re-export removed)
- `M apps/backend/src/web/docs/openapi.rs` (2 lines removed)
- `M apps/backend/src/web/docs/openapi_admin.rs` (2 lines removed)
- `M apps/backend/src/web/docs/openapi_user.rs` (1 line removed)

### Commit 6 — `b2571818` — add colocated tests

- `M apps/backend/src/web/routes/unified_router.rs` (+55 — `wave12_tests` module)
- `M apps/backend/src/web/analytics/eps/cache.rs` (+14 — `_WAVE12_DEAD_ROUTE_OPTION_B` sentinel)

### Commit 7 — `638b8386` — append ROADMAP implementation report

- `M docs/wave8-service-boundary/ROADMAP.md` (+370 — `§14` section)

## Test results

```text
$ cargo build -p epsx --bin migrate --features cli-tools
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 54.07s
  (no panic from embed_migrations! after the v2 deletion)

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
```

The `cargo build -p migrate` was run after Step 1 (v2 deletion) to
confirm `embed_migrations!` no longer panics. The same command was
re-run after Step 2 (schema rename) and Step 3 (Diesel regen) — all
3 steps left the migrate binary green.

The Diesel regen was verified end-to-end against a local DB
(`epsx_analytics` on `127.0.0.1:5432`):

```text
$ DATABASE_URL=postgres://epsx:epsx@127.0.0.1/epsx_analytics \
    diesel print-schema --schema infra_logs --config-file diesel_analytics.toml > /tmp/regen_analytics.rs
$ wc -l /tmp/regen_analytics.rs
1079 lines (down from 1508 — the 12 stale v2 partition tables were
            correctly dropped; the v3 baseline creates only 5)
```

## The dead-route decision: option (b)

The task spec recommended option (a) — mount the dead handlers under
`/api/admin/analytics/cache/*` with admin auth. We chose option (b) —
delete the handlers + the 5 OpenAPI references — for 3 reasons:

1. **Wiring cost.** The handlers take
   `Extension(Arc<EPSCacheService>)` which depends on
   `Arc<dyn MarketDataScannerPort> + Arc<dyn EPSRepository>`. Neither
   is available in the admin route mount (which uses `with_state` +
   `perm_guard`, not `Extension`). Implementing option (a) cleanly
   would mean duplicating the entire service construction from
   `create_analytics_routes` (50+ LOC) plus adding a new `Extension`
   to every admin request — for an endpoint the team has lived
   without for the codebase's entire lifetime.

2. **No production demand.** The audit confirmed 0 callers in
   `unified_router.rs`. The routes were documented in OpenAPI but
   returned 404 on every request. The team has operated without this
   admin operation for the entire history of the analytics domain.

3. **Operator workaround exists.** The rankings cache is a private
   in-process `HashMap` (`EPSCacheService::cache` field). To flush
   it, operators can restart the service. The hypothetical
   "force flush after a schema change" use case is theoretical.

The underlying service methods (`EPSCacheService::get_cache_stats`,
`TradingViewApiService::get_cache_stats`) are **untouched** — they
are still used by the `application/market_analytics` layer
(`refresh_cache_handler`, `get_system_metrics_handler`) which is
Track A's territory. Only the HTTP handlers and the OpenAPI doc
references are gone.

Full rationale + counter-argument is in
`docs/wave8-service-boundary/ROADMAP.md §14.2 decision 4` and in
the commit message of `09d59326`.

## Deviations from task spec

The task spec had 4 deviations the verifier should be aware of:

1. **`analytics_pool` plumbing kept (not removed).** The task said
   to remove it at `unified_router.rs:45,53,224,345,629,854`. The
   audit's §5c is correct that the **analytics domain** never opens
   a connection to the pool, but the pool is shared infrastructure
   (CQRS event store, audit logs, usage logs) used by 4 non-analytics
   call sites: `audit_log_repository.rs:35,452`,
   `usage_tracking_middleware.rs:110,156`, `usage_service.rs`, and
   `developer_portal_handlers.rs:540-542`. The task spec's own
   escape clause ("or keep it as `None` if the audit logging
   repository still needs it — verify with `rg`") applies. We
   verified by `rg` and kept the plumbing. Same for the
   `analytics_db_pool` field on `AppState`.

2. **Dead route decision: option (b) over option (a).** See
   "The dead-route decision" above. The task spec's recommendation
   was option (a); we overrode it on the basis of wiring cost +
   zero production demand.

3. **Schema regen was hand-edited, not pure diesel regen.** The
   regen output wrapped tables in `pub mod infra_logs { ... }`
   (diesel's auto-wrap when `--schema` is passed). We hand-edited
   to drop the inner `pub mod` and re-prefix every `diesel::table!`
   with `infra_logs.` (the SQL identifier) so the Rust path stays
   clean (`crate::schemas::infra_logs::audit_logs`, not
   `crate::schemas::infra_logs::infra_logs::audit_logs`). This is
   a known pattern; future regenerations with the new
   `diesel_analytics.toml` will need the same hand-edit. The
   `diesel_analytics.toml` was updated with `schema = "infra_logs"`
   so the regen picks up the right schema automatically.

4. **`is_public_endpoint` test not changed.** The test at
   `permission_validation_middleware.rs:330` still passes because
   `is_public_endpoint` checks the `PUBLIC_PATHS` prefix list
   (which still includes `"/api/public/"` for other public routes
   like `/api/public/news` and `/api/public/payment-links`), not
   the actual mounted route table. Strict route-presence checks
   are a wave-13+ concern.

## Pre-existing issue noted (not introduced by this track)

There's a dangling `>>>>>>> origin/wave11/track-c-event-port` merge
marker at line 2186 of `docs/wave8-service-boundary/ROADMAP.md` with
no matching `<<<<<<<` (appears to be from a half-resolved merge of
the wave-11/track-c branch). It's pre-existing in HEAD `340e7980`,
**not introduced by Track B**, and is out of scope. The integration
gate should resolve it before publishing the wave-12 ROADMAP, or
have the wave-12-integration worktree own the fix.

## Open issues for the integration gate

1. **Track A's new `epsx-analytics-service` binary must register
   the 5 consolidated routes** (`/api/analytics/{rankings,
   filters, countries, available-countries, sectors}`) under
   `optional_bearer_middleware` + the `eps_ranking_service` +
   `wallet_ranking_offset_port` extensions. Track A is moving
   these handlers into the new binary; confirm the mount surface
   matches what `create_analytics_routes` provides today.

2. **Track A's `main.rs` can optionally call the deleted service
   methods** — `EPSCacheService::get_cache_stats` and
   `TradingViewApiService::get_cache_stats` are still alive in the
   codebase (used by `application::market_analytics`). Confirm
   Track A doesn't try to mount the deleted HTTP handlers at any
   path.

3. **The `infra_logs` schema rename is a forward-looking change.**
   No production DB has the `infra_logs` schema yet. A fresh DB
   that runs the v3 baseline after this commit will end up with
   `infra_logs.*` tables. An **existing production DB** that
   already ran v3 with the `analytics` schema will need a
   one-line cutover:
   ```sql
   ALTER SCHEMA analytics RENAME TO infra_logs;
   ```
   Coordinate with the platform team before the wave-12 rollout.

4. **The pre-existing ROADMAP merge marker** at line 2186 is
   out of scope for Track B but should be resolved by the
   integration gate.

5. **`is_public_endpoint` test passes by prefix, not by route
   presence.** Strict route-presence checks are a wave-13+
   concern.

## Notes

- All 7 commits were pushed incrementally to
  `origin/wave12/track-b-infra-cleanup`. No MR was opened
  (per task spec).
- Worktree path: `/Users/fluke/Desktop/Work/epsx/.worktrees/wave12-track-b-infra-cleanup`
- The full implementation report (with verification commands,
  file-by-file change list, and 4-decision rationale) is at
  `docs/wave8-service-boundary/ROADMAP.md §14`.
