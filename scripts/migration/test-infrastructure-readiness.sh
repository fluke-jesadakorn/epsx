#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-infrastructure-readiness.sh"
contract="$repo_root/docs/migration/contracts/infrastructure-readiness.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-a13-self-test.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "18 stop blockers" "$temp_dir/integrity.out"
grep -q "no cluster, secrets, deployment" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_code=$?
set -e
if [ "$readiness_code" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "infrastructure-readiness self-test: expected readiness exit 3, got $readiness_code" >&2
  exit 1
fi
grep -q "P0 ledger is 1 passed, 4 partial, 2 blocked" "$temp_dir/readiness.out"

"$verify" --mode report >"$temp_dir/report-one.json"
"$verify" --mode report >"$temp_dir/report-two.json"
cmp "$temp_dir/report-one.json" "$temp_dir/report-two.json"
bun -e '
const report = JSON.parse(await Bun.file(process.argv[1]).text());
if (report.resources.total !== 15 || report.images.devOccurrences !== 3 || report.images.digestOccurrences !== 0 || report.nodePorts.length !== 6 || report.blockers.length !== 18 || JSON.stringify(report.p0StatusCounts) !== JSON.stringify({ passed: 1, partial: 4, blocked: 2 }) || report.productionReady !== false || report.clusterAccess !== false || report.readinessExit !== 3) process.exit(1);
' "$temp_dir/report-one.json"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/stale-anchor.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.evidence[0].anchors[0] = "tampered missing infrastructure anchor";
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/stale-anchor.json" >"$temp_dir/stale-anchor.out" 2>&1
anchor_code=$?
set -e
if [ "$anchor_code" -ne 1 ]; then
  cat "$temp_dir/stale-anchor.out" >&2
  echo "infrastructure-readiness self-test: expected stale-anchor exit 1, got $anchor_code" >&2
  exit 1
fi
grep -q "missing evidence anchor" "$temp_dir/stale-anchor.out"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/path-traversal.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.evidence[0].file = "../outside";
delete contract.evidence[0].sha256;
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/path-traversal.json" >"$temp_dir/path-traversal.out" 2>&1
path_code=$?
set -e
if [ "$path_code" -ne 1 ]; then
  cat "$temp_dir/path-traversal.out" >&2
  echo "infrastructure-readiness self-test: expected traversal exit 1, got $path_code" >&2
  exit 1
fi
grep -q "unsafe evidence path" "$temp_dir/path-traversal.out"

A13_CONTRACT_IN="$contract" A13_CONTRACT_OUT="$temp_dir/render-drift.json" bun -e '
const contract = await Bun.file(process.env.A13_CONTRACT_IN).json();
contract.renderExpected.deployments[0].images = ["epsx-admin:prod"];
await Bun.write(process.env.A13_CONTRACT_OUT, `${JSON.stringify(contract, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --contract "$temp_dir/render-drift.json" >"$temp_dir/render-drift.out" 2>&1
render_code=$?
set -e
if [ "$render_code" -ne 1 ]; then
  cat "$temp_dir/render-drift.out" >&2
  echo "infrastructure-readiness self-test: expected render-drift exit 1, got $render_code" >&2
  exit 1
fi
grep -q "rendered deployments drift" "$temp_dir/render-drift.out"

echo "infrastructure-readiness self-test: PASS (integrity=0, readiness-stop=3, deterministic=stable, anchor/path/render tamper=1)"
