# Wave 23 T1 — E2E Component Interaction Report

Generated: 2026-06-23T07:57:04.541Z

Prod: `https://epsx.io`  |  Dev: BFF port-forward (default `localhost:30101`)

## Per-route summary

| # | Slug | Path | pixel_diff_% | console_errors_dev | broken_links_dev | broken_buttons_dev | missing_components |
|---|------|------|-------------:|-------------------:|-----------------:|-------------------:|-------------------:|
| 1 | `portfolio` | `/portfolio` | 55.21 | 0 | 0 | 4 | 0 |

## Issue digest (aggregated across routes)

| Kind | Total occurrences | Routes affected |
|------|------------------:|----------------:|
| `missing-buttons` | 4 | 1 |
| `redirect-chain-differs` | 1 | 1 |

## Top 5 issues (by occurrence)

### `missing-buttons` — 4 occurrences

Affected routes (first 10):

- `portfolio` — sample: `"Connect"`

### `redirect-chain-differs` — 1 occurrences

Affected routes (first 10):

- `portfolio` — prod→`1: https://epsx.io/portfolio` dev→`0: http://localhost:3000/portfolio`

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
