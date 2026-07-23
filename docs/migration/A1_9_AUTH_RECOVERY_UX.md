# A1.9 Frontend Auth-Recovery UX

## Outcome

A1.9 adds a closed, server-derived auth-page state machine on top of the
one-shot session recovery proved by A1.8:

- a genuinely signed-out visitor can connect a wallet;
- a refresh-eligible visitor sees fixed recovery progress and cannot start a
  competing wallet flow;
- verifier unavailability is visible, nondisclosing, non-actionable and
  private-cache protected;
- a rejected automatic recovery emits one fixed token-free failure event;
- only a page that is still exactly `recovering` may consume that event and
  expose the `Try Again` wallet action;
- stale wallet-status events cannot mutate recovering or
  verifier-unavailable pages;
- a valid wallet click immediately renders `Opening wallet...`, disables
  duplicate activation and marks the page busy;
- the unsupported operational and numeric customer claims were replaced with
  truthful wallet/product-fit copy.

This is hermetic evidence, not production readiness. It performs no live
browser, network, database, Redis, service, deployment or production action.

## Frozen lineage

| Role | Ref | Commit |
| --- | --- | --- |
| TypeScript compatibility source | `origin/development` | `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` |
| A1.8 predecessor | `migration/dioxus-microservices` | `c238954cbbf9b8a5db57ef117f0be638c4613766` |
| Immediate implementation base | `migration/dioxus-microservices` | `346d520484e23532ec40a62d1e2fba9d7a10472c` |

A1.9 does not run the A1.8 verifier against the changed A1.9 bootstrap. The
A1.8 verifier intentionally pins the former fixed bootstrap byte-for-byte.
Instead, the A1.9 verifier materializes the historical A1.8 contract,
verifier and twelve evidence files from `c238954c`, runs that exact verifier
in a separate temporary Git/evidence fixture, and requires:

- 11/11 A1.8 invariants;
- 46/46 A1.8 evidence anchors;
- 9/9 A1.8 residual STOPs;
- static-self-test-only execution.

A1.9 then verifies the changed fixed bootstrap and current auth UX under its
own contract.

## Closed state contract

| Access observation on `/auth` | Own refresh cookie | Rendered state | Wallet action | Recovery bootstrap |
| --- | --- | --- | --- | --- |
| Missing or rejected | No | `signed_out` | Enabled | Absent |
| Missing or rejected | Yes | `recovering` | Disabled | Exactly one |
| Verifier unavailable | Either | `verifier_unavailable` | Disabled | Absent |
| Verified | Either | No auth page; private same-origin redirect | N/A | Absent |

Non-auth routes receive no auth-page state parameter. An unknown present
state fails closed as `verifier_unavailable`.

Recovery-bearing and verifier-unavailable auth HTML is
`private, no-store` and varies on `Cookie, Authorization`. `/offline` remains
the reviewed public, anonymous and recovery-free exception.

## Browser event boundary

The SSR bootstrap remains fixed and request-independent:

```text
window.epsxAuth.recover().catch(function(){try{document.dispatchEvent(new CustomEvent('epsx:auth:recovery',{detail:{version:1,state:'failed'}}));}catch(_){}});
```

The event contains only `version: 1` and `state: failed`. It contains no
error, token, wallet, user, permission or plan material.

The page accepts that event only while its current DOM state is exactly
`recovering`. A valid transition:

1. changes the DOM state to `recovery_failed`;
2. hides progress and clears `aria-busy`;
3. focuses a fixed `role=alert`;
4. renders fixed generic failure copy;
5. enables and relabels the wallet action to `Try Again`.

The ordinary wallet-status handler is guarded by `authActionable()`, which is
true only for `signed_out` and `recovery_failed`. Delayed wallet events
therefore cannot escape recovering or verifier-unavailable states.

## Evidence inventory

The contract pins exact SHA-256 digests for five files:

- `apps/frontend/src/main.rs`
- `apps/frontend/src/ssr.rs`
- `shared/rust/bff/src/browser_auth.rs`
- `shared/rust/dioxus_ui/src/pages/auth_page.rs`
- `scripts/migration/test-auth-recovery-ux.js`

The verifier duplicates all five expected digest tuples as literals and also
checks state classification, cache behavior, DOM guards, fixed copy, A1.8
single-flight/lock/reload structure and the exact test inventory.

## Hermetic execution

Integrity runs eleven exact or narrowly filtered Rust cases:

- one fixed-bootstrap BFF test;
- six auth-page state/action tests, including immediate
  `Opening wallet...` feedback;
- one truthful product-fit/pitch test;
- one frontend state-classifier test;
- one verifier-unavailable private-cache test;
- one full frontend router journey.

The Bun fake-DOM harness contributes four exact cases:

1. rejected recovery emits one exact closed event;
2. resolved recovery emits no failure event;
3. invalid or stale events leave the closed page unchanged;
4. valid failure becomes fixed actionable UI without payload reflection.

The harness extracts and executes both the real Rust-embedded browser bridge
and the real page listener. It supplies only fake DOM, Web Lock, channel,
location and fetch objects; it opens no real browser or network connection.

Run:

```bash
scripts/migration/verify-a1-9-auth-recovery-ux.sh --mode integrity
scripts/migration/verify-a1-9-auth-recovery-ux.sh --mode report
scripts/migration/verify-a1-9-auth-recovery-ux.sh --mode readiness
scripts/migration/test-a1-9-auth-recovery-ux.sh
```

Expected exits are 0 for integrity/report and 3 for readiness.

## Twelve enforced invariants

1. Historical A1.8 replay.
2. Closed auth-page state classification.
3. Announced, noninteractive recovering state.
4. Visible verifier outage that never recovers automatically.
5. Actionable genuine signed-out state.
6. Fail-closed unknown and stale state.
7. Fixed token-free recovery-failure event.
8. Versioned, generic and actionable recovery-failure UI.
9. Immediate wallet-opening feedback.
10. Retained A1.8 session safety.
11. Private/offline and truthful UI boundaries.
12. Retained backend policy authority.

## Thirteen residual STOPs

A1.9 retains the nine A1.8 readiness limits:

1. real-browser matrix unproved;
2. post-commit fault behavior unproved;
3. PostgreSQL refresh ordering unproved;
4. exactly-once cookie delivery unproved;
5. BFF-unreachable HttpOnly clearing unproved;
6. post-revocation access-token validity remains;
7. missing-access authority preflight remains unproved;
8. cross-document cookie acceptance remains unproved;
9. production actions remain unauthorized.

It adds four UX-specific STOPs:

10. assistive-technology validation unproved;
11. wallet-provider and network-switching matrix unproved;
12. responsive visual regression unproved;
13. admin auth-recovery UX parity unproved.

Passing A1.9 must not be interpreted as permission to use a live browser,
connect to a service or database, alter infrastructure, deploy, or operate on
production.

## Composition boundary

A1.9 belongs only to the A1 auth/session chain. It does not compose into A12:
A12 is an analytics/indexer contract with exactly 40 domain anchors, 24
blockers and A2.4–A2.11 boundary evidence. A1.9 discharges none of those
blockers, so A12 remains byte-unchanged.
