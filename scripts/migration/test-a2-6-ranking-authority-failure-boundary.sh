#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-6-ranking-authority-failure-boundary.sh"
CONTRACT_REL="docs/migration/contracts/a2-6-ranking-authority-failure-boundary.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-6-authority.XXXXXX")"
trap 'rm -rf -- "$TEMP_ROOT"' EXIT HUP INT TERM

copy_fixture() {
  local destination="$1"
  mkdir -p "$destination/docs/migration/contracts"
  cp "$REPO_ROOT/$CONTRACT_REL" "$destination/$CONTRACT_REL"
  while IFS= read -r file; do
    mkdir -p "$destination/$(dirname "$file")"
    cp "$REPO_ROOT/$file" "$destination/$file"
  done < <(bun -e '
const contract = await Bun.file(process.argv[1]).json();
for (const item of contract.implementationEvidence) console.log(item.file);
' "$REPO_ROOT/$CONTRACT_REL")
}

verify_fixture() {
  local root="$1"
  local mode="$2"
  "$VERIFY" --mode "$mode" --evidence-root "$root" --contract "$root/$CONTRACT_REL" --static-only
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
    echo "ranking-authority-failure-boundary-self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "ranking-authority-failure-boundary: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "10 invariants; 10 hermetic tests; 3 implementation digests; 8 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.invariants !== 10 || report.hermeticTests !== 10 || report.implementationEvidence !== 3 || report.residualStops.length !== 8) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_status=$?
set -e
if [[ "$readiness_status" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "ranking-authority-failure-boundary-self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "8 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure live-env env IDENTITY_GRPC_URL=http://example.invalid "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

grep -Fq 'git("rev-parse", `${expectedTarget.commit}^{commit}`)' "$VERIFY"
if grep -Fq 'git("rev-parse", `${expectedTarget.ref}^{commit}`)' "$VERIFY"; then
  echo "ranking-authority-failure-boundary-self-test: target base must remain valid after the branch advances" >&2
  exit 1
fi

mutate_contract() {
  local label="$1"
  local expression="$2"
  local root="$TEMP_ROOT/$label"
  copy_fixture "$root"
  CONTRACT_IN="$root/$CONTRACT_REL" EXPRESSION="$expression" bun -e '
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
mutate_contract source-tuple 'contract.sourceBaseline.evidence[0].anchor = "invented";'
mutate_contract invariant-inventory 'contract.invariants[0].id = "renamed";'
mutate_contract test-inventory 'contract.hermeticTests.pop();'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
mutate_contract execution-order 'contract.requiredExecutionOrder[0] = "E08 deploy first";'
mutate_contract evidence-traversal 'contract.implementationEvidence[0].file = "../outside";'
mutate_contract digest-tuple 'contract.implementationEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'

TAMPER_ROOT="$TEMP_ROOT/implementation-tamper"
copy_fixture "$TAMPER_ROOT"
printf '\n// mutation\n' >>"$TAMPER_ROOT/apps/analytics/src/grpc_client.rs"
expect_failure implementation-tamper verify_fixture "$TAMPER_ROOT" integrity

SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/apps/analytics/src/grpc_client.rs" "$TEMP_ROOT/outside-grpc-client.rs"
ln -s "$TEMP_ROOT/outside-grpc-client.rs" "$SYMLINK_ROOT/apps/analytics/src/grpc_client.rs"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "ranking-authority-failure-boundary-self-test: PASS 16/16 environment, static-mode, sentinel, immutable-base, pin, tuple, inventory, order, path, digest, tamper, and symlink cases"
