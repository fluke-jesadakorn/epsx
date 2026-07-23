#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a1-9-auth-recovery-ux.sh"
CONTRACT_REL="docs/migration/contracts/a1-9-auth-recovery-ux.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a1-9-auth-recovery.XXXXXX")"
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
for (const file of new Set(contract.implementationEvidence.map((item) => item.file))) console.log(file);
' "$REPO_ROOT/$CONTRACT_REL")
}

verify_fixture() {
  local root="$1"
  local mode="$2"
  local verifier="${3:-$VERIFY}"
  EPSX_A1_9_REPO_ROOT="$REPO_ROOT" \
    "$verifier" --mode "$mode" --evidence-root "$root" \
      --contract "$root/$CONTRACT_REL" --static-only
}

expect_failure() {
  local label="$1"
  shift
  local output="$TEMP_ROOT/$label.out"
  set +e
  "$@" >"$output" 2>&1
  local code=$?
  set -e
  if [[ "$code" -ne 1 ]]; then
    cat "$output" >&2
    echo "auth-recovery-ux-self-test: expected $label exit 1, got $code" >&2
    exit 1
  fi
  grep -Fq "auth-recovery-ux: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -Fq "12 invariants; 5 implementation digests; 15 exact hermetic tests including fake-DOM 4/4; 13 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (
  report.productionReady !== false ||
  report.readinessExit !== 3 ||
  report.sourceBaseline !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" ||
  report.implementationBase !== "346d520484e23532ec40a62d1e2fba9d7a10472c" ||
  report.predecessor !== "c238954cbbf9b8a5db57ef117f0be638c4613766" ||
  report.invariants !== 12 ||
  report.implementationEvidence !== 5 ||
  report.hermeticTests !== 15 ||
  report.fakeDomCases !== 4 ||
  report.residualStops.length !== 13
) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_code=$?
set -e
if [[ "$readiness_code" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "auth-recovery-ux-self-test: expected readiness exit 3, got $readiness_code" >&2
  exit 1
fi
grep -Fq "13 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env \
  env EPSX_ENV=production EPSX_A1_9_REPO_ROOT="$REPO_ROOT" \
  "$VERIFY" --mode integrity --evidence-root "$BASE" \
  --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure database-env \
  env DATABASE_URL=postgres://example.invalid/epsx EPSX_A1_9_REPO_ROOT="$REPO_ROOT" \
  "$VERIFY" --mode integrity --evidence-root "$BASE" \
  --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure network-env \
  env ALLOW_NETWORK=true EPSX_A1_9_REPO_ROOT="$REPO_ROOT" \
  "$VERIFY" --mode integrity --evidence-root "$BASE" \
  --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure canonical-static-only "$VERIFY" --mode integrity --static-only

mutate_contract() {
  local label="$1"
  local expression="$2"
  local root="$TEMP_ROOT/$label"
  copy_fixture "$root"
  CONTRACT_IN="$root/$CONTRACT_REL" EXPRESSION="$expression" bun -e '
const path = process.env.CONTRACT_IN;
const contract = await Bun.file(path).json();
new Function("contract", process.env.EXPRESSION)(contract);
await Bun.write(path, `${JSON.stringify(contract, null, 2)}\n`);
'
  expect_failure "$label" verify_fixture "$root" integrity
}

mutate_contract readiness-sentinel 'contract.productionReady = true;'
mutate_contract predecessor-pin 'contract.predecessor.commit = "0000000000000000000000000000000000000000";'
mutate_contract invariant-inventory 'contract.invariants[0].id = "renamed";'
mutate_contract test-inventory 'contract.hermeticTests[14].exactName = "invented";'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
mutate_contract implementation-hash 'contract.implementationEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract traversal 'contract.implementationEvidence[0].file = "../outside";'

semantic_tamper() {
  local label="$1"
  local implementation_id="$2"
  local old_source="$3"
  local new_source="$4"
  local expected_error="$5"
  local root="$TEMP_ROOT/$label"
  local patched_verify="$TEMP_ROOT/$label-verifier.sh"
  copy_fixture "$root"
  ROOT_IN="$root" CONTRACT_REL_IN="$CONTRACT_REL" VERIFY_IN="$VERIFY" \
    VERIFY_OUT="$patched_verify" IMPLEMENTATION_ID="$implementation_id" \
    OLD_SOURCE="$old_source" NEW_SOURCE="$new_source" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const item = contract.implementationEvidence.find(
  (entry) => entry.id === process.env.IMPLEMENTATION_ID,
);
if (!item) throw new Error("implementation id missing");
const path = `${process.env.ROOT_IN}/${item.file}`;
const oldDigest = item.sha256;
const original = await Bun.file(path).text();
if (!original.includes(process.env.OLD_SOURCE)) {
  throw new Error(`semantic tamper source anchor missing: ${process.env.OLD_SOURCE}`);
}
const content = original.replace(process.env.OLD_SOURCE, process.env.NEW_SOURCE);
await Bun.write(path, content);
item.sha256 = sha(content);
await Bun.write(contractPath, `${JSON.stringify(contract, null, 2)}\n`);
const verifierOriginal = await Bun.file(process.env.VERIFY_IN).text();
const verifier = verifierOriginal.replaceAll(oldDigest, item.sha256);
if (verifier === verifierOriginal) throw new Error("verifier digest pin was not patched");
await Bun.write(process.env.VERIFY_OUT, verifier);
'
  chmod +x "$patched_verify"
  expect_failure "$label" verify_fixture "$root" integrity "$patched_verify"
  grep -Fq "$expected_error" "$TEMP_ROOT/$label.out"
}

semantic_tamper \
  verifier-outage-cache-bypass \
  impl-frontend-ssr-state \
  'is_authenticated || recover_session || auth_page_verifier_unavailable' \
  'is_authenticated || recover_session' \
  'missing private verifier-outage cache branch'

semantic_tamper \
  stale-wallet-event-bypass \
  impl-auth-page-state-machine \
  'if (!d || !authActionable()) return;' \
  'if (!d) return;' \
  'missing stale wallet-event guard'

semantic_tamper \
  recovery-payload-disclosure \
  impl-auth-page-state-machine \
  "errorMsg.textContent = 'We could not restore your session. Try connecting your wallet again.';" \
  'errorMsg.textContent = d.message;' \
  'recovery-failure transition'

semantic_tamper \
  real-bridge-harness-bypass \
  impl-fake-dom-harness \
  'vm.runInContext(browserScript, context, { filename: "browser_auth.js" });' \
  'vm.runInContext("", context, { filename: "browser_auth.js" });' \
  'real-bridge fake-DOM evidence'

semantic_tamper \
  unknown-state-comment-spoof \
  impl-auth-page-state-machine \
  'Some(_) => Self::VerifierUnavailable,' \
  'Some(_) => Self::SignedOut, // Some(_) => Self::VerifierUnavailable,' \
  'fail-closed UI state'

state_file="$(bun -e '
const contract = await Bun.file(process.argv[1]).json();
process.stdout.write(contract.implementationEvidence.find((item) => item.id === "impl-auth-page-state-machine").file);
' "$BASE/$CONTRACT_REL")"
SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/$state_file" "$TEMP_ROOT/outside-auth-page.rs"
ln -s "$TEMP_ROOT/outside-auth-page.rs" "$SYMLINK_ROOT/$state_file"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "auth-recovery-ux-self-test: PASS deterministic report, readiness sentinel, exact A1.8 replay, offline env, path/symlink/hash inventories, and digest-bypassing cache/stale-event/disclosure/real-bridge/comment-spoof mutations"
