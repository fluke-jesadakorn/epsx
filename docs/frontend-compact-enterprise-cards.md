# Explore Rankings — Compact Enterprise Cards

Delivered locally on 8 September 2026. The current public preview at <http://127.0.0.1:3300/analytics> connects to the real native backend on port 8080. The fixture badge is absent and the old generated-session proxy on 3301 is stopped. Earlier fixture-based verification below is historical. No deployment or production route changes.

## TradingView action — latest follow-up

At the user's request, Explore's View details action now opens `https://www.tradingview.com/symbols/{symbol}` in a new tab, matching the inspected production link. It is a native anchor with an encoded symbol, `noopener noreferrer`, and an accessible label announcing the destination and new tab. The former company dialog is no longer rendered. The card design, report dates, progress bars, Save controls, filters and pagination remain in place. The dialog behavior described in the earlier delivery sections below is historical.

This is a frontend presentation change; it does not change the backend or deployment. Evidence and validation logs are separate in `target/tradingview-details/`.

Validation: 12 presentation tests, host Clippy, formatting, native frontend build and asset verification passed. The live SSR previews on 3300 and 3000 each render ten matching symbol links. A Chrome click opened TradingView's SZSE-002990 page in a new tab while retaining rankings page 2. Admin/Pay HTTP smoke checks passed; authenticated mutation and fixture fault-injection workflows were not rerun for this link-only change.

## Calendar progress and real backend

Explore cards and company dialogs now include a native progress bar for elapsed calendar time between the previous company report and the next report date. API dates use the actual interval when both dates are usable. Estimated dates retain the previous report +90-day rule. Missing, invalid or inconsistent intervals show a neutral track with an unavailable label, never a fabricated percentage. The bar caps at the target date; a passed date uses a muted bar and does not claim publication. The symbol remains the largest card element.

The user-facing preview now uses the same backend, issuer/JWKS, notifications, Admin and Pay URLs as the native development stack. No test wallet token or placeholder payment receiver is supplied. Anonymous access currently returns backend ranks 101–105, with BRIGHT, FNWB, 4635, CPR and APIC; the frontend does not override this access range. Real API data replaces the earlier sample companies. The current startup command is in `e2e/enterprise/README.md`. Evidence is separate in `target/report-progress-bars/`.

The twelve presentation tests pass, including real-length calendar intervals, +90-day estimates across leap day, UTC boundaries, invalid/missing dates, and date-passed semantics. Browser verification uses the real anonymous API; it does not claim a real wallet signature or an authenticated Save mutation.

## Symbol emphasis refinement

The follow-up makes the company symbol the visual focus: centered **42px** blue symbols on cards and **52px** in the detail panel. Full company names sit below, with Next action reduced to 22px. Rank badges, dividers, tinted actions and subtle shadows provide clearer hierarchy. Current normal desktop cards are **288px**; eight remain fully visible at 1440 × 900. All five breakpoints in both themes pass overflow checks. The existing nine presentation tests, format, native frontend build and asset verification pass. No runtime or backend logic changed. New screenshots and the exact before/after patch are separate in `target/ranking-symbol-refinement/`. The original delivery evidence below remains the baseline for this refinement.

## Result

Explore now uses one responsive company-card grid: one column below 768px, two from 768px, three from 1280px and four from 1440px. Each card shows backend Rank, company name, symbol, days-first Next action, full report date, Estimated where applicable, Save and View details. Normal desktop cards are 280px tall with 16px padding/gaps and 12px corners. No EPS, growth, price, score, chart or progress bar was added.

The existing country/sector toolbar, backend order and pagination remain. Legacy custom conditions survive forms, pagination and sign-in returns; the explicit Use EPSX ranking link clears custom ordering/thresholds while preserving country/sector.

View details opens a native modal dialog using the existing Rust/WASM runtime. It is a 480px right panel at 768px and above, and full-screen below 768px. Details use the already loaded response. Close, Escape and backdrop dismiss it; background scrolling is locked and focus returns to the original card. Forward/reverse Tab wrap within the dialog. A pointer gesture must start and finish on the backdrop to dismiss it.

Card and dialog Save controls share canonical membership. During a request both show Saving… and aria-disabled; a runtime busy guard rejects repeats while preserving keyboard focus. Only a confirmed symbols list updates Saved. Failure or HTTP 200 without the requested membership change preserves the previous state and shows an error. The panel stays open without reload. Existing consumers retain their default mutation behavior.

## Compatibility

Only Explore opts into `RankingCards` and in-place Save. Home retains its existing preview/table and absolute-date-first presentation. The 1440px light Home screenshot is **pixel-identical** to the before screenshot. Date rules remain API first, previous report +90 calendar days otherwise, UTC Today/Date passed, and unavailable for missing/invalid dates. Past dates do not imply publication. The reusable date-source explanation preserves its content.

No backend contract, rank/access calculation, permission, subscription rule, URL, alias, auth gate or return-path contract changed. Admin and Pay receive no frontend shell, card stylesheet or dialog markup. The new runtime module is activated only by frontend-specific hooks. The frontend stylesheet URL was versioned for cache invalidation; the route audit now checks for the versioned stylesheet without pinning a historical revision.

## Verification

| Check | Result |
| --- | --- |
| Rust UI / frontend / browser runtime | **973 passed**: 838 UI, 124 frontend BFF, 11 runtime |
| Final runtime tests after focus refinement | 11 passed |
| Format, scoped host Clippy with `-D warnings` | Pass |
| Rust/WASM runtime build | Pass |
| Asset verification / strict no-node audit | Pass |
| Native E2E doctor | Pass |
| SSR route/copy audit | 39 routes, no failures |
| Explore state audit | 8 cases pass: ready, empty, malformed, unavailable, limited ranks, filter failure and signed-out/in restrictions |
| Responsive cards and panels | 375, 768, 1024, 1280, 1440px × light/dark; no horizontal overflow; expected column count; native panel width; focus wrap and restoration |
| 1440 × 900 density | First card at **286.92px**; **8 complete cards**; normal card height **280px** |
| Mobile card actions | Save and View details are 44px high |
| Measured card text contrast | Minimum **4.75:1**, including Estimated, muted text and CTA across both themes |
| Browser workflows | Filters, page 2, custom view/reset, sign-in return retaining all conditions, details, Save from card/panel, pending focus, confirmed save/remove, failed removal, unacknowledged response, group move surviving reload |
| Modal interactions | Close, Escape, backdrop, forward/reverse Tab, focus restoration and background scroll lock |
| Home | 1440px light before/after pixel comparison: no differing pixels |
| Admin/Pay smoke | Rebuilt native binaries serve existing surfaces without frontend styles/dialogs |
| Service health | Backend `/health`, development frontend/Admin/Pay and all four review endpoints respond 200 |

Screenshots of desktop light/dark, mobile cards and long-name/past/estimated detail panels were inspected directly. The responsive matrix also records geometry and focus checks. Reduced-motion rules disable the new transitions and panel entrance animation.

Two issues discovered during browser testing were fixed before handoff: native Tab could leave the last dialog control in this browser, and setting native disabled on Save discarded focus. The scoped focus loop and aria-disabled busy controls address these cases; final browser checks pass.

## Existing limits, separate from this change

- **Native Chromium E2E remains blocked** because a WebDriver session cannot be created at `127.0.0.1:4444/session`. The browser evidence above is not reported as a passing native runner.
- **WASM-target Clippy still reports three existing diagnostics** in unrelated code: `possible_missing_else` in `browser/merchant_pay.rs`, `useless_format` in the payment error handler, and `let_unit_value` in the file-input click handler. The prior baseline and exact before files identify these independently of the new module. Host Clippy and WASM compilation pass.
- The in-app browser has no real wallet provider. Return URLs and generated authenticated fixture sessions were exercised; an external wallet signature is not claimed.
- Pay's landing page renders with an existing `fixture_route_not_implemented` banner because the historical fixture does not cover its merchant endpoint. This smoke check establishes rendering/isolation, not merchant or payment settlement functionality. Admin retains its existing guest/access behavior. Neither app's source was changed for this task.
- Cargo duplicate Dioxus binary-target warnings and the repository's unrelated, preexisting migration changes remain separate.

## Evidence and reproduction

Use the setup in `e2e/enterprise/README.md`, with `EPSX_QA_PREMIUM=1 EPSX_QA_NEXT_ACTION=1`. Run `python3 e2e/enterprise/cards_audit.py` for the state/smoke audit. The test-only adapter supports delayed and unacknowledged watchlist mutations; these modes never enter the application.

Evidence is in `target/compact-enterprise-cards/`, separate from previous redesign and migration artifacts:

- `baseline/`: exact pre-change working source, Git status and Home/Explore screenshots.
- `source.patch`, `changed-files.json`: this task's changes relative to the captured working files.
- `after/explore-{width}-{theme}.png`, `after/details-{width}-{theme}.png`, `after/responsive.json`: ten viewport/theme cases for both cards and panels.
- `after/workflow.json`, mutation screenshots, state screenshots, `after/states-ssr.json`, `after/states-browser.json`, `after/routes-copy.json`, contrast evidence and Admin/Pay smoke screenshots.
- Test/build/lint/asset/E2E logs, prior WASM lint baseline and `service-health.json`.

Fixtures were restored to healthy mode with no active faults before handoff. Public and authenticated local previews remain running.
