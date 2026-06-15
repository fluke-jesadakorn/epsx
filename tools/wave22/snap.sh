#!/usr/bin/env bash
# snap.sh — capture a single screenshot of a URL using playwright headless shell
#
# Usage:
#   snap.sh <url> [output.png] [user-data-dir-name]
#
# Exit codes:
#   0  PNG > 1KB (success)
#   1  timeout / network error / PNG too small
#   2  usage error
#
# Notes:
#   - pkill between shots to avoid the "DevTools listening on..." / zombie hangs.
#   - --virtual-time-budget=8000 freezes the page clock for 8s so async hydration
#     paints complete before the screenshot is taken.
set -u

URL="${1:-}"
OUT="${2:-/tmp/epsx-snap.png}"
UDIR_NAME="${3:-$(date +%s)-$$}"
CHROME="/Users/fluke/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell"
UDIR="/tmp/hs-${UDIR_NAME}"

if [[ -z "$URL" ]]; then
  echo "usage: $0 <url> [output.png] [user-data-dir-name]" >&2
  exit 2
fi

if [[ ! -x "$CHROME" ]]; then
  echo "snap.sh: chrome-headless-shell not found at $CHROME" >&2
  exit 2
fi

# kill any previous chrome-headless-shell from a prior shot
pkill -f chrome-headless-shell 2>/dev/null || true
sleep 0.5

rm -rf "$UDIR"
mkdir -p "$(dirname "$OUT")"

# 30s outer timeout for the chrome process itself
timeout 30 "$CHROME" \
  --no-sandbox \
  --disable-gpu \
  --user-data-dir="$UDIR" \
  --hide-scrollbars \
  --window-size=1280,800 \
  --virtual-time-budget=8000 \
  --screenshot="$OUT" \
  "$URL" \
  >/tmp/snap-stderr.log 2>&1
RC=$?

# also kill the chrome after the screenshot — leaving it running can wedge
# the next call
pkill -f chrome-headless-shell 2>/dev/null || true
sleep 0.2

# PNG size check
if [[ -f "$OUT" ]]; then
  SIZE=$(stat -f%z "$OUT" 2>/dev/null || stat -c%s "$OUT" 2>/dev/null || echo 0)
else
  SIZE=0
fi

if [[ $RC -ne 0 ]]; then
  echo "snap.sh: chrome exit=$RC size=$SIZE url=$URL" >&2
  echo "--- stderr ---" >&2
  tail -5 /tmp/snap-stderr.log >&2 || true
  exit 1
fi

if [[ "$SIZE" -le 1024 ]]; then
  echo "snap.sh: PNG too small (size=$SIZE url=$URL)" >&2
  exit 1
fi

echo "$OUT $SIZE"
exit 0
