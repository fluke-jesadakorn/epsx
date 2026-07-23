#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-9-identity-event-containment.sh"
CONTRACT_REL="docs/migration/contracts/a2-9-identity-event-containment.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-9-identity-containment.XXXXXX")"
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
const files = new Set();
for (const section of ["implementationEvidence", "testOnlyModuleEvidence", "unchangedEvidence", "staleDeploymentEvidence"]) {
  for (const item of contract[section]) files.add(item.file);
}
for (const file of files) console.log(file);
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
    echo "identity-event-containment-self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "identity-event-containment: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "12 invariants; 3 hermetic tests; 12 frozen digests; 11 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.targetBase !== "fd780ff257f0bc15910053704c5a59e5b3da4a3e" || report.invariants !== 12 || report.implementationEvidence !== 2 || report.testOnlyModules !== 3 || report.unchangedEvidence !== 4 || report.staleDeploymentEvidence !== 3 || report.hermeticTests !== 3 || report.residualStops.length !== 11) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_status=$?
set -e
if [[ "$readiness_status" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "identity-event-containment-self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "11 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure database-env env DATABASE_URL=postgres://example.invalid/epsx "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure network-env env ALLOW_NETWORK=true "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure identity-url-env env IDENTITY_SSE_URL=http://example.invalid "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

grep -Fq 'git("rev-parse", `${expectedTarget.commit}^{commit}`)' "$VERIFY"
if grep -Fq 'git("rev-parse", `${expectedTarget.ref}^{commit}`)' "$VERIFY"; then
  echo "identity-event-containment-self-test: target base must remain valid after the branch advances" >&2
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
mutate_contract target-pin 'contract.targetBase.commit = "0000000000000000000000000000000000000000";'
mutate_contract base-blob 'contract.targetBase.evidence[0].blob = "0000000000000000000000000000000000000000";'
mutate_contract invariant-inventory 'contract.invariants[0].id = "renamed";'
mutate_contract implementation-hash 'contract.implementationEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract test-only-inventory 'contract.testOnlyModuleEvidence.pop();'
mutate_contract unchanged-hash 'contract.unchangedEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract stale-anchor 'contract.staleDeploymentEvidence[0].anchor = "removed";'
mutate_contract test-inventory 'contract.hermeticTests[0] = "invented";'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
mutate_contract execution-order 'contract.requiredExecutionOrder.pop();'
mutate_contract implementation-traversal 'contract.implementationEvidence[0].file = "../outside";'
mutate_contract stale-traversal 'contract.staleDeploymentEvidence[0].file = "../outside";'

IMPLEMENTATION_TAMPER_ROOT="$TEMP_ROOT/implementation-tamper"
copy_fixture "$IMPLEMENTATION_TAMPER_ROOT"
implementation_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.implementationEvidence[0].file);' "$IMPLEMENTATION_TAMPER_ROOT/$CONTRACT_REL")"
printf '\n// mutation\n' >>"$IMPLEMENTATION_TAMPER_ROOT/$implementation_rel"
expect_failure implementation-tamper verify_fixture "$IMPLEMENTATION_TAMPER_ROOT" integrity

ROUTE_TAMPER_ROOT="$TEMP_ROOT/route-tamper"
copy_fixture "$ROUTE_TAMPER_ROOT"
ROUTE_PATCHED_VERIFY="$TEMP_ROOT/route-patched-verify.sh"
ROOT_IN="$ROUTE_TAMPER_ROOT" CONTRACT_REL_IN="$CONTRACT_REL" VERIFY_IN="$VERIFY" VERIFY_OUT="$ROUTE_PATCHED_VERIFY" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const item = contract.implementationEvidence.find((entry) => entry.id === "impl-production-grpc-only-main");
const path = `${process.env.ROOT_IN}/${item.file}`;
const oldDigest = item.sha256;
const content = (await Bun.file(path).text()).replace("#[cfg(test)]\nmod tests", "// /v1/emit\n#[cfg(test)]\nmod tests");
await Bun.write(path, content);
item.sha256 = sha(content);
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
const verifier = (await Bun.file(process.env.VERIFY_IN).text()).replaceAll(oldDigest, item.sha256);
await Bun.write(process.env.VERIFY_OUT, verifier);
'
chmod +x "$ROUTE_PATCHED_VERIFY"
expect_failure route-tamper env EPSX_A2_9_REPO_ROOT="$REPO_ROOT" "$ROUTE_PATCHED_VERIFY" --mode integrity --evidence-root "$ROUTE_TAMPER_ROOT" --contract "$ROUTE_TAMPER_ROOT/$CONTRACT_REL" --static-only
grep -q "forbidden production identity token /v1/" "$TEMP_ROOT/route-tamper.out"

CFG_TAMPER_ROOT="$TEMP_ROOT/cfg-tamper"
copy_fixture "$CFG_TAMPER_ROOT"
CFG_PATCHED_VERIFY="$TEMP_ROOT/cfg-patched-verify.sh"
ROOT_IN="$CFG_TAMPER_ROOT" CONTRACT_REL_IN="$CONTRACT_REL" VERIFY_IN="$VERIFY" VERIFY_OUT="$CFG_PATCHED_VERIFY" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const item = contract.implementationEvidence.find((entry) => entry.id === "impl-test-only-module-boundary");
const path = `${process.env.ROOT_IN}/${item.file}`;
const oldDigest = item.sha256;
const content = (await Bun.file(path).text()).replace("#[cfg(test)]\npub mod emit_handler;", "pub mod emit_handler;");
await Bun.write(path, content);
item.sha256 = sha(content);
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
const verifier = (await Bun.file(process.env.VERIFY_IN).text()).replaceAll(oldDigest, item.sha256);
await Bun.write(process.env.VERIFY_OUT, verifier);
'
chmod +x "$CFG_PATCHED_VERIFY"
expect_failure cfg-tamper env EPSX_A2_9_REPO_ROOT="$REPO_ROOT" "$CFG_PATCHED_VERIFY" --mode integrity --evidence-root "$CFG_TAMPER_ROOT" --contract "$CFG_TAMPER_ROOT/$CONTRACT_REL" --static-only
grep -q "emit_handler must remain explicitly cfg(test)" "$TEMP_ROOT/cfg-tamper.out"

STALE_TAMPER_ROOT="$TEMP_ROOT/stale-tamper"
copy_fixture "$STALE_TAMPER_ROOT"
stale_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.staleDeploymentEvidence[0].file);' "$STALE_TAMPER_ROOT/$CONTRACT_REL")"
printf '\n# mutation\n' >>"$STALE_TAMPER_ROOT/$stale_rel"
expect_failure stale-tamper verify_fixture "$STALE_TAMPER_ROOT" integrity

SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/$implementation_rel" "$TEMP_ROOT/outside-main.rs"
ln -s "$TEMP_ROOT/outside-main.rs" "$SYMLINK_ROOT/$implementation_rel"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "identity-event-containment-self-test: PASS deterministic report, readiness sentinel, immutable base, offline env, route/cfg/STOP/order/path/hash/stale/symlink tamper cases"
