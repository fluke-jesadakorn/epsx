# A2.5 market rankings provider boundary

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

The canonical market rankings handler now depends on a narrow
`MarketRankingsProviderPort`. It no longer constructs a TradingView client
inside each request and no longer calls the disabled WebSocket enhancement.
The existing card, rank and quarterly projection stays intact, while accessible
pagination totals now exclude ranks hidden below the server-owned plan offset.

Both the standalone analytics candidate and the monolith build one shared
TradingView service, wrap its single-attempt rankings adapter once, and inject
one process-shared bounded provider into their router.

## Request and resource policy

- Anonymous requests are capped at 10 rows, matching the pinned development
  public client.
- Authenticated requests are capped at 100 rows, matching the existing UI
  pagination vocabulary and provider maximum.
- Rank-offset and page arithmetic is checked before provider work.
- Provider pages that exceed the requested row count or carry a negative total
  fail closed; client totals exclude locked ranks.
- Source aliases `qoq_growth`, `growth_factor` and
  `ranking_position` normalize to the provider's real `eps_growth` sort.
  Unknown sorts return a stable validation error.
- At most five provider calls run concurrently per process. Saturation fails
  fast with `503`.
- A logical call performs at most three total attempts, retries transport,
  `408`, `429` and `5xx` failures only, and is enclosed by one 30-second
  deadline.
- The live adapter performs one HTTP attempt beneath that policy. Provider
  response bodies are capped at two MiB, and bodies or raw transport details
  are not returned to clients.

These are local process bounds, not claims about distributed quotas,
licensing, freshness, cross-replica rate control, or circuit breaking.

## Hermetic evidence

Eighteen focused tests use only fake ports, atomics, semaphores and in-process
transformations. They cover invalid and accessible pagination, anonymous and
authenticated request/response caps, bounded response bytes, HTTP-status
classification, missing-total normalization, sort normalization, provider-call
suppression, transient success and exhaustion, permanent failure, timeout
permit release, peak concurrency, opaque errors, and quarterly DTO preservation.

No test opens a listener or touches TradingView, identity, PostgreSQL, Redis,
RPC, a browser, Kubernetes, DNS, or deployment state. The machine contract is
`docs/migration/contracts/a2-5-market-provider-boundary.json`; integrity may
pass, while readiness always exits `3`.

## Deliberately deferred

A2.5 does not change the identity ranking-offset fallback. The standalone
gRPC adapter can still hide authority outages behind the free-plan offset, and
the identity service does not yet derive paid access from authoritative plan
assignments. That is the next dependency and must fail before provider work.

The following also remain STOP conditions:

- production route ownership and cutover;
- legacy public aliases, API-key behavior, filters and response compatibility;
- provider licensing, quota, schema, provenance, observation time, freshness,
  cache age and stale semantics;
- cross-replica resilience and legacy direct TradingView callers;
- authenticated analytics-to-identity service identity;
- unsafe upstream global ranking events and dormant SSE source;
- dependency-aware runtime readiness;
- the typed BFF/Dioxus loading, empty, error, stale, filter, pagination,
  watchlist and entitlement journey;
- live JWT/provider/staging/fault evidence; and
- every production build, secret, restart, canary, rollback or deployment
  action.

## Next execution order

1. Make the identity offset query authoritative and fail authenticated
   authority errors before this provider boundary.
2. Authenticate the internal analytics-to-identity call.
3. Freeze the remaining query, provenance, freshness and compatibility
   contracts against the pinned development source.
4. Select the production route owner, then implement the typed BFF and Dioxus
   states.
5. Run separately authorized staging and rollback exercises before requesting
   production deployment approval.
