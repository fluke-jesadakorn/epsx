# E2E Dev Pipeline Design

Date: 2026-03-03

## Problem

- `development` branch has zero E2E tests in CI (only lint + test + build)
- `dev.epsx.io` is down — `cloudflared-config.prod.yml` routes it to a non-existent Docker container instead of `host.docker.internal:3000`
- Several UI areas are untested: nav dropdowns, theme toggle, contact page, 404 pages, watchlist interactions, API error states

## Scope

### Fix dev tunnel

`cloudflared-config.prod.yml` already has the correct fix (routes to `host.docker.internal`) but it's uncommitted and cloudflared hasn't been restarted. `cloudflared-config.dev.yml` is untracked.

Actions:
- Commit both configs
- Restart `epsx-prod-cloudflared` to apply the new routing

### New E2E test files (frontend)

| File | Coverage |
|------|----------|
| `e2e/frontend/nav-theme.spec.ts` | Market/Developer/Company dropdown links, theme toggle light↔dark |
| `e2e/frontend/contact-404.spec.ts` | `/contact` page, 404/not-found handling, back-to-home link |
| `e2e/frontend/watchlist-actions.spec.ts` | Portfolio watchlist: items show, add/remove via mocked API, empty state |
| `e2e/frontend/api-errors.spec.ts` | API 500 error handling on analytics/plans, no crash, fallback UI |

All new specs use existing patterns: `mockAllApis` for API interception, `authedPage` fixture for auth, `capture()` for screenshots.

### GitLab CI — new dev E2E job

```yaml
test:e2e:dev:
  stage: test
  image: mcr.microsoft.com/playwright:v1.58.0-noble
  needs: ["lint:eslint", "lint:typecheck", "test:frontend"]
  script:
    - bun install --frozen-lockfile
    - |
      for i in 1 2 3; do
        curl -sf https://dev.epsx.io > /dev/null && break
        echo "dev.epsx.io not ready (attempt $i), retrying..."
        sleep 10
      done
    - TEST_ENV=dev bunx playwright test
  artifacts:
    when: always
    paths: [playwright-report/, test-results/]
    expire_in: 7 days
  timeout: 15 minutes
  allow_failure: true
  rules:
    - if: $CI_COMMIT_BRANCH == "development"
```

## Architecture

- Tests run against `https://dev.epsx.io` (always-running bun dev on host, exposed via Cloudflare Tunnel)
- `allow_failure: true` so E2E doesn't block dev branch pipeline if tunnel is down
- Health check with 3 retries before starting playwright
- Shares playwright config (`TEST_ENV=dev`) with staging job pattern

## Success Criteria

1. `dev.epsx.io` resolves correctly after cloudflared restart
2. All 4 new spec files pass locally against `localhost:3000`
3. `test:e2e:dev` job appears in GitLab CI on `development` branch push
4. Playwright report artifact is uploaded on completion
