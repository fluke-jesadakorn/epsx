#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
verify="$script_dir/verify-admin-live-data.sh"
contract="$repo_root/docs/migration/contracts/admin-live-data.json"
temp_dir=$(mktemp -d "${TMPDIR:-/tmp}/epsx-admin-live-data.XXXXXX")
trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM

assert_test_count() {
  output_file=$1
  expected=$2
  observed=$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed;.*/\1/p' "$output_file" | awk '{ total += $1 } END { print total + 0 }')
  if [ "$observed" -ne "$expected" ]; then
    cat "$output_file" >&2
    echo "admin-live-data self-test: expected $expected passing tests in $output_file, got $observed" >&2
    exit 1
  fi
}

(cd "$repo_root" && cargo test --offline --locked -p epsx-admin) >"$temp_dir/admin-rust.out" 2>&1
assert_test_count "$temp_dir/admin-rust.out" 124
(cd "$repo_root" && cargo test --offline --locked -p epsx-dioxus-ui wallet_wallets --lib) >"$temp_dir/wallet-ui-rust.out" 2>&1
assert_test_count "$temp_dir/wallet-ui-rust.out" 12
(cd "$repo_root" && cargo test --offline --locked -p epsx-dioxus-ui admin_pages::dashboard::tests --lib) >"$temp_dir/dashboard-ui-rust.out" 2>&1
assert_test_count "$temp_dir/dashboard-ui-rust.out" 6
(cd "$repo_root" && cargo test --offline --locked -p epsx dto_to_web_projection_preserves_large_counts_and_count_invariants) >"$temp_dir/wallet-backend-rust.out" 2>&1
assert_test_count "$temp_dir/wallet-backend-rust.out" 1
(cd "$repo_root" && cargo test --offline --locked -p epsx --lib dashboard_user_status --no-fail-fast) >"$temp_dir/dashboard-backend-rust.out" 2>&1
assert_test_count "$temp_dir/dashboard-backend-rust.out" 5
(cd "$repo_root" && cargo test --offline --locked -p epsx --lib exact_admin --no-fail-fast) >"$temp_dir/dashboard-audience-rust.out" 2>&1
assert_test_count "$temp_dir/dashboard-audience-rust.out" 2

"$verify" --mode integrity >"$temp_dir/integrity.out" 2>&1
grep -q "PASS integrity (27 source routes; 3 redirects; 2 aligned, 8 partial, 17 blocked; 20 STOP blockers" "$temp_dir/integrity.out"

set +e
"$verify" --mode readiness >"$temp_dir/readiness.out" 2>&1
readiness_status=$?
set -e
if [ "$readiness_status" -ne 3 ]; then
  cat "$temp_dir/readiness.out" >&2
  echo "admin-live-data self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "STOP readiness (25 non-aligned routes: 8 partial, 17 blocked; 20 cross-cutting blockers)" "$temp_dir/readiness.out"

"$verify" --mode emit >"$temp_dir/emit-one.json"
"$verify" --mode emit >"$temp_dir/emit-two.json"
cmp "$temp_dir/emit-one.json" "$temp_dir/emit-two.json"
bun -e 'const report = await Bun.file(process.argv[1]).json(); if (report.routeCount !== 27 || report.redirectCount !== 3 || report.stopBlockerCount !== 20 || report.productionReady !== false || report.readinessExit !== 3 || report.statuses.aligned !== 2 || report.statuses.partial !== 8 || report.statuses.blocked !== 17) process.exit(1);' "$temp_dir/emit-one.json"

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

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-wallet-adapter.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
const evidence = value.targetEvidence.find((item) => item.file === "apps/admin/src/wallet_stats_adapter.rs");
if (!evidence) process.exit(2);
evidence.anchors[0] = "tampered wallet adapter route anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-wallet-adapter.json" >"$temp_dir/stale-wallet-adapter.out" 2>&1
wallet_adapter_status=$?
set -e
[ "$wallet_adapter_status" -eq 1 ] || { cat "$temp_dir/stale-wallet-adapter.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-wallet-adapter.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-dashboard-adapter.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
const evidence = value.targetEvidence.find((item) => item.file === "apps/admin/src/dashboard_user_status_adapter.rs");
if (!evidence) process.exit(2);
evidence.anchors[0] = "tampered dashboard adapter route anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-dashboard-adapter.json" >"$temp_dir/stale-dashboard-adapter.out" 2>&1
dashboard_adapter_status=$?
set -e
[ "$dashboard_adapter_status" -eq 1 ] || { cat "$temp_dir/stale-dashboard-adapter.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-dashboard-adapter.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-dashboard-audience.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
const evidence = value.targetEvidence.find((item) => item.file === "apps/backend/src/web/middleware/bearer_middleware.rs");
if (!evidence) process.exit(2);
evidence.anchors[1] = "tampered dashboard audience guard anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-dashboard-audience.json" >"$temp_dir/stale-dashboard-audience.out" 2>&1
dashboard_audience_status=$?
set -e
[ "$dashboard_audience_status" -eq 1 ] || { cat "$temp_dir/stale-dashboard-audience.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-dashboard-audience.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-dashboard-route.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
const evidence = value.targetEvidence.find((item) => item.file === "apps/backend/src/web/admin/routes.rs");
if (!evidence) process.exit(2);
evidence.anchors[3] = "tampered dashboard production route guard anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-dashboard-route.json" >"$temp_dir/stale-dashboard-route.out" 2>&1
dashboard_route_status=$?
set -e
[ "$dashboard_route_status" -eq 1 ] || { cat "$temp_dir/stale-dashboard-route.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-dashboard-route.out"

ADMIN_CONTRACT_IN="$contract" ADMIN_CONTRACT_OUT="$temp_dir/stale-wallet-ssr.json" bun -e '
const value = await Bun.file(process.env.ADMIN_CONTRACT_IN).json();
const evidence = value.targetEvidence.find((item) => item.file === "apps/admin/src/ssr.rs");
if (!evidence) process.exit(2);
const index = evidence.anchors.indexOf("load_admin_wallet_stats(");
if (index < 0) process.exit(2);
evidence.anchors[index] = "tampered wallet SSR loader anchor";
await Bun.write(process.env.ADMIN_CONTRACT_OUT, `${JSON.stringify(value, null, 2)}\n`);
'
set +e
"$verify" --mode integrity --fixture "$temp_dir/stale-wallet-ssr.json" >"$temp_dir/stale-wallet-ssr.out" 2>&1
wallet_ssr_status=$?
set -e
[ "$wallet_ssr_status" -eq 1 ] || { cat "$temp_dir/stale-wallet-ssr.out" >&2; exit 1; }
grep -q "missing target anchor" "$temp_dir/stale-wallet-ssr.out"

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

echo "admin-live-data self-test: PASS (Rust admin/dashboard+wallet UI/backend exact counts, integrity=0, readiness-stop=3, deterministic emit, tamper/path/stale-target/dashboard-adapter/dashboard-audience/dashboard-route/wallet-adapter/wallet-SSR/stale-source/redirect-set/redirect-semantics=1)"
