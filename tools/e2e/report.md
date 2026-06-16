# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-16T05:03:08.338Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 99.94 | 1 | 5 | 4 | 5 |
| 2 | `about` | `/about` | 99.99 | 1 | 0 | 0 | 0 |
| 3 | `access-denied` | `/access-denied` | 99.93 | 1 | 0 | 2 | 0 |
| 4 | `account` | `/account` | 99.82 | 5 | 2 | 3 | 2 |
| 5 | `account-credits` | `/account/credits` | 99.67 | 3 | 0 | 2 | 0 |
| 6 | `analytics` | `/analytics` | 99.69 | 1 | 10 | 10 | 10 |
| 7 | `auth` | `/auth` | 99.96 | 1 | 0 | 0 | 0 |
| 8 | `chat` | `/chat` | 99.84 | 1 | 0 | 3 | 0 |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 98.02 | 1 | 0 | 0 | 0 |
| 10 | `chat-history` | `/chat/history` | 98.68 | 1 | 0 | 0 | 0 |
| 11 | `contact` | `/contact` | 99.76 | 1 | 0 | 0 | 0 |
| 12 | `dashboard` | `/dashboard` | 99.86 | 1 | 0 | 2 | 0 |
| 13 | `developer` | `/developer` | 99.95 | 1 | 0 | 2 | 0 |
| 14 | `developer-usage` | `/developer/usage` | 99.95 | 1 | 0 | 2 | 0 |
| 15 | `developer-docs` | `/developer/docs` | 99.96 | 1 | 0 | 2 | 0 |
| 16 | `manual` | `/manual` | 87.55 | 1 | 0 | 2 | 0 |
| 17 | `news` | `/news` | 99.93 | 1 | 9 | 2 | 9 |
| 18 | `news-sample-slug` | `/news/sample-slug` | 99.7 | 2 | 0 | 4 | 0 |
| 19 | `notifications` | `/notifications` | 99.31 | 1 | 0 | 0 | 0 |
| 20 | `offline` | `/offline` | 99.97 | 1 | 0 | 0 | 0 |
| 21 | `payment` | `/payment` | 99.95 | 1 | 0 | 2 | 0 |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 99.97 | 1 | 0 | 2 | 0 |
| 23 | `permissions` | `/permissions` | 98.75 | 1 | 0 | 0 | 0 |
| 24 | `plans` | `/plans` | 99.12 | 2 | 0 | 3 | 0 |
| 25 | `portfolio` | `/portfolio` | 99.79 | 1 | 0 | 5 | 0 |
| 26 | `privacy` | `/privacy` | 99.99 | 1 | 0 | 2 | 0 |
| 27 | `profile` | `/profile` | 98.78 | 1 | 0 | 0 | 0 |
| 28 | `terms` | `/terms` | 99.99 | 1 | 0 | 2 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 56 | 19 |
| `console-error-dev-only` | 28 | 28 |
| `redirect-chain-differs` | 28 | 28 |
| `missing-hrefs` | 26 | 4 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 56 occurrences

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

### `console-error-dev-only` — 28 occurrences

Affected routes (first 10):

- `home` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `about` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `access-denied` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `account` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `account-credits` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `analytics` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `auth` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `chat` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `chat-sample-conv-id` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`
- `chat-history` — sample: `"[error] Loading the script 'https://unpkg.com/lucide@latest' violates the following Content Security Policy directive: \"script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.jsdelivr.net\". No`

### `redirect-chain-differs` — 28 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:30199/`
- `about` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30199/about`
- `access-denied` — prod→`1: https://epsx.io/access-denied` dev→`0: http://localhost:30199/access-denied`
- `account` — prod→`1: https://epsx.io/account` dev→`0: http://localhost:30199/account`
- `account-credits` — prod→`1: https://epsx.io/account/credits` dev→`0: http://localhost:30199/account/credits`
- `analytics` — prod→`1: https://epsx.io/analytics` dev→`0: http://localhost:30199/analytics`
- `auth` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30199/auth`
- `chat` — prod→`1: https://epsx.io/chat` dev→`0: http://localhost:30199/chat`
- `chat-sample-conv-id` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30199/chat/sample-conv-id`
- `chat-history` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30199/chat/history`

### `missing-hrefs` — 26 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/GHC"`
- `account` — sample: `"/account/credits"`
- `analytics` — sample: `"https://www.tradingview.com/symbols/BHARTIARTL"`
- `news` — sample: `"/news/strategic-roadmap-future"`

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
