# A2.4 market analytics route and authorization boundary

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

The standalone `apps/analytics` candidate now exposes only `/health` and the
five canonical `/api/analytics/*` market routes. Raw root paths,
`/api/public/analytics/*`, the event service's `/api/v1/analytics/*` namespace,
wrong methods, trailing-path drift, and the former global ranking-offset SSE
route return `404` before handler or provider work.

Health and the four metadata routes remain deliberately public. They remove
any bearer and spoofable identity headers without invoking a verifier.
Rankings retain anonymous/free-tier access when no Authorization header is
present. When one is present, it must be exactly one strict bearer accepted by
the shared `epsx-service-auth` verifier. Only the exact frontend or admin
audience can establish a principal. Invalid, duplicate, malformed and
unsupported credentials fail before the ranking handler.

Every rankings response—including anonymous, authenticated, `401`, and
`403`—is `private, no-store` and varies on `Authorization`. This keeps the
wallet-dependent offset/query result out of shared intermediary caches.

The verified wallet is carried through a minimal server-owned
`AnalyticsWalletContext`; callers cannot choose it through headers or query
data. The existing monolith handler continues to accept its verified
`OpenIDUserContext`, while the standalone transport no longer fabricates
monolith-only claim fields. Analytics never interprets token permission strings
as plan or ranking-offset authority.

## SSE containment

The unauthenticated downstream `/v1/rankings/stream` route and candidate
consumer startup are removed. The historical parser module, tests and
dependencies remain source-only; the candidate binary neither starts nor
exposes them. The active Kubernetes manifest is intentionally unchanged
because every overlay still selects a pre-A2.4 image that expects the old SSE
configuration. Removing that environment value, supplying reviewed OIDC
configuration, selecting an immutable candidate image and changing route
ownership belong to one later cutover packet. This does not make the identity
event side safe: its unauthenticated emit/global stream, ephemeral delivery,
missing owner filter, and missing replay/cursor contract remain STOPs.

## Hermetic evidence

The focused suite uses an in-process Axum router, fake verifier, fake handlers,
and atomic call counters. It performs no provider, identity, database, Redis,
RPC, browser, listener, or service I/O. Ten tests prove:

- the exact route/method inventory;
- public credential omission;
- anonymous rankings;
- frontend/admin verified-wallet propagation;
- invalid and unsupported credential denial;
- malformed and duplicate bearer denial before verifier/handler;
- spoof-header removal;
- alias, wrong-method and SSE denial before verifier/handler;
- unsafe production OIDC URL rejection; and
- the production router's canonical inventory without calling a provider
  handler.

The machine contract pins `origin/development` at
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`, the pre-slice target at
`c238954cbbf9b8a5db57ef117f0be638c4613766`, and SHA-256 digests for the four
implementation files. Its verifier refuses live and
production-looking environments, forces Cargo offline, and returns readiness
exit `3` even after integrity passes.

## Residual STOP conditions

- The monolith remains the canonical route owner; no gateway, Cloudflare,
  Kubernetes, image, canary, rollback, or cutover action is authorized.
- Legacy public duplicate paths, API-key behavior, implicit `HEAD`,
  wrong-method `405`, response/query compatibility and anonymous cap semantics
  are not fully locked.
- Identity's gRPC server still returns the free offset for every wallet and has
  no authenticated service identity or authoritative owner binding.
- Authenticated authority errors still downgrade silently and may call the
  provider; they need a truthful fail-closed response before provider work.
- TradingView provenance, licensing, quotas, retries, timeout, normalization,
  freshness, cache, error behavior and public amplification controls are
  unproven.
- The checked-in overlays still select pre-A2.4 images/configuration; no
  reviewed candidate OIDC wiring exists, and `/health` proves only a static
  response.
- The frontend BFF and Dioxus analytics experience remain intentionally
  unavailable; no real JWT/browser/provider/staging journey exists.
- No database, Redis, RPC, live market-data, secret, service, image or
  production operation is authorized.

## Gate usage

```bash
./scripts/migration/verify-a2-4-market-analytics-authorization.sh --mode integrity
./scripts/migration/verify-a2-4-market-analytics-authorization.sh --mode report
./scripts/migration/verify-a2-4-market-analytics-authorization.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-4-market-analytics-authorization.sh
```

This slice proves a narrow, fail-closed direct-service boundary. It does not
claim market analytics is production-ready or authorize deployment.
