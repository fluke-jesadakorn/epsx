# EPSX Premium Fintech v2 — local delivery

Implemented 8 September 2026. This is the second frontend presentation pass, built on the existing Rust/Dioxus application and native BFF. No deployment, migration, production routing change, backend contract change, or Admin/Pay redesign is included.

## Open the review

- [Explore, signed out](http://127.0.0.1:3300/analytics)
- [Home](http://127.0.0.1:3300/)
- [Explore, fixture account](http://127.0.0.1:3301/analytics)
- [Watchlist, fixture account](http://127.0.0.1:3301/portfolio)
- [Account, fixture account](http://127.0.0.1:3301/account)

These loopback previews explicitly display **Preview data**. The fixture account is issued by the repository's Rust E2E authority. Prices, plans, example companies, account balances and notifications in these previews are test data, not current market or customer data. No application fallback inserts them when a real API fails. Startup instructions are in [the enterprise QA README](../e2e/enterprise/README.md).

## Result and implementation

Explore now has one compact toolbar/table surface, numeric alignment, 52px rows, bookmark actions and quarterly expansion. At 1440 × 900, the first company starts at **310.375px** and **10 company rows** are visible in both themes. Backend ranks and query contracts remain authoritative. Country, sector, sorting and pagination use existing GET parameters; no client ranking or global search was added.

The quarterly chart only includes finite, dated, reported EPS values and requires at least two distinct periods. Estimates remain labeled separately in the always-available quarterly table. Flat lines and extreme finite values are covered by tests. Missing EPS and prices stay `—`; prices are never taken from the legacy score field. Source currency is unavailable in the current rankings response, so the UI identifies price units without inventing a currency.

Home has an editorial hero and product preview drawn from the supplied company response. When that response is unavailable, the preview uses an abstract structure without numbers and a truthful status. Watchlist uses compact company rows, inline group selection and secondary actions. Sign-in has a split desktop composition and a single mobile flow, retaining wallet detection, error reporting and return URLs.

Account, Profile, Access, Credits and Billing share section navigation. Developer keys, usage and docs have their own secondary navigation. Notifications use an inbox surface with read/unread tabs and secondary type/priority filters. Support sits inside the BFF content area, preserving conversation forms and removing the old embedded demo conversation and unsupported marketing claims. Public articles, guide, About, Contact, legal and error pages use the same frontend palette and typography. Legal body text and dates are unchanged from the pre-v2 source snapshot; legacy forced dark styles no longer override the frontend theme.

Implementation owners:

- `apps/frontend/src/enterprise.rs`: single document shell, responsive navigation, page context, section navigation, frontend-only stylesheet and local fixture indicator. `/index` uses the Home marketing shell.
- `apps/frontend/public/enterprise.css`: frontend-scoped tokens, shell, component variants, responsive table/list layouts, forms, editorial pages and states. There are no substring class selectors or general `!important` overrides; `!important` is limited to hidden/reduced-motion behavior. Legacy legal styling remains restricted to hosts without the frontend shell.
- `shared/rust/dioxus_ui/src/enterprise.rs`: frontend page header, data state, company identity, table/mobile list, reported EPS chart, watch action and additional outline icons. Shared default chrome remains available to Admin/Pay.
- Frontend page modules: explicit presentation classes and page-specific composition, retaining original decoding, validation, URLs and mutation hooks.
- `shared/rust/browser-runtime/src/lib.rs`: SSR quarterly row toggle using the existing Rust/WASM runtime. Existing drawer, theme, watchlist and form behaviors are retained; they do not require Dioxus hydration.

The desktop sidebar is 240px and collapses to 72px; below 1024px it becomes a drawer. The top bar is 64px. Light/dark themes use the agreed palettes and saved preferences. Controls have explicit hover, focus, pending, disabled and error presentation; reduced-motion disables transitions and animations.

## Route and state coverage

The final matrix contains **39 URL cases × 4 widths × 2 themes = 312 captures**. Widths are 375, 768, 1024 and 1440px; capture height is 900px. Every case was checked for document horizontal overflow, one main landmark, one page h1, the frontend stylesheet and the selected theme. All 312 checks passed. Representative pages and error states were also visually inspected; automated geometry checks alone are not treated as visual approval.

| Area | Retained routes / cases |
|---|---|
| Public | `/`, `/index`, `/plans`, `/pricing?ref=enterprise-qa`, `/about`, `/contact`, `/manual`, `/terms`, `/privacy`, `/offline` |
| Explore / Watchlist | `/analytics`, `/portfolio`, `/portfolio/:address` |
| Account | `/dashboard`, `/account`, `/profile`, profile Account/Email/Data tabs, `/permissions`, `/account/credits`, `/payment`, `/payment/plan/:id` |
| Inbox / News | `/notifications`, read/unread filters, `/chat`, new conversation, `/chat/history`, `/chat/:id`, `/news`, `/news/:slug` |
| Developer / Auth / Errors | `/developer`, `/developer/docs`, `/developer/usage`, seven-day usage query, `/auth?return_url=%2Fanalytics`, `/access-denied`, unknown-route 404 |

Additional state checks cover ready, empty, malformed, unavailable, backend-limited ranks, restricted signed-in/signed-out, failed filter options with usable data, Home without data and failed watch mutation. A failed watch request never changes the control to saved. Access responses, ranking offsets and plan rules remain backend decisions.

Browser workflows exercised on production-style native SSR with the compiled runtime:

- Apply country/sector/sort, advance to page 2 and retain query parameters and backend ranks.
- Expand quarterly details, inspect the reported chart and numeric table, and use keyboard disclosure on mobile.
- Save AAPL after the backend response, move it to Research and reload to confirm group persistence. Inject a delayed failed watch response and verify pending/error feedback without a saved state.
- Follow sign-in with the Explore query encoded in the return URL; trigger and verify the actual missing-wallet error.
- Open the mobile drawer, make the background inert, wrap Tab/Shift+Tab focus, Escape to close and restore focus to the navigation trigger. Toggle mobile filters.
- Collapse the desktop sidebar to 72px and reload; change theme and reload to verify saved preference.

## Verification

| Check | Result |
|---|---|
| Native frontend build, locked | Passed |
| `cargo test -p epsx-dioxus-ui -p epsx-frontend -p epsx-browser-runtime --lib --bins --locked` | 830 UI + 124 frontend BFF + 10 runtime = **964 passed** |
| `cargo fmt --all --check` | Passed |
| Clippy, changed crates, all targets, locked, `-D warnings` | Passed; existing Cargo duplicate-bin manifest notices remain |
| Rust/WASM runtime build | Passed |
| `cargo xtask assets verify` | Passed, 5 required assets |
| `cargo xtask audit no-node --strict` | Passed |
| `cargo xtask e2e doctor` | Passed |
| Native Chromium E2E runner | **Blocked:** cannot create WebDriver session at `127.0.0.1:4444/session` |
| In-app browser route matrix and workflows | 312 geometry/theme/landmark cases passed; workflow evidence saved separately |
| Core color token contrast | 22 text/background pairs meet 4.5:1; this is not a complete accessibility certification |
| Admin/Pay isolation smoke | Four HTTP checks and visual checks passed; no frontend body class or stylesheet in either application |

Tests were scoped to changed crates, not the entire workspace. The native E2E runner is not reported as passing. The separate in-app browser checks cover the interactions above, not every mutation in every service.

## Remaining contract and test-environment limits

- This review browser has no injected MetaMask provider. A real SIWE signature and full wallet sign-in were not executed. Authenticated follow-up workflows use the verified local fixture session; no production auth gate was removed.
- Dedicated permission details and dashboard account summaries remain unavailable in the current frontend projection path. Verified session details and the working Account access overview are retained. Profile fields remain read-only where no supported write contract exists.
- The premium fixture supplies current read envelopes for plans, account/access, credits, payment history, notification preferences and inbox, developer and support. It does not implement every notification mutation/SSE stream, support send/create/resolve, API-key mutation or payment intent/escrow scenario. Their existing Rust contract tests were run, but full browser mutation coverage is not claimed.
- Checkout was inspected with a local-chain configuration and fixture plan. No transfer, real payment, external support message, API credential creation or push permission request was made.
- Historical fixture envelope mismatches and the missing WebDriver are preexisting test-infrastructure limitations. They were kept separate from v2 visual regressions. During visual review, inherited dark legal styles, credit card spacing, support shell overlap and dark control contrast were fixed and the affected routes recaptured.

## Evidence and separation from other work

All v2 evidence is under `target/premium-fintech-v2/`, separate from `target/enterprise-redesign/` and migration images:

- `before/git-status.txt`, `before/hashes.json`, `before/source/`: exact pre-v2 working-tree state. The two PNGs in `before/` are copied historical v1 captures, **not fresh live pre-v2 screenshots**.
- `after/audit.json` and route/theme PNGs: final 312-case matrix.
- `after/workflow.json`, `after/states.json`, expansion/drawer/failure screenshots: interaction and state evidence.
- `tests.log`, `clippy.log`, `format.log`, `assets.log`, `wasm.log`, `no-node.log`, `e2e-doctor.log`, `e2e-native.log`, `admin-pay-smoke.json`, `contrast.json`, `verification.json`: checks and limitations.
- `changed-files.json` and `redesign-v2.patch`: v2 source changes compared with the captured working tree, avoiding conflation with the repository's existing uncommitted migration/backend/payment work. Review fixture files and this report are listed separately.

Selected screenshots: [Explore desktop](../target/premium-fintech-v2/after/analytics-1440-light.png), [Explore dark](../target/premium-fintech-v2/after/analytics-1440-dark.png), [Explore mobile](../target/premium-fintech-v2/after/analytics-375-light.png), [Home](../target/premium-fintech-v2/after/home-1440-light.png), [Watchlist](../target/premium-fintech-v2/after/portfolio-1440-light.png), [Sign in](../target/premium-fintech-v2/after/auth-1440-light.png).

Design references: [Carbon data table usage](https://carbondesignsystem.com/components/data-table/usage/) for toolbar, expansion and pagination; [W3C Focus Not Obscured](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html) for sticky spacing and keyboard focus visibility.
