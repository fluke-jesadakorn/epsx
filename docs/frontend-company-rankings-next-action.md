# Company rankings and Next action — local delivery

Implemented on 8 September 2026, on top of Premium Fintech v2. Preview: <http://127.0.0.1:3300/>; signed-in fixture preview: <http://127.0.0.1:3301/portfolio>. These previews are explicitly marked **Preview data**. No deployment or production route changes were made.

## Result

- Home uses **Financial Technology Platform**, **Company rankings, made easier.**, the approved description and **Explore rankings** CTA. Its preview shows rank, company and Next action from the existing response.
- Explore shows rank, company, Next action and Save. Company details explain the event date and its source. EPS, growth, prices, quarterly numeric tables and charts are no longer rendered by these frontend components; none is relabeled as a score.
- **Saved companies** replaces frontend-owned Watchlist labels, including navigation, Overview, Sign in and the owner-address route. `/portfolio`, groups and mutations retain their existing contracts. Saved companies does not fetch report dates.
- About and Manual describe rankings and Next action consistently. Frontend plan access labels say Company rankings; shared PlanCard defaults remain unchanged.
- The toolbar offers country and sector, with **Order: EPSX ranking**. Legacy sort and threshold parameters remain hidden form inputs and survive pagination and sign-in links. **Custom view** makes their presence explicit. **Use EPSX ranking** removes the legacy conditions while retaining country/sector; **Clear filters** returns to the unfiltered default.
- Save / Saving… / Saved use canonical membership. Pending controls for the same company are disabled together. An error restores Save and its icon without claiming success. Native SSR sign-in links now retain page, country, sector and custom conditions; this also fixes a preexisting v2 return-link omission.

## Date semantics

`enterprise::NextAction` is presentation-only: `Api(date)`, `Estimated { date, previous }`, or `Unavailable`.

1. A usable `next_earnings_date` Unix-seconds value takes priority.
2. Otherwise, a usable `last_earnings_date` receives exactly 90 calendar days using checked date arithmetic.
3. Missing/sentinel or out-of-range timestamps stay unavailable. Zero/negative timestamps, milliseconds accidentally supplied as seconds, years above 9999 and overflow are rejected.
4. Absolute dates are primary. Today, In N days and Date passed compare UTC calendar dates. The SSR text is preserved by the native browser runtime; browser timezone conversion is not involved.
5. Neither backend `next_quarter_estimate`, formatted/relative helper fields nor `latest_date` supplies a fallback. Passed dates are not advanced and do not imply publication. API dates are not labeled Confirmed.

The Estimated explanation is **Estimated 90 days after the previous company report.** Saving does not create reminders or notification subscriptions.

## Ownership and compatibility

Backend/schema identifiers, proprietary ranking computation, permission and subscription decisions are unchanged. No backend, Admin or Pay application source was changed. New styles load only in the frontend shell. Shared runtime wording changes are conditional on the frontend shell; Admin/Pay defaults are retained.

Backend-owned plan features, notification bodies and existing support messages are preserved verbatim. The current local fixtures therefore still contain “Personal watchlist”, “Watchlist groups” and an older support question mentioning EPS. The route audit records these exact source-owned strings separately. News content, legal substance and technical API documentation are not rewritten. Billing amounts, currencies and payment statuses remain explicit. This presentation change neither hides API fields nor establishes any regulatory exemption.

## Verification

| Check | Result |
| --- | --- |
| Rust UI, frontend BFF and runtime tests | **969 passed**: 835 UI, 124 BFF, 10 runtime |
| New semantic coverage | API priority; UTC day boundary; +90 across months, years and leap day; invalid/missing/overflow dates; Today/past; exclusion of unrelated estimates and numeric score proxies; native return links and custom-query preservation |
| Final runtime-only tests after payment-copy adjustment | **10 passed** |
| `cargo fmt --all --check` | Pass |
| Changed crates' host Clippy, all targets, `-D warnings` | Pass |
| Rust/WASM runtime build | Pass |
| `xtask assets verify` / `audit no-node --strict` | Pass |
| SSR route/copy audit | 39 URL cases; expected statuses, single main, frontend stylesheet, no unexpected legacy product terms |
| Responsive capture | 39 URL cases × 4 widths × 2 themes = **312 unique cases**; no horizontal overflow, one main and expected theme throughout |
| Explore at 1440 × 900 | First row **315.42px** from top; **10 companies** visible with the density fixture |
| Browser workflows | Country/sector Apply; pagination; custom conditions; explicit default reset; event expansion with Enter; preserved sign-in return; Save; group move and reload; pending and backend failure recovery |
| Data states | Empty, malformed, unavailable, restricted signed out/in, independent filter failure; Home unavailable preview has no invented numbers |
| Keyboard and contrast | Drawer opens with Enter, makes main inert, closes with Escape and restores trigger focus; desktop/mobile details work with Enter; focused desktop toggle remains below the sticky header. Saved-state badge contrast improved to **4.90:1 light / 9.85:1 dark** |
| Admin / Pay smoke | Existing Admin access gate and Pay landing render; frontend shell and stylesheet absent |
| Local services | Backend, existing frontend/Admin/Pay development servers, and all four review endpoints respond HTTP 200 |

Representative screenshots were inspected directly for Home, Explore (including mobile and compact desktop), company details, Sign in and Saved companies. Geometry checks across the full matrix do not imply a pixel-by-pixel human review of every long page. Saved companies screenshots were refreshed after the final contrast adjustment.

## Known test limits and baseline findings

- **Native Chromium E2E cannot create a WebDriver session at `127.0.0.1:4444/session`.** The same environmental blocker was present in v2. Browser workflow evidence is separate; the native runner is not reported as passing. `e2e doctor` passes.
- An additional WASM-target Clippy run reports **three preexisting diagnostics**: `possible_missing_else` in `browser/merchant_pay.rs`, `useless_format` in the existing payment error handler, and `let_unit_value` in the existing file-input click handler. Their source text is present in the exact before snapshot; see `wasm-lint-baseline.json`. These unrelated sections were left intact. Host Clippy and the WASM build pass.
- Real wallet signing is unavailable in the in-app browser. The missing-MetaMask error was verified. Signed-in operations use the repository's generated, short-lived fixture session; no real wallet signature or transfer was made.
- The fixtures cover read contracts and stateful saved-company/group mutations. Complete payment settlement, support-message mutations, notification mutations and developer-key mutations are not claimed as end-to-end validated here. Existing Rust tests cover their contract handling.
- Cargo's duplicate Dioxus binary-target warnings predate this change. Initial failed assertions during implementation expected the former financial columns and Watchlist wording; those assertions were updated to verify the new approved behavior.

## Evidence and reproduction

Evidence lives in `target/company-rankings-next-action/`, separate from `target/premium-fintech-v2/` and migration references:

- `before/source/`, `before/git-status.txt`, baseline Home/Explore screenshots: exact pre-change working files, including existing uncommitted work.
- `source.patch`, `changed-files.json`: this round compared with that snapshot, not the repository's unrelated migration diff.
- `after/audit.json`, `routes-copy.json`, `workflow.json`, `states.json`, `contrast.json`, `admin-pay-browser.json` and screenshots.
- Individual Rust test, build, format, Clippy, WASM, asset and E2E logs; `service-health.json`.

Use the preview setup in `e2e/enterprise/README.md` with **`EPSX_QA_PREMIUM=1 EPSX_QA_NEXT_ACTION=1`** for the adapter. The second flag adds API, estimated, missing, invalid, Today and passed-date fixtures without changing v2's default fixture mode. It is fixture-only and is never used by application code. Run `python3 e2e/enterprise/rankings_audit.py` for the SSR audit. Restore fixture mode `healthy` and `{"faults":[]}` after state tests; both were restored for handoff.
