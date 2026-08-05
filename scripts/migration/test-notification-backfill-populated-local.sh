#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)

database_url="${NOTIFICATION_RECONCILIATION_DATABASE_URL-}"
allow_local=false

die() {
  echo "notification-backfill-populated-local: ERROR: $*" >&2
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
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command mutates only an explicitly-created local scratch database"
[ -n "$database_url" ] || die "--database-url or NOTIFICATION_RECONCILIATION_DATABASE_URL is required"
for command_name in cargo diesel git jq psql; do
  command -v "$command_name" >/dev/null 2>&1 || die "$command_name is required"
done

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
  epsx_notification_reconcile_*) ;;
  *) die "database name must start with epsx_notification_reconcile_" ;;
esac
case "$database_name" in
  *prod*|*production*|*staging*|*stage*) die "production/staging-looking database name refused" ;;
esac
case "$database_base" in
  postgres://*localhost:*|postgres://*127.0.0.1:*|postgres://*\[::1\]:*|postgresql://*localhost:*|postgresql://*127.0.0.1:*|postgresql://*\[::1\]:*) ;;
  *) die "database host must be localhost, 127.0.0.1, or ::1" ;;
esac

maintenance_url="${database_base}/postgres${query_suffix}"
database_owner=$(psql "$maintenance_url" -Atqc 'select current_user' 2>/dev/null || true)
[ -n "$database_owner" ] || die "could not connect to the local PostgreSQL maintenance database"
case "$database_owner" in
  *[!A-Za-z0-9_]*|'') die "current PostgreSQL role is not safe for scratch ownership" ;;
esac

database_exists=$(psql "$maintenance_url" -Atqc "select 1 from pg_database where datname = '$database_name'" 2>/dev/null || true)
[ -z "$database_exists" ] || die "database already exists; choose a fresh scratch name"

temp_root=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-backfill-populated.XXXXXX")
database_created=0
cleanup() {
  rm -rf -- "$temp_root"
  if [ "$database_created" = 1 ]; then
    psql "$maintenance_url" -v ON_ERROR_STOP=1 -qAtc "DROP DATABASE IF EXISTS \"$database_name\" WITH (FORCE)" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT HUP INT TERM

psql "$maintenance_url" -v ON_ERROR_STOP=1 -qAtc "CREATE DATABASE \"$database_name\" OWNER \"$database_owner\""
database_created=1
DATABASE_URL="$database_url" diesel migration run --migration-dir "$repo_root/apps/backend/migrations/notifications" >/dev/null

# Populate the legacy table with owner, delivered, failed, and broadcast rows.
psql "$database_url" -v ON_ERROR_STOP=1 -qAt <<'SQL'
INSERT INTO public.wallet_notifications
  (id, recipient_wallet_address, topic_name, title, body, notification_type, status)
VALUES
  ('11111111-1111-4111-8111-111111111111', '0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', NULL, 'Legacy pending', 'Backfill audit', 'system', 'created'),
  ('22222222-2222-4222-8222-222222222222', '0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB', NULL, 'Legacy sent', 'Backfill audit', 'system', 'delivered'),
  ('33333333-3333-4333-8333-333333333333', '0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC', NULL, 'Legacy failed', 'Backfill audit', 'system', 'failed'),
  ('44444444-4444-4444-8444-444444444444', 'all', NULL, 'Legacy broadcast', 'Backfill audit', 'system', 'suppressed');
SQL

psql "$database_url" -Atqc "
  SELECT json_build_object(
    'id', id::text,
    'recipient_wallet_address', recipient_wallet_address,
    'topic_name', topic_name,
    'status', status
  )::text
  FROM public.wallet_notifications
  ORDER BY id
" >"$temp_root/legacy.jsonl"

cargo xtask notification-backfill --dry-run --legacy --input "$temp_root/legacy.jsonl" >"$temp_root/backfill.out"
grep -q "format=legacy records=4 eligible=4 invalid=0 duplicate_source_events=0 legacy_records=4" "$temp_root/backfill.out" || {
  cat "$temp_root/backfill.out" >&2
  die "populated legacy rows were not mapped cleanly"
}
for mapped_status in '"failed":1' '"pending":1' '"sent":1' '"suppressed":1'; do
  grep -q "$mapped_status" "$temp_root/backfill.out" || die "missing mapped status $mapped_status"
done

# Materialize the same four identities into the target projection. The sent
# row also gets a durable provider acceptance so reconciliation checks provider
# identity instead of treating delivery as a plain status conversion.
psql "$database_url" -v ON_ERROR_STOP=1 -qAt <<'SQL'
WITH mapped AS (
  SELECT
    id,
    lower(recipient_wallet_address) AS wallet_address,
    CASE lower(status)
      WHEN 'created' THEN 'pending'
      WHEN 'queued' THEN 'pending'
      WHEN 'scheduled' THEN 'pending'
      WHEN 'sent' THEN 'sent'
      WHEN 'delivered' THEN 'sent'
      WHEN 'read' THEN 'sent'
      WHEN 'failed' THEN 'failed'
      WHEN 'suppressed' THEN 'suppressed'
      WHEN 'cancelled' THEN 'cancelled'
      WHEN 'expired' THEN 'expired'
      WHEN 'deleted' THEN 'deleted'
    END AS mapped_status
  FROM public.wallet_notifications
)
INSERT INTO public.notifications
  (id, user_id, channel, recipient, body, status, data, title, notification_type)
SELECT
  'legacy.wallet_notification:' || id::text,
  CASE WHEN wallet_address = 'all' THEN NULL ELSE wallet_address END,
  'in_app',
  wallet_address,
  'Backfill audit',
  mapped_status,
  jsonb_build_object(
    'source_event_id', 'legacy.wallet_notification:' || id::text,
    'broadcast', wallet_address = 'all'
  ),
  'Backfill audit',
  'system'
FROM mapped;

WITH delivered AS (
  SELECT id, lower(recipient_wallet_address) AS wallet_address
  FROM public.wallet_notifications
  WHERE lower(status) IN ('sent', 'delivered', 'read')
)
INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload)
SELECT
  'legacy.wallet_notification:' || id::text || ':outbox',
  'legacy.backfill',
  'legacy.wallet_notification:' || id::text,
  '{}'::jsonb
FROM delivered;

WITH delivered AS (
  SELECT id, lower(recipient_wallet_address) AS wallet_address
  FROM public.wallet_notifications
  WHERE lower(status) IN ('sent', 'delivered', 'read')
)
INSERT INTO public.notification_channel_jobs
  (id, source_event_id, notification_id, channel, recipient, state, idempotency_key, provider_message_id)
SELECT
  'legacy.wallet_notification:' || id::text || ':in_app',
  'legacy.wallet_notification:' || id::text || ':outbox',
  'legacy.wallet_notification:' || id::text,
  'in_app',
  wallet_address,
  'provider_accepted',
  'legacy.wallet_notification:' || id::text || ':in_app',
  'provider-message:' || id::text
FROM delivered;
SQL

psql "$database_url" -Atqc "
  SELECT json_build_object(
    'source_event_id', 'legacy.wallet_notification:' || id::text,
    'wallet_address', lower(recipient_wallet_address),
    'status', CASE lower(status)
      WHEN 'created' THEN 'pending'
      WHEN 'queued' THEN 'pending'
      WHEN 'scheduled' THEN 'pending'
      WHEN 'sent' THEN 'sent'
      WHEN 'delivered' THEN 'sent'
      WHEN 'read' THEN 'sent'
      WHEN 'failed' THEN 'failed'
      WHEN 'suppressed' THEN 'suppressed'
      WHEN 'cancelled' THEN 'cancelled'
      WHEN 'expired' THEN 'expired'
      WHEN 'deleted' THEN 'deleted'
    END,
    'provider_message_id', CASE WHEN lower(status) IN ('sent', 'delivered', 'read') THEN 'provider-message:' || id::text ELSE NULL END,
    'provider_event_id', NULL,
    'template_id', NULL,
    'preference_hash', NULL,
    'broadcast', lower(recipient_wallet_address) = 'all'
  )::text
  FROM public.wallet_notifications
  ORDER BY id
" >"$temp_root/source.jsonl"

psql "$database_url" -Atqc "
  SELECT json_build_object(
    'source_event_id', n.data->>'source_event_id',
    'wallet_address', n.recipient,
    'status', n.status,
    'provider_message_id', j.provider_message_id,
    'provider_event_id', NULL,
    'template_id', n.template_id,
    'preference_hash', NULL,
    'broadcast', n.recipient = 'all'
  )::text
  FROM public.notifications n
  LEFT JOIN public.notification_channel_jobs j ON j.notification_id = n.id
  ORDER BY n.id
" >"$temp_root/target.jsonl"

legacy_count=$(psql "$database_url" -Atqc 'SELECT count(*) FROM public.wallet_notifications')
target_count=$(psql "$database_url" -Atqc 'SELECT count(*) FROM public.notifications')
[ "$legacy_count" = 4 ] || die "expected four populated legacy rows, found $legacy_count"
[ "$target_count" = 4 ] || die "expected four populated target rows, found $target_count"

cargo xtask notification-reconcile --dry-run --source "$temp_root/source.jsonl" --target "$temp_root/target.jsonl" >"$temp_root/reconcile.out"
sed -n '/^{/,$p' "$temp_root/reconcile.out" | sed '/^notification-reconcile:/,$d' >"$temp_root/reconcile.json"
jq -e '
  .source_records == 4 and
  .target_records == 4 and
  .invalid_source_records == 0 and
  .invalid_target_records == 0 and
  .duplicate_source_events == 0 and
  .duplicate_target_events == 0 and
  .missing_target_events == 0 and
  .orphan_target_events == 0 and
  .target_sent_without_provider_id == 0 and
  .template_identity_drift == 0 and
  .preference_identity_drift == 0 and
  .provider_identity_drift == 0 and
  .wallet_checksum_match and
  .source_target_event_set_match and
  .status_distribution_match and
  .broadcast_count_match and
  .source_broadcast_records == 1 and
  .target_broadcast_records == 1
' "$temp_root/reconcile.json" >/dev/null || {
  cat "$temp_root/reconcile.out" >&2
  die "populated source/target reconciliation drifted"
}
grep -q "notification-reconcile: writes=0 network=0 database=0" "$temp_root/reconcile.out"

echo "notification-backfill-populated-local: PASS — 4 legacy rows mapped, 4 target rows reconciled, provider identity and broadcast parity verified"
echo "notification-backfill-populated-local: LIMIT — disposable local PostgreSQL only; no staging-scale, production, provider, or cutover evidence"
