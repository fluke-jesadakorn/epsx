# Production readiness plan: Dioxus and Rust microservices

Last evidence review: 2026-07-22 (Asia/Bangkok)

## Purpose and safety boundary

This plan moves `migration/dioxus-microservices` toward production parity with
`origin/development` without treating file coverage, route dispatch, compilation,
or visual similarity as functional completion.

The default release strategy is a **controlled hybrid**:

- `apps/backend` remains the source of truth for authentication, sessions,
  permissions, plans, subscriptions, credits, and durable business data.
- Dioxus frontend and admin remain UI/BFF layers. They may display backend
  decisions but must not implement permission, plan, ranking-offset, feature-
  flag, or subscription business rules.
- An extracted service receives production traffic only after its vertical
  contract, authorization, migration, observability, and rollback gates pass.
- Monolith fallbacks are removed last, one domain at a time.

> **Production guard:** do not deploy, apply Kubernetes resources, mutate a
> production database, change Cloudflare/DNS routing, restart production
> workloads, or remove a production fallback without explicit user approval for
> that exact action. Completing an agent package does not authorize deployment.

## Audited baseline

### Source baseline

- Branch/ref: `origin/development`
- Audited commit: `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`
- Behavioral baseline: the existing Rust monolith plus the production frontend
  and admin contracts represented on the development branch.
- Canonical backend rules: `apps/backend`, including SIWE/OIDC session handling,
  granular permissions, active plans, ranking offsets, payments, subscriptions,
  credits, notifications, and analytics routes.

### Target baseline

- Branch/ref: `migration/dioxus-microservices`
- Audited commit before this plan: `975c09567fe14ce278370720bd7a0e5aa571e116`
- UI/BFF targets: `apps/frontend`, `apps/admin`, and `apps/pay`.
- Candidate extracted services: `services/gateway`, `services/identity`,
  `services/wallet`, `services/pay`, `services/subscription`, `services/content`,
  `services/notification`, `services/analytics`, and `services/indexer`.
- Deployed production base currently includes the monolith, Dioxus frontend and
  admin, `apps/analytics`, a separate identity ranking-offset stub, and pay
  service/BFF resources. It does not deploy most candidate services.

The two identity implementations are not interchangeable:

1. `services/identity` is an HTTP SIWE/JWT/user service with its own database.
2. `shared/rust/epsx-identity-service` is the deployed gRPC/SSE ranking-offset
   service and currently returns the free-plan offset for every wallet.

Likewise, `apps/analytics` is the deployed market-ranking extraction while
`services/analytics` is a separate event-tracking service. Names alone must not
be used to infer parity or deployment.

## Route coverage: presence is not parity

The audited path inventory is in `docs/wave21-pixel-recheck/route-inventory.md`.

| Surface | Source pages | Target path counterparts | Path result | Important distinction |
|---|---:|---:|---|---|
| Frontend | 28 | 28 | 28/28 | Includes dynamic examples such as chat, news, and payment; says nothing about live behavior. |
| Admin | 27 | 27 | 27/27 | Two source paths intentionally redirect to canonical management paths. |

The frontend E2E inventory has 28 representative paths. The admin E2E inventory
has 29 scenarios because it includes target routing samples/additions beyond the
27-page source inventory. E2E scenario count and source-page count answer
different questions.

For each path, parity has six independent layers:

1. **Path:** a dispatcher recognizes the URL.
2. **Render:** the route returns meaningful target markup rather than a generic
   fallback, redirect loop, skeleton-only response, or sample page.
3. **Visual:** responsive layout, content hierarchy, states, and accessibility
   match the accepted baseline.
4. **Interaction:** every link, form, filter, modal, pagination control, wallet
   action, and error/retry state works with and without hydration as designed.
5. **Data and security:** the route uses live authorized data and backend-owned
   business decisions, with correct empty/loading/error states.
6. **Operations:** downstream failures, retries, observability, migrations,
   readiness, canary, and rollback are proven.

Only layer 1 is complete across 28/28 frontend and 27/27 admin source paths.

## Current architecture and production hot path

```text
epsx.io / admin.epsx.io
  -> Dioxus frontend/admin BFF
  -> shared API_URL / api.epsx.io
  -> apps/backend monolith

market analytics
  -> apps/analytics
  -> epsx-identity-service gRPC/SSE
  -> free-plan ranking-offset stub

pay.epsx.io
  -> Cloudflare localhost:4747
  -> NodePort 30082
  -> epsx-pay-svc directly (not the BFF at NodePort 30083)
```

Most `services/*` binaries compile but are not deployed by
`infrastructure/kubernetes/base/kustomization.yaml`. A service is not considered
migrated merely because a binary and route table exist.

## Evidence-backed findings

### Interaction and UI behavior

- Frontend account, rankings, plans, subscriptions, news, portfolio, credits,
  developer/API-key, usage, analytics, dashboard, and payment responses include
  static/sample payloads in `apps/frontend/src/api.rs`.
- Admin exposes proxy handlers, but SSR constructs empty page parameters in
  `apps/admin/src/ssr.rs`; multiple pages therefore render samples or skeletons
  instead of live domain state.
- `/payment` in the shared frontend UI intentionally renders a redirect stub to
  `pay.epsx.io`. Its tests must assert the redirect contract, not the retired
  embedded wizard.
- Server-rendered interaction must be tested explicitly. A component appearing
  in HTML does not prove that its click, submit, focus, validation, retry, or
  navigation behavior works.

### Authentication and session behavior

- The monolith remains the canonical issuer and durable session store. It emits
  persistent-key RS256 tokens and a bounded current/backup JWKS document.
- Both Rust BFFs now use their exact frontend/admin audience, verify issuer,
  audience, lifetime, subject/wallet equality, algorithm, and `kid` before
  establishing or forwarding a session, and preserve backend permissions
  verbatim.
- Both BFFs use the shared host-only HttpOnly access/refresh cookie pair, rotate
  it from the refresh cookie only, return token-free browser JSON, and always
  clear local state on logout. Logout is wired to monolith refresh revocation.
- A1.4 provides hermetic mock-backed proof for these contracts, but no real
  wallet/nonces or disposable-database test yet proves old-token rejection and
  durable revocation across the complete flow.
- The candidate HTTP identity service uses the same claim shape for access and
  refresh tokens and has no durable refresh rotation/revocation model.

Until the identity extraction passes parity, the monolith issuer, session store,
and revocation behavior remain canonical.

### Backend-only permissions and plans

- The gateway now verifies exact RS256/JWKS issuer and frontend/admin audience
  claims, uses a method-and-path allowlist, and denies unknown, internal, and
  unresolved payment routes before upstream I/O. This is an edge boundary only;
  it does not prove owner checks or direct-service isolation.
- Candidate subscription, wallet, content, notification, analytics, indexer,
  and pay routers now have direct fail-closed boundaries, but their route
  matrices remain partial or blocked until owner models, internal-service
  identity, domain semantics, and runtime integration are proven.
- Pay admin force-cancel/release/refund shapes now verify the admin audience and
  `admin:payments:manage`, then return `404` before their current DB-only
  handlers. Those handlers remain unsuitable for production until real chain,
  transaction, idempotency, and recovery contracts replace them.
- Both BFFs preserve verified backend permissions without expanding roles. The
  68-record permission inventory contains 53 canonical three-segment records,
  13 legacy two-segment gates, one unknown record, and one impossible/cross-
  grammar record. UI gates remain presentation controls, never policy authority.
- The deployed identity ranking service returns offset `100` for all wallets,
  including paid users. This is not acceptable entitlement behavior.

### Checkout and payment

- Pay BFF calls singular `/api/v1/pay/intent/{id}/execute`; the service exposes
  plural `/api/v1/pay/intents/{id}/confirm` and `/cancel` with no execute route.
- Gateway uses `/api/v1/payment/*` while the service exposes `/api/v1/pay/*` and
  does not rewrite the prefix.
- Intent confirmation trusts a supplied transaction hash without validating the
  receipt, chain, amount, token, payer, payee, or confirmations.
- Escrow release/refund/dispute handlers update PostgreSQL state rather than
  submitting and confirming escrow-contract operations.
- Kubernetes supplies `ESCROW_CONTRACT=0`; the on-chain webhook secret is not
  configured in the pay deployment.
- Cloudflare currently exposes the pay service port rather than the pay BFF
  port, expanding the unauthenticated service surface.

### Data and migration safety

- Candidate services generally create schemas at runtime and lack versioned
  service migrations/backfills.
- Provisioning creates `epsx_payments_*` databases while pay service manifests
  and compose files also refer to `epsx_pay`; other candidate service database
  names are not consistently provisioned.
- Applied migration history has been removed or edited relative to development.
  Existing databases will not rerun an edited baseline, while new databases can
  receive a different schema.
- Analytics request-usage partitions stop at 2026-04-01 without a default
  partition, so current writes can fail.
- Payments plan replication creates a future-write trigger but delegates the
  initial copy to a separate manual script.
- A notification migration drops `notification_subscriptions CASCADE`; data
  preservation and dependency impact are not demonstrated.

All structural changes must be new additive migrations with idempotent backfills,
row-count reconciliation, rollback/forward-fix procedures, and production-sized
timing evidence. Existing production data must not be dropped to simplify a cut.

### Infrastructure and release safety

- Rendering the production overlay currently produces `epsx-admin:dev`,
  `epsx-frontend:dev`, and `epsx-identity:dev`; image replacement keys do not
  match base image names. Pay tags also do not resolve to the intended wave tag.
- Most candidate microservices are absent from the production Kustomize base.
- Health endpoints are generally liveness-only and do not prove database,
  Redis, identity, chain RPC, or downstream readiness.
- There is no complete shadow/canary/rollback proof for the new topology.

## Definition-of-done scorecard

Scoring is evidence-based: `0 = not demonstrated`, `1 = partial`, `2 = gate
passed`. It is not a percentage estimate of engineering effort.

| Gate | Score | Current evidence | Done when |
|---|---:|---|---|
| Frontend path presence | 2 | 28/28 dispatcher counterparts | Inventory and dispatcher contract test stay green. |
| Admin path presence | 2 | 27/27 counterparts; two canonical redirects | Redirects and dynamic params have contract tests. |
| Shared UI package baseline | 2 | Targeted unit/doctest repair in this slice | `cargo test -p epsx-dioxus-ui --lib` and `--doc` pass. |
| Visual/responsive/accessibility | 1 | Historical screenshots exist; current accepted baseline is incomplete | All routes pass agreed viewport, state, keyboard, and accessibility thresholds. |
| Interaction parity | 0 | No complete click/form/wallet/navigation matrix | Every interactive control has E2E success and failure coverage. |
| Auth/session parity | 1 | A1.4 hermetic gate covers 71 focused tests across both BFFs; durable database-backed rotation/revocation and a real wallet flow remain unproven | SIWE -> SSR me -> rotation -> revocation works across both BFFs. |
| Backend authorization | 1 | Gateway is fail-closed with exact RS256/JWKS and granular edge policy; direct services and owner checks remain blocked | Anonymous/cross-owner calls fail at both gateway and service boundaries; granular backend permissions pass. |
| Live data parity | 0 | Frontend mocks and admin empty params remain | Sample payloads removed and real empty/error states proven. |
| Checkout/on-chain parity | 0 | Route mismatch and DB-only escrow transitions | Verified receipts and contract transactions drive state. |
| Backend/API contract parity | 1 | Both BFFs now return explicit HTML/JSON 404s and preserve 405/redirect semantics; payment prefixes and broader payload/status drift remain | Versioned contract matrix passes for monolith and replacement. |
| Migration/data safety | 0 | Runtime DDL, naming drift, baseline edits, expired partitions | Upgrade/backfill/reconcile/rollback tests pass on production-shaped data. |
| Production manifests/routing | 0 | Dev tags and direct pay-service ingress remain | Rendered manifests use approved immutable images and intended BFF ingress. |
| Observability/readiness | 0 | Shallow health checks and incomplete cross-service traces | Dependency readiness, SLO metrics, alerts, and trace IDs pass drills. |
| Canary/rollback | 0 | Not demonstrated | Shadow, canary, abort thresholds, and rollback rehearsal are approved. |

**Current evidence score: 10/28.** This score records gate evidence only. It must
not be used to forecast dates or authorize production traffic.

## Dependency DAG

```mermaid
flowchart TD
    B0["P0: Baseline contract and test harness"]
    B1["P0: Additive migration and database safety"]
    B2["P0: Canonical issuer, cookies, refresh, logout"]
    B3["P0: Fail-closed authentication, ownership, permissions"]
    B4["P0: Canonical plan and ranking entitlements"]
    B5["P0: BFF and service API contract alignment"]
    B6["P0: Verified on-chain checkout vertical slice"]
    B7["P1: Frontend/admin live data and interactions"]
    B8["P1: Content, notifications, analytics vertical slices"]
    B9["P1: Immutable manifests, readiness, internal routing"]
    B10["P2: Shadow, canary, SLO and rollback rehearsal"]

    B0 --> B1
    B0 --> B2
    B0 --> B3
    B0 --> B5
    B1 --> B4
    B2 --> B3
    B2 --> B5
    B3 --> B4
    B3 --> B6
    B4 --> B7
    B5 --> B6
    B5 --> B7
    B1 --> B8
    B3 --> B8
    B7 --> B9
    B8 --> B9
    B6 --> B9
    B9 --> B10
```

### Priority interpretation

- **P0:** security, canonical state, contract, payment, and data-safety work that
  blocks any traffic cutover.
- **P1:** domain and UI parity needed for a usable release after P0 gates pass.
- **P2:** scale, resilience, cleanup, and release optimization after correctness
  and rollback are proven.

## Agent-sized execution packages

Agents may work in parallel only where dependencies permit. Each package owns a
bounded path set. Shared contract files require coordination through package A0.

### A0 — Baseline contracts and test harness (P0)

- **Scope:** create a machine-readable route/API/cookie/status inventory under
  `docs/migration/contracts/`; add non-production test scripts under
  `scripts/migration/`; cover 28 frontend and 27 admin source paths, dynamic
  params, two admin canonical redirects, and monolith API baselines.
- **Do not change:** runtime routing, business logic, database schemas, or infra.
- **Dependencies:** none.
- **Acceptance:**

  ```bash
  cargo test -p epsx-dioxus-ui --lib
  cargo test -p epsx-dioxus-ui --doc
  ./scripts/migration/verify-route-inventory.sh
  ./scripts/migration/verify-contract-fixtures.sh
  ```

### A1 — Canonical authentication and session compatibility (P0)

- **Scope:** `apps/frontend/src/auth.rs`, frontend auth handlers in
  `apps/frontend/src/api.rs`, `apps/admin/src/auth.rs`, admin auth handlers in
  `apps/admin/src/main.rs`, and shared verifier/client changes strictly needed
  for monolith RS256 claims, secure cookies, refresh rotation, and logout
  revocation.
- **Strategy:** keep the monolith issuer/session store canonical. Do not activate
  `services/identity` as issuer in this package.
- **Dependencies:** A0.
- **Acceptance:**

  ```bash
  cargo test -p epsx-frontend auth
  cargo test -p epsx-admin auth
  cargo test -p epsx-auth
  ./scripts/migration/verify-auth-session-flow.sh
  ```

  The script must prove challenge, signed verification, SSR `/me`, refresh-token
  rotation, old-token rejection, logout revocation, secure cookie attributes,
  and access-token rejection when a refresh token is supplied.

- **A1.4 status:** the local hermetic gate passes only when its 71 focused tests
  and both baseline fixture checks pass. It proves BFF audience/verifier,
  token-redaction, cookie, local rotation/clearing, proxy rejection, and safe
  return-target contracts. It deliberately does not satisfy the full A1
  acceptance condition: real wallet signing, nonce consumption, durable
  database-backed old-token rejection/revocation, and production-shaped browser
  behavior remain blocked. See `docs/migration/A1_4_AUTH_SESSION_GATE.md`.

### A2 — Fail-closed service authorization (P0)

- **Scope:** `services/gateway`, shared auth middleware, and authentication,
  ownership, and granular permission guards for candidate service endpoints.
  Add a public-route allowlist; everything else is protected by default.
- **Dependencies:** A0 and A1 token contract.
- **Acceptance:**

  ```bash
  cargo test -p epsx-gateway
  cargo test -p epsx-identity
  cargo test -p epsx-wallet
  cargo test -p epsx-pay-svc
  cargo test -p epsx-subscription
  cargo test -p epsx-content
  cargo test -p epsx-notification
  cargo test -p epsx-analytics
  cargo test -p epsx-indexer
  ./scripts/migration/verify-service-authorization.sh
  ```

  The matrix must cover anonymous, expired, wrong audience, ordinary user,
  cross-owner, and granular-admin cases for every mutation.

- **A2.1 status:** gateway edge enforcement passes 18 focused tests and the
  117-route authorization fixture remains integrity-clean. The fixture still
  reports readiness as not proven because direct-service verification,
  cross-owner denial, internal service identity, and handler-level permission
  enforcement remain open. See `docs/migration/A2_GATEWAY_AUTHORIZATION.md`.

- **A2.2 status:** the canonical RS256/JWKS verifier and strict bearer-header
  API now live in `epsx-service-auth`, and the gateway consumes that shared
  implementation. Seven direct-service routers now consume the same verifier,
  while each route still requires its own audience, ownership, permission, and
  fail-closed evidence; extraction alone is not service-level authorization
  proof.

- **A2.3a status:** the candidate event-analytics service now consumes the
  shared verifier directly. Health is the sole anonymous allowlist, tracking
  requires a verified frontend/admin principal, operator reads require the
  admin audience plus `admin:analytics:view`, and internal/unknown routes fail
  closed before storage. Canonical wallet attribution, internal-service
  identity, runtime migrations, and analytics semantics remain unresolved.

- **A2.3b status:** the candidate content service also consumes the shared
  verifier. Public route shapes are explicit, CMS mutations require the admin
  audience plus canonical `admin:content:manage`, and editor-session routes
  remain fail-closed rather than persisting a caller-selected UUID. Published-
  only filtering, editor identity mapping, runtime migrations, content-domain
  semantics, and cache/validation behavior remain unresolved.

- **A2.3c status:** the notification service now consumes the shared verifier.
  Health is its only anonymous surface, template/send operations require the
  admin audience plus `admin:notifications:manage`, and all eight user routes
  derive and bind their SQL owner key from the verified wallet. The service
  matrix is now five aligned, 54 partial, and 58 blocked. Runtime DDL and legacy
  owner migration, internal publisher identity, delivery idempotency/outbox,
  SMTP behavior, template consistency, and DB integration remain unresolved.

- **A2.3d status:** the subscription service now consumes the shared verifier.
  Health is its only anonymous surface; plan reads require the admin audience
  plus `admin:plans:read`, and plan creation requires
  `admin:plans:manage`. Owner-subscription and vault routes intentionally return
  `404` before storage until the UUID owner model can be derived from the
  verified wallet and activation can be bound to finalized payment evidence.
  The service matrix is now six aligned, 53 partial, and 58 blocked. All 20 A9
  lifecycle blockers remain open; this boundary does not prove subscription
  production readiness.

- **A2.3e status:** the indexer now consumes the shared verifier. Health is the
  only anonymous surface. All four read projections return `404` before DB/RPC
  work because canonical ingestion, finality, reorg, and privacy semantics are
  unproven. Sync requires the admin audience plus `admin:indexer:manage`, then
  returns `404` before placeholder ingestion. A12 retains all 24 STOP blockers.

- **A2.3f status:** the pay service now consumes the shared verifier. Health and
  a bounded active pay-link projection are the only anonymous surfaces. Owner
  intent, escrow, and history reads bind SQL to the verified wallet; admin reads
  require `admin:payments:view`. All 14 financial/internal mutations remain
  unavailable; admin mutation shapes verify `admin:payments:manage` and then
  fail closed. The service matrix is now eight aligned, 52 partial, and 57
  blocked. All 17 A6 STOP blockers remain open.

- **A2.3g status:** the wallet service now consumes the shared verifier. Health
  and an 8 KiB-bounded, read-only signature recovery endpoint are its only
  anonymous surfaces. Account list/detail reads accept only the exact frontend
  or admin audience, derive the owner from the verified wallet, bind that owner
  in SQL, and exclude encrypted key material. Account creation, balance, send,
  signing, and gas estimation return `404` before unsafe custody/RPC handlers.
  The service matrix is now ten aligned, 50 partial, and 57 blocked; wallet
  custody, truthful chain behavior, schema migration, and runtime integration
  remain STOP conditions.

### A3 — Additive migrations and data reconciliation (P0)

- **Scope:** new migration directories only, database provisioning scripts,
  future analytics partitions/default strategy, pay database naming, identity
  backfill design, plan replica backfill, and reconciliation tooling.
- **Do not change:** an already-applied baseline migration; do not drop existing
  production data.
- **Dependencies:** A0 schema/contract inventory.
- **Acceptance:**

  ```bash
  ./scripts/migration/create-ephemeral-databases.sh
  ./scripts/migration/upgrade-development-snapshot.sh
  ./scripts/migration/reconcile-row-counts.sh
  ./scripts/migration/verify-forward-fix.sh
  ```

  Tests must start with a development-shaped database containing representative
  users, active plans, subscriptions, payments, notifications, and usage rows.
  Upgrade and retry must be idempotent; reconciliation must be exact.

- **A3.1/A3.2 status:** the read-only database preflight tooling is constrained
  to capture checksum-pinned, non-production evidence for core, analytics,
  notifications, and payments. The offline A3.2 classifier accepts exactly 13
  known history/schema classes, rejects tampered, hybrid, credential-bearing,
  unknown, traversal,
  symlink, and output-race inputs, and emits only redacted deterministic
  fingerprints. Exit `0` still declares `productionReady: false`; no live
  preflight, reconciliation, migration, repair, or database mutation has run.

### A4 — Canonical permission and entitlement authority (P0)

- **Scope:** move/adapter-wrap the backend `UnifiedPermissionService` behavior
  behind the identity authority; implement live ranking-offset queries and
  entitlement-change events without frontend rule duplication.
- **Strategy:** monolith remains canonical until shadow comparisons are exact.
- **Dependencies:** A1, A2, A3.
- **Acceptance:**

  ```bash
  cargo test -p epsx-identity
  cargo test -p epsx-identity-shared
  cargo test -p epsx-analytics
  ./scripts/migration/verify-entitlement-parity.sh
  ```

  Free, paid, expired, overlapping, revoked, and admin assignments must match the
  monolith decision and ranking offset. Protected paid data must not silently
  fall back to the free plan on authority failure.

- **A4.0/A8.1 status:** the deterministic permission-grammar inventory covers
  68 UI/service records. A8.2 additionally separates wallet-access and plan
  read surfaces from their mutation controls using literal backend guards;
  readiness intentionally stops with 13 legacy security gates and two
  presentation-only drift records. Entitlement and ranking-offset parity in the
  acceptance condition above is not yet implemented.

### A5 — BFF/API contract alignment (P0)

- **Scope:** `shared/rust/client`, frontend/admin/pay proxy handlers, and gateway
  prefix rewriting. Align singular/plural paths, status codes, error envelopes,
  cookie forwarding, correlation IDs, timeouts, and retry policy.
- **Dependencies:** A0 and A1; use A2 authorization contract.
- **Acceptance:**

  ```bash
  cargo test -p epsx-client
  cargo test -p epsx-frontend
  cargo test -p epsx-admin
  cargo test -p epsx-pay-bff
  cargo test -p epsx-gateway
  ./scripts/migration/verify-bff-contracts.sh
  ```

- **A5.0 status:** shared route dispatch and both BFF fallbacks now distinguish
  known routes, malformed dynamic arity, HTML page misses, JSON API misses,
  registered-path method mismatches, and intentional redirects. Payment route
  prefixes, live payloads, correlation/retry policy, and the pay BFF remain in
  later A5/A6 slices.

- **A5.1 status:** `epsx-client` now emits typed, body-free upstream status
  errors. The admin BFF preserves only the closed safe set `400`, `401`, `403`,
  `404`, `409`, `422`, `429`, `502`, `503`, and `504`; unknown statuses and
  legacy string errors become `502`, while timeout and connect failures become
  `504` and `503`. Typed error envelopes, validation detail, retryability,
  correlation, cross-hop consistency, and page consumption remain blocked.

### A6 — Checkout and escrow vertical slice (P0)

- **Scope:** `apps/pay`, `services/pay`, escrow contract adapter, receipt
  validation, webhook/indexer integration, payer/payee ownership, idempotency,
  confirmation depth, release/refund/dispute transactions, and failure recovery.
- **Dependencies:** A1, A2, A3, A5.
- **Acceptance:**

  ```bash
  bun dev:anvil
  bun setup:local
  cargo test -p epsx-pay-svc
  cargo test -p epsx-pay-bff
  ./scripts/migration/verify-pay-anvil-e2e.sh
  ```

  A database state transition passes only after the expected Anvil receipt is
  verified. Replaying create, webhook, confirm, release, or refund must not
  duplicate value or state transitions.

- **A6.0 status:** the pinned payment execution contract inventories nine
  route/lifecycle surfaces and 17 evidence-backed stop blockers. Its integrity
  and tamper tests pass, while readiness intentionally exits `3`; route-prefix,
  ownership, idempotency, receipt/finality, escrow transaction, webhook,
  migration, ingress, and end-to-end browser proof remain unimplemented.

- **A6.1 authorization status:** direct pay-service access now proves verified
  owner/admin read boundaries and hides foreign resources. It deliberately
  keeps every financial, escrow, link, webhook, deposit, resolve, and force
  mutation away from its current DB-only handler until A6 idempotency,
  transaction, chain/finality, audit, and recovery contracts pass.

### A7 — Frontend live data and interaction parity (P1)

- **Scope:** `apps/frontend` loaders/API handlers and frontend pages in
  `shared/rust/dioxus_ui/src/pages`. Replace static payloads route by route;
  implement loading, empty, error, retry, pagination, form, keyboard, wallet,
  and unauthenticated states.
- **Do not implement:** permissions, plan selection rules, ranking offsets,
  subscription eligibility, or payment decisions in the frontend.
- **Dependencies:** A1, A4, A5; checkout routes also depend on A6.
- **Acceptance:**

  ```bash
  cargo test -p epsx-frontend
  cargo test -p epsx-dioxus-ui --lib
  bun test:e2e --project=frontend
  ./scripts/migration/verify-no-frontend-sample-data.sh
  ```

  Close routes in small batches; all 28 must pass interaction and live-data
  fixtures before the frontend gate moves to done.

- **A7.0–A7.3 status:** the exact 28-route live-data contract now records three
  aligned routes, seven partial routes, and 18 blocked routes. `/about`
  removes invented claims and matches the pinned source order, copy, metadata,
  landmarks, and responsive keyboard behavior. `/access-denied`
  has bounded and escaped query rendering plus responsive keyboard browser
  proof. `/manual` exactly matches the pinned 35-feature catalog and proves all
  screenshot assets, responsive layout, links, dialog focus, and image-error
  fallback. `/developer/docs` now matches the pinned ten-endpoint catalog and
  proves responsive navigation, accordions, language tabs, copy controls, and
  keyboard behavior, while live requests remain disabled pending A1/A4/A5.
  `/offline` is public and has a native retry control, but fresh
  disconnected cache/service-worker delivery is not proven. Privacy and terms
  remain partial pending wallet/SIWE legal approval; terms also lacks a real
  subscription handler. The remaining 25 routes keep readiness at exit `3`.

### A8 — Admin live data and mutation parity (P1)

- **Scope:** `apps/admin` SSR/loaders/proxies and admin pages in
  `shared/rust/dioxus_ui/src/pages/admin_pages`. Replace empty params and samples;
  require server-side admin authorization for every mutation.
- **Dependencies:** A1, A2, A4, A5; payment views depend on A6.
- **Acceptance:**

  ```bash
  cargo test -p epsx-admin
  cargo test -p epsx-dioxus-ui --lib
  bun test:e2e --project=admin
  ./scripts/migration/verify-admin-mutation-authorization.sh
  ```

  All 27 source paths, dynamic params, and two canonical redirects must pass.

- **A8.0–A8.1 status:** the pinned admin contract covers the exact 27 source
  routes and both intentional redirects in seven execution batches. It records
  two aligned, two partial, and 23 blocked routes plus 20 cross-cutting STOP
  blockers. `/access-denied` and `/unauthorized` now preserve bounded escaped
  copy, inherited metadata, safe reauthentication/return behavior, keyboard
  order, responsive light/dark layout, and authenticated local browser proof.
  The admin SSR still provides no per-page loader, operational pages render
  samples, several form/BFF paths drift, and preserved statuses still lack
  typed envelopes and page-level consumption. Integrity/tamper checks pass and
  readiness intentionally exits `3`; no live service, database, chain, or
  deployment access is claimed.

### A9 — Subscription vertical slice (P1)

- **Scope:** `services/subscription`, canonical plan reads, verified-payment
  activation, renewal/expiry/cancel/switch flows, entitlement grant/revoke, and
  durable scheduling/outbox behavior.
- **Dependencies:** A2, A3, A4, A5, A6.
- **Acceptance:**

  ```bash
  cargo test -p epsx-subscription
  ./scripts/migration/verify-subscription-lifecycle.sh
  ./scripts/migration/verify-entitlement-parity.sh
  ```

- **A9.0 status:** the pinned subscription contract inventories 12
  route/lifecycle surfaces, 18 source anchors, 25 target anchors, and 20 stop
  blockers. It locks backend plan authority, owner/admin/service boundaries,
  verified-payment activation, manual renewal and expiry, effective-plan
  uniqueness, idempotency, outbox/reconciliation, entitlement/ranking
  projection, truthful UI states, and rollback. Integrity and tamper gates pass;
  readiness intentionally exits `3`. No lifecycle runtime or schema claim is
  made by the audit.

### A10 — Content vertical slice (P1)

- **Scope:** `services/content`, content migrations, public reads, authorized
  draft/create/update/publish/theme operations, media references, and BFF
  contract integration.
- **Dependencies:** A2, A3, A5.
- **Acceptance:**

  ```bash
  cargo test -p epsx-content
  ./scripts/migration/verify-content-lifecycle.sh
  ```

- **A10.0 status:** the pinned content lifecycle contract records 14 source
  anchors, 32 target anchors, eight route batches, 16 lifecycle requirements,
  and 20 stop blockers. It preserves A2.3b as partial and keeps editor routes
  fail-closed until canonical actor mapping and session ownership exist. The
  ordered implementation covers published-only immutable revisions, typed
  page/theme/block CRUD, media, filesystem trust, migrations/reconciliation,
  backend-owned plans/rankings/portfolio, wire/status parity, truthful UI
  states, audit/outbox/idempotency, shadowing, and rollback. Integrity passes;
  readiness intentionally exits `3`.

### A11 — Notification vertical slice (P1)

- **Scope:** `services/notification`, ownership-safe list/read/delete, authorized
  templates/send, outbox delivery, email/in-app delivery state, SSE/realtime
  behavior, retries, and migration/backfill.
- **Dependencies:** A2, A3, A5.
- **Acceptance:**

  ```bash
  cargo test -p epsx-notification
  ./scripts/migration/verify-notification-ownership.sh
  ./scripts/migration/verify-notification-delivery.sh
  ```

- **A11 authorization status:** direct health/admin/owner request boundaries
  are enforced by A2.3c, but the vertical slice remains incomplete until
  additive migrations, legacy owner reconciliation, internal publisher
  identity, durable outbox/retry/dead-letter delivery, safe SMTP behavior,
  realtime ownership, observability, and rollback are proven.

- **A11.0 status:** the deterministic lifecycle contract pins 14 source records
  and 36 target anchors across 12 blocked surfaces. Its 22 STOP blockers cover
  schema/startup DDL drift, truthful asynchronous delivery, preferences/SSE/
  push, publisher inbox/outbox and idempotency, retry/dead-letter behavior,
  templates/privacy, reconciliation, observability, deployability, single-writer
  cutover, and duplicate-safe rollback. Integrity and tamper checks pass;
  readiness intentionally exits `3`, and no live provider or infrastructure
  access is part of the evidence.

### A12 — Analytics/indexer vertical slice (P1)

- **Scope:** explicitly separate market rankings from event analytics; preserve
  canonical entitlement filtering; secure tracking/revenue/admin endpoints;
  add partition maintenance, indexer authentication, checkpoints, replay, and
  idempotency.
- **Dependencies:** A2, A3, A4, A5.
- **Acceptance:**

  ```bash
  cargo test -p epsx-analytics
  cargo test -p epsx-indexer
  ./scripts/migration/verify-ranking-entitlements.sh
  ./scripts/migration/verify-indexer-replay.sh
  ```

- **A12.0 status:** the audit enforces four distinct domains—market analytics,
  event analytics, chain indexing, and identity ranking-offset projection. It
  pins 14 source and 36 target anchors across 16 blocked surfaces, 31 required
  rules, 12 ordered execution phases, and 24 stop blockers. Direct service
  authorization, truthful live/stale UX, authoritative plan offsets, event
  taxonomy/privacy/revenue, canonical finality/reorg-aware indexing, durable
  backfill/reconciliation, observability, distinct workloads, and per-domain
  shadow/cutover/rollback remain open. Integrity passes; readiness intentionally
  exits `3`.

### A13 — Infrastructure, shadow, and rollback (P1/P2)

- **Scope:** fix Kustomize image matching, use immutable approved tags/digests,
  route public pay traffic through the BFF, add readiness checks, internal-only
  service exposure, SLO dashboards, shadow routing, canary thresholds, and a
  rehearsed rollback runbook.
- **Dependencies:** all P0 packages and any P1 domain selected for canary.
- **Acceptance before requesting production approval:**

  ```bash
  kubectl kustomize infrastructure/kubernetes/overlays/prod > /tmp/epsx-prod.yaml
  ! rg 'image: .*:dev' /tmp/epsx-prod.yaml
  ! rg 'ESCROW_CONTRACT.*(^|[=: ]0$)' /tmp/epsx-prod.yaml
  kubectl apply --dry-run=client -f /tmp/epsx-prod.yaml
  ./scripts/migration/verify-public-ingress-map.sh
  ./scripts/migration/verify-readiness-failures.sh
  ./scripts/migration/rehearse-rollback.sh --non-production
  ```

  These commands validate artifacts only. They do not authorize `kubectl apply`
  against production, DNS/Cloudflare changes, or production migrations.

- **A13.0 status:** a hermetic local render gate records 18 stop blockers. The
  current artifact contains three `:dev` images, no digest-pinned images, public
  pay ingress that bypasses the BFF, literal pay database credentials, a zero
  escrow address, no webhook configuration, eight absent candidate services,
  and no startup or dependency-readiness checks. The P0 ledger is one passed,
  four partial, and two blocked; readiness intentionally exits `3`. No cluster
  access or infrastructure mutation is performed by this gate.

## Release gates

Before any production-deployment request is presented to the user:

1. All P0 packages are green and their evidence is linked from the release
   candidate.
2. The selected vertical slice passes development, staging/shadow, data
   reconciliation, security, interaction, readiness, and rollback gates.
3. Rendered manifests contain only reviewed immutable images and secrets are
   referenced rather than embedded.
4. Monolith fallback remains available and tested during the canary.
5. Abort thresholds and rollback ownership are explicit.
6. The user gives explicit approval for the exact production deployment,
   migration, and routing actions to be taken.

Until then, the honest status is: **path coverage exists; production functional
parity and operational readiness do not.**
