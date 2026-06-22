#!/usr/bin/env bash
# start-bff-dev.sh - start the dev BFF with the env vars used for historical
# E2E pixel-diff measurement (matches Wave 25-44 baseline config).
#
# USES:
#   - EPSX_AUTH_BYPASS_DEV=1  capture script adds dev-bypass cookie
#   - EPSX_E2E_SKELETON=1     render skeleton for unauthed content
#   - EPSX_ENABLE_DEMO_LOGIN=1 admin only
#
# DOES NOT use EPSX_DEV_AUTH_BYPASS=1 (would force-unauth on BFF and break
# routes like /portfolio that match prod's anon landing).
#
# Usage:
#   bash tools/e2e/start-bff-dev.sh                # frontend on :3000
#   bash tools/e2e/start-bff-dev.sh admin          # admin on :3001
set -e

ROLE="${1:-frontend}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

case "$ROLE" in
  frontend|fe)
    PORT=3000
    BIN="target/release/bff-frontend"
    LOG="/tmp/bff-frontend.log"
    ;;
  admin)
    PORT=3001
    BIN="target/release/bff-admin"
    LOG="/tmp/bff-admin.log"
    ;;
  *)
    echo "Usage: $0 [frontend|admin]"
    exit 1
    ;;
esac

# Default env vars matching historical E2E baseline
export EPSX_AUTH_BYPASS_DEV="${EPSX_AUTH_BYPASS_DEV:-1}"
export EPSX_E2E_SKELETON="${EPSX_E2E_SKELETON:-1}"
# Admin also needs these
export EPSX_ENABLE_DEMO_LOGIN="${EPSX_ENABLE_DEMO_LOGIN:-1}"

cd "$ROOT"

# Kill any existing BFF on this port
lsof -ti:$PORT 2>/dev/null | xargs -r kill -9 2>/dev/null || true
sleep 1

# Start
env EPSX_AUTH_BYPASS_DEV="$EPSX_AUTH_BYPASS_DEV" \
    EPSX_E2E_SKELETON="$EPSX_E2E_SKELETON" \
    EPSX_ENABLE_DEMO_LOGIN="$EPSX_ENABLE_DEMO_LOGIN" \
    nohup ./$BIN > "$LOG" 2>&1 &

sleep 4
echo "Started $BIN on :$PORT (pid=$!)"
echo "Log: $LOG"
curl -s -o /dev/null -w "Smoke test: HTTP %{http_code}\n" "http://localhost:$PORT/"
