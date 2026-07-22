# A11.0 notification lifecycle and delivery execution gate

Status: **audit/design only; production readiness is STOP**. Direct notification-service authentication from A2.3c is a verified prerequisite, but remains **partial** for the lifecycle as a whole. This document does not authorize deployment, production access, database access, Redis access, SMTP/provider access, external network access, or migration execution.

The deterministic authority is [`contracts/notification-execution.json`](contracts/notification-execution.json). It pins 14 source evidence records from `origin/development` commit `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`, 53 current-target anchors, 12 lifecycle/API surfaces, 22 STOP blockers, and eight ordered execution batches.

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

- `/health` is the only anonymous notification-service surface;
- owner routes accept only verified frontend/admin audiences and derive the owner wallet from the principal;
- template/send routes require the admin audience and `admin:notifications:manage`;
- unknown and method-drift service paths fail closed.

It does not establish internal publisher identity, message delivery, preference enforcement, live schema/adoption correctness, migration-history safety, idempotency, replay, observability, deployment, or cutover readiness. Every A11 surface therefore remains blocked.

### Storage and state drift

A3.11 removes the four startup DDL statements and both startup seed paths. It adds a guarded fresh-schema migration for `public.templates` and `public.notifications`, public-qualifies all 19 application relation references, and makes startup reject any schema that differs from the exact 26-column, three-key, five-index contract before template cache load or listener binding. Template query/registration failures also stop startup instead of being discarded.

That is static boundary evidence, not migration readiness. The existing notification migration history still has a renamed/consolidated baseline adoption ambiguity and a destructive `DROP TABLE ... CASCADE` migration. No empty database, populated upgrade, ledger adoption, legacy mapping, backfill, or reconciliation was executed. `CREATE TABLE IF NOT EXISTS` cannot repair an incompatible table created by the former runtime DDL, so both the schema and migration A11 blockers remain STOP.

The legacy model is `wallet_notifications`, with different field names and richer channel/state fields. The target has no declared single write authority, mapping, backfill, reconciliation, or rollback. Read state is also overloaded with delivery status: marking a record read may change `pending` to `sent`, even though provider acceptance, in-app persistence, client delivery, and user read are distinct facts.

### Delivery drift

- `in_app` reports `sent=true` without Redis/SSE fanout or client receipt.
- Missing SMTP configuration logs recipient, subject, and body and reports success.
- Configured SMTP calls a blocking transport inside the async request path.
- Persistence occurs after the transport attempt, with no durable enqueue transaction.
- There is no inbox/outbox connection, idempotency key, worker lease, retry classifier, exponential backoff, dead-letter state, authorized redrive, or provider reconciliation.
- The canonical backend still uses an in-process notification port. The documented HTTP adapter is future work; publication errors may be logged and swallowed, and missing notification DB configuration leaves publisher call sites able to drop messages.

### Truthful owner read slice; lifecycle remains blocked

The extracted service has no SSE route, Redis subscription, durable replay cursor, `Last-Event-ID` contract, acknowledgement route, or push-subscription implementation. The protobuf declares preferences, but the HTTP service has no preference handlers.

The owner page now has a narrow truthful read path: authenticated SSR records exact `ok` or `error` dependency state, the UI requires the current service's nullable keys, parses timestamps as UTC datetimes, treats missing/malformed/error payloads as unavailable instead of plausible empty data, uses neutral title/type/priority presentation fallbacks, ignores unapproved action URLs, and renders static read-only rows with a loaded-list count. It no longer requires the frontend-only `notifications:read` token, falls back to samples, or exposes local filters, mark/delete/bulk, preferences, browser-permission, push, or action controls.

The frontend BFF read boundary is also narrower and fail-closed. `/api/v1/notifications` is GET-only with an explicit `HEAD` override to `405`; every other non-GET method is also rejected. It accepts only unique `status`, `limit`, and `offset` fields, bounds `limit` to `1..=100` and `offset` to `0..=1_000_000`, and rejects unknown fields, duplicates, and identity-bearing query parameters. It forwards only the verified bearer, streams the upstream response under a 2 MiB cap even when `Content-Length` is absent, parses the same bytes into the exact list DTO and passthrough JSON without cloning, and rejects every row whose `user_id` does not match the wallet from the verified session principal. `/api/v1/notifications/unread-count` has the same GET-only plus explicit-`HEAD`-`405` policy, caps streamed bodies at 4 KiB, and accepts only the exact non-negative `{ "count": i64 }` target DTO. Oversized, malformed, and owner-mismatched responses fail as `502`; these are target-hardening facts, not proof of source method, query, envelope, broadcast, expiry, or read-semantics parity.

The former dormant string-renderer badge no longer fetches `limit=1` or derives a global count from one list page; it remains explicitly unavailable. The active SSR shell instead mounts the shared `epsx_templates::epsx_header()` with one inert notification target whose badge starts empty, hidden, `aria-hidden`, and `data-state="unavailable"`. A server-verified authenticated session is the only condition that injects its route-scoped browser controller; signed-out responses inject no runtime or fetch path, and `/offline` excludes it even when a request carries a valid session so the public recovery shell stays free of owner activity. Authenticated non-offline SSR responses and every list/unread BFF outcome, including errors, carry `Cache-Control: private, no-store`; the browser fetch also requests `cache: no-store`.

That controller performs one read-only credentialed `GET` to the exact `/api/v1/notifications/unread-count` BFF route. It accepts only a plain object with the sole key `count` and a non-negative safe integer, resets to unavailable before each request, ignores superseded responses through a monotonic generation guard, aborts on visibility changes, hides zero/error/malformed results, caps only the visible text at `99+`, and preserves the exact count in the link's accessible label. The darker `#dc2626` badge gives its small white text AA contrast; payload text reaches the badge only through `textContent`, with no notification mutation, `innerHTML`, adjacent-HTML insertion, or document write. These are source-anchored static and unit-test facts, not live browser/runtime proof. Source-compatible pagination/filter/envelope/broadcast/expiry/read semantics, browser proof, mutations, preferences, push, SSE, and action-URL policy remain blocked, and all 22 A11 STOPs stay open. The admin UI also contains sample data and does not prove source-compatible stats, scheduling, or broadcast behavior.

### Template, privacy, operations, and deployment drift

Handlebars runs non-strictly. Template upsert/delete behavior is not synchronized safely with the in-memory registry, and version/preview/rollback semantics are absent. Message bodies and recipient data can reach logs; HTML and action/image URLs lack a locked sanitization and allowlist policy.

Generic tracing initialization plus a bare `200` health endpoint does not expose dependency readiness, queue age/depth, provider outcomes, dead letters, SSE lag, preference suppression, or reconciliation drift. The service is a Cargo workspace member but is absent from the Kubernetes base resource inventory, with no immutable artifact, managed secrets, worker topology, probes, resource policy, or rollback manifest.

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

Exit: compatibility fixtures and the authority matrix are reviewed; gateway, BFF, service, and UI paths agree; direct auth remains a prerequisite rather than a lifecycle proxy.

### N2 — Durable schema and migration

Extend the narrow A3.11 fresh-schema migration with versioned, additive migrations for template versions, preferences, publisher inbox, request idempotency, per-channel jobs, attempts, dead letters, provider events, and replay cursors. Add constraints for normalized wallets, allowed channels/types/priorities/states, timestamps, unique event/job identities, and guarded transitions. Repair/adopt the existing migration history and prove both clean and populated upgrade paths while keeping runtime DDL and startup samples absent.

Exit: empty-database and legacy-upgrade tests pass; no service startup performs DDL; destructive changes require separate reviewed necessity and recovery evidence.

### N3 — Publishers and targeting

Implement the `HttpNotificationAdapter` behind an internal identity boundary. Each payment/subscription/permission/chat/expiry producer writes a transactional outbox event. The notification service verifies the caller, deduplicates through an inbox, resolves wallet/plan/broadcast recipients server-side, and creates notification/channel jobs atomically.

Exit: duplicate, reordered, delayed, unauthorized, wrong-event-type, oversized fanout, producer crash, consumer crash, and replay tests pass without lost or duplicate logical notifications.

### N4 — Delivery workers

Separate request admission from delivery. Add email and in-app workers, leases, timeouts, transient/permanent error classification, bounded exponential backoff with jitter, terminal states, dead letters, redrive authorization, provider IDs, and reconciliation. Require provider configuration and TLS policy; never log message bodies or recipient addresses.

Exit: enqueue/attempt/accept/fail/retry/dead-letter/redrive/recovery tests pass; no missing provider reports success and no blocking provider call executes on an async request worker.

### N5 — Realtime, preferences, and push

Add owner-bound SSE with durable cursor/replay, broadcast inclusion, keepalive, bounded connections, backpressure, reconnect dedupe, and explicit acknowledgement. Implement owner preferences and quiet hours in the backend before job creation, with documented security/legal exceptions. Implement actual browser push subscription/status/unsubscribe and key rotation.

Exit: multi-instance Redis loss/recovery, disconnect/reconnect, `Last-Event-ID`, expiry, duplicate, preference, quiet-hour/time-zone, and push rotation/revocation tests pass.

### N6 — Templates and complete UX

Implement strict typed template variables, versions, preview, sanitization, size limits, action/image URL policy, atomic cache invalidation, rollback, and audit. Wire the user list, filters, counts, row/bulk mutations, settings, browser permission, realtime updates, empty/error/loading/offline states, and admin history/stats/send/schedule/broadcast/template flows to live APIs. Remove sample fallback from production behavior.

Exit: desktop/mobile browser tests cover keyboard, accessibility, responsive layout, optimistic rollback, auth/owner/admin boundaries, network/provider errors, SSE reconnect, and truthful success states.

### N7 — Backfill, reconciliation, and operations

Build dry-run and resumable backfill tooling with checkpoints and bounded locks. Reconcile counts, canonical-wallet checksums, status distributions, broadcasts, preferences, templates, duplicates, orphans, provider IDs, and source event IDs. Add dependency readiness, queue depth/age, per-channel outcomes, retry/dead-letter metrics, SSE connection/lag/replay metrics, privacy-safe traces, SLOs, alerts, dashboards, and runbooks.

Exit: staging-scale reports meet reviewed thresholds and recovery/runbook drills succeed. Integrity scripts alone cannot satisfy this exit.

### N8 — Shadow, canary, cutover, and rollback

Shadow-read and compare without serving target results. Canary explicitly allowlisted publisher event types and wallets. Switch exactly one writer by audited configuration. Preserve duplicate-safe rollback using the same inbox/idempotency/provider records; reconnect SSE without losing durable events. Disable legacy writes only after reconciliation and rollback-window approval.

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
./scripts/migration/verify-notification-execution.sh --mode readiness  # expected exit 3
./scripts/migration/verify-notification-execution.sh --mode report
./scripts/migration/test-notification-execution.sh
```

The verifier refuses database, Redis, SMTP/push, Kubernetes/internal-service, network/proxy, and production-looking environment variables. The self-test proves deterministic reporting, readiness exit `3`, source-anchor tamper rejection, stale-source rejection, path-traversal rejection, and environment refusal. No command in this gate connects to live infrastructure.
