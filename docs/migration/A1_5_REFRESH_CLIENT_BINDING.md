# A1.5 Refresh-Token Client Binding

Status: **compiled and hermetically verified; PostgreSQL and production
readiness STOP**.

The canonical monolith previously validated that a requested refresh client was
one of `epsx-frontend` or `epsx-admin`, but its durable row did not store the
original client. A holder could therefore submit one valid surface token with
the other valid client and receive a JWT for that caller-selected audience.
The two BFFs already sent fixed clients; the direct monolith endpoint was the
bypass.

## Implemented boundary

- `20260723000000_bind_refresh_tokens_to_client` adds nullable
  `openid_refresh_tokens.client_id VARCHAR(32)` and `family_id UUID`, plus a
  validated client allowed-value check and family lookup index. Catalog guards
  require both columns to be nullable, default-free, non-identity, and
  non-generated. Any pre-existing same-named constraint is refused for explicit
  reconciliation instead of guessed equivalent. The migration changes no row.
  Its reverse deliberately raises because dropping either binding would restore
  cross-client rotation or unsafe logout races.
- Both Diesel schemas carry `Nullable<Varchar>` client and `Nullable<Uuid>`
  family fields. Nullability is a rolling compatibility state, not permission to
  issue unbound tokens.
- Initial issuance writes the exact validated client and a fresh per-login
  family. The follow-on A1.6 runtime issues an opaque credential while storing
  only its keyed digest. Refresh preflight rejects unknown, expired, revoked,
  cross-client, missing-family, and legacy-`NULL` rows before loading
  profile/permission data.
- Rotation first takes a transaction-scoped PostgreSQL advisory lock keyed by
  the stored family, then conditionally matches token ID, preflight wallet,
  stored client, family, active state, and expiry before atomically setting the
  predecessor tuple to `is_revoked = true` and `consumed_at = now`. The successor
  copies the returned client/family; JWTs use the preflight stored client rather
  than the request DTO. Consume and successor insert remain one transaction. A
  rare 64-bit hash collision only serializes unrelated families; it cannot
  weaken mutual exclusion.
- Permission/profile reads, replacement generation, and both fallible JWT
  signatures finish before the conditional consume. A signing failure therefore
  leaves the predecessor active and publishes no successor; a losing concurrent
  request returns neither its prebuilt JWTs nor its candidate refresh value.
- Logout resolves the presented token's family, takes the same transaction-
  scoped advisory lock as rotation, and revokes only active rows in that
  lineage. A stale token can close its own descendants but cannot affect a later
  independent login for the same wallet/client. A legacy `NULL`-family token
  revokes only its exact row; no family is inferred.
- Successors retain the chain's original `created_at`, which is the JWT
  `auth_time`; refresh no longer advances authentication time on every rotation.
- The HTTP refresh request requires an explicit client. Invalid credentials are
  generic `401`, unsupported clients are `400`, dependency failure is `503`,
  and internal failure is `500`. Refresh responses use `Cache-Control:
  no-store`.
- Frontend and admin local cookies are named separately. Cookies do not isolate
  `localhost:3000` from `localhost:3001`; the old ambiguous names are therefore
  cleared but never read as refresh credentials. Production `__Host-` cookies
  remain naturally bound to their separate hosts. BFF refresh clears the browser
  session only for canonical `401`; upstream `400`, `408`, `429`, `5xx`, and
  transport failures preserve it for retry/diagnosis.

## Legacy policy and rollout order

Existing rows do not contain enough evidence to reconstruct their original
client or family. They must never be backfilled, inferred from wallet
permissions, grouped by wallet/client, or claimed by the first refresh request.
Runtime exact matching therefore leaves them unchanged and forces fresh
authentication.

The A1.5 client/family expansion's safe rolling sequence is:

1. apply only the nullable expansion;
2. deploy binding-aware code that writes every new client/family and rejects
   either `NULL` during refresh;
3. prove every old writer is gone and reconcile active legacy-null rows;
4. in a separately reviewed migration, revoke remaining active null rows and
   enforce that active rows are non-null.

The fourth step is intentionally absent from the runnable chain. Shipping it in
the same migration batch could break an old replica still issuing null rows.
No migration in this repository's active core root is currently authorized to
run because two baseline directories share Diesel version `00000000000001`.
The follow-on A1.6 digest/storage-version cutover is stricter: it requires a
drained maintenance window, no mixed old/new writers, and forced
reauthentication rather than a rolling raw-token compatibility phase.

## Hermetic evidence

```bash
./scripts/migration/verify-refresh-client-binding.sh
./scripts/migration/test-refresh-client-binding.sh
./scripts/migration/verify-auth-session-flow.sh
```

The binding verifier exact-pins both migration checksums, ten stable-ID
invariants, 49 file/anchor pairs, both schemas, fixed BFF clients, local
cookie/runtime-fixture isolation, sign-before-consume ordering, family-serialized
rotation/logout, and ten stable-ID residual STOP claims. Its readiness mode
intentionally exits `3`:

```bash
./scripts/migration/verify-refresh-client-binding.sh --mode readiness
```

The A1.4 auth/session gate now covers 82 focused tests plus its two fixture
checks with Cargo offline and loopback-only mocks. These prove compiled query
shape and database-free state classification; they do not prove PostgreSQL.

## Remaining STOP evidence

An explicitly authorized, isolated PostgreSQL harness must still prove:

- expansion from each observed deployed history without row loss;
- attempted forward-only down migration fails atomically without schema/data
  change;
- exact initial client/family storage and same-family successor binding;
- cross-client and legacy-null rejection with the predecessor unchanged;
- two-connection single-winner rotation and old-token rejection;
- keyed-digest lookup, consumed/revoked/replay transitions, and automatic
  active-descendant revocation after detected reuse;
- rollback when successor insertion is forced to fail;
- logout-first, rotation-first, stale-token logout, and distinct-family
  isolation under two PostgreSQL connections;
- restart persistence and the later active-null reconciliation/enforcement;
- A1.6 migration application, drained forced-reauthentication cutover, legacy
  plaintext reconciliation, and persistent production key rotation/retirement.

This bounded family identifier serializes rotation and scopes logout, but it is
not the dormant identity service's full refresh-session model. A1.6 now issues
opaque credentials, stores keyed digests, distinguishes consumed and explicitly
revoked state, records replay, and revokes active descendants in code. That is
compiled/static evidence, not disposable-PostgreSQL, concurrency, cutover,
plaintext-scrub, restart, or production key-lifecycle proof. Already-issued
access tokens remain valid until expiry. The dormant `services/identity`
family/session schema and its `404` routes are unchanged; A1.5 does not claim
A3.8 runtime integration.
