# A10.0 Content Lifecycle Execution Plan

Status: **AUDIT INTEGRITY PASSABLE; PRODUCTION READINESS STOPPED**

This plan is the execution gate for migrating the pinned `origin/development` content/news/public-plan/ranking/portfolio behavior to `migration/dioxus-microservices`. It is intentionally not a parity claim. The machine-readable source of truth is [`contracts/content-lifecycle.json`](contracts/content-lifecycle.json), verified by [`verify-content-lifecycle.sh`](../../scripts/migration/verify-content-lifecycle.sh).

## 1. Outcome and hard boundary

The required outcome is production-usable lifecycle behavior, not merely similar screens:

- anonymous reads can observe only immutable published revisions;
- authors, editors, and owners are derived by the backend from verified identity;
- page, theme, block, news, media, plan, ranking, and portfolio contracts remain exact through service, gateway, BFF, and UI;
- every mutation is validated, concurrency-safe, idempotent, audited, and transactionally connected to an outbox;
- migrations, backfill, reconciliation, observability, cutover, and rollback are executable and reviewable;
- loading, true empty, forbidden, not found, error, retry, and success remain distinct UI states.

This A10.0 deliverable does **not** modify runtime code, Cargo manifests, `Cargo.lock`, existing fixtures, UI, BFF, infrastructure, deployment state, databases, or production filesystems. Its verifier reads local Git/files only, refuses production-looking or data-service environments, performs no network/database/object-store/chain access, and reserves readiness as exit `3`.

## 2. Truth statement: what A2.3b and A3.10 did and did not prove

A2.3b is a useful boundary, but only a partial one:

- the content service has an exact method/path public allowlist;
- page/theme administration requires the admin audience plus `admin:content:manage`;
- spoofable identity headers are stripped;
- editor start/commit/list routes deliberately fail closed with HTTP `404` before their handlers run;
- this proves neither content lifecycle parity nor production readiness;
- it does not validate drafts versus published revisions, data provenance, CRUD validation, session ownership, publication atomicity, media integrity, migrations/backfill, entitlements, wire compatibility, UI states, reconciliation, or rollback.

Editor routes must stay fail-closed until phases 4, 5, and 10 below produce reviewed identity, ownership, concurrency, audit, and replay evidence. Making them reachable earlier is a regression, not progress.

A3.10 is also a bounded improvement, not lifecycle completion:

- it removed all four content-service runtime DDL findings and added a tracked additive migration root;
- it kept the service fail-fast on an absent or incompatible schema;
- the root has no reviewed runner or version ledger and has not been adopted on a populated source schema;
- it does not provide immutable revisions, a public pointer, audit/outbox/idempotency tables, legacy-news backfill, reconciliation, or rollback evidence;
- therefore content schema status remains `partial-a3.10` and readiness remains stopped.

## 3. Pinned development evidence

The source baseline is `origin/development` at commit `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`. Each row is verified from the pinned Git blob, so moving the ref, replacing a blob, or deleting an anchor fails integrity.

| ID | Pinned file | Blob | Exact anchor |
|---|---|---|---|
| `source-route-inventory` | `apps/backend/src/web/routes/unified_router.rs` | `46b97779e8726757560d7946d1a47114bed28861` | `.route("/news/featured", get(crate::web::public::news_handlers::list_featured_news))` |
| `source-news-public` | `apps/backend/src/web/public/news_handlers.rs` | `882e4356842318964936d385f6ef70f76b45fd76` | `let limit = query.limit.unwrap_or(10).clamp(1, 100);` |
| `source-news-repository` | `apps/backend/src/infrastructure/repositories/news_repository.rs` | `3a20f29fa878db8ddcaa9d2085c052423c9fbc14` | `.filter(news_articles::status.eq("published"))` |
| `source-news-admin` | `apps/backend/src/web/admin/news_handlers.rs` | `24f589a296247407c0a97caf8fa04ee397eebdbe` | `author_wallet: ctx.wallet_address.to_lowercase(),` |
| `source-news-schema` | `apps/backend/migrations/core/20260227000000_create_news/up.sql` | `b1421b397c7920bf22c733fb8315e6b2dbc13583` | `status VARCHAR(20) NOT NULL DEFAULT 'draft',` |
| `source-public-plans` | `apps/backend/src/web/public/plans_handlers.rs` | `471270ea83821ae5ffd01c95b9bb6f45f850077c` | `.filter(\|plan\| plan.is_public) // Only include public plans` |
| `source-ranking-entitlement` | `apps/backend/src/web/analytics/eps/rankings.rs` | `810461faff9c402631134d9ab8167e764c8d3038` | `rank_offset, // SECURITY: Enforced from user permissions` |
| `source-portfolio-owner` | `apps/backend/src/web/user/unified_user_handlers.rs` | `bb3ce3ff53fbe327d8026eeb1f2c36df3e9f3357` | `let wallet = ctx.wallet_address.to_lowercase();` |
| `source-watchlist-idempotency` | `apps/backend/src/web/user/watchlist_handlers.rs` | `5f31b34a2981c0d0d6146ff6f7899f599a1dc1ba` | `.on_conflict((user_watchlist::wallet_address, user_watchlist::symbol))` |
| `source-news-action` | `apps/frontend/app/actions/news.ts` | `90e68522c7f98836f5b9daf6453d20cfe05438b8` | `return newsClient().listPublished(page, limit);` |
| `source-plans-action` | `apps/frontend/app/actions/plans.ts` | `392234a7b5b109810ae136dc3a37165c2ebcfc3d` | `return plansApi.getPublicPlans(filters);` |
| `source-watchlist-action` | `apps/frontend/app/actions/watchlist.ts` | `eebeb3f5bb3e339d7f594433ed17cca1cf1aee10` | `'/api/users/portfolio/overview'` |
| `source-news-empty-ui` | `apps/frontend/components/news/news-list.tsx` | `99ff164588cede61f6bed263f65e344314dd9acb` | `No articles yet` |
| `source-portfolio-empty-ui` | `apps/frontend/components/portfolio/portfolio-grid.tsx` | `8a020b600a5f6a7ac01a80612b6f68492a59bf10` | `No stocks in watchlist` |

The development branch is evidence, not an unquestioned specification. Its useful contracts include published-news filtering, backend-derived author/portfolio identity, backend-owned ranking offsets, owner-scoped idempotent watchlists, and explicit empty UI. Known legacy defects must be corrected during migration: news handler error envelopes can be returned without authoritative HTTP error status; public plan detail does not visibly re-check `is_public`; several portfolio/watchlist failures degrade to empty success; media can fall back to a local temporary directory; and the news schema has no locale/revision/outbox/idempotency model.

## 4. Current target evidence inventory

The contract pins 32 current-worktree anchors. The exact full anchor strings live in `targetEvidence`; the groups below state what each anchor proves and where it currently resides.

| Current area | Evidence IDs and anchors | Audit finding |
|---|---|---|
| Router/auth | `target-content-router` (`pages/{slug}` CRUD route), `target-auth-boundary` (`EditorIdentityRequired \| Blocked`), `target-gateway-rewrite` (`/api/v1/news` → `/api/v1/content/news`) | Authorization is partial; public and admin route classification exists, editor routes are 404, lifecycle is unproved. |
| Schema boundary | `target-content-schema-boundary` (content findings `4 → 0`) | A3.10 removed startup DDL and added an additive root, but the root has no runner/version ledger, populated-source adoption, backfill, or reconciliation proof and does not model the required lifecycle revisions/outbox/idempotency. |
| Page lifecycle | `target-render-draft-query` (`FROM public.pages WHERE slug = $1`), `target-page-create` (`INSERT INTO public.pages...`), `target-page-status` (`req.status`), `target-publish-direct` (`SET status = 'published'`) | Public render can load a mutable non-published row; CRUD/publish lack canonical validation, optimistic concurrency, immutable revisions, audit/outbox, and invalidation. |
| Theme/block | `target-theme-empty-overwrite` (missing colors → `{}`), `target-block-admin-public` (public select includes `admin_only`), `target-registry-db-sync` (filesystem upsert) | Partial theme updates are destructive; public block projection can expose admin-only definitions; schemas are not lifecycle-versioned. |
| Editor | `target-editor-client-user` (`Uuid::parse_str(&req.user_id)`), `target-auth-boundary` | Handler trusts body identity, so A2.3b correctly keeps the surface unreachable; ownership/expiry/version/replay are absent. |
| Filesystem trust | `target-watcher-enabled`, `target-registry-file-read`, `target-registry-db-sync`, `target-site-file-read` | Mutable local files feed runtime registry/DB/site responses. Reload appends into the existing vectors and does not reconcile deletions or reject a full bad generation atomically. |
| News/plans | `target-news-fallback` (unknown slug returns “coming soon”), `target-plans-file-read` (`marketing/plans.json`) | Unknown news receives a synthesized 200; plans are content files rather than subscription-owned public/active records. |
| Ranking/portfolio | `target-ranking-static` (`GHC` row), `target-portfolio-placeholder` (`auth_required: true`) | Ranking is canned and has no entitlement authority; portfolio is public and address-selected, not verified-owner data. |
| Frontend BFF | `target-frontend-routes`, `target-bff-ranking-static`, `target-bff-plan-static`, `target-bff-news-upstream`, `target-bff-news-not-found`, `target-bff-portfolio-static` | News now uses strict live upstream adapters and preserves upstream not-found, but the content service still synthesizes an unknown-slug article and ranking/plan/portfolio BFF responses remain production-looking canned data. |
| Admin/client | `target-admin-plain-content`, `target-admin-status-allowlist`, `target-client-typed-status` | Some reads omit request context. The client now discards upstream error details and the admin BFF preserves an allowlisted status, but the bare response still lacks the versioned code/message/validation/retry/correlation envelope required by A5. |
| UI | `target-news-ui-error-state`, `target-portfolio-ui-static`, `target-analytics-ui-static` | News now exposes explicit empty/error/retry behavior without sample fallback; portfolio and analytics remain static, and complete state coverage across all lifecycle surfaces is absent. |
| Headers | `target-frontend-security-layer` | A frontend security middleware is installed, but no per-content-route cache/security contract is proven across all hops. |

## 5. Development-to-target contract comparison

| Lifecycle concern | Pinned development behavior | Current target behavior | Required production contract |
|---|---|---|---|
| Published reads | News repository filters `status = published`; ordered by publication time. | Page render selects by slug only; news list reads JSON; unknown detail is synthesized. | Anonymous reads resolve a locale+slug to an immutable published revision; every non-public state is 404. |
| Draft/admin reads | Admin news supports drafts, publish/unpublish, pin/unpin. | Protected page list/get exists, but page status is arbitrary; editor is 404. | Draft projection is admin-only, actor-attributed, versioned, non-cacheable, and never shares a public DTO/cache key. |
| Slug/locale | News slug is unique with suffix generation; no locale. | Page has locale column but lookup/unique key is slug-only; unknown news returns 200. | Normalize slug once; unique `(locale, slug)`; explicit default/fallback/rename/redirect policy; unknown/draft/deleted is HTTP 404. |
| Pagination | News page min 1, limit 1–100; featured limit 1–10. | Frontend news now normalizes a fixed 12-item page from a bounded upstream read, but the canonical cross-hop query/order/featured contract is not frozen. | Freeze query defaults/caps, sort tie-breaker, total/cursor/page fields, malformed-query status, and empty-page behavior. |
| Public plans | DB-backed, list filters `is_public`, Redis TTL 900, tier sort; detail has a visibility gap. | Content JSON and BFF buckets are canned. | Subscription service owns records; both list/detail require `is_public && is_active`; filter/order/promotion/cache semantics are identical. |
| Ranking access | Backend permission service derives rank offset/cap; anonymous uses backend free offset. | Content/BFF/UI contain sample companies. | Only backend permission/plan authority derives access; downstream may render but never widen results. |
| Portfolio/watchlist | Verified OpenID wallet scopes DB watchlist; add is conflict-do-nothing; overview joins rankings. | Arbitrary public address path returns placeholder or canned holdings. | Authenticated subject selects owner; foreign address is not a selector; mutations are validated/idempotent and overview uses live analytics. |
| Page CRUD | No equivalent legacy page lifecycle baseline. | Weak DTO/JSON validation; invalid theme UUID silently falls back; arbitrary status. | Typed schema, canonical identifiers, locale/theme/block/SEO validation, version/ETag precondition, exact 409/422. |
| Theme CRUD | No equivalent legacy theme baseline. | Arbitrary JSON; missing update groups become `{}`; default invariant absent. | Versioned token schema, merge semantics, one transactional default, reference-aware delete, preview compatibility. |
| Block CRUD | No equivalent legacy block lifecycle baseline. | Filesystem manifests upsert; public output includes `admin_only`; no full CRUD. | Versioned schema/defaults, compatibility/migration policy, public-safe projection, validation at page write/publish/render. |
| Editor identity | News author is derived from authenticated context. | Body `user_id` is bound by handler, while auth boundary blocks the route. | Principal extension is the sole actor; session ownership/expiry/version/close/list rules are server enforced. |
| Publish | News publish/unpublish changes status/time but has no complete revision/outbox contract. | Direct status update; commit can separately publish after closing session. | One transaction validates draft/version/media, creates immutable revision, flips public pointer, writes audit+outbox+idempotency, then invalidates cache. |
| Media | Upload to object storage; public route rejects traversal and can redirect to CDN; local fallback exists. | No target upload/reference lifecycle. | Size/MIME sniff/digest/namespace checks, durable metadata, published-reference integrity, content-addressed immutable delivery, safe GC. |
| Filesystem sync | Not the legacy news authority. | Mutable manifests/settings/news/plans are runtime authority; reload accumulates entries. | DB/versioned bundle authority is explicit. Production watch is disabled or signed/path-confined; reload is atomic and reconciles removals. |
| Wire/status | Legacy public/user routes and frontend actions provide prior paths/shapes, with some status/empty bugs. | Gateway rewrites coexist with a live news adapter and canned ranking/plan/portfolio handlers; admin/client behavior is not one frozen cross-hop contract. | One route matrix freezes external/internal prefixes, request body, envelope, pagination, status, request ID, cache/security headers. |
| UI states | News and portfolio have real empty UI; failures can still be degraded upstream. | News now distinguishes empty/error/retry without fake content, while portfolio/analytics remain static and the remaining lifecycle surfaces lack a complete state matrix. | Each data surface implements loading, empty, forbidden, not found, retryable error, retry, and success without fake content. |

## 6. Stop blockers

Readiness stays stopped until all 20 blockers in the contract have implementation and reviewed evidence:

1. `B01` public draft leak / no immutable publication pointer.
2. `B02` locale, slug, fallback, rename, redirect, and not-found semantics undefined.
3. `B03` cache validators/invalidation/negative-cache/security-header contract missing.
4. `B04` page CRUD validation and concurrency missing.
5. `B05` theme schema/merge/default/reference safety missing.
6. `B06` block CRUD/versioning/public projection incomplete.
7. `B07` editor identity is caller supplied; routes correctly remain 404.
8. `B08` editor session ownership, expiry, version, close, and replay unproved.
9. `B09` publish is not a transactional immutable lifecycle.
10. `B10` media metadata/reference validation and safe garbage collection missing.
11. `B11` watcher/sync trust, atomic replacement, and deletion reconciliation missing.
12. `B12` A3.10 removed runtime DDL, but migration runner/adoption, lifecycle revisions, backfill, and reconciliation remain missing.
13. `B13` plan authority/visibility/shape/cache parity missing.
14. `B14` ranking data and entitlements are canned/bypassed.
15. `B15` portfolio/watchlist owner semantics are replaced by an arbitrary public address route.
16. `B16` prefixes, bodies, envelopes, statuses, IDs, and headers diverge across hops.
17. `B17` audit, outbox, and mutation idempotency are missing.
18. `B18` news states improved, but portfolio/analytics still use static data and complete UI state coverage is unproved.
19. `B19` shadow/reconciliation/SLO readiness evidence is missing.
20. `B20` per-batch route/data/cache rollback is unproved.

No blocker may be changed to ready merely because code exists. Closure requires the evidence bundle named in the relevant phase: tests/fixtures, migration and reconciliation reports, security review where applicable, and rollback rehearsal.

## 7. Canonical route batches

Route ownership moves only in these eight batches. A later batch may not be used to smuggle an earlier blocked dependency into production.

| Batch | Routes | Dependencies | Required exit |
|---|---|---|---|
| `public-page-render` | `GET /api/v1/content/pages/{slug}/render` | schema, publication, locale/slug, cache | Published revision only; draft/deleted/unknown 404 matrix; locale/cache/header fixtures. |
| `public-content-discovery` | themes, blocks, navigation, site GETs | schema, public projection, filesystem decision | Only published/public-safe values; no `admin_only`; deterministic cache/not-found behavior. |
| `public-news-media` | list, featured, detail, image/media GETs | publication, media, migration, BFF contract | Published pagination/order; real 404; immutable safe media; one envelope/status/header shape. |
| `public-plans` | list/detail | subscription authority, cache, BFF contract | Active+public list/detail parity, filters/order/promotion validity, invalidation, real empty. |
| `public-rankings` | rankings GET | analytics + permission authority | Live data and backend-derived anonymous/free/paid rank boundaries. |
| `authenticated-portfolio-watchlist` | overview plus watchlist GET/POST/DELETE | verified owner, analytics, idempotency | Owner/foreign-owner tests, symbol validation, idempotent replay, live overview join. |
| `admin-content-lifecycle` | page/theme/block/news/media CRUD and publish | migrations, typed validation, actor, audit/outbox | Exact 401/403/404/409/422; optimistic concurrency; transactional publication and media integrity. |
| `editor-sessions` | start/commit/list | admin CRUD + publication + server actor | Keep 404 until ownership/version/expiry/replay/audit evidence is reviewed. |

Recommended cutover sequence after all implementation gates are green: public page render → public discovery → public news/media → public plans → public rankings → authenticated portfolio/watchlist → admin lifecycle → editor sessions. Each switch is independent and is followed by an observation window before the next batch.

## 8. Execution phases and agent work packets

The dependency order is mandatory. Independent implementation can run in parallel only after its declared prerequisite is merged and its contract fixture is frozen.

### Phase 1 — freeze wire and ownership

Agent packet: contract steward.

- Produce one route matrix with external prefix, internal service route, auth class, request/query/body schema, success body, error envelope, pagination, status codes, request-ID behavior, owner, cache, and security headers.
- Decide source of truth: content DB for page/theme/block/news/publication; subscription for plans; permission+analytics for rankings; verified user/watchlist plus analytics for portfolio.
- Preserve `admin:content:manage`; define any narrower permissions without frontend plan logic.
- Capture compatibility corrections instead of copying legacy bugs.
- Exit: reviewed decision records and cross-team fixture schemas.

### Phase 2 — schema and migrations

Agent packet: data model/migration owner.

- Build forward-only lifecycle migrations on the A3.10 fresh-schema boundary; retain zero content runtime DDL.
- Minimum durable model: content identity `(kind, locale, slug)`, mutable draft/version, immutable revision, public pointer, theme versions/default invariant, block type/version, media object/reference, editor session, mutation idempotency, audit, outbox.
- Add state constraints, foreign keys, unique keys, indexes, timestamps, actor/request fields, and retention rules.
- Never drop legacy data to simplify the move; use additive/rename/backfill transitions with `IF EXISTS`/`IF NOT EXISTS` guards where appropriate.
- Exit: reviewed runner/version ledger, populated-source adoption or forward-repair evidence, schema tests, migration lint, and continued zero runtime DDL.

### Phase 3 — backfill and reconciliation

Agent packet: migration/reconciliation owner.

- Inventory legacy news, current content files, page/theme/block registries, plan identifiers, and media references.
- Backfill idempotently with deterministic IDs/hashes; quarantine invalid/duplicate/ambiguous rows rather than silently choosing.
- Generate dry-run and applied reports: input/output count, published/draft count, locale/slug collisions, revision hash, missing theme/block/media, orphan media, invalid plan reference, rerun changes.
- Reconciliation must be repeatable and explain every mismatch.
- Exit: reviewed zero-unexplained-mismatch report and successful no-op rerun.

### Phase 4 — validated admin CRUD

Agent packets may split by page, theme, block, news/media after phase 2.

- Page: canonical slug/locale/title; validated SEO/theme/block props; permitted state; max sizes; `If-Match`/version; 409 stale write; 422 validation.
- Theme: versioned typed tokens; patch/merge semantics; exactly one default; reference-aware archival; preview compatibility.
- Block: versioned JSON schema/defaults; compatibility policy; public/admin projection; validation during draft write and publish.
- News/media: equivalent validation, author from principal, safe upload metadata/reference, pin/order rules.
- Every mutation accepts an owner-scoped idempotency key and writes actor/request audit.
- Exit: positive and negative service tests plus gateway/admin-BFF fixture coverage.

### Phase 5 — transactional publication

Agent packet: lifecycle owner.

- Define transitions for draft, scheduled, published, superseded, unpublished, archived, deleted, and failed publication.
- Compare-and-set the expected draft version.
- In one DB transaction: validate draft/block/theme/media references; create immutable revision; move public pointer; record before/after audit; insert outbox; persist idempotency response.
- Dispatch cache invalidation after commit through retryable outbox; duplicate dispatch is harmless.
- Same idempotency key and request hash returns original result; same key/different hash returns 409.
- Exit: transaction rollback/fault injection, concurrency, replay, unpublish/republish, outbox retry, and cache invalidation tests.

### Phase 6 — public content reads

Agent packet: content query/cache owner.

- Query only public pointer + immutable revision; never the draft table.
- Apply locale and slug rules before lookup. Do not reveal whether a non-public row exists.
- Public list empty is a successful empty collection; detail unknown/draft/deleted is HTTP 404.
- Proposed default policy to freeze in phase 1: public HTML/JSON `public, max-age=0, s-maxage=300, stale-while-revalidate=30` with revision ETag; anonymous negative 404 cache at most 30 seconds; content-addressed media `public, max-age=31536000, immutable`; admin/private `no-store`.
- Add `Vary` only for actual representation dimensions. Never vary public content on forwarded credentials; strip authorization before public upstream.
- Exit: service/gateway/BFF fixtures for locale, status, pagination, conditional GET/304, invalidation, negative cache, credential stripping, and security headers.

### Phase 7 — backend-owned plans, rankings, and portfolio

Agent packets: subscription plan contract; analytics entitlement; portfolio/watchlist ownership.

- Plans: subscription backend returns public+active list/detail, filters and tier order, promotion validity, and invalidation. Content/UI only render its DTO.
- Rankings: permission/plan backend derives offset/cap; analytics enforces it. Ignore/reject downstream attempts to request inaccessible ranks.
- Portfolio: authenticated subject selects the owner; remove public arbitrary-address semantics. Watchlist symbol normalization/length, owner uniqueness, add replay, delete ownership, and overview join are backend operations.
- Exit: anonymous/free/paid/admin/expired-plan and owner/foreign-owner fixtures; no canned response or content-file authority remains.

### Phase 8 — gateway and BFF contract

Agent packet: wire integration owner.

- Remove competing canned handlers only when live service fixtures pass.
- Preserve canonical body and authoritative HTTP status through gateway, `ServiceClient`, frontend BFF, and admin BFF.
- Required status matrix: 200/201 success, 204 only for truly empty mutation responses, 304 conditional public read, 400 malformed syntax, 401 absent/invalid authentication, 403 authenticated but insufficient permission, 404 unknown/non-public/foreign-owned, 409 stale version or idempotency mismatch, 422 semantic validation, 429 throttled, upstream 5xx preserved/mapped by explicit contract.
- Preserve request ID; return safe structured error code/message/details; never turn an envelope error into HTTP 200 or collapse every 4xx into 400.
- Exit: table-driven cross-hop tests for every route/status/header, including prefix and trailing/dynamic-segment rejection.

### Phase 9 — UI state machines

Agent packet: Dioxus UX owner.

- News, plans, rankings, portfolio, and every admin/editor form distinguish: initial loading/skeleton, real empty, forbidden, not found, retryable error, submitting, conflict, validation error, success.
- Remove production-looking defaults when JSON is absent/malformed or upstream fails. Test data is injected only by tests/story fixtures.
- Retry is keyboard/screen-reader accessible, bounded, and preserves filters, pagination, form draft, and request correlation.
- 409 shows refresh/merge/retry guidance; 422 binds field errors; 401/403 use their dedicated surfaces; 404 detail does not render a synthesized article.
- Exit: component/SSR/browser tests across all states plus visual/keyboard/accessibility review.

### Phase 10 — editor sessions

Agent packet: editor security owner.

- Derive actor from verified principal extension; reject body/header identity.
- Define page ownership/collaboration, session version, lease expiry/renewal, exclusive/shared edit policy, commit message, close/cancel, replay, and list visibility.
- Commit verifies session actor, page, expected version, expiry, and open state. Publishing delegates to the phase-5 transaction.
- Only after all fixtures pass may A2.3b classification change from fail-closed 404.
- Exit: spoof/foreign/expired/stale/double-commit/list-leak tests plus audit/outbox evidence.

### Phase 11 — shadow and observe

Agent packet: readiness/reconciliation owner.

- Shadow pinned-contract reads without serving target output; compare normalized status, envelope, IDs, ordering, pagination, public visibility, entitlement boundaries, and content hashes.
- Measure lifecycle transition failures, stale cache, negative-cache persistence, outbox age/retry, reconciliation mismatch, media missing, 4xx/5xx, p95/p99 latency, and UI fallback/error rates.
- Redact draft bodies, tokens, private portfolio data, and media secrets from logs/traces.
- Exit: reviewed thresholds and sustained observation report with zero security mismatch and no unexplained data mismatch.

### Phase 12 — batch cutover and rollback rehearsal

Agent packet: release/rollback owner. Deployment still requires a separate explicit user instruction.

- Switch only one route batch at a time and record previous/current ownership.
- Run smoke, parity, cache, security, and SLO checks; hold an observation window.
- Rehearse rollback before proceeding: revert route ownership, purge incompatible caches/negative caches, pause new consumers, continue/outbox-drain safely, and retain all migrated rows/revisions/audit/outbox data.
- Never roll back by deleting new rows or reversing published history. Prefer forward repair for schema/data.
- Exit: signed batch checklist, rollback duration, restored health/parity evidence, and go/no-go decision.

## 9. Publication, identity, and idempotency invariants

These invariants apply to every implementation agent:

1. Public identity is `(content kind, locale, canonical slug)`; a mutable draft is never the public record.
2. A published revision is immutable. Editing a published page creates a new draft/version.
3. The public pointer changes only inside the publication transaction.
4. Actor/owner is server derived from a verified principal; client identity fields are ignored or rejected.
5. Authorization and entitlement decisions live in Rust backend services, never Dioxus/frontend/admin UI.
6. Every write has expected version plus owner-scoped idempotency key and canonical request hash.
7. Audit and outbox commit with the state change; event consumers are at-least-once safe.
8. Cache invalidation references content identity and old/new revisions; rename invalidates both slugs and locale variants.
9. Media reference checks happen before publication; published media is not garbage-collected.
10. Unknown, draft, deleted, archived, scheduled, and foreign-owned resources do not leak existence through status/body/timing differences beyond reviewed policy.

## 10. Required test matrix

At minimum, each relevant route batch must cover:

- anonymous, malformed bearer, valid frontend, admin without permission, admin with exact permission, and invalid wildcard;
- locale default, explicit locale, unsupported locale, canonical/noncanonical slug, Unicode/length limits, duplicate locale+slug, renamed slug;
- draft, scheduled-before-time, scheduled-after-time, published, superseded, unpublished, archived, deleted, missing;
- empty list, first/middle/last/out-of-range page, maximum limit, invalid limit, deterministic tie ordering;
- valid/stale/missing version, simultaneous update/publish, publish validation rollback;
- first idempotent request, identical replay, different-body replay, concurrent same-key request;
- valid/missing/wrong MIME/oversize/hash-mismatch/orphan/in-use media;
- watcher disabled, invalid bundle, path escape/symlink, duplicate generation, deletion, partial write, restart reconciliation;
- anonymous/free/paid/expired/admin ranking access and requested-window widening attempts;
- owner/foreign-owner portfolio/watchlist and normalized/invalid/duplicate/missing symbol;
- upstream 401/403/404/409/422/429/500/timeout/malformed JSON at every hop;
- cache miss/hit/304/publish/unpublish/rename/delete/negative-cache expiry; credential and locale variance;
- UI loading, real empty, each terminal error, retry, conflict, validation, success, offline/timeout, and no-JS SSR behavior;
- rollback before/after publication, pending outbox, warm positive/negative caches, and schema-forward compatibility.

## 11. Readiness and rollback gates

Readiness requires all of the following; integrity alone is insufficient:

- all 20 blockers have reviewed evidence and no unresolved stop finding;
- eight route batches have green service, gateway, BFF, UI, security, cache, and rollback fixtures;
- versioned migrations and repeatable backfill/reconciliation are reviewed;
- A2.3b auth gates remain green, and editor routes stay 404 until their dedicated gate closes;
- canned/file-backed production authority is removed from served paths;
- shadow mismatch is zero for visibility/ownership/entitlement/security and within a reviewed tolerance for normalized presentation fields;
- outbox/reconciliation/cache/SLO observation meets phase-11 thresholds;
- each route batch rollback has been rehearsed without deleting durable data;
- production deployment receives a separate explicit user instruction.

Automatic rollback triggers must be frozen before cutover. They include any draft/private/foreign-owner exposure, entitlement widening, unexplained reconciliation mismatch, incorrect status/envelope, persistent stale publication, media integrity failure, outbox backlog over threshold, route error/latency threshold breach, or UI showing canned success during upstream failure.

## 12. Deterministic verification

Run from repository root with database/Redis/object-store/chain variables unset and no production-looking environment selector:

```bash
./scripts/migration/verify-content-lifecycle.sh --mode integrity
./scripts/migration/verify-content-lifecycle.sh --mode report
./scripts/migration/test-content-lifecycle.sh
```

Expected results:

- integrity exits `0` only when the pinned ref/commit, 14 blobs/anchors, 32 current anchors, partial A2.3b authorization plus partial A3.10 schema truth, eight batches, 16 lifecycle requirements, 20 blocked stop findings, rules, and 12-phase order are intact;
- report exits `0` and emits deterministic JSON;
- readiness exits `3` while the contract remains intentionally non-production;
- self-test proves deterministic reports and detects missing source/target anchors, stale source commit/blob, unsafe evidence path, readiness-sentinel tampering, and blocker-state tampering;
- none of these results contacts or mutates a database, Redis, object storage, chain, deployment, runtime, `Cargo.lock`, existing fixture, or production filesystem.

Before an implementation batch is handed off, also keep repository-wide route inventory, contract fixtures, and service-authorization gates green. A10.0 itself does not weaken those gates and does not authorize deployment.
