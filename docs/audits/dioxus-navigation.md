# Dioxus navigation audit

The Frontend, Admin and Pay use Dioxus 0.7.9 fullstack roots, hydrated by WASM. Direct HTTP requests still use the existing native Axum BFFs, server functions, authentication checks and response status handling. `ServeConfig::new()` uses disabled streaming: server futures resolve before the SSR response is sent. Adding client suspense boundaries does not switch these pages to client-only rendering.

## Changes

- `navigation::AppLink` preserves an HTML href in standalone SSR. Within a Router, ordinary local links use Dioxus Link; links with action callbacks preserve cancellation and use the router only when the action has not already handled navigation. Modified clicks and non-primary clicks retain browser behavior. External targets, downloads and API/assets endpoints remain document/browser operations.
- Frontend's migration allowlist is replaced with parsing the actual `FrontendRoute`. This includes previously omitted aliases and dynamic routes and strips fragments before route classification. Profile loading is isolated in the header; page loading has a separate skeleton boundary inside the persistent shell.
- Admin now has a route layout. Existing page shells yield their content when the outer shell owns navigation. Authentication changes update shared authentication state and invalidate page content instead of remounting the Router. Admin's nested return URL is read from the original router URL, preserving encoded separators.
- Pay's route layout owns its navbar, theme and loading boundary. Pages retain their contextual account/actions toolbar. Fragment links use the router when moving between Pay pages.
- The scroll/focus adapter observes the browser history changes made by Dioxus. It does not intercept document clicks, fetch pages, or implement a second router. New paths focus the content and scroll to the top; query updates preserve position; Back/Forward use Dioxus history positions. Fragment scrolling waits for the destination content. Pending restoration survives skeleton rendering.
- Same-origin absolute links and query-relative links are normalized using the hydrated browser origin, preserving encoded nested queries. Cross-origin links retain native navigation.
- Admin Wallets search/status are restored from the URL and submissions preserve the backend limit. Wallets and Credits server futures explicitly subscribe to route props; they no longer render a one-time copy of the initial response after query navigation. Successful mutations restart the current resource.
- The chat-history GET filter uses a shared query form. Existing hydrated mutation forms already call typed server functions and prevent native submission; their backend semantics are unchanged.

## Inventory and remaining native behavior

Run `python3 scripts/audit/navigation_inventory.py`. The generated `target/navigation-audit/inventory.json` lists file/line evidence for routes, links, forms, navigation calls, redirects and suspense. Counts are source lines/candidates, not claims of live-page coverage. Route declarations include aliases and wildcard routes. Authored UI anchors use the adapter; its underlying anchor is intentionally retained.

Native document navigation remains intentional for:

- External origins (including crossing Frontend/Admin/Pay origins), checkout handoff, downloads, API responses, mail links and explicit new-tab targets.
- User-requested reload/error recovery and offline runtime recovery.
- Standalone SSR renderers outside a Router, and no-JavaScript form fallbacks.
- Existing server-side redirects on initial/deep HTTP requests. Router navigation handles the in-app counterparts.

The older `shared/rust/browser-runtime` still contains reload-based mutation and authentication behavior. It is retained as legacy code, not loaded by the active fullstack roots. The native entrypoints use `FullstackState::render_handler`; Admin and Pay already have tests rejecting the legacy bootstrap in fullstack documents. Browser verification also checks that no legacy bootstrap is loaded.

Unbound forms in the old admin wallet/access/credit/plan page renderers are legacy SSR forms: current Router views use CoreWallets/CoreCredits, HydratedCatalog and the hydrated wallet controllers. `NewConversationPanel` returns the hydrated composer when ChatControls exists; its other forms are standalone/no-JS fallbacks. Payment-links SSR components are separate from Pay's current typed actions. These paths were not converted to arbitrary fetch-and-replace HTML.

## Validation evidence

Static inventory and browser evidence are generated under `target/navigation-audit/`. `e2e/enterprise/dioxus_navigation_audit.py` crawls actual app links in an isolated browser session, adds server-function latency, checks document and shell identity on every transition, samples intermediate frames, checks direct SSR responses and captures mobile/desktop evidence. It reports unvisited edges explicitly. Existing session/payment/wallet fixture audits provide mutation and authenticated coverage separately.

Browser evidence is fixture-based: upstreams use ephemeral local tokens and deterministic responses, including unavailable/forbidden data. It is not evidence that every production permission combination or external provider succeeds. Every route declaration has a direct SSR sample; dynamic routes use representative IDs. The deep-hydration pass additionally opens each sample, waits for the browser lifecycle and content, then verifies a home/brand click retains its document and shell. A direct open is intentionally a new document.

Admin authentication regression: 4 checks passed (rejected signing, SIWE/HttpOnly audience isolation, Router return, logout and responsive auth). Canonical Admin wallet regressions pass both filter/limit/history persistence and rapid Next/Back with a delayed request.

Session regression: 8 checks passed (refresh success/failure, SIWE return query, limit/backend cap, out-of-order response after Back, expired reopening, intentional guest pagination). Disconnect regression: 5 checks passed (origin protection; upstream 200/401/503; sign-in after disconnect). Payment action regression: 20 checks passed, including mutations, checkout, wallet rejection/wrong chain, duplicate submission and responsive layout. Scroll/focus adapter: 5 browser checks passed.

Native UI unit suite: 871 passed, 1 existing content assertion failed: `pages::profile::tests::fixture_values_and_unsupported_controls_are_absent` expects “Developer tools”, which the current profile content does not contain. That unrelated assertion was not weakened. The navigation unit tests pass. Formatting, inventory guard, strict no-node audit and asset verification pass. The older admin wallet fixture expects a superseded UI/API; canonical wallet search/pagination is tested by the new navigation fixture instead.

One deep-link browser `open /payment` command timed out during concurrent runs; a complete repeat passed. The first session run during concurrent builds timed out; an instrumented run and three sequential repeats passed, followed by the final session run. The new canonical wallet test uncovered an actual stale page response (URL page 1 while Next used page 2 data); this drove the reactive-resource correction rather than relaxing the expected page number.

No production deployment, database change or production route change is part of this work. External checkout handoffs, downloads and real wallet-provider popups are not executed by the navigation crawl; their native behavior is classified in the source audit, and modified-click defaults are checked separately.


## Reproduce

```sh
python3 scripts/audit/navigation_inventory.py --check
python3 e2e/enterprise/navigation_fixture.py frontend authenticated
python3 e2e/enterprise/navigation_fixture.py frontend guest
python3 e2e/enterprise/navigation_fixture.py admin authenticated
python3 e2e/enterprise/navigation_fixture.py admin guest
EPSX_NAV_DEEP_ONLY=1 python3 e2e/enterprise/navigation_fixture.py frontend authenticated
EPSX_NAV_DEEP_ONLY=1 python3 e2e/enterprise/navigation_fixture.py frontend guest
EPSX_NAV_DEEP_ONLY=1 python3 e2e/enterprise/navigation_fixture.py admin authenticated
EPSX_NAV_DEEP_ONLY=1 python3 e2e/enterprise/navigation_fixture.py admin guest
python3 e2e/enterprise/navigation_lifecycle_audit.py
python3 e2e/enterprise/dioxus_analytics_session_audit.py
python3 e2e/enterprise/dioxus_logout_audit.py
```

For Pay, start the existing ignored `pay_browser_fixture` Rust test with `DIOXUS_PUBLIC_PATH` pointing at `target/dx/epsx-pay/debug/web/public`, then run `dioxus_navigation_audit.py pay http://127.0.0.1:43127` and `dioxus_pay_audit.py`. Use `EPSX_NAV_DEEP_ONLY=1` for direct hydration coverage. Fixture processes are temporary and do not modify production data.


## Route coverage (14 September 2026)

| Application / fixture | Crawl route records | Direct SSR samples | Direct hydrated samples | Blank / theme-flash frames |
| --- | ---: | ---: | ---: | --- |
| Frontend authenticated | 35 | 34 | 34 | 0 / 0 |
| Frontend guest | 18 | 34 | 34 | 0 / 0 |
| Admin authenticated | 24 | 77 | 77 | 0 / 0 |
| Admin guest | 9 | 77 | 77 | 0 / 0 |
| Pay merchant/public-page fixture | 13 | 18 | 18 | 0 / 0 |

The route-by-route statuses and landed URLs are in each fixture directory's `*-ssr.json`, `*-deep-hydration.json` and `*-browser.json`. Some fixture routes intentionally return 401/403/404/502; their rendered error/auth state remains inside the shell. The crawl checks 250 ms server-function latency and records skeleton frames, document/shell identity, Back/Forward, modified clicks, menu closure and mobile overflow. Desktop/mobile screenshots are stored alongside the JSON. Unvisited link edges are recorded rather than counted as traversed; their route families are covered separately by direct hydration samples.


Crawl counts include the initial route record; form/history/mobile checks are additional. Across the five runs, 2,628 animation frames were observed, including 1,012 skeleton frames, with zero disconnected/empty-shell or theme-change frames. Page-error capture was empty. The Pay fixture covers merchant and public checkout/storefront pages with a deterministic provider; it does not independently prove real Pay cookie expiry or third-party wallet-extension behavior.

All three final DX builds completed. Local ports 3000/3001/3002 and `dev.epsx.io`, `dev-admin.epsx.io`, `dev-pay.epsx.io` returned HTTP 200 with an SSR main root. Each dev domain's `/_dioxus` handshake returned HTTP 101. Evidence: `target/navigation-audit/dev-http.json`. The existing three UI LaunchAgents remain running; temporary fixture servers are stopped after verification.
