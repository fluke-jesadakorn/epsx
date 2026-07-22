# A1.6 Refresh Digest and Replay-State Expansion

Status: **combined hermetic schema/runtime proof; migration-root,
PostgreSQL/MVCC, cutover, persistent-key, and production STOP**.

A1.5 binds each newly issued refresh bearer to one client and family, but its
storage contract represents the raw UUID bearer in
`public.openid_refresh_tokens.token_id`. Its lifecycle model also records only
`is_revoked`, so the durable row cannot distinguish successful consumption,
explicit revocation, or detected replay.
This package certifies the nullable schema and exact source-level composition of
the digest-only runtime wave. The migration itself does not hash a token,
backfill a row, provision a key, run against a database, or authorize
deployment. The hermetic runtime proof does not remove the PostgreSQL, MVCC,
cutover, persistent-key, or production STOPs.

## Additive boundary

`20260723100000_add_refresh_token_digest_replay_state` adds these nullable,
default-free, non-identity, non-generated columns:

- `token_digest BYTEA`;
- `digest_key_id VARCHAR(32)`;
- `digest_version SMALLINT`;
- `storage_version SMALLINT`;
- `consumed_at TIMESTAMPTZ`;
- `revoked_at TIMESTAMPTZ`;
- `replay_detected_at TIMESTAMPTZ`.

The digest metadata is all-null for untouched legacy rows or all-present with
`storage_version = 2`; every version-2 row also requires its existing
`client_id` and `family_id` bindings. Version 2 means the digest-row identifier
is not the bearer and only the keyed digest locates the session. Version 1
dual-write is deliberately not admitted: retaining a raw bearer for rolling
compatibility would fail the at-rest objective and create an unsafe mixed-reader
contract.

Validated checks require a 32-byte digest, a positive digest version, a
non-secret key identifier of 1-32 ASCII alphanumeric, underscore, or hyphen
characters, and exactly one terminal shape for digest rows:
active, consumed, or revoked. Legacy rows must keep every new digest and
lifecycle column NULL. Consumption/revocation cannot predate `created_at`, and
replay detection can occur only after consumption. A partial unique index covers
`(digest_key_id, digest_version, token_digest)` for digest rows. These are shape
constraints only. The migration alone does not name or prove a digest algorithm,
runtime state transition, key lookup, replay response, or `is_revoked`
synchronization; the separate hermetic runtime evidence below pins the current
source contract without claiming database execution.

Catalog guards reject incompatible pre-existing columns and any same-named
constraint or index rather than guessing equivalence. The forward migration has
no `UPDATE`, backfill, deletion, binding inference, or other row mutation. The
reverse migration raises before changing schema because dropping digest or
replay evidence would be a security regression.

## Hermetic runtime boundary

The pinned runtime source implements a strict `rt1` credential containing
exactly 32 random bytes from `OsRng`, canonical unpadded base64url framing, and a
32-byte HMAC-SHA256 digest under the domain separator
`epsx.refresh.v1\0`. Both backend factories require the dedicated HMAC keyring
from `REFRESH_TOKEN_HMAC_ACTIVE_KID` and `REFRESH_TOKEN_HMAC_KEYS_JSON`; there is
no generated or default refresh-key fallback.

New rows use independent UUID storage identifiers and persist only the digest,
key ID, digest version, and `storage_version = 2`. Validation and consumption
bind the exact digest tuple, client, and family. A winning rotation atomically
sets `is_revoked = true` with `consumed_at`, then inserts the digest-only
successor. Consumed-token reuse returns an internal success outcome so replay
recording and active-descendant revocation commit before the public boundary
maps the attempt to generic rejection. Rotation, logout, and replay response
share the transaction-scoped family advisory lock, and persisted lifecycle
timestamps come from PostgreSQL `clock_timestamp()`.

Both container paths inject the required keyring into `OpenIDTokenService` and
construct stateless auth with `new_with_openid`. Both Diesel schemas expose the
digest and lifecycle columns as `Bytea`, `Int2`, and `Timestamptz` shapes. The
token response plus logout and refresh request DTOs do not derive `Debug`.
`token_id` receives only independent UUID row IDs for new rows; raw UUID
comparison is bounded to storage-version-NULL legacy logout.

These are exact source and static-composition claims. They do not demonstrate a
successful PostgreSQL query, transaction interleaving, process restart, secret
mount, maintenance cutover, legacy scrub, or production request.

## Required maintenance cutover

There is no rolling mixed-version deployment for this wave. The runtime design
may emit only `storage_version = 2`, and pre-A1.6 rows with NULL digest metadata
must fail closed.

The release gate is therefore:

1. reconcile the duplicate active core baseline version before running any core
   migration;
2. provision and independently verify the durable digest-key ring without
   logging key material;
3. enter an explicitly authorized maintenance window, drain refresh traffic,
   and stop every old reader and writer;
4. apply the additive migration from every observed supported history;
5. deploy an independently verified digest-only runtime and force every
   pre-A1.6 session to authenticate again rather than letting a request claim or
   upgrade it;
6. reconcile and explicitly revoke or scrub legacy plaintext rows, prove zero
   accepted raw bearers, then reopen traffic.

Neither a request-supplied client nor the first caller may populate digest,
client, family, or lifecycle state on a legacy row. No active legacy row may be
silently assigned to a key version. Key rotation requires one active write key
and retained read keys; a key cannot be retired until PostgreSQL evidence proves
that no retained digest or replay record needs it. The hermetic runtime proof
does not prove any of that database or operational behavior.

## Hermetic evidence

```bash
./scripts/migration/verify-refresh-digest-replay.sh
./scripts/migration/test-refresh-digest-replay.sh
```

The verifier exact-pins both migration checksums, ten stable-ID schema
invariants, ten stable-ID runtime invariants, 72 exact evidence anchors, seven
column shapes, eight constraints, the partial unique index, credential/keyring
semantics, digest-only queries, lifecycle transitions, factory composition,
bearer DTO derives, and ten residual STOP claims. Readiness intentionally exits
`3`:

```bash
./scripts/migration/verify-refresh-digest-replay.sh --mode readiness
```

The self-test mutates production/database claims, legacy policy, semantic
evidence, schema/runtime invariant and STOP text, both migration bytes,
destructive SQL, digest/storage guards, constraint adoption, credential/keyring
semantics, query predicates, state transitions, factories, clocks, schemas,
dependencies, DTO derives, and forbidden live environment variables. No
database, Redis, network, service, browser, container, Kubernetes, migration, or
deployment target is contacted.

## Remaining STOP evidence

An explicitly authorized disposable PostgreSQL harness must still prove:

- expansion from both observed colliding baseline histories and every deployed
  partial history without row or schema loss;
- attempted down migration fails atomically without schema or data change;
- exact catalog types, nullability, constraints, validation state, and index
  definition;
- consume, revoke, and replay transitions under MVCC and the existing family
  advisory lock, including rollback after a forced successor failure;
- the drained old-to-new cutover has no mixed reader/writer interval;
- forced reauthentication, legacy revocation/scrubbing, row reconciliation, and
  zero accepted or retained raw bearers;
- persistent digest-key secret provisioning, restart persistence, production
  rotation/retirement, and PostgreSQL integration.

Production key material, actual secret provisioning and mount proof, deployment,
canary, rollback routing, and observability remain unauthorized and absent.
Configuration names are wired without production secret values, provisioning,
mounts, or live actions. Already-issued access tokens also remain valid until
their normal expiry.
