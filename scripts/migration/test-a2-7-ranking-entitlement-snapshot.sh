#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-7-ranking-entitlement-snapshot.sh"
CONTRACT_REL="docs/migration/contracts/a2-7-ranking-entitlement-snapshot.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-7-entitlement.XXXXXX")"
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
console.log(contract.fixtureEvidence.file);
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
    echo "ranking-entitlement-snapshot-self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "ranking-entitlement-snapshot: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "12 invariants; 30 fixtures; 7 hermetic tests; 5 frozen digests; 12 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.invariants !== 12 || report.fixtures !== 30 || report.hermeticTests !== 7 || report.implementationEvidence !== 4 || report.residualStops.length !== 12) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_status=$?
set -e
if [[ "$readiness_status" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "ranking-entitlement-snapshot-self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "12 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure live-env env IDENTITY_DATABASE_URL=postgres://example.invalid/identity "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

grep -Fq 'git("rev-parse", `${expectedTarget.commit}^{commit}`)' "$VERIFY"
if grep -Fq 'git("rev-parse", `${expectedTarget.ref}^{commit}`)' "$VERIFY"; then
  echo "ranking-entitlement-snapshot-self-test: target base must remain valid after the branch advances" >&2
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
mutate_contract fixture-digest 'contract.fixtureEvidence.sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract fixture-inventory 'contract.fixtureEvidence.fixtureIds[0] = "renamed";'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
mutate_contract execution-order 'contract.requiredExecutionOrder[0] = "E09 deploy first";'
mutate_contract fixture-traversal 'contract.fixtureEvidence.file = "../outside";'
mutate_contract implementation-traversal 'contract.implementationEvidence[0].file = "../outside";'
mutate_contract implementation-digest 'contract.implementationEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'

FIXTURE_TAMPER_ROOT="$TEMP_ROOT/fixture-tamper"
copy_fixture "$FIXTURE_TAMPER_ROOT"
fixture_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.fixtureEvidence.file);' "$FIXTURE_TAMPER_ROOT/$CONTRACT_REL")"
printf '\n' >>"$FIXTURE_TAMPER_ROOT/$fixture_rel"
expect_failure fixture-tamper verify_fixture "$FIXTURE_TAMPER_ROOT" integrity

IMPLEMENTATION_TAMPER_ROOT="$TEMP_ROOT/implementation-tamper"
copy_fixture "$IMPLEMENTATION_TAMPER_ROOT"
implementation_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.implementationEvidence[0].file);' "$IMPLEMENTATION_TAMPER_ROOT/$CONTRACT_REL")"
printf '\n// mutation\n' >>"$IMPLEMENTATION_TAMPER_ROOT/$implementation_rel"
expect_failure implementation-tamper verify_fixture "$IMPLEMENTATION_TAMPER_ROOT" integrity

SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/$implementation_rel" "$TEMP_ROOT/outside-implementation.rs"
ln -s "$TEMP_ROOT/outside-implementation.rs" "$SYMLINK_ROOT/$implementation_rel"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "ranking-entitlement-snapshot-self-test: PASS environment, static-mode, sentinel, immutable-base, source/target, invariant, test, fixture, STOP, order, path, digest, tamper, and symlink cases"
