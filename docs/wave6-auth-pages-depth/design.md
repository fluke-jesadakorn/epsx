# Wave 6 — Auth-Required User Pages: Close the Auth-Required UX/UI Gap

## Why this wave exists

Wave 5 closed the gap for the 12 marketing/auth pages a signed-out
visitor sees. After Wave 5 the homepage, /about, /plans, /contact,
/auth, /manual, /privacy, /terms, /news, /not-found, /offline,
/access-denied, and /error all match the Next.js source at section
level (unit-test assertions on section markers; the design doc at
`docs/wave5-page-depth/design.md` documents the methodology).

But the **16 auth-required user pages** — the pages signed-in users
actually live in — are still stub-thin. Same for the **admin side**,
which is a separate follow-up wave (Wave 6B) because its 27 page
routes + 22,000 LoC of admin sub-components is too big to fold in.

This wave (Wave 6A) tackles the user side. After it lands, every page
behind `AuthGate` should match the Next.js source at the same section
level Wave 5 hit on the marketing side.

## Honest scope acknowledgement

The Wave 1-3 port collapsed the Next.js component hierarchy into a
smaller set of primitives + page files. Some sub-components are
inlined into the ported page, some are absorbed into the shared
`primitives/` and `layout/` crates, some are still genuinely missing
(no equivalent code at all). A complete diff would require a
component-by-component map; Wave 6A is **not** a complete diff. It's
a depth pass that targets the same goal as Wave 5 — match the source
at the section level, with the same unit-test + curl-smoke gate.

If you want true 1:1 component parity (every Next.js sub-component
ported as a separate `#[component] fn`), say so and we'll plan that
as a separate wave (likely 6C) — the per-file LoC budget roughly
doubles for that mode.

## What "100% UX/UI" means for this wave

Same definition Wave 5 used, repeated for the new pages:

1. **All sections from the source Next.js page are present** in the
   Dioxus port — the same `#[component]` blocks (or inlined sections)
   the original has TSX sub-components for, in the same order.
2. **All copy/text is preserved** — strings come from the source
   page or its components, not invented.
3. **Visual class names match the design system** — same
   `tailwind`-style utility classes already used in
   `shared/rust/dioxus_ui/src/layout/` and `primitives/`.
4. **SSR renders the static parts at request time** — every page
   already uses the `#[component] fn render(ctx: &PageContext) -> Element`
   pattern. Wave 6A keeps the pattern and makes the components bigger.
5. **Each page has a unit test** that calls `render()` with a minimal
   `PageContext` and asserts the page body contains the expected
   section markers (CSS class names that identify each section).

What this wave does NOT do:

- gRPC, microservices extraction, or any backend work.
- The 27 admin page routes — those are Wave 6B.
- The 11 stub admin pages the handoff mentioned (audit-log, auth,
  wallet-management) — they're already ported at the file level;
  the 6B wave will deepen them like the rest of admin.
- A real Playwright visual diff against `apps-old`. Same as Wave 5:
  unit-test section markers + curl smoke.
- The 4 user pages in the
  `apps-old/frontend/components/{payments,credits,promotions}/...`
  sub-tree that don't have a matching `app/.../page.tsx` route — they
  appear as `<XPanel>` widgets inside other pages and get folded into
  those pages' scope (e.g. `payments-management.tsx` is a sub-component
  of `/admin/payments`).

## Source of truth (same as Wave 5)

| What | Where |
| --- | --- |
| Next.js page entry | `apps-old/frontend/app/{slug}/page.tsx` |
| Page-local sub-components | `apps-old/frontend/components/{slug}/*.tsx` |
| Page-local data | `apps-old/frontend/app/{slug}/data.{ts,tsx}` |
| Ported Dioxus page | `shared/rust/dioxus_ui/src/pages/{slug}.rs` |
| Shared design-system primitives | `shared/rust/dioxus_ui/src/primitives/` |
| Layout chrome | `shared/rust/dioxus_ui/src/layout/main_layout.rs` |
| BFF → page glue | `shared/rust/dioxus_ui/src/pages.rs` |

**Read the source TSX first.** Every port starts with
`wc -l apps-old/frontend/app/{slug}/page.tsx apps-old/frontend/components/{slug}/*.tsx`
to size the work, then read the source end-to-end, then port.

## The 16 pages, sized

Source LoC for each page = its `app/{slug}/page.tsx` + the total of
all `components/{slug}/*.tsx` files. The port LoC is current.

| # | Page | Port LoC | Source LoC (page + components) | Notes |
| - | ---- | -------- | ------------------------------ | ----- |
| 1 | `dashboard` | 155 | 75 + 295 = 370 | big client component, 1 main file |
| 2 | `account` | 107 | 33 + 1,076 = 1,109 | 4 client files; `account-client.tsx` is 362 LoC |
| 3 | `account_credits` | 51 | 11 + 0 = 11 | tiny page, mostly uses `credits-management.tsx` (431 LoC) widget — fold into `account.rs` |
| 4 | `analytics` | 78 | 84 + 2,951 = 3,035 | 19 sub-components, biggest single page surface in this wave |
| 5 | `chat` | 103 | 49 + 1,575 = 1,624 | 11 sub-components; split chat-history + chat-conversation already exists in port |
| 6 | `chat_history` | 30 | 169 + 0 = 169 | static list view; small |
| 7 | `chat_conversation` | 38 | 115 + 0 = 115 | conversation view; depends on chat/chat-input (177 LoC) + chat-message-item (131 LoC) |
| 8 | `developer` | 130 | 27 + 1,522 = 1,549 | 15 sub-components; api-key-manager, plan-transfer-list, usage-monitor, docs |
| 9 | `news` | 79 | 60 + 333 = 393 | news-list (185 LoC) + news-detail (148 LoC) |
| 10 | `news_detail` | 50 | 29 + 0 = 29 | static; small |
| 11 | `notifications` | 105 | 24 + 562 = 586 | 4 sub-components; notification-bell-client is the heaviest (266 LoC) |
| 12 | `payment` | 114 | 130 + 2,382 = 2,512 | 6 sub-components; payment-flow-steps is 783 LoC, biggest single file |
| 13 | `payment_detail` (the `[type]/[id]` route) | (split into `payment.rs` + `[type]/[id]` handler) | 186 + 0 = 186 | dynamic route — extend `payment.rs` with a parameterised detail panel |
| 14 | `permissions` | 93 | 91 + 0 = 91 | inline page, no sub-components; small |
| 15 | `portfolio` | 154 | 48 + 331 = 379 | watchlist-provider (113 LoC) is the heaviest |
| 16 | `profile` | 108 | 50 + 1,193 = 1,243 | 4 sub-components; data-management is 384 LoC, email-management is 350 LoC |

**Total source LoC** across the 16 pages: ~13,500 (rough sum;
some files counted twice in nested dirs).
**Total current port LoC**: 1,395.
**Total target port LoC** (post-Wave 6A): ~6,500–7,500 (depends on
how much is inlined vs lifted into shared primitives).

Wave 5 delivered ~3,500 LoC of port code across 12 pages in ~3 hours
of wall-clock. Wave 6A is ~2x that LoC, so plan budget is **4-5
hours wall-clock** with 4 parallel tracks.

## Track split (4 parallel coder tracks + integration gate)

Four tracks, sized to fit in 30-min per-track worker sessions. Each
track writes to a different `// === wave6-auth-pages-depth-track-X ===`
block in `shared/rust/templates/src/lib.rs` for any CSS region
additions (same Wave 5 pattern; only Track A is likely to add CSS).

### Track A — `dashboard` + `account` + `account_credits`

**Pages**: `dashboard.rs`, `account.rs`, `account_credits.rs`

**Why these together**: both have a single dominant client
component (`dashboard-client.tsx` 295 LoC, `account-client.tsx` 362
LoC) that wraps a number of section sub-components. The pattern
is "1 big `#[component]` function per page that orchestrates the
sections". Track A also gets the `account_credits.rs` because
the source `account/credits/page.tsx` is 11 LoC and almost entirely
delegates to `credits-management.tsx` widget — Track A folds the
widget into the parent `account.rs` since they're a unit.

**`dashboard.rs` target sections** (current 155 → 350+ LoC):
- `StatCardsRow` — 4 stat cards (earnings, watchlist, plans, API
  calls) — already present, ensure copy matches.
- `ActivityCard` — recent activity feed with "Refresh" button —
  already present, add the empty-state CTA link.
- `QuickActionsCard` — 5 quick action buttons — already present.
- **NEW** `EarningsChart` — line chart of 7-day earnings (uses
  `Charts` primitive from `primitives/charts.rs`).
- **NEW** `WatchlistSnapshot` — top 3 watchlist items with mini
  sparkline.
- **NEW** `PlanSummaryCard` — current plan, usage %, upgrade CTA.
- `YourAccountCard` — already present, ensure link to `/account`.

**`account.rs` target sections** (current 107 → 500+ LoC):
- `ProfileTab` — display name, email, bio, avatar — source has
  `account-client.tsx` `ProfileTab` sub-component.
- **NEW** `SubscriptionTab` — current plan, billing cycle,
  upgrade/downgrade.
- **NEW** `UsageTab` — API call chart, credits remaining.
- **NEW** `NotificationsTab` — per-channel notification preferences
  (the `notification-settings-panel.tsx` 110 LoC).
- **NEW** `ConnectedAccountsTab` — wallet, OAuth providers, sessions.
- `DangerZoneTab` — delete account, sign out everywhere.

**`account_credits.rs` target sections** (current 51 → 150 LoC):
- The 431 LoC `credits-management.tsx` widget — port as inline
  `#[component] fn CreditLedger` with 3 sub-components
  (balance, transaction list, top-up form).

**New primitive if needed**: nothing required; existing primitives
cover this. If the worker needs an `AccountTabNav` component,
add it to `shared/rust/dioxus_ui/src/data/tabs.rs` (existing tabs
primitive) — but only if the existing `Tabs` doesn't fit.

**Constraint**: no new CSS region. The credits widget uses
`Card` + `StatCard` + `EmptyState` + `DataTable` primitives, all
existing.

### Track B — `analytics` + `developer`

**Pages**: `analytics.rs`, `developer.rs`

**Why these together**: the two biggest "data dashboard" pages in
the user side. Both have a complex `filter-form.tsx` (187 + 199 LoC
respectively) and a "card dashboard" pattern (1 main client + 5+
sub-sections). Sharing the work lets one worker learn the pattern
once and apply it twice.

**`analytics.rs` target sections** (current 78 → 600+ LoC):
- `AnalyticsHeader` — date range picker, "Export" button, filter
  toggle — already partially present.
- `FilterPanel` — left sidebar with checkboxes (asset, exchange,
  event type) — port from `filter-panel.tsx` 175 LoC.
- `AnalyticsExportDialog` — modal with format selector (CSV,
  JSON, Parquet) — port from `analytics-export-dialog.tsx` 180 LoC.
- `AnalyticsCardGrid` — 2-col grid of metric cards (volume, MAU,
  revenue, etc.) — port from `analytics-card-grid.tsx` 75 LoC +
  the data wiring from `card-dashboard-view.tsx` 133 LoC.
- `AnalyticsChart` — large time-series chart (use `Charts` primitive).
- `AnalyticsTable` — paginated table of recent events — use
  `DataTable` primitive + `Pagination` primitive.
- `AnalyticsMetadata` — last refresh, data source, attribution —
  port from `analytics-metadata-display.tsx` 152 LoC.

**`developer.rs` target sections** (current 130 → 500+ LoC):
- `ApiKeysList` — table of API keys with create/revoke — port from
  `api-key-manager.tsx` 116 LoC.
- `ApiKeyCreateForm` — modal with name/permissions/expiry fields —
  port from `api-key-create-form.tsx` 145 LoC.
- `UsageMonitor` — chart of API calls over time, 429/500 error
  counts — port from `usage-monitor.tsx` 150 LoC.
- `PlanTransferList` — plan quota per endpoint, drag-to-reorder
  priorities — port from `plan-transfer-list.tsx` 131 LoC.
- **NEW** `DocsQuickLinks` — sidebar with "Quick start", "Auth",
  "Rate limits", "Webhooks" links to `/developer/docs`.
- `PermissionList` — current API key permissions display — port
  from `permission-list.tsx` 212 LoC.

**Constraint**: this track CAN add a new primitive
`shared/rust/dioxus_ui/src/data/export_dialog.rs` (the analytics
export dialog is a generic pattern). The worker must add it with
the `// === wave6-auth-pages-depth-track-b ===` CSS marker if it
brings new CSS. Document the new module path in the deliverable.

### Track C — `chat` + `chat_history` + `chat_conversation` + `notifications`

**Pages**: `chat.rs`, `chat_history.rs`, `chat_conversation.rs`,
`notifications.rs`

**Why these together**: the chat/notification surface is a
conversation-threaded UI with similar patterns (list + detail panel,
unread counter, push-notification support). Worker builds the chat
inbox shell and reuses parts of it for notifications.

**`chat.rs` target sections** (current 103 → 400+ LoC):
- `ChatInbox` — left list of conversations with unread badge,
  search, "New chat" button — port from `chat-inbox.tsx` 414 LoC.
- `ChatPanel` — right conversation panel; uses `chat-panel.tsx` 253
  LoC + `chat-message-item.tsx` 131 LoC.
- `ChatInput` — message composer with attach/emoji/voice — port
  from `chat-input.tsx` 177 LoC.
- `ChatTopicSelector` — modal to pick chat topic/tags — port from
  `chat-topic-selector.tsx` 233 LoC.

**`chat_history.rs` target sections** (current 30 → 100 LoC):
- Static list of past conversations with filter by date — match
  the `app/chat/history/page.tsx` 169 LoC structure.

**`chat_conversation.rs` target sections** (current 38 → 200 LoC):
- Reuse `ChatPanel` from `chat.rs` — but `chat_conversation.rs` is
  the dynamic route page that loads the conversation by ID. The
  pattern is "load + render via shared `ChatPanel` component".
- If `chat.rs` is the inbox shell (list + empty state when no chat
  selected) and `chat_conversation.rs` is the [id] route (single
  conversation), they're complementary. Both should exist.

**`notifications.rs` target sections** (current 105 → 350+ LoC):
- `NotificationList` — paginated list with type icons, read/unread
  state — port from `notification-bell-client.tsx` 266 LoC.
- `BrowserNotificationsPrompt` — "Allow browser notifications" CTA
  with permission status — port from
  `browser-notifications.tsx` 152 LoC.
- `NotificationSettings` — per-type toggle (news, payment, chat,
  system) — port from `notification-settings-panel.tsx` 110 LoC.

**Constraint**: this track DOES need a new primitive
`shared/rust/dioxus_ui/src/chat/message_bubble.rs` (the
`chat-message-item.tsx` is a generic pattern used in
`chat-conversation-view.tsx` admin too). Add it under the
`chat/` subdir. Track B/C CSS markers in the templates file if any
new CSS; otherwise no CSS region.

### Track D — `payment` + `payment_detail` + `permissions` + `portfolio` + `profile` + `news` + `news_detail`

**Pages**: `payment.rs` (and dynamic detail), `permissions.rs`,
`portfolio.rs`, `profile.rs`, `news.rs`, `news_detail.rs`

**Why these together**: the catch-all "smaller pages" track. Each
of these is <500 LoC of source. Combining them lets one worker do
all the small-stuff in one session without idle time, and keeps
the per-track worker at <30 min.

**`payment.rs` target sections** (current 114 → 500+ LoC):
- `PaymentFlowSteps` — 4-step wizard (choose plan → enter details →
  confirm → success) — port from `payment-flow-steps.tsx` 783 LoC
  (this is the largest single sub-component in the wave).
- `PlanComparisonCard` — 3-tier pricing comparison — port from
  `plan-comparison-card.tsx` 525 LoC.
- `CurrentAccessCard` — what the user currently has access to — port
  from `current-access-card.tsx` 214 LoC.
- `ChainVerificationCard` — wallet/chain compatibility check — port
  from `chain-verification-card.tsx` 427 LoC.
- `UpgradeBanner` — prompt to upgrade if over quota — port from
  `upgrade-banner.tsx` 181 LoC.
- `UnifiedPaymentFlow` — the wrapper that ties it all together —
  port from `unified-payment-flow.tsx` 252 LoC.
- **NEW** `PaymentDetailPanel` — handles the `[type]/[id]` dynamic
  route (186 LoC in source). When the URL has a type/id, render a
  detail view; otherwise render the wizard. The current `payment.rs`
  has no awareness of params — wire it via `ctx.params`.

**`permissions.rs` target sections** (current 93 → 250+ LoC):
- `PermissionsMatrix` — table of feature × plan — port the matrix
  grid from `permissions-display.tsx` 110 LoC.
- `FeatureList` — bulleted list of permissions per current plan.
- `RequestAccessCTA` — button to request additional permissions.

**`portfolio.rs` target sections** (current 154 → 350+ LoC):
- `WatchlistTable` — list of watched assets — port from
  `watchlist-provider.tsx` 113 LoC.
- `AddToWatchlistForm` — input to add an asset.
- `PerformanceChart` — line chart of portfolio value over time.

**`profile.rs` target sections** (current 108 → 500+ LoC):
- `Web3Integration` — wallet connection + signature — port from
  `web3-integration.tsx` 226 LoC.
- `EmailManagement` — change email, verify, magic-link resend — port
  from `email-management.tsx` 350 LoC.
- `DataManagement` — export data, delete account — port from
  `data-management.tsx` 384 LoC.
- `WalletProfile` — wallet info, ENS, balances — port from
  `wallet-profile-client.tsx` 233 LoC.

**`news.rs` target sections** (current 79 → 250+ LoC):
- `NewsList` — grid of news cards — port from `news-list.tsx` 185
  LoC.
- `NewsFilters` — category, date range, search.

**`news_detail.rs` target sections** (current 50 → 150+ LoC):
- `NewsDetailBody` — article body — port from `news-detail.tsx`
  148 LoC.
- `RelatedNewsList` — 3 related articles.

**Constraint**: this track DOES need a new primitive
`shared/rust/dioxus_ui/src/feedback/empty_chart_state.rs` (the
"no data yet" state for the performance chart) — add it. No CSS
regions expected.

### Integration gate (single track)

The integration agent:
1. Merges all 4 track branches into `wave6/integration` (no-ff).
2. Resolves any conflicts (CSS regions concatenate; page files
   shouldn't conflict because each track owns its own pages).
3. Runs the full cargo gate:
   - `cargo check --workspace --all-targets` — 0 errors expected.
   - `cargo build --workspace --bins` — 0 errors expected.
   - `cargo test -p epsx-dioxus-ui --lib` — 100+ tests expected
     (current 72 + ~30 new section-marker tests from Wave 6 tracks).
4. BFF-smokes all 16 auth-required routes on the frontend BFF
   (port 3000): hit each `/dashboard`, `/account`, etc. and assert
   HTTP 200 + the section markers in the rendered HTML.
5. Fast-forwards `migration/dioxus-microservices` to the integration
   tip.
6. Writes the final deliverable + pushes.

## What Wave 6A explicitly does NOT cover

- **Admin pages** (27 routes, 22,000 LoC of sub-components) →
  Wave 6B, separate plan.
- **Component-level 1:1 parity** (every Next.js sub-component as
  its own `#[component] fn`) → Wave 6C if you want it.
- **Real Playwright visual diff** against the Next.js originals →
  separate infra project.
- **Backend services extraction** → Wave 7.

## Source-of-truth pre-flight (read first)

The Wave 2 lesson (memory: "Bake the doc content into the task
prompts themselves — verbose but bulletproof") says don't trust
the design doc alone. Each track worker should:

1. `wc -l apps-old/frontend/app/{slug}/page.tsx apps-old/frontend/components/{slug}/*.tsx`
2. Read the source TSX end-to-end **before** touching port code.
3. Confirm the section list in this doc matches what the source
   actually has. If the source has 5 sections and the doc lists 6,
   the doc is wrong; port to the source. If the doc lists 5 and
   the source has 4, port the 4 and update the doc in the
   deliverable.
4. The unit-test section-marker assertion in each ported page is
   the contract. If a marker is missing, the test fails, the
   worker fixes it, the gate passes.

## Track worker constraints (carry-overs from Wave 5)

- Per-track workers do **not** run `cargo build --workspace --bins`
  (15-min link cost eats the 30-min cap). Per-track verify is
  `cargo check -p epsx-dioxus-ui --lib` + `cargo test -p epsx-dioxus-ui --lib`.
  Integration gate owns the full workspace build.
- Each track writes to its own `// === wave6-auth-pages-depth-track-X ===`
  block in `shared/rust/templates/src/lib.rs` for any CSS region
  additions.
- If a new shared primitive is needed (e.g. Track C's
  `chat/message_bubble.rs`), add it under the existing module
  hierarchy and `pub use` it from the parent `mod.rs` /
  `lib.rs`/`layout.rs` as appropriate. Document the new path in
  the deliverable so other tracks can import.
- Don't try to lower the worker's 30-min cap; budget the work to
  fit (Track D has 6 pages — worker may need to time-slice and
  admit 1-2 pages are partial if the source is huge).

## File ownership (no overlap)

| Track | Files owned | New files (allowed) |
| ----- | ----------- | ------------------- |
| A | `pages/dashboard.rs`, `pages/account.rs`, `pages/account_credits.rs` | none (uses existing primitives) |
| B | `pages/analytics.rs`, `pages/developer.rs` | `data/export_dialog.rs` (new primitive) |
| C | `pages/chat.rs`, `pages/chat_history.rs`, `pages/chat_conversation.rs`, `pages/notifications.rs` | `chat/message_bubble.rs` (new primitive) |
| D | `pages/payment.rs`, `pages/permissions.rs`, `pages/portfolio.rs`, `pages/profile.rs`, `pages/news.rs`, `pages/news_detail.rs` | `feedback/empty_chart_state.rs` (new primitive) |
| All | `shared/rust/templates/src/lib.rs` (CSS regions) | each track's own block |
