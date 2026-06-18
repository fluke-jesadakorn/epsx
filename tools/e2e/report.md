# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-18T02:26:02.602Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 13.54 | 0 | 6 | 5 | 6 |
| 2 | `about` *(SKIP)* | `/about` | 0 | 0 | 0 | 0 | 0 |
| 3 | `access-denied` | `/access-denied` | 93.76 | 0 | 0 | 2 | 0 |
| 4 | `account` | `/account` | 93.68 | 4 | 2 | 3 | 2 |
| 5 | `account-credits` | `/account/credits` | 93.47 | 2 | 0 | 2 | 0 |
| 6 | `analytics` | `/analytics` | 79.57 | 0 | 10 | 13 | 10 |
| 7 | `auth` | `/auth` | 99.96 | 0 | 0 | 0 | 0 |
| 8 | `chat` | `/chat` | 93.76 | 0 | 0 | 3 | 0 |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 99.96 | 0 | 0 | 0 | 0 |
| 10 | `chat-history` | `/chat/history` | 99.96 | 0 | 0 | 0 | 0 |
| 11 | `contact` *(SKIP)* | `/contact` | 0 | 0 | 0 | 0 | 0 |
| 12 | `dashboard` | `/dashboard` | 1.58 | 0 | 0 | 2 | 0 |
| 13 | `developer` | `/developer` | 99.82 | 0 | 0 | 2 | 0 |
| 14 | `developer-usage` | `/developer/usage` | 79.97 | 0 | 0 | 2 | 0 |
| 15 | `developer-docs` | `/developer/docs` | 99.8 | 0 | 0 | 2 | 0 |
| 16 | `manual` | `/manual` | 2.04 | 0 | 0 | 2 | 0 |
| 17 | `news` | `/news` | 93.76 | 0 | 0 | 2 | 0 |
| 18 | `news-sample-slug` | `/news/sample-slug` | 93.7 | 1 | 0 | 4 | 0 |
| 19 | `notifications` | `/notifications` | 99.96 | 0 | 0 | 0 | 0 |
| 20 | `offline` *(SKIP)* | `/offline` | 0 | 0 | 0 | 0 | 0 |
| 21 | `payment` | `/payment` | 99.34 | 0 | 0 | 2 | 0 |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 99.09 | 0 | 0 | 2 | 0 |
| 23 | `permissions` | `/permissions` | 99.96 | 0 | 0 | 0 | 0 |
| 24 | `plans` | `/plans` | 22.55 | 1 | 0 | 3 | 0 |
| 25 | `portfolio` | `/portfolio` | 20.47 | 0 | 0 | 4 | 0 |
| 26 | `privacy` | `/privacy` | 6.18 | 0 | 0 | 2 | 0 |
| 27 | `profile` | `/profile` | 99.96 | 0 | 0 | 0 | 0 |
| 28 | `terms` | `/terms` | 93.76 | 0 | 0 | 2 | 0 |

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
| `missing-hrefs` | 18 | 3 |
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

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:30199/`
- `access-denied` — prod→`1: https://epsx.io/access-denied` dev→`0: http://localhost:30199/access-denied`
- `account` — prod→`1: https://epsx.io/account` dev→`0: http://localhost:30199/account`
- `account-credits` — prod→`1: https://epsx.io/account/credits` dev→`0: http://localhost:30199/account/credits`
- `analytics` — prod→`1: https://epsx.io/analytics` dev→`0: http://localhost:30199/analytics`
- `auth` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30199/auth`
- `chat` — prod→`1: https://epsx.io/chat` dev→`0: http://localhost:30199/chat`
- `chat-sample-conv-id` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:30199/auth`
- `chat-history` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:30199/auth`
- `dashboard` — prod→`1: https://epsx.io/dashboard` dev→`0: http://localhost:30199/dashboard`

### `missing-hrefs` — 18 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/GHC"`
- `account` — sample: `"/account/credits"`
- `analytics` — sample: `"https://www.tradingview.com/symbols/PH"`

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
