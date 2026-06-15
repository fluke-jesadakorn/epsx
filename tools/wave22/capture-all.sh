#!/usr/bin/env bash
# capture-all.sh — capture all 28 prod routes via snap.sh
set -u

BASE="https://epsx.io"
OUT_DIR="/Users/fluke/Desktop/Work/epsx/.wave22/prod-baseline"
SNAP="/Users/fluke/Desktop/Work/epsx/.wave22/snap.sh"

mkdir -p "$OUT_DIR"

# (path|slug) pairs
ROUTES=(
  "/|home"
  "/about|about"
  "/access-denied|access-denied"
  "/account|account"
  "/account/credits|account-credits"
  "/analytics|analytics"
  "/auth|auth"
  "/chat|chat"
  "/chat/sample-conv-id|chat-sample-conv-id"
  "/chat/history|chat-history"
  "/contact|contact"
  "/dashboard|dashboard"
  "/developer|developer"
  "/developer/usage|developer-usage"
  "/developer/docs|developer-docs"
  "/manual|manual"
  "/news|news"
  "/news/sample-slug|news-sample-slug"
  "/notifications|notifications"
  "/offline|offline"
  "/payment|payment"
  "/payment/intent/sample-id|payment-intent-sample-id"
  "/permissions|permissions"
  "/plans|plans"
  "/portfolio|portfolio"
  "/privacy|privacy"
  "/profile|profile"
  "/terms|terms"
)

SUCCESS=0
FAIL=0
declare -a FAILED_ROUTES=()

for entry in "${ROUTES[@]}"; do
  path="${entry%%|*}"
  slug="${entry##*|}"
  url="${BASE}${path}"
  out="${OUT_DIR}/${slug}.png"
  udir="wave22-${slug}"

  printf "[%02d/%02d] %-44s -> %s\n" "$((SUCCESS + FAIL + 1))" "${#ROUTES[@]}" "$url" "$(basename "$out")"
  if bash "$SNAP" "$url" "$out" "$udir" >/tmp/snap-${slug}.log 2>&1; then
    SIZE=$(stat -f%z "$out" 2>/dev/null || echo 0)
    printf "  OK  %d bytes\n" "$SIZE"
    SUCCESS=$((SUCCESS+1))
  else
    printf "  FAIL\n"
    cat /tmp/snap-${slug}.log | tail -3 | sed 's/^/    /'
    FAILED_ROUTES+=("$slug")
    FAIL=$((FAIL+1))
  fi
done

echo
echo "=== summary ==="
echo "success: $SUCCESS / ${#ROUTES[@]}"
echo "failed:  $FAIL"
if (( FAIL > 0 )); then
  echo "failed routes: ${FAILED_ROUTES[*]}"
fi
