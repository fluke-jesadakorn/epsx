#!/bin/sh
set -eu

# Local wire-contract smoke audit for the extracted notification service and
# Rust/Dioxus BFFs. Callers provide short-lived bearer tokens; this script never
# prints or persists them. It performs read-only reads plus one guaranteed
# foreign-ID mutation probe, which must fail with 404 and cannot change state.

allow_local=false
service_url=${NOTIFICATION_COMPATIBILITY_SERVICE_URL:-http://127.0.0.1:8106}
frontend_url=${NOTIFICATION_COMPATIBILITY_FRONTEND_URL:-http://localhost:3000}
admin_url=${NOTIFICATION_COMPATIBILITY_ADMIN_URL:-http://localhost:3001}
owner_token=${NOTIFICATION_COMPATIBILITY_OWNER_TOKEN:-}
admin_token=${NOTIFICATION_COMPATIBILITY_ADMIN_TOKEN:-}
tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-compatibility.XXXXXX")

die() {
  echo "notification-compatibility-local: ERROR: $*" >&2
  exit 1
}

cleanup() {
  rm -rf -- "$tmp_dir"
}
trap cleanup EXIT HUP INT TERM

while [ "$#" -gt 0 ]; do
  case "$1" in
    --allow-local)
      allow_local=true
      shift
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this audit is restricted to loopback HTTP origins"
[ -n "$owner_token" ] || die "NOTIFICATION_COMPATIBILITY_OWNER_TOKEN is required"
[ -n "$admin_token" ] || die "NOTIFICATION_COMPATIBILITY_ADMIN_TOKEN is required"

for url in "$service_url" "$frontend_url" "$admin_url"; do
  case "$url" in
    http://localhost:*|http://127.0.0.1:*) ;;
    *) die "service, frontend, and admin URLs must be explicit local HTTP origins" ;;
  esac
done

command -v curl >/dev/null 2>&1 || die "curl is required"
command -v bun >/dev/null 2>&1 || die "bun is required"

request() {
  method=$1
  url=$2
  token=$3
  body_file=$4
  request_body=${5-}
  if [ -n "$request_body" ]; then
    curl -sS -o "$body_file" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $token" \
      -H 'Content-Type: application/json' \
      --data "$request_body" "$url"
  else
    curl -sS -o "$body_file" -w '%{http_code}' -X "$method" \
      -H "Authorization: Bearer $token" "$url"
  fi
}

check_shape() {
  label=$1
  body_file=$2
  shape=$3
  bun -e '
const value = await Bun.file(process.argv[1]).json();
const shape = process.argv[2];
const keys = Object.keys(value).sort();
const exact = (expected) => keys.join(",") === expected.split(",").sort().join(",");
if (shape === "list" && (!exact("items,total") || !Array.isArray(value.items) || !Number.isSafeInteger(value.total) || value.total < 0)) process.exit(1);
if (shape === "count" && (!exact("count") || !Number.isSafeInteger(value.count) || value.count < 0)) process.exit(1);
if (shape === "preferences" && (!exact("channels,quiet_hours,timezone,updated_at") || typeof value.channels !== "object" || value.channels === null)) process.exit(1);
if (shape === "admin-list" && (!exact("items,limit,offset,total") || !Array.isArray(value.items) || !Number.isSafeInteger(value.limit) || !Number.isSafeInteger(value.offset) || !Number.isSafeInteger(value.total) || value.limit < 1 || value.offset < 0 || value.total < 0)) process.exit(1);
if (shape === "admin-item" && (!exact("channel,created_at,id,notification_type,priority,sent_at,status,subject,title") || typeof value.id !== "string" || typeof value.channel !== "string" || typeof value.status !== "string" || typeof value.created_at !== "string")) process.exit(1);
' "$body_file" "$shape" || die "$label returned a malformed $shape envelope"
}

check_status_and_shape() {
  label=$1
  method=$2
  url=$3
  token=$4
  shape=$5
  body_file="$tmp_dir/${label}.json"
  status=$(request "$method" "$url" "$token" "$body_file" "${6-}")
  [ "$status" = 200 ] || die "$label returned HTTP $status"
  check_shape "$label" "$body_file" "$shape"
}

check_status_and_shape service-owner-list GET "$service_url/api/v1/notification/list?limit=1&offset=0" "$owner_token" list
check_status_and_shape service-owner-count GET "$service_url/api/v1/notification/unread-count" "$owner_token" count
check_status_and_shape service-owner-preferences GET "$service_url/api/v1/notification/preferences" "$owner_token" preferences

foreign_id="compatibility-local-foreign-$(date +%s)-$$"
foreign_body="$tmp_dir/foreign.json"
foreign_status=$(request POST "$service_url/api/v1/notification/$foreign_id/read" "$owner_token" "$foreign_body" '{}')
[ "$foreign_status" = 404 ] || die "foreign owner mutation returned HTTP $foreign_status instead of 404"

check_status_and_shape frontend-owner-list GET "$frontend_url/api/v1/notifications?limit=1&offset=0" "$owner_token" list
check_status_and_shape frontend-owner-count GET "$frontend_url/api/v1/notifications/unread-count" "$owner_token" count
check_status_and_shape frontend-owner-preferences GET "$frontend_url/api/v1/notifications/preferences" "$owner_token" preferences

admin_body="$tmp_dir/admin-list.json"
admin_status=$(request GET "$service_url/api/v1/notification/admin/list?limit=20&offset=0" "$admin_token" "$admin_body")
[ "$admin_status" = 200 ] || die "service-admin-list returned HTTP $admin_status"
check_shape service-admin-list "$admin_body" admin-list
bun -e 'const value=await Bun.file(process.argv[1]).json(); if(value.items.some(item => { const keys=Object.keys(item).sort().join(","); return keys !== "channel,created_at,id,notification_type,priority,sent_at,status,subject,title".split(",").sort().join(","); })) process.exit(1)' "$admin_body" || die "service-admin-list returned an unsafe item projection"

echo "notification-compatibility-local: PASS — owner list/count/preferences envelopes matched through service and Dioxus BFFs, foreign owner mutation failed closed with 404, and admin inventory projection was validated"
echo "notification-compatibility-local: LIMIT — disposable local bearer/database/service only; no pinned development payload parity, provider, staging, multi-instance, or production evidence"
