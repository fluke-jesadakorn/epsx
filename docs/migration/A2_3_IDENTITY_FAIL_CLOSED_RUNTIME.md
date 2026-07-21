# A2.3i identity fail-closed direct-service runtime

Status: **the eleven audited route boundaries are executable; identity
functionality and production readiness remain STOP**.

This slice turns the A2.3h identity audit into a hermetic direct-service
boundary. `GET`/`HEAD /health` is the only anonymous live behavior. Unsafe
challenge, SIWE, refresh, and demo routes return `404` before authentication,
body parsing, signature work, Redis, SQL, or a handler. `/auth/me` accepts only
the shared canonical RS256/JWKS access verifier with exactly the frontend or
admin audience, then returns `404` before candidate UUID or persistence
semantics. User routes require the exact admin audience and their existing
literal `admin:users:read/create/update/delete` permission, then return `404`
before selectors, JSON, SQL, or mutation.

Removing the candidate database, Redis, shared-secret JWT, demo, and bootstrap
configuration from the binary is intentional because every persisted identity
route is disabled in this slice. It is not evidence that persistence is
unnecessary, migrated, or production ready. Those inputs must return only with
reviewed additive schemas and proven nonce, session, identity, and account
lifecycle semantics.

## Exact runtime disposition

| Method and path | Executable boundary | Functionality |
|---|---|---|
| `GET`/`HEAD /health` | anonymous; authorization and spoofable identity headers stripped | aligned |
| `POST /api/v1/identity/auth/challenge` | unconditional `404` before body or I/O | blocked |
| `POST /api/v1/identity/auth/siwe` | unconditional `404` before body, signature, or I/O | blocked |
| `POST /api/v1/identity/auth/refresh` | unconditional `404` before body, credential, or I/O | blocked |
| `GET /api/v1/identity/auth/me` | canonical access token; frontend or admin audience; then `404` | blocked |
| `POST /api/v1/identity/auth/demo` | unconditional `404` before body, token, or I/O | blocked |
| `GET /api/v1/identity/users` | admin plus literal `admin:users:read`; then `404` | blocked |
| `POST /api/v1/identity/users` | admin plus literal `admin:users:create`; then `404` | blocked |
| `GET /api/v1/identity/users/{id}` | admin plus literal `admin:users:read`; then `404` | blocked |
| `PUT /api/v1/identity/users/{id}` | admin plus literal `admin:users:update`; then `404` | blocked |
| `DELETE /api/v1/identity/users/{id}` | admin plus literal `admin:users:delete`; then `404` | blocked |

Every wrong method, path, arity, trailing slash, encoded path, repeated slash,
reserved selector, unsafe selector, or unknown route returns `404` before token
verification or dispatch. Public health removes `Authorization` and spoofable
identity headers. Wildcard grants do not satisfy an identity administration
route in this slice; only its literal audited permission does.

## Hermetic evidence

The library tests use a mock access-token verifier and in-memory routers. They
prove public header stripping, strict route classification, malformed bearer
denial, exact audience checks, literal permission checks, wildcard rejection,
and lifecycle denial before malformed JSON or a downstream handler. Production
verifier configuration rejects HTTP and local issuer/JWKS endpoints. No test
contacts a database, Redis, JWKS, service, browser, container, Kubernetes, or a
deployment target.

The integrity gate SHA-pins the exact production `lib.rs` prefix (excluding its
test module), `main.rs`, and `Cargo.toml`. It also inventories all eight Axum
route paths and the sole anonymous classifier arm. Its self-test mutates copied
runtime sources to add an extra router path and a second `Public` arm while the
old anchors remain; both changes must fail integrity. Runtime pin refreshes are
therefore explicit reviewed contract changes rather than surviving-substring
claims.

```bash
cargo test --offline -p epsx-identity --lib
cargo check --offline -p epsx-identity --bin identity
./scripts/migration/verify-a2-3-identity-fail-closed-runtime.sh --mode integrity
./scripts/migration/verify-a2-3-identity-fail-closed-runtime.sh --mode report
./scripts/migration/test-a2-3-identity-fail-closed-runtime.sh
```

`--mode readiness` intentionally exits `3`. The machine-readable contract is
`docs/migration/contracts/a2-3-identity-fail-closed-runtime.json`.

## Remaining STOP boundary

No SIWE challenge or verification flow, refresh issuance or rotation, current
user response, user administration, logout/revocation, or durable identity
behavior is claimed. Before any hidden route can be enabled, the remaining
A2.3h blockers require additive schema ownership, canonical wallet identity,
atomic durable nonce and refresh transitions, exact SIWE binding, response and
error contracts, bounded public ingress, audit/redaction, hermetic concurrency
fixtures, disposable integration, source-shadow comparison, canary, and
rollback rehearsal. Production deployment still requires explicit user
approval after every STOP is closed.
