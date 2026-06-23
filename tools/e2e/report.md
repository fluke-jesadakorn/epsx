# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-23T07:09:59.629Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 12.38 | 0 | 6 | 4 | 6 |
| 2 | `about` | `/about` | 95.35 | 0 | 0 | 1 | 0 |
| 3 | `access-denied` | `/access-denied` | 59.82 | 0 | 0 | 1 | 0 |
| 4 | `account` | `/account` | 93.85 | 4 | 2 | 2 | 2 |
| 5 | `account-credits` | `/account/credits` | 93.21 | 2 | 0 | 1 | 0 |
| 6 | `analytics` | `/analytics` | 93.83 | 0 | 6 | 10 | 6 |
| 7 | `auth` | `/auth` | 88.63 | 0 | 0 | 1 | 0 |
| 8 | `chat` | `/chat` | 92.26 | 0 | 0 | 2 | 0 |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 96.03 | 0 | 0 | 1 | 0 |
| 10 | `chat-history` | `/chat/history` | 95.29 | 0 | 0 | 1 | 0 |
| 11 | `contact` | `/contact` | 99.97 | 0 | 0 | 1 | 0 |
| 12 | `dashboard` | `/dashboard` | 49.23 | 0 | 0 | 1 | 0 |
| 13 | `developer` | `/developer` | 99.67 | 0 | 0 | 4 | 0 |
| 14 | `developer-usage` | `/developer/usage` | 99.65 | 0 | 0 | 4 | 0 |
| 15 | `developer-docs` | `/developer/docs` | 99.69 | 0 | 0 | 4 | 0 |
| 16 | `manual` | `/manual` | 2.22 | 0 | 0 | 1 | 0 |
| 17 | `news` | `/news` | 93.6 | 0 | 0 | 1 | 0 |
| 18 | `news-sample-slug` | `/news/sample-slug` | 83.16 | 1 | 0 | 3 | 0 |
| 19 | `notifications` | `/notifications` | 91.54 | 0 | 0 | 1 | 0 |
| 20 | `offline` | `/offline` | 98.74 | 0 | 0 | 2 | 0 |
| 21 | `payment` | `/payment` | 99.95 | 0 | 1 | 8 | 1 |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 98.93 | 0 | 0 | 4 | 0 |
| 23 | `permissions` | `/permissions` | 83.69 | 0 | 0 | 1 | 0 |
| 24 | `plans` | `/plans` | 10.97 | 1 | 0 | 1 | 0 |
| 25 | `portfolio` | `/portfolio` | 55.21 | 0 | 0 | 4 | 0 |
| 26 | `privacy` | `/privacy` | 6.25 | 0 | 0 | 1 | 0 |
| 27 | `profile` | `/profile` | 90.95 | 0 | 0 | 1 | 0 |
| 28 | `terms` | `/terms` | 93.79 | 0 | 0 | 1 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 67 | 28 |
| `redirect-chain-differs` | 28 | 28 |
| `missing-hrefs` | 15 | 4 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 67 occurrences

Affected routes (first 10):

- `home` — sample: `"Connect"`
- `about` — sample: `"Connect Wallet"`
- `access-denied` — sample: `"Connect"`
- `account` — sample: `"Connect"`
- `account-credits` — sample: `"Connect"`
- `analytics` — sample: `"Connect"`
- `auth` — sample: `""`
- `chat` — sample: `"Connect"`
- `chat-sample-conv-id` — sample: `"Connect Wallet"`
- `chat-history` — sample: `"Connect Wallet"`

### `redirect-chain-differs` — 28 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:3000/`
- `about` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:3000/about`
- `access-denied` — prod→`1: https://epsx.io/access-denied` dev→`0: http://localhost:3000/access-denied`
- `account` — prod→`1: https://epsx.io/account` dev→`0: http://localhost:3000/account`
- `account-credits` — prod→`1: https://epsx.io/account/credits` dev→`0: http://localhost:3000/account/credits`
- `analytics` — prod→`1: https://epsx.io/analytics` dev→`0: http://localhost:3000/analytics`
- `auth` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:3000/auth`
- `chat` — prod→`1: https://epsx.io/chat` dev→`0: http://localhost:3000/chat`
- `chat-sample-conv-id` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:3000/chat/sample-conv-id`
- `chat-history` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:3000/chat/history`

### `missing-hrefs` — 15 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/UNIABEXAL"`
- `account` — sample: `"/account/credits"`
- `analytics` — sample: `"https://www.tradingview.com/symbols/2317"`
- `payment` — sample: `"/"`

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
