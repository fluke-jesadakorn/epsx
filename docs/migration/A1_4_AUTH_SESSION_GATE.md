# A1.4 hermetic authentication/session gate

`scripts/migration/verify-auth-session-flow.sh` is the local, executable release
gate for the canonical Rust BFF session contract. It is intentionally narrower
than a live end-to-end test: it runs 71 focused tests with Cargo offline, uses
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

- access tokens are bound to exactly one frontend or admin audience, and a
  wrong audience is rejected;
- RS256 `kid`, JWKS rotation/cache behavior, issuer, expiry/`nbf`, and malformed
  key material fail closed;
- browser JSON and the shared wallet bridge do not expose or store session
  tokens;
- canonical access/refresh cookies are paired, rotated, and cleared, while a
  refresh cookie is never accepted as an access token;
- BFF logout always clears local session cookies, including when its upstream
  is unavailable;
- the admin proxy rejects an unauthenticated request before contacting its
  upstream;
- auth return targets remain same-origin and the admin auth route is public;
- the monolith logout handler selects canonical refresh credentials and is
  wired to `revoke_refresh_token`.

## Still blocked

This gate does **not** claim a real wallet-signature flow, nonce consumption,
durable refresh-token rotation/old-token rejection/logout revocation against
PostgreSQL, or any production issuer, browser, cookie, or routing behavior.
Those require a disposable database-backed integration environment and then a
separately approved production-shaped rehearsal. A passing report therefore
keeps live auth-session and production-readiness contracts blocked.
