#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/notification-cutover-rollback.json"
allow_local=false

die() {
  echo "notification-cutover-contract: ERROR: $*" >&2
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
if (contract.schemaVersion !== 1 || contract.contractId !== "A11.8-notification-cutover-rollback") fail("contract sentinel drifted");
if (contract.productionReady !== false || !contract.scope.includes("static validation only")) fail("contract is not non-production scoped");
const writer = contract.singleWriter;
if (writer.authority !== "legacy" || writer.defaultValue !== "legacy" || writer.switchControl !== "NOTIFICATION_WRITE_AUTHORITY" || writer.dualWrite !== false || writer.auditRequired !== true || writer.unknownValue !== "fail_closed") fail("single-writer default is not fail-closed legacy authority");
if (JSON.stringify(writer.allowedValues) !== JSON.stringify(["legacy", "service"])) fail("single-writer allowlist drifted");
const shadow = contract.shadow;
if (shadow.enabledByDefault !== false || shadow.serveTargetResults !== false || shadow.maxUnexplainedMismatches !== 0 || shadow.source !== "legacy" || shadow.target !== "service") fail("shadow contract can serve target results or tolerate drift");
for (const field of ["source_event_id", "wallet_address", "status", "broadcast", "template_id", "preference_hash", "provider_message_id", "provider_event_id"]) if (!shadow.compareFields.includes(field)) fail(`shadow field missing: ${field}`);
const canary = contract.canary;
if (canary.enabledByDefault !== false || canary.requiresShadowClean !== true || canary.requiresReconciliationApproval !== true || canary.maxRecipientsPerEvent !== 10 || canary.maxProviderAttemptsPerEvent !== 1 || canary.allowlistedWallets.length !== 0 || JSON.stringify(canary.allowlistedEventTypes) !== JSON.stringify(["notification.canary"])) fail("canary is not disabled and bounded by the reviewed allowlist");
if (!Array.isArray(contract.abortThresholds) || contract.abortThresholds.length !== 5 || !contract.abortThresholds.some((value) => value.includes("owner or broadcast mismatch")) || !contract.abortThresholds.some((value) => value.includes("provider acceptance"))) fail("abort thresholds are incomplete");
const rollback = contract.rollback;
if (rollback.preserveDurableRecords !== true || rollback.preserveInboxIdempotencyProviderEvents !== true || rollback.replayAcceptedProviderSends !== false || rollback.destructiveSchemaRollback !== false || rollback.steps.length !== 6) fail("rollback does not preserve durable/idempotency/provider state");
if (!rollback.steps.some((value) => value.includes("NOTIFICATION_WRITE_AUTHORITY=legacy")) || !rollback.steps.some((value) => value.includes("without replaying accepted provider sends"))) fail("rollback authority or replay guard is missing");
const order = contract.executionOrder;
if (JSON.stringify(order) !== JSON.stringify(["shadow_compare_without_serving", "approve_allowlisted_canary", "switch_one_writer", "observe_abort_thresholds", "reconcile_after_window", "rollback_without_destructive_schema_or_provider_replay"])) fail("execution order drifted");
if (contract.requiredApprovals.length !== 5) fail("required approval set drifted");
console.log("notification-cutover-contract: PASS — single-writer default, shadow/no-serve, canary allowlist, and duplicate-safe rollback requirements verified");
' 

echo "notification-cutover-contract: LIMIT — static non-production contract only; no switch, shadow, canary, deployment, provider, database, or rollback execution"
