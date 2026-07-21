# A8.0 admin live-data and mutation readiness audit

Status: **integrity PASS target; production readiness STOP**. This is an audit and execution contract only. It authorizes no runtime change, production access, database or chain operation, or deployment. The machine-readable source of truth is [`contracts/admin-live-data.json`](contracts/admin-live-data.json).

## Baseline and conservative result

The source is pinned to `origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. The audit covers the exact 27 source admin pages in [`contracts/routes.json`](contracts/routes.json), with no target-only alias or migration addition counted as source parity. It separately locks the two source routes whose target kind is intentionally `redirect`:

- `/notifications` → `/notifications/manage` currently returns a 200 document whose script calls `location.replace`;
- `/wallet-management` → `/wallet-management/wallets` currently returns an HTTP 308.

Neither redirect is aligned merely because its destination exists. Their status, transport, cache, method, query, history, script-disabled, and authorization behavior need explicit acceptance against the source `redirect()` semantics.

The baseline is deliberately strict:

- **0 aligned**;
- **4 partial:** `/access-denied`, `/notifications`, `/unauthorized`, and `/wallet-management`;
- **23 blocked**;
- **27 non-aligned** and **20 cross-cutting STOP blockers**.

The two static denial surfaces are partial because markup presence and historical pixel-parity comments do not prove query safety, accepted HTTP semantics, keyboard focus, responsive overflow, or navigation behavior. Every operational page stays blocked because the admin SSR constructs `PageContext` with an empty `params` map and does not load page data. Fixed rows and values therefore remain samples even when a matching BFF proxy route exists.

## What the gate proves

Integrity mode is deterministic and offline. It verifies:

1. `origin/development` still resolves to the accepted full source SHA;
2. every one of the 27 pinned source files exists at that commit and contains its recorded anchors;
3. the current target implementation, dispatcher, admin SSR, and BFF contain the recorded anchors;
4. the route set, source files, and target handlers equal the checked route inventory exactly;
5. all 27 routes occur in exactly one of seven execution batches;
6. the two redirects equal the inventory's exact redirect-classified set;
7. every route inventories dynamic params, reads, fallbacks, read/manage gates, mutations, request/envelope/status findings, six async states, keyboard, responsive behavior, hydration, dependencies, and at least one blocker while non-aligned;
8. the accepted baseline remains 0 aligned / 4 partial / 23 blocked until evidence is deliberately updated.

This gate does **not** prove a service is reachable, a database is migrated, a browser interaction works, a mutation is durable, or production is ready. Readiness mode exits `3` while any route or global blocker remains.

## Key findings

### Page loaders and false-success fallbacks

`apps/admin/src/ssr.rs` authenticates the request, constructs an empty `params` map, and dispatches a page. It has no per-page data loader. The current pages consequently render hard-coded operational values, including wallets, balances, payments, audit events, health, API keys, sessions, news, notifications, chat messages, plans, credits, and timestamps.

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

Other surfaces currently gate read-only data behind broad manage permissions, notably chat, media, news, notifications, payments, and settings. The execution batches must introduce canonical read permissions only where the backend recognizes them; the UI must not invent role expansion or policy.

### BFF and API contracts

The admin BFF exposes proxy routes for several identity, wallet, payment, subscription, content, notification, analytics, and indexer operations. The pages generally do not consume them. Presence of a proxy handler is therefore not route readiness.

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

1. **B1 command and security — `/`, `/analytics`, `/audit-log`, `/settings`.** Define canonical aggregate/read models; remove sample operational success; establish a dedicated audit permission; wire query-driven ranges, filters, pagination, and export; render dependency-specific empty/degraded/error/retry states; implement settings as typed versioned backend mutations.
2. **B2 auth and denial — `/access-denied`, `/auth`, `/unauthorized`, `/developer-portal/api-keys/create`.** Close A1 return-target and revoked-session behavior; decide whether API-key creation remains intentionally denied; bound and escape denial query fields; match accepted redirect/status behavior; prove safe same-origin links, focus order, and mobile/desktop overflow.
3. **B3 support — `/chat`, `/chat/:id`, `/notifications`, `/notifications/manage`.** Add read/manage splits and authoritative list/detail data; implement cursor/query preservation, non-leaking detail errors, assignment/status conflicts, message delivery/reconnect, scheduled notification conflicts, and accepted HTTP redirect behavior.
4. **B4 content and media — `/media`, `/news`, `/news/create`, `/news/:id/edit`.** Finish A10 authority; wire list/detail/revision data; validate upload size/type/hash; establish publish/cache semantics; add optimistic revisions, autosave/recovery, unsaved-change protection, and accessible editor/upload state.
5. **B5 commerce — `/payments`, `/wallet-management/credits`, `/wallet-management/access`, `/wallet-management/access/plans`.** Finish A4/A6/A9; remove invented money and ledger data; consume backend decisions; keep existing read/manage UI separation; implement idempotent audited credit/access operations; preserve finality and conflict statuses.
6. **B6 wallets and plan detail — `/wallet-management`, `/wallet-management/wallets`, `/wallet-management/:address`, `/wallet-management/access/plans/:planId`.** Accept or correct redirect permanence; add URL-driven list pagination/filtering; canonicalize chain/address and plan ID; consume owner/resource-safe reads; align plan create/update method and body; prove optimistic conflict and non-leaking errors.
7. **B7 focused mutations — `/notifications/create`, `/developer-portal`, `/wallet-management/wallets/:address/disable`.** Align endpoint paths and bodies; add secret-once API-key behavior; authorize and audit notification delivery and wallet disable; implement idempotency, rate-limit, conflict, pending/success/error/retry, cancellation, and focus restoration.

## Per-route proof matrix

| Route | Current data/mutation truth | Permission finding | State |
|---|---|---|---|
| `/` | fixed dashboard KPIs, wallets, health, alerts | dashboard read UI gate only | blocked |
| `/access-denied` | static/query denial panel | presentation only | partial |
| `/analytics` | static charts/table; export only logs | analytics read gate; page does not consume service | blocked |
| `/audit-log` | sample actors, IPs, actions | analytics permission is semantically suspect | blocked |
| `/auth` | delayed query-driven script redirect | auth bootstrap coupled to permission | blocked |
| `/chat` | four canned conversations | manage gates reads | blocked |
| `/chat/:id` | canned messages independent of ID | manage gate; resource authorization absent | blocked |
| `/developer-portal` | sample API keys/modules/usage | read/manage split absent | blocked |
| `/developer-portal/api-keys/create` | source form replaced by target denial | target mutation path absent | blocked |
| `/media` | sample file inventory; unwired uploader | manage gates reads | blocked |
| `/news` | sample articles and publication history | manage gates reads | blocked |
| `/news/create` | static editor; no durable create | manage UI gate only | blocked |
| `/news/:id/edit` | ID-backed load/update absent | manage UI gate only | blocked |
| `/notifications` | 200 JavaScript redirect | destination owns auth | partial |
| `/notifications/create` | singular form path mismatches BFF | manage UI gate; service proof open | blocked |
| `/notifications/manage` | sample list and delivery stats | manage gates reads | blocked |
| `/payments` | sample payments/access/links | manage gates reads | blocked |
| `/settings` | sample keys/sessions/settings | manage gates reads | blocked |
| `/unauthorized` | static denial panel | presentation only | partial |
| `/wallet-management` | HTTP 308 redirect | destination owns auth | partial |
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
```

Readiness is intentionally a STOP gate:

```sh
./scripts/migration/verify-admin-live-data.sh --mode readiness
# exits 3 while any route or cross-cutting blocker remains
```

The self-test proves deterministic emit output, readiness exit `3`, conservative status tamper rejection, path traversal rejection, stale target-anchor rejection, stale pinned-source-anchor rejection, and redirect-set tamper rejection. It performs no network, database, chain, browser, cluster, or deployment operation.

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
