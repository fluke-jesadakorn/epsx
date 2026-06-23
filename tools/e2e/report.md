# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-23T03:51:14.346Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 12.38 | 0 | 6 | 4 | 6 |
| 2 | `about` | `/about` | 99.96 | 0 | 0 | 2 | 0 |
| 3 | `access-denied` | `/access-denied` | 93.86 | 0 | 0 | 1 | 0 |
| 4 | `account` | `/account` | 93.78 | 4 | 2 | 2 | 2 |
| 5 | `account-credits` | `/account/credits` | 93.54 | 2 | 0 | 1 | 0 |
| 6 | `analytics` | `/analytics` | 79.75 | 0 | 10 | 13 | 10 |
| 7 | `auth` | `/auth` | 99.96 | 0 | 0 | 2 | 0 |
| 8 | `chat` | `/chat` | 93.83 | 0 | 0 | 2 | 0 |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 99.96 | 0 | 0 | 2 | 0 |
| 10 | `chat-history` | `/chat/history` | 99.96 | 0 | 0 | 2 | 0 |
| 11 | `contact` | `/contact` | 99.96 | 0 | 0 | 2 | 0 |
| 12 | `dashboard` | `/dashboard` | 1.75 | 0 | 0 | 1 | 0 |
| 13 | `developer` | `/developer` | 99.82 | 0 | 0 | 4 | 0 |
| 14 | `developer-usage` | `/developer/usage` | 79.64 | 0 | 0 | 1 | 0 |
| 15 | `developer-docs` | `/developer/docs` | 99.8 | 0 | 0 | 4 | 0 |
| 16 | `manual` | `/manual` | 2.22 | 0 | 0 | 1 | 0 |
| 17 | `news` | `/news` | 93.86 | 0 | 0 | 1 | 0 |
| 18 | `news-sample-slug` | `/news/sample-slug` | 93.8 | 1 | 0 | 3 | 0 |
| 19 | `notifications` | `/notifications` | 99.96 | 0 | 0 | 2 | 0 |
| 20 | `offline` | `/offline` | 99.96 | 0 | 0 | 2 | 0 |
| 21 | `payment` | `/payment` | 99.24 | 0 | 0 | 4 | 0 |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 99.08 | 0 | 0 | 4 | 0 |
| 23 | `permissions` | `/permissions` | 99.96 | 0 | 0 | 2 | 0 |
| 24 | `plans` | `/plans` | 10.98 | 1 | 0 | 1 | 0 |
| 25 | `portfolio` | `/portfolio` | 8.81 | 0 | 0 | 3 | 0 |
| 26 | `privacy` | `/privacy` | 6.37 | 0 | 0 | 1 | 0 |
| 27 | `profile` | `/profile` | 99.96 | 0 | 0 | 2 | 0 |
| 28 | `terms` | `/terms` | 93.85 | 0 | 0 | 1 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 70 | 28 |
| `redirect-chain-differs` | 28 | 28 |
| `missing-hrefs` | 18 | 3 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 70 occurrences

Affected routes (first 10):

- `home` — sample: `"Connect"`
- `about` — sample: `""`
- `access-denied` — sample: `"Connect"`
- `account` — sample: `"Connect"`
- `account-credits` — sample: `"Connect"`
- `analytics` — sample: `"Connect"`
- `auth` — sample: `""`
- `chat` — sample: `"Connect"`
- `chat-sample-conv-id` — sample: `""`
- `chat-history` — sample: `""`

### `redirect-chain-differs` — 28 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:3000/`
- `about` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:3000/auth`
- `access-denied` — prod→`1: https://epsx.io/access-denied` dev→`0: http://localhost:3000/access-denied`
- `account` — prod→`1: https://epsx.io/account` dev→`0: http://localhost:3000/account`
- `account-credits` — prod→`1: https://epsx.io/account/credits` dev→`0: http://localhost:3000/account/credits`
- `analytics` — prod→`1: https://epsx.io/analytics` dev→`0: http://localhost:3000/analytics`
- `auth` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:3000/auth`
- `chat` — prod→`1: https://epsx.io/chat` dev→`0: http://localhost:3000/chat`
- `chat-sample-conv-id` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:3000/auth`
- `chat-history` — prod→`1: https://epsx.io/auth` dev→`1: http://localhost:3000/auth`

### `missing-hrefs` — 18 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/UNIABEXAL"`
- `account` — sample: `"/account/credits"`
- `analytics` — sample: `"https://www.tradingview.com/symbols/PH"`

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
