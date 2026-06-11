# Wave 6B — Admin Pages Depth: Close the Admin UX/UI Gap

## Why this wave exists

Wave 5 closed the gap for the 12 marketing/auth pages a signed-out
visitor sees. Wave 6A closed the gap for the 16 auth-required user
pages. Both used the same recipe: bring the Dioxus port from
"every page has a file" to "every page matches the Next.js source
at the section level", with section-marker unit tests and a
shared-CSS region per track.

Wave 6B is the last big UX surface. The admin side has **22 unique
page routes** (27 counting nested routes like `wallet-management/[address]`)
and **~27,760 LoC of admin sub-components** across 9 component
groups: wallet-management, access-control, admin, policies,
payments, plans, media, notifications, news. The Dioxus port
currently has 20 admin page files at ~1,442 LoC combined (avg
~72 LoC each), which is the same shallow-stub state Wave 5/6A
started from.

After Wave 6B lands, the admin surface should match the
Next.js source at the same section level Wave 6A hit on the
user side.

## What "100% UX/UI" means for this wave

Same definition Wave 5/6A used:

1. **All sections from the source Next.js page + components are
   present** in the Dioxus port — same `#[component]` blocks
   the source has TSX sub-components for, in the same order.
2. **All copy/text is preserved** — strings come from the source
   page or its components, not invented.
3. **Visual class names match the design system** — same
   `tailwind`-style utility classes already used in
   `shared/rust/dioxus_ui/src/layout/` and `primitives/`.
4. **SSR renders the static parts at request time** — every page
   already uses the `#[component] fn render(ctx: &PageContext) -> Element`
   pattern. Wave 6B keeps the pattern and makes the components
   bigger.
5. **Each page has a unit test** that calls `render()` with a
   minimal `PageContext` (with admin permissions populated
   via `src/tests/mod.rs::test_user_with()` or the per-page
   admin-scoped variant) and asserts the page body contains the
   expected section markers.

What this wave does NOT do:

- gRPC, microservices extraction, or backend work.
- 1:1 component parity (every Next.js sub-component as a
  separate `#[component] fn`) — that's a separate Wave 6C if
  the user wants it. Wave 6B is section-level parity, same
  scope as Wave 5/6A.
- Real Playwright visual diff against `apps-old`.
- Adding new auth flows (the SIWE + cookie + JWT + `AdminAuthGate`
  stack from Wave 2/3b is fine).

## Wave 6A lessons applied to 6B

- **No new CSS in inline styles, no new deps.** Reuse existing
  design tokens. New CSS only in the `// === wave6b-admin-pages-depth-track-X ===`
  region in `shared/rust/templates/src/lib.rs`.
- **Reuse `src/tests/mod.rs::test_user_with()`** for the
  per-page test fixture — Track D Wave 6A made this mistake
  (wrote 6 copies of `authed_ctx()` before finding the shared
  helper). All admin pages should use a single admin-scoped
  variant: `test_admin_user_with(extra_permissions: &[&str])`.
- **Per-track `cargo check -p epsx-dioxus-ui --lib` only.**
  Full `cargo build --workspace --bins` lives in the
  integration gate (15-min link cost eats the 30-min worker cap).
- **Don't pad LoC to hit per-page targets.** Track D's
  Wave 6A verifier rejected attempt 1 because 3 pages were
  under target. The retry was clean because the LoC fix-up
  added *real* design-doc-named components (`PaymentFlowSteps`,
  `PermissionsMatrix`, `TopMoversCard`), not padding. Carry
  this discipline to Wave 6B.
- **Use `MessageBubble` from Wave 6A in admin chat.** It's
  already extracted and re-exported; don't duplicate.

## Source of truth (same as Wave 5/6A)

| What | Where |
| --- | --- |
| Next.js page entry | `apps-old/admin-frontend/app/{slug}/page.tsx` |
| Page-local sub-components | `apps-old/admin-frontend/components/{slug}/*.tsx` |
| Ported Dioxus page | `shared/rust/dioxus_ui/src/pages/admin_pages/{slug}.rs` |
| Shared design-system primitives | `shared/rust/dioxus_ui/src/primitives/` |
| Layout chrome | `shared/rust/dioxus_ui/src/layout/main_layout.rs` |
| BFF → page glue | `shared/rust/dioxus_ui/src/pages.rs` |
| Shared test fixture | `shared/rust/dioxus_ui/src/tests/mod.rs::test_user_with()` |

**Read the source TSX first.** Every port starts with
`wc -l apps-old/admin-frontend/app/{slug}/page.tsx apps-old/admin-frontend/components/{slug}/*.tsx`
to size the work, then read the source end-to-end, then port.

## The 20 admin pages, sized

The Dioxus port has 20 admin page files (1 missing — `audit-log`
is in `apps-old/admin-frontend/app/audit-log/page.tsx` but
not yet in the port; `auth/page.tsx` and `wallet-management/page.tsx`
are the same — only 5-line stubs in source). Total source LoC =
~28,627 (867 page.tsx + 27,760 sub-components) but with massive
spread: `developer-portal/api-keys/create/page.tsx` is 211 LoC
on its own; `auth/page.tsx` is 5 LoC. Most of the work lives
in the sub-components.

**Per-page LoC target** = `current + 3x_source_LoC`, capped at
the design-doc target per page. Pages that already have
substantial sub-component work absorbed (e.g. `payments.rs`
48 LoC → ~500 LoC target; `wallet_wallets.rs` 150 LoC → ~700
LoC target) get bigger jumps. Trivial 5-line pages (`auth`,
`wallet-management` list view) stay small (~80-100 LoC target).

| # | Page (port → target) | Source LoC | Sub-component LoC | Notes |
| - | -------------------- | ---------- | ----------------- | ----- |
| 1 | `dashboard.rs` 103 → 400+ | 24 | 3,858 (admin) | The admin overview — wallets, transactions, alerts |
| 2 | `analytics.rs` 66 → 400+ | 78 | 1,250 (analytics subdir) | System-wide analytics with filters, export, plan-status bar |
| 3 | `audit.rs` 192 → 500+ | 107 | (uses shared audit-log components) | Audit log with filters, timeline, severity |
| 4 | `policies.rs` 97 → 350+ | 26 | 1,166 (policies) | Policy builder + monitor + stats |
| 5 | `settings.rs` 69 → 250+ | 26 | (settings components in `admin/`) | Admin settings dashboard |
| 6 | `media.rs` 42 → 250+ | 17 | 337 (media) | Media browser + uploader |
| 7 | `payments.rs` 48 → 500+ | 31 | 1,239 (payments) | Payment links + access management |
| 8 | `news.rs` 94 → 350+ | 18 | 829 (news) | News management + editor |
| 9 | `notifications.rs` 86 → 400+ | 5 (page) + 27 (create) | 598 (notifications) | Send/manage notifications |
| 10 | `chat.rs` 74 → 350+ | 28 + 38 ([id]) | 567 (chat subdir) | Admin chat inbox + conversation view (reuse `MessageBubble`) |
| 11 | `developer_portal.rs` 162 → 500+ | 21 + 211 (api-keys/create) | 1,400+ (developer_portal) | API key manager + usage analytics + docs |
| 12 | `wallet_wallets.rs` 150 → 700+ | 19 + 11 (wallets) + 27 ([address]) | 6,951 (wallet) | Largest admin surface — wallets, list, detail, disable |
| 13 | `wallet_plans.rs` 124 → 400+ | 15 + 10 ([planId]) | 1,018 (plans) | Plan editor + list + drawer |
| 14 | `wallet_credits.rs` 49 → 400+ | 63 | 1,239 (payments, shared) | Credits ledger + management |
| 15 | `wallet_access.rs` 24 → 250+ | 5 (access) | 791 (plan-editor subdir) | Wallet access manager |
| 16 | `auth_redirect.rs` 12 → 80+ | (none — admin redirect) | — | Small redirect-only stub |
| 17 | `wallet_redirect.rs` 12 → 80+ | (none) | — | Small redirect-only stub |
| 18 | `notifications_redirect.rs` 12 → 80+ | (none) | — | Small redirect-only stub |
| 19 | `unauthorized.rs` 12 → 100+ | 16 | 109 (admin-feature-gate) | Unauthorized view |
| 20 | `access_denied.rs` 14 → 100+ | 37 | 318 (error-boundary) | Access-denied view |

**Missing files to add** (not in port, in source):
- `audit-log/page.tsx` (107 LoC) → new `audit_log.rs` (the
  existing `audit.rs` is misnamed — this would replace it)
- `auth/page.tsx` (5 LoC) → new `auth_page.rs`
- `wallet-management/page.tsx` (5 LoC list) → fold into
  `wallet_wallets.rs` or create new `wallet_management.rs`

**Total target LoC** (post-Wave 6B): ~6,500-7,500 across the
20-23 admin page files. Same shape as Wave 6A (16 pages, ~6,500 LoC).
Wall-clock estimate: 4-5 hours.

## Track split (4 parallel coder tracks + integration gate)

Four tracks sized to fit in 30-min per-track worker sessions.
Each track writes to its own `// === wave6b-admin-pages-depth-track-X ===`
block in `shared/rust/templates/src/lib.rs` for any CSS region
additions (only Track A and Track D are likely to add CSS).

### Track A — `admin` + `analytics` + `policies` + `settings` + `media` (5 pages, admin shell)

**Pages**: `dashboard.rs`, `analytics.rs`, `policies.rs`, `settings.rs`, `media.rs`

**Why these together**: they share the "admin shell" pattern
(AdminAuthGate → MainLayout → PageHeader → content sections
with `DataTable` / `StatCard` / `Charts`). Worker builds the
admin shell recipe once, applies it to 5 pages.

**Section coverage** (all per design-doc spec):
- `dashboard.rs`: 5 sections — AdminStatsCards, WalletsByChain,
  RecentTransactions, SystemAlerts, ActivityStream
- `analytics.rs`: 7 sections — AnalyticsHeader, AnalyticsCardGrid,
  AnalyticsChart, AnalyticsTable, AnalyticsFilterPanel,
  AnalyticsExportDialog (reuse Wave 6A's `ExportDialog` primitive),
  AnalyticsMetadata
- `policies.rs`: 6 sections — PolicyStatsBar, PolicyBuilder,
  PolicyMonitor, PolicyCard, PolicyFilters, PolicyList
- `settings.rs`: 5 sections — SettingsDashboard, ApiKeysList,
  EmailSettings, NotificationSettings, SessionManagement
- `media.rs`: 4 sections — MediaBrowser, MediaUploader,
  MediaFilters, MediaStats

**New primitives** (likely needed): `AdminShell` (the
`<main>` + sidebar + breadcrumb layout primitive shared by all
admin pages) — register at `shared/rust/dioxus_ui/src/layout/admin_shell.rs`.

### Track B — `audit` + `news` + `notifications` (3 pages, content moderation)

**Pages**: `audit.rs` (rename to `audit_log.rs` + add new
`audit_log.rs`), `news.rs`, `notifications.rs`

**Why these together**: they all deal with content moderation /
lifecycle events. Audit log = immutable event stream. News =
publish/manage articles. Notifications = send + manage fan-out.

**Section coverage**:
- `audit_log.rs`: 5 sections — AuditFilters, AuditTimeline,
  AuditEntryDetail, AuditSeverityBreakdown, AuditExportButton
- `news.rs`: 6 sections — NewsManagementList, NewsEditor,
  NewsFeaturedCard, ArticleCard, NewsEmptyState, NewsPagination
- `notifications.rs`: 7 sections — NotificationList, SendForm,
  RecipientsPicker, NotificationTemplateEditor, NotificationPreview,
  NotificationScheduleDialog, NotificationManagementFilters

**Renames**: Move the existing `audit.rs` content into
`audit_log.rs` (the file is misnamed; `audit-log/page.tsx`
in source is the canonical name). Same for `auth_redirect.rs`
→ keep, no rename.

**New primitives** (likely needed): `AdminActionConfirm`
(modal pattern for "Are you sure you want to revoke this
API key?" / "Delete this notification?") — register at
`shared/rust/dioxus_ui/src/feedback/admin_action_confirm.rs`.

### Track C — `payments` + `wallet_credits` + `wallet_plans` + `wallet_access` (4 pages, financial surface)

**Pages**: `payments.rs`, `wallet_credits.rs`, `wallet_plans.rs`, `wallet_access.rs`

**Why these together**: they all touch the payment / wallet
domain. `payments.rs` is payment-links management (admin
creates and revokes payment links for users). `wallet_credits.rs`
is the credits ledger (admin grants/revokes credits).
`wallet_plans.rs` is the plan editor (admin creates/edits
subscription plans). `wallet_access.rs` is the wallet-access
manager (admin grants/revokes per-feature access).

**Section coverage**:
- `payments.rs`: 6 sections — PaymentLinksList, CreateLinkForm,
  LinkRevokeConfirm, PaymentLinkStats, AccessManagementList,
  PaymentsFilterPanel
- `wallet_credits.rs`: 5 sections — CreditsLedger,
  CreditsBalanceCards (3 sub-cards), CreditsTransactionList,
  CreditsTopupForm, CreditsRevokeDialog
- `wallet_plans.rs`: 6 sections — PlanListSidebar, PlanEditorPage,
  PlanEditorDrawer, PlanApiLimits, PlanPromotions, PlanItemCard
- `wallet_access.rs`: 4 sections — WalletAccessManager,
  PlanSelectorModal, AccessGrantForm, AccessRevokeDialog

**New primitives** (likely needed): `AdminTable` (the
`DataTable` + filters + pagination wrapper shared by all
admin list views) — register at
`shared/rust/dioxus_ui/src/primitives/admin_table.rs`.

### Track D — `wallet_wallets` + `chat` + `developer_portal` + missing pages (5 pages, biggest catch-all)

**Pages**: `wallet_wallets.rs`, `chat.rs`, `developer_portal.rs`,
+ new `audit_log.rs` (if Track B doesn't claim it),
+ new `auth_page.rs` (5 LoC source — admin auth gate redirect)

**Why these together**: the catch-all. `wallet_wallets.rs` is
the biggest admin surface (~6,951 LoC of wallet sub-components,
with 27 LoC detail page, 19 LoC list, 11 LoC disable page).
`chat.rs` is admin chat inbox + conversation (uses Wave 6A's
`MessageBubble` primitive). `developer_portal.rs` has the
biggest single page in the wave (`api-keys/create/page.tsx`
is 211 LoC).

**Section coverage**:
- `wallet_wallets.rs`: 8 sections — WalletStatsBar, WalletList,
  WalletDetailView, WalletTableRow, WalletCardSections,
  WalletDetailPanel, WalletDisableDialog, WalletReenableDialog
- `chat.rs`: 5 sections — AdminChatInbox, AdminChatConversationView
  (reuse `MessageBubble`), ChatReplyInput, ChatInboxSearch,
  ChatUnreadBadge
- `developer_portal.rs`: 7 sections — DeveloperPortalOverview,
  ApiKeysTab, ApiKeyCreateForm, ApiKeyRevokeModal, UsageAnalyticsTab,
  DocumentationTab, DeveloperPortalStats
- `auth_page.rs`: 2 sections — AuthMethodSelector, AuthRedirectHandler
- `audit_log.rs` (if Track B doesn't claim it): see Track B's
  5 sections

**New primitives** (likely needed): `AdminMetricCard` (the
StatCard variant for admin-specific metrics — "Active users
in last 24h", "API errors/hour", etc.) — register at
`shared/rust/dioxus_ui/src/primitives/admin_metric_card.rs`.

### Integration gate (single track)

The integration agent:
1. Merges all 4 track branches into `wave6b/integration` (no-ff).
2. Resolves any conflicts (CSS regions concatenate; page files
   shouldn't conflict because each track owns its own pages).
3. Renames `audit.rs` → `audit_log.rs` in the merge if both
   tracks touched it.
4. Runs the full cargo gate:
   - `cargo check --workspace` — 0 errors expected.
   - `cargo build --workspace --bins` — 0 errors expected.
   - `cargo test -p epsx-dioxus-ui --lib` — 130+ tests expected
     (current 115 + ~15-20 new admin page tests).
5. BFF-smokes all 20 admin routes on the admin BFF (port 3001):
   hit each `/admin/dashboard`, `/admin/audit-log`, etc. and
   assert HTTP 200 + the section markers in the rendered HTML.
6. Fast-forwards `migration/dioxus-microservices` to the integration
   tip.
7. Writes the final deliverable + pushes.

## File ownership (no overlap)

| Track | Files owned | New files (allowed) |
| ----- | ----------- | ------------------- |
| A | `pages/admin_pages/{dashboard,analytics,policies,settings,media}.rs` | `layout/admin_shell.rs` (new primitive) |
| B | `pages/admin_pages/{audit_log,news,notifications}.rs` | `feedback/admin_action_confirm.rs` (new primitive) |
| C | `pages/admin_pages/{payments,wallet_credits,wallet_plans,wallet_access}.rs` | `primitives/admin_table.rs` (new primitive) |
| D | `pages/admin_pages/{wallet_wallets,chat,developer_portal,auth_page,audit_log_if_not_claimed_by_B}.rs` | `primitives/admin_metric_card.rs` (new primitive) |
| All | `shared/rust/templates/src/lib.rs` (CSS regions) | each track's own block |

## Track worker constraints (carry-overs from Wave 5/6A)

- Per-track workers do **not** run `cargo build --workspace --bins`
  (15-min link cost eats the 30-min cap). Per-track verify is
  `cargo check -p epsx-dioxus-ui --lib` + `cargo test -p epsx-dioxus-ui --lib`.
  Integration gate owns the full workspace build.
- Each track writes to its own `// === wave6b-admin-pages-depth-track-X ===`
  block in `shared/rust/templates/src/lib.rs` for any CSS region
  additions.
- If a new shared primitive is needed (e.g. Track A's
  `AdminShell`, Track C's `AdminTable`), add it under the
  existing module hierarchy and `pub use` it from the parent
  `mod.rs` / `lib.rs` as appropriate. Document the new path in
  the deliverable so other tracks can import.
- Don't pad LoC to hit per-page targets. Real components only.
- Use the shared `src/tests/mod.rs::test_user_with()` fixture
  (or a new admin-scoped `test_admin_user_with(extra_perms)`)
  instead of writing per-page `authed_ctx()` copies.

## What Wave 6B explicitly does NOT cover

- **Backend services extraction** → Wave 7 (separate plan).
- **1:1 component parity** (every Next.js sub-component as
  its own `#[component] fn`) → Wave 6C if the user wants it.
- **Real Playwright visual diff** → separate infra project.
- **Adding new auth flows or roles** → admin roles / permissions
  expansion is a backend workstream, not a UI port.

## Source-of-truth pre-flight (read first)

The Wave 2 lesson (memory: "Bake the doc content into the task
prompts themselves — verbose but bulletproof") says don't trust
the design doc alone. Each track worker should:

1. `wc -l apps-old/admin-frontend/app/{slug}/page.tsx apps-old/admin-frontend/components/{slug}/*.tsx`
2. Read the source TSX end-to-end **before** touching port code.
3. Confirm the design-doc section list matches what the source
   actually has. If the source has 5 sections and the doc lists 6,
   the doc is wrong; port to the source. If the doc lists 5 and
   the source has 4, port the 4 and update the doc in the
   deliverable.
4. The unit-test section-marker assertion in each ported page is
   the contract. If a marker is missing, the test fails, the
   worker fixes it, the gate passes.
