# Notification operations and reconciliation gate

This runbook is an offline design/verification boundary. It does not authorize
database access, migration execution, provider access, deployment, or cutover.

The dry-run backfill command reads bounded JSONL records and reports invalid
wallets, duplicate source event IDs, unsupported states, and the resumable
checkpoint position. With `--legacy`, it accepts the pinned
`wallet_notifications` row shape, maps only explicit wallet/broadcast rows and
known lifecycle states, canonicalizes wallet case, and emits stable
`legacy.wallet_notification:<uuid>` identities. Topic-only rows and unknown
statuses fail closed rather than being guessed. It performs no SQL, network,
Redis, SMTP, push-provider, or filesystem writes. A production backfill must add an approved checkpoint
store, lock budget, audit actor, and rollback rehearsal before changing the
`productionReady` marker in `contracts/notification-operations.json`.

The local scratch migration harness also inserts one already-expired projected
notification and verifies that the owner visibility and channel-job claim
predicates exclude it. This is expiry-filter evidence only; it does not prove
worker timing, provider behavior, cleanup, or production replay.

Three guarded loopback audits provide additional disposable evidence:

- `scripts/migration/test-notification-runtime-local.sh --allow-local` boots the
  compiled service and verifies readiness, expiry terminalization, in-app
  provider acceptance, unauthenticated publisher rejection, then runs the real
  admin metrics projection against populated queue/provider/replay fixtures and
  rejects owner/content/token leakage from the bounded response.
- `scripts/migration/test-notification-delivery-local.sh --allow-local` runs the
  real `DeliveryWorker` against a migrated scratch database and verifies retry
  attempt persistence, dead-lettering, authorized redrive, and expired-lease
  reclamation.
- `scripts/migration/test-notification-preferences-local.sh --allow-local`
  verifies timezone/quiet-hour and push ownership behavior, disabled-channel
  suppression, and dependency-safe owner erasure across jobs, attempts, dead
  letters, provider events, expirations, engagement, and outbox rows.
- `scripts/migration/test-notification-template-startup-local.sh --allow-local`
  inserts an unsafe active template and verifies the service exits before
  readiness.
- `scripts/migration/test-notification-privacy.sh` validates the no-write
  per-channel retention, active legal-hold, owner/broadcast erasure, and
  redacted-audit policy contract.
- `scripts/migration/test-notification-redis-multi-instance-local.sh
  --allow-local` verifies two independent Redis listeners receive one
  owner-scoped wake-up and that local replay signaling remains bounded when
  Redis is unavailable.
- `scripts/migration/test-notification-push-provider-local.sh --allow-local`
  verifies encrypted Web Push payload construction, VAPID authorization, a
  loopback provider acceptance, and a deterministic provider ID.
- `scripts/migration/test-notification-colima-replicas-local.sh --allow-local`
  provisions only the notification Deployment/Service in disposable Colima
  `epsx-staging-audit-*`, checks two replicas and Redis loss/recovery readiness, and
  removes its scratch namespace/database on exit. Passing
  `--queue-records 1000` additionally inserts 1,000 future-dated channel jobs,
  maps 1,000 populated legacy rows into target rows, reconciles their checksums
  and status/event sets, and verifies `/ready` reports the exact bounded queue
  depth without claiming delivery.
- `scripts/migration/test-notification-backfill-populated-local.sh
  --allow-local --database-url <local-scratch-url>` runs every notification
  migration against a disposable PostgreSQL database, inserts four legacy
  `wallet_notifications` rows, materializes four target rows (including one
  provider-accepted delivery), and reconciles wallet checksums, statuses,
  broadcast identity, and provider IDs. It drops the database on exit.
- `scripts/migration/test-notification-observability-contract.sh --allow-local`
  validates the redacted metric-label allowlist, queue/provider/SSE SLOs,
  alert conditions, dashboard panel contract, and privacy guards.
- `scripts/migration/test-notification-observability-local.sh --allow-local`
  evaluates the healthy and unhealthy redacted snapshots, proving the healthy
  case stays alert-free and the unhealthy case raises bounded queue, provider,
  SSE-lag, and stream-query-failure alerts.

These commands are explicitly local-only and disposable. They do not prove
external SMTP/push delivery, browser permission/receipt, production broker
recovery, live telemetry/alerts/dashboards, deployment, or cutover readiness.

The cutover boundary is separately pinned in
`contracts/notification-cutover-rollback.json` and checked by
`scripts/migration/test-notification-cutover-contract.sh --allow-local`.
It keeps `NOTIFICATION_WRITE_AUTHORITY=legacy` as the fail-closed default,
disallows serving shadow results, requires an explicit event allowlist and
reconciliation approval for any canary, and preserves inbox/idempotency/provider
records during rollback without replaying accepted provider sends. The validator
is design evidence only; it does not switch writers, start shadow traffic,
deploy a canary, or execute rollback.

The authenticated Dioxus `/account` view now has a bounded native preferences
form at `POST /account/notification-preferences`. The Rust adapter is a same-
origin HTML boundary: it requires a verified session bearer, matching
`Origin`/`Host` headers, an optional `Sec-Fetch-Site` value of `same-origin` or
`same-site`, `application/x-www-form-urlencoded`, and the exact email/in-app/
push/quiet-hour/timezone fields. It forwards the canonical JSON PUT only after
the existing preference validator accepts the form, then redirects to explicit
`/account?preferences=saved` or `/account?preferences=error` SSR states. The
adapter sets a short-lived HttpOnly flash cookie and SSR requires the matching
cookie before displaying either status, then clears it; a user-supplied query
cannot manufacture a save-success banner. It does not request browser push
permission, accept provider secrets, or claim delivery;
browser/provider/reconnect evidence remains a separate gate.

The privacy policy contract is intentionally separate from runtime erasure.
Active legal holds must block purge; owner erasure removes owner-owned rows only
after dependent jobs and provider records are reconciled, while broadcast rows
retain the shared event and remove only the requesting owner's engagement. The
local owner-delete audit proves dependency ordering, but legal-hold storage,
retention scheduling, redacted erasure audit persistence, and production erasure
recovery still require implementation and review.

The companion `cargo xtask notification-reconcile --dry-run --source <source-jsonl>
--target <target-jsonl>` command compares bounded source and target records. It
reports event-set drift, canonical-wallet checksums, status distributions
(including durable `suppressed` preference outcomes),
broadcast counts, orphan/missing events, duplicate IDs, template/preference
identity drift, and provider identity drift (including sent rows without a
provider ID). It emits JSON to stdout only
and exits non-zero on drift; it does not connect to a database or provider.

The checked-in deterministic fixtures under `docs/migration/fixtures/` exercise
three records, a resumable checkpoint after the first event, suppressed-channel
status, broadcast inclusion, template/preference identity, provider identity,
and matching wallet checksums. `scripts/migration/test-notification-operations.sh`
replays those fixtures and verifies that a deliberate target drift fails closed;
the populated local audit exercises the same report against rows materialized in
both the legacy and target tables.

The offline `cargo xtask notification-readiness --dry-run --input <metrics-json>`
evaluator applies the redacted N7 thresholds in
`contracts/notification-readiness.json` to queue depth/age, retry and terminal
states, provider acceptance, replay age, stream lag, query failures, and stream
counter sanity. Healthy and unhealthy snapshots are checked in and exercised by
`scripts/migration/test-notification-readiness.sh`; this is a deterministic
metrics-contract test, not staging alert or live dependency evidence.

Readiness should expose database and lifecycle-schema compatibility separately
from liveness. Queue depth/age, per-channel outcomes, retry/dead-letter counts,
SSE connection/lag/replay, provider reconciliation, and privacy-safe traces
must be collected without logging wallet addresses, endpoints, recipients,
message bodies, or bearer tokens.

Email delivery configuration is fail-closed: port 465 uses an SMTPS wrapper and
other configured ports require STARTTLS. There is no plaintext fallback. Retry
exhaustion enters the terminal/dead-letter path at the final attempt boundary. The
notification worker sweeps at most 100 due expiry-projected notifications/jobs
per pass, marks them terminal/expired in PostgreSQL, and checks again after
leasing before any provider call; rows without an expiry projection retain
legacy delivery behavior.
Missing or invalid SMTP configuration leaves the provider unavailable, so queued jobs
remain observable as failed/retry work rather than being reported as delivered. Push jobs use
the same fail-closed rule: a matched VAPID key pair encrypts and signs one bounded request with
the rustls HTTP client, permanent 404/410 endpoint responses revoke the subscription, and
timeouts/server responses remain retryable. No provider response body or subscription secret is
logged.

Plan-targeted publisher events require the explicitly configured, read-only
`NOTIFICATION_PLAN_DATABASE_URL` core resolver pool. The notification store is
isolated from `wallet_plan_assignments`; omitting this URL therefore makes plan
targeting return `503` rather than querying the wrong database or claiming a
successful fanout. Every resolver connection sets PostgreSQL
`default_transaction_read_only=on`. The resolver caps one event at 10,000 active
memberships, then reads channel and quiet-hour policy from the notification store
before inserting rows/jobs; either bounded read failing returns `503`.

Push subscriptions persist `vapid_key_id`. Keep the active pair in
`NOTIFICATION_VAPID_PUBLIC_KEY`/`NOTIFICATION_VAPID_PRIVATE_KEY` with a bounded
`NOTIFICATION_VAPID_KEY_ID`. During rotation, deploy the former pair through
`NOTIFICATION_VAPID_PREVIOUS_PUBLIC_KEY`,
`NOTIFICATION_VAPID_PREVIOUS_PRIVATE_KEY`, and
`NOTIFICATION_VAPID_PREVIOUS_KEY_ID`; new subscriptions use the active ID while
queued subscriptions continue using the previous key. Remove the previous pair
only after its subscription population has been re-subscribed or revoked.

Provider callbacks use `POST /api/v1/notification/provider-events` with the
`epsx-notification-provider` audience and
`internal:notifications:provider-events` permission at both gateway and service
boundaries. The service accepts only bounded object payloads and the five
explicit delivery event types, deduplicates `(provider, provider_event_id)`,
and updates a referenced channel job and linked notification without overwriting
dead-lettered or terminally failed state. The service verifies the raw-body HMAC signature with
a five-minute timestamp window and fails closed without
`NOTIFICATION_PROVIDER_SIGNING_SECRET`; secret rotation and live reconciliation
drills remain required before production use.

The only currently implemented runtime readiness probe is the notification
service `GET /ready`; it returns `503` until both the legacy base schema and
all fourteen additive lifecycle relations pass their read-only compatibility
probes and the channel-job queue can be queried. A response includes
`queue_depth`, `queue_age_seconds`, non-authoritative
`redis_fanout_configured`, bounded `redis_reachable`, `plan_targeting_configured`, `smtp_configured`,
`vapid_configured`, and `provider_callbacks_configured` flags. `redis_reachable` is only a
one-second `PING` observation and does not gate readiness or prove stream receipt. The other
provider flags report local configuration only, not external reachability or delivery success. Redis
carries only stream wake-up hints; PostgreSQL remains authoritative, and a
not-ready response keeps queue fields null and never presents stale queue
health as success.

The admin-authorized `GET /api/v1/notification/admin/metrics` projection is a
read-only queue snapshot, and the admin BFF exposes its typed projection at
`GET /api/v1/notifications/metrics`. It reports queue depth/age, allowlisted per-channel
outcomes, durable preference suppression, retry-wait, attempting,
terminal-failed, dead-lettered, provider-accepted, provider-event and delivery
attempt counts, replay-cursor count/age, and process-local SSE active,
connection, reconnect, replay, average-lag, and query-failure counts
without recipients, bodies, wallet addresses, or provider payloads. It does
not replace staging-scale metrics, per-connection lag telemetry, alerts,
dashboards, or reconciliation.

The checked-in notification deployment is an artifact only and is not applied
by these audits. The local Kustomize render verifies the service image, managed
secret reference, resource/security settings, and `/health`/`/ready` probes for
dev, staging, and prod. Dev explicitly sets `EPSX_ENV=development`; staging
and prod retain production verifier mode. Immutable image provenance,
dependency readiness, worker topology, network policy, rollback, and live
cluster execution remain separate deployment gates.
