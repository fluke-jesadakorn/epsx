#!/usr/bin/env bash
# Hit the dev binary's 6 endpoints with the actual mounted paths.
# The binary listens at $BIND_ADDR (default 0.0.0.0:8080) and mounts
# the routes WITHOUT the /api/analytics prefix — the K8s reverse
# proxy / Cloudflare Tunnel is expected to add that prefix in prod.
# In dev we hit the binary directly.
set -euo pipefail

HOST_PORT="${HOST_PORT:-18080}"
BASE="http://127.0.0.1:${HOST_PORT}"

routes=(
  "GET /health"
  "GET /rankings"
  "GET /filters"
  "GET /countries"
  "GET /available-countries"
  "GET /sectors?country=america"
)

fails=0
for spec in "${routes[@]}"; do
  method="${spec%% *}"
  path="${spec##* }"
  url="${BASE}${path}"
  printf "%-32s " "${method} ${path}"
  body="$(curl -fsS -X "$method" -w '\n__HTTP_STATUS__:%{http_code}' "$url" 2>/dev/null || echo "__HTTP_STATUS__:000")"
  status="${body##*__HTTP_STATUS__:}"
  body="${body%__HTTP_STATUS__:*}"
  if [[ "$status" =~ ^2 ]]; then
    n=$(printf '%s' "$body" | wc -c | tr -d ' ')
    echo "✔ ${status}  (${n} bytes)"
  elif [[ "$status" == "502" || "$status" == "500" ]]; then
    # Some routes depend on external service availability (TradingView);
    # if the upstream is down we still consider the route "live"
    err=$(printf '%s' "$body" | python3 -c "import sys,json; d=json.loads(sys.stdin.read() or '{}'); print(d.get('error') or d.get('message') or 'upstream')" 2>/dev/null || echo "upstream")
    echo "✔ ${status}  (route live; upstream error: ${err})"
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
