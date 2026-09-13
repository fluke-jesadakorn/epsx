# Auth + Identity — Service-Boundary Audit

**Scope:** EPSX Rust backend `auth` and `user/identity` domains, in the
modular monolith at `apps/backend/`.

**Question being answered:** *How cleanly can auth + identity be split
into a standalone microservice, given the project-wide rule that
permission/plan logic must live in the Rust backend?*

**Read-only audit.** No source code was modified. All claims are backed
by line-numbered `rg` / `Read` evidence (see the file-level reports
collected below).

---

## 1. Constraints (must read first)

This audit is bounded by a hard architectural rule from the
project-level `CLAUDE.md` (file: `CLAUDE.md`, section: "Architecture
Constraints"):

> ### Permissions & Plan Logic — Backend Only
> All business logic related to permissions, plan access, ranking
> offsets, feature flags, and subscription rules **must be implemented
> in the Rust backend only**. Frontend (`apps/frontend`) and
> admin-frontend (`apps/admin-frontend`) are UI-only layers.

This rule constrains what the identity-service split can look like:

1. **A split into a separate identity microservice is allowed** for:
   - User / role / permission table ownership (`wallet_users`,
     `permissions`, `plans`, `wallet_plan_assignments`,
     `wallet_direct_permissions`).
   - SIWE challenge issuance and verification state.
   - OIDC token minting (RS256 access + refresh + id tokens).
   - JWKS publication at `/.well-known/jwks.json`.
   - Refresh-token rotation and revocation.

2. **RBAC enforcement (who can call what) MUST stay in the deployed
   business-service binary.** The identity service is **not trusted at
   request time.** Every business service must validate permissions
   locally. This is already how the code works — the canonical rule
   store is `apps/backend/src/core/permissions.rs` (217 L, 5 `pub fn`,
   pure CPU, no I/O) and the bearer middleware calls it directly:

   ```rust
   // src/web/middleware/permission_validation_middleware.rs
   crate::core::permissions::has_permission(&ctx.permissions, required)
   crate::core::permissions::has_permission(&user_context.permissions, required)
   ```

   This must not be replaced with a per-request network call to the
   identity service. A split that removes this file from the business
   binary violates the rule.

3. **A "centralized policy engine" split is OUT OF SCOPE.** That is a
   different design (move `core::permissions` to a shared service that
   all business services call) and the user did not ask for it. The
   audit's recommendation (below) preserves local rule evaluation.

4. **What is allowed, concretely, in two split shapes:**

   **Shape A — Identity service owns data + mints tokens; business
   services keep local rule evaluation.** The identity service
   publishes a public key at JWKS. Business services fetch JWKS at
   boot and cache the active public key. JWT validation stays
   in-process (no per-request network hop). Plan/role changes
   propagate via:
   - DB subscription: business services read from a logical
     `has_permission` projection that is built locally (or via a CDC
     stream from the identity DB).
   - Token rotation: identity publishes a `key.rotated` event; business
     services refresh their cached key.

   **Shape B — Shared library.** A new `epsx-identity` Rust crate
   re-exports `core::permissions`, the `KeyManager`, the
   `OpenIDTokenService::validate_access_token` function, and the
   `UnifiedPermissionService` (read-only API). Every business-service
   binary depends on it. The identity service is the additional crate
   plus the user/role/plan tables. The split is in the binary graph;
   the network shape is unchanged.

   **Both shapes preserve the "permissions in the Rust backend" rule.**
   Shape A is the larger refactor; Shape B is the smaller one. The
   audit's recommendation is **Shape B first**, then evolve toward
   Shape A if the data-ownership benefit materializes.

---

## 2. Domain map

### 2.1 `domain/auth/` — port layer only (1 trait, 22 lines)

- `apps/backend/src/domain/auth/mod.rs` (1 L) — re-export stub.
- `apps/backend/src/domain/auth/ports.rs` (22 L) — single trait
  `IdentityProviderPort` with three async methods:
  - `get_access_token(&self) -> Result<String, anyhow::Error>` (L8)
  - `set_custom_claims(&self, user_id: &str, claims: &HashMap<String, Value>) -> Result<(), anyhow::Error>` (L11–15)
  - `get_user_claims(&self, user_id: &str) -> Result<HashMap<String, Value>, anyhow::Error>` (L18–21)

  Uses `async_trait`. Returns `anyhow::Error`. No `Send + Sync`
  enforcement on `Value`.

- **No aggregates, no value objects, no events, no services.** This
  module is purely a hexagonal port for talking to an upstream
  identity provider. The "auth" responsibility is delegated entirely
  to the upstream provider; this layer only declares what the backend
  wants to do with it (mint tokens, set/get custom claims).
- **Clean split candidate** — only one trait to relocate.

### 2.2 `domain/permission_management/` — bounded context with loose ends

Layout under `apps/backend/src/domain/permission_management/`:

| Subdir | Files | Total LoC |
|---|---|---|
| `aggregates/` | `mod.rs`, `plan.rs`, `policy.rs` | 402 + 124 + small |
| `entities/` | `mod.rs`, `plan_assignment.rs` | 73 + small |
| `value_objects/` | `mod.rs` + 6 VOs | 287 |
| `domain_services/` | `mod.rs`, `permission_validation_service.rs` (43), `plan_assignment_service.rs` (47) | ~96 |
| `repository_ports/` | `mod.rs` + 3 ports (plan, plan_assignment, policy) | ~105 |
| `events/` | `mod.rs` + 2 files (plan_events, policy_events) | 7 events |

`mod.rs` re-exports `PlanId` from `domain::subscription_management` —
a **cross-context dep** that the split must resolve (move to shared
kernel, or duplicate as a newtype at the boundary).

**Key findings:**

1. **`Plan` aggregate is a god-aggregate** (`aggregates/plan.rs`, 402 L,
   21 fields). It bundles four concerns:
   - **Identity** (3 fields): `id`, `name`, `slug`.
   - **Catalog / pricing** (5 fields): `price`, `currency`,
     `billing_cycle`, `is_promoted`, `tier_level`.
   - **Display taxonomy** (3 fields): `plan_type`, `plan_category`,
     `plan_group`.
   - **Access control** (6 fields): `permissions: HashSet<PermissionString>`,
     `is_active`, `is_system`, `grace_period_hours`,
     `auto_assign_enabled`, `max_members`, `is_public`.
   - **Free-form** (1 field): `metadata: serde_json::Value`.
   - **Lifecycle** (1 field): `base: AggregateBase`.

   The pricing fields are clearly catalog/SSR concerns, not RBAC. The
   split should either (a) move the pricing fields to a `PlanOffering`
   value object on the catalog side, or (b) accept that the IAM
   service is also the catalog-of-record for plans.

2. **`Policy` aggregate is parallel and unconnected to `Plan`**
   (`aggregates/policy.rs`, 124 L, 6 fields: `id`, `name`, `description`,
   `rules: Vec<PolicyRule>`, `is_active`, `priority`). Has no
   `Plan → Policy` reference. Marked `#[allow(dead_code)]` in places.
   The split should pick one as the single source of truth.

3. **`PermissionValidationService` (`domain_services/permission_validation_service.rs`,
   43 L) is a pure stateless service.** Three public functions:
   - `validate_permission_format` (L9–25) — duplicates the
     `platform:resource:action` check that `PermissionString::new`
     already does (same regex-like check, two places).
   - `group_contains_permission` (L28–33) — 1-line
     `HashSet::contains` wrapper.
   - `matches_wildcard` (L36–42) — naive trailing-`*` only. No `?`
     single-char, no mid-pattern `*`.

   **Does NOT call `core::permissions`** (verified by `rg` — zero
   references). The domain layer is data shape and CRUD scaffolding;
   rule evaluation is happening above it (in
   `permission_validation_middleware.rs` and `auth_service.rs`).

4. **`PlanAssignment` entity** (`entities/plan_assignment.rs`, 73 L, 7
   fields) — link record binding `WalletAddress` to `PlanId`. Uses
   `WalletAddress` from `domain::wallet_management` — second
   cross-context dep. **No events emitted**; events exist in
   `plan_events.rs` (`WalletAssignedToPlanEvent` /
   `WalletRemovedFromPlanEvent`) but **nothing in the domain
   constructs them.**

5. **4 of 7 events are orphaned** (defined but never published):
   - `PlanDeletedEvent` (`plan_events.rs:82`)
   - `WalletAssignedToPlanEvent` (`plan_events.rs:113`)
   - `WalletRemovedFromPlanEvent` (`plan_events.rs:152`)
   - `PolicyUpdatedEvent` (`policy_events.rs:45`)

   Only `PlanCreatedEvent`, `PlanUpdatedEvent`, `PolicyCreatedEvent`
   fire (via `AggregateBase::add_event` in the aggregates' `create` /
   `update` constructors). If any consumer relies on the missing
   events, the split exposes it.

6. **Backward-compat aliases in `mod.rs` (L12–62)** — ~30% of the
   file. Migration dead weight (e.g. `PermissionGroup = Plan`,
   `PermissionPlan = Plan`, `GroupAssignment = PlanAssignment`).
   Should be dropped on the split.

---

## 3. Application map (admin + permission_management use cases)

### 3.1 `application/admin/` (4 files, ~82 lines) — single command

- `mod.rs` (1 L) → `commands/`.
- `commands/mod.rs` (1 L) → `assign_admin_plan.rs` (80 L).
- Defines `AssignAdminPlanCommand`, `AssignAdminPlanResponse`,
  `AssignAdminPlanHandler`. Depends on
  `crate::domain::auth::ports::IdentityProviderPort` (L5) only.
  **No touch of permission_management repos/aggregates/event bus.**
  Calls `identity_provider.set_custom_claims(...)` (L70).
- **Hard-coded permission strings at L48–60:**
  `admin:*:*`, `epsx:*:*`, `system_admin`, `module_management`,
  `database_access`, `developer_portal`. These should be domain
  constants or come from an "admin plan" PermissionPlan aggregate.
  Code smell: a domain concept is leaking into the application layer.

### 3.2 `application/permission_management/` (22 files, ~835 lines)

- 5 commands + 4 queries.
- `controllers/` (6 L), `dtos/` (5 L), `ports/inbound/` (6 L) are
  **stub directories** (comments only). The response DTOs live
  inline in `queries/models/*.rs`. There are no
  `permission_management` DTOs in the `dtos/` directory.
- 5 command handlers: `create_plan_handler.rs` (91 L),
  `update_plan_handler.rs` (87 L), `delete_plan_handler.rs` (49 L),
  `assign_wallet_handler.rs` (85 L), `remove_wallet_handler.rs` (49 L).
- 4 query handlers: `get_plan_handler.rs` (64 L),
  `list_plans_handler.rs` (79 L), `get_plan_members_handler.rs` (49 L),
  `get_wallet_plans_handler.rs` (60 L).
- All commands accept a `wallet_address: String` (no domain type at
  the boundary — DTO uses raw `String`).

**Inconsistent event publishing — pre-split bug:**

| Handler | Publishes events? |
|---|---|
| `create_plan_handler.rs:79–81` | Yes — iterates `plan.uncommitted_events()` and publishes |
| `update_plan_handler.rs:77–79` | Yes — same pattern |
| `delete_plan_handler.rs:12` | **No** — stores `event_bus` as `_event_bus`, never uses it |
| `assign_wallet_handler.rs:17` | **No** — same |
| `remove_wallet_handler.rs:13` | **No** — same |

This is a latent bug, not a split-time bug — the 3 `_event_bus` fields
should be wired up (or removed) before the split so the new service
publishes consistently.

**Other smells:**

- N+1 query patterns: `list_plans_handler.rs:54` calls
  `count_plan_members(plan.id())` per plan;
  `get_wallet_plans_handler.rs:41` calls `find_by_id` per assignment.
- **Cross-layer leak:** `list_plans_handler.rs:7` imports
  `crate::web::pagination::Pagination` from web into application.
- `UpdatePermissionPlanCommand.max_members: Option<Option<i32>>` (L17)
  — triple-state for "leave alone / set / clear". A `Patch<T>` newtype
  would be cleaner.
- `ApplicationError` (in `shared/error.rs:7–50`) has PascalCase alias
  methods (L122–156) for backward compat.

### 3.3 Cross-domain calls from permission_management

Only 3 handlers touch `domain::wallet_management`:

- `commands/handlers/assign_wallet_handler.rs:10` — `WalletAddress::new`
- `commands/handlers/remove_wallet_handler.rs:7` — `WalletAddress::new`
- `queries/handlers/get_wallet_plans_handler.rs:7` — `WalletAddress::new`

All three only call the value-object constructor for address-format
validation. **No permission_management handler imports from
`application::wallet_management::*` or any wallet-management
repository.** The blast radius in the application layer is **minimal
— 3 imports total.**

### 3.4 The two parallel "admin" mechanisms — key audit finding

1. `application/admin/commands/assign_admin_plan.rs` writes
   identity-provider custom claims via `IdentityProviderPort`.
2. `application/permission_management/commands/handlers/assign_wallet_handler.rs`
   writes a `PlanAssignment` row in `wallet_plan_assignments`.

**They do not coordinate.** A wallet can be an "admin" via custom
claims but have no `PermissionPlan` row, and vice versa. This is a
likely source of authorization drift, and the split must reconcile
these two paths into one. The `web/admin_assignment.rs:9` route
explicitly calls `AssignAdminPlanHandler::new(identity_provider)` and
bypasses the `PlanAssignmentRepositoryPort` path.

---

## 4. Top-level `auth/` module (the centerpiece)

`apps/backend/src/auth/`: 9 files, **~3,678 lines** total.

| File | LoC | Role |
|---|---|---|
| `mod.rs` | 79 | Re-exports |
| `auth_service.rs` | 526 | `UnifiedWeb3AuthService` — SIWE verify, DB upsert, BSC RPC |
| `token_service.rs` | **765** | `OpenIDTokenService` — OIDC token mint + RS256 JWT validate |
| `verification_service.rs` | 409 | `impl UnifiedWeb3AuthService` — wallet lifecycle, on-chain checks |
| `challenge_service.rs` | 162 | `impl UnifiedWeb3AuthService` — SIWE nonce lifecycle |
| `cache.rs` | 452 | **DEAD CODE** — `SimplifiedAuthCache`, `DashMap`-based, unused |
| `granular_permissions.rs` | 489 | Pure in-memory `GranularPermissionSet` — no I/O |
| `key_manager.rs` | 306 | `KeyManager` — env-driven RSA keypair, JWKS publish |
| `unified_permission_service.rs` | **919** | `UnifiedPermissionService` — DB+Redis permission lookup, 4 SQL functions |

### 4.1 JWT issuance vs JWT validation — the critical observation

These two pathways have very different coupling and must be split
differently.

| Concern | Issuance | Validation |
|---|---|---|
| Entry points | `authenticate_web3_and_issue_tokens` (L153), `issue_tokens_for_user` (L189), `refresh_tokens` (L257), `consume_refresh_token` (L325) | `validate_access_token` (L393) |
| Called from | Login / signup / refresh endpoints | Middleware on **every** request |
| DB / Redis access | Yes (`openid_refresh_tokens` row, `web3_auth_nonces` cleanup, `wallet_users` upsert) | **No** — pure in-memory RSA verify |
| Coupling to identity service | Must own it | **Can stay local** with cached public key |
| Latency budget | 200–800ms acceptable | Sub-ms required |

**The validation pathway is already self-contained.** The body of
`validate_access_token` is just `jsonwebtoken::decode` with the
in-memory RSA public key from `key_manager.current_key().decoding_key`.
**Zero I/O, pure CPU crypto, no cache, no DB.** This is the path that
must remain local even after a split. The bearer middleware
(`web/middleware/bearer_middleware.rs:198`) and the legacy
`web/middleware/auth_middleware.rs:259` both call it.

**The issuance pathway is the only one that needs to move to an
identity service.** Even then, the DB coupling is to a small set of
auth tables (`openid_refresh_tokens`, `web3_auth_nonces`,
`wallet_users`), all of which are owned by the identity service in
the proposed split.

### 4.2 `cache.rs` is dead code

`cache.rs` (452 L) defines `SimplifiedAuthCache` (a `DashMap`-backed
in-process cache) and an `UnifiedPermission` placeholder struct
(commented-out import at L12, empty body at L16). **Not imported by
any other auth file** (verified by `rg`). The actual production
permission cache is `UnifiedPermissionCache` in
`infrastructure/cache/unified_permission_cache.rs` (Redis-backed, 30s
TTL). `cache.rs` is a half-removed legacy; clean it up before the
split.

### 4.3 `granular_permissions.rs` is fully self-contained

`granular_permissions.rs` (489 L) defines `GranularPermissionClaim`,
`PermissionSource` enum (`Subscription | ManualGrant |
TimeLimitedAccess | AdminGrant | SystemGrant | TestAccess`),
`GranularPermissionSet` (with `version: u32`, `hash: String` computed
via non-crypto `DefaultHasher` per L333, `next_validation: i64`),
`PermissionValidationResult`, `ValidationContext`,
`GranularPermissionError`. **No `crate::` imports. No DB, no Redis, no
async, no network.** The `validate_permission` path (L260) is a
HashMap lookup + timestamp comparison. Easy to move.

Note: the `hash` field uses `std::collections::hash_map::DefaultHasher`
which is **not cryptographically secure**. The comment at L332
acknowledges this. If the hash is used for cache-busting across
processes after the split, this is a latent bug.

### 4.4 `key_manager.rs` is env-driven and DI-owned

`KeyManager` (L39) holds a `current_key: RSAKeyPair` plus
`backup_keys: HashMap<String, RSAKeyPair>` (max 3 after `rotate_keys`,
L191). Each `RSAKeyPair` carries both the raw `rsa::{RsaPrivateKey,
RsaPublicKey}` and pre-built `jsonwebtoken::{EncodingKey,
DecodingKey}`.

**Key source — K8s secret path** (`from_env_or_generate`, L84):

```rust
match get_env_var("RSA_PRIVATE_KEY") {
    Ok(private_pem) => match get_env_var("RSA_PUBLIC_KEY") {
        Ok(public_pem) => match get_env_var("RSA_KEY_ID") {
            Ok(kid) => Self::from_pem(&private_pem, &public_pem, &kid),
            ...
```

If all three env vars are present, it loads the persisted PEM keys.
If any is missing, it falls back to `Self::new()` (L74) — **a fresh
2048-bit RSA keygen on every restart**. In `debug_assertions` builds
(L122) it logs the new PEM to stdout so you can copy them into the
secret.

This honors the wave8 `CLAUDE.md` rule: "Persistent RSA keys are
mounted into the backend pod via secret `epsx-backend-keys` from
`.env.prod`. Do not let the backend generate new keys on restart or
sessions will expire."

**Instantiation site:** `infrastructure/container/stateless_service_factory.rs:132`
and `infrastructure/container/simple_container.rs:207` both call
`KeyManager::from_env_or_generate()` at boot. The result is wrapped
in `Arc<KeyManager>` and stored in the app container, then passed
into `OpenIDTokenService::new` (L133). **No `static` / `OnceLock`**
in this file — DI-owned.

**A second, **competing** key manager exists in
`infrastructure/security/key_management.rs` (188 L).** It uses a
global `static KEY_MANAGER: OnceLock<JwtKeyManager>` (line 84), loads
RSA from a `keys/` directory via `generate_or_load()` (L40–80), and
**is not used by any caller in the codebase** (verified by `rg`).
This is a third piece of dead code to clean up before the split —
otherwise the new service will pull in two key-management paths.

### 4.5 `unified_permission_service.rs` — the largest file (919 L)

The single entry point for permission lookups in business code.

**Public surface (line-numbered):**

- `pub struct PermissionDetail` (L41)
- `pub enum PermissionSource { Plan, Direct }` (L54)
- `pub struct PermissionStats` (L61)
- `pub struct GrantPermissionRequest` (L73), `RevokePermissionRequest` (L83)
- `pub struct AssignPlanRequest` (L92), `RemovePlanRequest` (L102)
- `pub struct UnifiedPermissionService` (L115) with `cache: Option<Arc<UnifiedPermissionCache>>`
- `new`, `new_without_cache` (L122, L127)
- `has_permission` (L137) — the most-called function, uses the
  `wallet_has_permission($1, $2)` SQL function
- `get_wallet_permissions` (L187) — uses
  `get_wallet_permissions_detailed_working($1)`
- `get_permission_strings` (L286)
- `has_permissions_batch` (L308) — `wallet_has_permissions_batch($1, $2)`
- `grant_permission` (L361), `revoke_permission` (L436)
- `assign_plan` (L491), `remove_plan` (L561)
- `get_permission_stats` (L615) — `get_wallet_permission_stats`
- `invalidate_wallet_cache` (L666)
- `get_wallet_ranking_offset` (L795) — joined with
  `core::constants::FREE_PLAN_RANKING_OFFSET`

**Dependencies:** `crate::prelude::TlsPool`, `core::errors::AppError`,
`infrastructure::cache::unified_permission_cache::UnifiedPermissionCache`
(Redis), `core::constants::{FREE_PLAN_NAME, FREE_PLAN_RANKING_OFFSET}`,
4 SQL functions. All I/O.

**Two parallel permission models coexist (latent issue, not
split-blocking):**

1. **DB-backed slow path** — `UnifiedPermissionService` (Postgres +
   Redis, 30s TTL).
2. **JWT-embedded fast path** — `OpenIDTokenService::validate_access_token`
   returns `AccessTokenClaims` whose `scope` field is a
   space-separated permission list baked at issue time
   (`token_service.rs:594`). The middleware parses this back into
   `Vec<String>` (`bearer_middleware.rs:202–207`).

   → **A freshly granted permission is invisible to a user until
   their access token expires (≤ 1 hour).** No token revocation list.
   This is the biggest design call-out for any split: the new identity
   service must either accept a 1-hour propagation lag (acceptable for
   non-strict permissions) or add a revocation channel.

### 4.6 Cross-cutting observations

- **No `OnceLock` / `OnceCell` / `Lazy` / `static mut`** in the auth
  module (verified by `rg`). All long-lived state is DI-owned,
  `Arc`-wrapped.
- `auth_service.rs::has_permission` (L384) and `is_admin` (L389) are
  pure `&[String]` checks delegated to `crate::core::permissions` — no
  DB, no JWT.
- `token_service.rs::verify_web3_authentication` (L421) **duplicates**
  the SIWE verification that `auth_service.rs::verify_and_authenticate`
  (L223) does. Intentional (the OIDC endpoint can be called directly
  without going through `UnifiedWeb3AuthService`) but a maintenance
  hazard.
- **Hot-path cost:** JWT validation ~0.05–0.2 ms; permission check
  via `unified_permission_service` ~1 ms on Redis hit, ~5–10 ms on
  miss + Postgres function; full login (SIWE + DB + NFT/Token/DAO RPC
  + JWT sign) dominated by 3 BSC `tokio::join!` calls in
  `auth_service.rs:317–321` — 200–800 ms depending on RPC provider.

---

## 5. Web layer (auth, middleware, admin, user)

### 5.1 `web/auth/` — login/signup/refresh HTTP handlers

Files: `mod.rs`, `handlers.rs`, `app_state.rs`, `integration_tests.rs`.
- `web/auth/handlers.rs:459` calls `validate_access_token` (admin-side
  validation path).
- `web/auth/app_state.rs` imports `domain::auth::ports` for app-state
  construction (1 import).

### 5.2 `web/middleware/`

- `auth_middleware.rs` (legacy SIWE-header flow) — calls
  `crate::auth::UnifiedWeb3AuthService` (L22) and `validate_access_token`
  (L259).
- `bearer_middleware.rs` (primary flow) — calls
  `validate_access_token` (L198); extracts from `Authorization: Bearer
  …` header or `epsx.access_token` HttpOnly cookie at L151–174; splits
  `claims.scope` at L202–207; **falls back to `validate_api_key`** at
  L123 → L273 (SHA-256 + DB lookup) on JWT failure.
- `permission_validation_middleware.rs` (475 L) — calls
  `crate::core::permissions::has_permission` directly (2 call sites).
  This is the in-process permission gate; it must stay in the business
  binary.
- `rate_limit_middleware.rs`, `rate_limiter.rs`,
  `multi_level_rate_limiter.rs`, `governor_limiter.rs` — non-auth
  (rate limiting only).
- `security_headers.rs`, `usage_tracking_middleware.rs` — non-auth.

### 5.3 `web/admin/auth_handlers/` — admin-side auth endpoints

Files: `mod.rs` (23 L), `types.rs`, `gate_handlers.rs` (admin gating),
`permission_handlers.rs` (admin permission grant/revoke),
`wallet_handlers.rs` (admin wallet endpoints).

### 5.4 `web/admin/permissions/` — admin permission CRUD HTTP

`mod.rs`, `validation.rs`, `available.rs`, `bulk.rs`, `direct.rs`,
`plans.rs`, `system.rs`, `assignments/`. Routes mounted under
`/admin/permissions/...`.

- `web/admin/permissions/validation.rs` references
  `crate::auth::unified_permission_service::PermissionSource::Plan`
  (1 import).
- `web/admin/wallet_management_handlers.rs:19` uses
  `crate::auth::unified_permission_service::UnifiedPermissionService`
  for permission checks.

### 5.5 `web/user/` — user self-service

- `permissions.rs` — single file, uses
  `crate::auth::granular_permissions::*` (L10). User self-service
  read path (not admin proxy).
- `unified_user_handlers.rs` — multi-purpose user endpoints (not
  auth-specific).
- `chat_handlers.rs:539` calls `validate_access_token` (chat uses JWT
  validation).
- `chat_upload_handlers.rs`, `developer_portal.rs`, `watchlist_handlers.rs`
  — not auth-specific.

### 5.6 Other auth-touching web files (full list)

- `web/analytics/eps/rankings.rs:14` — `UnifiedPermissionService` (analytics feature gating).
- `web/analytics/eps/cache.rs:14` — `UnifiedPermissionService` (analytics cache).
- `web/admin/wallet_management_handlers.rs:19` — `UnifiedPermissionService` (admin wallet CRUD).
- `web/notifications/sse_handlers.rs:110` — `validate_access_token` (SSE auth).
- `web/admin/chat_handlers.rs:415` — `validate_access_token`.
- `web/user/chat_handlers.rs:539` — `validate_access_token`.
- `web/routes/unified_router.rs:2` — `UnifiedPermissionService::new_without_cache` (route construction).

### 5.7 Cross-domain coupling — the blast radius

**14 non-auth files** import `crate::auth` or `crate::domain::auth` or
`crate::core::permissions` (counted by `rg -l`, includes intra-module
imports). Listing by file:

| File | Imports | Purpose |
|---|---|---|
| `infrastructure/container/simple_container.rs` | `UnifiedWeb3AuthService`, `OpenIDTokenService`, `KeyManager`, `UnifiedPermissionService` (L32–35) | DI container |
| `infrastructure/container/stateless_service_factory.rs` | Same 4 (L21–24) | DI factory |
| `infrastructure/cache/unified_permission_cache.rs` | `PermissionDetail` (L18) | Redis permission cache |
| `infrastructure/adapters/services/security_monitoring_service_adapter.rs` | `domain::auth` (1) | Security event adapter |
| `application/admin/commands/assign_admin_plan.rs` | `domain::auth::ports::IdentityProviderPort` (L5) | Admin claims |
| `web/auth/app_state.rs` | `domain::auth` (1) | App state |
| `web/analytics/eps/rankings.rs` | `UnifiedPermissionService` (L14) | Analytics |
| `web/analytics/eps/cache.rs` | `UnifiedPermissionService` (L14) | Analytics |
| `web/user/permissions.rs` | `granular_permissions::*` (L10) | User self-service |
| `web/admin/wallet_management_handlers.rs` | `UnifiedPermissionService` (L19) | Admin wallet |
| `web/admin/permissions/validation.rs` | `unified_permission_service::PermissionSource` (1) | Admin perms |
| `web/middleware/auth_middleware.rs` | `UnifiedWeb3AuthService` (L22) | Auth gate |
| `web/routes/unified_router.rs` | `UnifiedPermissionService::new_without_cache` (2 sites) | Router |
| `auth/token_service.rs` | `auth_service::Web3VerificationRequest`, `key_manager::KeyManager` (L18–19) | intra-module |

**14 files to touch for a full extraction. 5 of these are in the
hot-path (middleware + chat + SSE + router).** Of the 14, only 3 are
on the *issuance* path (`simple_container`, `stateless_service_factory`,
`security_monitoring_service_adapter`); the other 11 are on the
*validation* path and can be served by a local `epsx-identity` shared
crate (Shape B from §1) without an HTTP round trip.

---

## 6. Infrastructure / Security

`apps/backend/src/infrastructure/security/` — 4 files, ~750 lines.

| File | LoC | Role |
|---|---|---|
| `mod.rs` | 9 | Re-exports |
| `key_management.rs` | 187 | `JwtKeyManager` with global `OnceLock` + file-based RSA — **DEAD CODE** |
| `threat_detection.rs` | 504 | `ThreatDetectionService`, `SecurityEvent`, `SecurityContext`, `get_threat_detection_service` (global `Arc`) |
| `chat_filter.rs` | 51 | `sanitize_chat_content(content: &str) -> String` |

### 6.1 `key_management.rs` is dead code, but a split risk

`JwtKeyManager` (L33) is a **competing key manager** to the one in
`auth/key_manager.rs`. It uses a global `static KEY_MANAGER:
OnceLock<JwtKeyManager>` (L84), loads RSA from a `keys/` directory
via `generate_or_load()` (L40–80), and panics on init failure
(`.unwrap()`). The exposed function `get_key_manager() -> Result<&'static
JwtKeyManager, KeyError>` is **not used by any caller in the codebase**
(verified by `rg` — only definition sites and a test).

**Risk for the split:** if the new identity service includes this
module "because it's auth-related," it will pull in a second, stale
key-management path that loads from disk and panics on failure —
distinct from the env-driven `auth::key_manager::KeyManager` that the
real token service uses. **Drop this file before the split** (or
re-purpose it for the security-related key-rotation features that
the audit can't see).

### 6.2 `threat_detection.rs` is auth-adjacent but not in scope

504 L. Defines `SecurityEvent`, `ThreatLevel`, `SecurityContext`,
`SecurityError`, `ThreatDetectionService`, `SecuritySummary`, plus
`get_threat_detection_service() -> Arc<ThreatDetectionService>` (a
global `Arc`, not a `OnceLock`) and
`initialize_global_threat_detection(cache: Arc<dyn Cache>)`.

`web/middleware/governor_limiter.rs` is the only caller (1 site:
`crate::infrastructure::security::get_threat_detection_service()`).
This is rate-limit-adjacent, not identity-adjacent; the audit marks
it **out of scope** for the auth+identity split. Confirm with the
integration agent.

### 6.3 `chat_filter.rs` is content safety, not identity

`pub fn sanitize_chat_content(content: &str) -> String` (one function,
51 L). Called from `web/user/chat_handlers.rs` (3 sites) and
`web/admin/chat_handlers.rs` (1 site). **Not identity-related**; out
of scope for this split.

---

## 7. Migrations / schema

`apps/backend/migrations/`: **4 separate Postgres databases** (not 4
schemas in one DB), each with its own Diesel config:

- `migrations/core/` — auth, plans, permissions, sessions, read model
  → DB `epsx_prod`
- `migrations/analytics/` — audit logs, event store → DB `epsx_analytics_prod`
- `migrations/notifications/` — wallet_notifications, subscriptions → DB `epsx_notifications_prod`
- `migrations/payments/` — payments, subscriptions, wallet_credits → DB `epsx_payments_prod`

The only non-`public` schema is `read_model` in core, used only by
`projection_checkpoints`. The split is per-database, not per-schema.

### 7.1 Auth/identity tables — all in `core` DB

| Table | File (line) | FK to wallet_users? | Identity-service owner? |
|---|---|---|---|
| `wallet_users` | `core/00000000000001_consolidated_baseline_v6/up.sql:38–57` | (is the user table) | **YES — moves to identity** |
| `web3_auth_nonces` | `core/00000000000001_consolidated_baseline_v6/up.sql:224–234` | no (PK = nonce after 2026-04-27 migration) | **YES — moves to identity** |
| `openid_refresh_tokens` | `core/00000000000001_consolidated_baseline_v6/up.sql:239–249` | **YES, CASCADE** (v6:538) | **YES — moves to identity** |
| `permissions` (catalog) | `core/00000000000001_consolidated_baseline_v6/up.sql:69–91` | no | Stays core (shared with feature gating) |
| `plans` (catalog) | `core/00000000000001_consolidated_baseline_v6/up.sql:101–146` | no | Stays core (commerce/catalog) |
| `plan_permissions` | `core/00000000000001_consolidated_baseline_v6/up.sql:158–166` | to `plans`, `permissions` | Stays core |
| `wallet_plan_assignments` | `core/00000000000001_consolidated_baseline_v6/up.sql:173–190` | **YES, CASCADE** (v6:526) | Split — see §7.3 |
| `wallet_direct_permissions` | `core/00000000000001_consolidated_baseline_v6/up.sql:200–210` | **YES, CASCADE** (v6:532) | Split — see §7.3 |
| `api_keys` | `core/00000000000001_consolidated_baseline_v6/up.sql:330–353` | **YES, RESTRICT** (v6:541) | Stays core (developer portal) |
| `api_modules`, `api_key_module_access`, `api_key_permissions` | `core/00000000000001_consolidated_baseline_v6/up.sql:360–404` | internal | Stays core |
| `route_permissions` | `core/00000000000001_consolidated_baseline_v6/up.sql:259–288` | no | Stays core (auth config) |
| `system_settings` | `core/00000000000001_consolidated_baseline_v6/up.sql:313–324` | no | Stays core |
| `read_model.projection_checkpoints` | `core/00000000000001_consolidated_baseline_v6/up.sql:295–307` | no | Stays core |
| `user_watchlist` | `core/00000000000001_consolidated_baseline_v6/up.sql:415–425` | **YES, inline** (L417) | Stays core |
| `chat_conversations`, `chat_messages`, `chat_topics` | `core/00000000000001_consolidated_baseline_v6/up.sql:431–473` | no (stringly-typed) | Stays core |
| `news_articles` | `core/00000000000001_consolidated_baseline_v6/up.sql:479–499` | no | Stays core |
| `mv_web3_chain_distribution` | `core/00000000000001_consolidated_baseline_v6/up.sql:505–514` | (mat view over `wallet_users`) | Re-evaluate — see §7.4 |

### 7.2 No cross-database FKs — biggest architectural win

I ran a `REFERENCES` search across all migrations. There are **zero
FKs that point across DB boundaries** (e.g. no `analytics.… REFERENCES
core.wallet_users`). The only declared FKs to `wallet_users` are all
in the same DB (core), so the split does not require any DDL
cross-DB refactor. The system already enforces identity consistency
in app code (stringly-typed everywhere outside core); the split just
formalizes that boundary.

### 7.3 User-related tables OUTSIDE the auth domain

All carry `wallet_address VARCHAR(42)` as plain string, no FK:

- **notifications DB:** `wallet_notifications`, `notification_subscriptions`
- **analytics DB:** `unified_audit_log`, `permission_audit_log`,
  `payment_audit_log`, `assignment_audit_log`, `wallet_activity_logs`,
  `audit_logs`, `analytics_events`, `event_store`, `outbox_events`,
  `aggregate_snapshots`, `api_key_usage_logs`
- **payments DB:** `payments`, `subscriptions`,
  `stock_ranking_assignments`, `payment_contexts`, `wallet_credits`
  (**PK = wallet_address**), `credit_transactions`, `payment_audit_log`

### 7.4 Split-ownership recommendation

**Identity service owns (3 tables, all in core DB):**

1. `wallet_users` — the canonical user table. PK is `VARCHAR(42)`
   (the EVM address itself), not a UUID. Friendly to a split: globally
   unique, self-describing.
2. `web3_auth_nonces` — SIWE challenge state. PK was changed in
   `20260427000000_allow_multiple_web3_nonces/up.sql` to `nonce` to
   allow multiple outstanding challenges per wallet.
3. `openid_refresh_tokens` — token-management "session". FK to
   `wallet_users` CASCADE.

**Core keeps, with `REFERENCES wallet_users` → logical refactor:**

- `wallet_plan_assignments` — plan commerce is a Core/payments
  concern. The current CASCADE on the FK is too tight; when Identity
  is its own service, Core will need a tombstone/compensation flow
  via the `user.deleted` event from Identity.
- `wallet_direct_permissions` — per-wallet grants. Same pattern.
- `api_keys` (RESTRICT) — developer-portal artifact. Same pattern.
- `user_watchlist` — only one with an inline FK. Same pattern.

**Stays core unchanged (no user FK):** `permissions` (catalog), `plans`
(catalog), `plan_permissions`, `api_modules`, `api_key_module_access`,
`api_key_permissions`, `route_permissions`, `system_settings`,
`read_model.projection_checkpoints`, `chat_*`, `news_articles`.

**Stays in its current DB:** everything in analytics, notifications,
payments DBs (stringly-typed already, no FK needed).

**Contested decision — `wallet_credits` (PK = wallet_address) in
payments DB.** Conceptually a commerce balance, but the PK being a
wallet address makes it feel like identity. **Recommend: keep in
payments; on `user.deleted`, identity event zeros the balance or
marks tombstone.**

### 7.5 Other findings

- **`mv_web3_chain_distribution` reads from `wallet_users`.** If user
  table moves to Identity, this materialized view breaks and must
  either move with it or be rewritten as a periodic sync into Core.
  Worth flagging.
- `wallet_users.current_plan_id` is `INTEGER`, but `plans.id` is
  `UUID`. Mismatch — `current_plan_id` cannot FK to `plans.id` and
  is effectively dead in core. Should be dropped or fixed during the
  split.
- `consolidated_*_v2` files in `notifications/` and `payments/` are
  byte-identical duplicates of `consolidated_baseline_*`. Recommend
  removal to avoid migration run-order confusion.
- `web3_auth_nonces` PK migration (`20260427000000_…`) is the most
  recent core change (2026-04-27) and is not in the consolidated
  baseline v6. Confirm the live DB has run it.
- `permissions.created_by` is `VARCHAR(42)` — unauthenticated label,
  not a real user reference.

---

## 8. Risks / open questions / top refactors

### 8.1 Risks (must address before or during the split)

1. **Two parallel "admin" mechanisms do not coordinate** (issue called
   out in §3.4). The split must reconcile `application/admin/commands/assign_admin_plan.rs`
   (identity-provider custom claims) and
   `application/permission_management/commands/handlers/assign_wallet_handler.rs`
   (`PlanAssignment` row) into a single path. The simplest fix is to
   make `assign_admin_plan` create a `PermissionPlan` with
   `is_system=true` + `plan_category=System` and assign via the
   existing `assign_wallet` handler.

2. **4 of 7 domain events are orphaned** (issue called out in §2.2).
   `PlanDeletedEvent`, `WalletAssignedToPlanEvent`,
   `WalletRemovedFromPlanEvent`, `PolicyUpdatedEvent` are defined but
   never published. If any consumer relies on the missing events
   (analytics, notifications, audit), the split exposes it. Wire up
   the 3 unused `_event_bus` fields in the command handlers
   (see §3.2) before the split.

3. **Two parallel permission models propagate at different speeds**
   (issue called out in §4.5). JWT-embedded permissions go stale up
   to 1 hour after a grant. The identity service should emit a
   `permission.granted` event that the business services can use to
   short-circuit token rotation OR to publish a revocation list.

4. **`Plan` is a god-aggregate** (issue called out in §2.2). 21
   fields mixing pricing/catalog/taxonomy/access-control. The split
   is forced to either (a) drop pricing fields from the IAM-side
   `Plan`, or (b) accept the IAM service is also the catalog of
   record. Pick (a) — it is cleaner.

5. **Two dead-code modules to clean up before the split**:
   - `apps/backend/src/auth/cache.rs` (452 L, `SimplifiedAuthCache`,
     no caller).
   - `apps/backend/src/infrastructure/security/key_management.rs`
     (187 L, global `OnceLock<JwtKeyManager>`, no caller, panics on
     init).

6. **`granular_permissions.rs` uses non-cryptographic `DefaultHasher`**
   for the `hash` field (L333). If the hash is used for
   cross-process cache invalidation after the split, this is a latent
   bug. Use `Sha256` instead.

7. **Cross-layer leak:** `application/permission_management/queries/handlers/list_plans_handler.rs:7`
   imports `crate::web::pagination::Pagination`. Application must not
   depend on web. Move to `application/shared/` or inline.

8. **`web3_auth_nonces` PK migration may not be live.** The
   `20260427000000_allow_multiple_web3_nonces` migration is the most
   recent core change and is not in the consolidated baseline v6.
   Confirm the live DB has run it; otherwise the SIWE flow enforces
   "one outstanding nonce per wallet" (a bug).

9. **`wallet_users.current_plan_id` is dead** (INTEGER vs plans.id
   UUID). Drop or fix during the split.

10. **The 2 byte-identical migration files** in `notifications/` and
    `payments/` (`consolidated_*_v2` vs `consolidated_baseline_*`)
    cause migration run-order confusion. Remove the duplicates.

### 8.2 Open questions for the integration agent

- Should the `Plan` aggregate's pricing fields be removed first (a
  pre-split refactor) or as part of the split? Recommend: pre-split.
- Should `assign_admin_plan` be merged into the standard
  `assign_wallet` flow with an `is_system` plan, or kept as a
  separate path that calls the identity provider port? Recommend:
  merge.
- The `policy_management` Policy aggregate is currently
  `#[allow(dead_code)]`. Is it planned to be wired up, or is it
  dead and should be removed? Affects the split surface.
- `web/admin/auth_handlers/wallet_handlers.rs` admin wallet endpoints
  — are these for admin-side wallet CRUD (e.g. disable a wallet) or
  for admin-on-behalf-of-user auth? Affects whether they move to the
  identity service.

### 8.3 Top refactors (highest-value, lowest-risk)

1. **Remove `auth/cache.rs` and `infrastructure/security/key_management.rs`.**
   Both are dead code (verified by `rg`). Cleanup: -639 lines.
2. **Wire up the 3 unused `_event_bus` fields** in
   `delete_plan_handler.rs`, `assign_wallet_handler.rs`,
   `remove_wallet_handler.rs`. (Pre-split bug fix.)
3. **Fix the `granular_permissions.rs` `hash` field** to use a
   cryptographic hash (Sha256) for cross-process safety.
4. **Drop `wallet_users.current_plan_id`** (dead column) and the
   duplicate `consolidated_*_v2` migration files.
5. **Reconcile the two admin paths** (see §8.1.1).
6. **Move `application/permission_management/queries/handlers/list_plans_handler.rs:7`**
   `crate::web::pagination::Pagination` import to a shared location.

### 8.4 Split-readiness score (preliminary)

| Dimension | Score (1-5) | Rationale |
|---|---|---|
| Domain layer purity | **4** | `domain/auth/` is a single trait; `domain/permission_management/` has loose ends (god-aggregate, orphan events) |
| Application layer purity | **3** | 3 `_event_bus` fields unused; cross-layer `web::pagination` import; two parallel admin paths |
| Top-level auth module cohesion | **3** | `cache.rs` dead, SIWE verify duplicated, 919-L `unified_permission_service.rs` |
| Web layer separation | **4** | Bearer middleware is clean; admin and user sides are clearly partitioned |
| Schema isolation | **5** | No cross-DB FKs; `wallet_users` PK is wallet address (self-describing) |
| Hot-path safety (validation) | **5** | `validate_access_token` is self-contained (in-memory RSA); middleware calls `core::permissions` directly |
| Issuance coupling | **2** | 1-hour propagation lag between grant and JWT-embedded permission; no revocation list |
| Security module hygiene | **2** | Two key managers, one dead; threat-detection is a global `Arc` not DI-owned |
| **Overall** | **3.4 / 5** | **Extractable today, with §8.1 fixes; prefer Shape B (shared crate) before Shape A (network split).** |

---

## Appendix A — File inventory with line counts

### `apps/backend/src/auth/` (9 files, 3,678 L)

- `mod.rs` 79
- `auth_service.rs` 526
- `token_service.rs` 765
- `cache.rs` 452
- `granular_permissions.rs` 489
- `key_manager.rs` 306
- `verification_service.rs` 409
- `challenge_service.rs` 162
- `unified_permission_service.rs` **919** (largest)

### `apps/backend/src/domain/auth/` (2 files, 23 L)

- `mod.rs` 1
- `ports.rs` 22

### `apps/backend/src/domain/permission_management/` (~20 files, ~1,500 L)

- `mod.rs` 62
- `aggregates/plan.rs` **402** (21-field god-aggregate)
- `aggregates/policy.rs` 124
- `aggregates/mod.rs` small
- `entities/plan_assignment.rs` 73
- `entities/mod.rs` small
- `value_objects/mod.rs` 10
- `value_objects/permission_string.rs` 54
- `value_objects/plan_slug.rs` 40
- `value_objects/plan_category.rs` 57
- `value_objects/plan_group.rs` 42
- `value_objects/policy_id.rs` 58
- `value_objects/policy_rule.rs` 36
- `domain_services/permission_validation_service.rs` 43
- `domain_services/plan_assignment_service.rs` 47
- `domain_services/mod.rs` small
- `repository_ports/plan_repository_port.rs` 58
- `repository_ports/plan_assignment_repository_port.rs` 29
- `repository_ports/policy_repository_port.rs` 18
- `repository_ports/mod.rs` small
- `events/plan_events.rs` ~190
- `events/policy_events.rs` ~75
- `events/mod.rs` small

### `apps/backend/src/core/permissions.rs` (217 L)

5 `pub fn`: `has_permission` (L7), `is_admin` (L39),
`has_any_permission` (L46), `permission_platform` (L51),
`has_admin_platform_permission` (L57). Pure CPU, no I/O, no DB.

### `apps/backend/src/application/admin/` (4 files, ~82 L)

- `mod.rs` 1, `commands/mod.rs` 1, `commands/assign_admin_plan.rs` 80

### `apps/backend/src/application/permission_management/` (22 files, ~835 L)

- `mod.rs` 50
- `commands/mod.rs` 7, `commands/models/mod.rs` 13,
  `commands/handlers/mod.rs` 13
- 5 command models + 5 command handlers (~537 L total)
- 4 query models + 4 query handlers (~329 L total)
- `controllers/mod.rs` 6 (stub), `dtos/mod.rs` 5 (stub),
  `ports/inbound/mod.rs` 6 (stub)

### `apps/backend/src/infrastructure/security/` (4 files, ~750 L)

- `mod.rs` 9
- `key_management.rs` 187 (DEAD CODE)
- `threat_detection.rs` 504
- `chat_filter.rs` 51

### Migrations (4 DBs)

- `migrations/core/` — 4 baselines + 13 incremental (3 tables for
  identity service: `wallet_users`, `web3_auth_nonces`,
  `openid_refresh_tokens`)
- `migrations/analytics/` — 2 baselines + 1 incremental
- `migrations/notifications/` — 2 baselines (one duplicate)
- `migrations/payments/` — 2 baselines + 4 incremental (one duplicate)

## Appendix B — Coupling summary

**14 non-auth files** import `crate::auth` / `crate::domain::auth` /
`crate::core::permissions`:

```
infrastructure/container/simple_container.rs       (4 imports)
infrastructure/container/stateless_service_factory.rs (4 imports)
infrastructure/cache/unified_permission_cache.rs   (1)
infrastructure/adapters/services/security_monitoring_service_adapter.rs (1)
application/admin/commands/assign_admin_plan.rs    (1)
web/auth/app_state.rs                              (1)
web/analytics/eps/rankings.rs                      (1)
web/analytics/eps/cache.rs                         (1)
web/user/permissions.rs                            (1)
web/admin/wallet_management_handlers.rs            (1)
web/admin/permissions/validation.rs                (1)
web/middleware/auth_middleware.rs                  (1)
web/routes/unified_router.rs                       (2)
auth/token_service.rs                              (2, intra-module)
```

Of these 14, only 5 sit on the issuance path
(`simple_container`, `stateless_service_factory`,
`security_monitoring_service_adapter`, `application/admin/...`, and
`auth/token_service.rs` itself). The other 9 are on the validation
path and can be served by a local shared crate.
