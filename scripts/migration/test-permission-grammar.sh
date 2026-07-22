#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../.." && pwd)
verify="$script_dir/verify-permission-grammar.sh"
fixture="$repo_root/docs/migration/contracts/permission-grammar.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-permission-grammar.XXXXXX")
trap 'rm -rf "$temp_dir"' EXIT HUP INT TERM

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "permission-grammar self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "10 security-gate blockers" "$temp_dir/readiness.out"
grep -q "presentation-drift=2" "$temp_dir/readiness.out"

bun "$script_dir/verify-permission-grammar.ts" --emit-inventory >"$temp_dir/scan-one.json"
bun "$script_dir/verify-permission-grammar.ts" --emit-inventory >"$temp_dir/scan-two.json"
cmp "$temp_dir/scan-one.json" "$temp_dir/scan-two.json"
bun -e '
const scan = await Bun.file(process.argv[1]).json();
if (scan.summary.total !== 65 || scan.summary.sourceCounts["dioxus-security-gate"] !== 31 || scan.summary.classificationCounts["legacy-2-segment"] !== 10) process.exit(1);
if (scan.inventory.some((item) => item.file === "shared/rust/dioxus_ui/src/pages/notifications.rs" || item.permission === "notifications:read" || item.surface === "frontend:notifications")) process.exit(1);
' "$temp_dir/scan-one.json"

cp "$fixture" "$temp_dir/tampered.json"
EPSX_PERMISSION_TAMPER_FIXTURE="$temp_dir/tampered.json" bun -e '
const path = process.env.EPSX_PERMISSION_TAMPER_FIXTURE;
if (!path) throw new Error("missing tamper fixture path");
const fixture = await Bun.file(path).json();
fixture.inventory[0].line += 1;
await Bun.write(path, `${JSON.stringify(fixture, null, 2)}\n`);
'

set +e
"$verify" --mode integrity --fixture "$temp_dir/tampered.json" >"$temp_dir/tamper.out" 2>&1
tamper_status=$?
set -e
if [ "$tamper_status" -ne 1 ]; then
  cat "$temp_dir/tamper.out" >&2
  echo "permission-grammar self-test: expected tamper exit 1, got $tamper_status" >&2
  exit 1
fi
grep -q "inventory" "$temp_dir/tamper.out"

echo "permission-grammar self-test: PASS (65 records, 10 security-gate blockers, integrity=0, readiness-stop=3, deterministic=stable, tamper=1)"
