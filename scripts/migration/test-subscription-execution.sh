#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-subscription-execution.sh"
contract="$repo_root/docs/migration/contracts/subscription-execution.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-subscription-execution.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "20 stop blockers" "$temp_dir/integrity.out"
grep -q "no database, chain, deployment, migration" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "subscription-execution self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "20 stop blockers remain" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e 'const report = JSON.parse(await Bun.file(process.argv[1]).text()); if (report.readinessExit !== 3 || report.productionReady !== false || report.blockers.length !== 20 || report.routeContracts.length !== 12) process.exit(1);' "$temp_dir/report-one.json"

SUBSCRIPTION_CONTRACT_IN="$contract" SUBSCRIPTION_CONTRACT_OUT="$temp_dir/missing-source-anchor.json" bun -e '
const contract = await Bun.file(process.env.SUBSCRIPTION_CONTRACT_IN).json();
contract.source.evidence[0].anchor = "tampered missing source anchor";
await Bun.write(process.env.SUBSCRIPTION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/missing-source-anchor.json" >"$temp_dir/missing-source-anchor.out" 2>&1
source_anchor_status=$?
set -e
if [ "$source_anchor_status" -ne 1 ]; then
  cat "$temp_dir/missing-source-anchor.out" >&2
  echo "subscription-execution self-test: expected source-anchor tamper exit 1, got $source_anchor_status" >&2
  exit 1
fi
grep -q "missing source anchor" "$temp_dir/missing-source-anchor.out"

SUBSCRIPTION_CONTRACT_IN="$contract" SUBSCRIPTION_CONTRACT_OUT="$temp_dir/stale-source.json" bun -e '
const contract = await Bun.file(process.env.SUBSCRIPTION_CONTRACT_IN).json();
contract.source.commit = "0000000000000000000000000000000000000000";
await Bun.write(process.env.SUBSCRIPTION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-source.json" >"$temp_dir/stale-source.out" 2>&1
stale_status=$?
set -e
if [ "$stale_status" -ne 1 ]; then
  cat "$temp_dir/stale-source.out" >&2
  echo "subscription-execution self-test: expected stale-source exit 1, got $stale_status" >&2
  exit 1
fi
grep -q "stale source ref/commit" "$temp_dir/stale-source.out"

SUBSCRIPTION_CONTRACT_IN="$contract" SUBSCRIPTION_CONTRACT_OUT="$temp_dir/missing-target-anchor.json" bun -e '
const contract = await Bun.file(process.env.SUBSCRIPTION_CONTRACT_IN).json();
contract.targetEvidence[0].anchor = "tampered missing target anchor";
await Bun.write(process.env.SUBSCRIPTION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/missing-target-anchor.json" >"$temp_dir/missing-target-anchor.out" 2>&1
target_anchor_status=$?
set -e
if [ "$target_anchor_status" -ne 1 ]; then
  cat "$temp_dir/missing-target-anchor.out" >&2
  echo "subscription-execution self-test: expected target-anchor tamper exit 1, got $target_anchor_status" >&2
  exit 1
fi
grep -q "missing target anchor" "$temp_dir/missing-target-anchor.out"

SUBSCRIPTION_CONTRACT_IN="$contract" SUBSCRIPTION_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const contract = await Bun.file(process.env.SUBSCRIPTION_CONTRACT_IN).json();
contract.targetEvidence[0].file = "../outside";
await Bun.write(process.env.SUBSCRIPTION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
if [ "$traversal_status" -ne 1 ]; then
  cat "$temp_dir/traversal.out" >&2
  echo "subscription-execution self-test: expected traversal exit 1, got $traversal_status" >&2
  exit 1
fi
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

SUBSCRIPTION_CONTRACT_IN="$contract" SUBSCRIPTION_CONTRACT_OUT="$temp_dir/readiness-tamper.json" bun -e '
const contract = await Bun.file(process.env.SUBSCRIPTION_CONTRACT_IN).json();
contract.productionReady = true;
await Bun.write(process.env.SUBSCRIPTION_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/readiness-tamper.json" >"$temp_dir/readiness-tamper.out" 2>&1
readiness_tamper_status=$?
set -e
if [ "$readiness_tamper_status" -ne 1 ]; then
  cat "$temp_dir/readiness-tamper.out" >&2
  echo "subscription-execution self-test: expected readiness-tamper exit 1, got $readiness_tamper_status" >&2
  exit 1
fi
grep -q "readiness sentinel changed" "$temp_dir/readiness-tamper.out"

echo "subscription-execution self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, source/target-anchor/stale/path/readiness tamper=1)"
