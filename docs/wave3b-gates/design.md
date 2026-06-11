# Wave 3b — Per-Page Auth Gates, AccessDenied, ProgressiveAuthBanner

Shared design doc for the Wave 3b team plan. Read this end-to-end before
touching any page file. Wave 3a laid the layout foundation (chrome
rendered once per app, wallet state plumbed BFF → page); Wave 3b fills
in the per-page auth story that Wave 2/3a left as bare stubs.

## Goal

The four Rust auth primitives exist and work end-to-end:

| Component | Path | Purpose |
| --- | --- | --- |
| `AuthGate` | `shared/rust/dioxus_ui/src/auth/auth_gate.rs` | Renders `children` only if `user.is_some()` (or `required_permissions` is satisfied). |
| `AdminAuthGate` | same file | Same, but ALSO fires when `!user.is_admin()`. Renders the "Admin access required" panel. |
| `AccessDenied` | `shared/rust/dioxus_ui/src/auth/access_denied.rs` | Standalone "Permission required" page (used by `/access-denied` and as a body for forbidden routes). |
| `ProgressiveAuthBanner` | `shared/rust/dioxus_ui/src/auth/progressive_banner.rs` | Inline "sign in to unlock" strip shown above the page body on free pages. |

The Wave 2/3a port calls them, but **stubbed**: no `required_permissions`,
no `return_url`, admin pages use plain `AuthGate` (not `AdminAuthGate`),
and `ProgressiveAuthBanner` is never used at all. Wave 3b fills those
gaps so that the auth story matches the TS source behaviour for the
same routes:

- A signed-out user hitting `/account` lands on a `Sign in required`
  panel with a "Connect Wallet" link that round-trips back to `/account`
  on success (`?next=/account`).
- A signed-in but non-admin user hitting `/admin/audit-log` lands on an
  "Admin access required" panel (the **admin** variant, not the user
  variant) with a "Connect Admin Wallet" link.
- A signed-in user without `plans:subscribe` hitting `/plans` lands on
  a "Permission required" panel that lists the missing permission.
- A signed-out user on `/` (or `/about`, `/news`, etc.) sees a
  `ProgressiveAuthBanner` strip at the top of the page inviting them
  to connect — the page body still renders (browsing works without
  signing in).

## What this wave does NOT do (deferred)

- Layout-level auth gating (the TS `AuthLayout` pattern). The Rust port
  uses page-level `<AuthGate>` wrappers and Wave 3b keeps that
  convention; a future "auth refactor" wave could promote to layout
  level. Out of scope here.
- The `from_cookies` cookie-parser follow-up (real `WalletInfo` cookie
  read). Wave 3a left a stub; the auth gate does not depend on it
  (gates on `user`, not on `wallet`).
- The 4th main-track "design system polish" wave (rename
  `MainLayout`/`AuthLayout` collision, consolidate CSS markers). Not
  auth-related.
- Removing the old shadcn TS code in `apps-old/*`.
- New auth UX (sign-in flows, OAuth callback pages) — already
  present from Wave 2 Track C.

## Source of truth

| What | Where |
| --- | --- |
| Auth gate components | `shared/rust/dioxus_ui/src/auth/{auth_gate,access_denied,progressive_banner}.rs` |
| Pages that consume them | `shared/rust/dioxus_ui/src/pages/*.rs`, `shared/rust/dioxus_ui/src/pages/admin_pages/*.rs` |
| PageContext (BFF → page) | `shared/rust/dioxus_ui/src/pages.rs` |
| TS reference (frontend gate UX) | `apps-old/frontend/components/auth/{frontend-auth-gate,access-denied}.tsx` |
| TS reference (admin gate UX) | `apps-old/admin-frontend/components/auth/admin-auth-gate.tsx`, `apps-old/admin-frontend/components/layout/auth-layout.tsx` |
| TS reference (banner UX) | `apps-old/frontend/components/auth/auth-banner.tsx` (or whatever the TS ProgressiveAuthBanner is named) |

## Hard conventions — every track must follow

These are non-negotiable. The verifier will FAIL any track that
violates them.

### 1. Permission-string schema

Use the `domain:action` pattern, matching the existing precedent in
`pages/admin_pages/unauthorized.rs` (`"admin:*"`) and the TS
`access-denied` page (`${route}:access`). Conventions:

- `plans:subscribe` — `/plans`
- `payments:read` — `/account/credits`, `/payment`, `/portfolio`
- `chat:read` — `/chat`, `/chat/history`, `/chat/[id]`
- `chat:write` — `/chat/[id]` (send/reply)
- `analytics:read` — `/analytics` (frontend), `/admin/analytics`
- `notifications:read` — `/notifications`
- `profile:read` / `profile:write` — `/profile`, `/account`
- `audit:read` — `/admin/audit-log`
- `payments:manage` — `/admin/payments`
- `news:manage` — `/admin/news`, `/admin/news/create`
- `notifications:manage` — `/admin/notifications`, `/admin/notifications/create`
- `media:manage` — `/admin/media`
- `wallets:manage` — `/admin/wallet-management/*`
- `settings:manage` — `/admin/settings`
- `developer:read` — `/developer/*`
- `admin:*` — wildcard for any admin route (used by `unauthorized.rs`,
  can be used as a coarse fallback when a track can't decide the
  specific permission)

If a page already has a permission string in the TS source, prefer
that. If not, pick from the list above. Document the choice in the
deliverable.

### 2. Gate variant choice (admin vs user)

- **Admin pages** (everything in `pages/admin_pages/*`) MUST use
  `AdminAuthGate` (not `AuthGate`). The admin variant also fires
  when `!user.is_admin()` and renders the "Admin" pill.
- **Frontend user pages** (everything in `pages/*` that needs auth)
  MUST use `AuthGate`. Do NOT use `AdminAuthGate` on the frontend.
- The `/access-denied` and `/unauthorized` pages are the *output* of
  the gate, not a gated page themselves. They render
  `<AccessDenied>` unconditionally with the relevant `reason` and
  `required_permissions`.

### 3. `return_url` always set on gated pages

Every `<AuthGate>` / `<AdminAuthGate>` on a gated page MUST pass
`return_url={Some(ctx.path.clone())}` so the SIWE flow can bounce the
user back to the original destination post-login. The path is already
on `PageContext::path` (set by the BFF).

Exception: `/auth` itself (already full-bleed; the gate's
`return_url` is meaningless for the auth route).

### 4. Public API stability

Do NOT add, rename, or remove any prop on `AuthGate`, `AdminAuthGate`,
`AccessDenied`, or `ProgressiveAuthBanner`. They are complete from
Wave 2 Track C. Wave 3b only CALLS them with richer arguments. The
verifier will FAIL any track that touches the component signatures.

If a track discovers that a callsite needs a prop that doesn't exist
(stop and think first — they all exist), disclose it in the
deliverable and ask in the integration gate. Do NOT silently extend
the components.

### 5. `AccessDenied` is a page body, not a wrapper

`AccessDenied` is the *whole page body* on `/access-denied` and
`/unauthorized`. It is NOT a gate. It renders an "Access denied"
panel with the `reason` text and a `required_permissions` list.

`AuthGate` and `AccessDenied` solve different problems:
- `AuthGate` is a *gating* component: hides children when the user
  doesn't have access. Used inside page bodies.
- `AccessDenied` is a *destination* component: shown when the user
  navigates to a page that explicitly says "this is a 403". Used as
  the page body on `/access-denied` and `/unauthorized`.

A page either uses `<AuthGate>` to gate its body, OR renders
`<AccessDenied>` as the body. Never both. Never `<AccessDenied>`
inside `<AuthGate>` (the gate would block access to the AccessDenied
body, which is the opposite of what's wanted).

### 6. `ProgressiveAuthBanner` placement

`ProgressiveAuthBanner` is a horizontal strip rendered *above* the
page body (not inside the auth gate). It shows on free pages when
the user is signed out, inviting them to connect. When the user IS
signed in, the banner is hidden.

Use it on:
- `/` (home)
- `/about`
- `/news` and `/news/[slug]`
- `/contact`
- `/plans` (public browse; the gate fires only on Subscribe click)
- `/developer` (overview only — usage/docs are gated)

Do NOT use it on:
- `/auth` (full-bleed)
- Any page already inside an `<AuthGate>` (the gate handles the
  signed-out state)
- Any admin page

### 7. CSS region markers

Each track owns a unique CSS region in
`shared/rust/templates/src/lib.rs`. Use the comment marker:

```
// === wave3b-gates-track-X ===
```

where `X` is `a` / `b` / `c`. The integration gate will concatenate
the three blocks. **Do not reformat existing CSS** — only append
within your marker.

(If a track needs NO new CSS, do not write a marker block; the
integration gate will handle a missing block as "no work in that
region".)

### 8. Worktree discipline

Create your worktree at `/private/tmp/epsx-track3b-X-*` BEFORE
editing any file. Commit and stay there. The plan owner handles the
cross-worktree merge.

```
# Example for Track A
git worktree add /private/tmp/epsx-track3b-a-user-gates origin/migration/dioxus-microservices -b wave3b/track-a-user-gates
cd /private/tmp/epsx-track3b-a-user-gates
# ... edit, commit ...
git push -u origin wave3b/track-a-user-gates
```

The main checkout, the Wave 1/2/3a worktrees, and the
`feature-dioxus-move` worktree MUST NOT be touched. Do NOT `git
checkout` a new branch inside the main checkout.

### 9. Test additions

Each track adds at least **one** new unit test. Suggestion:
- Track A: render an authenticated-context `/account` and assert the
  page body is present (not the gate panel). Use
  `dioxus_ssr::render_element` per the Wave 3a Track A pattern.
- Track B: render an unauthenticated-context `/admin/audit-log` and
  assert the `auth-gate-admin` class is present. Then render an
  authenticated non-admin context and assert the same.
- Track C: render `/` with `user = None` and assert
  `progressive-auth-banner` is present. Then render `/` with
  `user = Some(...)` and assert it's absent.

The verifier will run the full `cargo test -p epsx_dioxus_ui` suite
(13 existing + 3 new = 16 expected).

## Track split

This wave has **3 parallel coder tracks + an integration gate**.
File scopes are disjoint.

### Track A — Frontend user-page gate enrichment

Scope: 12 frontend pages in `shared/rust/dioxus_ui/src/pages/*.rs`:
- `account.rs`, `account_credits.rs`
- `profile.rs`
- `dashboard.rs`
- `portfolio.rs`
- `payment.rs`
- `notifications.rs`
- `analytics.rs`
- `permissions.rs`
- `chat.rs`, `chat_history.rs`, `chat_conversation.rs`

For each:
- Switch the existing `<AuthGate>` call to:
  - `user: ctx.user.clone()` (unchanged)
  - `feature: Some(<feature name>)` (unchanged)
  - `required_permissions: Some(vec!["<perm>".to_string()])` per the
    schema in §1
  - `return_url: Some(ctx.path.clone())` (NEW)
- If a page does NOT currently have `<AuthGate>`, add one (the TS
  source is the reference for whether a page is gated).
- 1 new unit test in `shared/rust/dioxus_ui/src/pages/account.rs` (or
  a new `tests/gates.rs`): render authenticated → body present,
  unauthenticated → gate panel present.

Out of scope: admin pages, free pages (Track C), the
`ProgressiveAuthBanner` component itself.

### Track B — Admin page gate enrichment

Scope: 18 admin pages in
`shared/rust/dioxus_ui/src/pages/admin_pages/*.rs`:
- `dashboard.rs`, `audit.rs`, `payments.rs`, `analytics.rs`,
  `chat.rs`, `news.rs`, `notifications.rs`, `settings.rs`,
  `developer_portal.rs`, `media.rs`, `policies.rs`,
  `wallet_access.rs`, `wallet_credits.rs`, `wallet_plans.rs`,
  `wallet_wallets.rs`, `wallet_redirect.rs`, `notifications_redirect.rs`,
  `auth_redirect.rs` (where applicable — exclude pure-redirect pages
  that don't render a body)

For each:
- Switch the existing `<AuthGate>` call to `<AdminAuthGate>` (NEW).
  - `user: ctx.user.clone()` (unchanged)
  - `feature: Some(<feature name>)` (unchanged)
  - `required_permissions: Some(vec!["<admin perm>".to_string()])`
    per §1 (or `"admin:*".to_string()` for the coarse fallback)
  - `return_url: Some(ctx.path.clone())` (NEW)
- The `unauthorized.rs` and `access_denied.rs` pages render
  `<AccessDenied>` as the body — DO NOT wrap them in any gate. They
  are the destination of the gate, not gated themselves. Track B
  leaves them alone (they already work from Wave 2 Track C).
- 1 new unit test: render an admin page with
  `user = Some(admin_user)` and assert the body renders. Then with
  `user = Some(non_admin_user)` and assert the admin gate panel
  (`auth-gate-admin` class) renders.

Out of scope: frontend pages, free pages, the
`ProgressiveAuthBanner` component.

### Track C — ProgressiveAuthBanner on free pages

Scope: 5 free pages in `shared/rust/dioxus_ui/src/pages/*.rs`:
- `home.rs`
- `about.rs`
- `news.rs` (the list — `news_detail.rs` is per-article, see below)
- `contact.rs`
- `plans.rs` (public browse)
- `news_detail.rs` (per-article — banner is also appropriate here)

For each:
- Add a `ProgressiveAuthBanner` element ABOVE the existing page
  body (inside the `MainLayout`, before the page content). The
  banner only renders meaningfully when `ctx.user.is_none()`;
  when the user IS signed in, hide it with a conditional:
  ```
  if ctx.user.is_none() {
      ProgressiveAuthBanner { ... }
  }
  ```
  Use a sensible `feature` and `description` per page (e.g. for
  `/plans` → `feature: Some("plan subscription")`, for `/` → omit
  feature to use the default copy).
- 1 new unit test: render `/` with `user = None` and assert
  `progressive-auth-banner` is in the HTML. Then with
  `user = Some(...)` and assert it's absent.

Out of scope: gated pages, the `ProgressiveAuthBanner` component
itself (no API changes), admin pages.

### Integration gate

After all 3 tracks pass:
1. Create a worktree `/private/tmp/epsx-wave3b-integration` based on
   `origin/migration/dioxus-microservices`.
2. Merge each track branch with `--no-ff`.
3. Run `cargo check -p epsx_dioxus_ui -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`.
4. Run `cargo test -p epsx_dioxus_ui` — all existing 13 unit tests
   must still pass; the new tests from Track A + B + C must pass
   (16 total expected).
5. Smoke-test all 4 BFFs on ports 14000-14003. Verify each returns
   HTTP 200. Curl a signed-out `/account` request — the gate panel
   (`auth-gate`) MUST be in the HTML. Curl `/` — the
   `progressive-auth-banner` MUST be in the HTML. Curl
   `/admin/audit-log` — the admin gate (`auth-gate-admin`) MUST be
   in the HTML.
6. Commit integration report + push to
   `origin/migration/dioxus-microservices`.

If any track has conflicts, the integration agent resolves them by
preferring the track's stated prop additions (all additive). If a
track accidentally breaks a public API, that track must fix it
before integration.

## Risks & known sharp edges

- **Permission-string proliferation**: 12+ permission strings appear
  for the first time. The schema in §1 is the contract. If a track
  needs a string not in §1, ADD it to the schema in the deliverable
  and explain why. Do NOT invent a new shape (e.g. `domain/action`).
- **Wave 2 `unauthorized.rs` precedent**: it already uses
  `"admin:*"`. Track B may use the same wildcard for coarse-grained
  pages where the specific permission isn't obvious from the TS
  source. The verifier will accept this.
- **`<AccessDenied>` and `<AuthGate>` confusion**: see §5. The
  integration agent and verifier will check this explicitly.
- **`return_url` on `/auth`**: not meaningful, but the gate's
  default connect-href is `/auth` — leaving `return_url` unset on
  `/auth` is correct (it's the auth route, not a gated one).
- **Public API stability**: §4 is a hard rule. Wave 1 + Wave 2
  established this. Track workers who are tempted to "tidy up" the
  gate component signatures must NOT. Add new prop values, not new
  props.
- **SSR safety**: all gate components are pure render — no
  `use_signal`, no client-only state. The BFF can call them
  unconditionally. (Verified in Wave 2 Track C; Wave 3a re-verified.)
- **Test counts**: the existing 13 unit tests (9 Wave 1+2 + 3 Wave 3a
  + 1 stub) MUST all still pass after Wave 3b. The verifier runs
  the full `cargo test -p epsx_dioxus_ui` suite.

## Push cadence

1. Coder writes code + commits in their worktree.
2. Coder writes `deliverable.md` with: files changed, public API
   delta (should be empty for all 3 tracks), test additions,
   permission-string choices per page, any out-of-scope flags,
   branch name + final HEAD hash.
3. Coder pushes branch to `origin/wave3b/track-X-*`.
4. Coder returns the deliverable text.
5. Verifier independently re-derives `cargo check` + new tests +
   smoke-curl on the unauthenticated paths.
6. Plan owner reviews, accepts or retries, integration gate runs.
