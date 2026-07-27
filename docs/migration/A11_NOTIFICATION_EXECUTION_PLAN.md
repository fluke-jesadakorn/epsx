# A11.0 notification lifecycle and delivery execution gate

Continuation note: the active branch has completed and verified a bounded
owner-read pagination and filter slice. SSR accepts only canonical positive
page values plus the explicit `all`/`read`/`unread` status and bounded source
type/priority filters, derives a fixed 20-row offset, validates page
cardinality, renders native filter links and previous/next recovery, and reports invalid queries as 400 without sample
fallback. This improves truthful read parity only; the A11 lifecycle STOP
blockers below remain unchanged.

Status: **audit/design only; production readiness is STOP**. Direct notification-service authentication from A2.3c is a verified prerequisite, but remains **partial** for the lifecycle as a whole. This document does not authorize deployment, production access, database access, Redis access, SMTP/provider access, external network access, or migration execution.

The deterministic authority is [`contracts/notification-execution.json`](contracts/notification-execution.json). It pins 14 source evidence records from `origin/development` commit `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`, 88 current-target anchors, 12 lifecycle/API surfaces, 22 STOP blockers, and eight ordered execution batches.

The N1 authority decision is now recorded in
[`contracts/notification-authority-matrix.json`](contracts/notification-authority-matrix.json):
the target uses a versioned singular service path behind plural frontend/admin adapters, with
owner derivation, permission checks, state, and delivery admission owned by Rust. The matrix is
validated by `cargo xtask authority-audit --strict`; it is a static contract and does not claim
source parity or production readiness.

## Outcome and boundary

The target is not notification-production-ready. It has a useful authenticated owner/admin CRUD shell, but the following remain separate production concerns:

- durable event intake from payment, subscription, permission, chat, expiry, and admin publishers;
- canonical recipient resolution and source-compatible user/admin APIs;
- independent read, acknowledgement, delivery, click, expiry, and failure state machines;
- owner preferences, quiet hours, push subscriptions, SSE replay, and reconnect deduplication;
- asynchronous email and in-app workers with idempotency, retries, dead letters, and reconciliation;
- strict/versioned templates, content safety, privacy-safe logging, retention, and erasure;
- complete additive lifecycle migrations, legacy backfill, one write authority, observability, deployment, cutover, and rollback.

Integrity success proves only that this audit has not drifted. It does not contact or validate a database, Redis, SMTP, push provider, Kubernetes cluster, internal service, or production environment.

## Pinned source behavior to preserve or explicitly version

The development source exposes `/api/notifications` owner operations, `/api/admin/notifications` management, an authenticated SSE stream, Redis wallet/broadcast fanout, durable offline replay, notification preferences, browser-push client operations, and a `wallet_notifications` write model. These are compatibility evidence, not a claim that every legacy implementation detail is ideal.

| Source surface | Required compatibility decision |
|---|---|
| Owner list and unread count | Preserve filters, pagination, broadcast inclusion, expiry/read semantics, field names, envelope, and errors or publish a versioned adapter. |
| Read, unread, acknowledge, delete, mark-all, clear-all | Lock methods, retry behavior, affected counts, foreign-ID `404`, and distinct state semantics. |
| Preferences | Preserve owner-scoped channel/type/priority/quiet-hour behavior and move enforcement into the Rust backend. |
| SSE | Bind the verified wallet to the stream; replay durable owner+broadcast records; keepalive, reconnect, dedupe, and acknowledgement must be explicit. |
| Browser push | Implement real permission/subscription/status/unsubscribe behavior; a local UI signal is not permission or delivery. |
| Admin send | Resolve exactly one of wallet, plan, or broadcast; enforce bounded fanout, expiry, scheduling, actor audit, and permission. |
| Admin history/stats | Preserve global filters, delivery/read statistics, acknowledgement, and delete semantics under operation-specific authority. |
| Durable state | Reconcile legacy `wallet_notifications`, preferences, templates, and event identities into one declared target model. |

## Current target findings

### Authentication is partial, not lifecycle proof

A2.3c establishes a narrow service boundary:

- `/health` and the read-only `/ready` probe are the only anonymous notification-service surfaces;
- owner routes accept only verified frontend/admin audiences and derive the owner wallet from the principal;
- template/send routes require the admin audience and `admin:notifications:manage`;
- unknown and method-drift service paths fail closed.

It does not establish internal publisher identity, message delivery, preference enforcement, live schema/adoption correctness, migration-history safety, idempotency, replay, observability, deployment, or cutover readiness. Every A11 surface therefore remains blocked.

### Storage and state drift

A3.11 removes the four startup DDL statements and both startup seed paths. It adds a guarded fresh-schema migration for `public.templates` and `public.notifications`, public-qualifies all 21 application relation references, and makes startup reject any schema that differs from the exact 26-column, three-key, five-index contract before template cache load or listener binding. Template query/registration failures also stop startup instead of being discarded.

That is static boundary evidence, not migration readiness. The existing notification migration history still has a renamed/consolidated baseline adoption ambiguity and a destructive `DROP TABLE ... CASCADE` migration. The explicit legacy mapper and dry-run fixtures now cover field-shape/status/wallet normalization without writing target rows, but no populated database backfill, ledger adoption, or live reconciliation was executed. `CREATE TABLE IF NOT EXISTS` cannot repair an incompatible table created by the former runtime DDL, so both the schema and migration A11 blockers remain STOP.

The startup schema probe now models the additive lifecycle contract explicitly: it accepts only the
three reviewed base-table checks and eight validated, immediate `ON DELETE RESTRICT` foreign keys
introduced by the lifecycle migrations, while rejecting any extra or altered constraint. The local
runtime audit exercises this probe against a fresh migrated database.

The legacy model is `wallet_notifications`, with different field names and richer channel/state fields. The source baseline has no declared single write authority or mapping; the target now records an explicit, conservative dry-run mapping, and a disposable local PostgreSQL audit proves four populated legacy rows map into four target rows with checksum, status, broadcast, and provider-identity reconciliation. Staging-scale/live backfill, rollback, and one-writer runtime proofs remain absent. The extracted owner read route now changes only `read_at`, the owner `PUT .../{id}/acknowledge` route records a separate engagement receipt timestamp, and owner `POST .../{id}/click`/`dismiss` routes persist first-event timestamps independently; the Rust frontend BFF now forwards click/dismiss with only the verified bearer and request ID, rejects cookie-backed cross-origin mutations, and maps legacy `PUT` read/unread/mark-all plus `DELETE` clear-all aliases to the Rust-owned service mutations. Provider acceptance, in-app persistence, client delivery, and user engagement remain distinct facts. The Dioxus notifications page now exposes bounded row and bulk lifecycle controls through a closed Rust-served action map with explicit save/error/reload states; authenticated owner-route browser smoke is now proven locally, while source parity and live integration proof remain absent.

### Delivery drift

- `in_app` reports `sent=true` without Redis/SSE fanout or client receipt.
- Missing SMTP configuration now fails closed without logging message content or recipient data.
- Configured SMTP uses a TLS-wrapper transport on port 465 or required STARTTLS on other ports,
  and the blocking transport call is isolated from async request tasks.
- Persistence now precedes delivery through a transactional outbox and channel-job enqueue; operator redrive is guarded by the same durable expiry projection and cannot requeue an expired notification.
- Publisher intake now records inbox/idempotency/outbox state transactionally, and worker primitives include leases, retry classification, dead letters, an active asynchronous loop, and an admin-authorized redrive transition. Provider callbacks now require a timestamped raw-body HMAC, persist deduplicated events, reconcile linked channel-job and notification states without overriding terminal, accepted, or dead-lettered jobs, and resolve callbacks by either the internal job ID or the persisted provider message ID; callback event time is retained for the notification sent timestamp. Secret rotation, external delivery proof, and recovery drills remain unproven.
- The legacy backend notification pool factory and admin/user handlers now fail closed when
  `NOTIFICATIONS_DATABASE_URL` is missing or unavailable; they no longer fall back to the primary
  database schema.
- The canonical backend still defaults to an in-process notification port. An opt-in HTTP adapter
  now exists behind `NOTIFICATION_ADAPTER=remote`, but publication errors may still be logged and
  swallowed only for post-commit best-effort fanout, and production/explicit-remote startup now
  fails closed when the notification adapter cannot be built; non-production harnesses may still
  omit the adapter for isolated tests.

### Truthful owner read slice; lifecycle remains blocked

The extracted service now has owner-scoped preferences, an SSE polling/replay route with durable cursors,
`Last-Event-ID`, acknowledgement, bounded connections, keepalive, broadcast-inclusive queries, and
typed push status/subscribe/revoke routes. Owner-bound sends now enforce disabled channels before
enqueue and calculate timezone-aware quiet-hour release times in PostgreSQL; requests without an
owner identity remain ungoverned administrative delivery. Redis fanout, browser permission/provider
delivery, and live database/reconnect proof remain absent.

The owner page now has a narrow truthful read path: authenticated SSR records exact `ok` or `error` dependency state, admits only the canonical `all`/`read`/`unread` status, all ten bounded source notification types, five priorities, RFC3339 `start_date`/`end_date`, and bounded page/offset derivation, preserves those filters through native pagination, and the UI requires the current service's nullable keys, parses timestamps as UTC datetimes, treats missing/malformed/error payloads as unavailable instead of plausible empty data, uses a neutral title fallback, omits null/blank type or priority labels, and preserves present raw type/priority text while static presentation-only icon/chip maps decorate exact known tokens. Deterministic emitted-CSS tests prove the priority chips, loaded-list count, and timestamps meet the small-text `>= 4.5:1` threshold and the filled unread dot exceeds the non-text `>= 3:1` threshold across the declared app-page gradient, radial overlay, glass-card, read/unread, and hover surface envelope. Static SSR and emitted-CSS tests additionally prove that complete escaped hostile title/body head and tail remain in their semantic `h3`/`p` and normal wrapping flow without ellipsis, nowrap, line clamps, display-box clipping, or hidden overflow. The list BFF now caps both the whole response at 2 MiB across up to 100 rows and each identity, recipient, text, data, and action field; hostile, oversized, or owner-mismatched rows fail closed. This remains full-DOM/static-CSS evidence only, so browser/mobile and whole-page AA remain unproven. Deterministic boundary and row tests also prove that every `created_at` later than the shared server render instant receives a visible absolute UTC label including seconds while its canonical `datetime` and exact UTC title remain unchanged. The Dioxus rows now add bounded mark-read/unread, acknowledge, dismiss, and remove controls plus mark-all/remove-all controls; the route-scoped Rust-served controller uses a closed action map, same-origin credentials, exact success envelopes, and truthful error/reload states. Pinned `?id=` highlight, scroll and source parity remain deferred, and provider delivery, action URLs, browser/runtime, and whole-page AA remain unproven. The hollow/read dot is an unclaimed redundant cue and separators remain decorative and `aria-hidden`; no frontend-only `notifications:read` token or sample fallback is used.

The authenticated `/account` page now consumes the same bounded owner preferences response during SSR. It renders validated email, in-app, push, quiet-hour, timezone, and optional save-time values in a native Dioxus form. The form posts to a same-origin Rust adapter which requires a verified bearer, `Origin`/`Host` alignment, an optional same-site fetch marker, an exact bounded URL-encoded field set, and the existing JSON validator before forwarding the canonical owner PUT; successful and failed writes redirect to explicit SSR status states. The redirect also sets a short-lived HttpOnly flash cookie scoped to `/account`; SSR accepts a status query only when that cookie carries the matching adapter-issued value, then clears it, so a manually added query cannot claim a successful write. Signed-out, unavailable, and malformed states remain distinct. The authenticated account now also renders a native browser-push control whose account-only runtime validates the exact status envelope, requests permission only after a click, and forwards bounded subscribe/revoke requests through the Rust BFF. This is local SSR/BFF evidence only: disabled VAPID, unsupported browsers, provider delivery, exception policy, worker/reconnect integration, and live settings execution remain unproven.

The frontend BFF read boundary is also narrower and fail-closed. `/api/v1/notifications` is GET-only with an explicit `HEAD` override to `405`; every other non-GET method is also rejected. It accepts only unique `status`, `limit`, and `offset` fields, bounds `limit` to `1..=100` and `offset` to `0..=1_000_000`, and rejects unknown fields, duplicates, and identity-bearing query parameters. It forwards only the verified bearer, streams the upstream response under a 2 MiB cap even when `Content-Length` is absent, parses the same bytes into the exact list DTO and passthrough JSON without cloning, applies per-field identity/recipient/text/data/action bounds, and rejects every row whose `user_id` does not match the wallet from the verified session principal. `/api/v1/notifications/unread-count` has the same GET-only plus explicit-`HEAD`-`405` policy, caps streamed bodies at 4 KiB, and accepts only the exact non-negative `{ "count": i64 }` target DTO. Oversized, malformed, unsafe, and owner-mismatched responses fail as `502`; these are target-hardening facts, not proof of source method, query, envelope, broadcast, expiry, or read-semantics parity.

The former dormant string-renderer badge no longer fetches `limit=1` or derives a global count from one list page; it remains explicitly unavailable. The active SSR shell instead mounts the shared `epsx_templates::epsx_header()` with one inert notification target whose badge starts empty, hidden, `aria-hidden`, and `data-state="unavailable"`. A server-verified authenticated session is the only condition that injects its route-scoped browser controller; signed-out responses inject no runtime or fetch path, and `/offline` excludes it even when a request carries a valid session so the public recovery shell stays free of owner activity. Authenticated non-offline SSR responses and every list/unread BFF outcome, including errors, carry `Cache-Control: private, no-store`; the browser fetch also requests `cache: no-store`.

That controller performs one read-only credentialed `GET` to the exact `/api/v1/notifications/unread-count` BFF route. It accepts only a plain object with the sole key `count` and a non-negative safe integer, resets to unavailable before each request, ignores superseded responses through a monotonic generation guard, aborts on visibility changes, hides zero/error/malformed results, caps only the visible text at `99+`, and preserves the exact count in the link's accessible label. The darker `#dc2626` badge gives its small white text AA contrast; payload text reaches the badge only through `textContent`, with no notification mutation, `innerHTML`, adjacent-HTML insertion, or document write. The separate owner mutation controller uses its own closed action map and never treats a mutation response as provider delivery. Local authenticated and responsive browser smokes now prove owner route authentication, native settings controls, and mobile layout; source-compatible pagination/filter/envelope/expiry/read semantics, source parity, provider push delivery, and owner-facing action-link policy remain blocked, and all 22 A11 STOPs stay open.

Admin notification management now has a separate narrow global read slice rather than reusing the owner feed. Gateway and direct notification service admit only exact `GET /api/v1/notification/admin/list` for the admin audience with canonical `admin:notifications:manage`. The service accepts bounded unique `limit`/`offset` plus bounded status, type, priority, and canonical-wallet filters, orders by `created_at DESC, id DESC`, reads count and rows through one read-only repeatable-read transaction, rejects impossible page cardinality or whitespace-only IDs, fails closed on query/stored-field drift, and selects only `id`, title/subject, channel/status/type/priority, and sent/created timestamps. The admin adapter validates and forwards the same filters with only the verified bearer plus request ID, fixes pages at 20 rows, caps declared and chunked responses at 256 KiB, rejects unknown or impossible payloads, and records ready, authoritative-empty, recoverable out-of-range, forbidden, unavailable, or malformed SSR state. It now also loads the existing admin metrics endpoint through a separate 32 KiB strict projection, bounding counters and channel names before SSR. The shared UI deliberately hides IDs and emits no recipient/user/template identity, body/data/error/read/action fields, or unsupported mutation controls; it renders the bounded operational snapshot with explicit unavailable/malformed states and no individual-delivery claim, plus a canonical-wallet/in-app compose form whose same-origin Rust POST adapter validates fields, requires a verified admin session, forwards the service's durable enqueue request, and reports only cookie-paired queued/error feedback. Broadcast, plan targeting, scheduling, images, arbitrary data, and provider delivery claims remain absent. This is static/unit-test and local BFF evidence only: isolated database/service and authenticated browser compose execution are absent, and source-compatible global stats, acknowledgement, deletion, scheduling, broadcast, templates, preferences, realtime, idempotency, durability, and recovery remain blocked.

### Template, privacy, operations, and deployment drift

Handlebars now runs in strict mode for loaded templates; new template writes are bounded, compiled before commit, versioned, soft-deleted, and invalidated from the in-memory registry. Raw-output expressions and the same parser-backed Ammonia/html5ever allowlist are enforced for both new and startup-loaded templates, so stored markup drift fails startup closed. The admin boundary now exposes typed preview and version rollback routes that validate render data/content before rendering or restoring, and template create/update/delete/rollback mutations write durable actor/version audit records. Publisher action URLs and legacy admin-send action/image URLs are bounded and reject unsafe schemes, credentials, control characters, and host escapes; template validation also rejects unsafe markup, generic event handlers, metadata/srcdoc tags, unsafe image and link URL patterns, tags/attributes outside an explicit allowlist, and unbalanced or mismatched element nesting. The integrity verifier now scans notification SSE/queue/admin plus extracted service/worker tracing lines for direct wallet, recipient, email, subject, body, message, payload, token, and title interpolation. A strict no-write privacy policy contract now records per-channel retention, legal holds, and erasure semantics; runtime purge/erasure, rollback replay/read evidence, and privacy-safe provider delivery remain absent.

Generic tracing plus `GET /ready` now exposes a read-only database/lifecycle compatibility readiness decision, and an admin-authorized read-only metrics snapshot exposes queue age/depth, channel outcomes, dead letters, active streams, durable preference suppression, provider-event/attempt counts, replay-cursor age, and process-local SSE connection/reconnect/replay/average-lag/query-failure counters. Provider-event detail, durable per-connection SSE lag, staging alerts/dashboards, and reconciliation dashboards remain absent. A standalone Rust image, ClusterIP service, managed-secret wiring, resource limits, and `/health`/`/ready` probes are now checked into the Kubernetes base and dev/staging/prod overlays. The dev overlay explicitly resets `EPSX_ENV=development` so its local identity verifier does not inherit the production-only HTTPS policy; staging and prod retain production verifier mode. Live image provenance, dependency readiness, worker topology, network policy, rollback manifests, and cluster execution remain unproven.

## Locked state and delivery semantics

The implementation must model these as independent facts:

1. **Notification state**: created, cancelled, expired, deleted according to explicit guarded transitions.
2. **Channel job state**: queued, leased, attempting, retry-wait, provider-accepted, terminal-failed, dead-lettered.
3. **Realtime state**: fanout attempted, stream event emitted, replayed, and client-acknowledged.
4. **Engagement state**: unread/read, clicked, dismissed; delivery cannot overwrite engagement.
5. **Provider state**: provider message ID and provider-specific accepted/delivered/bounced/complained events.

An API response after durable enqueue is `202 Accepted`; it must not claim `sent` or `delivered`. Provider acceptance is not end-user delivery. An in-app row is durable before best-effort fanout. Foreign owner-scoped IDs return a uniform `404`. Conflicting idempotency reuse returns `409`. Validation uses one documented `400` or `422` policy.

Publisher idempotency is scoped to `(verified service principal, event type, idempotency key)` and stores the request hash plus stable original response. A delivery job is unique by `(source event identity, recipient, template/version, channel)`. Worker claims use leases and compare-and-set transitions; provider message IDs are persisted for reconciliation.

## Required execution order

### N1 — Compatibility and authority

Lock all 12 surfaces with method/path/body/envelope/status/error fixtures. Choose a versioned API or a compatibility adapter. Publish an authority matrix for owner, admin-read, admin-manage, template, delivery operator, dead-letter redrive, and internal publisher operations. Declare the one write authority.

The executable `cargo xtask notification-compatibility-audit --strict` now
checks every reviewed BFF and service fixture route against the checked-in Rust
frontend/admin/service router registrations. This catches method/path drift in
the target source; it intentionally does not claim payload, envelope, status,
legacy-source, live-service, or browser parity.

The Rust frontend now also exposes an explicit source compatibility adapter at
`/api/notifications`. It maps the pinned development list/count/preferences
envelopes and owner mutations onto the canonical target service, round-trips
target `0x` notification identities as source UUIDs, binds the optional legacy
`wallet_address` selector to the verified owner, and persists the source
`types`/`priority_filter` policy metadata inside the backend-owned preference
document. Unsupported SMS enablement still fails closed. Source push status,
nested-key subscription, and owner-wide unsubscribe aliases now map to the
typed target push contract with the source-specific status, active-subscription,
and action-result envelopes, including source bulk `updated_count` and
`deleted_count` results. The source SSE aliases now translate bounded
UUID-compatible target frames, owner-bind the wallet, map source reconnect
cursors, and fail closed on unsupported source filters or non-UUID target IDs;
owner list/get projections now preserve the target expiry timestamp and the
owner-scoped first-click timestamp through the legacy `expires_at` and
`clicked_at` fields, while still keeping delivery and engagement state
separate. Target `sent_at` remains provider acceptance and is intentionally
not projected as the source `delivered_at` fact;
when a target data object carries an `image_url`, the adapter now projects it
only after the same bounded relative/HTTPS validation used for notification
actions;
complete broadcast/plan-ID parity, legacy lifecycle status semantics, and
live/staging integration remain separate gates.

The guarded `scripts/migration/test-notification-compatibility-local.sh
--allow-local` audit now exercises the live extracted service and both Rust/Dioxus
BFFs with short-lived local bearers. It validates the owner list, unread-count,
and preferences envelopes, a foreign-owner mutation `404`, and the permissioned
admin inventory projection. This strengthens local wire evidence but does not
replace pinned development payload parity, provider, staging, or production
integration evidence.

The executable `cargo xtask notification-producer-audit --strict` additionally
checks the migrated payment, permission, chat, and subscription-expiry producer
call sites for the bounded retrying `NotificationPort` event-identity methods and
rejects legacy `NotificationService` calls in those sources. Service-identity
provisioning, producer crash recovery, and remote/staging replay remain separate gates.

Exit: compatibility fixtures and the authority matrix are reviewed; gateway, BFF, service, and UI paths agree; direct auth remains a prerequisite rather than a lifecycle proxy.

### N2 — Durable schema and migration

Extend the narrow A3.11 fresh-schema migration with versioned, additive migrations for template versions, preferences, publisher inbox, request idempotency, per-channel jobs, attempts, dead letters, provider events, and replay cursors. Add constraints for normalized wallets, allowed channels/types/priorities/states, timestamps, unique event/job identities, and guarded transitions. Repair/adopt the existing migration history and prove both clean and populated upgrade paths while keeping runtime DDL and startup samples absent.

The additive `20260723120000_add_notification_lifecycle_foundation`, ordered
`20260723130000_add_notification_idempotency_provider_events`, guarded
`20260723140000_add_notification_lifecycle_constraints`, and
`20260724120000_add_notification_template_audit` and
`20260724130000_add_notification_engagement_acknowledged` and
`20260724140000_add_notification_expirations` migrations now provide the fourteen
relation foundation plus database-level identity/channel/JSON-shape checks, a distinct client
receipt timestamp, and an additive expiry projection. The notification
crate exposes tested Rust transition types for record, inbox, outbox, and channel-job states. The
startup probe intentionally fails closed until all fourteen lifecycle tables exist. A guarded
loopback-only audit now executes the ordered chain on a disposable scratch database, inserts a
populated legacy row before the additive phase, verifies that row survives, checks the fourteen
lifecycle tables and constraints, and restores a `pg_dump` copy with matching schema and
row-count checksums. This is non-production evidence only; production adoption and source
reconciliation remain future work.

The checked-in `notification-migration-ledger.json` and `cargo xtask notification-migration-audit
--strict` now verify the seven migration directories, ordering, file presence, non-destructive
static SQL, up/down SHA-256 checksums, and the recorded local-scratch evidence report. The strict
audit is still not a production approval: the guarded legacy dry-run now validates and counts
bounded projection of wallet, title/body, type/priority, channels, action/data, and RFC3339
expiry/creation fields into the target shape, while source reconciliation, deployment, and
cutover remain blocked.

The backend Rust mapper now hydrates reviewed legacy rows into the domain notification aggregate
without inventing UUIDs, wallets, timestamps, or delivery facts. It accepts only the pinned
legacy aliases, preserves broadcast rows through the explicit broadcast topic, validates channel,
status, schedule, expiry, and action/image URL boundaries, and rejects malformed rows for
quarantine rather than silently coercing them. This improves local backfill compatibility; it is
not populated production reconciliation or cutover evidence.

Exit: empty-database and legacy-upgrade tests pass; no service startup performs DDL; destructive changes require separate reviewed necessity and recovery evidence.

### N3 — Publishers and targeting

Implement the `HttpNotificationAdapter` behind an internal identity boundary. Each payment/subscription/permission/chat/expiry producer writes a transactional outbox event. The notification service verifies the caller, deduplicates through an inbox, resolves wallet/plan/broadcast recipients server-side, and creates notification/channel jobs atomically.

The first N3 boundary is now implemented: `POST /api/v1/notification/publish` accepts only the
dedicated publisher audience, permission, and an explicit `svc:` service subject, validates bounded payloads, hashes requests, and
atomically records request idempotency, inbox receipt, and pending outbox state. Concrete wallet
events additionally materialize one in-app notification and channel job in that transaction;
broadcasts now materialize one deterministic durable `recipient='all'` row and wake owner streams;
publisher plan targets now resolve active, unexpired memberships through the explicitly configured
read-only core resolver pool for one bounded membership query, then resolves each bounded wallet's
channel and quiet-hour policy from the notification store before materializing deterministic
per-wallet rows/jobs. The core pool sets `default_transaction_read_only` on every session, and
the resolver fails closed when either bounded read cannot complete.
Provider fanout for broadcasts and the admin plan/broadcast surface remain separate work. The
backend `NotificationPort` has an opt-in remote adapter selected by `NOTIFICATION_ADAPTER=remote`;
it also exposes an explicit stable-event-id method, and the concrete payment, permission, chat,
and expiry publishers now pass source-derived identities so a network retry reuses the same
idempotency key instead of minting a duplicate UUID. The in-process adapter now compares the
immutable wallet/type/priority/content/data/action payload when a deterministic event ID already
exists and returns a conflict for mismatched reuse; remote publish requests carry the stable
event identity as their request ID and reject success bodies larger than 8 KiB;
plan-expiry identities additionally include the canonical recipient wallet, preventing two wallets
on the same plan and day from colliding in the remote inbox;
the publisher boundary also rejects the generic `notification.send`/`notification.broadcast`
event types when their concrete-versus-`all` target semantics do not match;
the default remains in-process until publisher reconciliation, plan/broadcast targeting, and
rollout proof are complete. Production and explicitly remote-configured backend startup now fails
closed when the adapter cannot be built, while non-production test harnesses may omit the port.
Gateway and direct-service policies now admit publisher and provider
callback routes only for their dedicated service audiences and permissions. The executable contract is
[`contracts/notification-publisher.json`](contracts/notification-publisher.json).

The notification service owner helper now rejects malformed EVM wallet identities, normalizes
verified and compatibility owner values to lowercase, and compares the optional compatibility
selector case-insensitively before any owner SQL predicate. Direct owner-list queries also reject
unknown fields, conflicting type aliases, unsupported delivery/read statuses, malformed or reversed
RFC3339 date ranges, and unbounded pagination values before SQL. The backend-owned query applies
bounded type, priority, date, read/unread/all, expiry, and broadcast predicates; the Rust/Dioxus BFF
accepts source-compatible page or offset forms without adding UI filter controls. A guarded local
runtime audit now exercises those predicates against real SQL, while source envelope/field parity,
foreign-resource behavior, and broader owner-operation integration remain open.

A guarded publisher audit now invokes the real admission handler against disposable notification
and core-membership databases and proves concrete-wallet replay deduplication, same-event
payload-conflict rejection, transactional rollback when plan resolution fails after idempotency,
inbox, and outbox insertion, read-only active-plan membership fanout, and one durable broadcast row
across duplicate requests. Reproduce it with
`scripts/migration/test-notification-publisher-local.sh --allow-local`; process-crash recovery,
source-compatible producer wiring, service-identity provisioning, and staging evidence remain open.

Producer reliability now has a bounded backend-owned boundary: every migrated payment, permission,
chat, and subscription-expiry call site is required by `cargo xtask notification-producer-audit --strict`
to use `NotificationPort::send_with_event_id_retry` (or its broadcast counterpart). The helper retries
once after 25ms only for classified infrastructure failures and reuses the same source-derived event ID;
permanent validation, authorization, conflict, and other non-infrastructure errors are returned without
retry. This proves local source wiring and stable retry identity, not a durable producer outbox, process
crash recovery, provisioned service credentials, or remote/staging replay.

Exit: duplicate, reordered, delayed, unauthorized, wrong-event-type, oversized fanout, producer crash, consumer crash, and replay tests pass without lost or duplicate logical notifications.

### N4 — Delivery workers

Separate request admission from delivery. Add email and in-app workers, leases, timeouts, transient/permanent error classification, bounded exponential backoff with jitter, terminal states, dead letters, redrive authorization, provider IDs, and reconciliation. Require provider configuration and TLS policy; never log message bodies or recipient addresses.

Exit: enqueue/attempt/accept/fail/retry/dead-letter/redrive/recovery tests pass; no missing provider reports success and no blocking provider call executes on an async request worker.

The current worker boundary includes a guarded operator-only dead-letter redrive endpoint, durable
attempt/lease/backoff primitives (including SKIP LOCKED reclamation of expired leases), and an active asynchronous loop that drains in-app jobs and
records terminal failures for channels without a configured provider. Retry exhaustion is converted
to the same terminal/dead-letter path at the final attempt boundary. The SMTP adapter is now
isolated in a blocking pool, rejects plaintext fallback, and missing configuration fails closed.
Provider callbacks now have a dedicated audience/permission boundary, timestamped raw-body HMAC
verification, bounded payload validation, provider/event idempotency, and guarded job/notification
state reconciliation. Provider callback vocabulary is centrally mapped to the durable
`provider_accepted`/`terminal_failed` states, and SQL guards preserve terminal or accepted outcomes
under reordered callbacks; the SMTP transport has a bounded 30-second timeout; successful SMTP attempts now carry a deterministic RFC-style Message-ID
derived from the durable channel-job identity into the delivery-attempt provider-id column. Push jobs now use standards-compliant Web Push
encryption/VAPID signing, bounded provider timeouts, deterministic provider IDs, and endpoint revocation on permanent endpoint errors. Secret
rotation retirement, delivery reconciliation drills, push-provider runtime proof, and redrive runtime drills remain deferred, so N4 is not
production-ready. Local callback evidence now covers bounded active/previous-key overlap; deployment
rotation retirement and external provider recovery remain separate gates. The callback contract is
[`contracts/notification-provider-events.json`](contracts/notification-provider-events.json).

A guarded local runtime audit now boots the compiled service against a fresh migrated PostgreSQL
database, verifies `/health` and `/ready`, proves an expired job becomes
`expired`/`terminal_failed`, proves an in-app job becomes `sent`/`provider_accepted` with an
accepted attempt, and checks unauthenticated publisher admission fails with `401`. It is
reproducible with `scripts/migration/test-notification-runtime-local.sh`; this is disposable
local evidence only and does not replace provider, staging, or browser proof.

The worker transition audit additionally runs the real `DeliveryWorker` against a disposable
migrated database and proves one retry attempt exhausts, persists a dead letter, and can be
redriven through the worker's authorized transition; it then forces an expired lease and proves
a replacement worker reclaims and accepts that job. It also exercises a signed provider callback,
duplicate callback replay, terminal-state reorder protection, and invalid-signature rejection.
Reproduce it with
`scripts/migration/test-notification-delivery-local.sh --allow-local`; the same runtime callback
proof signs one accepted event with the previous provider key and a reordered event with the active
key, proving bounded overlap acceptance without claiming deployment rotation or key retirement.
Provider, redrive-route, staging, and production evidence remain separate gates.
The guarded `scripts/migration/test-notification-push-provider-local.sh --allow-local` test also
exercises the actual Web Push encryption/VAPID path against a loopback provider and verifies a
successful response plus a deterministic job-derived provider ID. It is transport evidence only:
browser permission, a real push service, provider callbacks, and staging/production delivery
remain separate gates.

### N5 — Realtime, preferences, and push

Add owner-bound SSE with durable cursor/replay, broadcast inclusion, keepalive, bounded connections, backpressure, reconnect dedupe, and explicit acknowledgement. Implement owner preferences and quiet hours in the backend before job creation, with documented security/legal exceptions. Implement actual browser push subscription/status/unsubscribe and key rotation.

The first N5 boundary is now implemented with optional Redis wake-up fanout and a durable expiry
projection: owner preferences are
validated and persisted by the service, the BFF forwards bounded GET/PUT settings, and the service
exposes an owner-bound polling/replay SSE stream with `Last-Event-ID`, broadcast inclusion, a
bounded connection semaphore, durable replay cursors, explicit acknowledgement, keepalive, and
reconnect dedupe. Redis carries only a bounded wake-up hint; PostgreSQL remains authoritative for
payload, ownership, ordering, and replay, and streams continue polling when Redis is unavailable.
Push status/subscribe/revoke endpoints require a configured VAPID public/private key pair and persist endpoint
ownership with revocation timestamps; the BFF exposes the same typed boundary. These are static
and unit-tested contracts. Owner-bound send and concrete-wallet publisher paths now consult the
persisted channel map and schedule jobs after a valid quiet window; malformed preference values fail
closed, updates accept only PostgreSQL-known IANA time zones, and suppressed requests remain durable
notification records without a channel job. Expiry is validated to a bounded future window,
filtered from owner reads and engagement mutations, and swept in bounded batches before the worker
claim/attempt boundary; expired notifications/jobs become terminal without a provider call. The account
runtime now provides a guarded local browser permission/subscription control, but this is not proof of
multi-instance Redis recovery or live push-provider delivery. VAPID subscriptions now persist a bounded key
lineage ID so an active plus previous configured key pair can rotate without stranding queued
subscriptions; deployment rotation drills remain unproven.

The shared Rust `NotificationPort` now carries the same optional `expires_at` through both the
in-process and HTTP adapters, and the legacy stable-event identity comparison includes expiry so a
retry cannot reuse an event with a different lifecycle boundary. The in-process adapter applies the
same future/365-day validation as the extracted service before persisting or fanning out a row.

The account SSR path now reads this contract through the frontend BFF's strict
DTO validator and exposes a bounded native Dioxus form for the saved
channel/quiet-hour/timezone values. Its same-origin Rust adapter translates no
client JavaScript mutation state: it accepts only the exact form fields, reuses
the canonical JSON validation, forwards the verified bearer, and redirects back
to SSR `saved`/`error` states. The account-only push runtime requests browser
permission only after a user click, while the Rust-served public worker validates
received payloads and same-origin action URLs before showing a notification. No
client state claims provider acceptance or delivery success.
The authenticated Dioxus `/notifications` page now exposes a non-authoritative live
status, opens the owner SSE stream through the Rust BFF, validates the exact nine-field
notification event, and acknowledges its `Last-Event-ID` through the BFF before a
server-rendered reload. The acknowledgement route forwards only the verified bearer,
bounded cursor, and request ID to the notification service; malformed or foreign
responses remain fail-closed. This is local browser/runtime evidence only and does not
prove multi-instance fanout, offline browser receipt, or provider delivery.

Owner read, unread, and mark-all mutations now update the independent engagement row in the same
transaction as the legacy `read_at` projection, without changing delivery status. Owner click and
dismiss routes use idempotent first-event timestamps in that same engagement row and do not alter
read or delivery state; the Rust frontend BFF exposes matching owner POST adapters plus legacy PUT/
DELETE aliases that forward only the verified bearer and request ID to the corresponding Rust-owned
service mutation. Owner deletes remove engagement rows transactionally before
attempting the legacy delete; durable channel-job foreign keys still prevent unsafe deletion and
require lifecycle cleanup evidence.

A guarded preference audit now executes the Rust preference, push, and send handlers against a
disposable migrated database, proving known-IANA-timezone validation, a current quiet-hour release
calculation, owner-bound push subscribe/revoke and endpoint ownership rejection, and disabled-channel
suppression that persists a notification without creating a channel job.
The same guarded audit now exercises owner deletion with populated attempts, dead letters, provider
events, channel jobs, expirations, engagement, and outbox rows; cleanup is ordered transactionally
and preserves shared broadcast rows and any source event still used by another owner.
Reproduce it with `scripts/migration/test-notification-preferences-local.sh --allow-local`; live
browser submission/provider delivery, exception handling, and multi-instance recovery remain open.
The same disposable audit now proves a valid `Last-Event-ID` reconnect and
owner-bound durable acknowledgement cursor; the BFF acknowledgement boundary and
authenticated Dioxus stream controller are also statically and unit tested. Browser
receipt and multi-instance Redis recovery remain open.
Redis wake-up publication is now bounded to one second and always signals the
local PostgreSQL-backed stream first; an unavailable Redis endpoint is covered
by a deterministic fallback test. Multi-instance Redis recovery is now covered
against a disposable ephemeral broker restart; staging-scale, durable SSE
disconnect/reconnect, browser receipt, and production recovery remain open.
The guarded `scripts/migration/test-notification-redis-multi-instance-local.sh --allow-local`
audit now runs two independent Redis listeners against the local service code and verifies that
one owner-scoped wake-up reaches both listeners while an unavailable Redis connection still leaves
the local replay wake-up bounded; its second phase restarts an ephemeral
`redis-server` and verifies both listeners recover. This strengthens local
broker evidence, but it is not a live browser receipt, staging, or production
recovery drill.

Exit: multi-instance Redis loss/recovery, disconnect/reconnect, `Last-Event-ID`, expiry, duplicate, preference, quiet-hour/time-zone, and push rotation/revocation tests pass.

### N6 — Templates and complete UX

Implement strict typed template variables, versions, preview, sanitization, size limits, action/image URL policy, atomic cache invalidation, rollback, and audit. Wire the user list, filters, counts, row/bulk mutations, settings, browser permission, realtime updates, empty/error/loading/offline states, and admin history/stats/send/schedule/broadcast/template flows to live APIs. The admin compose route now has one bounded, same-origin canonical-wallet/in-app form backed by the verified Rust BFF and durable notification-service enqueue; unsupported broadcast, plan, scheduling, image, and arbitrary-data controls remain explicitly unavailable until their contracts are verified. Do not reintroduce sample fallback.

The service now rejects unbounded/invalid template definitions, requires typed variable schemas and
validates render data before strict Handlebars compilation, rejects missing template IDs instead of
falling back to raw bodies, and records each update or version rollback in
`notification_template_versions`, soft-deactivates deletes, and invalidates the in-memory registry
atomically after the database commit. A bounded admin preview route validates typed render data
before rendering, and an admin rollback route restores a validated version as a new version with
post-commit cache refresh. Admin sends now require a canonical wallet target, bind in-app recipients
to that wallet, validate email/push recipient shapes, reject broadcast/all and conflicting
inline/template content, verify template/channel agreement, and apply an explicit template tag/
attribute allowlist with balanced nesting checks. Template bodies now pass through a parser-backed
Ammonia/html5ever allowlist with canonical byte-equality, in addition to the application URL policy;
the guarded local rollback audit now proves Rust-handler replay/readback, while authenticated
browser/admin replay evidence, live provider push delivery, and broader browser-level UX remain
deferred until their contracts and provider evidence are complete.

The template audit-read path now fails closed on malformed stored rows and permits only the
version/action metadata emitted by the service (`template_name` or the two positive rollback
versions), with bounded identifiers, actor subjects, and metadata size. The admin BFF repeats the
same allowlist before forwarding the projection; this is deterministic privacy boundary evidence,
not a live rollback replay or browser proof.

Template preview and send render data also reject undeclared top-level variables, so the typed
schema is an allowlist rather than a permissive subset; nested value-size and full content
sanitization/provider evidence remain separate gates.

A guarded loopback audit now inserts an unsafe active template into a disposable migrated database
and proves the compiled service exits non-zero before readiness with the startup template-load
failure. Reproduce it with
`scripts/migration/test-notification-template-startup-local.sh --allow-local`; this is local
fail-closed evidence only and does not replace browser, provider, staging, or production proof.

A second guarded integration audit exercises the actual Rust template handlers through create,
update, rollback, and audit-readback, proving that version 1 content is restored as a new version
and its rollback metadata is durable. Reproduce it with
`scripts/migration/test-notification-template-rollback-local.sh --allow-local`; browser/admin,
provider, staging, and production evidence remain open.

The guarded `scripts/migration/test-notification-browser-local.sh --allow-local`
smoke audit now starts the local Rust/Dioxus BFF and verifies accessible public
home output, signed-out notification authentication redirect, truthful account
preference state, and no signed-out notification runtime. Provider delivery,
admin replay, a device matrix, and staging evidence remain separate gates; the
authenticated settings and responsive local audits are recorded below.

The guarded `scripts/migration/test-notification-browser-authenticated-local.sh
--allow-local` audit now accepts a caller-supplied short-lived local bearer and
verifies that the live Dioxus account page reloads owner preferences with native
controls, while the authenticated notifications page stays on its owner route
and does not render the signed-out state. With local VAPID intentionally absent,
it also verifies that the account push control remains unavailable and does not
request permission implicitly. The token is never printed or stored; the audit
is read-only and still does not prove provider delivery, multi-instance recovery,
admin replay, mobile layout, or staging.

The companion `scripts/migration/test-notification-browser-responsive-local.sh
--allow-local` audit sets a 390×844 viewport and verifies that the authenticated
account and notifications pages have no horizontal overflow, retain native
focusable controls, expose the live status marker, and render either the bounded
notification list or its truthful empty state. This is responsive local browser
evidence only; it is not a device matrix, provider, staging, or production proof.

The guarded `scripts/migration/test-notification-browser-admin-authenticated-local.sh
--allow-local` audit accepts a short-lived permissioned admin bearer and verifies
that the live Dioxus admin notification inventory stays on
`/notifications/manage`, renders the backend-authoritative empty or populated
state, and does not render the signed-out gate. It is read-only and local; it
does not prove provider delivery, staging, multi-instance recovery, mobile
layout, or production replay.

The guarded `scripts/migration/test-notification-browser-mutations-local.sh
--allow-local` audit now exercises the authenticated Dioxus admin enqueue,
owner preference save/reload, mark-read, acknowledgement, and disposable delete
through the same-origin Rust BFFs. It proves only a loopback bearer flow and a
disposable local row; provider, staging, multi-instance, device-matrix, and
production evidence remain separate gates.

Both Rust/Dioxus BFFs now accept the explicit `NOTIFICATION_SERVICE_URL`
origin: owner requests from the frontend and admin list/template/metrics/send
requests use the extracted service while identity, payments, and other domains
remain on `API_URL`. Disposable local owner/admin identities exercised the
service preferences, owner-list, metrics, and admin-list paths with their
required permissions and were removed after the checks; staging replay and
provider evidence remain separate gates.

Exit: desktop/mobile browser tests cover keyboard, accessibility, responsive layout, optimistic rollback, auth/owner/admin boundaries, network/provider errors, SSE reconnect, and truthful success states.

### N7 — Backfill, reconciliation, and operations

Build dry-run and resumable backfill tooling with checkpoints and bounded locks. Reconcile counts, canonical-wallet checksums, status distributions, broadcasts, preferences, templates, duplicates, orphans, provider IDs, and source event IDs. Add dependency readiness, queue depth/age, per-channel outcomes, retry/dead-letter metrics, SSE connection/lag/replay metrics, privacy-safe traces, SLOs, alerts, dashboards, and runbooks.

The N7 offline operations boundary now includes a reviewed operations contract, a no-write
`cargo xtask notification-backfill --dry-run --input <jsonl>` validator with a 100,000-record bound,
duplicate/source-wallet/status checks, resumable `--after` checkpoint selection, and an explicit
`--legacy` mapper for `wallet_notifications` rows. The legacy mapper canonicalizes wallet case,
conservatively rejects topic-only rows and unknown statuses, maps `created`/`queued`/`scheduled`
to `pending` and `delivered`/`read` to `sent`, and emits stable `legacy.wallet_notification:<uuid>`
event identities without writing target rows. A service
`GET /ready` probe that checks both base and fourteen-table lifecycle compatibility. It now also
includes `cargo xtask notification-reconcile --dry-run --source <source-jsonl> --target <target-jsonl>`
for event-set, wallet-checksum, status, broadcast, orphan, duplicate, template, preference, and
provider-identity drift.
Checked-in three-record source/target fixtures now exercise the resumable checkpoint,
suppressed/broadcast records, provider/template/preference identities, matching checksums, and
fail-closed drift rejection through `scripts/migration/test-notification-operations.sh`.
The guarded `scripts/migration/test-notification-backfill-populated-local.sh --allow-local`
also runs the notification migrations on a disposable PostgreSQL database, maps four populated
legacy rows into four target rows, and reconciles their wallet checksum, status distribution,
broadcast identity, and provider acceptance. This is stronger populated local evidence only;
staging-scale/live database reconciliation, rollback, and cutover approval remain open.
It also includes a no-write `notification-readiness` evaluator with checked-in healthy and
unhealthy redacted metrics snapshots; the evaluator applies bounded queue, retry, dead-letter,
provider-acceptance, replay, and SSE thresholds and fails closed on unhealthy input. This remains
an offline metrics-contract test and does not create staging alerts or prove live dependency health.
The service readiness response now also reports whether optional Redis wake-up fanout is configured
and whether a bounded Redis `PING` is reachable, alongside SMTP,
core plan-targeting resolution, active/previous VAPID rotation material, and provider-callback signing are configured, without treating any configuration flag as a
successful connectivity, delivery, or recovery claim.
The guarded local runtime audit additionally verifies that readiness observes the fourteen-table
lifecycle schema and that the active worker performs expiry and in-app acceptance transitions
against PostgreSQL; no staging-scale or multi-instance recovery claim is inferred.
The guarded `scripts/migration/test-notification-colima-replicas-local.sh --allow-local` preflight
now provisions only the notification Deployment/Service in a disposable Colima
`epsx-staging-audit-*` namespace,
verifies two Ready replicas against a scratch migrated database, observes Redis reachability loss
through a disposable Secret rollout, restores the endpoint, and confirms zero pod restarts before
removing the namespace and database. This is stronger local dependency/rollout evidence, but it
does not satisfy staging-scale, external provider, browser push, alert/dashboard, or production
cutover requirements.
When invoked with `--queue-records 1000`, the same disposable preflight inserts 1,000 future-dated
channel jobs, maps 1,000 populated legacy rows into target rows, reconciles their checksums and
status/event sets, and verifies `/ready` reports the exact bounded queue depth without claiming
delivery. This remains disposable local Colima evidence, not approved staging or production
reconciliation.
The admin-authorized `GET /api/v1/notification/admin/metrics` route now exposes a bounded read-only
queue/outcome snapshot including durable preference suppression count; the admin BFF adds a strict
typed projection at `/api/v1/notifications/metrics`. The redacted
`contracts/notification-observability.json` contract now pins metric labels, SLO thresholds,
alerts, dashboard panels, and privacy guards through
`scripts/migration/test-notification-observability-contract.sh --allow-local`. These remain local
design/evidence only; `scripts/migration/test-notification-observability-local.sh --allow-local`
also proves the healthy fixture stays alert-free and the unhealthy fixture raises the bounded
queue, provider, SSE-lag, and stream-query-failure alerts. No live telemetry, staging metrics,
alerts, dashboards, or recovery drills are claimed.
The disposable runtime audit also invokes the real metrics projection with populated queue,
provider-event, suppression, and replay-cursor fixtures, checks the exact redacted key set and
counts, and proves owner/content/token material is absent from the serialized response. This is
local database evidence only; it does not create a telemetry backend or staging dashboard.
The notification service also has
a standalone workspace Dockerfile and checked-in Kubernetes deployment/service resources; the
manifests deliberately remain unapplied until image, secret, dependency, and rollback evidence is
reviewed.

Exit: staging-scale reports meet reviewed thresholds and recovery/runbook drills succeed. Integrity scripts alone cannot satisfy this exit.

### N8 — Shadow, canary, cutover, and rollback

Shadow-read and compare without serving target results. Canary explicitly allowlisted publisher event types and wallets. Switch exactly one writer by audited configuration. Preserve duplicate-safe rollback using the same inbox/idempotency/provider records; reconnect SSE without losing durable events. Disable legacy writes only after reconciliation and rollback-window approval.

The non-production contract at `contracts/notification-cutover-rollback.json` now pins the
fail-closed legacy writer default, no-serve shadow mode, bounded canary allowlist, abort
thresholds, required approvals, and duplicate-safe rollback sequence. The static validator
does not execute any of those operations; the cutover gate remains blocked until reviewed
shadow, canary, reconciliation, and rollback evidence exists.

Exit: reviewed go/no-go evidence includes shadow parity, canary outcomes, migration reconciliation, alert health, single-writer proof, rollback rehearsal, and post-switch reconciliation. Production deployment still requires separate explicit user authorization.

## STOP blocker ledger

| ID | Blocked area |
|---|---|
| B01 | Source/target API compatibility |
| B02 | Legacy/candidate schema incompatibility and no clean/upgrade execution proof |
| B03 | Unsafe/ambiguous migration history and no adoption proof (runtime DDL/seeds are statically absent) |
| B04 | Complete ownership and normalization proof |
| B05 | Independent lifecycle state machines |
| B06 | Durable preferences and quiet-hour enforcement |
| B07 | Authenticated realtime stream |
| B08 | Offline replay, cursor, dedupe, acknowledgement |
| B09 | Real browser push lifecycle |
| B10 | Canonical admin targeting and bounded fanout |
| B11 | Internal publisher identity and HTTP adapter |
| B12 | Transactional inbox/outbox and atomic job creation |
| B13 | Request/event/job/provider idempotency |
| B14 | Retry, lease recovery, dead letter, redrive |
| B15 | Truthful, asynchronous, privacy-safe email delivery |
| B16 | Truthful in-app persistence/fanout semantics |
| B17 | Strict/versioned/synchronized templates |
| B18 | Privacy, content safety, retention, erasure, secrets |
| B19 | Dependency health, metrics, SLOs, alerts, runbooks |
| B20 | Backfill and reconciliation |
| B21 | Deployable service/worker topology and probes |
| B22 | Single-writer cutover and duplicate-safe rollback |

The machine contract owns the full blocker summaries, evidence references, and resolutions. Blockers may move only through reviewed implementation and runtime evidence; editing the status field alone must fail review.

## Verification

```sh
./scripts/migration/verify-notification-execution.sh --mode integrity
./scripts/migration/test-notification-runtime-local.sh --allow-local
./scripts/migration/test-notification-delivery-local.sh --allow-local
./scripts/migration/test-notification-publisher-local.sh --allow-local
./scripts/migration/test-notification-template-startup-local.sh --allow-local
./scripts/migration/test-notification-template-rollback-local.sh --allow-local
./scripts/migration/test-notification-privacy.sh
./scripts/migration/verify-notification-execution.sh --mode readiness  # expected exit 3
./scripts/migration/verify-notification-execution.sh --mode report
./scripts/migration/test-notification-execution.sh
```

The verifier refuses database, Redis, SMTP/push, Kubernetes/internal-service, network/proxy, and production-looking environment variables. The self-test proves deterministic reporting, readiness exit `3`, source-anchor tamper rejection, stale-source rejection, path-traversal rejection, and environment refusal. No command in this gate connects to live infrastructure.
