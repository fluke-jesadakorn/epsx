#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MIGRATIONS_DIR="$(dirname "$SCRIPT_DIR")/migrations"

PG_USER="${PG_USER:-epsx}"
PG_HOST="${PG_HOST:-localhost}"
PG_PORT="${PG_PORT:-5432}"
PG_PASSWORD="${PG_PASSWORD:-epsx}"

# Databases to migrate
DATABASES=(
  "epsx_identity"
  "epsx_wallet"
  "epsx_payment"
  "epsx_subscription"
  "epsx_content"
  "epsx_notification"
  "epsx_analytics"
  "epsx_indexer"
)

if [ -z "${1:-}" ]; then
  echo "Usage: $0 <action> [database]"
  echo "Actions:"
  echo "  apply    Apply all pending migrations to all databases (or specified one)"
  echo "  reset    Drop and recreate databases, then apply migrations"
  echo "  status   Show migration status"
  exit 1
fi

ACTION="$1"
TARGET_DB="${2:-}"

export PGPASSWORD="$PG_PASSWORD"

# Ensure databases exist
create_db_if_missing() {
  local db="$1"
  if psql -U "$PG_USER" -h "$PG_HOST" -p "$PG_PORT" -lqt | cut -d \| -f 1 | grep -qw "$db"; then
    echo "  [exists] $db"
  else
    echo "  [creating] $db"
    createdb -U "$PG_USER" -h "$PG_HOST" -p "$PG_PORT" "$db" 2>/dev/null || \
      psql -U "$PG_USER" -h "$PG_HOST" -p "$PG_PORT" -d postgres -c "CREATE DATABASE $db"
  fi
}

apply_migration() {
  local db="$1"
  local migration_file="$2"
  echo "  [apply] $db ← $(basename "$migration_file")"
  psql -U "$PG_USER" -h "$PG_HOST" -p "$PG_PORT" -d "$db" -v ON_ERROR_STOP=1 -f "$migration_file" >/dev/null
}

case "$ACTION" in
  apply)
    if [ -n "$TARGET_DB" ]; then
      DATABASES=("$TARGET_DB")
    fi
    for db in "${DATABASES[@]}"; do
      create_db_if_missing "$db"
      for f in "$MIGRATIONS_DIR"/*.sql; do
        apply_migration "$db" "$f"
      done
    done
    echo "Migrations applied."
    ;;
  reset)
    for db in "${DATABASES[@]}"; do
      echo "  [drop] $db"
      dropdb -U "$PG_USER" -h "$PG_HOST" -p "$PG_PORT" --if-exists "$db"
      create_db_if_missing "$db"
      for f in "$MIGRATIONS_DIR"/*.sql; do
        apply_migration "$db" "$f"
      done
    done
    echo "Databases reset and migrated."
    ;;
  status)
    for db in "${DATABASES[@]}"; do
      echo "=== $db ==="
      psql -U "$PG_USER" -h "$PG_HOST" -p "$PG_PORT" -d "$db" -c "\dt" 2>/dev/null || echo "  (not found)"
    done
    ;;
  *)
    echo "Unknown action: $ACTION"
    exit 1
    ;;
esac
