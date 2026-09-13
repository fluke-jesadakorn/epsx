# A2.3a analytics service authorization

This bounded slice makes analytics the first extracted service to consume
`epsx-service-auth` directly. It establishes a request boundary before handler
or database access; it does not claim analytics data semantics or overall A2
production readiness are complete.

## Enforced boundary

- `GET`/`HEAD /health` is the only anonymous allowlist.
- `POST /api/v1/analytics/track` requires a verified canonical access token
  with exactly the frontend or admin audience.
- `GET /api/v1/analytics/events`, `metrics/{metric}`, and `revenue` require the
  exact admin audience plus the canonical backend grant
  `admin:analytics:view`.
- `GET /api/v1/analytics/admin/audit-log` separately requires the exact admin
  audience plus canonical `admin:audit:read`. It reads only the canonical unified
  table through a fixed 20-row category-bound keyset contract and projects no
  actor/target identity, network/device data, state snapshots, or metadata.
- Both Prometheus paths return 404 before handler/store access until an
  authenticated internal-service identity contract exists.
- Unknown paths and unapproved methods are denied before handler/store access.
- Authentication uses `epsx-service-auth::authenticate_headers`; inbound
  identity, wallet, role, scope, and permission headers are stripped and never
  establish a principal.

The router accepts an `AnalyticsStore` trait and an `AccessTokenVerifier`, so
authorization behavior is tested without PostgreSQL. Production startup uses a
single redirect-disabled HTTP client with five-second connect and fifteen-second
total timeouts. `EPSX_ENV=production` requires HTTPS, non-local issuer and JWKS
URLs through the shared verifier configuration.

## Verification

```bash
cargo test -p epsx-analytics --no-fail-fast --locked
cargo check -p epsx-analytics --all-targets --locked
./scripts/migration/verify-service-authorization.sh
./scripts/migration/verify-contract-fixtures.sh
./scripts/migration/verify-permission-grammar.sh
./scripts/migration/test-permission-grammar.sh
git diff --check
```

The hermetic cases cover anonymous access, invalid and valid-but-unapproved
audiences, spoofed identity headers, missing permission, frontend tokens
carrying an admin grant, canonical global/resource wildcard grants, unrelated
wildcard rejection, granular analytics and audit admin success, strict
audit query/cursor parsing, redacted audit SELECT fields, internal endpoints, unknown routes,
unapproved methods, and production URL rejection. Every denial case asserts
the fake store was not called.

## Residual blockers

- Event attribution is not solved. Canonical subjects are wallet-address
  strings, while the existing optional `events.user_id` column is UUID. The
  request `user_id` remains accepted for compatibility but is cleared before
  the store, and SQL deliberately persists `NULL`. A schema/domain migration
  must define a wallet-safe attribution model before the track route can be
  marked fully aligned; unique-user analytics intentionally loses attribution
  in the interim.
- Internal observability callers have no service identity yet, so Prometheus
  remains unavailable rather than accepting user/admin credentials.
- Database queries, event semantics, retention, aggregation correctness, and
  revenue meaning remain A12 concerns. The protected reads stay `partial` in
  the authorization fixture rather than claiming broader production readiness.
- The audit table shape is checked at startup and static query/DTO tests pass,
  but the existing migration has not been proven through an isolated analytics
  database/service run. Authenticated browser behavior, reverse continuation,
  source-compatible search/date filters, field-specific identity/detail reads,
  and authorized audited server export also remain open.
- Other extracted services have not adopted the shared boundary. Direct-service
  isolation and ownership enforcement remain open across the matrix.

The service-authorization fixture therefore retains
`productionReadinessClaim: false` and reports runtime readiness as not proven.
