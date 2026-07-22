#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-admin-live-data.sh"
contract="$repo_root/docs/migration/contracts/admin-live-data.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-admin-live-data.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "PASS integrity (27 source routes; 3 redirects; 2 aligned, 6 partial, 19 blocked; 20 STOP blockers" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "admin-live-data self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "STOP readiness (25 non-aligned routes: 6 partial, 19 blocked; 20 cross-cutting blockers)" "$temp_dir/readiness.out"

"$verify" --mode emit >"$temp_dir/emit-one.json"
"$verify" --mode emit >"$temp_dir/emit-two.json"
cmp "$temp_dir/emit-one.json" "$temp_dir/emit-two.json"
bun -e 'const report = await Bun.file(process.argv[1]).json(); if (report.routeCount !== 27 || report.redirectCount !== 3 || report.stopBlockerCount !== 20 || report.productionReady !== false || report.readinessExit !== 3 || report.statuses.aligned !== 2 || report.statuses.partial !== 6 || report.statuses.blocked !== 19) process.exit(1);' "$temp_dir/emit-one.json"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/tampered.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
value.routes[0].status = "aligned";
value.routes[0].blockers = [];
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/tampered.json" >"$temp_dir/tampered.out" 2>&1
tampered_status=$?
set -e
[ "$tampered_status" -eq 1 ] || { cat "$temp_dir/tampered.out" >&2; exit 1; }
grep -q "baseline status count must remain conservative" "$temp_dir/tampered.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
value.routes[0].target.file = "../outside.rs";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
[ "$traversal_status" -eq 1 ] || { cat "$temp_dir/traversal.out" >&2; exit 1; }
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-target.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
value.targetEvidence[0].anchors[0] = "tampered stale target anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-target.json" >"$temp_dir/stale-target.out" 2>&1
target_status=$?
set -e
[ "$target_status" -eq 1 ] || { cat "$temp_dir/stale-target.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-target.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-source.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
value.routes[0].source.anchors[0] = "tampered stale source anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-source.json" >"$temp_dir/stale-source.out" 2>&1
source_status=$?
set -e
[ "$source_status" -eq 1 ] || { cat "$temp_dir/stale-source.out" >&2; exit 1; }
grep -q "missing source anchor" "$temp_dir/stale-source.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/redirect-tamper.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
value.redirects[0].target = "/tampered";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/redirect-tamper.json" >"$temp_dir/redirect-tamper.out" 2>&1
redirect_status=$?
set -e
[ "$redirect_status" -eq 1 ] || { cat "$temp_dir/redirect-tamper.out" >&2; exit 1; }
grep -q "redirect set must equal" "$temp_dir/redirect-tamper.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/redirect-semantics-tamper.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
value.redirects[0].proofGaps = [];
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/redirect-semantics-tamper.json" >"$temp_dir/redirect-semantics-tamper.out" 2>&1
redirect_semantics_status=$?
set -e
[ "$redirect_semantics_status" -eq 1 ] || { cat "$temp_dir/redirect-semantics-tamper.out" >&2; exit 1; }
grep -q "must retain the exact three redirect proof gaps" "$temp_dir/redirect-semantics-tamper.out"

echo "admin-live-data self-test: PASS (integrity=0, readiness-stop=3, deterministic emit, tamper/path/stale-target/stale-source/redirect-set/redirect-semantics=1)"
