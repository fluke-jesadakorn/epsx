# A3.8 Identity Lifecycle Schema

Status: **partial schema-only remediation; identity routes remain disabled;
production readiness STOP**.

A3.8 adds one ordered, additive SQL file for the minimum durable identity
lifecycle boundary. It does not wire a migration runner, database pool,
repository, handler, SIWE verifier, refresh flow, or route. A2.3i remains the
executable runtime authority: challenge, SIWE, refresh, and demo return `404`
before body parsing or I/O, while authenticated identity/user shapes stop at
their audited authorization boundary and return `404` before persistence.

## Grounded source decision

The contract pins nine exact Git blobs across three immutable revisions:

- `origin/development` at
  `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` supplies the established
  lowercase wallet identity, SIWE challenge, and refresh lifecycle evidence.
  Its active baseline makes `wallet_users.wallet_address` the primary
  identity, stores raw nonce/message material, and stores an opaque refresh
  token ID. Its later nonce migration permits more than one challenge per
  wallet. Its token service conditionally revokes an old token and inserts a
  successor inside one transaction, but still looks up the presented token as
  stored data. Its SIWE construction and the candidate both parse chain IDs as
  Rust `u64`, which fixes the current numeric upper bound at
  `18446744073709551615`. The archived `sessions` design stored raw access and
  refresh tokens and is evidence of a design that must not return.
- The immutable A2.3h candidate snapshot at
  `0cdd7ba1967d52e299000b7290873cd4d19dfd09` supplies the UUID-subject/user
  selector shape. It also demonstrates why the old candidate is not a target:
  startup DDL, Redis `GET` then `DEL` before SIWE verification, shared access
  and refresh JWT verification, and role arrays mixed into identity rows.
- The A2.3i fail-closed runtime at
  `f5aa5b393cc0856a1ab2cba42746daa05b8c25c1` proves that lifecycle routes are
  hidden while persistence semantics are unresolved.

The verifier also reproduces the complete 22-file lexical inventory of
development SQL that creates, alters, or drops `users`, `wallet_users`,
`web3_auth_nonces`, `openid_refresh_tokens`, or `sessions`. This includes
archived history and commented Diesel bootstrap templates. It is a source
inventory, not permission to apply any historical destructive migration.

## Ordered additive migration

`services/identity/migrations/20260722000000_create_identity_lifecycle.sql`
is exactly 6,417 bytes with SHA-256
`6415b7621d424b639e1f4692c924d4f42539fbf810d774024bc8bbbd152d008c`.
It contains ten statements: four `CREATE TABLE IF NOT EXISTS` statements
and six guarded index statements. It contains no `ALTER`,
`DROP`, `TRUNCATE`, data mutation, cascade, extension/schema creation, or
transaction control. Transaction ownership is reserved for a future reviewed
runner.

All UUIDs are application generated. This avoids silently requiring
`uuid-ossp` or `pgcrypto` in the identity database.

### `public.identity_users`

The table maps one immutable UUID subject to one unique, normalized lowercase
EVM wallet address. Timestamps cover record creation, update, and last
authentication. No role, permission, plan, tier, subscription, or frontend
authorization field exists. This bridges the candidate UUID selector shape to
development's wallet-address authority without duplicating backend business
policy.

The new table does not rename, overwrite, or auto-adopt either historical
`users` or core `wallet_users`. Mapping and backfill therefore remain explicit
STOP work.

`IF NOT EXISTS` is deliberately only a name-idempotence guard: PostgreSQL
skips an already named relation without
proving its columns, constraints, indexes, or semantics match this contract.
There is no catalog compatibility probe or migration-version adoption ledger,
so existing-relation adoption remains STOP.

### `public.identity_siwe_challenges`

Each challenge records an application-generated ID, normalized wallet, exact
`epsx-frontend` or `epsx-admin` client, canonical decimal chain ID, normalized
domain, 32-byte nonce digest, 32-byte complete-message digest, issuance/expiry,
and nullable `consumed_at`. Raw nonce and raw message columns are absent. No
digest algorithm is claimed by the schema; byte width alone cannot prove one.
The nonce digest is unique, and an active lookup index covers
`(wallet_address, client_id, nonce_hash)` where `consumed_at IS NULL`.

The chain constraint uses a `CASE` guard before casting to PostgreSQL
`NUMERIC`. It accepts canonical decimal strings from zero through the exact
Rust `u64` maximum and rejects negative, signed, padded, non-decimal, overlong,
or overflowing values without attempting to cast malformed input.

The required runtime order is deliberately not implemented in this package:

1. issue and store the challenge for exactly one validated frontend/admin
   client;
2. load the unexpired, unconsumed record and compare every stored binding,
   including the stored client;
3. perform complete SIWE parsing and cryptographic verification;
4. only after successful verification, conditionally set `consumed_at` where
   the same digest and stored client are still unconsumed and unexpired;
5. accept exactly one returned row and issue tokens only for the client
   returned by that transition, never a caller-substituted client.

That conditional transition lets concurrent valid attempts produce at most
one accepted consume. The schema shape does not itself prove the ordering or
concurrency behavior; those remain STOP blockers. A challenge issued for one
client must fail verification, consume, and token issuance when replayed for
the other client even if every other binding matches. The offline self-test
locks this cross-client replay requirement while explicitly recording that no
runtime replay fixture has passed.

### `public.identity_refresh_families`

Each refresh family owns one immutable `(family_id, user_id, client_id)` key.
The user reference is restrictive, the client is exactly frontend or admin,
and optional family revocation cannot predate family creation. This separate
owner relation prevents two roots from reusing one family ID for different
users or clients without blocking legitimate descendants in the same family.
A partial unique index additionally permits at most one root session per
family while leaving every non-root rotation descendant unaffected.

### `public.identity_refresh_sessions`

Refresh state stores a 32-byte token digest, a non-secret hash-key ID, hash
version, exact original BFF client (`epsx-frontend` or `epsx-admin`), user,
family, parent, generation, expiry, consume, and revocation timestamps.
There is no raw refresh token, access token, JWT, signing secret, password, or
signature column. `hash_key_id` and `hash_version` support runtime key/version
selection, but the schema neither names nor proves a digest algorithm.

The user, family, and parent foreign keys use `ON DELETE RESTRICT`; no lifecycle
evidence is silently cascaded away. Every session references the family owner
with the exact same user and client. Every child references a parent-side
composite unique key containing session, user, family, and client, so lineage
cannot cross any of those boundaries. Self-parenting is rejected and
`parent_session_id` is unique, so one session can have at most one committed
successor.

The required runtime rotation is a transaction that conditionally consumes
one active, unexpired digest and inserts one child with the same user, client,
and family. A failed child insert must roll back the consume. Roots are
declaratively generation zero and descendants are positive, but the schema
does not prove `child.generation = parent.generation + 1`; that exact increment
must be checked in the same audited transaction and remains a concurrency
STOP. Family indexing supports later family revocation after reuse detection,
but reuse policy and SQL are not yet implemented.

Rotation and family revocation must also share one atomic row-locking or
serializable boundary. Rotation must lock the corresponding
`identity_refresh_families` row, require `revoked_at IS NULL`, and ensure a
concurrent family revocation wins by failing the rotation before the parent is
consumed or a child is inserted. The schema alone cannot enforce or prove that
race ordering. A specific revoke-versus-rotate concurrency fixture remains
STOP, and no runtime proof is claimed.

## Offline evidence

```bash
scripts/migration/verify-a3-8-identity-lifecycle-schema.sh --mode integrity
scripts/migration/verify-a3-8-identity-lifecycle-schema.sh --mode report
scripts/migration/test-a3-8-identity-lifecycle-schema.sh
```

The verifier pins both A2.3 authorities, nine immutable source blobs and
their anchors, the 22-file schema-history inventory, the current disabled
route anchors, the exact migration root inventory, SQL bytes/digest, statement
counts, guarded object counts, client binding, exact `u64` range, composite
family/parent lineage, single-root enforcement, self-parent denial, required
constraints/indexes, absence of destructive commands, and absence of
business-authority or secret columns.
The self-test requires deterministic reports, readiness exit `3`, and failure
for readiness, hash, SQL-anchor, client, cross-client replay, chain bound,
family ownership, parent lineage, self-parent, history, route, authority,
single-successor uniqueness, one-root partial uniqueness, revoke-versus-rotate
overclaim, blocker, catalog-adoption, digest-algorithm,
production-environment, database-environment, and Redis-environment tampering.

No database, Redis, network, JWKS, service, container, browser, Kubernetes,
migration, or deployment target is contacted.

## Residual A3.8 STOP blockers

1. No migration runner or version ledger owns the identity migration root.
2. No approved baseline mapping reconciles existing `users`, `wallet_users`,
   nonce, refresh-token, or session state.
3. No additive identity/challenge/session backfill exists.
4. No populated source-version upgrade has preserved real rows.
5. No concurrent nonce-consume, cross-client replay, exact generation
   increment, revoke-versus-rotate ordering, refresh-rotation, reuse,
   revocation, or startup fixture has run.
6. No pre/post row, identity, constraint, orphan, expiry, or active-session
   reconciliation has run.
7. No audited repository/transaction implementation uses this schema; routes
   intentionally remain disabled.
8. Canonical issuer, key rotation, audience, and external JWKS integration are
   unexercised.
9. Neither migration nor compatibility behavior has run against a disposable
   or live database.
10. `IF NOT EXISTS` provides only name idempotence; no catalog compatibility
    check or version-ledger adoption protocol rejects an incompatible existing
    relation.

`--mode readiness` intentionally exits `3`. A3.8 cannot authorize route
enablement or deployment, and production deployment still requires an
explicit user instruction after every STOP is closed.
