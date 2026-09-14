# A2 gateway authorization slice

This slice replaces the gateway's optional shared-secret JWT enrichment with a
deny-by-default authorization boundary. It is a gateway milestone, not an
overall service-authorization or production-readiness claim.

The verifier foundation now lives in the dependency-light
`shared/rust/epsx-service-auth` crate. The gateway consumes that shared crate;
later service middleware can call `authenticate_headers` and place the returned
`VerifiedPrincipal` into request extensions without reading caller-supplied
identity, wallet, role, or permission headers. This extraction does not claim
that any direct service has adopted the boundary yet.

`services/gateway/src/auth.rs` remains a re-export facade because the locked A2
fixture still resolves that path in this bounded slice. A later fixture update
should repoint implementation evidence to the shared crate; no verifier logic
is duplicated in the facade.

## Implemented boundary

- Access tokens are accepted only when they have an RS256 header, a non-empty
  `kid` present in the bounded JWKS cache, the configured issuer, exactly one
  audience (`epsx-frontend` or `epsx-admin`), a valid `exp`/`nbf`/`iat`, and a
  non-empty subject equal to `wallet_address`.
- Production mode (`EPSX_ENV=production`) requires HTTPS issuer/JWKS URLs and
  rejects localhost or loopback identity configuration. All upstream bases are
  validated as credential-free HTTP(S) origins with no path, query, or
  fragment; production mode rejects implicit localhost defaults.
- A method-and-normalized-path table classifies requests as `Public`,
  `CredentialExchange`, `Authenticated`, `Permission`, `InternalOnly`, or
  `Blocked`. Anything not in the table is blocked before upstream I/O.
- Refresh is a credential-exchange route: it does not require a valid access
  JWT. Challenge, SIWE, refresh, and public reads remove any caller-supplied
  bearer token before forwarding.
- Protected routes require a verified bearer. Granular operator routes also
  require the `epsx-admin` audience and the exact canonical backend permission
  rules. Two-segment UI permission strings and arbitrary wildcard positions
  cannot become gateway grants.
- The JWKS document is capped at 64 KiB and eight RS256 signing keys. A
  singleflight refresh lock does not hold the cache lock during network I/O,
  and unknown-key refreshes are throttled to one per 30-second window. The
  shared HTTP client has connect/total timeouts and disables redirects.
- Spoofable identity/forwarding headers, connection-nominated headers, cookies,
  and hop-by-hop headers are removed. Verified Authorization values are
  forwarded unchanged only on protected routes. Request bodies are capped at
  1 MiB and upstream response bodies at 8 MiB. Request IDs are validated and
  bounded. Candidate service `Set-Cookie` responses are not exposed through
  the public API gateway.
- Internal Prometheus surfaces return 404. Indexer sync is narrowed to POST at
  the gateway. The legacy `/api/v1/payment/*` prefix and the currently
  unrouted `/api/v1/pay/*` surface both remain denied; this slice does not
  silently rewrite financial mutations.

## Required runtime configuration

`OIDC_ISSUER` is required. `OIDC_JWKS_URL` is optional and otherwise resolves
to `<OIDC_ISSUER>/.well-known/jwks.json`. `EPSX_ENV` is explicitly
`development` or `production`. Service URL flags keep local development
defaults, but those defaults are rejected when production mode is selected.

## Verification

```bash
cargo test -p epsx-gateway --no-fail-fast
cargo test -p epsx-service-auth --no-fail-fast
cargo check -p epsx-gateway --all-targets --locked
cargo check -p epsx-service-auth --all-targets --locked
./scripts/migration/verify-service-authorization.sh
```

The table tests cover every locked non-health downstream route currently
reachable through the gateway plus public, refresh, authenticated, granular
admin, internal, unknown-method/path, wrong-audience, expired, wrong-issuer,
wrong-algorithm, future-`iat`, duplicate-Authorization, header-stripping,
body-cap, request-ID, upstream-cookie, and payment-prefix cases. Denial tests
assert the upstream was not contacted.

## Residual blockers

- Direct access to candidate services bypasses this gateway boundary. Those
  routers still need equivalent canonical authentication or private network
  enforcement before production exposure.
- `epsx-service-auth` provides the verifier and service-facing header API only;
  identity, pay, wallet, subscription, notification, analytics, and indexer
  routers have not been wired to it in this foundation slice.
- Gateway `Authenticated` classification establishes a verified principal but
  cannot prove record ownership. Wallet, pay, subscription, notification, and
  other owner-only handlers must compare the verified subject/wallet to stored
  or derived ownership in the backend service.
- Internal webhook and metrics endpoints need authenticated service identity or
  their locked handler-level signature contract. External user/admin tokens do
  not satisfy `InternalOnly`.
- The candidate identity service has no logout route in the locked inventory,
  so none was invented here. The canonical BFF/backend logout and refresh
  revocation work remains the session contract.
- `/api/v1/payment/*` versus `/api/v1/pay/*` requires the A6 payment ownership
  and handler-parity decision. Both are intentionally unavailable here.
- The fixture therefore keeps `productionReadinessClaim: false`; downstream
  ownership, direct-service isolation, and remaining blocked matrix entries
  must be resolved before A2 can be considered complete.
