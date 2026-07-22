# A8.0 admin live-data and mutation readiness audit

Status: **integrity PASS target; production readiness STOP**. This is an audit and execution contract only. It authorizes no runtime change, production access, database or chain operation, or deployment. The machine-readable source of truth is [`contracts/admin-live-data.json`](contracts/admin-live-data.json).

## Baseline and conservative result

The source is pinned to `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. The audit covers the exact 27 source admin pages in [`contracts/routes.json`](contracts/routes.json), with no target-only alias or migration addition counted as source parity. It separately locks the two source routes whose target kind is intentionally `redirect`:

- `/notifications` → `/notifications/manage` currently returns a 200 document whose script calls `location.replace`;
- `/wallet-management` → `/wallet-management/wallets` currently returns an HTTP 308.

Neither redirect is aligned merely because its destination exists. The pinned source middleware handles `?logout`, session presence, and backend-owned admin verification before either page-level `redirect()` executes. The target notification route traverses SSR but returns a 200 script document; the wallet route returns its established 308 before SSR and therefore bypasses the target SSR session path. An in-process test proves only that both current GET targets are fixed and redirect-shaped query input cannot choose an external destination. It does not prove source parity, query preservation or dropping, cache behavior, method/body handling, browser history, RSC/client navigation, or authenticated middleware/logout/session ordering.

Both routes therefore remain partial until three evidence gaps close: authenticated browser/history/RSC/client-navigation behavior, a pinned-origin method/body/cache matrix, and parity with the source middleware/logout/session ordering. Runtime behavior remains unchanged while those gaps are open.

The baseline is deliberately strict:

- **2 aligned:** `/access-denied` and `/unauthorized`;
- **2 partial:** `/notifications` and `/wallet-management`;
- **23 blocked**;
- **25 non-aligned** and **20 cross-cutting STOP blockers**.

The B2.1 proof closes the two source denial surfaces. `/access-denied` now preserves all five source query fields with bounded, control-filtered decoding and escaped output; `route` is used only for display and the sanitized reauthentication return target. `/unauthorized` retains the exact static source copy and ignores query input. Both inherit the pinned admin title, description, and keywords, return an accepted SSR 200 denial document, expose one heading plus alert/navigation landmarks, use only sanitized same-origin links, invoke the canonical same-origin logout endpoint before reauthentication, and preserve source `history.back()` behavior with an accessible static `/` fallback. Focus order, light and dark rendering at 390×844 and 1440×900, non-transparent computed dark decoration, responsive overflow, unsafe targets, cookie clearing, and zero browser/page errors are covered by an ephemeral loopback admin RS256/JWKS Playwright fixture. This does not close A1's separate disposable-PostgreSQL proof for durable refresh-token revocation.

Every operational page stays blocked overall, but `/payments` now has the first route-scoped admin loader: the read-only payment-intents tab consumes a bounded authenticated Pay-service response and records explicit ready, empty, unavailable, or malformed state. Its access and payment-link tabs remain intentionally unavailable, and the route still lacks isolated service/database and browser proof. Other operational pages still dispatch without authoritative page data and continue to expose the sample/fallback risks inventoried below.

## What the gate proves

Integrity mode is deterministic and offline. It verifies:

1. `origin/development` still resolves to the accepted full source SHA;
2. every one of the 27 pinned source files exists at that commit and contains its recorded anchors;
3. the current target implementation, dispatcher, admin SSR, and BFF contain the recorded anchors;
4. the route set, source files, and target handlers equal the checked route inventory exactly;
5. all 27 routes occur in exactly one of seven execution batches;
6. the two redirects equal the inventory's exact redirect-classified set;
7. every route inventories dynamic params, reads, fallbacks, read/manage gates, mutations, request/envelope/status findings, six async states, keyboard, responsive behavior, hydration, dependencies, and at least one blocker while non-aligned;
8. the accepted baseline remains 2 aligned / 2 partial / 23 blocked until evidence is deliberately updated.

This gate does **not** prove a service is reachable, a database is migrated, a browser interaction works, a mutation is durable, or production is ready. Readiness mode exits `3` while any route or global blocker remains.

## Key findings

### Page loaders and false-success fallbacks

`apps/admin/src/ssr.rs` authenticates the request and normally dispatches with empty data params. The bounded `/payments?tab=payments` exception forwards only `payer`, `status`, `limit`, and `offset` to `GET /api/v1/admin/pay/intents`, rejects malformed or duplicate recognized values without an upstream request, validates the typed success payload, and never converts dependency or contract failure into an authoritative empty list. An empty page with a nonzero authoritative total remains a recoverable page state rather than being mislabeled as zero matches. Audit-log, news list/create/edit, and notification manage/create now render authenticated explicit-unavailable shells with no sample rows, counts, histories, forms, or mutations. Other pages still render hard-coded operational values, including wallets, balances, health, API keys, sessions, chat messages, plans, credits, and timestamps.

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

The wallet access and plan components now show useful UI-level operation splits: read gates expose data and nested manage gates expose mutation controls. That is not live-data readiness. The owning Rust service must repeat exact operation authorization for every request.

Other surfaces currently gate read-only data behind broad manage permissions, notably chat, media, news, notifications, and settings. The payment-intents tab now uses the backend-recognized `admin:payments:view` permission, while the Pay service repeats the exact admin-audience and permission check. The remaining execution batches must introduce canonical read permissions only where the backend recognizes them; the UI must not invent role expansion or policy.

### BFF and API contracts

The admin BFF exposes proxy routes for several identity, wallet, payment, subscription, content, notification, analytics, and indexer operations. The payment-intents proxy is now a typed exception: it targets the canonical admin-wide Pay route, forwards a closed query allowlist, rejects malformed success payloads, and is consumed by SSR. The other pages generally do not consume their proxies, so proxy presence remains insufficient evidence of route readiness.

The legacy admin BFF payment detail, confirm, cancel, escrow-list, and escrow-release routes are no longer registered. The page exposes no financial mutation controls. This is a safety boundary, not mutation readiness: every payment mutation remains blocked until A6/A9 authority, idempotency, chain/finality, audit, and durable read-after-write proof pass.

The shared client now represents upstream failures as body-free typed status errors. The admin BFF preserves a closed allowlist of safe upstream client statuses (400, 401, 403, 404, 409, 422, and 429) and explicit dependency classes (502, 503, and 504); typed timeout and connection failures map to 504 and 503. Unknown, arbitrary, and upstream 500 statuses fail closed to 502, legacy service strings are never parsed, and upstream bodies and headers are never forwarded. This is only an A5 prerequisite: handlers still emit bare statuses without a stable error code, message, validation detail, retryability, or correlation envelope, and pages do not consume those error states. A5 must still lock method, content type, request body, response envelope, and UI handling before a page can align.

Several target forms also have explicit drift:

- notification create posts to singular `/api/v1/notification/send`, while the BFF implements plural `/api/v1/notifications/send`;
- developer API-key revoke and expiration actions have no matching BFF routes;
- wallet disable/reenable form routes have no matching BFF routes;
- plan editor uses native form POST semantics, while BFF create/get support does not establish an update method or form-versus-JSON body contract.

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

1. **B1 command and security — `/`, `/analytics`, `/audit-log`, `/settings`.** `/audit-log` now fails closed without sample actors, IPs, timestamps, totals, filters, expansion, pagination, or export. Define canonical aggregate/read models; remove remaining sample operational success; establish a dedicated backend audit permission; wire query-driven ranges, filters, pagination, and authorized server export; render dependency-specific ready/empty/forbidden/degraded/error/retry states; implement settings as typed versioned backend mutations.
2. **B2 auth and denial — `/access-denied`, `/auth`, `/unauthorized`, `/developer-portal/api-keys/create`.** `/access-denied` and `/unauthorized` are aligned by the B2.1 adapter/browser proof. Finish A1's durable revoked-session behavior for the auth lifecycle, match `/auth` redirect/status behavior, and decide whether API-key creation remains intentionally denied or regains its source mutation flow.
3. **B3 support — `/chat`, `/chat/:id`, `/notifications`, `/notifications/manage`.** Keep `/notifications` partial until its three redirect proof gaps close. `/notifications/manage` now fails closed without sample rows, metrics, filters, dialogs, or local mutations. Define backend read/manage splits and authoritative list/detail data; implement cursor/query preservation, non-leaking detail errors, assignment/status conflicts, message delivery/reconnect, and scheduled notification conflicts.
4. **B4 content and media — `/media`, `/news`, `/news/create`, `/news/:id/edit`.** News list/create/edit now fail closed without sample articles, publication history, editor fields, filters, or actions; edit route references are bounded and explicitly unverified. Finish A10 authority; wire list/detail/revision data; validate upload size/type/hash; establish publish/cache semantics; add optimistic revisions, autosave/recovery, unsaved-change protection, and accessible editor/upload state.
5. **B5 commerce — `/payments`, `/wallet-management/credits`, `/wallet-management/access`, `/wallet-management/access/plans`.** `/payments` now has a typed read-only admin-intent adapter with truthful empty/dependency/malformed outcomes and native URL-driven filters/pagination; its other two tabs stay explicitly unavailable and all payment mutations stay unregistered. Finish isolated service/database and browser proof, then complete A4/A6/A9 for the remaining credit/access/payment-link work, idempotent audited operations, and finality/conflict semantics.
6. **B6 wallets and plan detail — `/wallet-management`, `/wallet-management/wallets`, `/wallet-management/:address`, `/wallet-management/access/plans/:planId`.** Keep `/wallet-management` partial until the established 308 is formally accepted or corrected with all three redirect proof gaps closed. Add URL-driven list pagination/filtering; canonicalize chain/address and plan ID; consume owner/resource-safe reads; align plan create/update method and body; prove optimistic conflict and non-leaking errors.
7. **B7 focused mutations — `/notifications/create`, `/developer-portal`, `/wallet-management/wallets/:address/disable`.** Notification create now fails closed without recipient/template/schedule/send controls or a mismatched form action. Define canonical endpoint paths and bodies; add secret-once API-key behavior; authorize and audit notification delivery and wallet disable; implement idempotency, rate-limit, conflict, pending/success/error/retry, cancellation, and focus restoration.

## Per-route proof matrix

| Route | Current data/mutation truth | Permission finding | State |
|---|---|---|---|
| `/` | fixed dashboard KPIs, wallets, health, alerts | dashboard read UI gate only | blocked |
| `/access-denied` | bounded/escaped source query denial panel with sanitized links and canonical logout action | presentation only; A1 remains logout-revocation authority | aligned |
| `/analytics` | static charts/table; export only logs | analytics read gate; page does not consume service | blocked |
| `/audit-log` | explicit unavailable shell; no inferred records, filters, or export | frontend analytics gate removed; dedicated backend audit permission still required | blocked |
| `/auth` | delayed query-driven script redirect | auth bootstrap coupled to permission | blocked |
| `/chat` | four canned conversations | manage gates reads | blocked |
| `/chat/:id` | canned messages independent of ID | manage gate; resource authorization absent | blocked |
| `/developer-portal` | sample API keys/modules/usage | read/manage split absent | blocked |
| `/developer-portal/api-keys/create` | source form replaced by target denial | target mutation path absent | blocked |
| `/media` | sample file inventory; unwired uploader | manage gates reads | blocked |
| `/news` | explicit unavailable shell; no sample articles/history/actions | frontend capability inference removed; backend read/manage decision required | blocked |
| `/news/create` | explicit unavailable shell; no editor or mutation | backend-authorized create/publish contract required | blocked |
| `/news/:id/edit` | bounded unverified route reference only; no record/editor/mutation | backend-authorized read/manage and revision contract required | blocked |
| `/notifications` | 200 JavaScript redirect; fixed GET target only | destination owns auth; source middleware/logout/session ordering parity unproven | partial |
| `/notifications/create` | explicit unavailable shell; mismatched form action removed | backend manage authorization and service proof open | blocked |
| `/notifications/manage` | explicit unavailable shell; no sample list, stats, filters, or mutations | frontend capability inference removed; backend read/manage decision required | blocked |
| `/payments` | canonical read-only payment intents; access/payment-link tabs explicitly unavailable; no mutations | backend-owned `admin:payments:view` repeated by Pay service; isolated runtime/browser proof open | blocked |
| `/settings` | sample keys/sessions/settings | manage gates reads | blocked |
| `/unauthorized` | exact static denial panel with sanitized links and canonical logout action | presentation only; A1 remains logout-revocation authority | aligned |
| `/wallet-management` | pre-SSR HTTP 308; fixed GET target only | destination owns auth; source middleware/logout/session ordering parity unproven | partial |
| `/wallet-management/:address` | sample detail independent of address | read UI gate; downstream proof open | blocked |
| `/wallet-management/access` | sample assignments | UI read/manage split exists; backend live proof open | blocked |
| `/wallet-management/access/plans` | static hub | read gate; target intent/data contract open | blocked |
| `/wallet-management/access/plans/:planId` | skeleton/sample editor | nested read/manage gates; API method/body drift | blocked |
| `/wallet-management/credits` | sample balances and ledger | explicit operation gates absent | blocked |
| `/wallet-management/wallets` | sample table/stats | read UI gate; live query absent | blocked |
| `/wallet-management/wallets/:address/disable` | confirmation form targets absent route | update UI gate; service mutation absent | blocked |

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
- both redirect routes have accepted and tested HTTP/browser semantics;
- every operational page consumes authoritative Rust service data without plausible sample fallback;
- all reads and mutations enforce exact backend-owned authorization and resource ownership;
- request bodies, response/error envelopes, and statuses are versioned and preserved by A5;
- applicable A1–A6 and A9–A12 dependencies have passed;
- focused Rust and browser tests cover applicable states, keyboard, responsive layout, hydration, and durable mutation behavior.

Until then the emitted `productionReady` value remains `false` and readiness exits `3`.
