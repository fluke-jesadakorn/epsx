#!/bin/sh
set -eu

# Hermetic local proof that an unsafe active template fails closed before the
# notification service serves traffic. This mutates only a disposable local
# PostgreSQL database and never targets staging or production.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
database_name="epsx_notification_runtime_audit_template_$(date +%s)_$$"
service_port=$((8200 + ($$ % 100)))
service_pid=""
runtime_log=""

die() {
  echo "notification-template-startup-local: ERROR: $*" >&2
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
  epsx_notification_runtime_audit_template_*) ;;
  *) die "database name must start with epsx_notification_runtime_audit_template_" ;;
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
command -v git >/dev/null 2>&1 || die "git is required"

database_owner=$(psql -h localhost -d postgres -Atqc 'select current_user' 2>/dev/null || true)
[ -n "$database_owner" ] || die "could not connect to the local PostgreSQL maintenance database"
case "$database_owner" in
  *[!A-Za-z0-9_]*|'') die "current PostgreSQL role is not safe for scratch ownership" ;;
esac

database_url="postgres://${database_owner}@localhost:5432/${database_name}"
runtime_log=$(mktemp "${TMPDIR:-/tmp}/epsx-notification-template-startup.XXXXXX")

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

# A triple-brace Handlebars expression is raw output and must be rejected by
# the same parser-backed validation used by template writes and startup load.
psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "
  INSERT INTO public.templates (id, name, channel, body, variables, active)
  VALUES ('runtime-unsafe-template', 'runtime-unsafe-template', 'in_app', '{{{name}}}', '{}'::jsonb, TRUE)
"

OIDC_ISSUER=${NOTIFICATION_RUNTIME_OIDC_ISSUER:-http://127.0.0.1:8080} \
OIDC_JWKS_URL=${NOTIFICATION_RUNTIME_JWKS_URL:-http://127.0.0.1:8080/.well-known/jwks.json} \
EPSX_ENV=development \
DATABASE_URL="$database_url" \
REDIS_URL="${NOTIFICATION_RUNTIME_REDIS_URL:-redis://:epsx@127.0.0.1:6379/15}" \
PORT="$service_port" \
HOST=127.0.0.1 \
  "$notification_binary" >"$runtime_log" 2>&1 &
service_pid=$!

service_rc=0
attempt=1
while [ "$attempt" -le 40 ]; do
  if ! kill -0 "$service_pid" 2>/dev/null; then
    set +e
    wait "$service_pid"
    service_rc=$?
    set -e
    break
  fi
  sleep 0.25
  attempt=$((attempt + 1))
done

if kill -0 "$service_pid" 2>/dev/null; then
  kill "$service_pid" 2>/dev/null || true
  set +e
  wait "$service_pid"
  service_rc=$?
  set -e
  die "service remained alive after unsafe active template was inserted"
fi
[ "$service_rc" -ne 0 ] || die "service exited successfully despite unsafe active template"
grep -q "active notification templates must load before startup" "$runtime_log" \
  || die "startup failure did not identify active template loading: $(sed -n '1,80p' "$runtime_log")"

echo "notification-template-startup-local: PASS — unsafe active template rejected before service readiness (exit=$service_rc)"
echo "notification-template-startup-local: LIMIT — disposable local PostgreSQL and local process only; no browser, provider, staging, or production evidence"
