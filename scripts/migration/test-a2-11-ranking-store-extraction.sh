#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
VERIFY="$SCRIPT_DIR/verify-a2-11-ranking-store-extraction.sh"
CONTRACT_REL="docs/migration/contracts/a2-11-ranking-store-extraction.json"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-11-ranking-store.XXXXXX")"
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
const files = new Set(contract.implementationEvidence.map((item) => item.file));
files.add(contract.fixtureEvidence.file);
for (const file of files) console.log(file);
' "$REPO_ROOT/$CONTRACT_REL")
}

verify_fixture() {
  local root="$1"
  local mode="$2"
  EPSX_A2_11_REPO_ROOT="$REPO_ROOT" \
    "$VERIFY" --mode "$mode" --evidence-root "$root" --contract "$root/$CONTRACT_REL" --static-only
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
    echo "ranking-store-extraction-self-test: expected $label exit 1, got $code" >&2
    exit 1
  fi
  grep -q "ranking-store-extraction: ERROR:" "$output"
}

BASE="$TEMP_ROOT/base"
copy_fixture "$BASE"
verify_fixture "$BASE" integrity >"$TEMP_ROOT/integrity.out"
grep -q "12 invariants; 12 exact hermetic tests; 12 implementation digests; 15 residual STOPs" "$TEMP_ROOT/integrity.out"

verify_fixture "$BASE" report >"$TEMP_ROOT/report-one.json"
verify_fixture "$BASE" report >"$TEMP_ROOT/report-two.json"
cmp "$TEMP_ROOT/report-one.json" "$TEMP_ROOT/report-two.json"
bun -e '
const report = await Bun.file(process.argv[1]).json();
if (report.productionReady !== false || report.readinessExit !== 3 || report.targetBase !== "005a604542271050279a6190fc00eada00f32137" || report.sourceEvidence !== 5 || report.baseEvidence !== 11 || report.invariants !== 12 || report.implementationEvidence !== 12 || report.fixtureCases !== 21 || report.hermeticTests !== 12 || report.residualStops.length !== 15) process.exit(1);
' "$TEMP_ROOT/report-one.json"

set +e
verify_fixture "$BASE" readiness >"$TEMP_ROOT/readiness.out" 2>&1
readiness_code=$?
set -e
if [[ "$readiness_code" -ne 3 ]]; then
  cat "$TEMP_ROOT/readiness.out" >&2
  echo "ranking-store-extraction-self-test: expected readiness exit 3, got $readiness_code" >&2
  exit 1
fi
grep -q "15 residual STOPs remain" "$TEMP_ROOT/readiness.out"

expect_failure production-env env EPSX_ENV=production EPSX_A2_11_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure database-env env DATABASE_URL=postgres://example.invalid/epsx EPSX_A2_11_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
expect_failure network-env env ALLOW_NETWORK=true EPSX_A2_11_REPO_ROOT="$REPO_ROOT" "$VERIFY" --mode integrity --evidence-root "$BASE" --contract "$BASE/$CONTRACT_REL" --static-only
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
mutate_contract source-pin 'contract.sourceBaseline.commit = "0000000000000000000000000000000000000000";'
mutate_contract target-pin 'contract.targetBase.commit = "0000000000000000000000000000000000000000";'
mutate_contract invariant-inventory 'contract.invariants[0].id = "renamed";'
mutate_contract implementation-hash 'contract.implementationEvidence[0].sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract fixture-hash 'contract.fixtureEvidence.sha256 = "0000000000000000000000000000000000000000000000000000000000000000";'
mutate_contract package-contract 'contract.packageContract.binaryTargets = 1;'
mutate_contract test-inventory 'contract.hermeticTests[0] = "invented";'
mutate_contract stop-inventory 'contract.residualStops[0].id = "renamed";'
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
  ROOT_IN="$root" CONTRACT_REL_IN="$CONTRACT_REL" VERIFY_IN="$VERIFY" VERIFY_OUT="$patched_verify" IMPLEMENTATION_ID="$implementation_id" OLD_SOURCE="$old_source" NEW_SOURCE="$new_source" bun -e '
import { createHash } from "node:crypto";
const sha = (content) => createHash("sha256").update(content).digest("hex");
const contractPath = `${process.env.ROOT_IN}/${process.env.CONTRACT_REL_IN}`;
const contract = await Bun.file(contractPath).json();
const item = contract.implementationEvidence.find((entry) => entry.id === process.env.IMPLEMENTATION_ID);
if (!item) throw new Error("implementation id missing");
const path = `${process.env.ROOT_IN}/${item.file}`;
const oldDigest = item.sha256;
const original = await Bun.file(path).text();
if (!original.includes(process.env.OLD_SOURCE)) throw new Error("semantic tamper source anchor missing");
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
  expect_failure "$label" env EPSX_A2_11_REPO_ROOT="$REPO_ROOT" "$patched_verify" --mode integrity --evidence-root "$root" --contract "$root/$CONTRACT_REL" --static-only
  grep -q "$expected_error" "$TEMP_ROOT/$label.out"
}

semantic_tamper \
  decoder-tamper \
  impl-ranking-store-library \
  'return Err(RankingEntitlementSnapshotError::Corrupt);' \
  'return Ok(RankingEntitlementSnapshot { normalized_wallet: expected_wallet.to_string(), observed_at: 0, assignments: Vec::new() });' \
  'extracted adapter drifted from the normalized immutable source'

semantic_tamper \
  pool-tamper \
  impl-ranking-store-library \
  'pool: TlsPool,' \
  "pool: &'static TlsPool," \
  'owned pool field\|leaked static pool wrapper\|extracted adapter drifted from the normalized immutable source'

semantic_tamper \
  shim-tamper \
  impl-backend-compatibility-reexport \
  'PostgresRankingEntitlementSnapshotRepository, RANKING_ENTITLEMENT_SNAPSHOT_SQL,' \
  'PostgresRankingEntitlementSnapshotRepository,' \
  'backend compatibility module must be an exact two-symbol re-export'

semantic_tamper \
  identity-dependency-tamper \
  impl-identity-manifest-unchanged \
  'epsx-contracts = { path = "../epsx-contracts" }' \
  $'epsx-contracts = { path = "../epsx-contracts" }\nepsx-ranking-store = { path = "../epsx-ranking-store" }' \
  'identity store dependency'

store_rel="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(c.implementationEvidence.find((item)=>item.id==="impl-ranking-store-library").file);' "$BASE/$CONTRACT_REL")"
SYMLINK_ROOT="$TEMP_ROOT/symlink-escape"
copy_fixture "$SYMLINK_ROOT"
mv "$SYMLINK_ROOT/$store_rel" "$TEMP_ROOT/outside-store.rs"
ln -s "$TEMP_ROOT/outside-store.rs" "$SYMLINK_ROOT/$store_rel"
expect_failure symlink-escape verify_fixture "$SYMLINK_ROOT" integrity

echo "ranking-store-extraction-self-test: PASS deterministic report, readiness sentinel, immutable source/base, offline env, exact package/test/STOP/path/hash/symlink checks, and digest-bypassing decoder/pool/shim/identity semantic tamper cases"
