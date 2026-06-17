# Wave 6C — Live-Rendering Fix + 1:1 Component Parity (admin + user)

## Why this wave exists

Wave 6B's deliverable flagged **5 PARTIAL admin smoke routes**
(analytics, policies, media, payments, wallet-management/wallets) — the
HTTP 200 is right, but `AdminAuthGate` intercepts the page body
because the admin BFF plumbs `permissions: vec![]` into the rendered
`User` (`apps/admin/src/ssr.rs:44` and `apps/frontend/src/ssr.rs:45`).
The 17 `*_section_markers` unit tests for those pages pass because the
tests' `admin_ctx()` fixture bakes the right `permissions` in by hand
— only the BFF runtime path is broken.

Wave 6C does three things:

1. **Fix the live-rendering gap** — populate `permissions` in the BFFs
   from the user's roles using a new
   `default_permissions_for_role(role) -> Vec<String>` helper, and
   have `AdminAuthGate` short-circuit the missing-permission check
   when `user.is_admin()`. Both layers, defense in depth.
2. **1:1 component parity for the admin side** — break the
   inlined JSX in Wave 6B's admin pages into individually-named
   sub-components that mirror the Next.js source's sub-component
   tree. **Plus** 1:1 parity for the user side, extracting named
   sub-components in the 16 user pages from Wave 6A.
3. **Move the Wave 7 design doc to a separate plan** (per scope
   decision: deferred to its own "wave7-design" plan launched
   after Wave 6C lands).

gRPC remains **explicitly out of scope** unless a non-Rust client
appears.

## What "live-rendering fix complete" means

- 16/16 admin routes return HTTP 200 with **all** design-doc section
  markers in the response body (not just 11/16). The 5 PARTIAL rows
  in the Wave 6B deliverable become PASS:
  - `/admin/analytics` → `analytics-chart` marker present
  - `/admin/policies` → `policy-filters`, `policy-list` present
  - `/admin/media` → `media-uploader` present
  - `/admin/payments` → `payments-stats` present
  - `/admin/wallet-management/wallets` → `wallet-stats-bar` present
- The 17 `*_section_markers` unit tests that validated these body
  markers in Wave 6B continue to pass.
- A new unit test: `admin_auth_gate_is_admin_short_circuits_required_permissions`
  — assert that when `user.is_admin() == true` and `permissions` is
  `vec![]`, the `AdminAuthGate` still passes the body through.
- A new unit test: `default_permissions_for_role_admin_returns_wildcard`
  (and 3 more role variants — see Track A).

## What "1:1 component parity" means

The Wave 6B design doc explicitly says it stopped at section-level
parity. Wave 6C goes one level deeper:

1. **Admin side**: every sub-component the Next.js source has at
   `apps-old/admin-frontend/components/{group}/*.tsx` becomes a
   named `#[component]` in the port — same name, same props
   (typed), same children shape. The 20+ admin pages in
   `shared/rust/dioxus_ui/src/pages/admin_pages/` become
   compositions of those sub-components instead of inlined JSX.
   The page file shrinks; the components tree grows.

2. **User side**: same extraction for the 16 user pages from
   Wave 6A. Reuse Wave 6A's existing extracted primitives
   (`MessageBubble`, `ExportDialog`, `PaymentFlowSteps`,
   `PlanComparisonCard`, `ChainVerificationCard`, `UpgradeBanner`,
   `UnifiedPaymentFlow`, `PermissionCategoryBreakdown`,
   `TopMoversCard`). Extract per-page sub-components where the
   source has them.

We don't extract every line into a sub-component — the same
"extract when ≥ 1 typed prop or ≥ 2 uses" rule from Wave 6B
applies. The point is to make the source → port diff a 1:1 walk
through named components, so future regressions are easy to spot.

## The live-rendering fix: 4 layers

| Layer | File | LoC delta | Notes |
| --- | --- | --- | --- |
| 1. Add `default_permissions_for_role(role) -> Vec<String>` helper | `shared/rust/auth/src/lib.rs` | +25 LoC + 5 unit tests | Pure function, no DB. New module: `permissions.rs` inside `shared/rust/auth/src/`, re-exported from `lib.rs`. |
| 2. BFFs use the helper to populate `permissions` | `apps/frontend/src/ssr.rs:38-52`, `apps/admin/src/ssr.rs:37-51` | +6 LoC each (replace `vec![]` with `default_permissions_for_role(roles)`) | Compute permissions from the JWT's `roles` when building the UI `User`. |
| 3. `AdminAuthGate` short-circuits on `is_admin()` | `shared/rust/dioxus_ui/src/auth/auth_gate.rs:163-175` | +1 line (explicit early return) | When `has_admin` is true, skip the missing-permissions check. Currently only short-circuits when `required_permissions.is_none()`. |
| 4. Per-page test fixtures plumb `permissions` consistently | `shared/rust/dioxus_ui/src/pages/admin_pages/*.rs` test fns | 0 LoC (already correct) | All admin pages already have `permissions: vec!["<perm>"]` in their test fixture. Confirm not broken. |

**Design decision (Layer 1)**: where do permissions come from?

- **(a) Role-derived**: hard-code `default_permissions_for_role(role) -> Vec<String>` in the helper. For an `admin` role, return `["*:*:*"]`. Simple, no DB roundtrip, fast. This is what we ship.
- **(b) Service-derived**: query the `wallet_permissions` table via the identity service. Slower (1 extra HTTP roundtrip per SSR), but reflects per-user grant/revoke state from the admin UI. Deferred to a future wave.

**Confirmed role-derived permission table** (per user pick):

```rust
// shared/rust/auth/src/permissions.rs (new file)
pub fn default_permissions_for_role(role: &str) -> Vec<String> {
    match role {
        "admin" | "super_admin" => vec!["*:*:*".into()],
        "editor" | "content_manager" => vec![
            "*:*:*:read".into(),
            "admin:content:write".into(),
        ],
        "merchant" => vec![
            "epsx:payments:*".into(),
            "epsx:wallet:*".into(),
        ],
        "user" | "_" => vec!["epsx:wallet:read".into()],
        _ => vec![],
    }
}

/// Flatten a list of roles into a single per-permission set,
/// preserving the wildcard syntax the backend's `has_permission`
/// already understands.
pub fn permissions_for_roles(roles: &[String]) -> Vec<String> {
    let mut perms: Vec<String> = roles
        .iter()
        .flat_map(|r| default_permissions_for_role(r))
        .collect();
    perms.sort();
    perms.dedup();
    perms
}
```

`permissions_for_roles` is the function the BFFs call. It iterates
all roles a user holds and unions their perms. The wildcard
syntax is the same one the backend's
`apps/backend/src/core/permissions.rs::has_permission` already
handles.

**Defense in depth (Layer 3)**: even if the BFF forgets to populate
permissions, the `AdminAuthGate` still passes the page through
when `user.is_admin() == true`. The current code at
`auth_gate.rs:163-175` enters the `has_admin` block but then runs
the `missing` check at line 167. The change is a one-line
short-circuit:

```rust
if has_admin {
    return rsx! { Fragment { {children} } };  // NEW: unconditional pass for admins
}
// ...existing permission check stays for non-admin users with required_permissions...
```

## The 1:1 component parity scope

For each of the 20+ admin pages, count sub-components in the source
and target the same count in the port. The biggest surfaces:

| Page (in port) | Source sub-components | Sub-components to extract |
| --- | --- | --- |
| `wallet_wallets.rs` | 6,951 LoC across 13 files | `WalletStatsBar`, `WalletList`, `WalletDetailView`, `WalletTableRow`, `WalletCardSections`, `WalletDetailPanel`, `WalletDisableDialog`, `WalletReenableDialog` |
| `developer_portal.rs` | 1,400+ LoC across 7 files | `DeveloperPortalOverview`, `ApiKeysTab`, `ApiKeyCreateForm`, `ApiKeyRevokeModal`, `UsageAnalyticsTab`, `DocumentationTab`, `DeveloperPortalStats` |
| `dashboard.rs` (admin) | 3,858 LoC in `components/admin/` | `AdminStatsCards`, `WalletsByChain`, `RecentTransactions`, `SystemAlerts`, `ActivityStream` |
| `analytics.rs` (admin) | 1,250 LoC in `components/analytics/` | `AnalyticsHeader`, `AnalyticsCardGrid`, `AnalyticsChart`, `AnalyticsTable`, `AnalyticsFilterPanel`, `AnalyticsExportDialog`, `AnalyticsMetadata` |
| `policies.rs` | 1,166 LoC | `PolicyStatsBar`, `PolicyBuilder`, `PolicyMonitor`, `PolicyCard`, `PolicyFilters`, `PolicyList` |
| `wallet_credits.rs` | (shared with payments, 1,239 LoC) | `CreditsLedger`, `CreditsBalanceCards`, `CreditsTransactionList`, `CreditsTopupForm`, `CreditsRevokeDialog` |
| `wallet_plans.rs` | 1,018 LoC | `PlanListSidebar`, `PlanEditorPage`, `PlanEditorDrawer`, `PlanApiLimits`, `PlanPromotions`, `PlanItemCard` |
| `news.rs` | 829 LoC | `NewsManagementList`, `NewsEditor`, `NewsFeaturedCard`, `ArticleCard`, `NewsEmptyState`, `NewsPagination` |
| `payments.rs` | 1,239 LoC | `PaymentLinksList`, `CreateLinkForm`, `LinkRevokeConfirm`, `PaymentLinkStats`, `AccessManagementList`, `PaymentsFilterPanel` |
| `notifications.rs` | 598 LoC | `NotificationList`, `SendForm`, `RecipientsPicker`, `NotificationTemplateEditor`, `NotificationPreview`, `NotificationScheduleDialog`, `NotificationManagementFilters` |
| `media.rs` | 337 LoC | `MediaBrowser`, `MediaUploader`, `MediaFilters`, `MediaStats` |
| `chat.rs` (admin) | 567 LoC | `AdminChatInbox`, `AdminChatConversationView`, `ChatReplyInput`, `ChatInboxSearch`, `ChatUnreadBadge` |
| `audit_log.rs` | (shared audit-log components) | `AuditFilters`, `AuditTimeline`, `AuditEntryDetail`, `AuditSeverityBreakdown`, `AuditExportButton` |
| `settings.rs` | (in `admin/`) | `SettingsDashboard`, `ApiKeysList`, `EmailSettings`, `NotificationSettings`, `SessionManagement` |
| `auth_page.rs` | — | `AuthMethodSelector`, `AuthRedirectHandler` (only 2 sections) |
| `wallet_access.rs` | 791 LoC | `WalletAccessManager`, `PlanSelectorModal`, `AccessGrantForm`, `AccessRevokeDialog` |

**User-side pages** (16 from Wave 6A) — same extraction where the
source has sub-components. The user side has fewer per-page
sub-components; reuse Wave 6A's existing extraction of
`MessageBubble`, `ExportDialog`, `PlanComparisonCard`, etc.

**Total new `#[component]`s**: ~70-80 admin + ~20-30 user = **~90-110
named sub-components**. Most already exist as inline JSX in Wave
6B/6A's pages; the work is the extraction + registration, not new
code. File ownership keeps the diff mostly file-moves (pages
shrink, sub-component files grow).

## Track split (5 parallel coder tracks + integration gate)

Same shape as Wave 6B but **5 tracks** (4 admin-side + 1 user-side).
Sized to fit in **45-min per-track worker sessions** with a 15-min
preempt extension if any track is at 5 min remaining and still
productive (per user pick).

### Track A — Live-rendering fix: auth helper + BFF wiring + AdminAuthGate short-circuit

**Why first**: every other track depends on the `permissions: Vec<String>`
field being populated by the BFFs. Do this first, then the other
tracks can extract sub-components without re-doing the auth
plumbing.

**Files**:
- **New file** `shared/rust/auth/src/permissions.rs` — define
  `default_permissions_for_role(role: &str) -> Vec<String>` and
  `permissions_for_roles(roles: &[String]) -> Vec<String>`. ~25 LoC
  + 5 unit tests (one per role + the wildcard case + the dedup
  case).
- **Edit** `shared/rust/auth/src/lib.rs` — re-export
  `pub mod permissions;` and `pub use permissions::*;`. ~2 LoC.
- **Edit** `apps/admin/src/ssr.rs:37-51` — replace
  `permissions: vec![]` with
  `permissions: epsx_auth::permissions_for_roles(&u.roles)`. ~3 LoC
  diff.
- **Edit** `apps/frontend/src/ssr.rs:38-52` — same change. ~3 LoC
  diff.
- **Edit** `shared/rust/dioxus_ui/src/auth/auth_gate.rs:163-175` —
  add the explicit `is_admin` short-circuit:
  ```rust
  if has_admin {
      return rsx! { Fragment { {children} } };
  }
  ```
  ~3 LoC diff (one new line + the surrounding context).

**New unit tests** (Track A):
- `default_permissions_for_role_admin_returns_wildcard`
- `default_permissions_for_role_editor_returns_read_plus_content_write`
- `default_permissions_for_role_merchant_returns_payment_and_wallet`
- `default_permissions_for_role_user_returns_wallet_read`
- `default_permissions_for_role_unknown_returns_empty`
- `permissions_for_roles_unions_and_dedups`
- `admin_auth_gate_is_admin_short_circuits_required_permissions`
- `admin_auth_gate_non_admin_still_enforces_required_permissions`

**Do not run** `cargo build --workspace --bins` — that's the
integration gate.

### Track B — Admin sub-components: dashboard + analytics + policies (3 pages, ~25 sub-components)

**Why these together**: they share the `AdminShell` + stats/chart
pattern. Worker builds the per-page sub-component shape once,
applies it to 3 of the biggest admin pages.

**Files**:
- 3 admin pages in `shared/rust/dioxus_ui/src/pages/admin_pages/`:
  `dashboard.rs`, `analytics.rs`, `policies.rs`. Each shrinks by
  ~30-50% as inlined JSX moves to named sub-components.
- 3 new module files under `shared/rust/dioxus_ui/src/components/admin/`:
  `dashboard.rs`, `analytics.rs`, `policies.rs`.
- Update `shared/rust/dioxus_ui/src/components/mod.rs` (or add to
  the existing `mod.rs` — match the Wave 6B pattern). New path:
  `components::admin`.
- CSS: minimal, only inside the existing
  `// === wave6b-admin-pages-depth-track-a ===` region in
  `shared/rust/templates/src/lib.rs` (do NOT add a new region).

**New sub-components** (from the table above):
- `AdminStatsCards`, `WalletsByChain`, `RecentTransactions`,
  `SystemAlerts`, `ActivityStream` (dashboard)
- `AnalyticsHeader`, `AnalyticsCardGrid`, `AnalyticsChart`,
  `AnalyticsTable`, `AnalyticsFilterPanel`, `AnalyticsExportDialog`,
  `AnalyticsMetadata` (analytics) — most of these are already
  factored as local `#[component]`s in `analytics.rs`; the
  extraction is mostly moving them to `components::admin::analytics`
  and re-importing.
- `PolicyStatsBar`, `PolicyBuilder`, `PolicyMonitor`, `PolicyCard`,
  `PolicyFilters`, `PolicyList` (policies)

Each gets the same `test_render_smoke` + the parent page's
`test_section_markers` test stays.

### Track C — Admin sub-components: payments + wallet + settings + media (6 pages, ~25 sub-components)

**Why these together**: they share the `AdminTable<T>` pattern (from
Wave 6B Track C) + the action-column wrapper.

**Files**:
- 6 admin pages: `payments.rs`, `wallet_credits.rs`, `wallet_plans.rs`,
  `wallet_access.rs` (the 4 financial-surface pages from Wave 6B
  Track C), plus `settings.rs` and `media.rs` (the smaller ones from
  Wave 6B Track A — moving them here because Track B is heavy on
  dashboard/analytics/policies and these need a home).
- New `components/admin/payments.rs`, `components/admin/wallet.rs`
  (handles `wallet_credits`, `wallet_plans`, `wallet_access`),
  `components/admin/settings.rs`, `components/admin/media.rs`.

**New sub-components** (from the table above):
- `PaymentLinksList`, `CreateLinkForm`, `LinkRevokeConfirm`,
  `PaymentLinkStats`, `AccessManagementList`, `PaymentsFilterPanel`
  (payments)
- `CreditsLedger`, `CreditsBalanceCards`, `CreditsTransactionList`,
  `CreditsTopupForm`, `CreditsRevokeDialog` (wallet_credits)
- `PlanListSidebar`, `PlanEditorPage`, `PlanEditorDrawer`,
  `PlanApiLimits`, `PlanPromotions`, `PlanItemCard` (wallet_plans)
- `WalletAccessManager`, `PlanSelectorModal`, `AccessGrantForm`,
  `AccessRevokeDialog` (wallet_access)
- `SettingsDashboard`, `ApiKeysList`, `EmailSettings`,
  `NotificationSettings`, `SessionManagement` (settings)
- `MediaBrowser`, `MediaUploader`, `MediaFilters`, `MediaStats`
  (media)

### Track D — Admin sub-components: wallet_wallets + chat + developer_portal + audit_log + auth_page + news + notifications (7 pages, ~30 sub-components)

**Why these together**: the remaining admin pages after Tracks B
and C, mostly the long-tail + Wave 6B Track B/D content. Worker
extracts sub-components for the remaining 7 pages.

**Files**:
- 7 admin pages: `wallet_wallets.rs`, `chat.rs`, `developer_portal.rs`,
  `audit_log.rs`, `auth_page.rs`, `news.rs`, `notifications.rs`
- New `components/admin/{wallets,chat,developer,audit,auth,news,notifications}.rs`

**New sub-components** (from the table above):
- `WalletStatsBar`, `WalletList`, `WalletDetailView`,
  `WalletTableRow`, `WalletCardSections`, `WalletDetailPanel`,
  `WalletDisableDialog`, `WalletReenableDialog` (wallet_wallets)
- `AdminChatInbox`, `AdminChatConversationView`, `ChatReplyInput`,
  `ChatInboxSearch`, `ChatUnreadBadge` (chat; reuse Wave 6A's
  `MessageBubble` from `shared/rust/dioxus_ui/src/chat/message_bubble.rs`)
- `DeveloperPortalOverview`, `ApiKeysTab`, `ApiKeyCreateForm`,
  `ApiKeyRevokeModal`, `UsageAnalyticsTab`, `DocumentationTab`,
  `DeveloperPortalStats` (developer_portal)
- `AuditFilters`, `AuditTimeline`, `AuditEntryDetail`,
  `AuditSeverityBreakdown`, `AuditExportButton` (audit_log)
- `AuthMethodSelector`, `AuthRedirectHandler` (auth_page)
- `NewsManagementList`, `NewsEditor`, `NewsFeaturedCard`,
  `ArticleCard`, `NewsEmptyState`, `NewsPagination` (news)
- `NotificationList`, `SendForm`, `RecipientsPicker`,
  `NotificationTemplateEditor`, `NotificationPreview`,
  `NotificationScheduleDialog`, `NotificationManagementFilters`
  (notifications)

### Track E — User-side 1:1 component parity (16 pages, ~20-30 new sub-components)

**Why separate**: the user side has 16 pages from Wave 6A; the
extraction work overlaps with admin sub-components in tooling but
not in files.

**Pages**: the 16 Wave 6A user pages: `account`, `account_credits`,
`analytics`, `chat`, `chat_conversation`, `chat_history`,
`dashboard`, `developer`, `home`, `manual`, `news`, `news_detail`,
`notifications`, `payment`, `permissions`, `plans`, `portfolio`,
`profile`, `about`, `contact`, `access_denied`.

(Note: actual page count from `pages/` is 30, but the design-doc
16-pages list reflects the 16 user-facing pages that the
`render_page` dispatcher routes to. Other 14 are sub-routes or
utility pages like `error_page`, `not_found`, `offline` — left
untouched.)

**Existing extracted sub-components from Wave 6A** (reused, no
change): `MessageBubble`, `ExportDialog`, `PaymentFlowSteps`,
`PlanComparisonCard`, `ChainVerificationCard`, `UpgradeBanner`,
`UnifiedPaymentFlow`, `PermissionCategoryBreakdown`,
`TopMoversCard`.

**New sub-components** to extract: per-page, mirror the source
where it makes sense. Same rule as admin: extract when the
inlined JSX would become a named sub-component with ≥ 1 typed
prop or ≥ 2 uses elsewhere.

**Files**:
- 16 user pages in `shared/rust/dioxus_ui/src/pages/`: each shrinks
  by ~20-40% as inlined JSX moves to named sub-components.
- New `components/user/{page}.rs` files (1 per page, or grouped
  by domain — pick what fits).

### Integration gate

Same as Wave 6B. 5 `--no-ff` merges into `wave6c/integration`,
full cargo gate, BFF smoke on **all 16 admin routes (now with 5
PARTIAL → PASS) + 16 user routes for parity**, fast-forward to
`migration/dioxus-microservices`, push.

The integration agent also:
1. Verifies the new `components::admin` and `components::user`
   modules are correctly re-exported from
   `shared/rust/dioxus_ui/src/lib.rs`.
2. Confirms the new `permissions` module is reachable from both
   BFFs (no crate-graph breaks).
3. Resolves any conflicts in `shared/rust/templates/src/lib.rs`
   (CSS region concatenates).
4. Rewrites `deliverable.md` to summarize the actual final state
   and includes that rewrite in the final integration commit
   (per the Wave 6B memory entry).

The deliverable at the end of this wave lists:
- Merge log (5 merge commits)
- Cargo gate output (target: <60s check, <60s build, full test pass)
- BFF smoke results: **16/16 admin routes PASS** (including the 5
  previously-PARTIAL ones); **16/16 user routes PASS**.
- `git diff bb18a078..HEAD --stat`
- Push confirmation + final HEAD hash
- Pointer to the separate `wave7-design` plan doc (not in this
  wave's deliverable).

## File ownership (no overlap)

| Track | Files owned | New files (allowed) |
| ----- | ----------- | ------------------- |
| A | `shared/rust/auth/src/{lib.rs,permissions.rs}`, `apps/admin/src/ssr.rs`, `apps/frontend/src/ssr.rs`, `shared/rust/dioxus_ui/src/auth/auth_gate.rs` | `shared/rust/auth/src/permissions.rs` |
| B | `pages/admin_pages/{dashboard,analytics,policies}.rs` | `components/admin/{dashboard,analytics,policies}.rs` |
| C | `pages/admin_pages/{payments,wallet_credits,wallet_plans,wallet_access,settings,media}.rs` | `components/admin/{payments,wallet,settings,media}.rs` |
| D | `pages/admin_pages/{wallet_wallets,chat,developer_portal,audit_log,auth_page,news,notifications}.rs` | `components/admin/{wallets,chat,developer,audit,auth,news,notifications}.rs` |
| E | `pages/{account,account_credits,analytics,chat,chat_conversation,chat_history,dashboard,developer,home,manual,news,news_detail,notifications,payment,permissions,plans,portfolio,profile,about,contact,access_denied}.rs` | `components/user/{page}.rs` (1 file per page or grouped) |
| All | `shared/rust/templates/src/lib.rs` (CSS regions) | each track's own block |
| All | `shared/rust/dioxus_ui/src/lib.rs`, `components/mod.rs` | new `components::admin` and `components::user` modules |

## Track worker constraints (carry-overs from Wave 5/6A/6B)

- Per-track workers do **not** run `cargo build --workspace --bins`
  (15-min link cost eats the worker cap). Per-track verify is
  `cargo check -p epsx-dioxus-ui --lib` + `cargo test -p epsx-dioxus-ui --lib`.
  Integration gate owns the full workspace build.
- Each track writes to its own `// === wave6c-1to1-track-X ===`
  block in `shared/rust/templates/src/lib.rs` for any CSS region
  additions.
- If a new shared primitive is needed, add it under
  `components/admin/` or `components/user/` and `pub use` it from
  the parent `mod.rs` as appropriate. Document the new path in
  the deliverable so other tracks can import.
- Don't pad LoC to hit per-page targets. Real components only.
  Extraction is mostly file moves; if a track is short on LoC, the
  right answer is to extract a sub-component with real props + 1
  test, not to pad.
- Use the shared `src/tests/mod.rs::test_user_with()` fixture
  (or a new admin-scoped `test_admin_user_with(extra_perms)`)
  instead of writing per-page `authed_ctx()` copies.
- **Source-of-truth pre-flight**: before extracting a sub-component
  for page X, `wc -l apps-old/admin-frontend/app/X/page.tsx
  apps-old/admin-frontend/components/X/*.tsx` and read the source
  end-to-end. Confirm the design-doc component list matches the
  source. If source has 5 components and doc lists 6, the doc is
  wrong; port to the source.

## What Wave 6C explicitly does NOT cover

- **Wave 7 design doc** → separate "wave7-design" plan launched
  after Wave 6C lands. This wave ships a stub README pointer only.
- **Backend services extraction** → Wave 7.
- **Service-derived permissions** (option (b) above; deferred to
  a future wave).
- **Real Playwright visual diff** against `apps-old`.
- **Adding new auth flows or roles** (the SIWE + cookie + JWT +
  `AdminAuthGate` stack is sufficient; the live-rendering fix is
  the only auth-related change).
- **gRPC** (explicitly out unless a non-Rust client appears).

## Source-of-truth pre-flight (read first)

Per the Wave 2 lesson (memory: "Bake the doc content into the task
prompts themselves — verbose but bulletproof"), each track worker
should:

1. `wc -l apps-old/{admin,user}-frontend/app/{slug}/page.tsx
   apps-old/{admin,user}-frontend/components/{slug}/*.tsx`
2. Read the source TSX end-to-end **before** touching port code.
3. Confirm the design-doc component list matches what the source
   actually has. If the source has 5 components and the doc lists
   6, the doc is wrong; port to the source.
4. The unit-test section-marker assertion in each ported page is
   the contract. If a marker is missing, the test fails, the
   worker fixes it, the gate passes.

## Wave 6C plan YAML (proposed)

```yaml
id: wave6c-live-render-and-1to1
title: "Wave 6C — Live-rendering fix + 1:1 component parity (admin + user)"
description: |
  Close the live-rendering gap (5 PARTIAL admin smoke rows) and
  extract named sub-components from the 20+ admin pages and 16
  user pages to mirror the Next.js source 1:1. Wave 7 design
  doc is deferred to a separate plan.

depends_on_plan: plan_083a2d9d  # Wave 6B
base_branch: wave6c/live-render-and-1to1  # new branch off migration/dioxus-microservices
target_branch: migration/dioxus-microservices

tasks:
  - id: track-a-live-render-fix
    role: produce
    title: "Track A — Live-rendering fix: permissions helper + BFF wiring + AdminAuthGate short-circuit"
    timeout_ms: 2700000  # 45 min
    agent: coder
    worktree_branch: wave6c/track-a-live-render
    prompt_file: tasks/track-a-live-render-fix.md
    verify_prompt_file: verify/track-a-live-render-fix.md

  - id: track-b-admin-components-a
    role: produce
    title: "Track B — Admin sub-components: dashboard + analytics + policies"
    timeout_ms: 2700000  # 45 min
    agent: coder
    worktree_branch: wave6c/track-b-admin-components-a
    depends_on: [track-a-live-render-fix]
    prompt_file: tasks/track-b-admin-components-a.md
    verify_prompt_file: verify/track-b-admin-components-a.md

  - id: track-c-admin-components-b
    role: produce
    title: "Track C — Admin sub-components: payments + wallet + settings + media"
    timeout_ms: 2700000  # 45 min
    agent: coder
    worktree_branch: wave6c/track-c-admin-components-b
    depends_on: [track-a-live-render-fix]
    prompt_file: tasks/track-c-admin-components-b.md
    verify_prompt_file: verify/track-c-admin-components-b.md

  - id: track-d-admin-components-c
    role: produce
    title: "Track D — Admin sub-components: wallet_wallets + chat + developer + audit + auth + news + notifications"
    timeout_ms: 2700000  # 45 min
    agent: coder
    worktree_branch: wave6c/track-d-admin-components-c
    depends_on: [track-a-live-render-fix]
    prompt_file: tasks/track-d-admin-components-c.md
    verify_prompt_file: verify/track-d-admin-components-c.md

  - id: track-e-user-side-1to1
    role: produce
    title: "Track E — User-side 1:1 component parity (16 pages, ~20-30 new sub-components)"
    timeout_ms: 2700000  # 45 min
    agent: coder
    worktree_branch: wave6c/track-e-user-side-1to1
    depends_on: [track-a-live-render-fix]
    prompt_file: tasks/track-e-user-side-1to1.md
    verify_prompt_file: verify/track-e-user-side-1to1.md

  - id: integration-gate
    role: produce
    title: "Integration gate — merge Tracks A+B+C+D+E, full cargo gate, BFF smoke 32 routes, push"
    timeout_ms: 2400000  # 40 min
    agent: coder
    worktree_branch: wave6c/integration
    depends_on:
      - track-a-live-render-fix
      - track-b-admin-components-a
      - track-c-admin-components-b
      - track-d-admin-components-c
      - track-e-user-side-1to1
    prompt_file: tasks/integration-gate.md
    verify_prompt_file: verify/integration-gate.md
```

## Wave 6A/6B lessons applied to 6C

- **No new CSS** unless absolutely necessary. The existing region
  markers in `shared/rust/templates/src/lib.rs` are the only
  allowed home for new CSS.
- **Reuse `src/tests/mod.rs::test_user_with()`** for the per-page
  test fixture. All admin pages should use a single
  admin-scoped variant: `test_admin_user_with(extra_permissions: &[&str])`.
  After Track A, the BFF plumbs
  `permissions: vec!["*:*:*".into()]` for admin users.
- **Per-track `cargo check -p epsx-dioxus-ui --lib` only.**
  Full `cargo build --workspace --bins` lives in the integration
  gate.
- **Don't pad LoC to hit per-page targets.** Extraction is mostly
  file moves (page shrinks, sub-component file grows); the net
  diff should be small. If a track is short on LoC, the right
  answer is to extract a sub-component with real props + 1 test,
  not to pad.
- **Reuse `MessageBubble` from Wave 6A in admin chat** (already
  done in Wave 6B; carry it into the extraction).
- **Commit the design doc to the base branch BEFORE launching
  the plan** (Wave 2 lesson re-applied).
- **Preemptively extend any track that hits 5 min remaining and
  is still productive** (Wave 6A/6B pattern; now 15 min preempt
  on the 45-min worker cap).
- **Integration gate final commit includes the rewritten
  `deliverable.md`** (Wave 6B memory entry).

## Wall-clock estimate

| Track | Time | Notes |
| ----- | ---- | ----- |
| A — Live-rendering fix | 30-45 min | New helper + 2 BFFs + 1 gate change + 8 tests |
| B — Admin sub-components A | 30-45 min | 3 admin pages, ~25 sub-components |
| C — Admin sub-components B | 30-45 min | 6 admin pages, ~25 sub-components |
| D — Admin sub-components C | 30-45 min | 7 admin pages, ~30 sub-components |
| E — User-side 1:1 | 30-45 min | 16 user pages, ~20-30 sub-components |
| Integration gate | 20-25 min | 5 merges + cargo + smoke + push |

**Total wall-clock**: 4-5 hours.
