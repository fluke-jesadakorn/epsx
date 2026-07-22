#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-4-market-analytics-authorization.sh"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-4-self-test.XXXXXX")"
trap 'rm -rf -- "$TEMP_ROOT"' EXIT HUP INT TERM

copy_fixture() {
  local destination="$1"
  mkdir -p \
    "$destination/apps/analytics/src" \
    "$destination/apps/backend/src/web/analytics/eps" \
    "$destination/docs/migration/contracts"
  cp "$REPO_ROOT/apps/analytics/Cargo.toml" "$destination/apps/analytics/Cargo.toml"
  cp "$REPO_ROOT/apps/analytics/src/auth.rs" "$destination/apps/analytics/src/auth.rs"
  cp "$REPO_ROOT/apps/analytics/src/main.rs" "$destination/apps/analytics/src/main.rs"
  cp "$REPO_ROOT/apps/backend/src/web/analytics/eps/cache.rs" "$destination/apps/backend/src/web/analytics/eps/cache.rs"
  cp "$REPO_ROOT/docs/migration/contracts/a2-4-market-analytics-authorization.json" "$destination/docs/migration/contracts/a2-4-market-analytics-authorization.json"
}

verify_fixture() {
  local root="$1"
  local mode="$2"
  "$VERIFY" --mode "$mode" --evidence-root "$root" --contract "$root/docs/migration/contracts/a2-4-market-analytics-authorization.json" --static-only
}

expect_failure() {
  local label="$1"
  shift
  local output="$TEMP_ROOT/$label.out"
  set +e
  "$@" >"$output" 2>&1
  local status=$?
  set -e
  if [[ "$status" -ne 1 ]]; then
    cat "$output" >&2
    echo "market-analytics-authorization-self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "market-analytics-authorization: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "6 routes; 11 invariants; 10 hermetic tests; 12 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.routes.length !== 6 || report.invariants !== 11 || report.hermeticTests !== 10 || report.implementationEvidence !== 4 || report.residualStops.length !== 12) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_status=$?
set -e
if [[ "$readiness_status" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "market-analytics-authorization-self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "12 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/docs/migration/contracts/a2-4-market-analytics-authorization.json" --static-only
expect_failure live-env env TRADINGVIEW_URL=https://example.invalid "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/docs/migration/contracts/a2-4-market-analytics-authorization.json" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

mutate_contract() {
  local label="$1"
  local expression="$2"
  local root="$TEMP_ROOT/$label"
  copy_fixture "$root"
  CONTRACT_IN="$root/docs/migration/contracts/a2-4-market-analytics-authorization.json" EXPRESSION="$expression" bun -e '
const path = process.env.CONTRACT_IN;
const contract = await Bun.file(path).json();
const mutate = new Function("contract", process.env.EXPRESSION);
mutate(contract);
await Bun.write(path, `${JSON.stringify(contract, null, 2)}\n`);
'
  expect_failure "$label" verify_fixture "$root" integrity
}

mutate_contract readiness-flip 'contract.productionReady = true;'
mutate_contract source-pin 'contract.sourceBaseline.commit = "0000000000000000000000000000000000000000";'
mutate_contract target-pin 'contract.targetBase.commit = "0000000000000000000000000000000000000000";'
mutate_contract route-policy 'contract.routes[1].policy = "public-credential-omitting";'
mutate_contract test-inventory 'contract.hermeticTests.pop();'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed-stop";'
mutate_contract execution-order 'contract.requiredExecutionOrder[0] = "E08 deploy first";'
mutate_contract evidence-traversal 'contract.implementationEvidence[0].file = "../outside";'
mutate_contract invariant-meaning 'contract.invariants[0].claim = "deployment authorized";'
mutate_contract stop-meaning 'contract.residualStops[0].claim = "production ready";'
mutate_contract execution-meaning 'contract.requiredExecutionOrder[0] = "E01 deploy first";'
mutate_contract source-tuple 'contract.sourceBaseline.evidence[0].file = contract.sourceBaseline.evidence[1].file; contract.sourceBaseline.evidence[0].blob = contract.sourceBaseline.evidence[1].blob; contract.sourceBaseline.evidence[0].anchor = contract.sourceBaseline.evidence[1].anchor;'

for label in auth-digest router-digest wallet-digest; do
  root="$TEMP_ROOT/$label"
  copy_fixture "$root"
  case "$label" in
    auth-digest) printf '\n// mutation\n' >>"$root/apps/analytics/src/auth.rs" ;;
    router-digest) printf '\n// mutation\n' >>"$root/apps/analytics/src/main.rs" ;;
    wallet-digest) printf '\n// mutation\n' >>"$root/apps/backend/src/web/analytics/eps/cache.rs" ;;
  esac
  expect_failure "$label" verify_fixture "$root" integrity
done

SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/apps/analytics/src/auth.rs" "$TEMP_ROOT/outside-auth.rs"
ln -s "$TEMP_ROOT/outside-auth.rs" "$SYMLINK_ROOT/apps/analytics/src/auth.rs"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "market-analytics-authorization-self-test: PASS 19/19 environment, static-mode, sentinel, meaning, inventory, pin, path, symlink, and implementation-tamper cases"
