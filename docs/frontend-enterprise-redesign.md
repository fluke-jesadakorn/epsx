# EPSX frontend enterprise redesign

Implemented locally on 8 September 2026. The frontend uses a Clean Finance presentation across public pages, the workspace and wallet sign-in. Admin and the separate Pay application keep their own shells and styles. No deployment or production routing change is included.

## Structure and implementation

- `apps/frontend/src/enterprise.rs` owns the frontend document, marketing header/footer, responsive workspace sidebar, account menu and section navigation. The BFF emits the shell once. URL aliases, verified-session gates and sign-in return targets are retained.
- `apps/frontend/public/enterprise.css` is served only by the frontend and is bundled into its native binary. All selectors are scoped to the frontend. Tokens cover light/dark colors, 4px spacing, 8px controls, tabular numbers, focus indicators and mobile touch sizes. Existing frontend form, tab, dialog, badge and state primitives receive the same presentation without changing shared Admin/Pay defaults.
- `shared/rust/dioxus_ui/src/enterprise.rs` supplies semantic page headers, data states, the Home hero, company tables/mobile lists and native quarterly disclosures. Existing form actions and Rust/WASM mutation hooks remain in use.
- `shared/rust/browser-runtime/src/lib.rs` owns sidebar collapse/drawer behavior and focus management. Mobile drawers trap keyboard focus, make background content inert and return focus on Escape. Saved theme precedes system theme; frontend fallback is light. Fixed support bubbles are hidden in this shell because Support is available through navigation and the bubbles can obscure focus.

Home uses “Explore company data, at your pace.”, real public projections, an explanation of the workflow and backend plan projections. Explore uses server-provided ranks, native GET filters and pagination, a desktop comparison table and mobile company summaries. No client sorting, global search, price proxy, fabricated KPI or optimistic saved state is introduced. Watchlist keeps the existing grouped organization and mutations. Overview links to Explore, Watchlist and Account using verified session information. Account/Access/Credits/Billing and Developer have consistent section navigation. About and the guide have been rewritten around supported behavior. Legal text is unchanged.

## Route inventory

| Shell / section | Routes retained |
|---|---|
| Marketing | `/`, `/plans`, `/about`, `/contact`, `/manual`, `/terms`, `/privacy`, `/offline` |
| Auth | `/auth` with the existing return URL and recovery states |
| Workspace | `/analytics`, `/portfolio`, `/portfolio/:address`, `/dashboard`, `/news`, `/news/:slug`, `/notifications`, `/chat`, `/chat/history`, `/chat/:id` |
| Account | `/account`, `/profile`, `/permissions`, `/account/credits`, `/payment`, `/payment/:ptype/:pid` |
| Developer | `/developer`, `/developer/usage`, `/developer/docs` |
| Aliases and errors | `/pricing` redirects to `/plans` preserving query; `/access-denied`; catch-all 404 |

The initial inventory is saved in `target/enterprise-redesign/before/routes.json`. Public, signed-out, verified-session, ready, empty, malformed, unavailable and restricted states are retained where the corresponding route supports them. A 403 from the analytics API is presented as restricted; the frontend does not infer access from permission strings or compute ranking offsets.

## Data and API boundaries

EPS and price remain absent when the response does not contain a valid figure. The legacy score field and growth-factor ratio are never displayed as price or percentage growth. Reported and estimated quarters remain distinct. Request timestamps are not presented as market update times. The current rankings response has no currency field, so prices are explicitly labeled as source units.

Profile mutations, canonical Access details, and some account summaries are unavailable in the current frontend contracts. The UI explains the missing data and exposes only available actions. Plan pricing, payment state, permissions, feature access and subscription decisions remain backend-owned. Existing mutation validation and owner/session verification are retained.

## Verification and evidence

Evidence lives under `target/enterprise-redesign/`, separate from migration artifacts:

- Initial repository status, source snapshots/hashes and 13 SSR HTML captures are under `before/`. Sixteen reference PNGs were captured from the existing development binary on port 3000; source snapshots were taken before redesign edits.
- `after/` contains 248 route/viewport/theme images and a machine-readable audit. An additional in-app browser audit checked 200 route/viewport/theme combinations without horizontal overflow or duplicate main landmarks.
- `states/` contains screenshots and assertions for the existing analytics fixture's empty, malformed, unavailable and limited-rank responses.
- Native UI and frontend BFF tests cover contracts, authorization, query preservation, data states and mutations. Browser runtime tests and WASM compilation cover the retained runtime. Format, clippy, asset verification and the strict no-node audit were run. Exact final results are recorded in `target/enterprise-redesign/verification.json`.
- Final result: 828 UI tests, 124 frontend BFF tests and 10 browser runtime tests passed (962 total). All 248 Chromium route/viewport/theme checks passed. The four analytics fixture state checks and six Admin/Pay isolation smoke checks passed. Frontend build, WASM build/check, format, clippy with warnings denied, asset verification, strict no-node audit and scoped whitespace checks passed.
- Fresh local Admin/Pay binaries were smoke-tested on separate ports. Neither receives the enterprise stylesheet or frontend body class. Their existing source and production processes were not changed by this redesign.

Browser checks exercised filters, server pagination, quarterly expansion, sign-in return navigation, watchlist add/group/move persistence, keyboard drawer trapping and focus restoration, theme persistence and desktop collapse. These checks use local fixture data, not a live payment or external wallet signature.

The sign-in action was also checked in a browser without an injected wallet: it presents a visible missing-wallet alert and does not claim sign-in success. Runtime files are served with `Cache-Control: no-cache`, so existing browsers revalidate the generated Rust/WASM module graph. The review preview uses port 3300 for signed-out access and port 3301 for the generated fixture account.

The old migration E2E runner could not create its Safari session because Remote Automation is disabled. Chromium viewport captures and in-app browser checks provide separate coverage; the migration suite is not reported as passing. Full-page Chromium capture also timed out, so the stored baseline uses viewport images. Older fixture envelopes for several secondary services do not meet current strict projections and render unavailable; existing Rust tests cover their ready and mutation states.

See `e2e/enterprise/README.md` for reproducible local setup. Existing native migration work in the working tree was preserved. Cargo lockfile churn from concurrent workspace work briefly blocked a locked build; no dependency was added for the redesign.

`target/enterprise-redesign/redesign-files.json` records the scoped before/after source hashes; `redesign-source-comparison.patch` compares against the captured working tree instead of Git HEAD. Concurrent payment-adapter changes are excluded. Shared files can overlap native migration work, so the original snapshots remain the reference when separating commits.

Design references: [Carbon data table guidance](https://carbondesignsystem.com/components/data-table/usage/) informed the toolbar/table/pagination structure; [W3C focus visibility guidance](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html) informed drawer focus and sticky spacing. These references are design guidance, not a claim of a complete accessibility certification.
