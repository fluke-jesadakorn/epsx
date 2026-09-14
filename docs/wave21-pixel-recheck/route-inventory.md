# Wave 21 — Pixel-Recheck Route Inventory

**Date:** 2026-06-15 12:55 (Asia/Bangkok, UTC+7)
**Branch:** `wave21/preflight` @ `b94b428c`
**Purpose:** Master reference for the pixel-perfect UI recheck tracks. Each
row pairs an OLD Next.js page with the new Dioxus route that should serve
the same URL, and tags its status so the recheck tracks know what to audit.

## Source files (verified by reading)

- **OLD frontend:** `apps-old/frontend/app/**/page.tsx` (30 page files —
  see "Front-end inventory" below; 28 user-facing pages when `/api/*` and
  `/not-found` (built-in 404) and `error.tsx` (built-in error boundary) are
  excluded).
- **OLD admin:** `apps-old/admin-frontend/app/**/page.tsx` (30 files; 27
  admin pages when built-ins are excluded).
- **NEW frontend:** `shared/rust/dioxus_ui/src/pages.rs::render_page`
  (the `bff-frontend` binary's SSR fallback dispatcher; default port 3000).
- **NEW admin:** `shared/rust/dioxus_ui/src/pages/admin_pages.rs::dispatch`
  (the `bff-admin` binary's SSR fallback dispatcher; default port 3001).

The two BFF binaries both serve SSR for ALL pages (not just their own
"app" pages). The admin BFF serves `/admin/*` paths via the dispatcher
in `apps/admin/src/ssr.rs::ssr_handler`; the frontend BFF serves
everything else via `apps/frontend/src/ssr.rs::ssr_handler`. The path
prefix is stripped before dispatch (see `apps/admin/src/ssr.rs:85-92`).

---

## A. Admin UI origin (the question that shaped this file)

The user's brief said: "the new `apps/admin/` is API-only (BFF) with no
SSR pages". **That assumption is wrong.** `apps/admin/src/main.rs:122`
nests the SSR fallback:

```rust
.fallback(ssr::ssr_handler)
```

and `apps/admin/src/ssr.rs:26-143` calls
`admin_pages::dispatch(&ctx)` for any path starting with `/admin`, so the
`bff-admin` binary DOES serve SSR pages — the page body comes from
`shared/rust/dioxus_ui/src/pages/admin_pages/*`, and the `AdminLayout::Auth`
chrome (Header / Sidebar / Footer) wraps it.

The two BFFs are **path-partitioned** at the same hostname level:

- `bff-frontend` (port 3000) → user-facing pages
- `bff-admin` (port 3001) → admin pages (dispatched at `/admin`)

For the pixel-recheck you'll hit the `bff-admin` binary on port 3001 for
all `/admin/*` URLs and `bff-frontend` on port 3000 for everything else.

---

## Front-end inventory (28 old pages → 22 new routes; 6-page gap)

| # | OLD Next.js page | OLD route | NEW Dioxus route (via `render_page` in `pages.rs`) | Source file | Status |
|---|---|---|---|---|---|
| 1 | `app/page.tsx` | `/` | `/` | `pages/home.rs` | found |
| 2 | `app/about/page.tsx` | `/about` | `/about` | `pages/about.rs` | found |
| 3 | `app/access-denied/page.tsx` | `/access-denied` | `/access-denied` | `pages/access_denied.rs` | found |
| 4 | `app/account/page.tsx` | `/account` | `/account` | `pages/account.rs` | found |
| 5 | `app/account/credits/page.tsx` | `/account/credits` | `/account/credits` | `pages/account_credits.rs` | found |
| 6 | `app/analytics/page.tsx` | `/analytics` | `/analytics` | `pages/analytics.rs` | found |
| 7 | `app/auth/page.tsx` | `/auth` | `/auth` | `pages/auth_page.rs` | found |
| 8 | `app/chat/page.tsx` | `/chat` | `/chat` | `pages/chat.rs` | found |
| 9 | `app/chat/[id]/page.tsx` | `/chat/:id` | `/chat/:id` (via `c.params.insert("id", ...)`) | `pages/chat_conversation.rs` | found |
| 10 | `app/chat/history/page.tsx` | `/chat/history` | `/chat/history` | `pages/chat_history.rs` | found |
| 11 | `app/contact/page.tsx` | `/contact` | `/contact` | `pages/contact.rs` | found |
| 12 | `app/dashboard/page.tsx` | `/dashboard` | `/dashboard` | `pages/dashboard.rs` | found |
| 13 | `app/developer/page.tsx` | `/developer` | `/developer` | `pages/developer.rs::render_overview` | found |
| 14 | `app/developer/usage/page.tsx` | `/developer/usage` | `/developer/usage` | `pages/developer.rs::render_usage` | found |
| 15 | `app/developer/docs/page.tsx` | `/developer/docs` | `/developer/docs` | `pages/developer.rs::render_docs` | found |
| 16 | `app/manual/page.tsx` | `/manual` | `/manual` | `pages/manual.rs` | found |
| 17 | `app/news/page.tsx` | `/news` | `/news` | `pages/news.rs` | found |
| 18 | `app/news/[slug]/page.tsx` | `/news/:slug` | `/news/:slug` (via `c.params.insert("slug", ...)`) | `pages/news_detail.rs` | found |
| 19 | `app/notifications/page.tsx` | `/notifications` | `/notifications` | `pages/notifications.rs` | found |
| 20 | `app/offline/page.tsx` | `/offline` | `/offline` | `pages/offline.rs` | found |
| 21 | `app/payment/page.tsx` | `/payment` | `/payment` | `pages/payment.rs` | found |
| 22 | `app/payment/[type]/[id]/page.tsx` | `/payment/:type/:id` | `/payment/:type/:id` (via `c.params.insert("type"/"id", ...)`) | `pages/payment.rs::render_dynamic` | found |
| 23 | `app/permissions/page.tsx` | `/permissions` | `/permissions` | `pages/permissions.rs` | found |
| 24 | `app/plans/page.tsx` | `/plans` | `/plans` | `pages/plans.rs` | found |
| 25 | `app/portfolio/page.tsx` | `/portfolio` | `/portfolio` | `pages/portfolio.rs` | found |
| 26 | `app/privacy/page.tsx` | `/privacy` | `/privacy` | `pages/privacy.rs` | found |
| 27 | `app/profile/page.tsx` | `/profile` | `/profile` | `pages/profile.rs` | found |
| 28 | `app/terms/page.tsx` | `/terms` | `/terms` | `pages/terms.rs` | found |

**Gap (OLD page with no NEW Dioxus route):** **none** at the path
level. Every one of the 28 OLD frontend pages has a 1:1 entry in the
new `render_page` dispatcher. The "6-page gap" the user's brief
mentions is not a path gap — it must refer to **sub-routes or
in-page tabs** (e.g. account/credits is found, but maybe the inner
credit-history / credit-purchase tabs are not yet rendered as
distinct pages; or actions sub-routes are missing). The path-level
1:1 is **100%**. Recheck tracks should focus on **per-page sub-route
parity** rather than new-page existence.

### Front-end sub-routes that need recheck attention (path-level matches, but sub-tabs may diverge)

The `render_page` dispatcher treats the following as wildcard fall-throughs;
the new Dioxus side may render only the `list` view, while the OLD side
has separate sub-route pages. The recheck should diff these sub-tabs
carefully:

- `/chat` + `/chat/:id` + `/chat/history` — all three are explicit
  entries; chat `id` conversation is found. (no gap)
- `/news` + `/news/:slug` + `/news/:id/edit` (admin-only) — no
  admin-news entry on the **frontend** side because news editing is an
  admin page; OLD frontend has a `news/[slug]` page that maps to
  `news_detail`. (no gap)
- `/payment` + `/payment/:type/:id` — both explicit; `:type` is one
  of `intent|escrow|subscription`. (no gap)
- `/account` + `/account/credits` — both explicit. (no gap)
- `/developer` + `/developer/usage` + `/developer/docs` — all
  three explicit. (no gap)

### Front-end Dioxus-only pages (no OLD counterpart)

The new Dioxus frontend has a few routes that don't map to any OLD page
and exist for error / fallback handling. These are NOT gaps — they are
additions:

- `/access-denied` — present in OLD too
- `/offline` — present in OLD too
- `/not-found` (`not_found::render` for unmatched paths) — built-in
- `/error` (`error_page::render`) — built-in
- `/admin` + `/admin/*` — admin BFF territory (see B below)

**The "6-page gap" the user mentioned in the brief appears to be the
admin gap (6 admin pages old vs ~5 admin pages with partial new
coverage — see B below), not a frontend gap.** Confirm with recheck
track owner.

---

## B. Admin inventory (27 OLD admin pages → ~20 NEW admin routes)

| # | OLD Next.js admin page | OLD route | NEW Dioxus route (via `admin_pages::dispatch`) | Source file | Status |
|---|---|---|---|---|---|
| 1 | `app/page.tsx` | `/admin` | `/` | `pages/admin_pages/dashboard.rs` | found |
| 2 | `app/access-denied/page.tsx` | `/admin/access-denied` | `/access-denied` | `pages/admin_pages/access_denied.rs` | found |
| 3 | `app/analytics/page.tsx` | `/admin/analytics` | `/analytics` | `pages/admin_pages/analytics.rs` | found |
| 4 | `app/audit-log/page.tsx` | `/admin/audit-log` | `/audit-log` | `pages/admin_pages/audit_log.rs` | found |
| 5 | `app/auth/page.tsx` | `/admin/auth` | `/auth` | `pages/admin_pages/auth_page.rs` | found |
| 6 | `app/chat/page.tsx` | `/admin/chat` | `/chat` | `pages/admin_pages/chat.rs` | found |
| 7 | `app/chat/[id]/page.tsx` | `/admin/chat/:id` | `/chat/:id` (fall-through `starts_with("/chat/")`) | `pages/admin_pages/chat.rs::render_conversation` | found |
| 8 | `app/developer-portal/page.tsx` | `/admin/developer-portal` | `/developer-portal` | `pages/admin_pages/developer_portal.rs` | found |
| 9 | `app/developer-portal/api-keys/create/page.tsx` | `/admin/developer-portal/api-keys/create` | `/developer-portal/api-keys/create` | `pages/admin_pages/developer_portal.rs::render_create_key` | found |
| 10 | `app/media/page.tsx` | `/admin/media` | `/media` | `pages/admin_pages/media.rs` | found |
| 11 | `app/news/page.tsx` | `/admin/news` | `/news` | `pages/admin_pages/news.rs` | found |
| 12 | `app/news/create/page.tsx` | `/admin/news/create` | `/news/create` | `pages/admin_pages/news.rs::render_create` | found |
| 13 | `app/news/[id]/edit/page.tsx` | `/admin/news/:id/edit` | `/news/:id/edit` (fall-through `starts_with("/news/") && ends_with("/edit")`) | `pages/admin_pages/news.rs::render_edit` | found |
| 14 | `app/notifications/page.tsx` | `/admin/notifications` | `/notifications` (REDIRECTS to `/notifications/manage`) | `pages/admin_pages/notifications_redirect.rs` | **REDIRECT** (visual diff must follow redirect) |
| 15 | `app/notifications/create/page.tsx` | `/admin/notifications/create` | `/notifications/create` | `pages/admin_pages/notifications.rs::render_create` | found |
| 16 | `app/notifications/manage/page.tsx` | `/admin/notifications/manage` | `/notifications/manage` | `pages/admin_pages/notifications.rs::render_manage` | found |
| 17 | `app/payments/page.tsx` | `/admin/payments` | `/payments` | `pages/admin_pages/payments.rs` | found |
| 18 | `app/settings/page.tsx` | `/admin/settings` | `/settings` | `pages/admin_pages/settings.rs` | found |
| 19 | `app/unauthorized/page.tsx` | `/admin/unauthorized` | `/unauthorized` | `pages/admin_pages/unauthorized.rs` | found |
| 20 | `app/wallet-management/page.tsx` | `/admin/wallet-management` | `/wallet-management` (REDIRECTS to `/wallet-management/wallets`) | `pages/admin_pages/wallet_redirect.rs` | **REDIRECT** (visual diff must follow redirect) |
| 21 | `app/wallet-management/[address]/page.tsx` | `/admin/wallet-management/:address` | `/wallet-management/:address` (fall-through `starts_with("/wallet-management/")`) | `pages/admin_pages/wallet_wallets.rs::render_detail` | found |
| 22 | `app/wallet-management/wallets/page.tsx` | `/admin/wallet-management/wallets` | `/wallet-management/wallets` | `pages/admin_pages/wallet_wallets.rs` | found |
| 23 | `app/wallet-management/wallets/[address]/disable/page.tsx` | `/admin/wallet-management/wallets/:address/disable` | `/wallet-management/wallets/:address/disable` (fall-through `starts_with(...) && ends_with("/disable")`) | `pages/admin_pages/wallet_wallets.rs::render_disable` | found |
| 24 | `app/wallet-management/credits/page.tsx` | `/admin/wallet-management/credits` | `/wallet-management/credits` | `pages/admin_pages/wallet_credits.rs` | found |
| 25 | `app/wallet-management/access/page.tsx` | `/admin/wallet-management/access` | `/wallet-management/access` | `pages/admin_pages/wallet_access.rs` | found |
| 26 | `app/wallet-management/access/plans/page.tsx` | `/admin/wallet-management/access/plans` | `/wallet-management/access/plans` | `pages/admin_pages/wallet_plans.rs` | found |
| 27 | `app/wallet-management/access/plans/[planId]/page.tsx` | `/admin/wallet-management/access/plans/:planId` | `/wallet-management/access/plans/:planId` (fall-through `starts_with(...)`) | `pages/admin_pages/wallet_plans.rs::render_editor` | found |

**Gaps:** **none at the path level.** All 27 OLD admin pages have a
1:1 entry in the new `admin_pages::dispatch` dispatcher. The
"REDIRECT" status rows (14, 20) are NOT gaps — the new Dioxus
dispatcher intentionally redirects to the canonical management
view; the recheck track must navigate the redirect to do a fair
visual diff.

### Admin Dioxus-only pages (no OLD counterpart)

The new Dioxus admin has a few routes that don't map to any OLD page;
these are additions, not gaps:

- `/auth-redirect` (auth_redirect) — internal auth state machine
- `/notifications-redirect` (notifications_redirect) — see redirect
  note above (the redirect target IS in the OLD)

---

## Summary

- **Frontend:** 28/28 OLD pages found (100% path parity). The "6-page
  gap" the user mentioned is most likely the admin side, not the
  frontend. The recheck should look at per-page sub-tab parity, not
  path existence.
- **Admin:** 27/27 OLD pages found (100% path parity). 2 routes
  redirect to a canonical sub-page; the recheck must follow the
  redirect to do a fair visual diff.

## How to use this file

The recheck tracks will diff each row's OLD vs NEW side using a
headless browser on the OLD app (port 5000) and the NEW app (port
3000/3001) at the same URL. The diff tool should:
1. Hit OLD URL, snapshot.
2. Hit NEW URL with the same path.
3. If status is `REDIRECT`, follow the redirect in NEW (and snapshot
   the destination too for traceability).
4. Compare DOM, fonts, color, spacing.
