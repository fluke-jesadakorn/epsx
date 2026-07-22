#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-notification-execution.sh"
contract="$repo_root/docs/migration/contracts/notification-execution.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-execution.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "14 source records, 58 target anchors, 12 surfaces, and 22 stop blockers" "$temp_dir/integrity.out"
grep -q "A2.3c auth and A3.11 schema boundary remain partial" "$temp_dir/integrity.out"
grep -q "no database, upgrade, reconciliation, Redis, SMTP, push, network, deployment" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "notification-execution self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "22 stop blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.readinessExit !== 3 || report.productionReady !== false) process.exit(1);
if (report.source.evidence !== 14 || report.targetEvidence !== 58 || report.surfaces.length !== 12 || report.blockers.length !== 22) process.exit(1);
if (report.directAuthPrerequisite !== "partial" || report.batches.join(",") !== "N1,N2,N3,N4,N5,N6,N7,N8") process.exit(1);
if (report.schemaBoundary.status !== "partial-static" || report.schemaBoundary.runtimeDdlFindings !== 0 || report.schemaBoundary.startupSeedCalls !== 0) process.exit(1);
' "$temp_dir/report-one.json"

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/missing-anchor.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.source.evidence[0].anchor = "tampered missing source anchor";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/missing-anchor.json" >"$temp_dir/missing-anchor.out" 2>&1
anchor_status=$?
set -e
if [ "$anchor_status" -ne 1 ]; then
  cat "$temp_dir/missing-anchor.out" >&2
  echo "notification-execution self-test: expected missing-anchor exit 1, got $anchor_status" >&2
  exit 1
fi
grep -q "missing source anchor" "$temp_dir/missing-anchor.out"

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/stale-a3-anchor.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.targetEvidence.find((item) => item.id === "tgt-startup-no-seeds").anchor = "tampered A3.11 startup boundary";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-a3-anchor.json" >"$temp_dir/stale-a3-anchor.out" 2>&1
stale_a3_status=$?
set -e
if [ "$stale_a3_status" -ne 1 ]; then
  cat "$temp_dir/stale-a3-anchor.out" >&2
  echo "notification-execution self-test: expected stale-A3.11-anchor exit 1, got $stale_a3_status" >&2
  exit 1
fi
grep -q "missing target anchor tgt-startup-no-seeds" "$temp_dir/stale-a3-anchor.out"

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/wrong-existing-ssr-anchor.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.targetEvidence.find((item) => item.id === "tgt-frontend-ssr-ok").anchor = ".get_with_ctx(\"/api/v1/notification/list\", &request_context)";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/wrong-existing-ssr-anchor.json" >"$temp_dir/wrong-existing-ssr-anchor.out" 2>&1
wrong_ssr_status=$?
set -e
if [ "$wrong_ssr_status" -ne 1 ]; then
  cat "$temp_dir/wrong-existing-ssr-anchor.out" >&2
  echo "notification-execution self-test: expected wrong-existing-SSR-anchor exit 1, got $wrong_ssr_status" >&2
  exit 1
fi
grep -q "tgt-frontend-ssr-ok: notification semantic anchor drifted" "$temp_dir/wrong-existing-ssr-anchor.out"

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/wrong-existing-ui-anchor.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.targetEvidence.find((item) => item.id === "tgt-user-ui-target-dto").anchor = "struct ServiceNotificationList {";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/wrong-existing-ui-anchor.json" >"$temp_dir/wrong-existing-ui-anchor.out" 2>&1
wrong_ui_status=$?
set -e
if [ "$wrong_ui_status" -ne 1 ]; then
  cat "$temp_dir/wrong-existing-ui-anchor.out" >&2
  echo "notification-execution self-test: expected wrong-existing-UI-anchor exit 1, got $wrong_ui_status" >&2
  exit 1
fi
grep -q "tgt-user-ui-target-dto: notification semantic anchor drifted" "$temp_dir/wrong-existing-ui-anchor.out"

assert_wrong_existing_target_anchor() {
  label=$1
  evidence_id=$2
  replacement_anchor=$3
  expected_error=$4
  tampered="$temp_dir/$label.json"
  output="$temp_dir/$label.out"
  NOTIFICATION_CONTRACT_IN="$contract" \
  NOTIFICATION_CONTRACT_OUT="$tampered" \
  NOTIFICATION_TAMPER_ID="$evidence_id" \
  NOTIFICATION_TAMPER_ANCHOR="$replacement_anchor" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
const item = contract.targetEvidence.find((candidate) => candidate.id === process.env.NOTIFICATION_TAMPER_ID);
if (!item) process.exit(2);
item.anchor = process.env.NOTIFICATION_TAMPER_ANCHOR;
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
  set +e
  "$verify" --mode integrity --contract "$tampered" >"$output" 2>&1
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    cat "$output" >&2
    echo "notification-execution self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "$expected_error" "$output"
}

assert_wrong_existing_target_anchor \
  wrong-existing-head-policy-anchor \
  tgt-frontend-get-only-list \
  'get(notification_unread_count)' \
  'tgt-frontend-get-only-list: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-query-anchor \
  tgt-frontend-query-contract \
  'const NOTIFICATION_LIST_OFFSET_MAX: u32 = 1_000_000;' \
  'tgt-frontend-query-contract: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-body-size-anchor \
  tgt-frontend-body-limits \
  'const NOTIFICATION_UNREAD_BODY_MAX: usize = 4 * 1024;' \
  'tgt-frontend-body-limits: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-owner-anchor \
  tgt-frontend-owner-cross-check \
  'struct NotificationListWire {' \
  'tgt-frontend-owner-cross-check: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-bearer-anchor \
  tgt-frontend-bearer-only \
  'let client = state.notification.clone_for_bearer();' \
  'tgt-frontend-bearer-only: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-unread-anchor \
  tgt-frontend-unread-contract \
  'struct NotificationListWire {' \
  'tgt-frontend-unread-contract: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-dormant-nav-anchor \
  tgt-dormant-nav-unavailable \
  'title="Notifications"' \
  'tgt-dormant-nav-unavailable: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-mount-anchor \
  tgt-active-header-mount \
  'epsx_templates::html_text_escape_pub(&meta.title)' \
  'tgt-active-header-mount: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-auth-anchor \
  tgt-active-header-auth-runtime \
  'let is_authenticated = user.is_some();' \
  'tgt-active-header-auth-runtime: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-offline-anchor \
  tgt-active-header-offline-exclusion \
  'if path == "/auth" {' \
  'tgt-active-header-offline-exclusion: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-endpoint-anchor \
  tgt-active-header-endpoint \
  '/api/v1/notification/list' \
  'tgt-active-header-endpoint: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-validation-anchor \
  tgt-active-header-exact-validation \
  'if (count === 0)' \
  'tgt-active-header-exact-validation: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-race-anchor \
  tgt-active-header-race-guard \
  'requestGeneration += 1;' \
  'tgt-active-header-race-guard: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-dom-anchor \
  tgt-active-header-initial-dom \
  'data-epsx-notification-badge-target="true"' \
  'tgt-active-header-initial-dom: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-accessibility-anchor \
  tgt-active-header-accessibility \
  "target.setAttribute('aria-label', 'Notifications');" \
  'tgt-active-header-accessibility: notification semantic anchor drifted'
assert_wrong_existing_target_anchor \
  wrong-existing-header-text-only-anchor \
  tgt-active-header-text-only \
  "badge.textContent = '';" \
  'tgt-active-header-text-only: notification semantic anchor drifted'

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/nav-blocker-removed.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.surfaceContracts.find((surface) => surface.id === "owner-list-and-count").targetObserved =
  "The active shared header consumes the unread count and renders a production-ready badge.";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/nav-blocker-removed.json" >"$temp_dir/nav-blocker-removed.out" 2>&1
nav_blocker_status=$?
set -e
if [ "$nav_blocker_status" -ne 1 ]; then
  cat "$temp_dir/nav-blocker-removed.out" >&2
  echo "notification-execution self-test: expected nav-blocker-removal exit 1, got $nav_blocker_status" >&2
  exit 1
fi
grep -q "owner-list-and-count target observation or shared-header residual blockers drifted" "$temp_dir/nav-blocker-removed.out"

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/stale-source.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.source.commit = "0000000000000000000000000000000000000000";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-source.json" >"$temp_dir/stale-source.out" 2>&1
stale_status=$?
set -e
if [ "$stale_status" -ne 1 ]; then
  cat "$temp_dir/stale-source.out" >&2
  echo "notification-execution self-test: expected stale-source exit 1, got $stale_status" >&2
  exit 1
fi
grep -q "stale source ref/commit" "$temp_dir/stale-source.out"

NOTIFICATION_CONTRACT_IN="$contract" NOTIFICATION_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const contract = await Bun.file(process.env.NOTIFICATION_CONTRACT_IN).json();
contract.targetEvidence[0].file = "../outside";
await Bun.write(process.env.NOTIFICATION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
if [ "$traversal_status" -ne 1 ]; then
  cat "$temp_dir/traversal.out" >&2
  echo "notification-execution self-test: expected traversal exit 1, got $traversal_status" >&2
  exit 1
fi
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

assert_refused_env() {
  env_name=$1
  env_value=$2
  output=$3
  set +e
  env "$env_name=$env_value" "$verify" --mode integrity >"$output" 2>&1
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    cat "$output" >&2
    echo "notification-execution self-test: expected $env_name refusal exit 1, got $status" >&2
    exit 1
  fi
  grep -q "$env_name" "$output"
}

assert_refused_env EPSX_ENV production "$temp_dir/production-env.out"
assert_refused_env NOTIFICATIONS_DATABASE_URL postgresql://local.invalid/db "$temp_dir/database-env.out"
assert_refused_env REDIS_URL redis://local.invalid/0 "$temp_dir/redis-env.out"
assert_refused_env SMTP_HOST smtp.invalid "$temp_dir/smtp-env.out"
assert_refused_env HTTPS_PROXY http://proxy.invalid "$temp_dir/network-env.out"

echo "notification-execution self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, source/A3.11/wrong-existing-SSR+UI+HEAD+query+size+owner+bearer+unread+dormant-nav+header-mount+auth+offline+endpoint+validation+race+DOM+a11y+text-only/residual-blocker/stale/traversal tamper=1, prod/db/redis/smtp/network env refusal=1)"
