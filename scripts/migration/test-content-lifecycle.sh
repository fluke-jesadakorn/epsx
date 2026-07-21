#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-content-lifecycle.sh"
contract="$repo_root/docs/migration/contracts/content-lifecycle.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-content-lifecycle.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

expect_integrity_failure() {
  fixture=$1
  expected=$2
  label=$3
  set +e
  "$verify" --mode integrity --contract "$fixture" >"$temp_dir/$label.out" 2>&1
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    sed -n '1,160p' "$temp_dir/$label.out" >&2
    echo "content-lifecycle self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "$expected" "$temp_dir/$label.out"
}

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "20 stop blockers, 8 route batches" "$temp_dir/integrity.out"
grep -q "authorization and A3.10 schema are partial" "$temp_dir/integrity.out"
grep -q "four content runtime DDL findings are removed" "$temp_dir/integrity.out"
grep -q "lifecycle parity, and production readiness remain absent" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  sed -n '1,160p' "$temp_dir/readiness.out" >&2
  echo "content-lifecycle self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "20 stop blockers remain across 8 route batches" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.readinessExit !== 3 || report.productionReady !== false || report.lifecycleParity !== false) process.exit(1);
if (report.blockers.length !== 20 || report.routeBatches.length !== 8 || report.lifecycleRequirements.length !== 16) process.exit(1);
if (report.currentBoundary.authorization !== "partial" || report.currentBoundary.editorRoutes !== "fail-closed-404") process.exit(1);
if (report.currentBoundary.contentSchemaBoundary !== "partial-a3.10" || report.currentBoundary.contentRuntimeDdlFindingsRemoved !== 4 || report.currentBoundary.contentRuntimeDdlFindingsRemaining !== 0) process.exit(1);
if (report.currentBoundary.contentMigrationRunner !== "absent" || report.currentBoundary.contentPopulatedUpgradeProof !== false) process.exit(1);
' "$temp_dir/report-one.json"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/missing-source-anchor.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.source.evidence[0].anchor = "tampered missing pinned anchor";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/missing-source-anchor.json" "missing source anchor" "missing-source-anchor"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/missing-target-anchor.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.targetEvidence[0].anchor = "tampered missing current anchor";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/missing-target-anchor.json" "missing target anchor" "missing-target-anchor"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/stale-source.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.source.commit = "0000000000000000000000000000000000000000";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/stale-source.json" "stale source ref/commit" "stale-source"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/stale-blob.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.source.evidence[0].blob = "0000000000000000000000000000000000000000";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/stale-blob.json" "stale source blob" "stale-blob"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/path-traversal.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.targetEvidence[0].file = "../outside";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/path-traversal.json" "unsafe evidence path" "path-traversal"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/readiness-tamper.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.productionReady = true;
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/readiness-tamper.json" "readiness sentinel changed" "readiness-tamper"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/schema-boundary-tamper.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.currentBoundary.contentMigrationRunner = "wired";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/schema-boundary-tamper.json" "A2.3b/A3.10 boundary facts drifted" "schema-boundary-tamper"

CONTENT_CONTRACT_IN="$contract" CONTENT_CONTRACT_OUT="$temp_dir/blocker-tamper.json" bun -e '
const contract = await Bun.file(process.env.CONTENT_CONTRACT_IN).json();
contract.blockers[0].status = "ready";
await Bun.write(process.env.CONTENT_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_integrity_failure "$temp_dir/blocker-tamper.json" "stop blocker state changed" "blocker-tamper"

echo "content-lifecycle self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, source/target-anchor+stale-ref/blob+path+sentinel/schema-boundary/blocker tamper=1)"
