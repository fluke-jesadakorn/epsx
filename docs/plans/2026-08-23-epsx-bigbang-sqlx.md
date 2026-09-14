# EPSX Big-Bang Architecture Improvement — Single Branch `migration/dioxus-microservices`

**Date:** 2026-08-23
**Branch:** `migration/dioxus-microservices` (base `12dde61c`)
**Mode:** Big-Bang (4 DBs at once), single branch, no sub-branches
**Decisions locked:**
- ORM: **Diesel → sqlx 0.8** (big-bang)
- Kernel: **epsx-contracts canonical** (deprecate `kernel`)
- Auth: **RS256 JWKS only** (`epsx-service-auth`, remove HS256 fallback)
- Analytics: **Extract `epsx-analytics-protocol` crate** (cut `epsx` dep)
- Event bus: **Redis Streams canonical** (`epsx-events`, remove InMemory no-op)
- BFF: **Full generic `TypedBffSession<C: CookieClient>`**
- Infra: single-branch parallel edits, no prod guard

---

## 1. Context

Monorepo 42 members (`Cargo.toml:3`) with dual ORM (`diesel` 47 migrations + `sqlx`), dual kernel (`kernel` vs `epsx-contracts`), 4 auth impls, analytics compiling whole backend (`apps/analytics/Cargo.toml:21`), god container 1004 LOC (`container/simple_container.rs:46`), dual event buses, BFF duplication ~400 LOC. Previous review identified these as critical tech debt; user chose big-bang on single branch with accepted downtime.

Current branch `migration/dioxus-microservices` at `12dde61c` has 50 modified files uncommitted (`git status` 2026-08-23) — snapshot as `commit 0` before refactor.

---

## 2. Goals / Non-Goals

**Goals:**
- Single `sqlx::PgPool` per DB, remove `diesel` + `diesel-async` + `deadpool-diesel` + `tokio-postgres`
- Single kernel `epsx-contracts`, fix `Token::decimals` USDT/USDC 18→6
- Single auth `JwksVerifier` RS256, remove HS256 dev secret
- Analytics true microservice (no `epsx` dep) via protocol crate
- Single event bus Redis Streams, fix outbox atomicity (`simple_container.rs:257`)
- Single BFF session generic, harden pay BFF, split dioxus_ui leaf
- Infra: remove `hostAliases 192.168.5.1`, `Box::leak`, split container

**Non-Goals:**
- No schema changes beyond `diesel_initial_setup` removal; `up.sql` reused
- No new business logic
- No prod deployment (user accepts downtime, but no auto-deploy)

---

## 3. Execution Model — Single Branch, Parallel Agents

All 5 agents edit **same branch `migration/dioxus-microservices`**, file-level exclusive (no overlap). No sub-branches.

```
12dde61c ──► commit 0 snapshot (50 dirty files) ──► parallel edits (A-E) ──► single squash commit ──► verify
```

If agents touch same file → sequential merge, not parallel.

---

## 4. Phase 1 — Data Big-Bang (Agent A)

### 4.1 Snapshot & Freeze
- `git add -A && git commit -m "chore: snapshot WIP before bigbang-sqlx"`
- `pg_dump epsx_prod epsx_payments_prod epsx_analytics_prod epsx_notifications_prod` + `git tag pre-sqlx-bigbang`

### 4.2 Pool Unification (big-bang)
- **Delete:** `shared/rust/epsx-database-pools/src/diesel_connection_manager.rs:395` (`TlsPool`, `TlsConnectionManager`, `OnceLock` globals), `apps/backend/diesel*.toml` (4 files), `apps/backend/src/schemas/primary.rs:1500` (gitignore + generate)
- **Add:** `shared/rust/epsx-database-pools/src/sqlx_pool.rs`:
  ```rust
  pub struct SqlxPoolConfig { url, max_conns 10/5/8, idle_timeout 30s, acquire_timeout 5s }
  pub fn create_pools() -> (PgPool, PgPool, PgPool, PgPool)
  ```
  Replaces `GLOBAL_DIESEL_POOL`, `GLOBAL_ANALYTICS_POOL`, `GLOBAL_NOTIFICATIONS_POOL`, `GLOBAL_PAYMENTS_POOL` at `diesel_connection_manager.rs:26`
- **Fix:** `apps/backend/src/main.rs:52` `Box::leak(TlsPool)` → `Arc<PgPool>`
- **Migrate:** `apps/backend/src/bin/migrate.rs` + `xtask/src/workspace_tools.rs:589` → `sqlx::migrate!("./migrations/core")` etc.

### 4.3 Repositories (all 4 DBs)
- `apps/backend/src/infrastructure/adapters/repositories/mod.rs:113` (6 adapters) + `infrastructure/repositories/*` (4) + `apps/backend/src/auth/unified_permission_service.rs:919` + `domain/developer_portal/entitlement_service.rs:1` + `usage_service.rs:1` + `domain/wallet_management/domain_services/wallet_permission_service.rs:1`
- Transform: `diesel::sql_query` / `schema::table.filter().load()` → `sqlx::query_as!(Struct, "SELECT ...")`
- PG function `wallet_has_permission($1,$2)` stays in SQL, called via `sqlx`

### 4.4 Migrations
- Keep 47 `up.sql` as-is, remove `00000000000000_diesel_initial_setup` + `.diesel_lock`, add `sqlx migrate` runner
- Files: `migrations/core/` (23), `migrations/payments/` (8), `migrations/notifications/` (12), `migrations/analytics/` (4)

### 4.5 Outbox + Event Bus
- `simple_container.rs:257` move `TransactionalOutbox` from `analytics_pool` → `primary_pool` (atomic `BEGIN; INSERT agg; INSERT outbox; COMMIT`)
- `shared/rust/events/src/lib.rs:1` stays canonical; delete `shared/rust/epsx-contracts/src/event_publisher_port.rs` InMemory impl + `simple_container.rs:488` no-op, add `RedisEventPublisher: EventPublisherPort` bridge

### 4.6 Verification
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --locked`
- `psql` row-count diff 4 DBs == 0

---

## 5. Phase 2 — Kernel / Auth / Proto (Agent B)

### 5.1 Kernel Canonical
- Move `ChainId` (56/97/1/42161/8453/137), `Token` (USDT/USDC/BNB/ETH), `Money` from `shared/rust/kernel/src/lib.rs:338` → `shared/rust/epsx-contracts/src/value_objects/` or `shared/rust/web3`
- Fix `Token::decimals() 18 → 6` for USDT/USDC (`kernel/src/lib.rs:166`)
- Deprecate `kernel`: `pub use epsx_contracts as kernel` shim one release, then delete crate from `Cargo.toml:4`
- Sync `shared/rust/web3/src/lib.rs:provider_for_chain` to support 6 chains (not just 56/97)

### 5.2 Auth RS256 Only
- Delete: `shared/rust/bff/src/auth_helpers.rs:88`, `shared/rust/auth/src/lib.rs:1`, `shared/rust/crypto/src/lib.rs` HS256 `JwtService`
- Delete fallback `EPSX_JWT_SECRET="epsx-dev-secret-do-not-use-in-prod"` in `shared/rust/bff/src/session.rs:1`
- Keep: `shared/rust/epsx-service-auth/src/lib.rs:1` `JwksVerifier` only; add local JWKS mock for `cargo xtask dev --all`
- Delete legacy `bff/middleware.rs:verify_bearer_or_cookie`

### 5.3 Proto Dedup
- Delete `shared/proto/identity.proto` (stale duplicate), keep `proto/identity/v1/identity.proto`
- Fix `shared/rust/epsx-identity-service/build.rs:1` + `apps/analytics/build.rs:1` to single path
- Align `prost 0.13` / `tonic 0.12` in `Cargo.toml:66` (`prost 0.13` bump comment)

---

## 6. Phase 3 — Service Boundary (Agent C)

### 6.1 Analytics Protocol Extract
- **New crate** `shared/rust/epsx-analytics-protocol/Cargo.toml` (deps: `serde`, `chrono`, `uuid` only):
  - `WalletRankingOffsetQuery` trait (`epsx-contracts/src/wallet_ranking_offset_query.rs:1`)
  - `RankingOffset` VO, `GetWalletRankingOffsetReq/Resp` DTOs, `RankingOffsetChange` SSE schema
- Fix `apps/analytics/Cargo.toml:21`: `epsx={path="../backend"}` → `epsx-analytics-protocol` + `epsx-service-auth`
- Fix `apps/analytics/src/grpc_client.rs:1`, `src/lib.rs:46` re-export seam, `build.rs` tonic codegen

### 6.2 Split `epsx-identity-shared`
- Split `shared/rust/epsx-identity-shared/src/lib.rs:1` (currently pulls `ethers 2.0`+`diesel`+`dashmap`):
  - `epsx-identity-protocol` (light DTOs) → consumed by BFFs/services
  - `epsx-identity-service-impl` (heavy) → only identity service

### 6.3 Domain Leaks
- `domain/developer_portal/entitlement_service.rs:1` + `usage_service.rs:1` → create `EntitlementRepositoryPort` instead of `DbPool`+`diesel::sql_query`
- `domain/payment/repository_ports/payment_context_port.rs:1` → port defines own DTO, not import adapter type (`infrastructure/adapter_repositories::DbPool` leak)
- `domain/wallet_management/domain_services/wallet_permission_service.rs:1` → depend on `BlockchainValidationPort` trait, not concrete `BlockchainValidationClient`
- `apps/backend/src/web/mod.rs:148` → route through `application` use-cases, not direct `domain` (reduce `web` grep 91 `use crate::domain`)

---

## 7. Phase 4 — BFF Consolidation (Agent D)

### 7.1 TypedBffSession Generic
- **New** `shared/rust/bff/src/typed_session.rs`:
  ```rust
  pub struct TypedBffSession<C: CookieClient> { verifier: Arc<JwksVerifier>, client_id: &'static str, _m: PhantomData<C> }
  impl<C> TypedBffSession<C> { verified_access_token(), refresh_token(), current_user(), handle_siwe_verify() }
  ```
- Migrate `apps/frontend/src/auth.rs:145` (122 lines) + `apps/admin/src/auth.rs:141` (101 lines) + `apps/admin/src/session_auth.rs:433` (SIWE) → single generic
- Eliminates ~400 LOC duplication, single `FRONTEND_CLIENT_ID` vs `ADMIN_CLIENT_ID` param

### 7.2 Pay BFF Hardening
- `apps/pay/src/main.rs:330` `resolve_pay_link_redirect` per-request `ServiceClient` → reuse `Arc<ServiceClient>` + add `security_headers` + `validate_auth_url` (like frontend/admin `main.rs:262`)
- Add `Cache-Control: private, no-store` + `Vary: Cookie` to `pay_ssr_fallback:390`
- Add `browser_runtime_router` if needed

### 7.3 Dioxus UI Split
- Split `shared/rust/dioxus_ui/src/primitives/` (50 files) → `shared/rust/epsx-dioxus-primitives` leaf crate with `ssr`/`web` features (reduces rebuild)
- Document boundary: `epsx-templates` (CSS vars) vs `epsx-renderer` (inline style) vs `dioxus_ui` (`rsx!`)

---

## 8. Phase 8 — Infra (Agent E)

- `infrastructure/kubernetes/base/backend/deployment.yaml:1` `hostAliases host.docker.internal → 192.168.5.1` → `Service ExternalName` or `hostPort` + DNS (`postgres.epsx-prod.svc.cluster.local`)
- `apps/backend/src/main.rs:52` `Box::leak(TlsPool)` → `Arc<PgPool>` with graceful shutdown
- `apps/backend/src/infrastructure/container/simple_container.rs:1004` → split into `ContainerBuilder` + `ReadModelContainer`/`WriteModelContainer` + `ServerlessContainer`
- `Cargo.toml:202` vendored `vendor/aws-runtime` patch → remove when `aws-runtime 1.7.5+` released
- `apps/backend/src/schemas/primary.rs` → `.gitignore` + generate in CI (`cargo xtask` build time)

---

## 9. Verification Gates (all phases, single branch)

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo xtask audit no-node --strict && cargo xtask assets verify && cargo xtask k8s-audit
cargo xtask e2e doctor && cargo xtask e2e verify-artifacts   # groups 0-9
forge test --root apps/contracts
psql row-count diff 4 DBs == 0
curl checks: api.epsx.io/health, epsx.io, admin.epsx.io, pay.epsx.io
```

Rollback: `pg_restore` 4 dumps + `git revert pre-sqlx-bigbang` (drill once on staging/Colima before prod).

---

## 10. Risks (big-bang accepted)

| Risk | Mitigation |
|------|------------|
| 1 DB fail → all 4 revert | Snapshot + tag + freeze window; drill rollback on Colima |
| `wallet_has_permission` PG function mismatch | Test `permission_validation_middleware.rs:487` 100% paths |
| Workspace compile fail blocks all | No Diesel fallback; fix forward only |
| 50 dirty files + big-bang conflict | Commit 0 snapshot first, then parallel file-exclusive edits |

---

## 11. Commit Plan (single branch)

```
12dde61c (base)
  → 12dde61d chore: snapshot WIP before bigbang-sqlx (50 files)
  → <parallel A-E edits, single branch>
  → xxxxxxxx refactor(bigbang): Diesel→sqlx + kernel canonical + RS256 only + protocol extract + TypedBffSession
```

No sub-branches. All agents target `migration/dioxus-microservices`.

---

## 12. References

- `Cargo.toml:3` workspace 42 members, `Cargo.toml:66` prost skew, `Cargo.toml:202` aws-runtime patch
- `apps/backend/src/infrastructure/container/simple_container.rs:46` god container, `:257` outbox atomicity
- `apps/backend/src/main.rs:52` Box::leak, `apps/analytics/Cargo.toml:21` backend dep
- `shared/rust/kernel/src/lib.rs:338` dual kernel, `:166` decimals bug
- `shared/rust/epsx-service-auth/src/lib.rs:1` canonical JWKS, `shared/rust/bff/src/auth_helpers.rs:88` HS256 fallback
- `proto/identity/v1/identity.proto` vs `shared/proto/identity.proto` duplicate
- `shared/rust/events/src/lib.rs:1` vs `epsx-contracts InMemoryEventBus` dual bus
- `apps/frontend/src/auth.rs:145` vs `apps/admin/src/auth.rs:141` BFF duplication
- `infrastructure/kubernetes/base/backend/deployment.yaml:1` hostAliases
