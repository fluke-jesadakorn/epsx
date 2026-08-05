# Migration status: development to Dioxus and Rust services

This branch is an active migration, not a completed production cutover.

- Source baseline audited: `origin/development` at `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`.
- Target baseline audited: `migration/dioxus-microservices` at
  `975c09567fe14ce278370720bd7a0e5aa571e116` before the readiness-document update.
- Production system of record: the Rust monolith in `apps/backend` until each
  extracted domain passes its contract, security, data, and rollback gates.
- Deployment status: no production deployment is authorized by this document.

## What route parity means today

The Dioxus dispatchers have path-level counterparts for all 28 audited frontend
pages and all 27 audited admin pages. This is path presence only:

- `28/28 frontend` means every audited source page has a target dispatcher path.
- `27/27 admin` means every audited source page has a target dispatcher path;
  two paths intentionally redirect to canonical sub-pages.
- The E2E inventories contain route samples, dynamic examples, redirects, and
  target-only routes. Their sample count is not a functional-completion score.

Path presence does not prove matching interactions, authentication, live data,
backend authorization, checkout behavior, visual fidelity, or production
operability. Those gates remain incomplete.

## Verified narrow baseline

The `epsx-dioxus-ui` unit and documentation tests are the narrow test baseline
for shared UI components. A green result for this package does not imply that
the workspace, BFFs, microservices, migrations, Kubernetes manifests, or E2E
flows are production-ready.

Use the detailed execution plan and current evidence in
[`docs/migration/PRODUCTION_READINESS_PLAN.md`](docs/migration/PRODUCTION_READINESS_PLAN.md).

## Current continuation checks

The active branch now has Rust-native repository checks that do not merge or
modify development:

    cargo xtask sync-audit
    cargo xtask authority-audit --strict
    cargo xtask notification-compatibility-audit --strict
    cargo xtask notification-producer-audit --strict
    ./scripts/migration/test-notification-browser-local.sh --allow-local
    NOTIFICATION_BROWSER_AUTH_TOKEN=<short-lived-local-bearer> \
      ./scripts/migration/test-notification-browser-authenticated-local.sh --allow-local
    NOTIFICATION_BROWSER_AUTH_TOKEN=<short-lived-local-bearer> \
      ./scripts/migration/test-notification-browser-responsive-local.sh --allow-local
    NOTIFICATION_COMPATIBILITY_OWNER_TOKEN=<short-lived-owner-bearer> \
    NOTIFICATION_COMPATIBILITY_ADMIN_TOKEN=<short-lived-admin-bearer> \
      ./scripts/migration/test-notification-compatibility-local.sh --allow-local
    NOTIFICATION_ADMIN_BROWSER_AUTH_TOKEN=<short-lived-local-admin-bearer> \
      ./scripts/migration/test-notification-browser-admin-authenticated-local.sh --allow-local
    cargo xtask notification-backfill --dry-run --input <bounded-jsonl>
    cargo xtask notification-backfill --dry-run --legacy --input <legacy-wallet-notifications-jsonl>
    cargo xtask notification-reconcile --dry-run --source <source-jsonl> --target <target-jsonl>
    cargo xtask notification-migration-audit --report
    cargo xtask rust-audit --report
    cargo xtask migration-audit --report

The sync audit treats origin/development as a behavior reference only.
The compatibility audit checks all 23 reviewed BFF/service method-path
registrations against the checked-in Rust routers; payload, envelope, status,
legacy-source, live-service, and browser parity remain separate gates.
The producer audit checks seven migrated backend producer files, eight stable
event-identity call-site anchors, and zero legacy shim calls; service identity
provisioning, crash recovery, and remote/staging replay remain separate gates.
The disposable preferences audit also proves owner-bound stream cursor
reconnect/acknowledgement against PostgreSQL; browser receipt and multi-instance
Redis recovery remain separate gates.
Redis wake-up publication is bounded and local PostgreSQL replay is signalled
before the optional Redis hint; deterministic Redis-loss fallback is covered.
The guarded browser smoke audit exercises the local Rust/Dioxus BFF for public
home accessibility, signed-out notification redirect, truthful account state,
and absence of signed-out notification runtime; authenticated browser/provider
behavior remains a separate gate.
The authenticated browser audit additionally checks live owner preferences and
the authenticated notification route without persisting the supplied bearer;
when local VAPID is intentionally absent it also verifies that the account push
control stays unavailable and never requests permission implicitly.
The local compatibility audit checks owner list/count/preferences envelopes
through both the extracted service and frontend BFF, a foreign-owner `404`,
and the permissioned admin inventory projection; its supplied bearers are
never printed or stored.
The admin authenticated browser audit checks the permissioned Dioxus global
notification inventory through the admin BFF, also without persisting its
short-lived bearer; both are local read-only evidence only.
Set `NOTIFICATION_SERVICE_URL` for both Rust/Dioxus BFFs to route notification
requests to the extracted service; it falls back to `API_URL` when unset.
Set `CONTENT_SERVICE_URL` for the Rust/Dioxus frontend BFF to route public
content/news requests to the extracted content service; it falls back to
`API_URL` when unset. The local content service listens on `127.0.0.1:8105`
when started with the development content database.
The strict Rust and migration audits are completion gates and intentionally
fail while the tracked legacy toolchain, embedded browser scripts,
migration-version collision, and reviewed destructive SQL remain.

The current notification continuation slice is verified by the notification
service tests, focused frontend/Dioxus notification tests, and deterministic
A11 integrity/self-test gates. It adds bounded native owner pagination,
owner preferences, a bounded owner SSE stream with durable replay cursors and
acknowledgement (including a durable owner receipt timestamp), push subscription status/rotation boundaries, strict template
versioning, and a static additive lifecycle foundation with guarded Rust
transition types. It also provides a no-write source/target reconciliation report
with event-set, wallet-checksum, status, broadcast, orphan, provider-ID, and
template/preference/provider-identity drift checks. A no-write `notification-readiness` evaluator
also checks redacted queue, retry, dead-letter, provider-acceptance, replay, and SSE thresholds
against healthy/unhealthy fixtures; it remains offline evidence only and does not create staging
alerts or prove live dependency health. Email transport configuration now has no
plaintext fallback, and
template sends require typed variable schemas and reject missing IDs instead of
using an untyped raw-body fallback; raw-output Handlebars expressions are
rejected for new and startup-loaded templates, and admin preview/version
rollback restore only validated template versions; template mutations leave
durable actor/version audit records and unsafe markup/event-handler/metadata/image URL patterns
are rejected. Publisher action URLs and
legacy admin-send action/image URLs are restricted to bounded relative or
HTTPS paths without credentials, unsafe schemes, or host escapes.
Template audit reads now fail closed on malformed rows and allow only bounded identifiers,
actor subjects, and the service-emitted template-name or positive-version metadata; the admin BFF
repeats that allowlist before forwarding the audit projection. This remains static boundary
evidence and does not prove live rollback replay or browser behavior.
Template preview and send payloads now also reject undeclared top-level variables before Handlebars
rendering, keeping the typed schema an allowlist.
The notification migration audit verifies checked-in up/down checksums and a guarded
non-production clean/populated upgrade plus dump/restore evidence; its strict mode still does not
authorize production adoption or cutover.
Extracted admin sends now require a canonical wallet target, bind in-app recipients to that wallet,
validate email/push recipient shapes, and reject broadcast/all or conflicting template/inline content
before durable enqueue; the internal publisher now resolves bounded active plan memberships through
an explicitly configured read-only core resolver pool with per-wallet preference enforcement, while
source-compatible admin plan/broadcast resolution remains a separate backend gate.
The notification service owner boundary now rejects malformed EVM identities and normalizes verified
and compatibility owner selectors before owner-scoped SQL; live source-parity integration remains
unproven.
The remote notification port also exposes stable-event-id methods, and the concrete payment,
permission, chat, and subscription-expiry publishers pass source-derived identities so retrying one
logical event reuses its idempotency key rather than minting a duplicate UUID.
The legacy backend notification pool factory and admin/user notification handlers also fail closed
when `NOTIFICATIONS_DATABASE_URL` is missing or unavailable; notification paths no longer silently
write to the primary database schema.
The admin-only notification metrics snapshot exposes queue/channel outcomes,
durable preference-suppression, provider-attempt, and replay-cursor counters
without recipient or message fields, plus process-local SSE connection,
reconnect, replay, average-lag, and query-failure counters; it is not a staging
metrics system.
The notification integrity verifier also scans legacy SSE/queue/admin handlers
and extracted service/worker tracing calls for direct sensitive-field interpolation;
retention, erasure, full content sanitization, and provider-specific privacy proof
remain open.
Owner read, unread, and mark-all mutations now persist the independent engagement state in the
same transaction as the legacy `read_at` projection, while delivery status remains untouched;
owner click/dismiss mutations persist idempotent first-event timestamps without changing read or
delivery state; owner deletes remove engagement rows before the legacy delete and still respect
durable channel-job foreign keys.
Provider callbacks now use a dedicated internal audience/permission, timestamped
raw-body HMAC verification, bounded event payloads, provider/event deduplication,
and guarded channel-job/notification state reconciliation; secret rotation and live delivery
drills remain required. Successful SMTP attempts now use a deterministic RFC-style Message-ID
derived from the durable channel-job identity and persist it as the delivery-attempt provider
identity; this improves reconciliation without claiming provider acceptance or live delivery.
Retry exhaustion now enters the same terminal/dead-letter path at the
final attempt boundary instead of remaining retry-only.
Owner-bound sends and concrete-wallet publisher materialization also consult persisted
channel preferences and schedule jobs after timezone-aware quiet hours; direct sends
without an owner remain explicitly ungoverned administrative delivery. Disabled-channel
requests are retained as `suppressed` notification records without a channel job, and
preference writes reject unknown PostgreSQL/IANA time zones.
The Rust/Dioxus account page now reads the same bounded owner preference DTO
during SSR and renders channel, quiet-hour, timezone, and save-time values in a
native form. A same-origin Rust adapter reuses the canonical JSON validator,
forwards only the verified bearer, and redirects to explicit SSR save/error
states backed by a short-lived HttpOnly flash cookie; SSR clears that cookie after
consumption, so a manually supplied query cannot claim a successful write.
Signed-out, unavailable, and malformed responses stay distinct. The authenticated
account also has a native browser-push control: its account-only runtime validates
the exact status envelope, requests permission only after an explicit click, and
forwards bounded subscribe/revoke requests through the Rust BFF; disabled VAPID,
unsupported browsers, permission denial, and provider-delivery uncertainty remain
truthful fail-closed states.
The same Rust-served worker accepts only the exact bounded push payload, rejects
cross-origin action URLs, and handles notification display/clicks without writing
user data to CacheStorage.
The authenticated Dioxus `/notifications` page also exposes a live status and opens
the owner SSE stream through the Rust BFF. It validates the exact event envelope,
acknowledges each event through the bounded `/api/v1/notifications/stream/ack` proxy,
and reloads server-rendered rows only after the durable cursor response is accepted;
signed-out, hidden, malformed, and failed-ack states do not claim live delivery.
This remains local browser/runtime evidence, not proof of multi-instance fanout,
offline receipt, or a live provider.
The responsive local browser audit also checks the authenticated account and
notifications pages at 390×844 for horizontal overflow, native keyboard focus,
and a truthful list/empty-state rendering; it does not stand in for a real-device
matrix or staging evidence.
The migration is executed only by the explicitly guarded local-scratch audit and is not applied
to production or backfilled; the service fails closed until its fourteen lifecycle base tables
exist, and the dry-run commands perform no writes, so this does not claim lifecycle readiness.
The notification service now also has
a standalone Rust Dockerfile plus checked-in Kubernetes Deployment/Service and
dev/staging-only image overlays with secret-backed configuration and `/health`/
`/ready` probes. The production overlay intentionally does not include the
notification resources; no manifest was applied, and the source artifacts do
not prove cluster or provider readiness. Production and explicit remote-adapter startup now fail
closed when the notification port cannot be built; only non-production test
harnesses may omit it.

## Migration strategy

Use a controlled hybrid:

1. Keep monolith authentication, permissions, plans, and durable domain data as
   the source of truth.
2. Put contract and security tests around the BFF/monolith boundary.
3. Extract one vertical domain at a time behind internal routing.
4. Shadow and canary only after migration/backfill and rollback rehearsals pass.
5. Remove a monolith fallback only after its replacement meets the full
   definition of done.

## Production guard

Never deploy, change DNS/Cloudflare routing, apply Kubernetes resources, run a
production migration, or remove a monolith fallback without explicit user
approval for that specific production action. A passing test suite, merged
branch, or completed plan is not deployment approval.
