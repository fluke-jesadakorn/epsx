# tools/e2e-admin — Wave 24 T1' Admin E2E Component Interaction Harness

Captures 29 prod + 29 dev admin routes, clicks every interactive element,
records what happened, then diffs prod vs dev to find broken / missing /
different components. Mirrors the wave-23 frontend harness at
`tools/e2e/`, but for `https://admin.epsx.io`.

## Quick start

```bash
# Subset smoke (3 routes) — ~1 min total
bash tools/e2e-admin/capture-prod-admin.sh admin-dashboard,admin-settings,admin-policies
bash tools/e2e-admin/capture-dev-admin.sh  admin-dashboard,admin-settings,admin-policies
bash tools/e2e-admin/diff-admin.sh         admin-dashboard,admin-settings,admin-policies

# Full 29
bash tools/e2e-admin/capture-prod-admin.sh                 # 10-15 min
bash tools/e2e-admin/capture-dev-admin.sh                  # 10-15 min
bash tools/e2e-admin/diff-admin.sh                         #  5 min
cat tools/e2e-admin/report.md                              # read
```

## Auth

- `EPSX_ADMIN_PROD_COOKIE='session=...; csrf=...'` — full Cookie header for
  prod admin (use admin account; needed for routes that 307 to `/auth` when
  unauth).
- `EPSX_DEV_AUTH_BYPASS=1` — add `0x...d3v1` dev-bypass cookie for dev BFF
  (mirrors frontend `EPSX_DEV_AUTH_BYPASS`).
- If no auth: public routes are captured; gated routes 307 to `/auth` and the
  capture is recorded (redirects.log shows the 307 chain).

## Artifacts (per route, per environment)

```
baselines/{prod-admin,dev-admin}/<slug>.png                 1280x800 screenshot
baselines/{prod-admin,dev-admin}/<slug>.html                post-hydration HTML
baselines/{prod-admin,dev-admin}/<slug>.console.log         browser console + pageerror
baselines/{prod-admin,dev-admin}/<slug>.interactions.jsonl  one JSON per click
baselines/{prod-admin,dev-admin}/<slug>.network.jsonl       request/response/failed
baselines/{prod-admin,dev-admin}/<slug>.redirects.log       framenavigated chain
baselines/{prod-admin,dev-admin}/_summary.json              per-slug status + ok flag
```

## Diff artifacts

```
diff-admin/<slug>.json                  structured per-slug diff
diff-admin/<slug>.diff.png              pixel-diff visualization
diff-admin/_summary.tsv                 pixel_diff per slug
report.md                               Markdown summary + issue digest
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
| `dev-returns-404` | dev BFF returned 404 for this route (admin Dioxus shell is mostly empty; expected initially) |

## Layout

```
tools/e2e-admin/
├── capture-prod-admin.sh         # shell wrapper
├── capture-dev-admin.sh          # shell wrapper
├── diff-admin.sh                 # shell wrapper
├── scripts/
│   ├── routes.json               # 29 admin routes from apps-old/admin-frontend
│   ├── capture.js                # Playwright runner
│   ├── diff.js                   # per-slug diff (pixel + interaction + HTML)
│   └── report.js                 # Markdown assembly
├── specs/                        # reserved
├── baselines/                    # gitignored, populated by capture-*.sh
├── diff-admin/                   # gitignored, populated by diff-admin.sh
├── logs/                         # gitignored
└── report.md                     # gitignored, generated
```

## Routes (29)

`scripts/routes.json` is the source of truth. As of wave 24 T1':

```
/                                                  (admin home)
/access-denied, /unauthorized, /auth
/dashboard, /settings, /policies, /analytics, /audit-log
/chat, /chat/[id]
/developer-portal, /developer-portal/api-keys/create
/media
/news, /news/create, /news/[id]/edit
/notifications, /notifications/create, /notifications/manage
/payments
/wallet-management, /wallet-management/access
/wallet-management/access/plans, /wallet-management/access/plans/[planId]
/wallet-management/credits
/wallet-management/[address]
/wallet-management/wallets, /wallet-management/wallets/[address]/disable
```

Dynamic segments use `sample-conv-id`, `sample-id`, `sample-plan-id`, and
`0x000000000000000000000000000000000000d3c0` (the wave-23 dev-bypass
address) as fixture values.

## Pre-flight

```bash
# playwright must be installed
ls /Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules/playwright

# chromium-headless-shell must be cached
ls /Users/fluke/Library/Caches/ms-playwright/chromium_headless_shell-1208/

# dev BFF must be reachable (or set EPSX_DEV_BASE)
curl -sf http://localhost:3001/
```

## Expected initial pass: many dev routes return 404

The Dioxus admin frontend is at the start of the wave-24 work — most
admin routes are not yet ported. The harness **records `status=404` and a
`dev-returns-404` issue kind** for routes that don't exist yet, then
continues. This is structural, not a bug. As T2/T3/T4 wave-24 tracks port
admin pages, the dev-404 count should drop.
