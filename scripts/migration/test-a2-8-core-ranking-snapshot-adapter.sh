#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-8-core-ranking-snapshot-adapter.sh"
CONTRACT_REL="docs/migration/contracts/a2-8-core-ranking-snapshot-adapter.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-8-core-snapshot.XXXXXX")"
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
    echo "core-ranking-snapshot-adapter-self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "core-ranking-snapshot-adapter: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "12 invariants; 21 fixtures; 12 hermetic tests; 9 frozen digests; 14 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.invariants !== 12 || report.fixtures !== 21 || report.hermeticTests !== 12 || report.implementationEvidence !== 7 || report.residualStops.length !== 14 || !/^[0-9a-f]{64}$/.test(report.sqlDigest)) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_status=$?
set -e
if [[ "$readiness_status" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "core-ranking-snapshot-adapter-self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "14 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure database-env env DATABASE_URL=postgres://example.invalid/epsx "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure network-env env ALLOW_NETWORK=true "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

grep -Fq 'git("rev-parse", `${expectedTarget.commit}^{commit}`)' "$VERIFY"
if grep -Fq 'git("rev-parse", `${expectedTarget.ref}^{commit}`)' "$VERIFY"; then
  echo "core-ranking-snapshot-adapter-self-test: target base must remain valid after the branch advances" >&2
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

mutate_contract readiness-sentinel 'contract.productionReady = true;'
mutate_contract source-pin 'contract.sourceBaseline.commit = "0000000000000000000000000000000000000000";'
mutate_contract target-pin 'contract.targetBase.commit = "0000000000000000000000000000000000000000";'
mutate_contract invariant-inventory 'contract.invariants[0].id = "renamed";'
mutate_contract test-inventory 'contract.hermeticTests.pop();'
mutate_contract fixture-hash 'contract.fixtureEvidence.sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract implementation-hash 'contract.implementationEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract sql-hash 'contract.sqlEvidence.sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract sql-inventory 'contract.sqlEvidence.qualifiedTables[3] = "permissions";'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
mutate_contract execution-order 'contract.requiredExecutionOrder[0] = "E10 deploy first";'
mutate_contract fixture-traversal 'contract.fixtureEvidence.file = "../outside";'
mutate_contract implementation-traversal 'contract.implementationEvidence[0].file = "../outside";'

FIXTURE_TAMPER_ROOT="$TEMP_ROOT/fixture-tamper"
copy_fixture "$FIXTURE_TAMPER_ROOT"
fixture_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.fixtureEvidence.file);' "$FIXTURE_TAMPER_ROOT/$CONTRACT_REL")"
printf '\n' >>"$FIXTURE_TAMPER_ROOT/$fixture_rel"
expect_failure fixture-tamper verify_fixture "$FIXTURE_TAMPER_ROOT" integrity

SENTINEL_TAMPER_ROOT="$TEMP_ROOT/sentinel-tamper"
copy_fixture "$SENTINEL_TAMPER_ROOT"
ROOT_IN="$SENTINEL_TAMPER_ROOT" CONTRACT_REL_IN="$CONTRACT_REL" bun -e '
import { createHash } from "node:crypto";
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const fixturePath = `${process.env.ROOT_IN}/${contract.fixtureEvidence.file}`;
const fixture = await Bun.file(fixturePath).json();
fixture.cases.find((item) => item.id === "sentinel-empty").rows[0].permissionString = "invented";
const content = `${JSON.stringify(fixture, null, 2)}\n`;
await Bun.write(fixturePath, content);
contract.fixtureEvidence.sha256 = createHash("sha256").update(content).digest("hex");
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_failure sentinel-tamper verify_fixture "$SENTINEL_TAMPER_ROOT" integrity
grep -q "fixture evidence is not frozen\|sentinel fact must remain null" "$TEMP_ROOT/sentinel-tamper.out"

SQL_TAMPER_ROOT="$TEMP_ROOT/sql-tamper"
copy_fixture "$SQL_TAMPER_ROOT"
ROOT_IN="$SQL_TAMPER_ROOT" CONTRACT_REL_IN="$CONTRACT_REL" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const adapterEvidence = contract.implementationEvidence.find((item) => item.id === "impl-core-snapshot-adapter");
const adapterPath = `${process.env.ROOT_IN}/${adapterEvidence.file}`;
let adapter = await Bun.file(adapterPath).text();
adapter = adapter.replace("LEFT JOIN public.permissions AS permission", "LEFT JOIN permissions AS permission");
await Bun.write(adapterPath, adapter);
adapterEvidence.sha256 = sha(adapter);
const marker = `pub const ${contract.sqlEvidence.constant}: &str = r#\"`;
const sql = adapter.split(marker)[1].split("\"#;")[0];
contract.sqlEvidence.sha256 = sha(sql);
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
'
expect_failure sql-tamper verify_fixture "$SQL_TAMPER_ROOT" integrity
grep -q "implementation tuple drifted\|SQL digest is not frozen\|SQL public-qualified table inventory drifted" "$TEMP_ROOT/sql-tamper.out"

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

echo "core-ranking-snapshot-adapter-self-test: PASS deterministic report, readiness sentinel, immutable pins, offline env, fixture/sentinel/SQL/test/STOP/order/path/hash/tamper/symlink cases"
