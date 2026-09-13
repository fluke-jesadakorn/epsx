# A2.7 ranking entitlement snapshot

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A2.7 adds one pure, deterministic resolver for a caller-supplied ranking
entitlement snapshot. The resolver owns no database, clock, network or runtime
wiring. It evaluates every row against one explicit epoch-microsecond
`observed_at`, requires an
active assignment joined to a present active plan, excludes assignments whose
expiry is equal to or before that instant, validates relevant ranking-offset
candidates strictly, and selects the minimum candidate from the Free Plan seed.

The result keeps three successful states distinct: no effective plan, effective
plans without a selected ranking grant, and a plan-derived grant. All three are
different from malformed-snapshot failure. A plan-derived offset of `100`
therefore remains provenance-bearing success rather than being confused with an
empty result or authority outage.

This is a policy-kernel slice only. The identity gRPC runtime deliberately keeps
the always-Free implementation. No repository, SQL, schema ownership, service
authentication, event delivery, BFF or UI path is activated.

## Frozen source semantics

The source baseline is `origin/development` at
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`.

The source ranking query:

- considers only active assignments joined to active plans;
- treats `expires_at > NOW()` or no expiry as effective, so equality is expired;
- initializes the minimum to the Free Plan offset and takes lower valid metadata
  or `epsx:rankings:offset:*` permission values, retaining the source parser's
  decimal leading-zero behavior before strict range validation;
- returns Free Plan successfully when no joined row is effective; and
- does not apply `assigned_at` as a scheduled-start policy and does not apply a
  grace period.

Other source surfaces conflict with that narrow ranking query. The subscription
view applies plan grace, while `PlanFeatures` and token/direct-permission paths
suggest additional entitlement sources. A2.7 records those differences as
STOPs. It does not invent scheduling, grace, direct-grant, token-claim or
`PlanFeatures` authority.

The executable source presentation treats offsets `0` and `1` as full access.
For `N > 1`, ranks `1..N-1` are locked and rank `N` is the first visible rank.
Thus offset `100` means ranks `1-99` are locked. Stale comments that describe
`101+` do not override the executable handler and UI behavior. No UI is edited
by A2.7.

## Pinned target base

The immutable target base is
`395db722e2d71ff73a606d7eac14d6c4ef9d972d`, the completed post-A2.6
snapshot. A2.6 already proves that authenticated authority errors stop before
market-provider work. A2.7 does not reinterpret or weaken that historical
boundary.

## Hermetic proof boundary

The focused Rust tests replay the checked-in fixture ledger through the pure
resolver. They cover the three typed success outcomes, microsecond expiry boundaries,
inactive and unrelated input, overlapping grants, strict candidate validation,
duplicate/order invariance and error separation. The verifier runs only the
enumerated tests by their fully qualified names with `--exact` and Cargo offline.

The machine-readable contract is
`docs/migration/contracts/a2-7-ranking-entitlement-snapshot.json`; its fixture
ledger is
`docs/migration/fixtures/a2-7-ranking-entitlement-snapshot.json`.

No gate opens a database, connects to identity, Redis or a provider, binds a
listener, launches a browser, invokes Kubernetes, or performs deployment.

## Residual STOP conditions

- Core still owns the plan, assignment and permission tables. No reviewed
  core-owned adapter supplies the identity service.
- No single SQL statement or transaction proves one atomic owner-scoped
  snapshot.
- Schema adoption, compatibility probing, populated reconciliation and source
  parity remain unproved.
- The identity runtime still serves the always-Free stub.
- Scheduled-start and grace semantics remain explicitly unresolved.
- Direct permissions, token claims and `PlanFeatures` are not ranking authority
  in this slice and require an explicit later decision.
- Identity RPC has no workload identity, exact caller authorization or TLS
  policy.
- Ranking events have no transactional outbox, monotonic revision, durable
  cursor, replay or gap repair.
- No typed BFF or Dioxus UI consumes this resolver, and no browser flow is
  claimed ready.
- There is no live fixture parity, shadow read, staging, observability or
  rollback evidence.
- Route ownership, image, configuration, gateway, Kubernetes and cutover remain
  unchanged.
- Passing integrity never authorizes a production or deployment action.

## Gate usage

```bash
./scripts/migration/verify-a2-7-ranking-entitlement-snapshot.sh --mode integrity
./scripts/migration/verify-a2-7-ranking-entitlement-snapshot.sh --mode report
./scripts/migration/verify-a2-7-ranking-entitlement-snapshot.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-7-ranking-entitlement-snapshot.sh
```

Integrity is offline. Readiness intentionally exits `3` while the twelve
recorded STOPs remain.
