#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-frontend-live-data.sh"
contract="$repo_root/docs/migration/contracts/frontend-live-data.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-frontend-live-data.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "PASS integrity (28 routes; 3 aligned, 7 partial, 18 blocked" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "frontend-live-data self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "STOP readiness (25 non-aligned routes" "$temp_dir/readiness.out"

"$verify" --mode emit >"$temp_dir/emit-one.json"
"$verify" --mode emit >"$temp_dir/emit-two.json"
cmp "$temp_dir/emit-one.json" "$temp_dir/emit-two.json"
bun -e 'const report = await Bun.file(process.argv[1]).json(); if (report.routeCount !== 28 || report.productionReady !== false || report.readinessExit !== 3 || report.statuses.aligned !== 3 || report.statuses.partial !== 7 || report.statuses.blocked !== 18) process.exit(1);' "$temp_dir/emit-one.json"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/tampered.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes[0].path = "/tampered";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/tampered.json" >"$temp_dir/tampered.out" 2>&1
tampered_status=$?
set -e
[ "$tampered_status" -eq 1 ] || { cat "$temp_dir/tampered.out" >&2; exit 1; }
grep -q "not in the checked frontend route inventory" "$temp_dir/tampered.out"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/traversal.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes[0].target.file = "../outside.rs";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/traversal.json" >"$temp_dir/traversal.out" 2>&1
traversal_status=$?
set -e
[ "$traversal_status" -eq 1 ] || { cat "$temp_dir/traversal.out" >&2; exit 1; }
grep -q "unsafe evidence path" "$temp_dir/traversal.out"

FRONTEND_CONTRACT_IN="$contract" FRONTEND_CONTRACT_OUT="$temp_dir/stale-anchor.json" bun -e '
const value = await Bun.file(process.env.FRONTEND_CONTRACT_IN).json();
value.routes[0].target.anchors[0] = "tampered stale target anchor";
await Bun.write(process.env.FRONTEND_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-anchor.json" >"$temp_dir/stale-anchor.out" 2>&1
anchor_status=$?
set -e
[ "$anchor_status" -eq 1 ] || { cat "$temp_dir/stale-anchor.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-anchor.out"

echo "frontend-live-data self-test: PASS (integrity=0, readiness-stop=3, deterministic emit, tamper/path/stale-anchor=1)"
