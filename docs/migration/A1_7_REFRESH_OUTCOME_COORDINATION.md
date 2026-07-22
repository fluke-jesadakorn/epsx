# A1.7 refresh outcome and browser coordination

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A1.7 closes two benign paths that become destructive once A1.6 treats every
consumed refresh credential as replay:

1. two tabs refreshing the same HttpOnly credential concurrently; and
2. a BFF retrying or preserving an old credential after the backend committed
   rotation but the response was lost.

The Rust backend now attests one closed, token-free result on every refresh
response through `x-epsx-refresh-outcome`:

| Backend marker | Meaning | BFF action |
|---|---|---|
| `rotated` | A complete successor response follows a committed rotation. | Verify every field and JWT, then atomically replace both cookies. |
| `not_rotated` | This exact error path is proven to finish before rotation. | Preserve the existing pair and surface the error without retry. |
| `rejected` | The credential is invalid, expired, revoked, or replayed. | Clear locally and never retry. |
| `outcome_unknown` | Commit or replay response cannot be proven either way. | Clear locally and never retry. |

Status alone is not mutation evidence. Missing, invalid, or status-inconsistent
markers fail closed. In particular, a raw `503` is ambiguous because a database
commit can take effect before its acknowledgement is lost.

Both Rust BFFs expose the resulting local state through the token-free
`x-epsx-session-state: rotated|preserved|cleared` header. Transport errors,
unexpected redirects/statuses, malformed or invalid 2xx bodies, and
post-rotation JWT/identity/cookie failures clear the local session. Only the
exact `not_rotated` matrix preserves it.

## Browser behavior

The shared hydration-free bridge now:

- keeps one same-window in-flight refresh promise;
- runs refresh and logout through the same origin-scoped exclusive Web Lock;
- refuses refresh before network I/O when Web Locks are unavailable;
- performs exactly one refresh fetch and never automatically retries it;
- uses a fixed BroadcastChannel with closed, token-free session events;
- degrades BroadcastChannel construction or publication failures to same-tab
  events without breaking a successful mutation;
- redirects/broadcasts session end only after the BFF confirms local cookie
  clearing, including the best-effort clear after an ambiguous transport loss;
  and
- binds the same delegated controller to authenticated customer desktop/mobile
  actions, both active admin chrome variants, the connected-wallet disconnect
  hook, and admin denial reauthentication action.

Authenticated customer SSR now renders Account and Sign out actions instead of
the misleading Connect action. This is presentation and session plumbing only;
permissions, plans, entitlements, and subscription rules remain backend-owned.

Credential-bearing auth clients do not follow redirects. A 307/308 therefore
cannot replay refresh, logout, signed challenge, verification, or bearer data
to a redirect target.

## Hermetic evidence

The A1.7 verifier and self-test use only repository files, Rust unit tests,
loopback mocks, and a Bun VM. The VM executes the exact browser script embedded
by Rust and proves:

- same-window single flight;
- cross-tab mutual exclusion with a shared fake Web Lock manager;
- refresh/logout ordering through the same lock;
- zero refresh fetches without Web Locks;
- no retry or session-end event for a preserved result;
- one session-end event for a cleared result;
- no session-end event or success redirect when local clearing is unconfirmed;
- sender and receiver handling over a shared fake BroadcastChannel bus;
- constructor/post failures degrading to same-tab events;
- a delegated authenticated-header logout click; and
- the exact token-free broadcast schema.

Focused Rust tests pin the backend marker matrix, the shared BFF classifier,
frontend/admin cookie outcomes, redirect refusal, browser bridge source
contract, and truthful customer header.

## Residual STOP conditions

- No real browser matrix proves Web Locks, BroadcastChannel, HttpOnly cookie
  application, navigation, or multi-tab behavior.
- No proxy or browser fault injection proves timeout/reset after backend commit.
- No PostgreSQL exercise proves the A1.6 consume/replay transaction or
  commit-ack ambiguity paths.
- A BFF/browser disconnect after backend rotation but before `Set-Cookie`
  delivery still forces reauthentication; seamless exactly-once delivery would
  require a separately designed server receipt/idempotency protocol.
- Clearing HttpOnly cookies while the BFF itself is unreachable is impossible
  to prove from JavaScript and remains a production-operability STOP.
- The shared refresh controller has no automatic customer/admin runtime caller;
  an expired access-cookie journey still falls back to explicit
  reauthentication instead of seamless recovery.
- Already-issued access JWTs remain valid until expiry after logout or family
  revocation.
- Production proxy retry policy, TLS/origin routing, canary, rollback, browser,
  database, secret, and deployment proof remain absent and unauthorized.

This slice is therefore a partial hermetic production-safety proof, not
production readiness or deployment authorization.
