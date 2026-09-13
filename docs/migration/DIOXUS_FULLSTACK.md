# Dioxus Fullstack migration ledger

Frontend, Admin, and Pay now use Dioxus Fullstack for their active UI routes.
The local/dev migration and native release package validation are complete.
This is not a production deployment; that requires a separate instruction.

## Shared runtime

- Dioxus remains locked at 0.7.9. Each app selects `web` for WASM and `server`
  for native. Neither renderer is enabled globally. Enabling `web` on a native
  Fullstack build drops server-future hydration payloads even when compilation passes.
- Axum middleware, BFF session verification and existing REST endpoints remain.
  Only app-owned `/_server/frontend/*`, `/_server/admin/*`, or `/_server/pay/*`
  functions are mounted. Shared legacy pilot functions are not public endpoints.
- Request-local response header effects preserve every Set-Cookie when a server
  function delegates login/logout/refresh to the existing auth boundary. Cookies
  are HTTP response headers, never typed UI data.
- Native packaging now builds all three Fullstack bundles and copies the entire
  generated public directory, including index.html, debug `/wasm` and hashed release
  `/assets` files. Original
  native binary names and external configuration paths remain. Release packaging
  and startup from the resulting package passed for all three BFFs.
- The canonical design-system CSS was extracted from the old HTML header into a
  shared CSS literal. Legacy headers and Dioxus document styles share this source.
  Dioxus owns frontend shell menus, theme and layout state.

## Frontend progress

Typed providers/components and native Fullstack routes now cover Analytics,
Purchases list/detail, Account/preferences/push, Credits, Profile, Dashboard,
Permissions, News list/detail, Plans, About, Contact, Privacy, Terms, Notifications,
and Home/index. Availability-only Dashboard/Permissions retain their existing
verified-session semantics; no new entitlement logic is inferred in UI.

The final Analytics browser audit passed 12 outcomes, including actual ranking/plans
links and light/dark persistence through navigation and Back. Browser validation
has passed for Analytics limits 10/25/50/100, preserving filters,
Back/Forward, retaining results on failure, retry, response races and empty data.
The same document and shell remain mounted. The final Analytics audit also passed
the ranking-reset and plan links without reload, and light/dark theme persistence
through navigation and Back (including storage, root class and visual checks).
Its 12-outcome report is `target/dioxus-migration/analytics/results.json`.
Authenticated Purchases and Account
checks use an ephemeral real RSA/JWKS verifier, not an authentication bypass:
single SSR read, refresh/detail/history/error/retry, preferences save, repeated
cookie clearing and desktop/mobile layout have passed.

Developer, auth/payment, watchlist and chat now have typed controllers and native
providers. Actual signed-session browser fixtures also passed wallet rejection and
retry, login/logout HttpOnly cookies, checkout pending/confirmed/send-once,
watchlist grouping and keyboard cancel/commit, and chat attachment retry without
duplicate conversation creation. Frontend native `run()` now selects the Fullstack
application with asset preflight. The public `build_app` constructor also selects
Fullstack; the former HTML catchall and Purchases renderer are registered only in
regression tests. Remaining references to `ssr.rs` are typed data loaders and query
validation, not an active PageContext renderer. Frontend unknown pages use the
Dioxus 404 route; legacy browser UI runtime assets are no longer served.

Offline HTML is byte-identical with and without a valid session and performs zero
profile or verifier reads. An isolated browser test stops the fixture server to
exercise cached offline recovery, then restarts it and retries the original URL.
The public recovery page and two exact public stylesheets are cached; authenticated
page HTML is not. The styled recovery screenshot, subsequent hydration and logout
passed. The final matching native/WASM bundle passed the combined Purchases, Account,
watchlist, chat, offline and logout browser audit after the latest navigation and
form-completion refinements. Desktop and mobile Purchases screenshots were visually
inspected: table spacing, status badges and mobile stacked rows render correctly.

The extended matching-bundle audit also passed Developer create failure/retry with
preserved draft and idempotency identity, one-time secret display, confirmed revoke,
and a real Try request. Notifications passed shell navigation, read/unread, a named
SSE notification updating the list, and exactly one EventSource close when leaving.
The named-event check found and fixed the adapter previously listening only for
default message events. The final combined report has nine successful outcomes at
`target/dioxus-migration/account/report.json`; all 130 frontend native library tests
passed. Offline acceptance uses the generated `target/epsx-service-worker` assets.

## Admin and Pay

See `apps/admin/FULLSTACK_MIGRATION.md` for exact Admin route families. Signed-in
Settings, Catalog, Chat, Auth, Media and Escrow browser checks now pass. Media has
one initial SSR read, typed authenticated delete/upload, bucket navigation and Back
without reload, and desktop/mobile evidence. News passed 62 related UI tests and five browser outcomes, including version
conflicts, preserved drafts, image uploads and responsive layout. Developer passed
create/retry/one-time-secret/revoke and repeated-command identity checks. Wallet
management passed seven browser outcomes including Limit/history, versioned
mutations, conflict/retry and sidebar navigation without reload. Native Admin
entrypoints now use Fullstack; former SSR renderers are test-only. Notifications passed mark/read/delete/send/empty-state and distinct successful
command identities. Payments passed Limit/history, versioned intent cancellation,
link creation/disable and User Access navigation. Root Admin session recovery now
performs exactly one client refresh, rotates both HttpOnly cookies and remounts
the Router without a document reload; its two browser checks passed. All 11
Admin routing regressions pass, including raw malformed paths and `/admin` aliases.

Pay's native binary and DX native server now use the same BFF host with typed
read/action providers and a Dioxus-owned wallet flow. Native tests and isolated
browser fixtures cover merchant forms, checkout states, duplicate sends, rejection,
chain mismatch, history and responsive layout. The Pay-only SIWE audience,
HttpOnly cookies and same-document logout also passed. The shared wallet adapter
no longer makes direct authentication REST requests; existing external REST
contracts remain mounted for callers outside the UI.

The user-reported sign-in card lost padding, border and rounding through an
enterprise stylesheet override. The card now has responsive padding, a visible
border and separated footer spacing; its internal links use the Dioxus shell
router. Nine auth/payment browser outcomes pass on the authoritative v2
bundle, including no-wallet guidance, rejection/retry, pending double-click
suppression and verified same-document login. CUA visually inspected 1440px
and 390px in both themes; screenshots are recorded in the task conversation.
The dev watcher now serves a completed private server/assets snapshot, preventing
concurrent DX builds from mixing old SSR with new WASM. Live dev `/auth` and
CSS v2 both returned 200 after the change.

Narrow rendering exceptions: `admin_textarea.rs` escapes plain text for Dioxus
0.7 raw-text serialization (injection and browser tests pass), and Pay integration
docs render only checked-in Markdown. Neither exception fetches UI fragments,
installs global UI event handlers, or replaces the document DOM. The no-node
audit explicitly lists browser adapter expressions and pins the wallet bridge;
new or changed eval code requires review. Offline recovery uses a versioned
public cache and stylesheet URL so upgrades cannot retain mismatched markup/CSS.

## Reproducible local checks

```
python3 scripts/audit/dioxus_features.py
python3 scripts/audit/dioxus_inventory.py
cargo test -p epsx-bff --features fullstack fullstack::tests --locked
dx build --fullstack true --package epsx-frontend --bin dx-frontend --web --no-default-features --locked
python3 e2e/enterprise/dioxus_analytics_audit.py
EPSX_AUDIT_WATCHLIST=1 EPSX_AUDIT_CHAT=1 python3 e2e/enterprise/dioxus_account_audit.py
python3 e2e/enterprise/dioxus_admin_media_audit.py
python3 scripts/audit/dioxus_ownership.py
```

Do **not** append `--features web` to a Fullstack DX build: that applies it to both
renderers. The CLI selects the app's server/web features separately.

Inventory findings include separately classified test-only code; counts are not
production route coverage. The strict retirement gate passes with item-level
cfg(test) analysis, isolated deployment feature graphs and native entrypoint
checks. Regression tests reject reintroduced legacy runtime ownership. Generated reports and browser artifacts live under
`target/dioxus-migration/`. No production migration, service restart, tunnel change or deployment
is performed by these checks. Production deployment requires a separate instruction.

Final unit regression results: shared UI 854, Frontend 130, Admin 191 and Pay 9
passed. The ignored Pay test is an interactive fixture server, not a skipped
assertion suite. BFF transport/isolation 3 and static asset scope 2 passed; xtask
node-free/build-helper regressions 15 passed. Formatting and strict ownership,
feature, route-alias and precommit no-node audits pass.

Six pre-existing untracked Workers/PostCSS files were preserved (not deleted)
outside active source at `target/dioxus-migration/legacy-source-backup/`, with a
relative-path manifest. This keeps the native source audit clean without blanket
JavaScript exemptions.

The optimized Frontend release bundle also passed all nine auth/payment browser
groups (`auth-payment-release/report.json`): real release hydration, missing-wallet
retry, signing rejection, duplicate-click prevention, verified-cookie navigation
without reload, and backend-confirmed payment. Wallet and payment responses were
mocked; no real signature or transaction was requested.

Final native release: `target/native-releases/dioxus-fullstack-20260910`.
The package smoke audit verified 458 manifest SHA256 entries, ten executable native
binaries, matching worker JS/WASM hashes, and all three BFFs serving SSR plus hashed
release assets. Unknown routes return 404 and foreign-app server functions remain
unmounted. Evidence: `target/dioxus-migration/package/report.json`. Reproduce with:

```sh
cargo xtask native package --output target/native-releases/<new-release>
python3 e2e/enterprise/dioxus_native_package_audit.py target/native-releases/<new-release>
```

All package smoke processes and mocked payment fixtures were stopped after testing.
The local Frontend watcher serves an immutable matching server/assets snapshot.
