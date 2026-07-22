#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-notification-execution.sh"
contract="$repo_root/docs/migration/contracts/notification-execution.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-notification-execution.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "14 source records, 36 target anchors, 12 surfaces, and 22 stop blockers" "$temp_dir/integrity.out"
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
if (report.source.evidence !== 14 || report.targetEvidence !== 36 || report.surfaces.length !== 12 || report.blockers.length !== 22) process.exit(1);
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

echo "notification-execution self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, source/A3.11-anchor/stale/traversal tamper=1, prod/db/redis/smtp/network env refusal=1)"
