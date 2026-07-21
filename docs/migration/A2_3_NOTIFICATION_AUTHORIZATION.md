# A2.3c notification service authorization

This bounded slice makes the notification service consume
`epsx-service-auth` directly. It establishes authentication, granular operator
authorization, and wallet-owner filtering before the existing handlers and
side effects. It does not claim notification delivery or overall A2 production
readiness is complete.

## Audited surface and ownership model

The service mounts fourteen method/path contracts: health, five template/send
operator contracts, and eight user notification contracts. Before this slice,
the router installed no authentication. List/count/bulk endpoints accepted an
optional caller-selected `user_id`, omission selected every row, and
notification-by-id queries and mutations used only the notification id.

The candidate schema stores `notifications.user_id` as `VARCHAR(66)`. The
shared verifier proves `subject == wallet_address`, so the verified wallet
string is the only canonical owner key used by this slice. No wallet-to-UUID
mapping is invented. Existing sample or legacy rows whose `user_id` is not a
canonical wallet (including the startup seed value `demo`) are intentionally
not reachable through owner routes and need a separate audited data migration.

## Enforced boundary

- `GET`/`HEAD /health` is the only anonymous allowlist. The middleware strips
  bearer and spoofable identity headers before dispatch.
- Template list/create/get/delete and notification send require a verified
  token with exactly the admin audience plus a canonical grant matching
  `admin:notifications:manage`. The shared backend grammar accepts its valid
  resource/domain wildcards and rejects arbitrary wildcard placement.
- User list, unread count, mark-all-read, clear-all, get, read, unread, and
  delete require a verified token with exactly the frontend or admin audience.
  The verified wallet is inserted as the principal and used as the owner key.
- Compatibility `user_id` query parameters may be absent or exactly equal to
  the verified wallet. A different or empty value returns 403 before SQL.
- Per-notification reads and mutations include both `id` and `user_id` in their
  SQL predicate. Cross-owner and nonexistent records therefore share the same
  404 behavior after authentication.
- Unknown paths, reserved-path collisions, encoded paths, wrong arity, and
  unapproved methods return 404 before database, template-cache, or SMTP work.
- Production startup constructs the shared verifier before PostgreSQL access.
  The JWKS client disables redirects and uses five-second connect,
  fifteen-second total, and bounded idle-pool timeouts. Production identity
  endpoints must be HTTPS and non-local.

## Verification

```bash
cargo test -p epsx-notification --no-fail-fast --locked
cargo check -p epsx-notification --all-targets --locked
./scripts/migration/verify-service-authorization.sh
./scripts/migration/verify-contract-fixtures.sh
./scripts/migration/verify-permission-grammar.sh
./scripts/migration/test-permission-grammar.sh
git diff --check
```

The eight hermetic tests use a fake token verifier and downstream handlers; no
PostgreSQL, Redis, SMTP, or live JWKS endpoint is contacted. Cases cover the
health allowlist, missing/invalid tokens, exact frontend/admin audiences,
wrong audiences, missing grants, literal and valid canonical wildcard grants,
invalid wildcard placement, spoofed identity/permission headers, cross-owner
selection, owner derivation, strict path arity, reserved collisions, method
drift, and production URL rejection. Boundary denials assert downstream was
not called.

## Residual blockers

- Runtime DDL and seed writes still occur during process startup. Schema
  creation and data migration must move to additive, versioned migrations.
- Startup seeds non-wallet `demo` owner rows. Existing environments need an
  explicit data-quality audit and safe migration decision; this slice does not
  relabel ownership.
- Admin send still accepts target `user_id`/recipient data by design, but the
  delivery workflow has no idempotency key, durable outbox, retry contract, or
  audited validation/abuse controls.
- An absent SMTP configuration currently logs full email bodies and reports
  delivery as successful. Production must fail closed or use an explicit,
  non-production-only mock mode without sensitive-body logging.
- SMTP transport is synchronous inside an async handler, HTML output wraps
  unescaped rendered content, template mutations are not atomic with the
  in-memory Handlebars cache, and template deletion does not invalidate that
  cache.
- No database-backed integration suite proves ownership predicates against
  real rows, transaction behavior, pagination totals, or recovery semantics.
  The protected routes therefore remain `partial` in the authorization
  fixture even though the request boundary is enforced.

The service-authorization fixture retains
`productionReadinessClaim: false`; runtime production readiness is not proven.
