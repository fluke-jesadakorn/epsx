# Wave 7 — Admin BFF permissions wiring

**Branch:** `wave7/admin-permissions` (off `migration/dioxus-microservices` @ `59a494ec`)
**Track:** single-track, integration-gate-style. The fix is small and localized — no need for multi-track split.
**Scope:** populate `UiUser.permissions` from JWT roles in the admin BFF (and frontend BFF) so `AdminAuthGate` stops misfiring for admin users.

## Root cause (confirmed by reading the code)

- `apps/admin/src/ssr.rs:44` hard-codes `permissions: vec![]` when building the `UiUser` from the JWT-decoded `AuthUser`.
- `shared/rust/dioxus_ui/src/auth/user.rs:63` `User::has_permission` does **exact-string** comparison only — no wildcards.
- `shared/rust/dioxus_ui/src/auth/auth_gate.rs:163-170` `AdminAuthGate` does the right check: if `is_admin()`, run the same `has_permission` filter as `AuthGate`. With empty perms, every required permission looks "missing" → gate fires → page body is replaced with the gate panel → body markers (`analytics-chart`, `policy-filters`, `media-uploader`, `payment-link-stats`, `wallet-stats-bar`) are missing → wave6b smoke shows 5 PARTIAL routes.

The wave6c deliverable (`deliverable.md` §"Wave 6C follow-up notes" item 3) flagged this exactly:
> *Wire the admin BFF's `permissions` field properly. Wave 3a sets `permissions: vec![]` for admin users in `ssr.rs:42`. The fix is to either (a) decode the user's role-based permissions from the JWT and plumb them into the `UiUser`, or (b) make `AdminAuthGate` skip the permission check when the user is `is_admin()`.*

## Chosen fix: (a) with a role→permission map

Option (a) wins because:

- It matches the existing `AuthUser::is_admin` / `is_editor` / `is_merchant` predicates in `shared/rust/auth/src/lib.rs` — the JWT already carries roles; the BFF is the natural place to expand them.
- (b) would require touching `AdminAuthGate` in `shared/rust/dioxus_ui` and would change semantics for every call site (a behavioral ripple we don't need).
- Option (a) keeps the gate component unchanged and gives non-admin roles a useful perm set too (editors see content pages, merchants see payments).

**Role → permission mapping (admin BFF):**

| Role(s) | Permissions granted |
|---|---|
| `admin`, `super_admin` | every admin-side perm checked by `AdminAuthGate` across the 16 admin routes |
| `editor`, `content_manager` | content-moderation perms (`news:manage`, `audit:read`, `notifications:manage`, `policies:manage`) |
| `merchant` | financial perms (`payments:read`, `payments:manage`, `wallets:manage`) |
| (none / unknown) | empty |

`is_admin` / `is_editor` / `is_merchant` predicates are reused verbatim — same role strings the JWT and `auth::AuthUser` already use. No new auth grammar.

## Files touched

| File | Change |
|---|---|
| `apps/admin/src/auth.rs` | add `permissions_for_roles(roles: &[String]) -> Vec<String>` helper + 6 unit tests |
| `apps/admin/src/ssr.rs` | replace `permissions: vec![]` with `permissions: auth::permissions_for_roles(&u.roles)` |
| `apps/frontend/src/ssr.rs` | same swap (symmetric, same module pattern) |
| `WAVE7_DELIVERABLE.md` | integration-gate record (mirror wave6b's `deliverable.md` shape) |

No contract / proto / DB / K8s changes. No `apps/backend/` changes (CLAUDE.md: Permissions & Plan Logic is backend-only — the admin BFF derives from the JWT roles the backend already mints, this is purely presentation-layer).

## Done criteria

1. `cargo check --workspace` clean.
2. `cargo test --workspace --lib` green; new `permissions_for_roles` tests pass.
3. Re-run wave6b's admin BFF smoke on the 5 PARTIAL routes — all 5 flip to PASS.
4. `WAVE7_DELIVERABLE.md` records the before/after marker counts.
5. Single commit on `wave7/admin-permissions`. No force-push. Branch pushed to origin.

## Out of scope (explicitly)

- The wildcard-aware `has_permission` upgrade (`platform:resource:action` with `*:*` matching) — that's a separate, larger change. The 2-segment exact match is what the gates actually use today.
- A real `permissions` column on `users` (DB-level). The JWT role set is the source of truth per the existing backend `epsx_auth` setup.
- Fixing the `apps/frontend` BFF's *separate* `permissions: vec![]` in the same patch — actually that IS in scope for symmetry, see "Files touched" above.
- Removing the `apps-old/` reference directory. Untouched.
