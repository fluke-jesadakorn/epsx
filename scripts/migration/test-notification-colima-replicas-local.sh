#!/bin/sh
set -eu

# Reproducible, disposable Colima evidence for the notification deployment.
# This intentionally applies only the notification Deployment/Service to a
# temporary namespace and uses a scratch local PostgreSQL database. It never
# applies the full staging/prod overlay and never touches a production DB.

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
namespace="epsx-staging-audit-$$"
database_name="epsx_notification_colima_audit_$(date +%s)_$$"
database_url=""
queue_records=0
scale_temp_dir=""
port_forward_pid=""
port_forward_log=""
local_port=$((18100 + ($$ % 100)))

die() {
  echo "notification-colima-replicas-local: ERROR: $*" >&2
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
      [ "$#" -ge 2 ] || die "--port requires a local port"
      local_port=$2
      shift 2
      ;;
    --queue-records)
      [ "$#" -ge 2 ] || die "--queue-records requires a bounded integer"
      queue_records=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this mutates only disposable local state"
case "$database_name" in
  epsx_notification_colima_audit_*) ;;
  *) die "database name must start with epsx_notification_colima_audit_" ;;
esac
case "$database_name" in
  *prod*|*production*|*staging*|*stage*) die "production/staging-looking database name refused" ;;
esac
case "$local_port" in
  *[!0-9]*|'') die "port must be numeric" ;;
esac
[ "$local_port" -ge 1024 ] && [ "$local_port" -le 65535 ] || die "port must be between 1024 and 65535"
case "$queue_records" in
  *[!0-9]*|'') die "queue-records must be a non-negative integer" ;;
esac
[ "$queue_records" -le 5000 ] || die "queue-records is bounded at 5000"

for command_name in cargo colima docker kubectl psql diesel curl jq git redis-cli; do
  command -v "$command_name" >/dev/null 2>&1 || die "$command_name is required"
done

colima_status=$(colima status --profile epsx 2>&1 || true)
printf '%s\n' "$colima_status" | grep -q 'runtime: docker' || die "Colima profile epsx is not running with Docker"
docker --context colima-epsx image inspect epsx-notification:staging >/dev/null 2>&1 \
  || die "epsx-notification:staging is not loaded in the Colima Docker context"

kubectl_context=colima-epsx
kubectl --context "$kubectl_context" get node >/dev/null 2>&1 \
  || die "Kubernetes context $kubectl_context is unavailable"
if kubectl --context "$kubectl_context" get namespace "$namespace" >/dev/null 2>&1; then
  die "namespace $namespace already exists; refusing to touch non-disposable state"
fi

database_owner=$(psql -h localhost -d postgres -Atqc 'select current_user' 2>/dev/null || true)
[ -n "$database_owner" ] || die "could not connect to the local PostgreSQL maintenance database"
case "$database_owner" in
  *[!A-Za-z0-9_]*|'') die "current PostgreSQL role is not safe for scratch ownership" ;;
esac
database_url="postgres://${database_owner}@localhost:5432/${database_name}"

remove_namespace() {
  kubectl --context "$kubectl_context" delete namespace "$namespace" \
    --ignore-not-found --wait=false >/dev/null 2>&1 || true
  attempt=1
  while [ "$attempt" -le 30 ]; do
    if ! kubectl --context "$kubectl_context" get namespace "$namespace" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
    attempt=$((attempt + 1))
  done
  # k3s can leave terminating pod objects behind while a disposable namespace
  # is waiting on its finalizer. The namespace is already deletion-marked, so
  # force-delete only its audit pods before finalizing the exact scratch target.
  kubectl --context "$kubectl_context" -n "$namespace" delete pods --all \
    --force --grace-period=0 --ignore-not-found >/dev/null 2>&1 || true
  # Colima's local k3s can retain the namespace finalizer after all content is
  # gone. This exact namespace was created by this script, so finalize only
  # that disposable target rather than leaving scratch state behind.
  if kubectl --context "$kubectl_context" get namespace "$namespace" >/dev/null 2>&1; then
    kubectl --context "$kubectl_context" get namespace "$namespace" -o json \
      | jq '.spec.finalizers=[]' \
      | kubectl --context "$kubectl_context" replace --raw \
        "/api/v1/namespaces/$namespace/finalize" -f - >/dev/null 2>&1 || true
  fi
}

cleanup() {
  if [ -n "$port_forward_pid" ] && kill -0 "$port_forward_pid" 2>/dev/null; then
    kill "$port_forward_pid" 2>/dev/null || true
    wait "$port_forward_pid" 2>/dev/null || true
  fi
  remove_namespace
  psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc \
    "DROP DATABASE IF EXISTS \"$database_name\" WITH (FORCE)" >/dev/null 2>&1 || true
  rm -rf -- "$scale_temp_dir"
  rm -f -- "$port_forward_log"
}
trap cleanup EXIT
trap 'cleanup; exit 130' HUP INT TERM

existing=$(psql -h localhost -d postgres -Atqc \
  "select 1 from pg_database where datname = '$database_name'" 2>/dev/null || true)
[ -z "$existing" ] || die "database already exists; choose a fresh --database-name"
psql -h localhost -d postgres -v ON_ERROR_STOP=1 -qAtc \
  "CREATE DATABASE \"$database_name\" OWNER \"$database_owner\""
DATABASE_URL="$database_url" diesel migration run \
  --migration-dir "$repo_root/apps/backend/migrations/notifications" >/dev/null

if [ "$queue_records" -gt 0 ]; then
  # Keep the jobs queued but not claimable yet. This gives the service a
  # deterministic bounded workload while the delivery worker remains idle.
  psql "$database_url" -v ON_ERROR_STOP=1 -v scale_records="$queue_records" -qAt <<'SQL'
WITH rows AS (
  SELECT n, format('colima-scale-%s', n) AS suffix
  FROM generate_series(1, :'scale_records'::int) AS n
)
INSERT INTO public.notifications (id, user_id, channel, recipient, body, status, data)
SELECT
  'colima-scale-notification-' || suffix,
  '0x1111111111111111111111111111111111111111',
  'in_app',
  '0x1111111111111111111111111111111111111111',
  'Colima bounded queue audit',
  'pending',
  jsonb_build_object('source_event_id', 'colima-scale-event-' || suffix)
FROM rows;

WITH rows AS (
  SELECT n, format('colima-scale-%s', n) AS suffix
  FROM generate_series(1, :'scale_records'::int) AS n
)
INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload)
SELECT
  'colima-scale-event-' || suffix,
  'notification.colima.scale',
  'colima-scale-notification-' || suffix,
  '{}'::jsonb
FROM rows;

WITH rows AS (
  SELECT n, format('colima-scale-%s', n) AS suffix
  FROM generate_series(1, :'scale_records'::int) AS n
)
INSERT INTO public.notification_channel_jobs
  (id, source_event_id, notification_id, channel, recipient, state, idempotency_key, available_at)
SELECT
  'colima-scale-job-' || suffix,
  'colima-scale-event-' || suffix,
  'colima-scale-notification-' || suffix,
  'in_app',
  '0x1111111111111111111111111111111111111111',
  'queued',
  'colima-scale-idempotency-' || suffix,
  NOW() + INTERVAL '1 hour'
FROM rows;
SQL
  queued_rows=$(psql "$database_url" -Atqc "SELECT count(*) FROM public.notification_channel_jobs WHERE id LIKE 'colima-scale-job-%' AND state = 'queued'")
  [ "$queued_rows" = "$queue_records" ] || die "expected $queue_records queued scale jobs, found $queued_rows"

  # Populate the legacy and target projections with the same bounded workload
  # so the local Colima rehearsal also exercises the backfill/reconciliation
  # path against real rows rather than only checked-in fixtures.
  psql "$database_url" -v ON_ERROR_STOP=1 -v scale_records="$queue_records" -qAt <<'SQL'
WITH rows AS (
  SELECT
    ('00000000-0000-4000-8000-' || lpad(to_hex(n), 12, '0'))::uuid AS id
  FROM generate_series(1, :'scale_records'::int) AS n
)
INSERT INTO public.wallet_notifications
  (id, recipient_wallet_address, topic_name, title, body, notification_type, status)
SELECT
  id,
  '0x1111111111111111111111111111111111111111',
  NULL,
  'Colima legacy scale audit',
  'Colima populated backfill audit',
  'system',
  'created'
FROM rows;

WITH legacy AS (
  SELECT id::text AS legacy_id, lower(recipient_wallet_address) AS wallet_address
  FROM public.wallet_notifications
)
INSERT INTO public.notifications
  (id, user_id, channel, recipient, body, status, data, title, notification_type)
SELECT
  'legacy.wallet_notification:' || legacy_id,
  wallet_address,
  'in_app',
  wallet_address,
  'Colima populated backfill audit',
  'pending',
  jsonb_build_object(
    'source_event_id', 'legacy.wallet_notification:' || legacy_id,
    'broadcast', false
  ),
  'Colima legacy scale audit',
  'system'
FROM legacy;
SQL

  scale_temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-colima-scale.XXXXXX")
  psql "$database_url" -Atqc "
    SELECT json_build_object(
      'id', id::text,
      'recipient_wallet_address', recipient_wallet_address,
      'topic_name', topic_name,
      'status', status
    )::text
    FROM public.wallet_notifications
    ORDER BY id
  " >"$scale_temp_dir/legacy.jsonl"
  cargo xtask notification-backfill --dry-run --legacy --input "$scale_temp_dir/legacy.jsonl" >"$scale_temp_dir/backfill.out"
  grep -q "format=legacy records=1000 eligible=1000 invalid=0 duplicate_source_events=0 legacy_records=1000" "$scale_temp_dir/backfill.out" \
    || die "populated Colima legacy backfill did not map 1000 rows"
  grep -q '"pending":1000' "$scale_temp_dir/backfill.out" \
    || die "populated Colima legacy backfill status distribution drifted"

  psql "$database_url" -Atqc "
    SELECT json_build_object(
      'source_event_id', 'legacy.wallet_notification:' || id::text,
      'wallet_address', lower(recipient_wallet_address),
      'status', 'pending',
      'provider_message_id', NULL,
      'provider_event_id', NULL,
      'template_id', NULL,
      'preference_hash', NULL,
      'broadcast', false
    )::text
    FROM public.wallet_notifications
    ORDER BY id
  " >"$scale_temp_dir/source.jsonl"
  psql "$database_url" -Atqc "
    SELECT json_build_object(
      'source_event_id', n.data->>'source_event_id',
      'wallet_address', n.recipient,
      'status', n.status,
      'provider_message_id', NULL,
      'provider_event_id', NULL,
      'template_id', n.template_id,
      'preference_hash', NULL,
      'broadcast', false
    )::text
    FROM public.notifications n
    WHERE n.id LIKE 'legacy.wallet_notification:%'
    ORDER BY n.id
  " >"$scale_temp_dir/target.jsonl"
  legacy_scale_count=$(psql "$database_url" -Atqc "SELECT count(*) FROM public.wallet_notifications")
  target_scale_count=$(psql "$database_url" -Atqc "SELECT count(*) FROM public.notifications WHERE id LIKE 'legacy.wallet_notification:%'")
  [ "$legacy_scale_count" = "$queue_records" ] || die "expected $queue_records populated legacy rows, found $legacy_scale_count"
  [ "$target_scale_count" = "$queue_records" ] || die "expected $queue_records populated target rows, found $target_scale_count"

  cargo xtask notification-reconcile --dry-run \
    --source "$scale_temp_dir/source.jsonl" \
    --target "$scale_temp_dir/target.jsonl" >"$scale_temp_dir/reconcile.out"
  sed -n '/^{/,$p' "$scale_temp_dir/reconcile.out" \
    | sed '/^notification-reconcile:/,$d' >"$scale_temp_dir/reconcile.json"
  jq -e '
    .source_records == 1000 and
    .target_records == 1000 and
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
    .broadcast_count_match
  ' "$scale_temp_dir/reconcile.json" >/dev/null || {
    cat "$scale_temp_dir/reconcile.out" >&2
    die "populated Colima source/target reconciliation drifted"
  }
fi

redis-cli -h localhost -p 6379 -a epsx ping 2>/dev/null | grep -qx PONG \
  || die "host Redis did not respond to PING"

kubectl --context "$kubectl_context" create namespace "$namespace" >/dev/null
kubectl --context "$kubectl_context" -n "$namespace" create secret generic epsx-notification \
  --save-config \
  --from-literal=DATABASE_URL="postgres://$database_owner@host.docker.internal:5432/$database_name" \
  --from-literal=OIDC_ISSUER="https://staging-identity.invalid" \
  --from-literal=OIDC_JWKS_URL="https://staging-identity.invalid/.well-known/jwks.json" \
  --from-literal=REDIS_URL="redis://:epsx@host.docker.internal:6379" \
  --from-literal=SMTP_HOST='' \
  --from-literal=SMTP_PORT=587 \
  --from-literal=SMTP_USER='' \
  --from-literal=SMTP_PASSWORD='' \
  --from-literal=FROM_ADDRESS='notification-audit@example.invalid' \
  --from-literal=FROM_NAME='Notification audit' >/dev/null

{
  sed -e 's/image: epsx-notification:prod/image: epsx-notification:staging/' \
      -e 's/replicas: 1/replicas: 2/' \
      -e 's/value: "production"/value: "development"/' \
      "$repo_root/infrastructure/kubernetes/base/notification/deployment.yaml"
  printf '%s\n' '---'
  sed -n '/^apiVersion:/,$p' "$repo_root/infrastructure/kubernetes/base/notification/service.yaml"
} | kubectl --context "$kubectl_context" -n "$namespace" apply -f - >/dev/null

wait_for_replicas() {
  desired=$1
  attempt=1
  while [ "$attempt" -le 90 ]; do
    ready_replicas=$(kubectl --context "$kubectl_context" -n "$namespace" \
      get deployment epsx-notification -o jsonpath='{.status.readyReplicas}' 2>/dev/null || true)
    [ "${ready_replicas:-0}" = "$desired" ] && return 0
    if [ "$attempt" -eq 90 ]; then
      kubectl --context "$kubectl_context" -n "$namespace" get pods -o wide >&2 || true
      kubectl --context "$kubectl_context" -n "$namespace" describe deployment epsx-notification >&2 || true
      return 1
    fi
    sleep 2
    attempt=$((attempt + 1))
  done
}

wait_for_rollout() {
  kubectl --context "$kubectl_context" -n "$namespace" rollout status \
    deployment/epsx-notification --timeout=180s >/dev/null || return 1
  wait_for_replicas 2
}

wait_for_rollout || die "two notification replicas did not become Ready"
replica_summary=$(kubectl --context "$kubectl_context" -n "$namespace" \
  get deployment epsx-notification -o jsonpath='{.status.replicas}/{.status.updatedReplicas}/{.status.readyReplicas}/{.status.availableReplicas}')
[ "$replica_summary" = '2/2/2/2' ] || die "unexpected replica summary: $replica_summary"

port_forward_log=$(mktemp "${TMPDIR:-/tmp}/epsx-notification-colima.XXXXXX")
start_port_forward() {
  if [ -n "$port_forward_pid" ] && kill -0 "$port_forward_pid" 2>/dev/null; then
    kill "$port_forward_pid" 2>/dev/null || true
    wait "$port_forward_pid" 2>/dev/null || true
  fi
  : >"$port_forward_log"
  kubectl --context "$kubectl_context" -n "$namespace" port-forward \
    service/epsx-notification "$local_port:8106" >"$port_forward_log" 2>&1 &
  port_forward_pid=$!
  sleep 1
}

start_port_forward
ready=false
attempt=1
while [ "$attempt" -le 30 ]; do
  if ready_payload=$(curl -fsS "http://127.0.0.1:$local_port/ready" 2>/dev/null) \
    && printf '%s' "$ready_payload" | jq -e '.status == "ready" and .database == true and .lifecycle == true and .redis_fanout_configured == true and .redis_reachable == true' >/dev/null; then
    ready=true
    break
  fi
  sleep 1
  attempt=$((attempt + 1))
done
[ "$ready" = true ] || { cat "$port_forward_log" >&2; die "notification readiness did not report Redis reachable"; }

if [ "$queue_records" -gt 0 ]; then
  printf '%s' "$ready_payload" | jq -e --argjson expected "$queue_records" '.queue_depth == $expected and .queue_age_seconds == 0' >/dev/null \
    || die "readiness did not expose the bounded queued workload: $ready_payload"
fi

# Exercise a dependency-loss/recovery transition without stopping the host
# Redis daemon: replace only the disposable Secret, restart the replicas, and
# assert that readiness observes the loss before restoring the real endpoint.
kubectl --context "$kubectl_context" -n "$namespace" create secret generic epsx-notification \
  --from-literal=DATABASE_URL="postgres://$database_owner@host.docker.internal:5432/$database_name" \
  --from-literal=OIDC_ISSUER="https://staging-identity.invalid" \
  --from-literal=OIDC_JWKS_URL="https://staging-identity.invalid/.well-known/jwks.json" \
  --from-literal=REDIS_URL='redis://:epsx@host.docker.internal:1' \
  --from-literal=SMTP_HOST='' --from-literal=SMTP_PORT=587 --from-literal=SMTP_USER='' \
  --from-literal=SMTP_PASSWORD='' --from-literal=FROM_ADDRESS='notification-audit@example.invalid' \
  --from-literal=FROM_NAME='Notification audit' --dry-run=client -o yaml \
  | kubectl --context "$kubectl_context" -n "$namespace" apply -f - >/dev/null
kubectl --context "$kubectl_context" -n "$namespace" rollout restart deployment/epsx-notification >/dev/null
wait_for_rollout || die "replicas did not recover after Redis loss configuration"
start_port_forward
redis_loss_payload=''
attempt=1
while [ "$attempt" -le 30 ]; do
  redis_loss_payload=$(curl -fsS "http://127.0.0.1:$local_port/ready" 2>/dev/null || true)
  if printf '%s' "$redis_loss_payload" | jq -e '.status == "ready" and .database == true and .lifecycle == true and .redis_fanout_configured == true and .redis_reachable == false' >/dev/null 2>&1; then
    break
  fi
  sleep 1
  attempt=$((attempt + 1))
done
printf '%s' "$redis_loss_payload" | jq -e '.status == "ready" and .database == true and .lifecycle == true and .redis_fanout_configured == true and .redis_reachable == false' >/dev/null \
  || die "readiness did not expose bounded Redis loss: $redis_loss_payload"

kubectl --context "$kubectl_context" -n "$namespace" create secret generic epsx-notification \
  --from-literal=DATABASE_URL="postgres://$database_owner@host.docker.internal:5432/$database_name" \
  --from-literal=OIDC_ISSUER="https://staging-identity.invalid" \
  --from-literal=OIDC_JWKS_URL="https://staging-identity.invalid/.well-known/jwks.json" \
  --from-literal=REDIS_URL="redis://:epsx@host.docker.internal:6379" \
  --from-literal=SMTP_HOST='' --from-literal=SMTP_PORT=587 --from-literal=SMTP_USER='' \
  --from-literal=SMTP_PASSWORD='' --from-literal=FROM_ADDRESS='notification-audit@example.invalid' \
  --from-literal=FROM_NAME='Notification audit' --dry-run=client -o yaml \
  | kubectl --context "$kubectl_context" -n "$namespace" apply -f - >/dev/null
kubectl --context "$kubectl_context" -n "$namespace" rollout restart deployment/epsx-notification >/dev/null
wait_for_rollout || die "replicas did not recover after Redis restore configuration"
start_port_forward
redis_restore_payload=''
attempt=1
while [ "$attempt" -le 30 ]; do
  redis_restore_payload=$(curl -fsS "http://127.0.0.1:$local_port/ready" 2>/dev/null || true)
  if printf '%s' "$redis_restore_payload" | jq -e '.status == "ready" and .database == true and .lifecycle == true and .redis_fanout_configured == true and .redis_reachable == true' >/dev/null 2>&1; then
    break
  fi
  sleep 1
  attempt=$((attempt + 1))
done
printf '%s' "$redis_restore_payload" | jq -e '.status == "ready" and .database == true and .lifecycle == true and .redis_fanout_configured == true and .redis_reachable == true' >/dev/null \
  || die "readiness did not recover Redis reachability: $redis_restore_payload"

restarts=$(kubectl --context "$kubectl_context" -n "$namespace" \
  get pods -l app=epsx-notification -o jsonpath='{range .items[*]}{.status.containerStatuses[0].restartCount}{"\n"}{end}')
if printf '%s\n' "$restarts" | grep -q '[1-9]'; then
  die "notification replicas reported restarts: $restarts"
fi

echo "notification-colima-replicas-local: PASS — Colima two-replica rollout, Redis reachable/lost/recovered readiness, and zero pod restarts"
if [ "$queue_records" -gt 0 ]; then
  if [ "$queue_records" = 1000 ]; then
    echo "notification-colima-scale-local: PASS — 1000 future-dated channel jobs remained queued and populated backfill/reconciliation matched"
  else
    echo "notification-colima-scale-local: PASS — $queue_records future-dated channel jobs remained queued and populated backfill/reconciliation matched"
  fi
fi
echo "notification-colima-replicas-local: LIMIT — disposable local Colima/PostgreSQL/Redis only; no provider delivery, browser push, staging-scale, production, or cutover evidence"
