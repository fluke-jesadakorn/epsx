# Wave 7 — Admin BFF Permissions Wiring — Integration Gate Deliverable

> **Status:** ✅ All 4 PARTIAL admin routes flipped to PASS. 1 PARTIAL remains — pre-existing smoke-script typo (`payment-link-stats` not in source; `payments-stats` is). Full cargo gate green. Branch `wave7/admin-permissions` ready to push to `migration/dioxus-microservices`.

## What landed

The `AdminAuthGate` misfire in wave6b (5 PARTIAL admin routes) was caused by `apps/admin/src/ssr.rs:44` hard-coding `permissions: vec![]` when building the SSR `UiUser`. The shared `User::has_permission` does **exact-string match only** (no wildcards), so every required permission looked "missing" and the gate fired — page body was replaced with the gate panel, body markers disappeared.

### Fix (chosen: deliverable.md option (a) — role-based perm expansion in the BFF)

| File | Change |
|---|---|
| `apps/admin/src/auth.rs` | added `permissions_for_roles(roles: &[String]) -> Vec<String>` (8 new unit tests) |
| `apps/admin/src/ssr.rs` | replaced `permissions: vec![]` with `permissions: auth::permissions_for_roles(&u.roles)` (1-line swap, plus doc comment) |
| `apps/frontend/src/auth.rs` | mirrored the same helper for the user-side 2-segment perm grammar (7 new unit tests) |
| `apps/frontend/src/ssr.rs` | mirrored the same swap |
| `apps/admin/src/ssr.rs` | (incidental) pre-existing test-helper `build_ctx` was missing the new `wallet` field on `PageContext` — populated it with `ConnectedWalletState::default()` (matches production path line 66-67) |

### Role → permission mapping

| Role | Permissions granted |
|---|---|
| `admin`, `super_admin` (admin BFF also accepts `Admin`) | full admin set — every 2-segment perm checked by `AdminAuthGate` across the 16 admin routes |
| `editor`, `content_manager` | content-moderation perms (`audit:read`, `news:manage`, `notifications:manage`, `policies:manage`, `media:manage`) |
| `merchant` | financial perms (`payments:read`, `payments:manage`, `wallets:manage`) |
| (none / unknown) | empty |

`is_admin_role` / `is_editor_role` / `is_merchant_role` reuse the **same role strings the JWT already carries** (no new auth grammar). Output is `BTreeSet`-sorted, so SSR cache keys and snapshot tests stay stable.

## Cargo gate

| Step | Result | Time |
|---|---|---|
| `cargo check --workspace` | ✅ Clean (only the 15 pre-existing wave6b dead-code warnings on `epsx-frontend` / `epsx-admin`) | 59.32s |
| `cargo test --workspace --lib` | ✅ 181 passed in `epsx-dioxus-ui` (unchanged baseline) + 8 passed in `epsx-kernel` + 17 passed in `epsx-templates` + 3 passed in `epsx-renderer` — **0 failed across the workspace** | <60s |
| `cargo test --workspace --bins` | ✅ `bff-admin` 10/10 (8 new `permissions_for_roles` tests + 2 pre-existing ssr tests), `bff-frontend` 8/8 (7 new + 1 pre-existing) | <120s |

**New test count: 15 (8 admin + 7 frontend).** The 181-test `epsx-dioxus-ui` baseline is **unchanged** — the gate behavior didn't regress.

## Admin BFF smoke (BFF running on :13001, JWT minted with `admin` role)

| Route | wave6b status | wave7 status | Markers found |
|---|---|---|---|
| `/admin/dashboard` | PASS | **PASS** | 5/5 |
| `/admin/analytics` | PARTIAL | **PASS** | 7/7 |
| `/admin/policies` | PARTIAL | **PASS** | 6/6 |
| `/admin/settings` | PASS | **PASS** | 5/5 |
| `/admin/media` | PARTIAL | **PASS** | 4/4 |
| `/admin/audit-log` | PASS | **PASS** | 5/5 |
| `/admin/news` | PASS | **PASS** | 6/6 |
| `/admin/notifications` | PASS | **PASS** | 7/7 |
| `/admin/payments` | PARTIAL | **PARTIAL** | 5/6 (missing `payment-link-stats`) |
| `/admin/wallet-management/credits` | PASS | **PASS** | 5/5 |
| `/admin/wallet-management/access/plans` | PASS | **PASS** | 6/6 |
| `/admin/wallet-management/access` | PASS | **PASS** | 4/4 |
| `/admin/wallet-management/wallets` | PARTIAL | **PASS** | 8/8 |
| `/admin/chat` | PASS | **PASS** | 5/5 |
| `/admin/developer-portal` | PASS | **PASS** | 7/7 |
| `/admin/auth` | PASS | **PASS** | 2/2 |

**Summary: 15 PASS, 1 PARTIAL, 0 FAIL** — out of 16 admin routes.

The 1 remaining PARTIAL (`/admin/payments` missing `payment-link-stats`) is a **pre-existing smoke-script bug**, not a wave7 issue:

- The wave6b smoke script expects the body marker `payment-link-stats`.
- The actual source (`shared/rust/dioxus_ui/src/pages/admin_pages/payments.rs:112`) uses `payments-stats` (plural, no `-link-`).
- Before wave7, the gate was intercepting the body anyway, so the typo was masked. Now that the gate passes, the body renders fully — and contains `payments-stats` (verified by grep), `payment-links-list`, `create-link-form`, `link-revoke-confirm`, `access-management-list`, `payments-filter-panel` — just not `payment-link-stats`.
- This is a **smoke-script fix** in the wave6b deliverable, not a wave7 deliverable item. Tracking note added to "Out of scope / next wave" below.

## UI-layer concern note (CLAUDE.md compliance)

CLAUDE.md states: *"Permissions & Plan Logic — Backend Only. All business logic related to permissions, plan access, ranking offsets, feature flags, and subscription rules must be implemented in the Rust backend only. Frontend (`apps/frontend`) and admin-frontend (`apps/admin-frontend`) are UI-only layers."*

The `permissions_for_roles` helper added in this wave is **UI-layer presentation only**. The canonical RBAC grammar (`platform:resource:action` with wildcards) remains in `apps/backend/src/core/permissions.rs`, which is the system of record. The admin BFF's role→2-segment-perm expansion is a translation table that lets `AdminAuthGate`'s exact-string `has_permission` consume the role list the backend mints into the JWT. Real authorization (which routes a user can call on the API) is still enforced in the backend. The admin BFF is not "deciding" who can do what — it's deciding which gate panels to render in the SSR HTML.

This pattern is consistent with the rest of the codebase:
- `epsx_auth::AuthUser::is_admin` / `is_editor` / `is_merchant` are role-derived predicates that the BFFs use to decide UI flow.
- `epsx_dioxus_ui::auth::User::has_permission` is a presentation-layer check used by the gate components.
- Neither bypasses backend enforcement.

## Out of scope / next wave

- **Smoke-script typo:** `/admin/payments` body expects `payment-link-stats` (not in source). Either fix the smoke script to expect `payments-stats` (correct), or rename the source class. Wave 6B-era deliverable bug.
- **Wildcard-aware `has_permission` upgrade:** `User::has_permission` does exact-string match only. A future wave could upgrade it to support `*:*`, `platform:*:*`, `platform:resource:*` wildcards (the same grammar the backend already uses). That would let the BFF emit a single `admin:*:*` perm for super-admins instead of enumerating 16+ admin perms. Bigger refactor — out of scope for this wave.
- **DB-level permissions column:** The JWT role set is currently the source of truth. A real per-user permissions table in the identity DB would let admins grant fine-grained perms without changing roles. CLAUDE.md says backend-only; wave8+ candidate.
- **The 15 pre-existing dead-code warnings** on `epsx-frontend` / `epsx-admin` are noted in MIGRATION.md §"Open items carried over" and remain untouched here.

## Branch

`wave7/admin-permissions` (off `migration/dioxus-microservices` @ `59a494ec`).

```
$ git log --oneline wave7/admin-permissions ^migration/dioxus-microservices
<this commit>  wave7(admin-permissions): wire UiUser.permissions from JWT roles
```

## Files changed (final)

```
WAVE7_DESIGN.md                          |  90 +++++++++++++
apps/admin/src/auth.rs                   | 215 ++++++++++++++++++++++++++++++++
apps/admin/src/ssr.rs                    |  22 +++-
apps/frontend/src/auth.rs                | 204 +++++++++++++++++++++++++++++
apps/frontend/src/ssr.rs                 |   9 +-
```

## Verification commands

```bash
# Cargo gate
cargo check --workspace
cargo test --workspace --lib
cargo test --workspace --bins

# Smoke (BFF on :13001, JWT minted with admin role via /tmp mint binary)
JWT=$(EPSX_JWT_SECRET=epsx-dev-secret-do-not-use-in-prod /path/to/mint)
curl -sS -H "Cookie: epsx_token=$JWT" -o /tmp/p.html -w "%{http_code}\n" \
  http://localhost:13001/admin/analytics
grep -c "analytics-chart" /tmp/p.html  # expect >=1
```

## Push strategy

Single commit on `wave7/admin-permissions`. Branch pushed to origin. **Not** auto-merged to `migration/dioxus-microservices` — that's your call. No force-push. No rebase.
