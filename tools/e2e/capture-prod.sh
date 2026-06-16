#!/usr/bin/env bash
# capture-prod.sh — capture all 28 prod routes via Playwright Node runner.
#
# Output: tools/e2e/baselines/prod/<slug>.{png,html,console.log,interactions.jsonl,network.jsonl,redirects.log}
#
# Usage:
#   bash tools/e2e/capture-prod.sh                 # full 28
#   bash tools/e2e/capture-prod.sh 1,2,3           # subset (CSV of slugs)
#   EPSX_AUTH_COOKIE='session=...; csrf=...' bash tools/e2e/capture-prod.sh   # authenticated
#
# Env:
#   EPSX_AUTH_COOKIE   optional Cookie header for authenticated routes
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPTS="$HERE/scripts"
OUT_DIR="$HERE/baselines/prod"
LOG_DIR="$HERE/logs"
mkdir -p "$OUT_DIR" "$LOG_DIR"

# Resolve playwright module from apps-old/frontend (already installed in wave 22)
PLAYWRIGHT_DIR="/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules"
if [[ ! -d "$PLAYWRIGHT_DIR/playwright" ]]; then
  echo "capture-prod.sh: playwright not found at $PLAYWRIGHT_DIR" >&2
  echo "  (wave 22 used chromium-headless-shell, but we now use the @playwright/test npm module)" >&2
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

echo "=== capture-prod ==="
echo "  base:  https://epsx.io"
echo "  out:   $OUT_DIR"
echo "  routes: $ROUTES_CSV"
echo "  cookie: ${EPSX_AUTH_COOKIE:-(none)}"
echo

node "$SCRIPTS/capture.js" \
  --base "https://epsx.io" \
  --out "$OUT_DIR" \
  --routes "$ROUTES_CSV" \
  --viewport "1280x800" \
  --wait-ms "4000" \
  --click-budget "30" \
  2>&1 | tee "$LOG_DIR/capture-prod.log"
