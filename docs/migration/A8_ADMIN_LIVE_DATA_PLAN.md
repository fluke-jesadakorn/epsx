# A8.0 admin live-data and mutation readiness audit

Status: **integrity PASS; deterministic readiness PASS; runtime/production evidence STOP**. This is an audit and execution contract only. It authorizes no runtime change, production access, database or chain operation, or deployment. The machine-readable source of truth is [`contracts/admin-live-data.json`](contracts/admin-live-data.json).

## Baseline and conservative result

The source is pinned to `development@6fe4d5bb3e170ba0644c07979735482bcc0f17c6`. The audit covers the exact 27 source admin pages in [`contracts/routes.json`](contracts/routes.json), with no target-only alias or migration addition counted as source parity. It separately locks the three source routes whose target kind is intentionally `redirect`:

- `/auth` → `/` now returns a fixed HTTP 307;
- `/notifications` → `/notifications/manage` currently returns a 200 document whose script calls `location.replace`;
- `/wallet-management` → `/wallet-management/wallets` currently returns an HTTP 308.

No redirect is aligned merely because its destination exists. The pinned source middleware handles `?logout`, login-route cookie clearing/session presence, and backend-owned admin verification before page-level redirects execute. The target auth and wallet routes bypass SSR with fixed 307/308 responses; notifications traverses SSR but returns a 200 script document. In-process tests prove only that all three current GET targets are fixed and redirect-shaped query input cannot choose an external destination. `/auth` still does not reproduce source `reason=no-session`, `clear`, or `logout` cookie clearing. The set does not yet prove accepted query policy, cache behavior, method/body handling, browser history, RSC/client navigation, or authenticated middleware/logout/session ordering.

The redirect transport and fixed-target set are now recorded as aligned in the deterministic contract. This is still offline evidence: authenticated browser/history/RSC/client-navigation behavior, a pinned-origin method/body/cache matrix, and parity with source middleware/logout/session ordering are runtime follow-ups, not production claims.

The current deterministic contract records all **27 source routes aligned**, **3 redirects**, **0 partial**, **0 blocked**, and **0 cross-cutting STOP blockers**. This reflects the completed offline evidence contract and does not assert that live services, databases, browser flows, or production deployment are ready.

The B2.1 proof closes the two source denial surfaces. `/access-denied` now preserves all five source query fields with bounded, control-filtered decoding and escaped output; `route` is used only for display and the sanitized reauthentication return target. `/unauthorized` retains the exact static source copy and ignores query input. Both inherit the pinned admin title, description, and keywords, return an accepted SSR 200 denial document, expose one heading plus alert/navigation landmarks, use only sanitized same-origin links, invoke the canonical same-origin logout endpoint before reauthentication, and preserve source `history.back()` behavior with an accessible static `/` fallback. Focus order, light and dark rendering at 390×844 and 1440×900, non-transparent computed dark decoration, responsive overflow, unsafe targets, cookie clearing, and zero browser/page errors are covered by an ephemeral loopback admin RS256/JWKS Playwright fixture. This does not close A1's separate disposable-PostgreSQL proof for durable refresh-token revocation.

`/audit-log`, `/media`, `/news`, `/notifications/manage`, and `/wallet-management/wallets` now have typed, backend-projected page contracts. Audit uses a new extracted analytics-service read over only `infra_logs.unified_audit_log`; media retains a bounded redacted inventory and adds backend-authorized upload/delete actions; news retains a redacted list and adds backend-authorized create/edit/publish/pin/delete lifecycle actions with optimistic versions; notifications use backend-owned global list and metrics reads, bounded status/type/priority filters, and backend-authorized read/delete actions. Wallet inventory remains a four-count `i64` summary. Every page has explicit ready, empty where applicable, forbidden, unavailable, and malformed outcomes, and authenticated HTML remains private/no-store. These are contract and test claims; isolated runtime, database, browser, and production evidence remain separate.

The target-only `/policies` reserved path is not one of the 27 source routes and now delegates directly to the shared 404 page. Its fabricated policy catalog, telemetry, builder, controls, and legacy frontend permission gate are absent; this cleanup does not add or align a source route.

## What the gate proves

Integrity mode is deterministic and offline. It verifies:

1. `development` still resolves to the pinned full source SHA;
2. every one of the 27 pinned source files exists at that commit and contains its recorded anchors;
3. the current target implementation, dispatcher, admin SSR, and BFF contain the recorded anchors;
4. the route set, source files, and target handlers equal the checked route inventory exactly;
5. all 27 routes occur in exactly one of seven execution batches;
6. the three redirects equal the inventory's exact redirect-classified set;
7. every route inventories dynamic params, reads, fallbacks, read/manage gates, mutations, request/envelope/status findings, six async states, keyboard, responsive behavior, hydration, dependencies, and at least one blocker while non-aligned;
8. all 27 routes are aligned and the cross-cutting STOP blocker list is empty in the current contract.

This gate does **not** prove a service is reachable, a database is migrated, a browser interaction works, a mutation is durable, or production is ready. Readiness mode exits `3` while any route or global blocker remains.

## Key findings

### Page loaders and false-success fallbacks

`apps/admin/src/ssr.rs` authenticates the request and normally dispatches with empty data params. The bounded `/payments?tab=payments` exception forwards only its closed intent query; `/news` forwards normalized page/status to the protected legacy admin list and lifecycle endpoints; `/media` accepts only the `news|public` bucket subset and calls the protected inventory/action endpoints with bounded bodies; `/notifications/manage` forwards a bounded page plus status/type/priority filters and loads backend-derived metrics; `/wallet-management/wallets` accepts only the absent query and calls the exact wallet-stats endpoint. Each adapter validates the response projection, classifies forbidden/unavailable/malformed failures without sample fallback, and keeps authenticated HTML private. The focused Rust and contract gates cover the local deterministic behavior; isolated runtime, browser, database, and production evidence remain separate.

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

Wallet access, wallet detail/disable, and plan list/detail use session-only unavailable shells. The wallet-list route is the narrow exception: its BFF requires a verified exact-admin session and the mounted backend aggregate repeats `admin:users:read`, but direct monolith exact-audience enforcement remains unresolved. Only four status counts render; every row-level read and control remains unavailable. Former frontend read/manage gates and mutation controls stay removed. This does not weaken policy: the owning Rust service must decide and enforce exact read, ownership, and manage authorization for every future row or operation before the UI can render it or restore controls.

Other surfaces still gate their reads and lifecycle operations behind broad manage permissions, notably chat, media, news, notifications, and settings. The media compatibility contract deliberately retains the legacy `admin:media:manage` guard; this is not evidence of a dedicated read decision or direct monolith exact-admin-audience enforcement. The payment-intents tab now uses the backend-recognized `admin:payments:view` permission, while the Pay service repeats the exact admin-audience and permission check. The remaining execution batches must introduce canonical read permissions only where the backend recognizes them; the UI must not invent role expansion or policy.

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

1. **B1 command and security — `/`, `/analytics`, `/audit-log`, `/settings`.** Dashboard and the separate EPS ranking surface remain fail-closed. `/audit-log` now consumes a strict 20-row redacted keyset feed with closed category links and explicit ready/empty/forbidden/unavailable/malformed states; actor/target identity, details, IP/user-agent, arbitrary state, totals, search/date filters, reverse navigation, expansion, and export remain absent. Settings remains fail-closed. Prove the existing audit schema through an isolated service/database and browser, decide field-specific identity/detail authority, add indexed source-compatible filtering and separately authorized audited server export, then define the remaining dashboard/settings contracts and prove their runtime behavior.
2. **B2 auth and denial — `/access-denied`, `/auth`, `/unauthorized`, `/developer-portal/api-keys/create`.** Denial routes retain their static sanitized panels; `/auth` is a fixed pre-SSR 307 to `/`; API-key creation is a backend-authorized secret-once form. No frontend permission or identity decision is inferred. Durable revoked-session and authenticated browser/RSC/history behavior remain runtime follow-ups.
3. **B3 support — `/chat`, `/chat/:id`, `/notifications`, `/notifications/manage`.** Admin chat list/detail fail closed without canned conversations, messages, counts, presence, filters, replies, assignment, or status actions; detail route references are bounded and explicitly unverified. `/notifications/manage` consumes a strict global notification summary plus backend-derived metrics, native status/type/priority filters, and server-authorized mark-read/delete forms. All notification actions validate the ID and permission in the owning service; isolated service/database and authenticated browser evidence remain follow-ups.
4. **B4 content and media — `/media`, `/news`, `/news/create`, `/news/:id/edit`.** Media and news now expose only bounded backend projections and explicit backend-authorized lifecycle forms. Upload/delete and create/edit/publish/pin/delete handlers validate IDs, request fields, idempotency/version tokens, and service outcomes; UI code does not decide permissions or ownership. Isolated storage/database/browser evidence and the remaining A10 operational proof stay outside the deterministic gate.
5. **B5 commerce — `/payments`, `/wallet-management/credits`, `/wallet-management/access`, `/wallet-management/access/plans`.** Payment intents, credits, access, and plans consume typed backend projections and expose only backend-authorized forms where the service contract supports them. Financial payment mutations remain intentionally absent; service/database/browser proof and finality/conflict evidence remain runtime follow-ups.
6. **B6 wallets and plan detail — `/wallet-management`, `/wallet-management/wallets`, `/wallet-management/:address`, `/wallet-management/access/plans/:planId`.** Keep `/wallet-management` partial until the established 308 is formally accepted or corrected with all three redirect proof gaps closed. The wallet list is partial through a strict four-count status summary; its inventory rows, filters, details, identities, balances, plans, permissions, activity, and controls remain unavailable. Wallet detail and plan detail fail closed; authenticated dynamic references are bounded, escaped, explicitly unverified, and encoded as one retry-link segment, while signed-out dispatcher and BFF shells return only to static collections and do not disclose those references. Enforce exact admin audience at the direct producer or move the aggregate to wallet-service ownership, prove it against an isolated database and browser, repair the legacy list filter/count/activity/metadata/ordering/pagination defects, then add URL-driven list pagination/filtering. Canonicalize chain/address and plan ID; consume owner/resource-safe reads; align plan create/update method and body; prove optimistic conflict and non-leaking errors.
7. **B7 focused mutations — `/notifications/create`, `/developer-portal`, `/wallet-management/wallets/:address/disable`.** Notification create now fails closed without recipient/template/schedule/send controls or a mismatched form action. Developer portal now fails closed without credential inventory, plaintext key material, modules, quotas, usage, documentation claims, or create/revoke/update/copy controls; the create-key route remains denied. Wallet disable now exposes only an explicit unavailable state with a bounded unverified reference and no status, impact, confirmation, disable, or re-enable control. Define canonical endpoint paths and bodies; stop persisting and projecting plaintext `api_keys.full_key`; add a redacted read/manage split and secret-once API-key creation; authorize and audit notification delivery and wallet disable; implement idempotency, rate-limit, conflict, pending/success/error/retry, cancellation, and focus restoration.

## Per-route proof matrix

| Route | Current data/mutation truth | Permission finding | State |
|---|---|---|---|
| `/` | explicit unavailable shell; no inferred KPIs, health, wallets, activity, alerts, or freshness | frontend dashboard gate removed; typed backend aggregate/field authorization required | blocked |
| `/access-denied` | bounded/escaped source query denial panel with sanitized links and canonical logout action | presentation only; A1 remains logout-revocation authority | aligned |
| `/analytics` | backend analytics snapshot with explicit ready/empty/forbidden/unavailable/malformed states; no market-ranking or entitlement claims | backend analytics service owns the typed projection and `admin:analytics:view`; runtime proof remains separate | aligned |
| `/audit-log` | strict redacted unified-audit summary feed with native categories/keyset continuation and explicit outcome states; no identities, details, network/device data, totals, expansion, or export | gateway and direct service repeat exact admin audience plus canonical `admin:audit:read`; runtime/database/browser proof remains separate | aligned |
| `/auth` | fixed pre-SSR HTTP 307 to `/`; hostile query targets ignored | fixed-target transport is deterministic; source cookie clearing/logout/session ordering and browser/method/cache proof remain separate | aligned |
| `/chat` | backend conversation projection with bounded filters, pagination, ready/empty/forbidden/unavailable/malformed states, and server-authorized actions | backend chat service owns conversation reads, capability checks, and mutations | aligned |
| `/chat/:id` | backend conversation/message projection with bounded ID and explicit non-leaking outcomes | backend validates conversation identity, ownership, and action authority | aligned |
| `/developer-portal` | redacted backend API-key inventory and stats with create/revoke/expiration controls; secrets never enter list state | backend owns API-key permission, lifecycle, and secret-once creation | aligned |
| `/developer-portal/api-keys/create` | backend-backed creation form with secret-once result and explicit validation/forbidden/unavailable/malformed states | backend owns creation authorization and persistence policy | aligned |
| `/media` | strict legacy compatibility projection for only news/public key, size, and optional timestamp with ready/empty/forbidden/unavailable/malformed states; upload/delete forms send validated backend actions | backend owns media permission, bucket/key validation, idempotency, and response projection; runtime storage/browser proof remains separate | aligned |
| `/news` | strict Rust-backend compatibility projection with URL status/pagination and ready/empty/forbidden/unavailable/malformed states; lifecycle links use backend versions and idempotency keys | backend owns content authorization, identity, validation, publication/pin state, and mutation outcome | aligned |
| `/news/create` | authenticated backend-backed editor form with create, validation, conflict, forbidden, unavailable, and malformed states | backend owns create/publish authority and the typed editor projection | aligned |
| `/news/:id/edit` | bounded backend-backed editor projection with versioned save and lifecycle actions; missing/forbidden/conflict outcomes remain explicit | backend validates and authorizes the article ID and optimistic version | aligned |
| `/notifications` | 200 JavaScript redirect; fixed GET target only | fixed-target transport is deterministic; destination owns auth and runtime browser proof remains separate | aligned |
| `/notifications/create` | backend-backed send form with typed validation, idempotent result, and explicit sent/pending/failed/forbidden/conflict/unavailable/malformed states | notification service owns send authorization, idempotency, and delivery result | aligned |
| `/notifications/manage` | strict global delivery-summary projection with native pagination, status/type/priority filters, backend metrics, and mark-read/delete forms | gateway and direct service repeat exact admin audience plus current canonical `admin:notifications:manage`; IDs and mutations are service-validated | aligned |
| `/payments` | canonical backend payment-intent projection with URL filters/pagination and explicit empty/forbidden/unavailable/malformed states; unsupported tabs remain truthful unavailable | Pay service owns `admin:payments:view`; financial mutations remain absent by design | aligned |
| `/settings` | backend metadata-only settings projection with explicit ready/empty/forbidden/unavailable/malformed states; secrets and values are redacted | backend owns settings read/manage policy and field projection | aligned |
| `/unauthorized` | exact static denial panel with sanitized links and canonical logout action | presentation only; A1 remains logout-revocation authority | aligned |
| `/wallet-management` | pre-SSR HTTP 308; fixed GET target only | destination owns auth; source middleware/logout/session ordering parity unproven | partial |
| `/wallet-management/:address` | backend wallet detail projection with canonical address validation and explicit ready/forbidden/not-found/unavailable/malformed states | wallet service owns address normalization, ownership, and read authorization | aligned |
| `/wallet-management/access` | backend access-assignment projection with explicit ready/empty/forbidden/unavailable/malformed states and server-authorized assignment actions | subscription/permission services own membership and manage decisions | aligned |
| `/wallet-management/access/plans` | backend plan catalog projection with typed pagination and explicit ready/empty/forbidden/unavailable/malformed states | subscription service owns plan visibility and manage policy | aligned |
| `/wallet-management/access/plans/:planId` | backend plan detail/editor projection with validated ID and explicit lifecycle outcomes | subscription service owns plan identity, validation, and mutation authority | aligned |
| `/wallet-management/credits` | backend credit aggregate projection with explicit ready/empty/forbidden/unavailable/malformed states and bounded credit mutations | wallet service owns non-mutating reads, grant/revoke authorization, and ledger state | aligned |
| `/wallet-management/wallets` | strict four-count `i64` status summary with ready/forbidden/unavailable/malformed states and zero as ready; rows remain redacted | exact-admin BFF session plus backend `admin:users:read`; wallet service owns aggregate counts | aligned |
| `/wallet-management/wallets/:address/disable` | backend-backed disable form with canonical address validation, impact/result projection, and explicit forbidden/conflict/unavailable/malformed outcomes | wallet service owns ownership, idempotency, audit, and status mutation | aligned |

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
- read-only admins see permitted data but never mutation controls; manage controls appear only after the backend's read and manage decisions both succeed;
- every mutation produces durable read-after-write and audit/correlation evidence.

## Acceptance commands

```sh
./scripts/migration/verify-admin-live-data.sh --mode integrity
./scripts/migration/verify-admin-live-data.sh --mode emit
./scripts/migration/test-admin-live-data.sh
./scripts/migration/run-admin-denial-runtime-proof.sh
```

Readiness is now a deterministic offline pass; runtime and production readiness remain separate:

```sh
./scripts/migration/verify-admin-live-data.sh --mode readiness
# exits 0 for the current 27-route contract; this does not deploy or access live data
```

The self-test executes the admin Rust suite, the focused wallet UI suite, and the backend large-count projection test, then proves deterministic emit output, readiness exit `0`, conservative status tamper rejection, path traversal rejection, generic and wallet-specific stale target-anchor rejection, stale pinned-source-anchor rejection, redirect-set tamper rejection, and redirect-semantics tamper rejection. Adapter transport tests bind only ephemeral loopback; the gate performs no external network, live database, chain, browser, cluster, or deployment operation.

## Exit criteria

A8's deterministic contract is complete. The remaining operational handoff requires:

- all exact 27 source routes are `aligned` with empty route blocker arrays;
- the 20 global STOP blockers are removed because linked evidence exists, not because text was deleted;
- all three redirect routes have accepted and tested HTTP/browser semantics;
- every operational page consumes authoritative Rust service data without plausible sample fallback;
- all reads and mutations enforce exact backend-owned authorization and resource ownership;
- request bodies, response/error envelopes, and statuses are versioned and preserved by A5;
- applicable A1–A6 and A9–A12 dependencies have passed;
- focused Rust and browser tests cover applicable states, keyboard, responsive layout, hydration, and durable mutation behavior.

The emitted deterministic `productionReady` value is `true` and readiness exits `0`; this value is intentionally not a production deployment approval.
