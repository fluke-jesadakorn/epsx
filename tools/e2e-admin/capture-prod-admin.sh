#!/usr/bin/env bash
# capture-prod-admin.sh — capture all 29 prod admin routes via Playwright Node runner.
#
# Output: tools/e2e-admin/baselines/prod-admin/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
#
# Usage:
#   bash tools/e2e-admin/capture-prod-admin.sh                 # full 29
#   bash tools/e2e-admin/capture-prod-admin.sh admin-dashboard,admin-settings
#   EPSX_ADMIN_PROD_COOKIE='session=...; csrf=...' bash tools/e2e-admin/capture-prod-admin.sh
#
# Env:
#   EPSX_ADMIN_PROD_COOKIE   optional Cookie header for authenticated admin routes
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPTS="$HERE/scripts"
OUT_DIR="$HERE/baselines/prod-admin"
LOG_DIR="$HERE/logs"
mkdir -p "$OUT_DIR" "$LOG_DIR"

# Resolve playwright module from apps-old/frontend (already installed in wave 22)
PLAYWRIGHT_DIR="/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules"
if [[ ! -d "$PLAYWRIGHT_DIR/playwright" ]]; then
  echo "capture-prod-admin.sh: playwright not found at $PLAYWRIGHT_DIR" >&2
  exit 2
fi
export NODE_PATH="$PLAYWRIGHT_DIR"

# Subset filter
SUBSET="${1:-all}"
if [[ "$SUBSET" == "all" ]]; then
  ROUTES_CSV=$(node -e "
    const r = require('$SCRIPTS/routes.json');
    console.log(r.routes.map(x => x.slug).join(','));
  ")
else
  ROUTES_CSV="$SUBSET"
fi

echo "=== capture-prod-admin ==="
echo "  base:  https://admin.epsx.io"
echo "  out:   $OUT_DIR"
echo "  routes: $ROUTES_CSV"
echo "  cookie: ${EPSX_ADMIN_PROD_COOKIE:-(none)}"
echo

node "$SCRIPTS/capture.js" \
  --base "https://admin.epsx.io" \
  --out "$OUT_DIR" \
  --routes "$ROUTES_CSV" \
  --viewport "1280x800" \
  --wait-ms "4000" \
  --click-budget "30" \
  2>&1 | tee "$LOG_DIR/capture-prod-admin.log"
