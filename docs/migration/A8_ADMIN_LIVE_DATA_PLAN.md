# A8.0 admin live-data and mutation readiness audit

Status: **integrity PASS target; production readiness STOP**. This is an audit and execution contract only. It authorizes no runtime change, production access, database or chain operation, or deployment. The machine-readable source of truth is [`contracts/admin-live-data.json`](contracts/admin-live-data.json).

## Baseline and conservative result

The source is pinned to `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. The audit covers the exact 27 source admin pages in [`contracts/routes.json`](contracts/routes.json), with no target-only alias or migration addition counted as source parity. It separately locks the three source routes whose target kind is intentionally `redirect`:

- `/auth` → `/` now returns a fixed HTTP 307;
- `/notifications` → `/notifications/manage` currently returns a 200 document whose script calls `location.replace`;
- `/wallet-management` → `/wallet-management/wallets` currently returns an HTTP 308.

No redirect is aligned merely because its destination exists. The pinned source middleware handles `?logout`, login-route cookie clearing/session presence, and backend-owned admin verification before page-level redirects execute. The target auth and wallet routes bypass SSR with fixed 307/308 responses; notifications traverses SSR but returns a 200 script document. In-process tests prove only that all three current GET targets are fixed and redirect-shaped query input cannot choose an external destination. `/auth` still does not reproduce source `reason=no-session`, `clear`, or `logout` cookie clearing. The set does not yet prove accepted query policy, cache behavior, method/body handling, browser history, RSC/client navigation, or authenticated middleware/logout/session ordering.

All three redirects therefore remain non-aligned until three evidence gaps close: authenticated browser/history/RSC/client-navigation behavior, a pinned-origin method/body/cache matrix, and parity with the source middleware/logout/session ordering. `/auth` is blocked because its cookie-lifecycle semantics are absent; `/notifications` and `/wallet-management` remain partial because only their fixed destinations are proven.

The baseline is deliberately strict:

- **2 aligned:** `/access-denied` and `/unauthorized`;
- **3 partial:** `/news`, `/notifications`, and `/wallet-management`;
- **22 blocked**;
- **25 non-aligned** and **20 cross-cutting STOP blockers**.

The B2.1 proof closes the two source denial surfaces. `/access-denied` now preserves all five source query fields with bounded, control-filtered decoding and escaped output; `route` is used only for display and the sanitized reauthentication return target. `/unauthorized` retains the exact static source copy and ignores query input. Both inherit the pinned admin title, description, and keywords, return an accepted SSR 200 denial document, expose one heading plus alert/navigation landmarks, use only sanitized same-origin links, invoke the canonical same-origin logout endpoint before reauthentication, and preserve source `history.back()` behavior with an accessible static `/` fallback. Focus order, light and dark rendering at 390×844 and 1440×900, non-transparent computed dark decoration, responsive overflow, unsafe targets, cookie clearing, and zero browser/page errors are covered by an ephemeral loopback admin RS256/JWKS Playwright fixture. This does not close A1's separate disposable-PostgreSQL proof for durable refresh-token revocation.

`/news` is now the first partial operational admin route: a strict SSR-only compatibility adapter consumes the pinned Rust backend's protected `GET /api/admin/news` read, forwards only a verified bearer and normalized `page`/`status`, caps the upstream body at 8 MiB before JSON allocation, projects away full content, author identity, cover media, and unused timestamps, and renders distinct ready, authoritative-empty, forbidden, unavailable, malformed, and recoverable out-of-range states. Authenticated HTML is private/no-store and varies on cookie/authorization. The public file-backed content feed is explicitly not treated as admin data, and create/edit/publish/pin/delete/upload stay unavailable. `/payments` retains its bounded read-only payment-intents loader but stays blocked overall because its other source tabs, isolated service/database/browser proof, and all financial mutation authority remain open. Other operational pages dispatch without authoritative page data and continue to fail closed instead of presenting plausible sample state.

The target-only `/policies` reserved path is not one of the 27 source routes and now delegates directly to the shared 404 page. Its fabricated policy catalog, telemetry, builder, controls, and legacy frontend permission gate are absent; this cleanup does not add or align a source route.

## What the gate proves

Integrity mode is deterministic and offline. It verifies:

1. `origin/development` still resolves to the accepted full source SHA;
2. every one of the 27 pinned source files exists at that commit and contains its recorded anchors;
3. the current target implementation, dispatcher, admin SSR, and BFF contain the recorded anchors;
4. the route set, source files, and target handlers equal the checked route inventory exactly;
5. all 27 routes occur in exactly one of seven execution batches;
6. the three redirects equal the inventory's exact redirect-classified set;
7. every route inventories dynamic params, reads, fallbacks, read/manage gates, mutations, request/envelope/status findings, six async states, keyboard, responsive behavior, hydration, dependencies, and at least one blocker while non-aligned;
8. the accepted baseline remains 2 aligned / 3 partial / 22 blocked until evidence is deliberately updated.

This gate does **not** prove a service is reachable, a database is migrated, a browser interaction works, a mutation is durable, or production is ready. Readiness mode exits `3` while any route or global blocker remains.

## Key findings

### Page loaders and false-success fallbacks

`apps/admin/src/ssr.rs` authenticates the request and normally dispatches with empty data params. The bounded `/payments?tab=payments` exception forwards only `payer`, `status`, `limit`, and `offset` to `GET /api/v1/admin/pay/intents`, rejects malformed or duplicate recognized values without an upstream request, validates the typed success payload, and never converts dependency or contract failure into an authoritative empty list. The `/news` exception accepts only a bounded page and exact all/draft/published status, fixes the source limit at 20, forwards the verified bearer to `GET /api/admin/news`, strictly decodes the legacy success envelope, and stores only a minimal read projection. Both loaders keep an empty page with a nonzero authoritative total recoverable instead of mislabeling it as zero matches. Dashboard, analytics, audit-log, chat list/detail, media, news create/edit, notification manage/create, developer portal, settings, wallet credits/access/list/detail/disable, and wallet plan list/detail retain verified-session explicit-unavailable shells with no plausible operational records, aggregates, credentials, balances, plan catalogs, forms, upload controls, or mutations.

Generic `AuthPageOverlay` and `SkeletonPage` rendering is useful visual-capture evidence only. It cannot satisfy a loading contract because no corresponding loaded, empty, dependency-error, or retry transition is wired.

### Dynamic parameters and ownership

The dispatcher restricts dynamic paths to one or two segments and places identifiers in `PageContext`, but that is only route-shape evidence. The following still require service-owned validation and authorization:

- chat conversation IDs;
- news IDs and revision identifiers;
- wallet chain/address decode, normalization, checksum, and lookup;
- wallet disable targets;
- plan IDs, including any reserved `new` value.

The selected backend service must decide whether an invalid, absent, or foreign resource returns validation, forbidden, or a non-leaking not-found response. The UI must preserve that decision rather than converting it to a plausible sample.

### Read versus manage permission splits

Wallet access, wallet list/detail/disable, and plan list/detail now use session-only unavailable shells. Their former frontend read/manage gates and mutation controls were removed because no authoritative data or operation is exposed. This does not weaken policy: the owning Rust service must decide and enforce exact read, ownership, and manage authorization for every future request before the UI can render data or restore controls.

Other surfaces currently gate read-only data behind broad manage permissions, notably chat, media, news, notifications, and settings. The payment-intents tab now uses the backend-recognized `admin:payments:view` permission, while the Pay service repeats the exact admin-audience and permission check. The remaining execution batches must introduce canonical read permissions only where the backend recognizes them; the UI must not invent role expansion or policy.

### BFF and API contracts

The admin BFF exposes proxy routes for several identity, wallet, payment, subscription, content, notification, analytics, and indexer operations. The payment-intents proxy is now a typed exception: it targets the canonical admin-wide Pay route, forwards a closed query allowlist, rejects malformed success payloads, and is consumed by SSR. The other pages generally do not consume their proxies, so proxy presence remains insufficient evidence of route readiness.

The legacy admin BFF payment detail, confirm, cancel, escrow-list, and escrow-release routes are no longer registered. The page exposes no financial mutation controls. This is a safety boundary, not mutation readiness: every payment mutation remains blocked until A6/A9 authority, idempotency, chain/finality, audit, and durable read-after-write proof pass.

The shared client now represents upstream failures as body-free typed status errors. The admin BFF preserves a closed allowlist of safe upstream client statuses (400, 401, 403, 404, 409, 422, and 429) and explicit dependency classes (502, 503, and 504); typed timeout and connection failures map to 504 and 503. Unknown, arbitrary, and upstream 500 statuses fail closed to 502, legacy service strings are never parsed, and upstream bodies and headers are never forwarded. This is only an A5 prerequisite: handlers still emit bare statuses without a stable error code, message, validation detail, retryability, or correlation envelope, and pages do not consume those error states. A5 must still lock method, content type, request body, response envelope, and UI handling before a page can align.

The corresponding source mutation contracts still have explicit drift even though the unsafe target controls are now absent:

- notification send path/body semantics remain unresolved between the singular source contract and plural BFF route;
- developer API-key revoke, expiration, and secret-once creation contracts remain absent;
- wallet disable/reenable methods, bodies, idempotency, and BFF routes remain absent;
- plan create/update methods and form-versus-JSON bodies remain unresolved.

### Mutation acceptance

A rendered button, dialog, form action, signal update, or emitted log is not a mutation proof. Each mutation needs:

- exact backend-owned permission and resource check;
- validated request and typed response/error envelope;
- idempotency or optimistic version behavior where retry/duplication matters;
- pending, success, validation, forbidden, conflict, dependency-error, and retry UI;
- durable read-after-write evidence;
- immutable audit record, correlation ID, redacted logs/metrics, and recovery behavior.

Financial, credit, plan, subscription, entitlement, publication, and permission mutations remain blocked on their corresponding backend work. No such rule may be reimplemented in the frontend.

## Seven executable batches

Execute a batch only after its named dependencies are evidence-ready. A route moves to aligned only after focused Rust adapter tests and an authenticated local browser fixture prove every applicable contract field.

1. **B1 command and security — `/`, `/analytics`, `/audit-log`, `/settings`.** Dashboard now fails closed without fabricated KPIs, health, alerts, wallets, transactions, activity, uptime, or freshness. The pinned `/analytics` page is an EPS market-ranking surface with plan-access and watchlist inputs—not the target event-analytics events/metrics/revenue API—and fails closed without rankings, entitlement, freshness, filters, or status claims; those domains must remain separate. `/audit-log` fails closed without sample actors, IPs, timestamps, totals, filters, expansion, pagination, or export; settings fails closed without invented configuration, API keys, devices, sessions, defaults, editors, or mutation controls. Define canonical aggregate/read models and backend field authorization; establish dedicated dashboard, audit, and settings read permissions; add a typed field-redacted versioned settings adapter instead of default-on-empty behavior; wire strict ranking and audit queries, pagination, and authorized server export; render dependency-specific ready/empty/forbidden/degraded/error/retry states; prove optimistic concurrency and audited settings mutations.
2. **B2 auth and denial — `/access-denied`, `/auth`, `/unauthorized`, `/developer-portal/api-keys/create`.** `/access-denied` and `/unauthorized` are aligned by the B2.1 adapter/browser proof. `/auth` now has a fixed pre-SSR 307 to `/`, with no invented permission gate, sign-in selector, delay, request reflection, or open-redirect target; it remains blocked until source cookie-clearing/logout/session ordering, method/cache, and authenticated browser/RSC/history behavior are proven. Finish A1's durable revoked-session behavior and decide whether API-key creation remains intentionally denied or regains its source mutation flow.
3. **B3 support — `/chat`, `/chat/:id`, `/notifications`, `/notifications/manage`.** Keep `/notifications` partial until its three redirect proof gaps close. Admin chat list/detail now fail closed without canned conversations, messages, counts, presence, filters, replies, assignment, or status actions; detail route references are bounded and explicitly unverified. `/notifications/manage` fails closed without sample rows, metrics, filters, dialogs, or local mutations. Define backend read/manage splits and authoritative list/detail data; implement cursor/query preservation, non-leaking detail errors, assignment/status conflicts, message delivery/reconnect, and scheduled notification conflicts.
4. **B4 content and media — `/media`, `/news`, `/news/create`, `/news/:id/edit`.** Media remains fail-closed without sample objects, buckets, storage totals, filters, upload, preview, copy, or delete controls. `/news` now has a strict read-only compatibility path to the pinned Rust backend's protected admin list, with source-like status filtering, bounded pagination, escaped summary cards, and explicit ready/empty/forbidden/unavailable/malformed states; it never consumes the public marketing feed or exposes a content mutation. Create/edit remain fail-closed, authenticated edit references are bounded and explicitly unverified, and signed-out dispatcher and BFF shells return only to `/news` without disclosing the reference. Move the list producer into canonical content-service storage, split backend read/manage authority, add deterministic ordering and isolated DB/browser proof, then finish A10 detail/revision, upload, publish/cache, optimistic conflict, autosave/recovery, unsaved-change, and accessible editor/upload behavior.
5. **B5 commerce — `/payments`, `/wallet-management/credits`, `/wallet-management/access`, `/wallet-management/access/plans`.** `/payments` now has a typed read-only admin-intent adapter with truthful empty/dependency/malformed outcomes and native URL-driven filters/pagination; its other two tabs stay explicitly unavailable and all payment mutations stay unregistered. Credits, wallet access, and wallet-plan list now fail closed without invented balances, assignments, catalogs, permissions, prices, metrics, grant/revoke, or plan controls. Finish isolated service/database and browser proof, split credit read/manage authority, replace the current GET path's get-or-create side effect with a non-mutating bounded read, then complete A4/A6/A9 for credit/access/plan/payment-link work, idempotent audited operations, notification durability, and finality/conflict semantics.
6. **B6 wallets and plan detail — `/wallet-management`, `/wallet-management/wallets`, `/wallet-management/:address`, `/wallet-management/access/plans/:planId`.** Keep `/wallet-management` partial until the established 308 is formally accepted or corrected with all three redirect proof gaps closed. Wallet list/detail and plan detail now fail closed; authenticated dynamic references are bounded, escaped, explicitly unverified, and encoded as one retry-link segment, while signed-out dispatcher and BFF shells return only to static collections and do not disclose those references. Add URL-driven list pagination/filtering; canonicalize chain/address and plan ID; consume owner/resource-safe reads; align plan create/update method and body; prove optimistic conflict and non-leaking errors.
7. **B7 focused mutations — `/notifications/create`, `/developer-portal`, `/wallet-management/wallets/:address/disable`.** Notification create now fails closed without recipient/template/schedule/send controls or a mismatched form action. Developer portal now fails closed without credential inventory, plaintext key material, modules, quotas, usage, documentation claims, or create/revoke/update/copy controls; the create-key route remains denied. Wallet disable now exposes only an explicit unavailable state with a bounded unverified reference and no status, impact, confirmation, disable, or re-enable control. Define canonical endpoint paths and bodies; stop persisting and projecting plaintext `api_keys.full_key`; add a redacted read/manage split and secret-once API-key creation; authorize and audit notification delivery and wallet disable; implement idempotency, rate-limit, conflict, pending/success/error/retry, cancellation, and focus restoration.

## Per-route proof matrix

| Route | Current data/mutation truth | Permission finding | State |
|---|---|---|---|
| `/` | explicit unavailable shell; no inferred KPIs, health, wallets, activity, alerts, or freshness | frontend dashboard gate removed; typed backend aggregate/field authorization required | blocked |
| `/access-denied` | bounded/escaped source query denial panel with sanitized links and canonical logout action | presentation only; A1 remains logout-revocation authority | aligned |
| `/analytics` | explicit unavailable page-owned shell; no market rankings, plan-access/watchlist claims, freshness, filters, or export | event `admin:analytics:view` is not ranking authority; backend market-ranking entitlement and field authorization remain required | blocked |
| `/audit-log` | explicit unavailable shell; no inferred records, filters, or export | frontend analytics gate removed; dedicated backend audit permission still required | blocked |
| `/auth` | fixed pre-SSR HTTP 307 to `/`; hostile query targets ignored | invented frontend auth permission removed; source cookie clearing/logout/session ordering and browser/method/cache proof remain open | blocked |
| `/chat` | explicit unavailable shell; no conversations, counts, presence, filters, or actions | frontend capability inference removed; backend read/manage decision required | blocked |
| `/chat/:id` | bounded encoded unverified route reference only; no conversation/messages/actions | backend resource authorization and typed detail/version contract required | blocked |
| `/developer-portal` | explicit unavailable shell; no credentials, usage, modules, documentation claims, or actions | plaintext-key persistence/list projection, redacted read/manage split, secret-once creation, and BFF adapter remain open | blocked |
| `/developer-portal/api-keys/create` | source form replaced by target denial | target mutation path absent | blocked |
| `/media` | explicit unavailable shell; no objects, storage totals, filters, or upload/delete controls | frontend capability inference removed; backend media read/manage decision required | blocked |
| `/news` | strict read-only Rust-backend compatibility projection with URL status/pagination and ready/empty/forbidden/unavailable/malformed states; no content or mutations | legacy backend repeats `admin:content:manage`; canonical service storage/read split and runtime proof remain open | partial |
| `/news/create` | explicit unavailable shell; no editor or mutation | backend-authorized create/publish contract required | blocked |
| `/news/:id/edit` | bounded unverified route reference only; no record/editor/mutation | backend-authorized read/manage and revision contract required | blocked |
| `/notifications` | 200 JavaScript redirect; fixed GET target only | destination owns auth; source middleware/logout/session ordering parity unproven | partial |
| `/notifications/create` | explicit unavailable shell; mismatched form action removed | backend manage authorization and service proof open | blocked |
| `/notifications/manage` | explicit unavailable shell; no sample list, stats, filters, or mutations | frontend capability inference removed; backend read/manage decision required | blocked |
| `/payments` | canonical read-only payment intents; access/payment-link tabs explicitly unavailable; no mutations | backend-owned `admin:payments:view` repeated by Pay service; isolated runtime/browser proof open | blocked |
| `/settings` | explicit unavailable page-owned shell; no settings defaults, keys, sessions, editors, or actions | backend manage gate still covers reads; typed field-redacted versioned read/manage contract required | blocked |
| `/unauthorized` | exact static denial panel with sanitized links and canonical logout action | presentation only; A1 remains logout-revocation authority | aligned |
| `/wallet-management` | pre-SSR HTTP 308; fixed GET target only | destination owns auth; source middleware/logout/session ordering parity unproven | partial |
| `/wallet-management/:address` | explicit unavailable detail with bounded, escaped, encoded, unverified reference; no wallet data or actions | frontend gate removed; backend read/ownership decision and typed detail contract required | blocked |
| `/wallet-management/access` | explicit unavailable shell; no assignments, plans, permissions, or mutations | frontend read/manage gates removed; backend permission-system read/manage decisions required | blocked |
| `/wallet-management/access/plans` | explicit unavailable list shell; no catalog, prices, metrics, permissions, or actions | frontend read/manage gates removed; backend A9 plan projection and authorization required | blocked |
| `/wallet-management/access/plans/:planId` | explicit unavailable detail with bounded, escaped, encoded, unverified reference; no editor or mutations | frontend gates removed; backend plan read/manage, ID validation, and API method/body contract required | blocked |
| `/wallet-management/credits` | explicit unavailable shell; no balances, zero defaults, metrics, ledger, grant, or revoke controls | backend read currently mutates via get-or-create and shares manage authority; typed non-mutating bounded read and A6 mutation proof required | blocked |
| `/wallet-management/wallets` | explicit unavailable list shell; no rows, counts, balances, plans, filters, or actions | frontend gate removed; backend authorized pagination/query contract required | blocked |
| `/wallet-management/wallets/:address/disable` | explicit unavailable operation state with bounded unverified reference; no status, impact, confirmation, or mutation | frontend update gate removed; authorized idempotent audited service mutation remains absent | blocked |

The JSON contract holds the full request, envelope, status, async-state, UX, hydration, dependency, and blocker details. This table is intentionally a compact index, not a second source of truth.

## Browser and adapter acceptance for each route

Each route requires a focused fixture at mobile and desktop widths using a short-lived local authenticated admin session and deterministic service fixtures. The fixture must never bypass production authentication behavior and must bind only loopback.

Applicable assertions include:

- exact loaded payload and no sample fallback on dependency failure;
- loading, empty, validation, forbidden, non-leaking not-found, conflict, rate-limit, dependency-error, retry, and success states;
- URL persistence for pagination, filters, search, tabs, and dynamic identifiers across reload and back/forward;
- keyboard reachability, visible focus, dialog focus trap/restore, form labeling, native and server validation, and non-color-only state;
- 390×844 and 1440×900 overflow/layout behavior;
- hydration or a constant CSP-compatible browser bridge for every interactive control;
- read-only admins see permitted data but never mutation controls; manage-only users do not bypass required read access;
- every mutation produces durable read-after-write and audit/correlation evidence.

## Acceptance commands

```sh
./scripts/migration/verify-admin-live-data.sh --mode integrity
./scripts/migration/verify-admin-live-data.sh --mode emit
./scripts/migration/test-admin-live-data.sh
./scripts/migration/run-admin-denial-runtime-proof.sh
```

Readiness is intentionally a STOP gate:

```sh
./scripts/migration/verify-admin-live-data.sh --mode readiness
# exits 3 while any route or cross-cutting blocker remains
```

The self-test proves deterministic emit output, readiness exit `3`, conservative status tamper rejection, path traversal rejection, stale target-anchor rejection, stale pinned-source-anchor rejection, redirect-set tamper rejection, and redirect-semantics tamper rejection. It performs no network, database, chain, browser, cluster, or deployment operation.

## Exit criteria

A8 may pass only when all of the following are true:

- all exact 27 source routes are `aligned` with empty route blocker arrays;
- the 20 global STOP blockers are removed because linked evidence exists, not because text was deleted;
- all three redirect routes have accepted and tested HTTP/browser semantics;
- every operational page consumes authoritative Rust service data without plausible sample fallback;
- all reads and mutations enforce exact backend-owned authorization and resource ownership;
- request bodies, response/error envelopes, and statuses are versioned and preserved by A5;
- applicable A1–A6 and A9–A12 dependencies have passed;
- focused Rust and browser tests cover applicable states, keyboard, responsive layout, hydration, and durable mutation behavior.

Until then the emitted `productionReady` value remains `false` and readiness exits `3`.
