# Dev env: wave 12 / epsx-analytics-service (isolated, no K8s, no Cloudflare)
#
# The new analytics binary is a self-contained axum service with:
#   - 0 PostgreSQL connections (per Q2 ROADMAP §7)
#   - 0 Redis connections (in-process memory cache)
#   - 1 outbound dep: TradingView (lazy, fires on first request)
#   - 0 required env vars
#   - 1 hard-coded bind: 0.0.0.0:8080
#
# So the dev env is a single container, no compose, no upstream services.
# This file documents the *shape* of the dev env so it's reproducible.
# The actual `docker run` lives in `scripts/dev-up.sh`.

Image:       epsx-analytics:dev
Host port:   18080  (host-side; container listens on 8080)
Network:     bridge (default)
Volumes:     none
Env vars:    none required
Healthcheck: GET http://localhost:18080/health
Smoke:       GET http://localhost:18080/api/analytics/rankings
             GET http://localhost:18080/api/analytics/filters
             GET http://localhost:18080/api/analytics/countries
             GET http://localhost:18080/api/analytics/available-countries
             GET http://localhost:18080/api/analytics/sectors

Isolated from:
  - prod K8s cluster (epsx-prod namespace) — no apply
  - dev K8s cluster (epsx-dev namespace) — no apply
  - Cloudflare Tunnel routing — no /etc/cloudflared config edit
  - origin/migration/dioxus-microservices — was reset to 340e7980 (wave-11)

Tear down:   `scripts/dev-down.sh` (stops + removes the container, keeps the image)
Full reset:  `scripts/dev-reset.sh` (down + rmi + rebuild from source)

Notes:
  - Port 18080 (not 8080) on the host so the dev container can run
    alongside the existing dev monolith if needed without conflict.
  - The first request to /api/analytics/* will block for ~1-3s while
    the TradingView HTTP client connects + the EPSRankingService
    does its initial scan. Subsequent requests are fast (in-process cache).
  - The 6th wave-12 smoke test (`apps/backend/tests/wave12_smoke.rs`)
    runs as a `cargo test` against the workspace, NOT against this
    running container. To smoke the *container* itself use
    `scripts/smoke-container.sh`.
