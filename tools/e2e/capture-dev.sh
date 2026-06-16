#!/usr/bin/env bash
# capture-dev.sh — capture all 28 dev BFF routes via Playwright Node runner.
#
# Output: tools/e2e/baselines/dev/<slug>.{png,html,console.log,...}
#
# Usage:
#   bash tools/e2e/capture-dev.sh                  # full 28
#   bash tools/e2e/capture-dev.sh 1,2,3            # subset
#   EPSX_DEV_BASE=http://localhost:30101 bash tools/e2e/capture-dev.sh
#   EPSX_AUTH_BYPASS_DEV=1 bash tools/e2e/capture-dev.sh
#
# Env:
#   EPSX_DEV_BASE          base URL (default: http://localhost:30101)
#   EPSX_AUTH_BYPASS_DEV    set to "1" to send 0x...d3v1 dev-bypass cookie
#   EPSX_AUTH_COOKIE        full Cookie header (overrides bypass)
#
# Pre-flight: dev BFF must be reachable. If K8s port-forward is needed:
#   kubectl port-forward -n epsx-dev svc/epsx-frontend 30101:3000 &
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPTS="$HERE/scripts"
OUT_DIR="$HERE/baselines/dev"
LOG_DIR="$HERE/logs"
mkdir -p "$OUT_DIR" "$LOG_DIR"

BASE="${EPSX_DEV_BASE:-http://localhost:30101}"

PLAYWRIGHT_DIR="/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules"
if [[ ! -d "$PLAYWRIGHT_DIR/playwright" ]]; then
  echo "capture-dev.sh: playwright not found at $PLAYWRIGHT_DIR" >&2
  exit 2
fi
export NODE_PATH="$PLAYWRIGHT_DIR"

# Pre-flight: check dev BFF reachable
if ! curl -sf -o /dev/null --max-time 5 "$BASE/"; then
  echo "capture-dev.sh: dev BFF not reachable at $BASE" >&2
  echo "  hint: kubectl port-forward -n epsx-dev svc/epsx-frontend 30101:3000 &" >&2
  exit 3
fi

SUBSET="${1:-all}"
if [[ "$SUBSET" == "all" ]]; then
  ROUTES_CSV=$(node -e "
    const r = require('$SCRIPTS/routes.json');
    console.log(r.routes.map(x => x.slug).join(','));
  ")
else
  ROUTES_CSV="$SUBSET"
fi

echo "=== capture-dev ==="
echo "  base:  $BASE"
echo "  out:   $OUT_DIR"
echo "  routes: $ROUTES_CSV"
echo "  bypass: ${EPSX_AUTH_BYPASS_DEV:-off}  cookie: ${EPSX_AUTH_COOKIE:-(none)}"
echo

node "$SCRIPTS/capture.js" \
  --base "$BASE" \
  --out "$OUT_DIR" \
  --routes "$ROUTES_CSV" \
  --viewport "1280x800" \
  --wait-ms "4000" \
  --click-budget "30" \
  2>&1 | tee "$LOG_DIR/capture-dev.log"
