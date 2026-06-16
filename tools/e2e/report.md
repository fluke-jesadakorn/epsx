# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-16T16:27:29.084Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 93.79 | 0 | 5 | 4 | 5 |
| 2 | `about` *(SKIP)* | `/about` | 0 | 0 | 0 | 0 | 0 |
| 3 | `access-denied` | `/access-denied` | 93.82 | 0 | 0 | 2 | 0 |
| 4 | `account` | `/account` | 93.75 | 4 | 2 | 3 | 2 |
| 5 | `account-credits` | `/account/credits` | 93.57 | 2 | 0 | 2 | 0 |
| 6 | `analytics` | `/analytics` | 79.62 | 0 | 10 | 13 | 10 |
| 7 | `auth` | `/auth` | 99.96 | 0 | 0 | 0 | 0 |
| 8 | `chat` | `/chat` | 93.82 | 0 | 0 | 3 | 0 |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 99.96 | 0 | 0 | 0 | 0 |
| 10 | `chat-history` | `/chat/history` | 99.96 | 0 | 0 | 0 | 0 |
| 11 | `contact` *(SKIP)* | `/contact` | 0 | 0 | 0 | 0 | 0 |
| 12 | `dashboard` | `/dashboard` | 93.79 | 0 | 0 | 2 | 0 |
| 13 | `developer` | `/developer` | 99.82 | 0 | 0 | 2 | 0 |
| 14 | `developer-usage` | `/developer/usage` | 99.8 | 0 | 0 | 2 | 0 |
| 15 | `developer-docs` | `/developer/docs` | 99.79 | 0 | 0 | 2 | 0 |
| 16 | `manual` | `/manual` | 89.01 | 0 | 0 | 2 | 0 |
| 17 | `news` | `/news` | 93.82 | 0 | 0 | 2 | 0 |
| 18 | `news-sample-slug` | `/news/sample-slug` | 93.79 | 1 | 0 | 4 | 0 |
| 19 | `notifications` | `/notifications` | 99.96 | 0 | 0 | 0 | 0 |
| 20 | `offline` *(SKIP)* | `/offline` | 0 | 0 | 0 | 0 | 0 |
| 21 | `payment` | `/payment` | 99.34 | 0 | 0 | 2 | 0 |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 99.09 | 0 | 0 | 2 | 0 |
| 23 | `permissions` | `/permissions` | 99.96 | 0 | 0 | 0 | 0 |
| 24 | `plans` | `/plans` | 82.5 | 1 | 0 | 3 | 0 |
| 25 | `portfolio` | `/portfolio` | 64.73 | 0 | 0 | 5 | 0 |
| 26 | `privacy` | `/privacy` | 93.82 | 0 | 0 | 2 | 0 |
| 27 | `profile` | `/profile` | 99.96 | 0 | 0 | 0 | 0 |
| 28 | `terms` | `/terms` | 93.81 | 0 | 0 | 2 | 0 |

## Skipped routes (intentional — see `routes-skip.json`)

| Slug | Reason |
|------|--------|
| `about` | prod serves the Next.js SPA fallback (returns the home page) — comparing prod /about against dev /about compares two different pages |
| `contact` | prod serves the Next.js SPA fallback (returns the home page) — same as /about |
| `offline` | prod serves the Next.js SPA fallback (returns the home page) — same as /about |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 59 | 19 |
| `redirect-chain-differs` | 25 | 25 |
| `missing-hrefs` | 17 | 3 |
| `skipped-route` | 3 | 3 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 59 occurrences

Affected routes (first 10):

- `home` — sample: `"Market"`
- `access-denied` — sample: `"Market"`
- `account` — sample: `"Market"`
- `account-credits` — sample: `"Market"`
- `analytics` — sample: `"Market"`
- `chat` — sample: `"Market"`
- `dashboard` — sample: `"Market"`
- `developer` — sample: `"Market"`
- `developer-usage` — sample: `"Market"`
- `developer-docs` — sample: `"Market"`

### `redirect-chain-differs` — 25 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:30200/`
- `access-denied` — prod→`1: https://epsx.io/access-denied` dev→`0: http://localhost:30200/access-denied`
- `account` — prod→`1: https://epsx.io/account` dev→`0: http://localhost:30200/account`
- `account-credits` — prod→`1: https://epsx.io/account/credits` dev→`0: http://localhost:30200/account/credits`
- `analytics` — prod→`1: https://epsx.io/analytics` dev→`0: http://localhost:30200/analytics`
- `auth` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30200/auth`
- `chat` — prod→`1: https://epsx.io/chat` dev→`0: http://localhost:30200/chat`
- `chat-sample-conv-id` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:30200/auth`
- `chat-history` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:30200/auth`
- `dashboard` — prod→`1: https://epsx.io/dashboard` dev→`0: http://localhost:30200/dashboard`

### `missing-hrefs` — 17 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/GHC"`
- `account` — sample: `"/account/credits"`
- `analytics` — sample: `"https://www.tradingview.com/symbols/COF"`

### `skipped-route` — 3 occurrences

Affected routes (first 10):

- `about`
- `contact`
- `offline`

## Skipped / missing artifacts

_none_

## How to reproduce

```bash
# 1. Capture prod (10-15 min)
bash tools/e2e/capture-prod.sh

# 2. Capture dev (10-15 min, requires port-forward 30101)
bash tools/e2e/capture-dev.sh

# 3. Diff + report (~5 min)
bash tools/e2e/diff.sh

# 4. Subset (e.g. smoke 3 routes)
bash tools/e2e/capture-prod.sh home,about,auth
bash tools/e2e/capture-dev.sh  home,about,auth
bash tools/e2e/diff.sh        home,about,auth
```
