#!/usr/bin/env bash
# Hit the dev container's 5 user-facing routes + /health.
# Assumes scripts/dev-up.sh has been run.
set -euo pipefail

HOST_PORT="${HOST_PORT:-18080}"
BASE="http://localhost:${HOST_PORT}"

routes=(
  "GET /health"
  "GET /api/analytics/rankings"
  "GET /api/analytics/filters"
  "GET /api/analytics/countries"
  "GET /api/analytics/available-countries"
  "GET /api/analytics/sectors"
)

fails=0
for spec in "${routes[@]}"; do
  method="${spec%% *}"
  path="${spec##* }"
  url="${BASE}${path}"
  printf "%-6s %-44s " "$method" "$path"
  body="$(curl -fsS -X "$method" -w '\n__HTTP_STATUS__:%{http_code}' "$url" 2>/dev/null || echo "__HTTP_STATUS__:000")"
  status="${body##*__HTTP_STATUS__:}"
  body="${body%__HTTP_STATUS__:*}"
  if [[ "$status" =~ ^2 ]]; then
    n=$(printf '%s' "$body" | wc -c | tr -d ' ')
    echo "✔ ${status}  (${n} bytes)"
  elif [[ "$status" == "401" || "$status" == "403" ]]; then
    # Some routes require auth — that's a 200 on the route, not a fail
    echo "✔ ${status}  (auth-gated; route present, see message body for expected auth scheme)"
  elif [[ "$status" == "404" ]]; then
    echo "✘ ${status}  (route missing!)"
    fails=$((fails+1))
  else
    echo "✘ ${status}  (unexpected; first 200 bytes: $(printf '%s' "$body" | head -c 200))"
    fails=$((fails+1))
  fi
done

echo
if [[ $fails -eq 0 ]]; then
  echo "✔ all 6 endpoints reachable"
  exit 0
else
  echo "✘ $fails endpoint(s) failed"
  exit 1
fi
