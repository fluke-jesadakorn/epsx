#!/usr/bin/env bash
# diff-admin.sh — diff prod vs dev admin baselines, emit per-slug diffs + a Markdown report.
#
# Usage:
#   bash tools/e2e-admin/diff-admin.sh                    # all 29
#   bash tools/e2e-admin/diff-admin.sh admin-dashboard,admin-settings   # subset
#
# Outputs:
#   tools/e2e-admin/diff-admin/<slug>.json                structured per-slug diff
#   tools/e2e-admin/diff-admin/<slug>.diff.png            pixel-diff visualization
#   tools/e2e-admin/diff-admin/_summary.tsv               one row per slug
#   tools/e2e-admin/report.md                             Markdown table + issue digest
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPTS="$HERE/scripts"
PROD_DIR="$HERE/baselines/prod-admin"
DEV_DIR="$HERE/baselines/dev-admin"
DIFF_DIR="$HERE/diff-admin"
LOG_DIR="$HERE/logs"
PIXEL_DIFF_SH="/Users/fluke/Desktop/Work/epsx/.wave22/pixel-diff.sh"

mkdir -p "$DIFF_DIR" "$LOG_DIR"

PLAYWRIGHT_DIR="/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules"
export NODE_PATH="$PLAYWRIGHT_DIR"

SUBSET="${1:-all}"
if [[ "$SUBSET" == "all" ]]; then
  ROUTES_CSV=$(node -e "
    const r = require('$SCRIPTS/routes.json');
    console.log(r.routes.map(x => x.slug).join(','));
  ")
else
  ROUTES_CSV="$SUBSET"
fi

echo "=== diff-admin ==="
echo "  prod: $PROD_DIR"
echo "  dev:  $DEV_DIR"
echo "  out:  $DIFF_DIR"
echo

: > "$DIFF_DIR/_summary.tsv"

IFS=',' read -ra SLUGS <<<"$ROUTES_CSV"
for slug in "${SLUGS[@]}"; do
  prodPng="$PROD_DIR/${slug}.png"
  devPng="$DEV_DIR/${slug}.png"
  if [[ ! -f "$prodPng" || ! -f "$devPng" ]]; then
    printf '%s\tN/A\tN/A\tskipped (missing artifact)\n' "$slug" | tee -a "$DIFF_DIR/_summary.tsv"
    continue
  fi
  echo "[$slug] diff..."
  if ! node "$SCRIPTS/diff.js" \
    --slug "$slug" \
    --prod "$PROD_DIR" \
    --dev "$DEV_DIR" \
    --out "$DIFF_DIR" \
    --pixel-diff-sh "$PIXEL_DIFF_SH" \
    >"$DIFF_DIR/${slug}.stdout" 2>"$DIFF_DIR/${slug}.stderr"; then
    echo "  diff failed for $slug"
    tail -5 "$DIFF_DIR/${slug}.stderr" | sed 's/^/    /'
    continue
  fi
  metric=$(head -1 "$DIFF_DIR/${slug}.stdout")
  PIXEL_DIFF=$(echo "$metric" | grep -oE 'PIXEL_DIFF=[0-9]+' | sed 's/PIXEL_DIFF=//')
  DIFF_PCT=$(echo "$metric" | grep -oE 'DIFF_PCT=[0-9.]+' | sed 's/DIFF_PCT=//')
  printf '%s\t%s\t%s\n' "$slug" "${PIXEL_DIFF:-0}" "${DIFF_PCT:-0}" | tee -a "$DIFF_DIR/_summary.tsv"
done

# Aggregate per-slug JSON files into a Markdown report
node "$SCRIPTS/report.js" \
  --diff-dir "$DIFF_DIR" \
  --prod-dir "$PROD_DIR" \
  --dev-dir "$DEV_DIR" \
  --routes "$ROUTES_CSV" \
  --out "$HERE/report.md" \
  2>&1 | tee "$LOG_DIR/report.log"
