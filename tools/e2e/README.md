# tools/e2e — Wave 23 T1 E2E Component Interaction Harness

Captures 28 prod + 28 dev routes, clicks every interactive element, records
what happened, then diffs prod vs dev to find broken / missing / different
components.

## Quick start

```bash
# Subset smoke (3 routes) — ~1 min total
bash tools/e2e/capture-prod.sh home,about,auth
bash tools/e2e/capture-dev.sh  home,about,auth
bash tools/e2e/diff.sh        home,about,auth

# Full 28
bash tools/e2e/capture-prod.sh                 # 10-15 min
bash tools/e2e/capture-dev.sh                  # 10-15 min
bash tools/e2e/diff.sh                         #  5 min
cat tools/e2e/report.md                        # read
```

## Auth

- `EPSX_AUTH_COOKIE='session=...; csrf=...'` — full Cookie header for prod.
- `EPSX_AUTH_BYPASS_DEV=1` — add `0x...d3v1` dev-bypass cookie for dev BFF.
- If no auth: public routes are captured; gated routes 307 to /auth and the
  capture is recorded (redirects.log shows the 307 chain).

## Artifacts (per route, per environment)

```
baselines/{prod,dev}/<slug>.png                 1280x800 screenshot
baselines/{prod,dev}/<slug>.html                post-hydration HTML
baselines/{prod,dev}/<slug>.console.log         browser console + pageerror
baselines/{prod,dev}/<slug>.interactions.jsonl  one JSON per click
baselines/{prod,dev}/<slug>.network.jsonl       request/response/failed
baselines/{prod,dev}/<slug>.redirects.log       framenavigated chain
```

## Diff artifacts

```
diff/<slug>.json                  structured per-slug diff
diff/<slug>.diff.png              pixel-diff visualization
diff/_summary.tsv                 pixel_diff per slug
report.md                         Markdown summary + issue digest
```

## Issue kinds (consumed by report.md)

| Kind | Meaning |
|------|---------|
| `console-error-dev-only` | console error in dev that prod did not emit |
| `broken-clicks` | dev click threw / element stale |
| `click-does-not-navigate` | dev click did not change URL when prod's same-selector click did |
| `missing-hrefs` | dev HTML lacks an `<a href>` that prod has |
| `missing-buttons` | dev HTML lacks a `<button>` text that prod has |
| `redirect-chain-differs` | prod 307 chain ended at a different URL than dev |

## Layout

```
tools/e2e/
├── capture-prod.sh            # shell wrapper
├── capture-dev.sh             # shell wrapper
├── diff.sh                    # shell wrapper
├── scripts/
│   ├── routes.json            # 28 routes from .wave22/ROUTES.md
│   ├── capture.js             # Playwright runner
│   ├── diff.js                # per-slug diff (pixel + interaction + HTML)
│   └── report.js              # Markdown assembly
├── specs/                     # reserved (1 parameterized spec)
├── baselines/                 # gitignored, populated by capture-*.sh
├── diff/                      # gitignored, populated by diff.sh
├── logs/                      # gitignored
└── report.md                  # gitignored, generated
```

## Pre-flight

```bash
# playwright must be installed (wave 22 left it in apps-old/frontend)
ls /Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules/playwright

# chromium-headless-shell must be cached
ls /Users/fluke/Library/Caches/ms-playwright/chromium_headless_shell-1208/

# dev BFF must be reachable (or set EPSX_DEV_BASE)
curl -sf http://localhost:30101/
```
