#!/bin/sh
set -eu

# Hermetic local runtime proof for the extracted notification service. This
# intentionally exercises only a disposable PostgreSQL database and a local
# service process; it never targets staging or production.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
database_name="epsx_notification_runtime_audit_$(date +%s)_$$"
database_url=""
service_port=$((8100 + ($$ % 100)))
service_pid=""
runtime_log=""

die() {
  echo "notification-runtime-local: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --allow-local)
      allow_local=true
      shift
      ;;
    --database-name)
      [ "$#" -ge 2 ] || die "--database-name requires an audit-only database name"
      database_name=$2
      shift 2
      ;;
    --port)
      [ "$#" -ge 2 ] || die "--port requires a local service port"
      service_port=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command starts a local process and mutates only a disposable local database"
case "$database_name" in
  epsx_notification_runtime_audit_*) ;;
  *) die "database name must start with epsx_notification_runtime_audit_" ;;
esac
case "$database_name" in
  *prod*|*production*|*staging*|*stage*) die "production/staging-looking database name refused" ;;
esac
case "$service_port" in
  *[!0-9]*|'') die "service port must be numeric" ;;
esac
[ "$service_port" -ge 1024 ] && [ "$service_port" -le 65535 ] || die "service port must be between 1024 and 65535"

command -v psql >/dev/null 2>&1 || die "psql is required"
command -v diesel >/dev/null 2>&1 || die "diesel is required"
command -v curl >/dev/null 2>&1 || die "curl is required"
command -v jq >/dev/null 2>&1 || die "jq is required"
command -v git >/dev/null 2>&1 || die "git is required"
command -v cargo >/dev/null 2>&1 || die "cargo is required for the redacted metrics audit"

database_owner=$(psql -h localhost -d postgres -Atqc 'select current_user' 2>/dev/null || true)
[ -n "$database_owner" ] || die "could not connect to the local PostgreSQL maintenance database"
case "$database_owner" in
  *[!A-Za-z0-9_]*|'') die "current PostgreSQL role is not safe for scratch ownership" ;;
esac

database_url="postgres://${database_owner}@localhost:5432/${database_name}"
runtime_log=$(mktemp "${TMPDIR:-/tmp}/epsx-notification-runtime.XXXXXX")

cleanup() {
  if [ -n "$service_pid" ] && kill -0 "$service_pid" 2>/dev/null; then
    kill "$service_pid" 2>/dev/null || true
    wait "$service_pid" 2>/dev/null || true
  fi
  psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$database_name\" WITH (FORCE)" >/dev/null 2>&1 || true
  rm -f -- "$runtime_log"
}
trap cleanup EXIT HUP INT TERM

existing=$(psql -h localhost -d postgres -Atqc "select 1 from pg_database where datname = '$database_name'" 2>/dev/null || true)
[ -z "$existing" ] || die "database already exists; choose a fresh --database-name"
psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$database_name\" OWNER \"$database_owner\""

DATABASE_URL="$database_url" diesel migration run \
  --migration-dir "$repo_root/apps/backend/migrations/notifications" >/dev/null

notification_binary=${NOTIFICATION_BINARY:-$repo_root/target/debug/notification}
[ -x "$notification_binary" ] || die "notification binary is missing; build it before this audit"

oidc_issuer=${NOTIFICATION_RUNTIME_OIDC_ISSUER:-http://127.0.0.1:8080}
jwks_url=${NOTIFICATION_RUNTIME_JWKS_URL:-$oidc_issuer/.well-known/jwks.json}
redis_url=${NOTIFICATION_RUNTIME_REDIS_URL:-redis://:epsx@127.0.0.1:6379/15}

DATABASE_URL="$database_url" \
OIDC_ISSUER="$oidc_issuer" \
OIDC_JWKS_URL="$jwks_url" \
EPSX_ENV=development \
REDIS_URL="$redis_url" \
PORT="$service_port" \
HOST=127.0.0.1 \
  "$notification_binary" >"$runtime_log" 2>&1 &
service_pid=$!

ready=false
attempt=1
while [ "$attempt" -le 40 ]; do
  if health_code=$(curl -sS -o /dev/null -w '%{http_code}' "http://127.0.0.1:$service_port/health" 2>/dev/null) \
    && [ "$health_code" = 200 ]; then
    ready=true
    break
  fi
  if ! kill -0 "$service_pid" 2>/dev/null; then
    cat "$runtime_log" >&2
    die "notification service exited before health became ready"
  fi
  sleep 0.25
  attempt=$((attempt + 1))
done
[ "$ready" = true ] || die "notification service did not become healthy"

ready_payload=$(curl -fsS "http://127.0.0.1:$service_port/ready")
if ! printf '%s' "$ready_payload" | jq -e '
  .status == "ready" and .database == true and .lifecycle == true and .redis_reachable == true
' >/dev/null; then
  die "readiness did not report compatible database/lifecycle state: $ready_payload"
fi

psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "
  INSERT INTO public.notifications (id, user_id, channel, recipient, body, status)
  VALUES
    ('runtime-expiry-notification', '0x1111111111111111111111111111111111111111', 'in_app', '0x1111111111111111111111111111111111111111', 'runtime expiry', 'pending'),
    ('runtime-inapp-notification', '0x1111111111111111111111111111111111111111', 'in_app', '0x1111111111111111111111111111111111111111', 'runtime inapp', 'pending');
  INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload)
  VALUES
    ('runtime-expiry-event', 'expiry.runtime', 'runtime-expiry', '{}'::jsonb),
    ('runtime-inapp-event', 'notification.runtime', 'runtime-inapp', '{}'::jsonb);
  INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key)
  VALUES
    ('runtime-expiry-job', 'runtime-expiry-event', 'runtime-expiry-notification', 'in_app', '0x1111111111111111111111111111111111111111', 'runtime-expiry-key'),
    ('runtime-inapp-job', 'runtime-inapp-event', 'runtime-inapp-notification', 'in_app', '0x1111111111111111111111111111111111111111', 'runtime-inapp-key');
  INSERT INTO public.notification_expirations (notification_id, expires_at)
  VALUES ('runtime-expiry-notification', NOW() - INTERVAL '1 minute');
"

runtime_result=""
attempt=1
while [ "$attempt" -le 40 ]; do
  runtime_result=$(psql "$database_url" -Atqc "
    SELECT n.id || ':' || n.status || ':' || j.state || ':' || COALESCE(a.outcome, 'none')
    FROM public.notifications n
    JOIN public.notification_channel_jobs j ON j.notification_id = n.id
    LEFT JOIN public.notification_delivery_attempts a ON a.job_id = j.id
    WHERE n.id IN ('runtime-expiry-notification', 'runtime-inapp-notification')
    ORDER BY n.id
  ")
  case "$runtime_result" in
    *'runtime-expiry-notification:expired:terminal_failed:none'*'runtime-inapp-notification:sent:provider_accepted:accepted'*)
      break
      ;;
  esac
  sleep 0.25
  attempt=$((attempt + 1))
done
case "$runtime_result" in
  *'runtime-expiry-notification:expired:terminal_failed:none'*'runtime-inapp-notification:sent:provider_accepted:accepted'*) ;;
  *) die "worker runtime transition did not complete: $runtime_result" ;;
esac

unauthorized_code=$(curl -sS -o /dev/null -w '%{http_code}' \
  -X POST "http://127.0.0.1:$service_port/api/v1/notification/publish" \
  -H 'content-type: application/json' -d '{}')
[ "$unauthorized_code" = 401 ] || die "publisher boundary returned $unauthorized_code without credentials"

# Stop the worker before the direct metrics projection audit so its queued
# fixture cannot be claimed while the snapshot assertions run.
if [ -n "$service_pid" ] && kill -0 "$service_pid" 2>/dev/null; then
  kill "$service_pid" 2>/dev/null || true
  wait "$service_pid" 2>/dev/null || true
  service_pid=""
fi
NOTIFICATION_RUNTIME_DATABASE_URL="$database_url" \
  cargo test -p epsx-notification --bin notification --locked \
    tests::runtime_owner_list_filters_match_source_semantics \
    -- --ignored --exact --nocapture

NOTIFICATION_RUNTIME_DATABASE_URL="$database_url" \
  cargo test -p epsx-notification --bin notification --locked \
    tests::runtime_metrics_snapshot_is_redacted_and_bounded \
    -- --ignored --exact --nocapture

echo "notification-runtime-local: PASS — readiness=database+lifecycle+redis_ping, expiry=terminal, in_app=provider_accepted, unauthenticated publisher=401, owner-list-filters=source-compatible, redacted metrics=bounded"
echo "notification-runtime-local: LIMIT — disposable local PostgreSQL/Redis and a local service only; no browser, external provider, staging, or production evidence"
