# A2.3d subscription service authorization

This bounded slice makes the subscription service consume
`epsx-service-auth` directly. It establishes a fail-closed request boundary
before the existing SQL handlers. It does not claim the A9 subscription
lifecycle, payment integration, plan authority, or production-readiness gate
is complete.

## Audited surface and schema decision

The service mounts nine method/path contracts: health, three plan operations,
four owner subscription operations, and one vault configuration read. Before
this slice, none had direct authentication or authorization.

The candidate `subscriptions.user_id` column is a UUID while the shared
verifier proves a wallet string as both subject and wallet address. There is no
trusted wallet-to-UUID mapping in this service. The create path also defaults
new rows directly to `active` without a finalized payment cause. Consequently,
all owner create/list/detail/cancel routes remain fail-closed with 404 before
SQL; this slice does not hash, coerce, or invent an owner identifier.

The candidate plan schema has an `active` flag but no independently reviewed
public/publication flag or canonical public DTO. Anonymous plan reads could
therefore expose a second, incomplete plan authority. Until A9 resolves that
projection, existing plan reads are admin-only rather than public. The vault
handler returns a zero-address placeholder and is likewise blocked before the
handler.

## Enforced boundary

- `GET`/`HEAD /health` is the only anonymous allowlist. Bearer and spoofable
  identity headers are removed before dispatch.
- `GET /api/v1/subscription/plans` and the exact one-segment detail route
  require the exact admin audience plus `admin:plans:read`.
- `POST /api/v1/subscription/plans` requires the exact admin audience plus
  `admin:plans:manage`. Manage-only credentials do not imply read access.
- Canonical resource, domain, and global wildcard grants are interpreted only
  by `epsx-service-auth`; arbitrary wildcard placement is denied.
- Owner subscription routes return 404 before token verification or SQL until
  trusted wallet ownership and verified-payment activation exist.
- `GET /api/v1/subscription/vault/{chain_id}` returns 404 before the canned
  zero-address handler.
- Unknown paths, wrong methods, encoded paths, reserved path segments, and
  wrong arity return 404 before database work.
- Production startup constructs the shared verifier before PostgreSQL access.
  The JWKS client disables redirects and uses five-second connect,
  fifteen-second total, and bounded idle-pool timeouts. Production identity
  endpoints must be HTTPS and non-local.

## Verification

```bash
cargo test -p epsx-subscription --no-fail-fast --locked
cargo check -p epsx-subscription --all-targets --locked
scripts/migration/verify-service-authorization.sh
scripts/migration/verify-subscription-execution.sh --mode integrity
scripts/migration/verify-contract-fixtures.sh
scripts/migration/verify-permission-grammar.sh
git diff --check
```

The eight hermetic service tests use a fake verifier and downstream handler;
they contact no PostgreSQL, Redis, chain, or live JWKS endpoint. Cases cover
missing and invalid tokens, exact audiences, read/manage separation, literal
and canonical wildcard permissions, invalid wildcard placement, spoofed
identity headers, exact public allowlisting, fail-closed owner routes,
zero-vault suppression, and unknown method/path/arity drift.

## A9 status and residual blockers

The A9.0 execution contract in
`docs/migration/contracts/subscription-execution.json` remains authoritative
for lifecycle readiness. All 20 STOP blockers remain blocked. In particular:

- no safe public plan projection, canonical DTO, or single backend plan
  authority exists;
- UUID owner rows cannot be bound to the verified wallet without a trusted
  identity mapping and audited data migration;
- create still means active-before-finality inside the blocked handler;
- there is no immutable payment cause, idempotent activation, lifecycle state
  machine, renewal/expiry worker, outbox, entitlement projection, or
  reconciliation;
- startup still owns schema DDL, and the plan/admin SQL semantics have no
  database-backed integration proof; and
- the HTTP/BFF/UI, shadow, canary, observability, cutover, and rollback gates
  remain unresolved.

For these reasons, plan routes are recorded as `partial`, owner and vault
routes remain `blocked`, and the service authorization fixture retains
`productionReadinessClaim: false`. A9 readiness must continue to exit `3`.
