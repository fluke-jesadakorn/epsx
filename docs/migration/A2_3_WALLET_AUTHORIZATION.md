# A2.3g wallet direct-service authorization

Status: **fail-closed service boundary implemented; wallet production readiness remains STOP**.

This package changes only the candidate `services/wallet` HTTP boundary, its
owner-safe account reads, and local evidence. It does not execute or change
database migrations, contact PostgreSQL, RPC providers, chains, or live JWKS
endpoints, alter key custody, submit transactions, change Kubernetes, or
authorize deployment.

## Locked route decisions

| Method and path | Boundary decision | Reason |
|---|---|---|
| `GET`/`HEAD /health` | anonymous | The exact health surface strips bearer and spoofable identity headers. |
| `GET /api/v1/wallet/accounts` | verified frontend/admin owner | The handler derives a canonical EVM address from `VerifiedPrincipal`, binds `lower(address) = $1`, and selects only address, chain, label, and role. |
| `GET /api/v1/wallet/accounts/{address}` | verified frontend/admin owner | The path may only agree with the canonical token wallet. Foreign, malformed, and missing accounts are hidden; SQL still binds the verified wallet and never selects `encrypted_pk`. |
| `POST /api/v1/wallet/verify-message` | anonymous, maximum 8192-byte JSON body | This is bounded, read-only Alloy signature recovery. It performs no SQL, provider call, key custody, signing, broadcast, or state mutation. Credentials and spoofable identity headers are stripped. |
| `POST /api/v1/wallet/accounts` | `404` before auth/body/handler | The handler accepts caller-selected address, role, or raw private key and can generate keys; provisioning and custody policy are absent. |
| `GET /api/v1/wallet/balance/{chain}/{address}` | `404` before handler/provider | The mutable shared provider can race across chain requests and failures are silently reported as zero balance. |
| `POST /api/v1/wallet/send` | `404` before auth/body/handler | Raw private keys, a database nonce, fabricated number-derived hash, missing broadcast, and absent replay/idempotency semantics are unsafe. |
| `POST /api/v1/wallet/sign-message` | `404` before auth/body/handler | A caller-supplied raw private key is neither ownership nor a production signing/custody contract. |
| `POST /api/v1/wallet/estimate-gas` | `404` before body/handler/provider | The handler ignores material request fields, always returns a 21000 gas limit, and substitutes canned fees on provider failure. |
| every other method/path/arity, encoded/ambiguous path, or reserved dynamic segment | `404` before verifier/handler | The boundary is a strict normalized-path allowlist. |

Account ownership comes only from the shared RS256/JWKS verifier. Exact
`epsx-frontend` and `epsx-admin` audiences are accepted for these owner reads;
caller body, query, path, or identity-looking headers can never choose a
different account. This slice introduces no wallet admin operation or new
permission: no existing canonical backend permission proves safe custody,
provisioning, signing, or transaction authority.

## Verification

Run only locked, hermetic evidence:

```bash
cargo test --locked -p epsx-wallet --all-targets --no-fail-fast
cargo check --locked -p epsx-wallet --all-targets
cargo test --locked -p epsx-service-auth --lib
./scripts/migration/verify-service-authorization.sh
./scripts/migration/verify-permission-grammar.sh --mode integrity
./scripts/migration/test-permission-grammar.sh
./scripts/migration/verify-contract-fixtures.sh
git diff --check
```

The nine wallet tests are hermetic. They cover the two anonymous shapes,
bounded public JSON, exact owner audiences, invalid token-wallet format,
canonical/cross-owner address behavior, spoofed identity headers, unsafe-route
pre-handler closure, strict methods/arity/path normalization, and production
HTTPS verifier configuration. These tests do not prove live database rows,
RPC truth, chain state, custody, broadcasting, or deployed routing.

## Residual STOP blockers

- Runtime `CREATE TABLE IF NOT EXISTS` statements must move to reviewed,
  additive migrations with constraints, backfill, reconciliation, and rollback.
- Account creation needs an explicit provisioning model, server-authoritative
  roles, safe import/generation policy, durable audit, and approved custody.
- Key storage/encryption, KMS/HSM ownership, rotation, export policy, recovery,
  signer authorization, and secrets observability are absent.
- Transaction preparation needs canonical RPC nonce handling, EIP-1559 gas,
  typed request validation, simulation, signing/broadcast responsibility,
  idempotency, replay protection, receipt/finality/reorg tracking, and truthful
  response states. The current handler remains unreachable.
- Balance and estimation need per-chain provider isolation, supported-chain
  policy, timeout/error/freshness semantics, and deterministic fixtures before
  either can become public.
- Database-backed owner integration, live JWKS smoke, chain fixtures, shadow
  comparison, canary evidence, and cutover/rollback approval remain absent.

This A2.3g boundary narrows exposure only. It does not make the wallet service
production-ready and does not authorize ingress or deployment.
