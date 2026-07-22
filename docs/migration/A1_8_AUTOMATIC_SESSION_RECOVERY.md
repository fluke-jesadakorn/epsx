# A1.8 one-shot automatic session recovery

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A1.8 closes A1.7's `automatic-refresh-entrypoint-unproved` STOP with a narrow
SSR bootstrap. On a renderable SSR response, when a Rust BFF observes both:

1. no acceptable access credential (`MissingOrRejected`); and
2. its own HttpOnly refresh cookie,

it emits one fixed, token-free call to `window.epsxAuth.recover()`. Protected
frontend routes first redirect privately to `/auth`; that render emits the
bootstrap. The shared
controller performs the already hardened A1.7 refresh operation once and
reloads the current document only after the BFF attests a successful rotation.
No credential, identity, permission, plan, or request value is interpolated
into JavaScript.

The frontend suppresses recovery on `/offline`. That page remains the reviewed
credential-omitted public service-worker shell. Every frontend response that
does carry the recovery bootstrap is `private, no-store` and varies on
`Cookie, Authorization`; admin HTML already has the same private contract.
The frontend's credential-dependent protected-route and verified `/auth`
redirects use that private, credential-varying boundary as well, so a shared
cache cannot replay one session's redirect decision to another.

## Verifier-outage boundary

The old `Option` adapter collapsed all access verification failures into a
signed-out user. That is unsafe for automatic recovery: a JWKS outage could
otherwise rotate successfully, reload, fail verification again, and loop.

The shared verifier now returns exactly:

- `Verified { token, user }` for a cryptographically accepted backend identity;
- `MissingOrRejected` for an absent or rejected browser credential; or
- `VerifierUnavailable` for invalid verifier configuration, JWKS transport
  failure, malformed JWKS authority data, or an unknown key ID even after the
  verifier's one forced JWKS refresh.

Only `MissingOrRejected` permits recovery. `VerifierUnavailable` renders the
existing signed-out/unavailable surface without a refresh bootstrap. An
unknown key ID is ambiguous between a forged credential and issuer/JWKS
publication lag, so automatic rotation preserves the local family and requires
explicit reauthentication instead. Malformed JWTs, wrong algorithms, invalid
claims, and expired tokens remain credential rejection.

## Browser behavior

`window.epsxAuth.recover()` owns a page-lifetime promise independent of the
manual refresh promise. The recovery promise is never reset. Therefore:

- duplicate bootstraps in one page share one refresh operation;
- a verified `rotated` result reloads exactly once;
- `preserved` rejects without reload, redirect, or retry;
- rejected/unknown outcomes use A1.7's confirmed-clear navigation;
- an ambiguous transport performs only A1.7's one best-effort local clear;
- an unconfirmed clear never navigates as success; and
- browsers without Web Locks perform zero refresh network I/O.

The SIWE verification POST, which establishes a new cookie pair, uses the same
origin-wide session-mutation lock as refresh/recovery when Web Locks are
available. Challenge acquisition and wallet signing remain outside the lock.
This prevents an old-family recovery response from racing and overwriting or
clearing a newly established SIWE principal; browsers without Web Locks remain
usable because automatic recovery already refuses network I/O there.

There are no timers, readable expiry cookies, global fetch interception,
application-request replay, 401 retry, client-side JWT decoding, or
browser-owned permission/plan logic. Those patterns existed in the
`origin/development` TypeScript initialization/client flow and are deliberately
not migrated.

## Hermetic evidence

`scripts/migration/verify-automatic-session-recovery.sh` validates the exact
typed outcome, both SSR trigger predicates, cache boundaries, fixed bootstrap,
one-shot controller, and retained A1.7 no-retry/Web-Lock rules. It runs:

- 14 shared verifier tests;
- 8 shared browser-bridge tests;
- one frontend SSR cache-policy test;
- one frontend full-router recovery journey;
- one admin full-router recovery journey; and
- 20 Bun VM browser coordination/outcome cases.

The router journeys cover missing credentials, rejected credentials, a
syntactically valid access token whose JWKS authority is unavailable, absence
of a refresh cookie, token non-disclosure, exact bootstrap cardinality, and the
frontend protected-route/return journey and `/offline` output-equivalence
exception. The verifier self-test mutates representative critical boundaries,
including root-alias and SIWE-lock bypasses, and requires every mutation to
fail evidence validation.

## Residual STOP conditions

- No accepted real-browser matrix proves actual Web Locks, BroadcastChannel,
  HttpOnly cookie application, reload, redirect, or multi-tab scheduling.
- No proxy/browser fault injection proves timeout or connection reset after a
  backend rotation commits.
- No PostgreSQL exercise proves consume/replay/family-lock ordering or
  acknowledgement ambiguity.
- A disconnect after rotation but before `Set-Cookie` delivery still forces
  reauthentication; there is no receipt/idempotency protocol.
- JavaScript cannot prove clearing HttpOnly cookies while the BFF itself is
  unreachable.
- Already-issued access JWTs remain valid until expiry after logout or family
  revocation.
- With no access cookie there is no key ID to probe before recovery; a JWKS
  outage observed only while validating the rotated successor can still force
  local clearing and reauthentication after the backend commit.
- A page-lifetime promise cannot prove cross-document loop freedom if a real
  browser accepts the refresh cookie but selectively rejects/evicts the access
  cookie; cookie acceptance and reload behavior need real-browser proof or a
  separately reviewed server-owned attempt guard.
- No production browser, proxy, TLS, issuer, database, secret, canary,
  rollback, deployment, or service action is authorized or proven.

This is a hermetic usability and fail-closed recovery slice. It does not claim
production readiness or deployment authorization.
