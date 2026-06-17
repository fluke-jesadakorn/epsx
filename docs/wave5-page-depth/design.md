# Wave 5 — Marketing/Auth Page Depth: Close the UX/UI Gap

## Why this wave exists

The Wave 1–3 port brought `shared/rust/dioxus_ui` from zero to "every
page has a file" (28 frontend pages, 20 admin pages) and a working
chrome (Wave 2) + auth gates (Wave 3b). The Wave 4 cleanup removed
the dead string-template SSR. **But the port is shallow**: a real
port-to-pixel-parity pass would touch every page. This wave attacks
the **12 marketing/auth frontend pages** — the routes that anonymous
visitors see first, and the source of most user complaints about
"this doesn't look like the old site."

After Wave 5 ships, every page a signed-out user can hit (`/`,
`/about`, `/plans`, `/auth`, `/manual`, `/contact`, `/news`,
`/privacy`, `/terms`, `/access-denied`, `/offline`, `/not-found`)
should match the `apps-old/frontend/app/{page}/` original at a
visual level. Wave 6 will do the same for the 16 app/admin pages
behind auth.

## What "100% UX/UI" means for this wave

For each ported marketing page in this wave, the deliverable is:

1. **All sections from the source Next.js page are present** in
   the Dioxus port — same number of `#[component]` blocks as the
   original has TSX sub-components, in the same order.
2. **All copy/text is preserved** — strings come from the source
   page, not invented.
3. **Visual class names match the design system** — the same
   `tailwind`-style utility classes already used in
   `shared/rust/dioxus_ui/src/layout/` and `primitives/` (no new
   CSS, no new design tokens).
4. **SSR renders the static parts at request time** — the existing
   `#[component] fn render(ctx: &PageContext) -> Element` pattern
   in each page already does this. Wave 5 keeps the same pattern
   and just makes the components bigger.
5. **Each page has a unit test** that calls `render()` with a
   minimal `PageContext` and asserts the page body contains the
   expected section markers (e.g. the home page should contain
   "hero-title", "trust-bar", "top-performers", "features-grid",
   "pricing-teaser", "news-preview", "cta-section" CSS class
   names). This is the "verify against source-of-truth" check.

What this wave does NOT do:

- gRPC, microservices extraction, or any backend work.
- New auth flows (the existing SIWE + cookie + JWT + `AuthGate`
  stack is fine; Wave 3b is solid).
- The 16 admin pages or the 16 auth-required user pages — those
  are Wave 6.
- A page-by-page Playwright visual-diff harness. The unit-test
  section-marker check is the practical compromise: it catches
  "the homepage is missing the news preview" regressions without
  the cost of running a real browser in CI.

## Source of truth (and how to use it)

| What | Where |
| --- | --- |
| Next.js page entry | `apps-old/frontend/app/{slug}/page.tsx` |
| Page-local sub-components | `apps-old/frontend/components/{slug}/*.tsx` |
| Page-local data | `apps-old/frontend/app/{slug}/data.{ts,tsx}` |
| Ported Dioxus page | `shared/rust/dioxus_ui/src/pages/{slug}.rs` |
| Shared design-system primitives | `shared/rust/dioxus_ui/src/primitives/` |
| Layout chrome | `shared/rust/dioxus_ui/src/layout/main_layout.rs` |
| BFF → page glue | `shared/rust/dioxus_ui/src/pages.rs` |

**Read the source TSX first.** Every port starts with `wc -l
apps-old/frontend/app/{slug}/page.tsx apps-old/frontend/components/{slug}/*.tsx`
to size the work, then read the source end-to-end, then port.

## The 12 pages, sized

| # | Page | Port LoC | Source LoC | Gap | Source sub-components |
| - | ---- | -------- | ---------- | --- | --------------------- |
| 1 | `home` | 356 | 2,019 (components) | 5.7x | hero, dynamic-pricing-client, dynamic-pricing-section, financial-data-table, server-news, server-top-performers, share-button |
| 2 | `auth` | 121 | 1,427 (page+components) | 11.8x | frontend-auth-modal, wallet-connect-auth, wallet-connection-modal, connected-wallet-dropdown, progressive-auth-banner, frontend-auth-gate, permissions-display, global-auth-guard |
| 3 | `manual` | 103 | 153 (page) | 1.5x | data.ts (CATEGORIES + FEATURES), screenshot-img |
| 4 | `about` | 77 | 319 (page+components) | 4.1x | data-tech-section |
| 5 | `plans` | 107 | 349 (page+components) | 3.3x | plan-selection |
| 6 | `contact` | 44 | 153 (page) | 3.5x | contact-form (CopyEmailBtn, MailtoBtn) |
| 7 | `privacy` | 26 | 116 (page) | 4.5x | inline |
| 8 | `terms` | 28 | 171 (page) | 6.1x | inline |
| 9 | `not_found` | 25 | (in error.tsx / not-found.tsx) | — | inline |
| 10 | `error_page` | 24 | 181 (error.tsx) | 7.5x | inline |
| 11 | `offline` | 23 | 94 (page) | 4.1x | inline |
| 12 | `access_denied` | 19 | 24 (page) | 1.3x | inline (uses `AccessDenied` primitive) |

**Total target LoC** (post-Wave 5): ~7,000 LoC across the 12 pages,
up from ~950 today. Roughly 6x growth, matching the 5.7–11.8x
gap to source. The BFFs compile and serve all 12 routes today —
the gap is purely page content.

## Track split (2 parallel coder tracks + integration gate)

### Track A — "Hero pages" (Track A: home + auth + about)

**Pages**: `home.rs`, `auth_page.rs`, `about.rs`

**Why these together**: the most user-facing, biggest gap, and
they share the same PancakeSwap-style gradient background
(`bg-gradient-to-br from-blue-50 via-orange-50 to-yellow-50
dark:from-slate-900 dark:via-slate-800 dark:to-slate-900`).
Reusing the same `<MarketingBackground>` component across all
three is a real win — coders should land that as a small
extraction in `shared/rust/dioxus_ui/src/layout/marketing_bg.rs`
and have all three pages use it.

**Specific section coverage targets** (each is a
`#[component]` function in the ported file):

`home.rs` (356 → 700 LoC target):
- `Hero` — already present, expand to include the share button,
  the live chain selector, and the responsive mobile collapse
  (look at `apps-old/frontend/components/home/hero-section.tsx`
  + `share-button.tsx`).
- `TrustBar` — already present, add 2 more trust logos from the
  source (Binance, Ethereum Foundation — the source uses 4, port
  has 2).
- `TopPerformers` — already present, add the "data freshness"
  timestamp + the row-level click-through to `/portfolio/{addr}`.
- `FeaturesGrid` — already present, ensure all 6 feature cards
  match the source copy.
- `PricingTeaser` — already present, link the "View all plans"
  button to `/plans` and ensure responsive collapse.
- `NewsPreview` — already present, link each card to
  `/news/{slug}` (the source uses server-side fetch, port uses
  `ctx.params.get("data_news")`).
- `CTASection` — already present, add the secondary "Talk to
  sales" link.
- **NEW** `TestimonialsSection` — 3 testimonial cards, the
  source has this; the port is missing it.
- **NEW** `FAQSection` — 6-question accordion, the source has
  this; the port is missing it.

`auth_page.rs` (121 → 400 LoC target):
- Two-column layout: left side has the marketing pitch (Hero
  copy + value props + testimonial), right side has the auth
  form. The current port is form-only.
- The auth form needs to support: SIWE wallet connect button
  (using `ConnectedWalletDropdown` from
  `shared/rust/dioxus_ui/src/auth/wallet_button.rs`),
  "Continue with email" magic link, "Continue with Google"
  OAuth button (visual only — the actual OAuth call is to
  `/api/v1/auth/oauth/{provider}` which Wave 2 Track C wired up).
- Error states: "wallet not installed" CTA to wallet guides,
  "wrong network" prompt to switch, "signature rejected" inline
  error.
- Loading states: spinner during challenge fetch, button
  disabled + "Check your wallet..." text during signature.

`about.rs` (77 → 350 LoC target):
- PancakeSwap-style hero with floating gradient orbs (use the
  new `<MarketingBackground>`).
- `MissionSection` — 3-column "what we do" cards.
- `StatsSection` — 4 stat cards (active users, transactions
  processed, countries served, uptime %).
- `TeamSection` — placeholder grid of 6 team-member cards.
- `TimelineSection` — vertical timeline of company milestones
  (5–6 entries).
- `CTASection` — "Join us" footer with two buttons.
- Inline `data-tech-section.tsx` port — that source file is 229
  LoC and renders the "Our data + tech stack" grid. Important
  because it lives at the bottom of the source `page.tsx`.

**Constraint**: this track should NOT touch
`shared/rust/dioxus_ui/src/primitives/` or
`shared/rust/dioxus_ui/src/layout/main_layout.rs`. The
`<MarketingBackground>` extraction goes in a new file
`shared/rust/dioxus_ui/src/layout/marketing_bg.rs` and the
worker module-pub-uses it from `shared/rust/dioxus_ui/src/layout.rs`.

### Track B — "Info pages" (Track B: manual + plans + contact + 6 utility pages)

**Pages**: `manual.rs`, `plans.rs`, `contact.rs`, `privacy.rs`,
`terms.rs`, `not_found.rs`, `error_page.rs`, `offline.rs`,
`access_denied.rs`

**Why these together**: all 9 are static-content pages, no
client-side state, no wallet/auth interaction. They share a
common pattern (max-width container + sectioned content) and
the work is mostly transcription + layout polish.

**Specific section coverage targets**:

`manual.rs` (103 → 500 LoC target):
- The source has 8 categories (Auth, Wallet, Plans, Payments,
  Portfolio, News, Notifications, Developer) each with 3–6
  feature entries. Each feature has a title, description, and
  screenshot path (`/screenshots/{category}/{feature}.png`).
- Port should mirror: sticky left sidebar with 8 category links,
  right content with 8 `<details>` blocks (or accordion
  primitives) each rendering 3–6 feature rows. The
  `screenshot-img.tsx` component is just `<img loading="lazy"
  src={path} alt={alt} />`.
- Screenshot paths use the existing public asset
  `apps/frontend/public/screenshots/...` (already wired in the
  BFF's static handler).

`plans.rs` (107 → 400 LoC target):
- 3-tier pricing grid: Free / Pro / Enterprise.
- The `plan-selection.tsx` source is 275 LoC and contains the
  full pricing card with feature lists, "Most popular" badge,
  monthly/annual toggle, CTA buttons.
- Below the grid: comparison table (5 rows × 3 columns) +
  FAQ accordion (4–6 questions) + "Need custom?" enterprise
  contact CTA.

`contact.rs` (44 → 250 LoC target):
- PancakeSwap-style gradient background (use the
  `<MarketingBackground>` from Track A — coordinate via the
  design doc).
- 3 info cards (General, Support, Response time) using
  `Card` + `Icon` primitives.
- Contact form: name, email, subject, message, submit button.
  Client-side validation only — actual submit hits
  `/api/v1/contact` which is already wired (Wave 1).
- Two inline buttons (`CopyEmailBtn`, `MailtoBtn`) — small
  client components, port as inline `#[component] fn` calls.

`privacy.rs`, `terms.rs` (26+28 → 250+350 LoC):
- The source is a single `<main>` with a series of `<h1>` /
  `<h2>` / `<p>` blocks. Port is a transcription task.
- Add a sticky table-of-contents at the top (same pattern as
  the manual page sidebar, but inline at the top instead of
  fixed-left because these pages are text-heavy).

`not_found.rs`, `error_page.rs`, `offline.rs` (25+24+23 →
150+200+150 LoC):
- 404 page: centered illustration + "Page not found" + "Go
  home" button.
- Error page: same as 404 but with the error message from
  `PageContext.error`.
- Offline page: centered icon + "You're offline" + "Retry"
  button (calls `navigator.onLine`).

`access_denied.rs` (19 → 80 LoC):
- Already wraps `<AccessDenied>` primitive. Just add the
  "go back" link and the optional "request access" CTA for
  users who think they should have access.

**Constraint**: Track B is mostly mechanical transcription
and component polish. No new files outside
`shared/rust/dioxus_ui/src/pages/*.rs`. If the worker needs
the `<MarketingBackground>` from Track A, import it from the
new module path that Track A documents in its deliverable.

### Integration gate (single track)

**Goal**: prove the 12 pages compile, the BFFs route them
correctly, and the section-marker unit tests pass.

**Steps**:
1. `cargo check -p epsx-dioxus-ui --lib` — must pass with
   zero errors.
2. `cargo test -p epsx-dioxus-ui --lib pages::` — the
   section-marker tests added by both tracks must pass.
3. `cargo build -p epsx-frontend --release` — the frontend
   BFF must build (this is the 15-min link step; the
   integration agent owns the full build per the Wave 3a
   memory entry).
4. Smoke test: start the BFF on port 3000, hit each of the
   12 routes with `curl -sI`, assert HTTP 200 and
   `content-type: text/html` for all of them.
5. If the unit-test approach is the visual contract, write
   ONE additional end-to-end smoke: hit `/` and assert the
   response body contains the strings `"hero-title"`,
   `"trust-bar"`, `"top-performers"`, `"features-grid"`,
   `"pricing-teaser"`, `"news-preview"`, `"cta-section"`,
   `"testimonials-section"`, `"faq-section"`. If any are
   missing, that's a Track A failure and gets routed back.

## Conventions all tracks must follow

These are non-negotiable. The team plan verifiers will check
each one.

1. **No new CSS**. Use only the design-system classes already
   in `shared/rust/dioxus_ui/src/styles/`. If a section needs
   a gradient/orb that doesn't have a class, add it to
   `styles/gradients.css` with a `/* wave5-track-{a,b} */`
   marker, and document it in the design doc. Do NOT
   inline-style with `style="..."`.
2. **No new dependencies**. The workspace is on Dioxus 0.7 +
   Axum + tokio. If you find yourself wanting `serde_yaml` or
   `pulldown-cmark`, that's a design doc issue — bring it up
   to the orchestrator, don't sneak it into a Cargo.toml.
3. **Section markers**. Every `#[component]` block in a ported
   page should render a top-level `section` with a
   `class="{section-name}"` (e.g. `class="hero"`,
   `class="trust-bar"`). The unit tests will grep for these
   class names. If you forget, the test fails and your track
   gets rejected.
4. **Copy from source**. Strings in the port must come from
   the source `apps-old/frontend/app/{slug}/page.tsx` or its
   components. No invented text. If the source has a typo,
   preserve the typo (and file a `// TODO: source typo` comment
   next to it).
5. **PageContext discipline**. Use `ctx.params.get(...)` for
   data the BFF pre-fetched, `ctx.user` for the current user,
   `ctx.path` for the current URL. Don't reach into
   `reqwest` directly from a page component — the BFF is the
   only thing that talks to the backend.
6. **Auth integration**. Pages that were free in the source
   (home, about, manual, contact, news, plans, privacy, terms,
   not_found, error, offline, access_denied) stay free in the
   port. The `<ProgressiveAuthBanner>` from Wave 3b is
   permitted on these pages for signed-out users — use the
   same pattern as `home.rs` already does.
7. **Tests**. Each ported page must add ONE `#[cfg(test)] mod
   tests` block with at least:
   - `test_render_smoke` — calls `render(&empty_ctx())` and
     asserts the returned `Element` is non-empty.
   - `test_section_markers` — calls `dioxus_ssr::render_element`
     on the returned `Element` and asserts the resulting HTML
     string contains every `class="{section-name}"` the page
     claims to have.
   For pages that are essentially just text (privacy, terms,
   not_found, error, offline, access_denied) the smoke test
   is enough — section markers don't apply.

## Out of scope (deferred)

- **Admin pages** (20 existing + 3 missing) — Wave 6.
- **Auth-required user pages** (dashboard, profile, account,
  analytics, chat, etc.) — Wave 6.
- **The new "marketing" routes added during Wave 4 cleanup**
  (`/portfolio`, `/payment`, `/developer`) — Wave 6, they're
  not actually marketing, they need auth.
- **Real Playwright visual diff** — separate infra project,
  not part of the migration port.
- **Backend → service crate extraction** — Wave 7.
- **gRPC, microservices deployment** — explicitly rejected in
  the Wave 5 plan; see the orchestrator's "Why HTTP/JSON" answer
  in the session transcript. Defer until there is a non-Rust
  client that needs a stable contract.

## What ships at end of Wave 5

A `migration/dioxus-microservices` HEAD where, when you
`cargo build -p epsx-frontend --release` and run the BFF, all
12 marketing routes (`/`, `/about`, `/plans`, `/auth`,
`/manual`, `/contact`, `/news`, `/privacy`, `/terms`,
`/access-denied`, `/offline`, `/not-found`) render
visually-faithful Dioxus SSR HTML. MIGRATION.md gets a new
"Wave 5" section. The 16 admin/auth-required pages are
unchanged (still at their Wave 1–4 depth) and are explicitly
tracked in MIGRATION.md as the next wave.

The plan YAML (`docs/wave5-page-depth/plan.yaml` or
`.mavis/plans/wave5-page-depth.yaml`) drives the team
execution; the integration deliverable file is
`docs/wave5-page-depth/integration.md` and gets committed
with the merge commit.
