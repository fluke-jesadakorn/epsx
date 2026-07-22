# A1.4 hermetic authentication/session gate

`scripts/migration/verify-auth-session-flow.sh` is the local, executable release
gate for the canonical Rust BFF session contract. It is intentionally narrower
than a live end-to-end test: it runs 82 focused tests with Cargo offline, uses
only loopback mock HTTP servers created by those tests, and then runs the route
and API-contract fixture checks.

The machine-readable case/evidence manifest is
`docs/migration/contracts/auth-session-gate.json`. A successful run writes a
deterministic derived report to
`target/migration/auth-session-gate-report.json`.

## Run

```bash
./scripts/migration/verify-auth-session-flow.sh
```

The script accepts no arguments or URLs. It refuses production-looking
environment markers, production-looking application URLs, and any configured
PostgreSQL or Redis URL. It unsets service URLs and proxies before executing the
tests and sets `CARGO_NET_OFFLINE=true`.

## Proven locally

- initial SIWE issuance binds each access token to exactly the requested
  frontend or admin audience, and both BFFs reject wrong or multiple audiences;
- RS256 `kid`, JWKS rotation/cache behavior, issuer, expiry/`nbf`, and malformed
  key material fail closed;
- browser JSON and the shared wallet bridge do not expose or store session
  tokens;
- canonical access/refresh cookies are paired, rotated, and cleared, while a
  refresh cookie is never accepted as an access token;
- the canonical monolith implementation stores and conditionally matches an
  exact frontend/admin refresh client and per-login family, copies both into
  its successor, uses the stored client for JWT audience, rejects
  legacy-null/cross-client state in the pure fail-closed model, pre-signs the
  candidate response before family-serialized conditional consumption, and
  requires the HTTP client explicitly;
- local frontend/admin cookies are client-specific because browser cookies do
  not isolate localhost ports; the ambiguous old local names are clearing-only,
  runtime fixtures use the scoped names, and only upstream `401` clears refresh
  state while retryable/configuration failures preserve it;
- BFF logout always clears local session cookies, including when its upstream
  is unavailable; canonical rotation/logout share a transaction-scoped
  per-login family advisory lock and logout revokes only that lineage;
- the admin proxy rejects an unauthenticated request before contacting its
  upstream;
- auth return targets remain same-origin and the admin auth route is public;
- the monolith logout handler selects canonical refresh credentials and is
  wired to `revoke_refresh_token`.

## Still blocked

This gate does **not** claim a real wallet-signature flow, nonce consumption,
durable refresh-token rotation/old-token rejection/logout revocation against
PostgreSQL, or any production issuer, browser, cookie, or routing behavior. The
client-binding implementation and additive nullable migration are present, but
the active core root still contains a duplicate baseline version and no
authorized disposable PostgreSQL run has proved migration application,
cross-client non-consumption, legacy-row preservation/cutover, concurrent
single-winner rotation, rollback, or restart persistence. Legacy `NULL` rows
are deliberately never claimed or guessed and must force fresh authentication.
Raw UUID refresh credentials, consumed-versus-revoked state, automatic
descendant revocation on replay, and two-connection family-lock ordering proof
also remain open. A passing report therefore keeps live auth-session and
production-readiness contracts blocked.
