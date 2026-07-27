#!/bin/sh
set -eu

# Hermetic local proof for the Rust template version/rollback handlers. The
# ignored integration test uses a disposable migrated database only.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
database_name="epsx_notification_runtime_audit_rollback_$(date +%s)_$$"

die() {
  echo "notification-template-rollback-local: ERROR: $*" >&2
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
  epsx_notification_runtime_audit_rollback_*) ;;
  *) die "database name must start with epsx_notification_runtime_audit_rollback_" ;;
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
cleanup() {
  psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$database_name\" WITH (FORCE)" >/dev/null 2>&1 || true
}
trap cleanup EXIT HUP INT TERM

existing=$(psql -h localhost -d postgres -Atqc "select 1 from pg_database where datname = '$database_name'" 2>/dev/null || true)
[ -z "$existing" ] || die "database already exists; choose a fresh --database-name"
psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$database_name\" OWNER \"$database_owner\""
DATABASE_URL="$database_url" diesel migration run \
  --migration-dir "$repo_root/apps/backend/migrations/notifications" >/dev/null

NOTIFICATION_RUNTIME_DATABASE_URL="$database_url" \
  cargo test -p epsx-notification --bin notification --locked tests::runtime_template_rollback_replays_body_and_audit \
    -- --ignored --exact --nocapture

echo "notification-template-rollback-local: PASS — version 1 was restored through the Rust handler and rollback audit metadata was read back"
echo "notification-template-rollback-local: LIMIT — disposable local PostgreSQL and Rust handlers only; no browser, provider, staging, or production evidence"
