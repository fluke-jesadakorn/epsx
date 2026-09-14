# Wave 2 — Dioxus UI Chrome Components Parity

Shared design doc for all three coder tracks in the Wave 2 team plan.
Read this end-to-end before touching any layout/auth file.

## Goal

Bring `shared/rust/dioxus_ui/src/layout/*` and
`shared/rust/dioxus_ui/src/auth/*` to **1:1 functional parity** with the
Next.js components in `apps-old/frontend/components/nav/`,
`apps-old/admin-frontend/components/layout/`, and the admin auth/gating
stack (`apps-old/admin-frontend/components/auth/`,
`apps-old/frontend/components/auth/`).

After Wave 1, the Dioxus primitives exist and compile. **Scaffolds** for
the layout and auth modules exist (~290 lines / ~206 lines) but are
**not full ports**: they are minimal Dioxus shapes that compile but do
not match the TS source for the cases that matter (auth state, nested
sidebar with expand/collapse, mobile nav sheet, wallet provider chip,
breadcrumb auto-generation, route-aware active states, etc.).

Wave 2's job: bring each chrome component to TS parity so the existing
pages can be migrated next.

## Source of truth

| What | Where |
| --- | --- |
| Rust layout (current — scaffolds) | `shared/rust/dioxus_ui/src/layout/*.rs` |
| Rust auth (current — scaffolds) | `shared/rust/dioxus_ui/src/auth/*.rs` |
| Frontend nav TS | `apps-old/frontend/components/nav/{desktop-nav,mobile-nav,nav-actions,nav-config,navbar-skeleton,navigation-client,wallet-provider-icon,index}.{ts,tsx}` |
| Admin layout TS | `apps-old/admin-frontend/components/layout/{sidebar,header,breadcrumb,main-layout,auth-layout,layout-wrapper,admin-notification-bell-client}.{ts,tsx}` |
| Admin auth TS | `apps-old/admin-frontend/components/auth/admin-auth-gate.tsx`, `admin-wallet-connect-auth.tsx` |
| Frontend auth TS | `apps-old/frontend/components/auth/*.tsx` |
| Primitives (consumed by chrome) | `shared/rust/dioxus_ui/src/primitives/*.rs` (Wave 1 output) |
| Pages (consumers) | `shared/rust/dioxus_ui/src/pages/*.rs` |
| Design system CSS | `shared/rust/templates/src/lib.rs` (epsx-templates) |
| Icon helper | `epsx_templates::lucide(name, size, class)` returns SVG string |

## Hard conventions — every chrome component must follow

These are non-negotiable. If a track violates them, the verifier will FAIL.

### 1. Component signature

```rust
use dioxus::prelude::*;

#[component]
pub fn Navbar(
    user: Option<User>,                        // domain types first
    current_path: Option<String>,              // then routing
    on_theme_toggle: Option<EventHandler<MouseEvent>>,  // then callbacks
    class_name: Option<String>,                // then class overrides
    id: Option<String>,
    children: Option<Element>,                 // children last
) -> Element { ... }
```

- `Option<T>` with `unwrap_or` defaults — never required booleans/strings.
- `class_name` (snake_case) is the public prop name, **not** `class`
  (Dioxus reserves `class` for the built-in attribute). Inside `rsx!`
  the attribute is still `class:`.
- `children: Element` is always the last parameter (or `Option<Element>`
  when the children are optional).
- Public structs (e.g. `NavItem`, `SidebarItem`, `Crumb`, `NotificationItem`)
  must be `Clone, Debug, PartialEq`.

### 2. State handling

- **Owned by the chrome component** when the data is purely UI
  (sidebar expand/collapse, mobile sheet open/closed, dropdown menus).
  Use `use_signal` internally. Expose `open: Option<bool>` /
  `default_open: Option<bool>` / `on_open_change: Option<EventHandler<bool>>>`
  for controlled-mode callers.
- **Owned by the caller** when the data is domain (user, notifications,
  breadcrumb list). Component takes a typed prop and renders.
- Never call `use_signal` outside a `#[component]` function.

### 3. Accessibility (a11y) — every interactive chrome component

| Component | Required a11y |
| --- | --- |
| `Navbar` / `MobileNav` | `aria-label="Primary"`, landmark `role="navigation"` |
| `Sidebar` | `aria-label="Sidebar"`, `role="navigation"` |
| Sidebar item with children | `aria-expanded` on the parent button, `aria-controls` pointing to the children list id, children list `role="list"` |
| Active nav item | `aria-current="page"` on the link |
| `Sheet` (mobile nav) | `role="dialog"`, `aria-modal="true"`, focus trap, Escape closes |
| `Breadcrumbs` | `aria-label="breadcrumb"`, list uses `<ol>`, current item `aria-current="page"` |
| `Modal` (auth) | `role="dialog"`, `aria-modal`, `aria-labelledby`, focus trap, Escape closes |
| `AuthGate` overlay | `role="alertdialog"`, `aria-labelledby`, `aria-describedby` |
| `IconButton` (theme toggle, hamburger) | `aria-label`, focus visible |
| `AccessDenied` | `role="alert"`, `aria-labelledby` |
| `ProgressiveAuthBanner` | `role="status"` |
| `NotificationBell` | `aria-label="Notifications"`, badge gets `aria-label="unread notifications: N"` |

When in doubt, mirror what the TS shadcn / Radix source does.

### 4. Styling

- All visual styling is provided by the **global CSS emitted by
  `epsx_templates::design_system_head`** — Tailwind v2.2.19 CDN + EPSX
  design tokens.
- **Coders MAY add CSS to `shared/rust/templates/src/lib.rs`** when a
  chrome component requires a class name that does not yet exist.
  This follows the Wave 1 addendum (`docs/wave1-primitives/design-addendum-1.md`).
  Constraints:
  - Add CSS at the bottom of the existing CSS block in `lib.rs` inside
    a clearly-marked `/* wave2-chrome */` comment region.
  - Do **not** rewrite or reformat existing CSS.
  - Class names follow the kebab-case convention already used in the file
    (e.g. `admin-shell`, `mobile-nav-sheet`, `breadcrumb-current`,
    `wallet-provider-card`).
  - Tailwind v2 utilities available via CDN: `flex`, `gap-2`, `text-sm`,
    `text-muted-foreground`, `rounded`, `border`, `border-border`,
    `text-red-500`, `text-center`, `py-8`, `text-2xl`, `font-semibold`,
    `mt-1`, `ml-1`, `opacity-30`, `cursor-pointer`, `select-none`,
    `bg-card`, `bg-background`, `bg-gradient-to-r`, `from-[#1fc7d4]`,
    `to-[#7645d9]`, `shadow-lg`, `shadow-cyan-500/20`, `animate-pulse`,
    `min-w-0`, `truncate`, `flex-shrink-0`, `transition-all`,
    `active:scale-95`, `hover:bg-muted/30`, `lg:flex`, `md:hidden`,
    `lg:hidden`, `sm:flex`, etc.

### 5. Icons

- Use `<Icon name="..." size={Some(16)} />` from
  `crate::primitives::icon::Icon` (or `super::icon::Icon`).
- Icon names must be **lucide** and exist in `epsx_templates::lucide`.
  Current registry covers all icons used by the TS sources (Bell, Menu,
  ChevronRight, ChevronDown, Code, BarChart3, Newspaper, CreditCard,
  Wallet, MessageCircle, ImageIcon, History, Globe, Home, etc.).
- If you need an icon that's not in the registry, **don't add a new
  helper** — note it in a `// TODO: add to lucide registry:` comment.

### 6. Event handlers

- All event handlers use `EventHandler<T>` (Dioxus 0.7 API) and are
  **always optional**: `onclick: Option<EventHandler<MouseEvent>>`. Inside
  `rsx!`, wrap calls in `if let Some(h) = &onclick { h.call(e); }`.
- For the `use_event` crate (keyboard handlers on the auth modal's
  Escape key), prefer `use_event_listener` or a `use_effect` with a
  `gloo_events` listener on the document body. **Do NOT block on
  `EventHandler::KeyboardEvent` only** — wire a real listener.

### 7. Public API stability

- **Don't break** the function signature of any existing chrome component
  that is consumed by `pages/*.rs`. If you need to add a parameter, give
  it a default (`#[props(default = ...)]`).
- Re-export everything from `layout.rs` and `auth.rs` at the crate root
  via `pub use layout::*` and `pub use auth::*` (these are already wired
  in `dioxus_ui/src/lib.rs`).
- New structs (`NavGroup`, `SidebarItem` variants, `NotificationItem`)
  must be exported and re-exported through `pub use` so pages can name them.

### 8. Pages don't migrate in Wave 2

Pages in `shared/rust/dioxus_ui/src/pages/*.rs` are still consumers.
**You may not edit pages** to match a signature change. Instead, your
changes must be **additive**: new params have defaults, removed params
never happen, renamed params keep the old name as a deprecated alias.

Exception: if a chrome component rename is unavoidable, add a
`#[deprecated]` re-export at the bottom of the file and keep the old
function as a thin wrapper that calls the new one. Document the rename
in your `deliverable.md`.

## Track scope

### Track A — Admin shell (sidebar + header + breadcrumb + main-layout)

Owner: `coder` #1
Worktree: `/private/tmp/epsx-track-a-admin` on branch `wave2/track-a-admin`

Files to port or rewrite:
- `layout/sidebar.rs` — full port of `apps-old/admin-frontend/components/layout/sidebar.tsx` (336 lines):
  - `SidebarItem` struct with `children: Option<Vec<SidebarItem>>`, `requires_auth: bool`, `disabled: bool`, `tab: Option<String>`, `chat_count: Option<u32>`
  - Expand/collapse state via `use_signal<HashSet<String>>`
  - Nested children rendering with `NavChildren` subcomponent
  - Disabled state with `Lock` icon overlay
  - `NavItemBadge` (chat count pill / active dot)
  - Connect-wallet CTA card when unauthed
  - User pill at the bottom (authed: "Admin user" + emerald dot; guest: "?" + grey dot)
  - "ADMIN" subtext under the EPSX logo
- `layout/header.rs` — full port of `header.tsx` (86 lines):
  - `Header` component with `user`, `initial_notifications`, `initial_unread_count`
  - Sticky top-0 with z-40, border-b border-border/40, bg-card
  - Left: `Breadcrumb`
  - Right: `AdminNotificationBell` (Track A doesn't have to fully port the bell client — slot component with `notification_count` + click handler is enough for now), `UnifiedThemeToggle` slot, `ChainSelector` slot (dev-only via `is_production` prop), `WalletConnectButton`
- `layout/breadcrumbs.rs` — full port of `breadcrumb.tsx` (130 lines):
  - `Breadcrumb` component reads `current_path: String`
  - `ROUTE_CONFIG` static map mirroring TS source (Dashboard, Users, Analytics, Notifications, etc.)
  - `generate_breadcrumbs(pathname: &str) -> Vec<BreadcrumbItem>` helper
  - Single-crumb and multi-crumb render branches
  - Gradient text on the last item
  - For emoji icons in route config: render as text in `<span>`
- `layout/shell.rs` — extend existing `DashboardShell` + `DeveloperShell`:
  - Add `MainLayout` component that composes `Sidebar` + `Header` + scrollable main + footer
  - Add `AuthLayout` slot that wraps `MainLayout` and shows `AuthGate` overlay when needed
  - `AdminLayout` enum (None/Dashboard/Developer/Auth) dispatches the right shell
- Add `layout/footer.rs` admin version (or extract into `AdminFooter`):
  - The TS footer is 3 lines ("EPSX Admin Dashboard" / "Version 2.0") — add a thin `AdminFooter` component in `shell.rs` and export it.

Read first:
- `apps-old/admin-frontend/components/layout/sidebar.tsx` (336 lines)
- `apps-old/admin-frontend/components/layout/header.tsx` (86 lines)
- `apps-old/admin-frontend/components/layout/breadcrumb.tsx` (130 lines)
- `apps-old/admin-frontend/components/layout/main-layout.tsx` (74 lines)
- `apps-old/admin-frontend/components/layout/auth-layout.tsx` (63 lines)
- `apps-old/admin-frontend/components/layout/layout-wrapper.tsx` (30 lines)

### Track B — Frontend nav cluster (desktop + mobile + skeleton + actions + wallet)

Owner: `coder` #2
Worktree: `/private/tmp/epsx-track-b-frontend-nav` on branch `wave2/track-b-frontend-nav`

Files to port:
- `layout/navbar.rs` — rewrite to match `navigation-client.tsx` + `desktop-nav.tsx`:
  - `NavigationClient` (orchestrator) — `is_hydrated` signal, `pathname`, `is_connected` + `is_authenticated` props
  - Sign-in banner: gradient sticky top-0 strip with "Your wallet is connected — Sign In with Wallet — to access all features"
  - On `/auth` page: render `Fragment` (hide entire nav)
  - On hydration: render `NavbarSkeleton`
  - Otherwise: header (sticky top-0 z-50), mobile logo, `DesktopNav`, `NavActions`
  - `DesktopNav` — uses `NAV_GROUPS` config, `GroupDropdown` with `DropdownMenu` primitive from Wave 1
  - `NAV_GROUPS` static const (move from TS: Market / Developer / Company groups, with `LineChart`, `TrendingUp`, `Code`, `BookOpen`, `Info`, `Newspaper`, `Mail`, `HelpCircle` icons)
- `layout/navbar_skeleton.rs` (NEW file) — port of `navbar-skeleton.tsx` (18 lines):
  - `NavbarSkeleton` component (header + skeleton pulse placeholders)
- `layout/mobile_nav.rs` (NEW file) — port of `mobile-nav.tsx` (159 lines):
  - `MobileNav` with `is_authenticated: bool` prop
  - Uses `Sheet` + `SheetContent` + `SheetTrigger` from Wave 1
  - Hamburger button with `Menu` icon, `aria-label="Open menu"`
  - Mobile menu: header with logo, optional wallet card (when `is_connected` && `address`), `MobileGroup` subcomponents, Notifications link when authed, `ChainSelector` (dev only), `WalletProviderIcon` at bottom
- `layout/nav_actions.rs` (NEW file) — port of `nav-actions.tsx` (36 lines):
  - `NavActions` with `is_authenticated: bool`
  - Three responsive variants: `hidden md:flex` (desktop with notification + theme + chain + wallet), `hidden sm:flex md:hidden` (tablet compact), mobile (`<MobileNav />`)
- `layout/footer.rs` — keep existing 4-column site footer; add a `SiteFooter` re-export alias. (Admin footer is in Track A's shell.rs.)
- `layout/page_header.rs` — keep existing `PageHeader`; ensure it has a `class_name: Option<String>` param.
- `layout/breadcrumbs.rs` — leave as-is (Track A owns breadcrumbs). DO NOT touch.

Add nav-config helpers:
- `layout/nav_config.rs` (NEW file) — mirror `nav-config.ts`:
  - `NavItem` struct (`label: String, href: String, key: String, icon: Option<String>, desc: Option<String>`)
  - `NavGroup` struct (`label: String, key: String, icon: Option<String>, items: Vec<NavItem>`)
  - `pub const NAV_GROUPS: &[NavGroup]` const data
  - `pub const FOOTER_LINKS: &[NavItem]` const data
  - `is_group_active(group, pathname)` and `is_item_active(item, pathname)` helpers
  - `is_production()` shim (reads `cfg!(debug_assertions)`) so dev-only items work
- Re-export `nav_config` from `layout.rs`.

Read first:
- `apps-old/frontend/components/nav/navigation-client.tsx` (99 lines)
- `apps-old/frontend/components/nav/desktop-nav.tsx` (87 lines)
- `apps-old/frontend/components/nav/mobile-nav.tsx` (159 lines)
- `apps-old/frontend/components/nav/nav-actions.tsx` (36 lines)
- `apps-old/frontend/components/nav/navbar-skeleton.tsx` (18 lines)
- `apps-old/frontend/components/nav/nav-config.ts` (76 lines)
- `apps-old/frontend/components/nav/wallet-provider-icon.tsx` (379 lines)

### Track C — Auth cluster (modal + gate + access-denied + progressive + user + wallet-button + shells)

Owner: `coder` #3
Worktree: `/private/tmp/epsx-track-c-auth` on branch `wave2/track-c-auth`

Files to port or extend:
- `auth/auth_modal.rs` — extend existing scaffold:
  - Add focus trap (use the same `use_event_listener` pattern as Wave 1's `Modal`).
  - Add `WalletOption` subcomponent with `name` + `icon` + `on_click` handler.
  - `DemoButton` slot for the demo-account CTA.
  - Real `Escape` key listener that calls `on_close`.
  - `on_open_change: Option<EventHandler<bool>>` for controlled callers.
- `auth/auth_gate.rs` — extend existing scaffold:
  - `feature: Option<String>` already there; add `required_permissions: Option<Vec<String>>`, `return_url: Option<String>` for the gated redirect.
  - `AdminAuthGate` variant component (in same file) for the admin context — renders with admin-specific copy + `AdminWalletConnectAuth` slot.
  - `is_gated: bool` prop so callers can stack the gate on top of the layout.
- `auth/access_denied.rs` — extend existing scaffold:
  - Add `required_permissions` list rendering (already there).
  - Add `return_url: Option<String>` for the "back to" link.
  - Add `icon_name: Option<String>` prop (default `info`) so the icon is themable.
- `auth/progressive_banner.rs` — extend existing scaffold:
  - Add `feature: Option<String>` (already there), `cta_label: Option<String>` (default "Sign in"), `on_click: Option<EventHandler<MouseEvent>>` for the CTA button (so the caller can open the auth modal rather than navigating to `/auth`).
  - Add `dismissible: Option<bool>` slot — if true, render a close button. Default false.
- `auth/user.rs` — extend existing scaffold:
  - Add `auth_method: Option<AuthMethod>` enum (`Siwe`, `EoA`, `Demo`).
  - Add `last_login_at: Option<i64>` (unix timestamp).
  - Add `is_admin()`, `has_role(role)`, `has_any_role(roles)` helpers.
  - Add `display_name() -> String` that prefers `email` over `short_address()`.
- `auth/wallet_button.rs` — extend existing scaffold:
  - Add `WalletProviderDropdown` (port of wallet-provider-icon.tsx — but condensed):
    - `ConnectButton` (not connected): orange gradient, compact or full
    - `ConnectedDropdown` (connected): trigger pill with truncated address + chevron, dropdown with: wallet info card (icon + name + full address + status), copy button, explorer button, sign-in CTA (when not authed), retry CTA (when auth failed), account settings link, developer portal link, disconnect button (danger styling)
    - Provider registry: metaMask, walletConnect, injected, coinbase, rainbow (with emoji icon)
    - State: `is_open`, `is_copied`, `is_authenticating`, `is_disconnecting`, `auth_retry_count`, `last_auth_error`
    - Required callbacks: `on_copy: EventHandler<()>` (no clipboard API in Dioxus SSR — caller wires it), `on_view_explorer: EventHandler<()>`, `on_sign_in: EventHandler<()>`, `on_disconnect: EventHandler<MouseEvent>`, `on_open_change: EventHandler<bool>`
- `layout/shell.rs` — Track A owns the main `DashboardShell`/`DeveloperShell`/`MainLayout`/`AuthLayout` machinery. **Track C does NOT edit shell.rs directly** to avoid a merge conflict. Instead, Track C adds a thin re-export at the top of `shell.rs` via a `pub use` of any new shell helpers from a new `auth_shell.rs` file IF AND ONLY IF a new shell helper is required for the auth flow. **Default: do not touch shell.rs.**

Read first:
- `apps-old/admin-frontend/components/auth/admin-auth-gate.tsx`
- `apps-old/admin-frontend/components/auth/admin-wallet-connect-auth.tsx`
- `apps-old/frontend/components/auth/*.tsx` (any wallet / sign-in / SIWE)
- `apps-old/frontend/components/nav/wallet-provider-icon.tsx` (379 lines)
- Wave 1's `shared/rust/dioxus_ui/src/primitives/dropdown.rs` and `primitives/modal.rs` (the `Dropdown`/`Modal` primitives Track C composes)

## Per-track deliverables

Each track produces:
1. **Code changes** in the files listed above.
2. **Re-exports** in `layout.rs` and `auth.rs` for any new public items.
3. **A `deliverable.md`** at the track's worktree root listing:
   - Components added or enhanced
   - Any signatures changed (and the re-export alias path)
   - Any a11y gaps intentionally left for Wave 3 (with a one-line reason)
   - `cargo check --workspace` final result (paste the last 5 lines)
   - List of CSS classes added to `epsx-templates` lib.rs (if any) with their line numbers

## Verifier gate (per-track)

After each track reports done, a single verifier run:
1. `cargo check --workspace` — must be green (warnings ok, errors not).
2. `cargo build --workspace --bins` — must succeed.
3. `cargo test -p epsx-dioxus-ui --lib` — if any `#[cfg(test)]` modules exist, they must pass.
4. `git diff migration/dioxus-microservices..HEAD --stat` — show file change summary.
5. Spot-check **3 random chrome components** (verifier picks) — open the .rs file, confirm:
   - Follows the signature conventions above.
   - Has the required a11y attributes.
   - Uses existing or just-added (with `/* wave2-chrome */` marker) class names.
6. Spot-check that **pages still compile** by running `cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`.
7. Confirm no page files were edited (`git diff --stat HEAD -- shared/rust/dioxus_ui/src/pages/` should be empty).
8. Confirm no `shared/rust/dioxus_ui/src/primitives/*.rs` was edited by chrome tracks (primitives are Wave 1's territory — any new primitive must be filed as Wave 3+ work).

## Commit & push

Per the user's push cadence ("per batch, after cargo green"):
- Each track commits locally as it goes (no need to wait for others).
- The **plan owner** (me) will fast-forward `migration/dioxus-microservices` to the combined integration commit and push **after** all three tracks pass verification AND the integration gate is green. Workers do not push to remote.

## Integration gate (after all 3 tracks land)

A 4th task (the integration gate, assigned to `verifier`) runs after
all 3 tracks PASS. It:
1. `git checkout migration/dioxus-microservices` in the integration worktree
2. For each track in order A, B, C: `git merge --no-ff wave2/track-X-<slug>` — resolve any conflicts (expected: Track A and Track B both touch `layout.rs` re-exports, expected: Track A and Track C both touch `shell.rs` — Track C was instructed NOT to, so this should be clean).
3. Run `cargo check --workspace` + `cargo build --workspace --bins`.
4. Run a BFF smoke check by starting each BFF (`apps/frontend`, `apps/admin`, `apps/pay`, `apps/preview`) on its port and hitting `/` with `curl` — must return 200.
5. Write `deliverable.md` (the integration report) at the repo root with: merge log, gate results, diff stat, BFF smoke results, push confirmation, HEAD hash.

The integration gate is **not** a parallel track — it depends on all 3
tracks, runs after they all PASS, and is the final task in the plan.

## What is out of scope for Wave 2

- **Pages** (`shared/rust/dioxus_ui/src/pages/*`) — not edited.
- **Primitives** (`shared/rust/dioxus_ui/src/primitives/*`) — not edited. (Track A/B/C must compose existing primitives; if a primitive is missing, file a one-line note in `deliverable.md`.)
- BFF route changes.
- Removing deprecated shadcn re-exports in `apps-old/*`.
- Per-page port (using the new chrome components in actual pages) — that's a follow-up wave.
- Wiring the new components into the BFF templates — same, follow-up.

## Worktree rule (CRITICAL — Wave 1 lesson)

Each coder MUST create their worktree **before editing any file**. The
shared main checkout will be poisoned if multiple coders edit it in
place (Wave 1 hit this exact problem in the first attempt — they
recovered via `/private/tmp/epsx-track-X` worktrees).

Use the following worktree paths (hardcoded for plan owner convenience):

- Track A: `/private/tmp/epsx-track-a-admin` on branch `wave2/track-a-admin` based on `origin/migration/dioxus-microservices`
- Track B: `/private/tmp/epsx-track-b-frontend-nav` on branch `wave2/track-b-frontend-nav` based on `origin/migration/dioxus-microservices`
- Track C: `/private/tmp/epsx-track-c-auth` on branch `wave2/track-c-auth` based on `origin/migration/dioxus-microservices`

Each coder's prompt bakes the worktree rule in. Do not edit files in
the main checkout. After each commit in your worktree, do `git push
origin wave2/track-X-<slug>` so the integration step can merge.

## Reference: existing public API (don't break these)

```rust
// From layout.rs re-exports (existing scaffolds)
Navbar, NavItem, NavGroup, main_nav_groups,
Footer, SiteFooter,
Sidebar, SidebarItem,
DashboardShell, DeveloperShell,
PageHeader,
Breadcrumbs, Crumb,

// From auth.rs re-exports (existing scaffolds)
User, User::is_authed, User::short_address, User::is_admin, User::has_permission,
AuthModal,
AuthGate,
AccessDenied,
ProgressiveAuthBanner,
WalletConnectButton, ConnectedWalletDropdown,
```

All of the above must remain importable from `epsx_dioxus_ui::*` after Wave 2.
