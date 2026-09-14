# A2.3f pay service authorization

This bounded slice makes the pay service consume `epsx-service-auth`
directly. It enforces exact method/path policy, canonical RS256/JWKS access
tokens, owner predicates, and granular payment administration before the
existing handlers. It does not claim the A6 payment lifecycle, chain,
durability, routing, or production-readiness gates are complete.

## Audited route decisions

The service mounts 22 method/path contracts.

- `GET`/`HEAD /health` is anonymous. The middleware removes bearer and
  spoofable identity headers before dispatch.
- `GET /api/v1/pay/links/{slug}` remains anonymous because the pinned source
  exposes a public slug lookup. The candidate projection contains only opaque
  link/intent identifiers, slug, use bounds/timestamps, and its relative URL;
  it excludes payer, payee, amount, token, chain, status, transaction hash,
  and description. Missing links return 404. Expired or exhausted links return
  410 and are never returned as usable.
- Intent list/detail, escrow list/detail, and wallet history require a verified
  token with the exact frontend or admin audience. A principal extension
  carries the wallet; caller headers are never identity sources.
- Owner list/detail/history SQL always filters by the lower-cased verified
  wallet. A compatibility `payer` query or history path may agree
  case-insensitively but cannot select another wallet. Foreign and missing
  resources share 404 behavior.
- `GET /api/v1/admin/pay/intents` requires the exact admin audience plus
  `admin:payments:view` before its read query. Escrow dispute resolution and
  all three force-operation shapes require the exact admin audience plus
  `admin:payments:manage`, but even a valid operator receives 404 before the
  handler. Valid resource/domain/global wildcard grants are interpreted only
  by the shared canonical grammar; invalid wildcard placement is denied.
- Intent create/confirm/cancel, escrow release/refund/dispute, and pay-link
  create/redeem return 404 before authentication or SQL. All currently trust
  caller-controlled financial coordinates or perform DB-only changes without
  the A6 idempotency, transaction, audit, and chain transition proofs. The
  service does not make flawed lifecycle operations newly reachable merely by
  authenticating their caller.
- Escrow confirm-deposit and the on-chain webhook return 404 before handler
  execution. The existing body HMAC has no approved service/provider identity,
  key ID, signed timestamp, replay window, or atomic inbox-transition-outbox
  transaction. This slice does not promote it to an internal trust boundary.
- Unknown paths, unapproved methods, wrong arity, empty/trailing segments,
  encoded separators, backslashes, and dot segments return 404 before handler,
  database, or chain work.

## Verification

```bash
cargo test -p epsx-pay-svc --all-targets --no-fail-fast --locked
cargo check -p epsx-pay-svc --all-targets --locked
cargo test -p epsx-service-auth --lib --locked
./scripts/migration/verify-service-authorization.sh
./scripts/migration/verify-contract-fixtures.sh
./scripts/migration/verify-permission-grammar.sh
./scripts/migration/test-permission-grammar.sh
git diff --check
```

The 11 hermetic pay tests contact no PostgreSQL, live JWKS endpoint, chain,
webhook provider, or deployed service. Nine boundary tests cover the public
  allowlist, missing/invalid tokens, exact audiences, read/manage separation,
canonical and invalid wildcard grants, spoofed identity headers, owner
derivation, forced-closed financial/internal routes, strict path handling, and
production URL rejection. Two pay-link tests lock usable-link semantics and
the anonymous projection's excluded financial/identity fields.

## Residual A6 blockers

- Intent creation still lacks a server-authoritative plan/price/receiver/token
  quote, idempotency key, and declared write authority; it remains blocked.
- Confirmation and deposit paths still lack receipt/Transfer verification,
  configured chain/contract checks, finality, reorg handling, and atomic
  state/entitlement changes; they remain blocked.
- Runtime DDL, weak constraints, split writes, missing inbox/outbox/audit
  transactions, and absent migration/backfill/reconciliation/rollback remain.
- Escrow, link, resolve, cancel, and force handlers still implement unsafe
  database-only state changes, but the boundary prevents every one from being
  invoked. They remain blocked pending the full A6 lifecycle.
- The existing webhook HMAC, event-id-only dedupe, and non-atomic state update
  are not a sufficient internal identity or replay contract; the route remains
  blocked.
- The pay BFF/gateway prefixes and bodies still drift, the public hostname
  bypasses the intended BFF, and the checkout success UI is not driven by a
  verified terminal backend state.
- There is no database-backed ownership/transition integration suite, chain
  fixture, replay/reorg/recovery proof, shadow comparison, canary, or reviewed
  cutover/rollback evidence.

All A6 STOP blockers remain authoritative. This authorization slice neither
executes migrations nor changes production readiness.
