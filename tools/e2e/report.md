# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-17T13:32:15.963Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 13.54 | 0 | 6 | 5 | 6 |
| 2 | `dashboard` | `/dashboard` | 1.58 | 0 | 0 | 2 | 0 |
| 3 | `developer-usage` | `/developer/usage` | 83.66 | 0 | 0 | 2 | 0 |
| 4 | `manual` | `/manual` | 2.04 | 0 | 0 | 2 | 0 |
| 5 | `plans` | `/plans` | 78.01 | 1 | 0 | 3 | 0 |
| 6 | `portfolio` | `/portfolio` | 20.47 | 0 | 0 | 4 | 0 |
| 7 | `privacy` | `/privacy` | 6.17 | 0 | 0 | 2 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 20 | 7 |
| `redirect-chain-differs` | 7 | 7 |
| `missing-hrefs` | 6 | 1 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 20 occurrences

Affected routes (first 10):

- `home` — sample: `"Market"`
- `dashboard` — sample: `"Market"`
- `developer-usage` — sample: `"Market"`
- `manual` — sample: `"Market"`
- `plans` — sample: `"Market"`
- `portfolio` — sample: `"Market"`
- `privacy` — sample: `"Market"`

### `redirect-chain-differs` — 7 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:30199/`
- `dashboard` — prod→`1: https://epsx.io/dashboard` dev→`0: http://localhost:30199/dashboard`
- `developer-usage` — prod→`1: https://epsx.io/developer/usage` dev→`0: http://localhost:30199/developer/usage`
- `manual` — prod→`1: https://epsx.io/manual` dev→`0: http://localhost:30199/manual`
- `plans` — prod→`1: https://epsx.io/plans` dev→`0: http://localhost:30199/plans`
- `portfolio` — prod→`1: https://epsx.io/portfolio` dev→`0: http://localhost:30199/portfolio`
- `privacy` — prod→`1: https://epsx.io/privacy` dev→`0: http://localhost:30199/privacy`

### `missing-hrefs` — 6 occurrences

Affected routes (first 10):

- `home` — sample: `"https://www.tradingview.com/symbols/GHC"`

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
