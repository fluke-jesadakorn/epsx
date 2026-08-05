#!/bin/sh
set -eu

# Offline observability proof. It evaluates only checked-in redacted metrics
# snapshots and the notification readiness contract; it never contacts a
# telemetry backend, provider, database, cluster, or production service.
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
allow_local=false
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-observability.XXXXXX")

die() {
  echo "notification-observability-local: ERROR: $*" >&2
  exit 1
}

cleanup() {
  rm -rf -- "$temp_dir"
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

[ "$allow_local" = true ] || die "pass --allow-local; this command is offline and local-only"
for command_name in bun cargo jq git; do
  command -v "$command_name" >/dev/null 2>&1 || die "$command_name is required"
done

healthy_out="$temp_dir/healthy.out"
unhealthy_out="$temp_dir/unhealthy.out"
cargo xtask notification-readiness --dry-run \
  --input "$repo_root/docs/migration/fixtures/notification-metrics-healthy.json" \
  >"$healthy_out" 2>&1 || die "healthy metrics snapshot was rejected"
set +e
cargo xtask notification-readiness --dry-run \
  --input "$repo_root/docs/migration/fixtures/notification-metrics-unhealthy.json" \
  >"$unhealthy_out" 2>&1
unhealthy_exit=$?
set -e
[ "$unhealthy_exit" -ne 0 ] || die "unhealthy metrics snapshot was accepted"
grep -q 'notification-readiness: healthy=true production_ready=false writes=0 network=0 database=0' "$healthy_out" \
  || die "healthy readiness output drifted"
grep -q 'notification readiness thresholds failed' "$unhealthy_out" \
  || die "unhealthy readiness did not fail closed"

CONTRACT="$repo_root/docs/migration/contracts/notification-observability.json" \
HEALTHY="$repo_root/docs/migration/fixtures/notification-metrics-healthy.json" \
UNHEALTHY="$repo_root/docs/migration/fixtures/notification-metrics-unhealthy.json" \
bun -e '
const contract = await Bun.file(process.env.CONTRACT).json();
const healthy = await Bun.file(process.env.HEALTHY).json();
const unhealthy = await Bun.file(process.env.UNHEALTHY).json();
const fail = (message) => { console.error(message); process.exit(1); };
const alertsFor = (snapshot) => {
  const acceptance = snapshot.delivery_attempts === 0 ? 100 : snapshot.provider_accepted * 100 / snapshot.delivery_attempts;
  const alerts = [];
  if (snapshot.queue_depth > contract.slos.maxQueueDepth) alerts.push("notification_queue_depth_high");
  if (snapshot.queue_age_seconds !== null && snapshot.queue_age_seconds > contract.slos.maxQueueAgeSeconds) alerts.push("notification_queue_age_high");
  if (acceptance < contract.slos.minProviderAcceptancePercent) alerts.push("notification_provider_acceptance_low");
  if (snapshot.dead_lettered > contract.slos.maxDeadLetters) alerts.push("notification_dead_letters_present");
  if (snapshot.stream_lag_seconds !== null && snapshot.stream_lag_seconds > contract.slos.maxSseLagSeconds) alerts.push("notification_sse_lag_high");
  if (snapshot.stream_query_failures_total > contract.slos.maxStreamQueryFailures) alerts.push("notification_stream_query_failures");
  return alerts;
};
if (alertsFor(healthy).length !== 0) fail("healthy snapshot produced alerts");
const expected = ["notification_queue_depth_high", "notification_queue_age_high", "notification_provider_acceptance_low", "notification_sse_lag_high", "notification_stream_query_failures"];
if (JSON.stringify(alertsFor(unhealthy)) !== JSON.stringify(expected)) fail(`unhealthy alert set drifted: ${JSON.stringify(alertsFor(unhealthy))}`);
if (contract.alerts.some((alert) => !alertsFor(unhealthy).includes(alert.name) && alert.name !== "notification_dependency_not_ready" && alert.name !== "notification_dead_letters_present")) fail("contract alert is not exercised by the unhealthy fixture");
console.log("notification-observability-local: PASS — healthy snapshot stayed alert-free and unhealthy snapshot raised bounded queue, provider, SSE-lag, and query-failure alerts");
' 

echo "notification-observability-local: LIMIT — offline fixtures and readiness evaluator only; no live telemetry, alert receiver, dashboard, provider, deployment, or production evidence"
