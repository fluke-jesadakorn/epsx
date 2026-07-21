# A2.3h identity direct-service authorization audit

Status: **static integrity is executable; identity production readiness remains STOP**.

This package audits the exact eleven `services/identity` route shapes recorded
by A2 against the pinned `origin/development` identity/session authority, the
current A1 hermetic gate, and the shared extracted-service access-token
verifier. It changes no runtime code or existing contract. Source behavior is
evidence, not an automatic production target: A1/A2 fail-closed requirements
take precedence where the source itself is incomplete.

The machine-readable contract is
`docs/migration/contracts/a2-3-identity-authorization.json`. The verifier is
offline and read-only. It never contacts PostgreSQL, Redis, JWKS, a service, a
browser, Docker, Kubernetes, or a deployment target and never executes a
migration.

## Exact route decisions

| Method and path | Intended production boundary | Audit status |
|---|---|---|
| `GET`/`HEAD /health` | Exact anonymous health only; strip credentials and spoofable identity headers | partial |
| `POST /api/v1/identity/auth/challenge` | Public bootstrap only after bounded input/rate and durable exact SIWE challenge issuance | blocked |
| `POST /api/v1/identity/auth/siwe` | Public bootstrap; validate signature and every stored SIWE binding before one atomic nonce-consumption transition | blocked |
| `POST /api/v1/identity/auth/refresh` | Refresh credential only; atomically rotate a durable session bound to its original single BFF audience | blocked |
| `GET /api/v1/identity/auth/me` | Canonical RS256/JWKS access token, exactly one frontend/admin audience, server-derived wallet owner | blocked |
| `POST /api/v1/identity/auth/demo` | Always `404` before body, SQL, or token issuance in production-like builds | blocked |
| `GET /api/v1/identity/users` | Exact admin audience plus `admin:users:read` | blocked |
| `POST /api/v1/identity/users` | Exact admin audience plus `admin:users:create` | blocked |
| `GET /api/v1/identity/users/{id}` | Exact admin audience plus `admin:users:read`; path ID is only a selector | blocked |
| `PUT /api/v1/identity/users/{id}` | Exact admin audience plus `admin:users:update`; backend owns authority changes | blocked |
| `DELETE /api/v1/identity/users/{id}` | Exact admin audience plus `admin:users:delete`; session/dependency effects must be transactional | blocked |

Every other method, path, arity, encoded ambiguity, reserved dynamic segment,
or malformed selector must fail before verification, parsing, Redis, SQL, or
signature work. No new permission name is introduced; the four admin grants
are the ones already recorded in the A2 contract.

## High-signal findings

- The candidate uses a defaultable shared-secret JWT and a noncanonical claim
  shape. It does not provide the A1/A2 RS256 issuer, `kid`/JWKS, exact single
  audience, scope, and `sub == wallet_address` access-token contract.
- Access and refresh JWTs use the same claims and verifier. There is no durable
  refresh session, atomic rotation, old-token rejection, reuse response, or
  logout/account revocation in the candidate.
- SIWE performs Redis `GET` then `DEL`; deletion is non-atomic and happens
  before signature verification. Invalid attempts can consume a legitimate
  challenge. Full stored-message/address/domain/URI/chain/time binding remains
  unproven.
- `GET /users/{id}` has no authorization check. Other user routes check a
  database `admin` role rather than exact admin audience plus granular
  permission. Startup configuration can mutate an admin role, and user writes
  accept caller-selected role arrays.
- Candidate UUID subjects and `{id}` selectors have not been reconciled with
  the pinned wallet-address identity authority. The service also creates its
  authority table at startup rather than through reviewed additive migration.
- Challenge/SIWE body size, request rate, concurrency, signature cost, typed
  errors, redaction, audit, source envelope/status parity, and account lifecycle
  semantics are not locked.

## Offline verification

```bash
./scripts/migration/verify-a2-3-identity-authorization.sh --mode integrity
./scripts/migration/verify-a2-3-identity-authorization.sh --mode report
./scripts/migration/test-a2-3-identity-authorization.sh
```

Integrity verifies the pinned source commit/blobs/anchors, current target
anchors, the exact A2 route IDs/methods/paths/permissions, conservative status
counts, ten required invariants, twenty STOP blockers, and the twelve-step
execution order. `--mode readiness` intentionally exits `3` while all twenty
STOP blockers remain. A passing integrity run never means production ready.

## Execution boundary

The required order is: freeze identity/DTO/audience contracts; add strict
routing and public bounds; adopt the canonical issuer/verifier; implement
atomic durable nonce and refresh lifecycles; enforce granular admin policy;
migrate/reconcile data additively; lock API/audit/account semantics; run
hermetic concurrency and rotation fixtures; then separately approve disposable
database/Redis/JWKS/BFF integration, source-shadow comparison, canary, and
rollback rehearsal. Production deployment still requires an explicit user
instruction after every STOP is closed.
