# Wave 3a — Global Wiring: Nav + Connect Button in Shell

Shared design doc for the Wave 3a team plan. Read this end-to-end before
touching any page or shell file.

## Goal

Replace the existing per-page `<Navbar>` call sites (and per-page
`<Footer>`) with the Wave 2 chrome cluster
(`NavigationClient` / `MobileNav` / `SignInBanner`) so that:

- **The frontend nav is rendered once per app, not once per page.**
  All 30+ `apps/frontend/src/pages/*.rs` files stop calling
  `<Navbar { user, current_path, on_theme_toggle } />` and instead
  receive the chrome via a layout-level wrapper.
- **The wallet/auth state that the Wave 2 chrome needs is plumbed from
  the BFF into the page render path.** No `wagmi`/React-context
  equivalents — the cluster takes state as props, so the BFF reads
  the existing session and forwards it.
- **The 4 admin BFFs render the admin shell cluster
  (`AdminLayout::Auth` → `Header` + `Sidebar` + `Footer`)** — this is
  already the shape from Wave 2 Track A; Wave 3a just makes the admin
  pages actually consume it (they currently each call
  `DashboardShell` directly).
- **`ConnectButton` + `ConnectedWalletDropdown` are the canonical wallet
  entry point** in the navbar, replacing any remaining ad-hoc wallet
  buttons in the Wave 2 chrome's "actions" slot.

What this wave does **NOT** do (deferred to Wave 3b):

- Per-page `AuthGate` / `AdminAuthGate` / `ProgressiveAuthBanner` /
  `AccessDenied` rewiring. Those are inside page bodies, not the shell.
- New auth UX (sign-in flows, OAuth callback pages). The auth modal
  already exists from Wave 2 Track C; this wave only plumbs the entry
  point.
- Removing the old shadcn TS code in `apps-old/*`.

## Source of truth

| What | Where |
| --- | --- |
| Wave 2 frontend nav cluster | `shared/rust/dioxus_ui/src/layout/{navbar,navbar_skeleton,nav_actions,nav_config,mobile_nav}.rs` |
| Wave 2 admin shell cluster | `shared/rust/dioxus_ui/src/layout/{sidebar,header,breadcrumbs,shell,footer}.rs` |
| Wave 2 wallet button | `shared/rust/dioxus_ui/src/auth/wallet_button.rs` |
| Wave 2 auth entry point | `shared/rust/dioxus_ui/src/auth/auth_modal.rs` |
| Pages that consume the cluster | `shared/rust/dioxus_ui/src/pages/*.rs`, `shared/rust/dioxus_ui/src/pages/admin_pages/*.rs` |
| PageContext (BFF → page) | `shared/rust/dioxus_ui/src/pages.rs` |
| Frontend BFF render path | `apps/frontend/src/ssr.rs` (constructs `PageContext`, calls `render_page`) |
| Admin BFF render path | `apps/admin/src/ssr.rs` (same shape) |
| Pay + Preview BFFs | `apps/pay/src/ssr.rs`, `apps/preview/src/ssr.rs` (Pay uses a different shell — see Track A scope) |
| TS reference for frontend nav | `apps-old/frontend/components/nav/*` |
| TS reference for admin shell | `apps-old/admin-frontend/components/admin/*` |

## Hard conventions — every track must follow

These are non-negotiable. The verifier will FAIL any track that
violates them.

### 1. Layout ownership

**Frontend pages MUST NOT call `<Navbar>` or `<Footer>` directly after
this wave.** The navbar cluster is rendered by the layout. Pages
become body-only.

Exception list (still call `<Navbar>` directly):
- `pages/auth_page.rs` — the `/auth` route is full-bleed with no
  navbar; the cluster already short-circuits on `path == "/auth"`,
  but this page never enters the layout in the first place.
- `pages/admin_pages/*` — these are admin pages, served by the admin
  BFF, which uses a different layout (see below).

**Admin pages MUST NOT call `<DashboardShell>` directly after this
wave.** They render into the admin layout (`AdminLayout::Auth` →
`<Header> + <Sidebar> + page body + <AdminFooter>`). The layout
owns the shell chrome; pages own their content.

Exception list:
- `pages/admin_pages/access_denied.rs` — uses `<AccessDenied>` as the
  page body, but still inside the admin layout (it must still
  render the admin chrome, not bare error page).
- `pages/admin_pages/not_found.rs` (if it exists) — bare error
  variant; see Track C scope.

**Pay BFF (`apps/pay/src/ssr.rs`)** uses its own minimal layout
already (no Navbar, no admin shell). Wave 3a does NOT change Pay's
layout. Track A scope explicitly excludes Pay/preview.

### 2. Public API stability

**Do not rename or remove existing props on `Navbar`, `Footer`,
`PageHeader`, `DashboardShell`, `PageMeta`, `PageContext`.** Wave 1
established a stability rule and the BFFs depend on every signature.
Add new fields as `Option<T>` with `#[props(default)]` — never break
a callsite.

If a track discovers that a callsite MUST change (e.g. a prop's
meaning flipped), disclose it in the deliverable and ask in the
integration gate. Do NOT silently break the API.

### 3. CSS region markers

Each track owns a unique CSS region in
`shared/rust/templates/src/lib.rs`. Use the comment marker:

```
/* === wave3a-wiring-track-X === */
```

where `X` is `a` / `b` / `c`. The integration gate will concatenate
the three blocks. **Do not reformat existing CSS** — only append
within your marker.

### 4. Worktree discipline

Create your worktree at `/private/tmp/epsx-track3a-X-*` BEFORE
editing any file. Commit and stay there. The plan owner handles the
cross-worktree merge.

```
# Example for Track B
git worktree add /private/tmp/epsx-track3b-b-shell-nav origin/migration/dioxus-microservices -b wave3a/track-b-shell-nav
cd /private/tmp/epsx-track3b-b-shell-nav
# ... edit, commit ...
git push -u origin wave3a/track-b-shell-nav
```

The main checkout and the Wave 1/2 worktrees MUST NOT be touched.
Do NOT `git checkout` a new branch inside the main checkout.

### 5. BFF render path invariants

**The BFF must continue to produce a complete HTML document**
(`page_shell_with_body_class(...)` wraps the rendered body). Do NOT
move the navbar rendering INTO the BFF's render call — keep the
Dioxus `rsx!` boundary clean: the BFF constructs `PageContext`,
calls `render_page`, gets back a `PageMeta` + `Element`, renders
to HTML, wraps in the page shell.

**`PageContext` gains no required fields in this wave.** Add fields
as `Option<T>` with `Default::default()` (the struct already
`#[derive(Default)]`).

### 6. Wallet state plumbing (Track B + C, frontend only)

`ConnectButton` / `ConnectedWalletDropdown` need:
- `address: Option<String>` — short address (`0x1234…abcd`).
- `connector_id: Option<String>` — wallet provider id
  (e.g. `"metaMask"`).
- `is_authenticated: bool` — whether the user has a valid session.
- `chain_id: Option<u64>` — current chain id (56 mainnet, 97 testnet).
- `is_authenticating: bool` — whether a sign-in is in progress.

These map 1:1 to `ConnectedWalletState` in
`auth/wallet_button.rs` — Track B reuses that struct on
`PageContext`. Do NOT introduce a new type.

In Wave 2 the BFF has none of these — the wallet state is purely
client-side. For Wave 3a we add server-side stubs:

- `address` / `connector_id` — derived from a `WalletInfo` cookie
  that the wagmi-equivalent client writes when it connects. If
  absent, defaults to `None`. (Stub for now; Wave 3b or later
  waves wire the real cookie read.)
- `is_authenticated` — derived from the existing `User` (already
  on `PageContext`). Track B's BFF code sets
  `wallet.is_authenticated = ctx.user.is_some()`.

The verifier will check that **the BFF can be compiled and the page
renders with these stubs defaulting to `None`/`false`**, even if the
wallet never actually connects. SSR must not crash on missing
cookie.

## Track split

This wave has **3 parallel coder tracks + an integration gate**.
File scopes are disjoint.

### Track A — Frontend layout wrapper (`MainLayout`)

Scope:
- Create `shared/rust/dioxus_ui/src/layout/main_layout.rs` exporting:
  - `pub fn MainLayout(ctx: &PageContext, page_body: Element) -> Element`
    — renders the standard frontend chrome: `<NavigationClient>` (or
    `NavbarSkeleton` if `!ctx.is_hydrated`), then the page body, then
    `<Footer>`. Hides the chrome on `path == "/auth"`.
  - `pub fn AuthLayout(ctx: &PageContext, page_body: Element) -> Element`
    — full-bleed layout for `/auth`, no chrome.
- Update all 18 frontend pages in
  `shared/rust/dioxus_ui/src/pages/*.rs` to:
  - Remove the `use crate::layout::{Navbar, Footer}` import (keep
    other layout imports).
  - Remove the `<Navbar>` and `<Footer>` elements from their
    `rsx!` blocks.
  - Wrap their existing body in `MainLayout` (or `AuthLayout` for
    `auth_page.rs`).
- Add a `MainLayout` smoke test under
  `shared/rust/dioxus_ui/src/layout/main_layout.rs` (or a new
  `tests/main_layout.rs`) covering: (a) `/` renders `<header>` +
  body + `<footer>`, (b) `/auth` does not render the header, (c)
  body content is preserved.

Out of scope: admin pages, Pay, Preview, the `connect` button
wiring, BFF changes.

### Track B — BFF plumbing for wallet state

Scope:
- **Reuse** the existing `ConnectedWalletState` struct in
  `shared/rust/dioxus_ui/src/auth/wallet_button.rs` (Wave 2 Track
  C). Do NOT add a new `WalletState` — that's a duplicate of
  `ConnectedWalletState` minus a few fields.
- Add `pub wallet: ConnectedWalletState` to `PageContext` (with
  `#[derive(Default)]` — the struct already supports it).
- Update `apps/frontend/src/ssr.rs` to read a `WalletInfo` cookie
  (or default) and populate `PageContext::wallet`. Add a
  `// Stub: cookie read happens here` comment block.
- Add a `ConnectedWalletState::from_cookies(headers: &HeaderMap) -> Self`
  helper that returns `Default::default()` for now (the real read
  is Wave 3b+).
- Update `apps/admin/src/ssr.rs` and `apps/pay/src/ssr.rs` to
  construct the default `ConnectedWalletState` (admin doesn't
  render the wallet dropdown yet, pay doesn't render it at all).
- Add a unit test in `shared/rust/dioxus_ui/src/auth/wallet_button.rs`
  for `ConnectedWalletState::from_cookies(&HeaderMap::new())`
  returning all defaults.

Out of scope: page body changes, layout changes, anything in
`pages/*.rs`.

### Track C — Admin shell wiring + footer swap

Scope:
- Update all admin pages in
  `shared/rust/dioxus_ui/src/pages/admin_pages/*.rs` to:
  - Remove the `use crate::layout::DashboardShell` import.
  - Remove `<DashboardShell { ... }>` wrapper.
  - The page `rsx!` now returns just the page body content.
- Update `apps/admin/src/ssr.rs` to wrap the rendered body in
  `AdminLayout::Auth` (already exists in `shell.rs`) before calling
  `render_element` and `page_shell_with_body_class`.
- Add a smoke test verifying the admin dashboard page renders with
  `<header class="...admin...">` (the Header component) somewhere
  in the final HTML.
- The `apps/frontend/src/ssr.rs` path continues to call
  `render_page` directly (the layout swap is in Track A, not here).

Out of scope: WalletState, MainLayout, Pay/Preview BFFs.

### Integration gate

After all 3 tracks pass:
1. Create a worktree `/private/tmp/epsx-wave3a-integration` based on
   `origin/migration/dioxus-microservices`.
2. Merge each track branch with `--no-ff`.
3. Run `cargo check -p epsx_dioxus_ui -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`.
4. Run `cargo test -p epsx_dioxus_ui` — all existing 9 unit tests
   must still pass; the new tests from Track A + B must pass.
5. Smoke-test all 4 BFFs on ports 14000-14003 (or 3000/3001/8080/
   9180 if nothing's running). Verify each returns HTTP 200 with
   the expected `<title>`.
6. Commit integration report + push to
   `origin/migration/dioxus-microservices`.

If any track has conflicts, the integration agent resolves them by
preferring the track's stated public API additions. If a track
breaks an existing public API, that track must fix it before
integration.

## Risks & known sharp edges

- **Worktree contention**: 3 parallel coders on the same repo →
  each MUST use its own worktree. Hard rule.
- **Design doc visibility**: this doc IS on the base branch
  (`docs/wave3a-wiring/design.md`) and will be visible from every
  worker's worktree. If you can't see it, stop and ask — do NOT
  guess the convention.
- **Public API breakage**: Wave 1 + Wave 2 established a stability
  rule. Do not break it. Add new fields, don't rename or remove.
- **SSR / hydration**: the navbar cluster is hydration-aware
  (`is_hydrated` prop). For SSR, the BFF always passes
  `is_hydrated = true` (Dioxus SSR is hydration-less in the
  Next.js sense; the BFF gets the final HTML on first render).
- **No `wagmi` in Rust**: wallet state is plumbed as plain props.
  The client-side wallet connection logic remains on the
  TypeScript side; this wave only plumbs the SSR-side stub.

## Push cadence

1. Coder writes code + commits in their worktree.
2. Coder writes `deliverable.md` with: files changed, public API
   delta, test additions, any out-of-scope flags, branch name +
   final HEAD hash.
3. Coder pushes branch to `origin/wave3a/track-X-*`.
4. Coder returns the deliverable text.
5. Verifier independently re-derives `cargo check` + new tests +
   public API delta.
6. Plan owner reviews, accepts or retries, integration gate runs.
