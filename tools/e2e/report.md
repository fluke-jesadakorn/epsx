# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-16T01:42:06.218Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 99.83 | 1 | 5 | 3 | 5 |
| 2 | `about` | `/about` | 99.93 | 1 | 0 | 1 | 0 |
| 3 | `access-denied` | `/access-denied` | 93.8 | 1 | 0 | 1 | 0 |
| 4 | `account` | `/account` | 84.02 | 5 | 2 | 2 | 2 |
| 5 | `account-credits` | `/account/credits` | 93.72 | 3 | 0 | 1 | 0 |
| 6 | `analytics` | `/analytics` | 74.37 | 1 | 10 | 9 | 10 |
| 7 | `auth` | `/auth` | 99.96 | 1 | 0 | 0 | 0 |
| 8 | `chat` | `/chat` | 93.79 | 1 | 0 | 2 | 0 |
| 9 | `chat-sample-conv-id` | `/chat/sample-conv-id` | 94.98 | 1 | 0 | 1 | 0 |
| 10 | `chat-history` | `/chat/history` | 97.1 | 1 | 0 | 1 | 0 |
| 11 | `contact` | `/contact` | 99.99 | 1 | 0 | 1 | 0 |
| 12 | `dashboard` | `/dashboard` | 93.39 | 1 | 0 | 1 | 0 |
| 13 | `developer` | `/developer` | 99.82 | 1 | 0 | 1 | 0 |
| 14 | `developer-usage` | `/developer/usage` | 99.8 | 1 | 0 | 1 | 0 |
| 15 | `developer-docs` | `/developer/docs` | 99.79 | 1 | 0 | 1 | 0 |
| 16 | `manual` | `/manual` | 91.76 | 7 | 0 | 1 | 0 |
| 17 | `news` | `/news` | 91.06 | 1 | 10 | 1 | 10 |
| 18 | `news-sample-slug` | `/news/sample-slug` | 93.8 | 2 | 0 | 3 | 0 |
| 19 | `notifications` | `/notifications` | 97.23 | 1 | 0 | 1 | 0 |
| 20 | `offline` | `/offline` | 99.97 | 1 | 0 | 0 | 0 |
| 21 | `payment` | `/payment` | 98.84 | 1 | 0 | 1 | 0 |
| 22 | `payment-intent-sample-id` | `/payment/intent/sample-id` | 98.74 | 1 | 0 | 1 | 0 |
| 23 | `permissions` | `/permissions` | 99.28 | 1 | 0 | 1 | 0 |
| 24 | `plans` | `/plans` | 81.42 | 2 | 0 | 2 | 0 |
| 25 | `portfolio` | `/portfolio` | 72.47 | 1 | 0 | 4 | 0 |
| 26 | `privacy` | `/privacy` | 93.79 | 1 | 0 | 1 | 0 |
| 27 | `profile` | `/profile` | 98.88 | 1 | 0 | 1 | 0 |
| 28 | `terms` | `/terms` | 93.79 | 1 | 0 | 1 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 37 | 19 |
| `console-error-dev-only` | 34 | 28 |
| `broken-clicks` | 31 | 26 |
| `redirect-chain-differs` | 28 | 28 |
| `missing-hrefs` | 27 | 4 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 37 occurrences

Affected routes (first 10):

- `home` — sample: `"Connect"`
- `access-denied` — sample: `"Connect"`
- `account` — sample: `"Connect"`
- `account-credits` — sample: `"Connect"`
- `analytics` — sample: `"Connect"`
- `chat` — sample: `"Connect"`
- `dashboard` — sample: `"Connect"`
- `developer` — sample: `"Connect"`
- `developer-usage` — sample: `"Connect"`
- `developer-docs` — sample: `"Connect"`

### `console-error-dev-only` — 34 occurrences

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

### `broken-clicks` — 31 occurrences

Affected routes (first 10):

- `home` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `about` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `access-denied` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `account` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `account-credits` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `analytics` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `chat` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `chat-sample-conv-id` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `chat-history` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`
- `contact` — sample: `{"selector":"a.navbar-brand","text":"EPSX","error":"elementHandle.click: Timeout 2000ms exceeded.\nCall log:\n  - attempting click action\n    2 × waiting for element to be visible, enabled and stable`

### `redirect-chain-differs` — 28 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:30101/`
- `about` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30101/about`
- `access-denied` — prod→`1: https://epsx.io/access-denied` dev→`0: http://localhost:30101/access-denied`
- `account` — prod→`1: https://epsx.io/account` dev→`0: http://localhost:30101/account`
- `account-credits` — prod→`1: https://epsx.io/account/credits` dev→`0: http://localhost:30101/account/credits`
- `analytics` — prod→`1: https://epsx.io/analytics` dev→`0: http://localhost:30101/analytics`
- `auth` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30101/auth`
- `chat` — prod→`1: https://epsx.io/chat` dev→`0: http://localhost:30101/chat`
- `chat-sample-conv-id` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30101/chat/sample-conv-id`
- `chat-history` — prod→`1: https://epsx.io/auth` dev→`0: http://localhost:30101/chat/history`

### `missing-hrefs` — 27 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/APIC"`
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
