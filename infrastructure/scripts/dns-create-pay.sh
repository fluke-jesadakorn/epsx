#!/bin/bash
# ============================================================================
# wave49(slice-6): Create Cloudflare DNS CNAME records for pay.epsx.io
#
# Creates three CNAMEs pointing at the production Cloudflare Tunnel:
#   - pay.epsx.io         → pay (prod, via tunnel 6bee9b58-...)
#   - staging-pay.epsx.io → pay (staging, same tunnel)
#   - dev-pay.epsx.io     → pay (dev, same tunnel)
#
# All three CNAMEs target the same tunnel because the tunnel ingress rules
# (cloudflared-config.prod.yml + cloudflared-config.dev.yml) route each
# hostname to a different host port (4747 = staging, 4749 = dev, 4747
# again for prod since prod + staging share NodePort 30082 on this Mac's
# colima cluster).
#
# PREREQUISITES (one-time setup):
#   1. Cloudflare account owns epsx.io (or whichever zone you're targeting)
#   2. A Cloudflare API token with `Zone.DNS:Edit` permission scoped to
#      that zone. Create one at:
#        https://dash.cloudflare.com/profile/api-tokens
#        → "Create Token" → "Edit zone DNS" template
#   3. Know the Zone ID for epsx.io:
#        https://dash.cloudflare.com → click "epsx.io" → right sidebar
#        shows "Zone ID" (32-char hex)
#
# USAGE:
#   export CLOUDFLARE_API_TOKEN="<your-token>"
#   export CLOUDFLARE_ZONE_ID="<32-char-hex>"   # optional — script will
#                                               # auto-discover if unset
#   ./infrastructure/scripts/dns-create-pay.sh [hostname1 hostname2 ...]
#
# If no hostnames are passed, creates all three (pay, staging-pay, dev-pay).
#
# IDEMPOTENT: re-running the script with the same hostname is a no-op.
# Use --force to overwrite an existing record's content.
#
# SAFETY: this script only creates CNAMEs, never A/AAAA/MX records. It
# will not touch other records in the zone.
# ============================================================================

set -euo pipefail

CLOUDFLARE_API_BASE="https://api.cloudflare.com/client/v4"
TUNNEL_ID="6bee9b58-eede-4b4c-815c-94c0ee38fe58"
TUNNEL_TARGET="${TUNNEL_ID}.cfargotunnel.com"

# Default hostnames — all three pay.* subdomains
DEFAULT_HOSTNAMES=(
  "pay.epsx.io"
  "staging-pay.epsx.io"
  "dev-pay.epsx.io"
)

FORCE=false

# ---- arg parsing ----
while [[ $# -gt 0 ]]; do
  case "$1" in
    --force|-f)
      FORCE=true
      shift
      ;;
    --help|-h)
      grep '^# ' "$0" | sed 's/^# //'
      exit 0
      ;;
    -*)
      echo "Unknown flag: $1" >&2
      exit 1
      ;;
    *)
      HOSTNAMES+=("$1")
      shift
      ;;
  esac
done
HOSTNAMES=("${HOSTNAMES[@]:-${DEFAULT_HOSTNAMES[@]}}")

# ---- preflight ----
if [[ -z "${CLOUDFLARE_API_TOKEN:-}" ]]; then
  echo "ERROR: CLOUDFLARE_API_TOKEN env var is not set." >&2
  echo "Get a token: https://dash.cloudflare.com/profile/api-tokens" >&2
  echo "Required scope: Zone.DNS:Edit on the epsx.io zone." >&2
  exit 2
fi

# ---- discover zone ID if not provided ----
if [[ -z "${CLOUDFLARE_ZONE_ID:-}" ]]; then
  echo "CLOUDFLARE_ZONE_ID not set — discovering..."
  ZONE_RESPONSE=$(curl -s \
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
    -H "Content-Type: application/json" \
    "${CLOUDFLARE_API_BASE}/zones?name=epsx.io")
  CLOUDFLARE_ZONE_ID=$(echo "$ZONE_RESPONSE" | \
    python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["result"][0]["id"] if d.get("result") else "")' 2>/dev/null)
  if [[ -z "$CLOUDFLARE_ZONE_ID" ]]; then
    echo "ERROR: could not discover zone ID for epsx.io" >&2
    echo "Response: $ZONE_RESPONSE" >&2
    exit 3
  fi
  echo "Discovered zone ID: $CLOUDFLARE_ZONE_ID"
fi

# ---- create / verify each record ----
CREATE_COUNT=0
SKIP_COUNT=0
ERROR_COUNT=0

for HOSTNAME in "${HOSTNAMES[@]}"; do
  echo ""
  echo "── $HOSTNAME ──"

  # Check if a record already exists for this hostname
  LIST_RESPONSE=$(curl -s \
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
    -H "Content-Type: application/json" \
    "${CLOUDFLARE_API_BASE}/zones/${CLOUDFLARE_ZONE_ID}/dns_records?type=CNAME&name=${HOSTNAME}")
  EXISTING_ID=$(echo "$LIST_RESPONSE" | \
    python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["result"][0]["id"] if d.get("result") else "")' 2>/dev/null)
  EXISTING_CONTENT=$(echo "$LIST_RESPONSE" | \
    python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["result"][0]["content"] if d.get("result") else "")' 2>/dev/null)

  if [[ -n "$EXISTING_ID" ]]; then
    if [[ "$EXISTING_CONTENT" == "$TUNNEL_TARGET" && "$FORCE" == "false" ]]; then
      echo "  ✓ already points at tunnel (skipping)"
      SKIP_COUNT=$((SKIP_COUNT + 1))
      continue
    fi
    echo "  → updating existing record $EXISTING_ID (was: $EXISTING_CONTENT)"
    UPDATE_BODY=$(cat <<JSON
{
  "type": "CNAME",
  "name": "${HOSTNAME}",
  "content": "${TUNNEL_TARGET}",
  "proxied": true,
  "ttl": 1,
  "comment": "wave49(slice-6) pay.epsx.io microservice"
}
JSON
)
    RESPONSE=$(curl -s -X PUT \
      -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
      -H "Content-Type: application/json" \
      -d "$UPDATE_BODY" \
      "${CLOUDFLARE_API_BASE}/zones/${CLOUDFLARE_ZONE_ID}/dns_records/${EXISTING_ID}")
  else
    echo "  → creating new CNAME → $TUNNEL_TARGET"
    CREATE_BODY=$(cat <<JSON
{
  "type": "CNAME",
  "name": "${HOSTNAME}",
  "content": "${TUNNEL_TARGET}",
  "proxied": true,
  "ttl": 1,
  "comment": "wave49(slice-6) pay.epsx.io microservice"
}
JSON
)
    RESPONSE=$(curl -s -X POST \
      -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
      -H "Content-Type: application/json" \
      -d "$CREATE_BODY" \
      "${CLOUDFLARE_API_BASE}/zones/${CLOUDFLARE_ZONE_ID}/dns_records")
  fi

  SUCCESS=$(echo "$RESPONSE" | \
    python3 -c 'import json,sys; d=json.load(sys.stdin); print("true" if d.get("success") else "false")' 2>/dev/null)
  if [[ "$SUCCESS" == "true" ]]; then
    echo "  ✓ OK"
    CREATE_COUNT=$((CREATE_COUNT + 1))
  else
    echo "  ✗ FAILED"
    echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$RESPONSE"
    ERROR_COUNT=$((ERROR_COUNT + 1))
  fi
done

# ---- summary ----
echo ""
echo "════════════════════════════════════════"
echo "Created/Updated: $CREATE_COUNT"
echo "Skipped (already current): $SKIP_COUNT"
echo "Errors: $ERROR_COUNT"
echo "════════════════════════════════════════"

if [[ $ERROR_COUNT -gt 0 ]]; then
  exit 4
fi
