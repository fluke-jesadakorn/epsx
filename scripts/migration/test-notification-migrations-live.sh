#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)

database_url="${NOTIFICATION_MIGRATION_AUDIT_DATABASE_URL-}"
allow_local=false
report_path=""

die() {
  echo "notification-migration-live: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --database-url)
      [ "$#" -ge 2 ] || die "--database-url requires a local PostgreSQL URL"
      database_url=$2
      shift 2
      ;;
    --allow-local)
      allow_local=true
      shift
      ;;
    --report)
      [ "$#" -ge 2 ] || die "--report requires a writable local path"
      report_path=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command mutates only an explicitly-created local scratch database"
[ -n "$database_url" ] || die "--database-url or NOTIFICATION_MIGRATION_AUDIT_DATABASE_URL is required"
command -v psql >/dev/null 2>&1 || die "psql is required"
command -v diesel >/dev/null 2>&1 || die "diesel is required"
command -v pg_dump >/dev/null 2>&1 || die "pg_dump is required"
command -v pg_restore >/dev/null 2>&1 || die "pg_restore is required"
command -v git >/dev/null 2>&1 || die "git is required"

case "$database_url" in
  postgres://*|postgresql://*) ;;
  *) die "database URL must use postgres:// or postgresql://" ;;
esac

url_without_query=${database_url%%\?*}
database_name=${url_without_query##*/}
database_base=${url_without_query%/*}
query_suffix=""
case "$database_url" in
  *\?*) query_suffix="?${database_url#*\?}" ;;
esac

case "$database_name" in
  epsx_notification_audit_*) ;;
  *) die "database name must start with epsx_notification_audit_" ;;
esac
case "$database_name" in
  *prod*|*production*|*staging*|*stage*) die "production/staging-looking database name refused" ;;
esac
case "$database_base" in
  postgres://*localhost:*|postgres://*127.0.0.1:*|postgres://*\[::1\]:*|postgresql://*localhost:*|postgresql://*127.0.0.1:*|postgresql://*\[::1\]:*) ;;
  *) die "database host must be localhost, 127.0.0.1, or ::1" ;;
esac

maintenance_url="${database_base}/postgres${query_suffix}"
recovery_name="${database_name}_recovery"
recovery_url="${database_base}/${recovery_name}${query_suffix}"
case "$recovery_name" in
  *prod*|*production*|*staging*|*stage*) die "derived recovery database name is unsafe" ;;
esac

database_owner=$(psql "$maintenance_url" -Atqc 'select current_user' 2>/dev/null || true)
[ -n "$database_owner" ] || die "could not connect to the local PostgreSQL maintenance database"
case "$database_owner" in
  *[!A-Za-z0-9_]*|'') die "current PostgreSQL role is not safe for scratch ownership" ;;
esac

database_exists=$(psql "$maintenance_url" -Atqc "select 1 from pg_database where datname = '$database_name'" 2>/dev/null || true)
[ -z "$database_exists" ] || die "database already exists; choose a fresh scratch database name"
recovery_exists=$(psql "$maintenance_url" -Atqc "select 1 from pg_database where datname = '$recovery_name'" 2>/dev/null || true)
[ -z "$recovery_exists" ] || die "derived recovery database already exists; remove it or choose a fresh scratch name"

temp_root=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-migration-live.XXXXXX")
migration_root="$temp_root/migrations"
dump_path="$temp_root/notification.dump"
trap 'rm -rf -- "$temp_root"; if [ "${recovery_created:-0}" = 1 ]; then psql "$maintenance_url" -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$recovery_name\" WITH (FORCE)" >/dev/null 2>&1 || true; fi; if [ "${database_created:-0}" = 1 ]; then psql "$maintenance_url" -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$database_name\" WITH (FORCE)" >/dev/null 2>&1 || true; fi' EXIT HUP INT TERM

mkdir -p "$migration_root"
cp "$repo_root/apps/backend/migrations/notifications/.diesel_lock" "$migration_root/.diesel_lock"

copy_migration() {
  migration=$1
  mkdir -p "$migration_root/$migration"
  cp "$repo_root/apps/backend/migrations/notifications/$migration/up.sql" "$migration_root/$migration/up.sql"
  cp "$repo_root/apps/backend/migrations/notifications/$migration/down.sql" "$migration_root/$migration/down.sql"
}

for migration in \
  00000000000000_diesel_initial_setup \
  00000000000001_consolidated_baseline_v2 \
  20260613000000_drop_notification_subscriptions; do
  copy_migration "$migration"
done

psql "$maintenance_url" -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$database_name\" OWNER \"$database_owner\""
database_created=1

run_migrations() {
  DATABASE_URL="$database_url" diesel migration run --migration-dir "$migration_root" >/dev/null
}

run_migrations

legacy_schema_checksum=$(psql "$database_url" -Atqc "select md5(coalesce(string_agg(format('%s.%s.%s.%s.%s.%s', table_schema, table_name, column_name, data_type, is_nullable, coalesce(column_default, '')), E'\\n' order by table_schema, table_name, ordinal_position), '')) from information_schema.columns where table_schema = 'public' and table_name = 'wallet_notifications'")
legacy_table_count=$(psql "$database_url" -Atqc "select count(*) from information_schema.tables where table_schema = 'public' and table_name = 'wallet_notifications'")
[ "$legacy_table_count" = 1 ] || die "legacy baseline did not produce wallet_notifications"

psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "insert into public.wallet_notifications (recipient_wallet_address, title, body, notification_type) values ('0x1111111111111111111111111111111111111111', 'Migration audit row', 'Non-production upgrade proof', 'system')"
legacy_rows_before=$(psql "$database_url" -Atqc 'select count(*) from public.wallet_notifications')
[ "$legacy_rows_before" = 1 ] || die "populated legacy fixture was not inserted"

for migration in \
  20260722040000_create_notification_service_tables \
  20260723120000_add_notification_lifecycle_foundation \
  20260723130000_add_notification_idempotency_provider_events \
  20260723140000_add_notification_lifecycle_constraints \
  20260724120000_add_notification_template_audit \
  20260724130000_add_notification_engagement_acknowledged \
  20260724140000_add_notification_expirations \
  20260724150000_add_notification_vapid_key_lineage; do
  copy_migration "$migration"
done

run_migrations

target_schema_checksum=$(psql "$database_url" -Atqc "select md5(coalesce(string_agg(format('%s.%s.%s.%s.%s.%s', table_schema, table_name, column_name, data_type, is_nullable, coalesce(column_default, '')), E'\\n' order by table_schema, table_name, ordinal_position), '')) from information_schema.columns where table_schema = 'public' and table_name in ('wallet_notifications', 'templates', 'notifications', 'notification_template_versions', 'notification_preferences', 'notification_inbox', 'notification_outbox', 'notification_channel_jobs', 'notification_delivery_attempts', 'notification_dead_letters', 'notification_replay_cursors', 'notification_push_subscriptions', 'notification_request_idempotency', 'notification_provider_events', 'notification_engagement', 'notification_template_audit', 'notification_expirations')")
legacy_rows_after=$(psql "$database_url" -Atqc 'select count(*) from public.wallet_notifications')
migration_rows=$(psql "$database_url" -Atqc 'select count(*) from public.__diesel_schema_migrations')
target_tables=$(psql "$database_url" -Atqc "select count(*) from information_schema.tables where table_schema = 'public' and table_name in ('templates', 'notifications', 'notification_template_versions', 'notification_preferences', 'notification_inbox', 'notification_outbox', 'notification_channel_jobs', 'notification_delivery_attempts', 'notification_dead_letters', 'notification_replay_cursors', 'notification_push_subscriptions', 'notification_request_idempotency', 'notification_provider_events', 'notification_engagement', 'notification_template_audit', 'notification_expirations')")
[ "$legacy_rows_after" = "$legacy_rows_before" ] || die "legacy row count changed during additive upgrade"
[ "$migration_rows" = 11 ] || die "expected eleven migration ledger rows, found $migration_rows"
[ "$target_tables" = 16 ] || die "expected sixteen target tables, found $target_tables"

psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "insert into public.notifications (id, user_id, channel, recipient, body, status) values ('expiry-audit-notification', '0x1111111111111111111111111111111111111111', 'in_app', '0x1111111111111111111111111111111111111111', 'Expiry audit row', 'pending'); insert into public.notification_outbox (event_id, event_type, aggregate_id, payload) values ('expiry-audit-event', 'expiry.audit', 'expiry-audit', '{}'::jsonb); insert into public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key) values ('expiry-audit-job', 'expiry-audit-event', 'expiry-audit-notification', 'in_app', '0x1111111111111111111111111111111111111111', 'expiry-audit-job-key'); insert into public.notification_expirations (notification_id, expires_at) values ('expiry-audit-notification', NOW() - INTERVAL '1 minute')"
expired_owner_rows=$(psql "$database_url" -Atqc "select count(*) from public.notifications n left join public.notification_expirations x on x.notification_id = n.id where n.user_id = '0x1111111111111111111111111111111111111111' and (x.expires_at is null or x.expires_at > now())")
expired_claimable_jobs=$(psql "$database_url" -Atqc "select count(*) from public.notification_channel_jobs j left join public.notification_expirations x on x.notification_id = j.notification_id where j.state in ('queued', 'retry_wait', 'leased') and j.available_at <= now() and (j.lease_until is null or j.lease_until <= now()) and (x.expires_at is null or x.expires_at > now())")
[ "$expired_owner_rows" = 0 ] || die "expired owner notification remained visible"
[ "$expired_claimable_jobs" = 0 ] || die "expired channel job remained claimable"
psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "EXPLAIN SELECT j.id FROM public.notification_channel_jobs j LEFT JOIN public.notification_expirations x ON x.notification_id = j.notification_id WHERE j.state IN ('queued', 'retry_wait', 'leased') AND j.available_at <= NOW() AND (j.lease_until IS NULL OR j.lease_until <= NOW()) AND (x.expires_at IS NULL OR x.expires_at > NOW()) ORDER BY j.available_at ASC, j.created_at ASC, j.id ASC FOR UPDATE OF j SKIP LOCKED LIMIT 1" >/dev/null
psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "EXPLAIN UPDATE public.notification_channel_jobs j SET state = 'terminal_failed', lease_until = NULL, updated_at = NOW() FROM public.notification_expirations x WHERE j.id = 'expiry-audit-job' AND j.notification_id = x.notification_id AND x.expires_at <= NOW() AND j.state IN ('leased', 'attempting') RETURNING j.notification_id" >/dev/null
psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "EXPLAIN WITH due AS (SELECT j.id FROM public.notification_channel_jobs j JOIN public.notification_expirations x ON x.notification_id = j.notification_id JOIN public.notifications n ON n.id = j.notification_id WHERE x.expires_at <= NOW() AND j.state IN ('queued', 'retry_wait', 'leased', 'attempting') AND n.status IN ('pending', 'suppressed') ORDER BY x.expires_at ASC, j.id ASC FOR UPDATE OF j SKIP LOCKED LIMIT 100), expired_jobs AS (UPDATE public.notification_channel_jobs j SET state = 'terminal_failed', lease_until = NULL, updated_at = NOW() FROM due WHERE j.id = due.id RETURNING j.notification_id) UPDATE public.notifications n SET status = 'expired', error = 'notification_expired' FROM (SELECT DISTINCT notification_id FROM expired_jobs) expired WHERE n.id = expired.notification_id AND n.status IN ('pending', 'suppressed')" >/dev/null
psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "EXPLAIN WITH due AS (SELECT n.id FROM public.notifications n JOIN public.notification_expirations x ON x.notification_id = n.id WHERE x.expires_at <= NOW() AND n.status IN ('pending', 'suppressed') ORDER BY x.expires_at ASC, n.id ASC FOR UPDATE OF n SKIP LOCKED LIMIT 100) UPDATE public.notifications n SET status = 'expired', error = 'notification_expired' FROM due WHERE n.id = due.id AND n.status IN ('pending', 'suppressed')" >/dev/null
expired_swept=$(psql "$database_url" -Atqc "WITH due AS (SELECT j.id FROM public.notification_channel_jobs j JOIN public.notification_expirations x ON x.notification_id = j.notification_id JOIN public.notifications n ON n.id = j.notification_id WHERE x.expires_at <= NOW() AND j.state IN ('queued', 'retry_wait', 'leased', 'attempting') AND n.status IN ('pending', 'suppressed') ORDER BY x.expires_at ASC, j.id ASC FOR UPDATE OF j SKIP LOCKED LIMIT 100), expired_jobs AS (UPDATE public.notification_channel_jobs j SET state = 'terminal_failed', lease_until = NULL, updated_at = NOW() FROM due WHERE j.id = due.id RETURNING j.notification_id) UPDATE public.notifications n SET status = 'expired', error = 'notification_expired' FROM (SELECT DISTINCT notification_id FROM expired_jobs) expired WHERE n.id = expired.notification_id AND n.status IN ('pending', 'suppressed') RETURNING n.id")
[ "$expired_swept" = "expiry-audit-notification" ] || die "expired projection was not swept"
swept_job_state=$(psql "$database_url" -Atqc "select state from public.notification_channel_jobs where id = 'expiry-audit-job'")
swept_notification_status=$(psql "$database_url" -Atqc "select status from public.notifications where id = 'expiry-audit-notification'")
[ "$swept_job_state" = "terminal_failed" ] || die "expired job did not become terminal_failed"
[ "$swept_notification_status" = "expired" ] || die "expired notification did not become expired"

expect_constraint_failure() {
  sql=$1
  if psql "$database_url" -v ON_ERROR_STOP=1 -qAtc "$sql" >/dev/null 2>&1; then
    die "constraint accepted invalid fixture"
  fi
}

expect_constraint_failure "insert into public.notification_preferences (user_id, channels) values ('not-a-wallet', '{}'::jsonb)"
expect_constraint_failure "insert into public.notification_engagement (notification_id, owner_id, read_at) values ('missing-notification', 'not-a-wallet', NOW())"
expect_constraint_failure "insert into public.notification_outbox (event_id, event_type, aggregate_id, payload, state) values ('audit-invalid-state', 'audit', 'aggregate', '{}'::jsonb, 'invalid')"
expect_constraint_failure "insert into public.notification_request_idempotency (principal_subject, event_type, idempotency_key, request_hash, response_status, response_body) values ('audit', 'audit', 'invalid-hash', 'not-a-sha', 202, '{}'::jsonb)"
expect_constraint_failure "insert into public.notification_expirations (notification_id, expires_at) values ('   ', NOW() + INTERVAL '1 hour')"
expect_constraint_failure "insert into public.notification_push_subscriptions (endpoint, user_id, p256dh, auth, vapid_key_id) values ('https://push.example.test/audit', '0x1111111111111111111111111111111111111111', 'p256dh', 'auth', 'bad key')"

pg_dump --format=custom --file="$dump_path" "$database_url" >/dev/null
psql "$maintenance_url" -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$recovery_name\" OWNER \"$database_owner\""
recovery_created=1
pg_restore --exit-on-error --dbname="$recovery_url" "$dump_path" >/dev/null

recovery_rows=$(psql "$recovery_url" -Atqc 'select count(*) from public.wallet_notifications')
recovery_checksum=$(psql "$recovery_url" -Atqc "select md5(coalesce(string_agg(format('%s.%s.%s.%s.%s.%s', table_schema, table_name, column_name, data_type, is_nullable, coalesce(column_default, '')), E'\\n' order by table_schema, table_name, ordinal_position), '')) from information_schema.columns where table_schema = 'public' and table_name in ('wallet_notifications', 'templates', 'notifications', 'notification_template_versions', 'notification_preferences', 'notification_inbox', 'notification_outbox', 'notification_channel_jobs', 'notification_delivery_attempts', 'notification_dead_letters', 'notification_replay_cursors', 'notification_push_subscriptions', 'notification_request_idempotency', 'notification_provider_events', 'notification_engagement', 'notification_template_audit', 'notification_expirations')")
[ "$recovery_rows" = "$legacy_rows_after" ] || die "recovery row count differs from source"
[ "$recovery_checksum" = "$target_schema_checksum" ] || die "recovery schema checksum differs from source"

if [ -n "$report_path" ]; then
  case "$report_path" in
    http://*|https://*) die "report must be a local path" ;;
  esac
  report_dir=$(dirname -- "$report_path")
  mkdir -p "$report_dir"
  printf '{\n  "schemaVersion": 1,\n  "databaseClass": "local-scratch-only",\n  "cleanMigrationLedgerRows": %s,\n  "legacyRowsBeforeUpgrade": %s,\n  "legacyRowsAfterUpgrade": %s,\n  "targetTables": %s,\n  "legacySchemaChecksum": "%s",\n  "targetSchemaChecksum": "%s",\n  "recoveryRows": %s,\n  "recoverySchemaChecksum": "%s",\n  "constraintsVerified": true,\n  "expiryFilterVerified": true,\n  "productionReady": false\n}\n' "$migration_rows" "$legacy_rows_before" "$legacy_rows_after" "$target_tables" "$legacy_schema_checksum" "$target_schema_checksum" "$recovery_rows" "$recovery_checksum" > "$report_path"
fi

echo "notification-migration-live: PASS — clean ledger=$migration_rows, populated legacy rows=$legacy_rows_before->$legacy_rows_after, target tables=$target_tables, constraints=verified, expiry-filter=verified, dump/restore=matched"
echo "notification-migration-live: LIMIT — scratch local PostgreSQL only; no production database, deployment, provider, or cutover evidence"
