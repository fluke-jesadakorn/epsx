# Enterprise frontend QA

## Current product review uses the real backend

`http://127.0.0.1:3300/analytics` now connects directly to the existing native backend at `http://127.0.0.1:8080`, with notifications at `http://127.0.0.1:8106`, Admin at `http://localhost:3001` and Pay at `http://localhost:3002`. Keep this user-facing port on the real backend. Do not enable `EPSX_UI_PREVIEW=fixture` here or replace its data with fixture companies. The old generated-session proxy on 3301 has been stopped; real authentication uses the backend's issuer and JWKS.

To restart this frontend once the native development services are running:

```sh
ENV=development EPSX_ENV=development HOST=127.0.0.1 PORT=3300 API_URL=http://127.0.0.1:8080 BACKEND_URL=http://127.0.0.1:8080 OIDC_ISSUER=http://127.0.0.1:8080 OIDC_JWKS_URL=http://127.0.0.1:8080/.well-known/jwks.json ADMIN_FRONTEND_URL=http://localhost:3001 PAY_FRONTEND_URL=http://localhost:3002 NOTIFICATION_SERVICE_URL=http://127.0.0.1:8106 ./target/debug/bff-frontend
```

The fixture recipes below describe earlier isolated QA runs. Use a separate port for any future fixture server; they are not the current product preview. Evidence for the real backend and calendar progress bars is in `target/report-progress-bars/`.

## Compact Explore cards review

The card review uses the same `EPSX_QA_PREMIUM=1 EPSX_QA_NEXT_ACTION=1` fixture setup, with public Explore on `http://127.0.0.1:3300/analytics` and a generated fixture session on `http://127.0.0.1:3301/analytics`. Run `python3 e2e/enterprise/cards_audit.py` for the eight SSR states and Admin/Pay smoke checks; it restores healthy fixtures afterward. Evidence and screenshots are separate in `target/compact-enterprise-cards/`. See `docs/frontend-compact-enterprise-cards.md` for the full results.

The adapter also accepts `watchlist-delayed` (one-second response delay) and `watchlist-unacknowledged` (HTTP 200 with unchanged canonical membership) through the existing local `__qa/state` control. These are test-only fault modes for pending and confirmation checks. Restore `{"faults":[]}` before handoff. Browser screenshots use 375/768/1024/1280/1440px in light/dark; native WebDriver availability is reported separately.

## Company rankings / Next action review

The follow-up uses the same loopback preview setup below. Start the adapter with `EPSX_QA_PREMIUM=1 EPSX_QA_NEXT_ACTION=1 python3 e2e/enterprise/fixture_adapter.py` to enable report-date cases. Run `python3 e2e/enterprise/rankings_audit.py` for the native SSR route/copy audit. Evidence is separate in `target/company-rankings-next-action/`; see `docs/frontend-company-rankings-next-action.md` for outcomes, ownership exceptions and test limits. Without the extra flag, v2 fixture behavior remains unchanged.

## Premium Fintech v2 review

The v2 preview and evidence are separate from the historical v1 scripts below. See `docs/frontend-premium-fintech-v2.md` and `target/premium-fintech-v2/`. The final v2 matrix was captured with the in-app browser at 375/768/1024/1440px in light/dark themes, covering 39 URL cases (312 images). `after/audit.json` records the exact URLs and image names.

Build the frontend and browser runtime and start the Rust fixture on 48081 using the commands below. Then start these three processes in separate terminals:

```sh
EPSX_QA_PREMIUM=1 python3 e2e/enterprise/fixture_adapter.py
```

```sh
EPSX_ENV=local EPSX_UI_PREVIEW=fixture HOST=127.0.0.1 PORT=3300 API_URL=http://127.0.0.1:48082 BACKEND_URL=http://127.0.0.1:48082 OIDC_ISSUER=http://127.0.0.1:48081 NEXT_PUBLIC_BLOCKCHAIN_NETWORK=local NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL=0x0000000000000000000000000000000000000001 ./target/debug/bff-frontend
```

```sh
python3 e2e/enterprise/session_proxy.py
```

Port 3300 is the public preview; 3301 attaches a generated fixture session. Both show **Preview data**. The adapter uses `premium_fixtures.py` only when explicitly enabled and binds to loopback. It supplies current read contracts and twelve explicitly illustrative ranking rows for density and pagination checks. No application data fallback or production environment file is involved.

For local failure checks, `PUT http://127.0.0.1:48082/__qa/state` with header `x-epsx-e2e-token: epsx-enterprise-local-fixture` accepts a JSON `faults` array containing `filters-unavailable`, `watchlist-unavailable` or `rankings-restricted`. Restore `{"faults":[]}` after checking. The original Rust fixture modes remain available for empty/malformed/unavailable/limited rankings. Run state checks sequentially: all previews share the fixture state.

The v2 report distinguishes read fixtures and tested stateful watchlist operations from unimplemented support, notification, developer and payment mutation scenarios. No actual payment or external wallet signature is executed. The native Chromium E2E runner currently cannot reach WebDriver on port 4444; in-app browser workflow evidence is separate and is not a passing result for that runner.

## Historical v1 capture workflow

This suite is separate from `e2e/migration`; it does not replace the migration reference images. The application remains Rust/Dioxus and uses the native BFF and generated Rust/WASM runtime. `agent-browser` is an external local QA tool, not an application dependency.

Start each service in a separate terminal from the repository root. These ports bind only to loopback. No production environment file is used.

```sh
cargo build -p epsx-frontend --bin bff-frontend --locked
./target/debug/xtask browser-runtime build
./target/debug/xtask e2e fixture-serve --bind 127.0.0.1:48081 --token epsx-enterprise-local-fixture
```

```sh
python3 e2e/enterprise/fixture_adapter.py
```

```sh
EPSX_ENV=local HOST=127.0.0.1 PORT=3300 API_URL=http://127.0.0.1:48082 BACKEND_URL=http://127.0.0.1:48082 OIDC_ISSUER=http://127.0.0.1:48081 ./target/debug/bff-frontend
```

```sh
python3 e2e/enterprise/session_proxy.py
```

After the frontend is listening, run:

```sh
EPSX_QA_BROWSER_SESSION=epsx-enterprise-captures python3 e2e/enterprise/capture.py
EPSX_QA_BROWSER_SESSION=epsx-enterprise-states python3 e2e/enterprise/states.py
```

`capture.py` writes 248 viewport PNGs and `audit.json` to `target/enterprise-redesign/after/`. It checks all 31 route patterns/aliases at 375, 768, 1024 and 1440 pixels in both themes, one main landmark, one page h1, frontend stylesheet isolation, and whole-document horizontal overflow. Use `--widths 375 1440` for a smaller inspection. Viewport images are used because Chromium full-page capture timed out on this Mac. The 200 additional in-app browser checks are summarized in the delivery report.

`states.py` exercises the existing Rust fixture's empty, malformed, unavailable and limited-rank states. It always restores healthy mode. Backend HTTP 403 is separately covered by the native BFF test and frontend rendering test; it must show restricted access and clear stale rows.

The scripts use isolated browser sessions with GPU acceleration disabled. If a reused Chromium session stalls during a screenshot, close only that QA session with `agent-browser --session SESSION_NAME close`, or use a fresh `EPSX_QA_BROWSER_SESSION` value. Run the capture and state suites sequentially because state checks change the shared local fixture mode.

The adapter supplies current country/sector and grouped-watchlist wire shapes, which the historical fixture predates. All other requests are forwarded unchanged to the Rust fixture. Some older account, plans, payment, notification, support and developer fixtures do not satisfy the current strict owner projections and correctly render unavailable. Their ready/empty/error projection and mutation behavior is also covered by the existing Rust tests; these images are not evidence of live account or payment data.

`session_proxy.py` attaches only a generated fixture token. Port 3300 is signed out; port 3301 is the fixture account. This checks the same session verifier and SSR authorization path, but does not perform an external wallet signature or a real payment. The proxy is intentionally limited to fixed loopback targets.

Manual browser workflows verified during implementation:

- Apply country/sector filters, select server-supported sorting and change pages with query parameters preserved.
- Expand a company's native quarterly details on mobile.
- Follow Sign in with the current Explore query preserved in the return URL.
- Add AAPL to the watchlist, create a group, move MSFT into it, reload and verify persistence against the fixture.
- Open the mobile navigation, wrap keyboard focus, Escape to close, restore focus to the trigger, and clear `inert` on main content.
- Toggle theme and preserve the saved preference; collapse desktop navigation and restore its state.

The historical `cargo xtask e2e run --group 0` could not start its Safari WebDriver session because Allow Remote Automation is disabled. The setting was not changed. The Chromium and in-app checks above are separate evidence, not a claimed pass of that migration suite.
