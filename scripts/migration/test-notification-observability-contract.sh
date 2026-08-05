#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/notification-observability.json"
allow_local=false

die() {
  echo "notification-observability-contract: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --allow-local)
      allow_local=true
      shift
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[ "$allow_local" = true ] || die "pass --allow-local; this command only reads the local contract"
[ -f "$contract" ] || die "missing contract: $contract"
command -v bun >/dev/null 2>&1 || die "bun is required"
command -v git >/dev/null 2>&1 || die "git is required"

CONTRACT="$contract" bun -e '
const contract = await Bun.file(process.env.CONTRACT).json();
const fail = (message) => { console.error(message); process.exit(1); };
if (contract.schemaVersion !== 1 || contract.contractId !== "A11.7-notification-observability") fail("contract sentinel drifted");
if (contract.productionReady !== false || !contract.scope.includes("design only")) fail("observability contract is not non-production scoped");
const requiredLabels = ["service", "environment", "channel", "state", "outcome"];
if (JSON.stringify(contract.metricLabels) !== JSON.stringify(requiredLabels)) fail("metric label allowlist drifted");
for (const forbidden of ["wallet", "recipient", "email", "endpoint", "body", "token", "subject", "payload"]) if (!contract.forbiddenMetricLabels.includes(forbidden)) fail(`forbidden label missing: ${forbidden}`);
const requiredMetrics = ["notification_queue_depth", "notification_queue_age_seconds", "notification_provider_acceptance_percent", "notification_dead_letters", "notification_sse_lag_seconds", "notification_stream_query_failures_total"];
if (contract.metrics.length !== requiredMetrics.length || contract.metrics.some((metric) => !requiredMetrics.includes(metric.name) || metric.redacted !== true)) fail("redacted metric set drifted");
const slos = contract.slos;
if (slos.maxQueueDepth !== 10000 || slos.maxQueueAgeSeconds !== 300 || slos.minProviderAcceptancePercent !== 95 || slos.maxDeadLetters !== 0 || slos.maxSseLagSeconds !== 30 || slos.maxStreamQueryFailures !== 0) fail("SLO thresholds drifted");
const requiredAlerts = ["notification_dependency_not_ready", "notification_queue_depth_high", "notification_queue_age_high", "notification_provider_acceptance_low", "notification_dead_letters_present", "notification_sse_lag_high", "notification_stream_query_failures"];
if (contract.alerts.length !== requiredAlerts.length || contract.alerts.some((alert) => !requiredAlerts.includes(alert.name) || !["warning", "critical"].includes(alert.severity) || alert.runbook !== "NOTIFICATION_OPERATIONS_RUNBOOK.md")) fail("alert contract drifted");
if (contract.dashboardPanels.length !== 6 || contract.dashboardPanels.some((panel) => panel.includes("wallet") || panel.includes("recipient") || panel.includes("email"))) fail("dashboard panel set is not redacted");
if (contract.privacy.logsContainNoUserContent !== true || contract.privacy.logsContainNoBearerTokens !== true || contract.privacy.providerPayloadsExcluded !== true || contract.privacy.dashboardLabelsAreAllowlisted !== true) fail("privacy observability guard drifted");
console.log("notification-observability-contract: PASS — redacted metrics, SLO thresholds, alerts, dashboard panels, and privacy guards verified");
' 

echo "notification-observability-contract: LIMIT — static design contract only; no telemetry backend, alert, dashboard, provider, deployment, or production execution"
