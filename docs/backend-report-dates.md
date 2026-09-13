# Backend report-date recovery — local only

## Coverage finding

Live checks on 8 September 2026 confirmed that `SET:CPR`, `MYX:BRIGHT`, and `IDX:APIC` have no current or previous release date in the connected TradingView feed. The generic, quarterly, annual, and calendar fields were checked using exchange-qualified symbols. Companies on other exchanges with the same ticker were deliberately not substituted.

The old API returned a separate estimate even when both release fields were null. That estimate used the current quarter and a symbol hash; it was not a report date and did not satisfy the agreed previous-report-plus-90-days rule.

## Changes

- Request the quarterly previous and next release dates and annual next release date in the existing scanner request. Append columns so ranking-related offsets do not change.
- Share validated date extraction between the scanner and legacy mapper. Keep explicit next dates even when today or passed. Recover the most recent previous release from the available reported-date fields.
- Preserve ISO calendar-date formatting in the batch-symbol adapter; raw timestamp strings previously failed downstream date parsing.
- Replace the legacy symbol-based date estimate with exactly 90 UTC calendar days after a supplied previous report. Mark the estimate explicitly; keep raw `next_earnings_date` reserved for provider-supplied dates so the frontend continues to distinguish estimates correctly.
- Use UTC calendar-day differences consistently and clamp the legacy progress percentage to its valid range.
- Remove the disabled WebSocket estimation path, which could not recover report dates and required an unnecessary blocking runtime call.

No API keys, company-specific dates, fiscal period ends, or request-time dates were added as release dates. The initial date-recovery change did not alter ranking eligibility. The later eligibility change is recorded below. Neither change requires a database migration, frontend redesign, or production deployment.

## Ranking eligibility — subsequent user request

CPR, BRIGHT, and APIC cannot acquire a supported date from the current feed. The user subsequently requested that companies without usable dates be excluded from ranking responses.

- The backend scanner now requires at least one valid previous or next report-date field before provider sorting, pagination, and `totalCount`. It combines this condition with the existing stock-type, country, sector, and ranking filters. It does not drop rows from an already-paginated response.
- Dates must be positive Unix seconds within the supported calendar range. Explicit next dates retain priority, including Today and Date passed. A previous report alone remains eligible for the existing labeled +90 UTC calendar-day estimate.
- A defensive response validator rejects malformed provider pages that violate the date filter, rather than returning undated cards or incorrect pagination totals. Generic symbol lookup remains available even when the symbol has no report date.
- This changes the eligible ranking population and its total; positions are assigned by the backend in the resulting order. Public access continues at rank 100 without an inventory cap, with ten companies per page. Existing authenticated plan rules remain unchanged.
- Missing-date UI remains defensive; the backend ranking list no longer intentionally sends companies that would require it. No dates were fabricated, and no underlying company records were deleted.

Evidence for this subsequent change is kept separately in `target/dated-rankings/`, including the direct provider proof that excludes the three uncovered companies while retaining FNWB and 4635.

### Eligibility verification

- Backend library suite: 572 passed, 10 ignored, no failures. Formatting, host Clippy, and the native backend build passed. Existing duplicate Cargo target warnings remain.
- Restarted only the local backend on `:8080`; the real-data preview on `:3300` continues to use it. API pages 1, 2, and 20 each return ten companies with a usable next action date. At verification time, the eligible public inventory was 7,726 companies; page 773 contained the final six with `hasNext: false`.
- Followed pagination links in actual SSR HTML. Pages 1 and 2 each contain ten cards and no `Date not available` text. Country filtering also returns ten dated cards and preserves its filter in the Next link.
- Admin and Pay HTTP smoke checks returned 200. No browser interaction or authenticated payment E2E was run in this backend-only change.
- A manually entered page beyond the last available page still returns 502 because the provider rejects an out-of-range interval with 400. A direct request without the new date filter reproduces that existing provider behavior. The valid final page works and disables Next. Evidence: `provider-existing-range-boundary.json` and `verification.json`.

Logs and isolated changes are saved as `backend-tests.log`, `backend-clippy.log`, `backend-build.log`, `live-verification.log`, and `backend-only.patch` in `target/dated-rankings/`. No production deployment was performed.

## Evidence

- `target/report-date-fix/before-api.json`: original live rankings response.
- `target/report-date-fix/provider-before.json`: the same fields directly from the provider.
- `target/report-date-fix/provider-report-fields.json`: additional report-date field probe.
- `target/report-date-fix/provider-calendar-fields.json`: fiscal-period calendar fields; these are not publication dates and are not used as fallbacks.
- Provider field catalog: https://github.com/tradingview/scanner_data/blob/master/scanner.qf.json
- `target/report-date-fix/backend-tests.log`: 568 passed, 10 ignored, no failures.

The tests cover date precedence, missing/invalid timestamps, quarterly field recovery, existing response mapping, UTC Today/Date passed, leap day/year transitions, and estimates that do not move with request time or company symbol.

## Initial date-recovery verification (before eligibility filtering)

- Backend unit suite: 568 passed, 10 ignored, no failures.
- Backend host Clippy (`--lib --bin epsx -- -D warnings`), workspace formatting, native binary build, and asset verification passed. Existing duplicate-target Cargo manifest warnings remain.
- The updated backend runs at `127.0.0.1:8080`, and the review BFF at `127.0.0.1:3300` still uses it. Live ranks, pagination, and access boundaries match the pre-change response. FNWB reports 50 UTC calendar days and 4635 reports 63; the three uncovered companies remain unavailable.
- No full authenticated/payment E2E was run in this backend-only change. Native browser inspection was unavailable because the Mac was locked; the SSR preview was checked over HTTP.
- Local startup also reported unavailable RPC `:8545`, an S3 bucket connection failure, and a notification-cleanup schema warning; these paths were not changed by the report-date patch.

Live API and SSR evidence is stored in `target/report-date-fix/after-api.json`, `after-preview.html`, and `after-smoke.json`. Local app process identifiers are in `dev-services.json`; the backend runtime log is `backend-runtime.log`.
