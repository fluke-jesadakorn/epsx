# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-22T13:18:13.218Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `home` | `/` | 12.28 | 0 | 3 | 5 | 3 |
| 2 | `dashboard` | `/dashboard` | 1.58 | 0 | 0 | 1 | 0 |
| 3 | `manual` | `/manual` | 2.04 | 0 | 0 | 1 | 0 |
| 4 | `plans` | `/plans` | 22.55 | 1 | 0 | 2 | 0 |
| 5 | `portfolio` | `/portfolio` | 20.47 | 0 | 0 | 3 | 0 |
| 6 | `privacy` | `/privacy` | 6.18 | 0 | 0 | 1 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 13 | 6 |
| `redirect-chain-differs` | 6 | 6 |
| `missing-hrefs` | 3 | 1 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 13 occurrences

Affected routes (first 10):

- `home` — sample: `"Connect"`
- `dashboard` — sample: `"Connect"`
- `manual` — sample: `"Connect"`
- `plans` — sample: `"Connect"`
- `portfolio` — sample: `"Connect"`
- `privacy` — sample: `"Connect"`

### `redirect-chain-differs` — 6 occurrences

Affected routes (first 10):

- `home` — prod→`1: https://epsx.io/` dev→`0: http://localhost:3000/`
- `dashboard` — prod→`1: https://epsx.io/dashboard` dev→`0: http://localhost:3000/dashboard`
- `manual` — prod→`1: https://epsx.io/manual` dev→`0: http://localhost:3000/manual`
- `plans` — prod→`1: https://epsx.io/plans` dev→`0: http://localhost:3000/plans`
- `portfolio` — prod→`1: https://epsx.io/portfolio` dev→`0: http://localhost:3000/portfolio`
- `privacy` — prod→`1: https://epsx.io/privacy` dev→`0: http://localhost:3000/privacy`

### `missing-hrefs` — 3 occurrences

Affected routes (first 10):

- `home` — sample: `"/news/optimizing-high-throughput-analytics-rust"`

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
