#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-10-authenticated-ranking-rpc.sh"
CONTRACT_REL="docs/migration/contracts/a2-10-authenticated-ranking-rpc.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-10-authenticated-ranking-rpc.XXXXXX")"
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
for (const section of ["implementationEvidence", "unchangedEvidence"]) {
  for (const item of contract[section]) files.add(item.file);
}
for (const file of files) console.log(file);
' "$REPO_ROOT/$CONTRACT_REL")
}

verify_fixture() {
  local root="$1"
  local mode="$2"
  EPSX_A2_10_REPO_ROOT="$REPO_ROOT" \
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
    echo "authenticated-ranking-rpc-self-test: expected $label exit 1, got $status" >&2
    exit 1
  fi
  grep -q "authenticated-ranking-rpc: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "16 invariants; 11 exact hermetic tests; 14 frozen digests; 7 status mappings; 9 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.targetBase !== "60ababc75a79d173b3b217df8e9b9155795a1117" || report.invariants !== 16 || report.implementationEvidence !== 2 || report.unchangedEvidence !== 12 || report.statusMappings !== 7 || report.hermeticTests !== 11 || report.residualStops.length !== 9) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_status=$?
set -e
if [[ "$readiness_status" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "authenticated-ranking-rpc-self-test: expected readiness exit 3, got $readiness_status" >&2
  exit 1
fi
grep -q "9 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production EPSX_A2_10_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure database-env env DATABASE_URL=postgres://example.invalid/epsx EPSX_A2_10_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure network-env env ALLOW_NETWORK=true EPSX_A2_10_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure identity-url-env env IDENTITY_GRPC_URL=http://example.invalid EPSX_A2_10_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

grep -Fq 'git("rev-parse", `${expectedTarget.commit}^{commit}`)' "$VERIFY"
if grep -Fq 'git("rev-parse", `${expectedTarget.ref}^{commit}`)' "$VERIFY"; then
  echo "authenticated-ranking-rpc-self-test: target base must remain valid after the branch advances" >&2
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
mutate_contract unchanged-hash 'contract.unchangedEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract authentication-contract 'contract.authenticationContract.expectedAudience = "wrong";'
mutate_contract wallet-contract 'contract.walletContract.byteLength = 41;'
mutate_contract status-contract 'contract.statusContract[2].code = "Unauthenticated";'
mutate_contract test-inventory 'contract.hermeticTests[0] = "invented";'
mutate_contract path-scope 'contract.unchangedPathScopes.pop();'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
mutate_contract execution-order 'contract.requiredExecutionOrder.pop();'
mutate_contract implementation-traversal 'contract.implementationEvidence[0].file = "../outside";'

implementation_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.implementationEvidence[0].file);' "$BASE/$CONTRACT_REL")"
IMPLEMENTATION_TAMPER_ROOT="$TEMP_ROOT/implementation-tamper"
copy_fixture "$IMPLEMENTATION_TAMPER_ROOT"
printf '\n// mutation\n' >>"$IMPLEMENTATION_TAMPER_ROOT/$implementation_rel"
expect_failure implementation-tamper verify_fixture "$IMPLEMENTATION_TAMPER_ROOT" integrity

semantic_tamper() {
  local label="$1"
  local old_source="$2"
  local new_source="$3"
  local expected_error="$4"
  local root="$TEMP_ROOT/$label"
  local patched_verify="$TEMP_ROOT/$label-verifier.sh"
  copy_fixture "$root"
  ROOT_IN="$root" CONTRACT_REL_IN="$CONTRACT_REL" VERIFY_IN="$VERIFY" VERIFY_OUT="$patched_verify" OLD_SOURCE="$old_source" NEW_SOURCE="$new_source" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const item = contract.implementationEvidence.find((entry) => entry.id === "impl-authenticated-ranking-rpc");
const path = `${process.env.ROOT_IN}/${item.file}`;
const oldDigest = item.sha256;
const original = await Bun.file(path).text();
if (!original.includes(process.env.OLD_SOURCE)) throw new Error("semantic tamper source anchor missing");
const content = original.replace(process.env.OLD_SOURCE, process.env.NEW_SOURCE);
await Bun.write(path, content);
item.sha256 = sha(content);
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
const verifier = (await Bun.file(process.env.VERIFY_IN).text()).replaceAll(oldDigest, item.sha256);
if (verifier === await Bun.file(process.env.VERIFY_IN).text()) throw new Error("verifier digest pin was not patched");
await Bun.write(process.env.VERIFY_OUT, verifier);
'
  chmod +x "$patched_verify"
  expect_failure "$label" env EPSX_A2_10_REPO_ROOT="$REPO_ROOT" "$patched_verify" --mode integrity --evidence-root "$root" --contract "$root/$CONTRACT_REL" --static-only
  grep -q "$expected_error" "$TEMP_ROOT/$label.out"
}

semantic_tamper \
  ordering-tamper \
  'let bearer = parse_bearer(request.metadata())?;' \
  $'let _wallet_before_authorization = normalize_evm_wallet(&request.get_ref().wallet)?;\n        let bearer = parse_bearer(request.metadata())?;' \
  'authentication must precede subject/audience, wallet validation, and query work'

semantic_tamper \
  status-tamper \
  'Status::permission_denied(CALLER_FORBIDDEN)' \
  'Status::unauthenticated(CALLER_FORBIDDEN)' \
  'missing exact status mapping Status::permission_denied(CALLER_FORBIDDEN)'

LIB_TAMPER_ROOT="$TEMP_ROOT/lib-export-tamper"
copy_fixture "$LIB_TAMPER_ROOT"
LIB_PATCHED_VERIFY="$TEMP_ROOT/lib-export-verifier.sh"
ROOT_IN="$LIB_TAMPER_ROOT" CONTRACT_REL_IN="$CONTRACT_REL" VERIFY_IN="$VERIFY" VERIFY_OUT="$LIB_PATCHED_VERIFY" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const item = contract.implementationEvidence.find((entry) => entry.id === "impl-library-export");
const path = `${process.env.ROOT_IN}/${item.file}`;
const oldDigest = item.sha256;
const content = (await Bun.file(path).text()).replace("pub mod authenticated_ranking_rpc;", "#[cfg(test)]\npub mod authenticated_ranking_rpc;");
await Bun.write(path, content);
item.sha256 = sha(content);
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
const verifier = (await Bun.file(process.env.VERIFY_IN).text()).replaceAll(oldDigest, item.sha256);
await Bun.write(process.env.VERIFY_OUT, verifier);
'
chmod +x "$LIB_PATCHED_VERIFY"
expect_failure lib-export-tamper env EPSX_A2_10_REPO_ROOT="$REPO_ROOT" "$LIB_PATCHED_VERIFY" --mode integrity --evidence-root "$LIB_TAMPER_ROOT" --contract "$LIB_TAMPER_ROOT/$CONTRACT_REL" --static-only
grep -q "normal non-test library export" "$TEMP_ROOT/lib-export-tamper.out"

SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/$implementation_rel" "$TEMP_ROOT/outside-rpc.rs"
ln -s "$TEMP_ROOT/outside-rpc.rs" "$SYMLINK_ROOT/$implementation_rel"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "authenticated-ranking-rpc-self-test: PASS deterministic report, readiness sentinel, immutable base, offline env, exact auth/wallet/status/test/STOP/path/hash/symlink checks, and digest-bypassing ordering/status/export semantic tamper cases"
