# Wave 11 / Integration Gate — Final Report

## 1. Summary

All 3 wave-11 producer branches merged into `wave11/integration` on top of
`origin/migration/dioxus-microservices` (`1014d8c4`). Cross-track conflicts
resolved in `domain/payment/repository_ports/mod.rs` (the inline
`PaymentContextRepositoryPort` trait vs the `pub use` re-export — kept
the `pub use` form, deleted the inline trait), `infrastructure/adapters/repositories/mod.rs`
(the duplicate `pub mod payment_repository_adapter` — removed the second),
`web/auth/app_state.rs` (the 4 new port fields — took the union), `web/mod.rs`
(the `create_router` wiring — kept all 4 port setters), and `web/routes/unified_router.rs`
(the `UnifiedRouteBuilder` struct + the `with_*_port` methods + the
`create_app_state` chain — kept all 4 ports).

5 new ports are now in the production DI graph:
- `Arc<dyn PaymentRepositoryPort>` (Track A — collapses the 8 cross-pool sites)
- `Arc<dyn CreditRepositoryPort>` (Track A — credit balance read)
- `Arc<dyn PaymentContextRepositoryPort>` (Track B — V2 dynamic payment links)
- `Arc<dyn SubscriptionRepositoryPort>` (Track B — subscriptions + stock-ranking query)
- `Arc<dyn EventPublisherPort>` (Track C — R7 + the 3 R8 orphan events)

Final HEAD on `wave11/integration`: see `git log origin/migration/dioxus-microservices..HEAD --oneline` — 4 commits (3 merge commits + the final integration commit).

## 2. Merge log

| # | Commit | Description |
|---|--------|-------------|
| 1 | `6d96dd6a` | Merge Track A — PaymentRepositoryPort + 8 cross-pool sites collapsed (owner-takeover, 9 commits) |
| 2 | `a6a18128` | Merge Track B — outbound-leakage fold + SubscriptionRepositoryPort (7 commits, cross-track conflict in `repositories/mod.rs` resolved: removed duplicate `pub mod payment_repository_adapter`) |
| 3 | `5ba83160` | Merge Track C — EventPublisherPort + R8 orphan events (4 commits, conflict in `ROADMAP.md` §13 preamble resolved by keeping the "HEAD" version) |
| 4 | (this commit) | Integration gate: schema cutover migration, replicate-plans script, end-to-end smoke test, this report |

## 3. Production cutover checklist (6 steps)

The integration gate's code is on origin, but the **production database** is not yet cut over. The wave-11 cutover is a 6-step runbook that the production team executes by hand:

1. **Apply the new migration** against `$DATABASE_URL`:
   ```
   diesel migration run --database-url $DATABASE_URL
   ```
   This creates the `payments` schema, the `payments.plans` replica, the `sync_plans_from_public()` function, and the `sync_plans_to_payments_schema` trigger.

2. **Run the one-shot bulk-copy script** against the same DB:
   ```
   DATABASE_URL=$DATABASE_URL ./infrastructure/scripts/wave11-replicate-plans.sh
   ```
   The script does `INSERT INTO payments.plans SELECT * FROM public.plans ON CONFLICT (id) DO NOTHING` in a single transaction. Idempotent — re-runs are safe. The script verifies the source/destination row counts match + that the sync trigger is registered before exiting 0.

3. **Set `PAYMENTS_DATABASE_URL`** in `.env.prod` to point at the payments database. Until this is set, the in-process port's `get_diesel_pool()` fallback still works (the pre-cutover state), but the new `payments` schema's `plans` replica is unused.

4. **Restart the backend.** The new `UnifiedRouteBuilder` chain + the 5 ports get wired up at boot. Verify the container logs show:
   - `PaymentRepositoryPort wired: PaymentRepositoryAdapter`
   - `CreditRepositoryPort wired: CreditRepositoryAdapter`
   - `PaymentContextRepositoryPort wired: PaymentContextRepositoryAdapter`
   - `SubscriptionRepositoryPort wired: PaymentSubscriptionRepositoryAdapter`
   - `EventPublisherPort wired: InProcessEventPublisher`

5. **Verify `/api/payments/*` routes are healthy.** Hit `GET /api/payments/admin/payment-analytics?window=last_30_days` and check the response includes `daily_revenue`, `plan_breakdown`, `payment_methods`, `trends` slices. Pre-cutover, the response is 500 (`plans` not in `payments` schema); post-cutover, the response is 200 with the JOIN'd data.

6. **Verify the `payments.plans` trigger is firing.** In one psql session, run:
   ```
   UPDATE public.plans SET name = name || ' [wave11-test]' WHERE id = '<some-uuid>';
   ```
   then in another session:
   ```
   SELECT name FROM payments.plans WHERE id = '<some-uuid>';
   ```
   The name should reflect the update. Roll back the test name after.

If all 6 steps pass, the wave-11 cutover is complete. The 5 new ports + the cross-schema `plans` replica are now the production data path for all 8 cross-pool handler sites.

## 4. End-to-end smoke test (6/6 pass)

`apps/backend/tests/wave11_smoke.rs` (new file in this commit) — exercises the integration at the type level (no live DB needed):

```
running 6 tests
test all_eight_wave10_wave11_ports_are_dyn_safe ... ok
test payment_repository_adapter_impls_port ... ok
test cutover_migration_has_required_ddl_statements ... ok
test cutover_script_has_required_production_steps ... ok
test three_r8_orphan_events_construct_and_dispatch ... ok
test payment_row_with_plan_name_round_trips_through_serde ... ok

test result: ok. 6 passed; 0 failed
```

The 6 tests cover:
1. **All 8 ports are dyn-safe** — the `_assert_*_object_safe` functions + the explicit `Arc<dyn ...Port>` type annotations force the trait checker to verify object-safety for all 5 wave-11 ports + the 3 wave-10 ports (Notification, PermissionAuthority, Pubsub) + `WalletRankingOffsetQuery` (the kernel-level port from wave 10 R6).
2. **`PaymentRepositoryAdapter` impls `PaymentRepositoryPort`** — the trait bound forces the type checker to verify the impl. This is the production DI graph's core coercion.
3. **The 4 required DDL statements are in the cutover up.sql** — `CREATE SCHEMA`, `CREATE TABLE IF NOT EXISTS LIKE`, `CREATE OR REPLACE FUNCTION`, `CREATE TRIGGER`. A regression that silently deletes the trigger breaks the cutover; this test catches it.
4. **The cutover script has the 5 required production steps** — `psql`, `INSERT INTO payments.plans`, `ON CONFLICT`, `DATABASE_URL` (env-var check), and the trigger-name verification. The script is a one-shot ops action; a regression that drops the `ON CONFLICT` makes the script non-idempotent.
5. **The 3 R8 orphan events construct + dispatch** through `Box<dyn DomainEvent>` — confirms the R7 `EventPublisherPort` can fan them out at runtime.
6. **`PaymentRowWithPlanName` round-trips through serde** — the DTO travels over the future HTTP boundary (the `epsx-payments` binary), so a serde regression breaks the wire protocol. The test catches that without needing a live HTTP server.

## 5. Final cargo results

```
cargo check --workspace           : clean (warnings only — pre-existing)
cargo test -p epsx --lib          : 458 passed, 0 failed, 8 ignored (was 414 pre-wave-10; +44 over the wave-11 lift)
cargo test -p epsx-contracts --lib : 47 passed, 0 failed
cargo test -p epsx --test wave11_smoke : 6 passed, 0 failed (the new smoke test)
cargo build -p epsx --bins         : clean (epsx-backend, content, identity, notification all build)
```

The +44 tests are the new payment-port tests, the SubscriptionRepositoryAdapter round-trip tests, the stock-ranking canary tests, the EventPublisherPort tests, and the 4 in-process adapter tests. The 8 ignored are pre-existing (Diesel async pool construction without a live DB).

## 6. Open issues for wave 12 (analytics service lift)

Per the wave 11 plan §5, wave 12 is the analytics service lift. Open carryovers from wave 11:

1. **`admin_list_payments_handler`** still uses `plans::table` directly on the primary pool — out of scope for wave 11's 8-site collapse. Wave 12 should add a `list_admin_payments_with_plan_names` port method (same shape as the user-side variant) and route this handler through the port.

2. **The 88 `event_bus` direct references** migrated to `Arc<dyn EventPublisherPort>` in this wave, but 7 out-of-scope references remain (the `DomainEventBus` trait def, `SimpleEventBus` impl, module re-exports, prelude re-export, shim comments). Wave 12 cleanup: delete the `DomainEventBus` trait + `SimpleEventBus` shim + the prelude re-export.

3. **The `EventPublisherPort`'s defensive `Option<...>` on the AppState field** — the `None` default means a misconfigured server returns 503 from event-publishing handlers instead of panicking. Wave 12 can decide whether to keep the `Option` or make it a required field.

4. **The in-process `EventPublisherPort::with_bus(legacy_bus)` constructor** exists for backward compat. Wave 12 can drop the `SimpleEventBus` shim + the `with_bus` method once the kernel event bus is fully retired.

5. **`payment_subscription_port` field on `AppState` is `Option<...>`** — the in-process adapter is built in `web/mod.rs::create_router` from the payments pool. If the pool is missing, the field is `None` and the stock-ranking query returns empty. Wave 12 can make it required (panic-fast) if production pools are guaranteed.

6. **`epsx_payments` binary** (the future service) is not in this integration — the HTTP impl of the 5 ports lives in the binary worktree (wave-N+2). The integration gate ports are all the in-process impl.

7. **Production cutover checklist** (the 6 steps in §3 above) — the production team runs this manually. If anything fails, the rollback path is: revert `PAYMENTS_DATABASE_URL`, restart, the new ports fall back to `None` and the legacy code paths take over.

## 7. Files added / modified in the integration gate

| File | Action | Purpose |
|------|--------|---------|
| `apps/backend/migrations/payments/20260613000000_replicate_plans_into_payments_schema/up.sql` | NEW | Schema cutover: CREATE SCHEMA, CREATE TABLE LIKE, sync function, sync trigger |
| `apps/backend/migrations/payments/20260613000000_replicate_plans_into_payments_schema/down.sql` | NEW | Idempotent down: DROP TRIGGER, DROP FUNCTION (no DROP of public.plans) |
| `infrastructure/scripts/wave11-replicate-plans.sh` | NEW | One-shot bulk-copy script: 6-step runbook with audit log + rollback safety |
| `apps/backend/tests/wave11_smoke.rs` | NEW | 6-test end-to-end smoke (object-safety, port impls, serde round-trip, DDL shape, script shape, R8 events) |
| `deliverable.md` | REPLACED | This file (replaces the Track A / Track B / Track C deliverables) |
| `docs/wave8-service-boundary/ROADMAP.md` §13 | MERGED | Track A / B / C implementation reports — all 3 are now in the ROADMAP |

## 8. Verification

- `cargo test -p epsx --test wave11_smoke` → 6/6 pass
- `cargo test -p epsx --lib` → 458/458 pass
- `cargo test -p epsx-contracts --lib` → 47/47 pass
- `cargo check --workspace` → clean
- `cargo build -p epsx --bins` → clean
- Branch is on top of `origin/migration/dioxus-microservices` @ `1014d8c4` (wave-10 integration) + 4 integration commits + 1 final commit
- All 3 producer branches are on origin (`wave11/track-a-payment-repo-port` @ `cebeb049`, `wave11/track-b-leakage-fold` @ `853d359e`, `wave11/track-c-event-port` @ `48274b9a`)
- The schema cutover migration is reversible (down.sql has DROP TRIGGER + DROP FUNCTION, no DROP of `public.plans`)

## 9. Final commit hash

The final commit hash for this integration branch is recorded in
`git log origin/migration/dioxus-microservices..HEAD` and in the
per-track `/Users/fluke/.mavis/plans/plan_a0283b27/outputs/` deliverables
(which are kept for audit).
