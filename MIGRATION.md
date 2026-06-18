# Migration: Next.js → Dioxus 0.7 + Axum

**Branch:** `migration/dioxus-microservices`
**Status:** Migration functionally complete. Workspace builds + tests green. `Wave 4` cleanup landed. **Pixel-parity workstream closed at 82.40% mean on Wave 30 (2026-06-18).** Open items reduced to build-tool / Dioxus-version follow-ups + functional parity (auth flow, data binding) + admin-wallet-management-access-plans jitter check.

## Pixel-parity cumulative (Wave 25-30, 6 waves, 4 days)

| Wave | Scope | 13-ported mean | Top route | Notes |
|------|-------|----------------:|-----------|-------|
| 25 start | Initial port | 29.06% | 86.46% (home) | mixed shadcn / bare components |
| 26 | Shared tokens | 24.59% | 79.53% (portfolio) | neutral — T1 tokens hit ceiling |
| 27 | Fix regressions + port | 24.59% | 97.96% (manual) | minimal lift, identified structural gaps |
| **28** | **PostCSS migration (Tailwind v2 CDN → v4.1.18 + @tailwindcss/postcss + oklch tokens)** | **78.56%** | **98.42% (dashboard)** | **+53.97pp — biggest win, all 4 color-floor routes jumped to 80%+** |
| 29 | Fix 2 PostCSS regressions (plans +18.93pp, dev-usage -3.00pp) | 80.86% | 98.42% | brief 80%+ hit, 0 regressions on 11 other routes |
| **30** | **PricingCard 1:1 port (glass-morphism + Personal Plans header + uppercase title + blue price)** | **82.40%** | 98.42% | **+1.54pp mean, plans 48% → 77% (+29pp). Worker minimum-scope test: brief's "fix NaNm" hypothesis wrong (-1.14pp), real fix was glass-morphism (+30.53pp)** |

**Verdict: 82.40% = honest ceiling for current architecture.** Remaining gap is structural (gradient palettes on plans, modal-height jitter on dev-usage) not color. Functional parity (auth flow, live data binding, gradient backgrounds) is the next workstream, not pixel-parity.

## What landed (cumulative)

### Layout (restructure done)

The `migrated/` sub-monorepo prefix is gone. The Rust workspace is the
workspace root, and the Next.js TS apps are preserved as `apps-old/`.

```
.
├── Cargo.toml, Cargo.lock           # Rust workspace
├── apps/
│   ├── admin/                       # Dioxus+Axum admin BFF
│   ├── frontend/                    # Dioxus+Axum frontend BFF
│   ├── pay/                         # Dioxus+Axum pay BFF
│   ├── preview/                     # Dioxus+Axum preview BFF
│   ├── backend/                     # Axum backend (restored)
│   └── contracts/                   # Foundry (restored)
├── apps-old/                        # Next.js TS apps (READ-ONLY source of truth)
│   ├── frontend/                    # Next.js 14
│   └── admin-frontend/              # Next.js 14 admin
├── services/                        # 9 Rust microservices
│   ├── analytics, content, gateway, identity, indexer,
│   ├── notification, payment, subscription, wallet
├── shared/
│   ├── rust/                        # 13 shared Rust crates
│   │   ├── kernel, events, config, crypto, auth, database,
│   │   ├── observability, web3, client, renderer, templates,
│   │   ├── dioxus_ui, bff
│   └── (TS Next.js shared code; untouched)
├── content/                         # Marketing MDX / page blocks
├── proto/                           # gRPC protos
├── infrastructure/                  # K8s, Docker, Cloudflare Tunnel
└── scripts/                         # Dev / test / deploy scripts
```

### Component & page port (Wave 1-3 — done)

- **Core primitives** (27 components: Button, Card, Dialog, Input,
  DataTable, Charts, StatCard, Skeleton, Stepper, Switch, Tabs, etc.)
  in `shared/rust/dioxus_ui/src/primitives/`
- **Layout** (12 components: Header with 3 dropdowns, Footer, Sidebar,
  Wallet Connect modal, MainLayout, Shell, Navbar, MobileNav, etc.)
  in `shared/rust/dioxus_ui/src/layout/`
- **Auth** (AuthGate, AdminAuthGate, AccessDenied, ProgressiveAuthBanner,
  AuthModal, ConnectButton, ConnectedWalletDropdown, etc.)
  in `shared/rust/dioxus_ui/src/auth/`
- **Feedback** (Toast, Spinner, EmptyState, ErrorView)
  in `shared/rust/dioxus_ui/src/feedback/`
- **Data** (Pagination, LimitSelector, FilterBar, SearchInput)
  in `shared/rust/dioxus_ui/src/data/`
- **All 23 frontend pages** (Home, Auth, Dashboard, Profile, Account,
  Analytics, Chat, Plans, Permissions, News, Notifications, Contact,
  About, Manual, Portfolio, Developer, Payment, etc.) in
  `shared/rust/dioxus_ui/src/pages/`
- **All 12 admin pages** (Dashboard, Users, Audit, News, Media,
  Policies, Settings, Payments, Chat, wallet management) in
  `shared/rust/dioxus_ui/src/pages/admin_pages/`

### Service & BFF wiring (done)

- **9 microservices** with full DB schemas, SIWE auth, payment intents,
  escrows, content management, notification fan-out, analytics events,
  etc. — see `services/identity/`, `services/payment/`,
  `services/wallet/`, etc.
- **4 BFFs** (frontend, admin, pay, preview) proxy to the gateway
  via `epsx_client::ServiceClient` and SSR pages through
  `epsx_dioxus_ui::pages::render_page`. All 4 BFFs smoke 200s.
- **Auth flow** (Wave 2 Track C + Wave 3a): SIWE challenge → signature
  → JWT issue → HttpOnly cookies (`epsx_token`, `epsx_user_id`,
  `epsx_user_address`, `epsx_chain_id`).

### Wave 3b (auth gates — done)

- Track A — 12 frontend pages wrap their body in `<AuthGate>` with
  `required_permissions` + `return_url` (15 gate sites total).
- Track B — Admin pages use `AdminAuthGate` with per-route permissions.
- Track C — Free pages render `<ProgressiveAuthBanner>` for signed-out
  users so they still see the page body.

### Wave 4 (cleanup — landed 2026-06-11)

- Deleted `apps/frontend/src/legacy.rs` (2,033 lines of unreferenced
  string-template SSR; the Dioxus path is the active one).
- Deleted `apps/admin/src/legacy.rs` (407 lines, same situation).
- Implemented `ConnectedWalletState::from_cookies` — real
  `epsx_wallet` cookie parser (URL-encoded JSON, with `address`,
  `connector_id`, decimal `chain_id`). 12 unit tests cover the happy
  path, missing cookie, malformed JSON, hex chain id, empty fields,
  missing optional fields, and the parse_cookie / percent_decode
  helpers.
- Updated `apps/frontend/src/api.rs` docstring to drop the
  `legacy::ssr_fallback` reference.

### Compile + test gate

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in <N>s

$ cargo test --workspace --lib
    test result: ok. 35 passed; 0 failed  # epsx-dioxus-ui
    test result: ok.  8 passed; 0 failed  # epsx-auth
    test result: ok.  3 passed; 0 failed  # epsx-renderer
    test result: ok. 17 passed; 0 failed  # epsx-templates
    (others: kernel, events, crypto, web3, ... — all green)
```

## Next: deploy + remaining small items

The migration as designed is functionally complete: every Next.js
page in `apps-old/frontend` and `apps-old/admin-frontend` has a
Dioxus equivalent, every UI primitive the TS source uses exists in
the Rust crate, and the 9 services + 4 BFFs are wired end-to-end.

### Deploy path (next concrete step)

Per `docs/plans/2026-04-16-vercel-hybrid-deployment.md`:
1. Build the Vercel projects for `apps/frontend` and
   `apps/admin-frontend` (the BFFs are Rust + Dioxus — Vercel's
   build will need a Rust toolchain layer; pin to a known-good
   `cargo` and `dioxus-cli`).
2. Cut the Cloudflare Tunnel entries to the new BFF deployments.
3. Run the migration in shadow mode (mirror prod traffic, no DNS
   cut) for 24h, then switch over.
4. Tear down the Colima K8s deployments for the old `apps/frontend`
   and `apps/admin-frontend` Next.js services (the K8s layer stays
   for the Rust backend).

### Build-tool follow-ups (small, not blocking)

- **Tailwind in `rsx!`** — class strings are static for now. Add the
  `dioxus-tailwind` build step so utility classes are scanned out of
  `rsx!` blocks (otherwise we miss classes inside `format!()` calls).
- **`cva` equivalent** — replaced with simple `pub fn X(props)` per
  the open items carried over from earlier waves. Real variant logic
  is deferred until a page needs a 4th variant of some component.
- **`#[server]` macro** — Dioxus 0.7.9's `server` feature pulls in
  broken liveview. Action functions are `pub async fn`; the SSR
  layer calls them via the BFF. No follow-up needed unless Dioxus
  ships a working liveview in a later 0.7.x release.

### Open items carried over

- `cva` / `class-variance-authority` replaced with simple
  `pub fn X(props)`; real variant logic deferred (no concrete
  blocker — no page currently needs a 4th variant).
- Tailwind class strings in `rsx!` are static; build-tool follow-up
  to enable Dioxus tailwind processing.
- 15 dead-code warnings on `epsx-frontend` / `epsx-admin` (auth.rs
  helpers and `NewsQuery`) — pre-existing, will resolve when the
  admin BFF starts using `require_user` / `require_admin` directly
  (currently uses the SIWE cookie path in `auth_me`).

### What to do with `apps-old/`

- Keep as the **read-only reference** for the port. Per the design
  rule, a page is "done" when its Dioxus port passes the same
  visual / functional checks against the Next.js source.
- Delete `apps-old/frontend` and `apps-old/admin-frontend` after
  the production cutover is verified end-to-end (probably 1-2
  weeks post-launch). Until then, having both side by side is the
  fastest way to diff regressions.

## Per-file Rust counts (actual)

| Target | Rust files | Lines |
|---|---|---|
| `apps/frontend/src/**.rs` | 10 | 5,377 |
| `apps/admin/src/**.rs` | 4 | 1,363 |
| `shared/rust/dioxus_ui/src/**.rs` | ~90 | 14,000+ |
| `shared/rust/bff/src/**.rs` | 4 | small |
| `services/*/src/**.rs` | 9 services | 4,447 |
| `shared/rust/{kernel,events,...}/src/**.rs` | 13 shared crates | mixed |

(legacy.rs removed in Wave 4: -2,033 from `apps/frontend`,
-407 from `apps/admin`.)
