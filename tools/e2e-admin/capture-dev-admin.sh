#!/usr/bin/env bash
# capture-dev-admin.sh — capture all 29 dev admin BFF routes via Playwright Node runner.
#
# Output: tools/e2e-admin/baselines/dev-admin/<slug>.{png,html,console.log,...}
#
# Usage:
#   bash tools/e2e-admin/capture-dev-admin.sh                  # full 29
#   bash tools/e2e-admin/capture-dev-admin.sh admin-dashboard  # single
#   EPSX_DEV_BASE=http://localhost:3001 bash tools/e2e-admin/capture-dev-admin.sh
#   EPSX_DEV_AUTH_BYPASS=1 bash tools/e2e-admin/capture-dev-admin.sh
#
# Env:
#   EPSX_DEV_BASE          base URL (default: http://localhost:3001)
#   EPSX_DEV_AUTH_BYPASS   set to "1" to send 0x...d3v1 dev-bypass cookie
#   EPSX_AUTH_COOKIE       full Cookie header (overrides bypass)
#
# Pre-flight: dev BFF must be reachable. If K8s port-forward is needed:
#   kubectl port-forward -n epsx-dev svc/epsx-admin 3001:3000 &
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPTS="$HERE/scripts"
OUT_DIR="$HERE/baselines/dev-admin"
LOG_DIR="$HERE/logs"
mkdir -p "$OUT_DIR" "$LOG_DIR"

BASE="${EPSX_DEV_BASE:-http://localhost:3001}"

PLAYWRIGHT_DIR="/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules"
if [[ ! -d "$PLAYWRIGHT_DIR/playwright" ]]; then
  echo "capture-dev-admin.sh: playwright not found at $PLAYWRIGHT_DIR" >&2
  exit 2
fi
export NODE_PATH="$PLAYWRIGHT_DIR"

# Pre-flight: check dev BFF reachable (allow 4xx — 404 from stub is OK)
HTTP_CODE=$(curl -s -o /dev/null --max-time 5 -w '%{http_code}' "$BASE/" 2>/dev/null || echo "000")
if [[ -z "$HTTP_CODE" || "$HTTP_CODE" == "000" ]]; then
  echo "capture-dev-admin.sh: dev BFF not reachable at $BASE" >&2
  echo "  hint: kubectl port-forward -n epsx-dev svc/epsx-admin 3001:3000 &" >&2
  echo "        (or use tools/e2e-admin/scripts/stub-dev-admin.js for a local stub)" >&2
  exit 3
fi
echo "  pre-flight: $BASE/ -> HTTP $HTTP_CODE (4xx is OK for stub/dev-empty)"

SUBSET="${1:-all}"
if [[ "$SUBSET" == "all" ]]; then
  ROUTES_CSV=$(node -e "
    const r = require('$SCRIPTS/routes.json');
    console.log(r.routes.map(x => x.slug).join(','));
  ")
else
  ROUTES_CSV="$SUBSET"
fi

echo "=== capture-dev-admin ==="
echo "  base:  $BASE"
echo "  out:   $OUT_DIR"
echo "  routes: $ROUTES_CSV"
echo "  bypass: ${EPSX_DEV_AUTH_BYPASS:-off}  cookie: ${EPSX_AUTH_COOKIE:-(none)}"
echo

node "$SCRIPTS/capture.js" \
  --base "$BASE" \
  --out "$OUT_DIR" \
  --routes "$ROUTES_CSV" \
  --viewport "1280x800" \
  --wait-ms "4000" \
  --click-budget "30" \
  2>&1 | tee "$LOG_DIR/capture-dev-admin.log"
