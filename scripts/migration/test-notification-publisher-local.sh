#!/bin/sh
set -eu

# Hermetic local proof for publisher admission and replay semantics. The
# ignored Rust integration test invokes the real handler against a disposable
# migrated database; it never targets staging or production.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
database_name="epsx_notification_runtime_audit_publisher_$(date +%s)_$$"
plan_database_name="${database_name}_core"

die() {
  echo "notification-publisher-local: ERROR: $*" >&2
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
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command mutates only a disposable local database"
case "$database_name" in
  epsx_notification_runtime_audit_publisher_*) ;;
  *) die "database name must start with epsx_notification_runtime_audit_publisher_" ;;
esac
case "$plan_database_name" in
  *prod*|*production*|*staging*|*stage*) die "production/staging-looking core database name refused" ;;
esac
case "$database_name" in
  *prod*|*production*|*staging*|*stage*) die "production/staging-looking database name refused" ;;
esac

command -v psql >/dev/null 2>&1 || die "psql is required"
command -v diesel >/dev/null 2>&1 || die "diesel is required"
command -v git >/dev/null 2>&1 || die "git is required"

database_owner=$(psql -h localhost -d postgres -Atqc 'select current_user' 2>/dev/null || true)
[ -n "$database_owner" ] || die "could not connect to the local PostgreSQL maintenance database"
case "$database_owner" in
  *[!A-Za-z0-9_]*|'') die "current PostgreSQL role is not safe for scratch ownership" ;;
esac

database_url="postgres://${database_owner}@localhost:5432/${database_name}"
plan_database_url="postgres://${database_owner}@localhost:5432/${plan_database_name}"
cleanup() {
  psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$database_name\" WITH (FORCE)" >/dev/null 2>&1 || true
  psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$plan_database_name\" WITH (FORCE)" >/dev/null 2>&1 || true
}
trap cleanup EXIT HUP INT TERM

existing=$(psql -h localhost -d postgres -Atqc "select 1 from pg_database where datname = '$database_name'" 2>/dev/null || true)
[ -z "$existing" ] || die "database already exists; choose a fresh --database-name"
plan_existing=$(psql -h localhost -d postgres -Atqc "select 1 from pg_database where datname = '$plan_database_name'" 2>/dev/null || true)
[ -z "$plan_existing" ] || die "core database already exists; choose a fresh --database-name"
psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$database_name\" OWNER \"$database_owner\""
psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$plan_database_name\" OWNER \"$database_owner\""
DATABASE_URL="$database_url" diesel migration run \
  --migration-dir "$repo_root/apps/backend/migrations/notifications" >/dev/null
psql "$plan_database_url" -v ON_ERROR_STOP=1 -qAtc "CREATE TABLE public.wallet_plan_assignments (wallet_address varchar(42) NOT NULL, plan_id uuid NOT NULL, is_active boolean NOT NULL, expires_at timestamptz)"

NOTIFICATION_RUNTIME_DATABASE_URL="$database_url" \
NOTIFICATION_RUNTIME_PLAN_DATABASE_URL="$plan_database_url" \
  cargo test -p epsx-notification --bin notification --locked tests::runtime_publisher_replay_is_idempotent_and_broadcast_is_single_row \
    -- --ignored --exact --nocapture

echo "notification-publisher-local: PASS — concrete replay dedupe, payload-conflict rejection, transactional producer rollback, read-only plan fanout, and single durable broadcast row were verified"
echo "notification-publisher-local: LIMIT — disposable local PostgreSQL/core membership database and Rust admission handler only; no process-crash, external service identity, staging, or production evidence"
