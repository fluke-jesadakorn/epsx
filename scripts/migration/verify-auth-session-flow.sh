#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
MANIFEST="$REPO_ROOT/docs/migration/contracts/auth-session-gate.json"
REPORT_DIR="$REPO_ROOT/target/migration"
REPORT="$REPORT_DIR/auth-session-gate-report.json"

die() {
  echo "auth-session-gate: ERROR: $*" >&2
  exit 1
}

if (( $# != 0 )); then
  die "this hermetic gate accepts no arguments or URLs"
fi

for tool in bun cargo git mktemp; do
  command -v "$tool" >/dev/null 2>&1 || die "$tool is required"
done

for name in EPSX_ENV ENV APP_ENV ENVIRONMENT NODE_ENV RUST_ENV DEPLOY_ENV COOKIE_ENVIRONMENT; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    prod|production|prod-*|production-*|*-prod|*-production)
      die "$name identifies a production-looking environment"
      ;;
  esac
done

for name in DATABASE_URL TEST_DATABASE_URL ANALYTICS_DATABASE_URL NOTIFICATIONS_DATABASE_URL \
  PAYMENTS_DATABASE_URL IDENTITY_DATABASE_URL WALLET_DATABASE_URL \
  SUBSCRIPTION_DATABASE_URL CONTENT_DATABASE_URL NOTIFICATION_DATABASE_URL \
  INDEXER_DATABASE_URL PAY_DATABASE_URL REDIS_URL IDENTITY_REDIS_URL \
  WALLET_REDIS_URL SUBSCRIPTION_REDIS_URL CONTENT_REDIS_URL \
  NOTIFICATION_REDIS_URL ANALYTICS_REDIS_URL INDEXER_REDIS_URL PAY_REDIS_URL; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this gate never uses a database or Redis"
done

for name in API_URL BACKEND_URL OIDC_ISSUER FRONTEND_URL ADMIN_FRONTEND_URL IDENTITY_URL; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    *epsx.io*|*production*|*prod.*|*prod-*)
      die "$name contains a production-looking URL"
      ;;
  esac
done

export CARGO_NET_OFFLINE=true
export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy
unset API_URL BACKEND_URL OIDC_ISSUER FRONTEND_URL ADMIN_FRONTEND_URL IDENTITY_URL

mkdir -p "$REPORT_DIR" || die "cannot create report directory"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/epsx-auth-session-gate.XXXXXX")" || die "cannot create temporary work directory"
[[ -n "$WORK_DIR" && -d "$WORK_DIR" ]] || die "temporary work directory is invalid"
RESULTS="$WORK_DIR/results.tsv"
: > "$RESULTS" || die "cannot initialize result ledger"
trap '[[ -n "${WORK_DIR:-}" && -d "$WORK_DIR" ]] && rm -rf -- "$WORK_DIR"' EXIT

bun -e '
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const [root, path] = process.argv.slice(1);
const fail = (message) => { console.error(`auth-session-gate: ERROR: ${message}`); process.exit(1); };
let manifest;
try { manifest = JSON.parse(readFileSync(path, "utf8")); }
catch (error) { fail(`invalid manifest JSON: ${error.message}`); }

if (manifest.schemaVersion !== 1 || manifest.gateId !== "A1.4-hermetic-auth-session") fail("unexpected schemaVersion or gateId");
if (manifest.mode !== "local-hermetic-only" || manifest.productionReady !== false) fail("gate must remain local-only and non-production");
if (manifest.liveWalletFlowProven !== false || manifest.durableRefreshStoreProven !== false) fail("live and durable claims must remain false");
if (!Array.isArray(manifest.blockedClaims) || manifest.blockedClaims.length < 4) fail("blockedClaims must preserve live-flow limits");
if (!Array.isArray(manifest.capabilities) || manifest.capabilities.length !== 7) fail("seven capability records are required");
if (!Array.isArray(manifest.cases) || manifest.cases.length !== 15) fail("fifteen fixed cases are required");

const ids = new Set();
let expectedTotal = 0;
let evidenceTotal = 0;
for (const item of manifest.cases) {
  if (!item || !/^[a-z][a-z0-9-]+$/.test(item.id) || ids.has(item.id)) fail(`invalid or duplicate case id: ${item?.id}`);
  ids.add(item.id);
  if (!new Set(["cargo-test", "fixture-check"]).has(item.kind)) fail(`${item.id}: invalid kind`);
  if (!Number.isInteger(item.expectedTests) || item.expectedTests < 0) fail(`${item.id}: invalid expectedTests`);
  if (item.kind === "cargo-test" && item.expectedTests === 0) fail(`${item.id}: cargo test must expect tests`);
  if (item.kind === "fixture-check" && item.expectedTests !== 0) fail(`${item.id}: fixture check cannot count tests`);
  expectedTotal += item.expectedTests;
  if (!Array.isArray(item.command) || item.command.length === 0 || item.command.some((part) => typeof part !== "string" || !part)) fail(`${item.id}: invalid command`);
  const commandText = item.command.join(" ");
  if (/https?:\/\/|\bpsql\b|\bkubectl\b|\bdocker\b|\bcurl\b/.test(commandText)) fail(`${item.id}: command is not hermetic`);
  if (!Array.isArray(item.evidence) || item.evidence.length === 0) fail(`${item.id}: evidence is required`);
  for (const evidence of item.evidence) {
    if (typeof evidence.file !== "string" || evidence.file.startsWith("/") || evidence.file.split("/").includes("..")) fail(`${item.id}: invalid evidence path`);
    if (typeof evidence.anchor !== "string" || !evidence.anchor) fail(`${item.id}: invalid evidence anchor`);
    let content;
    try { content = readFileSync(resolve(root, evidence.file), "utf8"); }
    catch { fail(`${item.id}: missing evidence file ${evidence.file}`); }
    if (!content.includes(evidence.anchor)) fail(`${item.id}: missing evidence anchor in ${evidence.file}: ${JSON.stringify(evidence.anchor)}`);
    evidenceTotal += 1;
  }
}
if (expectedTotal !== 71) fail(`expected focused test total changed: ${expectedTotal}`);
for (const capability of manifest.capabilities) {
  if (!Array.isArray(capability.caseIds) || capability.caseIds.length === 0) fail(`${capability.id}: caseIds are required`);
  for (const id of capability.caseIds) if (!ids.has(id)) fail(`${capability.id}: unknown case ${id}`);
}
console.log(`auth-session-gate: manifest 15/15 cases, ${evidenceTotal}/${evidenceTotal} anchors, ${expectedTotal} focused tests`);
' -- "$REPO_ROOT" "$MANIFEST" || exit 1

OVERALL="pass"
PASSED_TESTS=0

run_test() {
  id="$1"
  expected="$2"
  shift 2
  log="$WORK_DIR/$id.log"
  command_text="$*"
  echo "auth-session-gate: RUN $id ($expected tests)"
  if (cd "$REPO_ROOT" && "$@" >"$log" 2>&1); then
    passed="$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed;.*/\1/p' "$log" | awk '{ total += $1 } END { print total + 0 }')"
    if [[ "$passed" == "$expected" ]]; then
      status="pass"
      PASSED_TESTS=$((PASSED_TESTS + passed))
      echo "auth-session-gate: PASS $id ($passed/$expected)"
    else
      status="fail"
      OVERALL="fail"
      echo "auth-session-gate: FAIL $id expected $expected passing tests, observed $passed" >&2
      tail -n 80 "$log" >&2
    fi
  else
    status="fail"
    passed=0
    OVERALL="fail"
    echo "auth-session-gate: FAIL $id" >&2
    tail -n 80 "$log" >&2
  fi
  printf '%s\t%s\t%s\t%s\t%s\n' "$id" "$status" "$expected" "$passed" "$command_text" >> "$RESULTS"
}

run_check() {
  id="$1"
  shift
  log="$WORK_DIR/$id.log"
  command_text="$*"
  echo "auth-session-gate: RUN $id"
  if (cd "$REPO_ROOT" && "$@" >"$log" 2>&1); then
    status="pass"
    echo "auth-session-gate: PASS $id"
  else
    status="fail"
    OVERALL="fail"
    echo "auth-session-gate: FAIL $id" >&2
    tail -n 80 "$log" >&2
  fi
  printf '%s\t%s\t0\t0\t%s\n' "$id" "$status" "$command_text" >> "$RESULTS"
}

run_test identity-token-contract 11 cargo test --offline --locked -p epsx-identity-shared token_service::tests --no-fail-fast
run_test identity-jwks-contract 6 cargo test --offline --locked -p epsx-identity-shared key_manager::tests --no-fail-fast
run_test bff-verifier 11 cargo test --offline --locked -p epsx-bff session::tests --no-fail-fast
run_test bff-cookie-contract 6 cargo test --offline --locked -p epsx-bff cookies::tests --no-fail-fast
run_test bff-browser-bridge 3 cargo test --offline --locked -p epsx-bff browser_auth::tests --no-fail-fast
run_test frontend-session 7 cargo test --offline --locked -p epsx-frontend api::auth_session_tests --no-fail-fast
run_test frontend-cookie-reader 3 cargo test --offline --locked -p epsx-frontend auth::tests --no-fail-fast
run_test frontend-production-config 1 cargo test --offline --locked -p epsx-frontend configuration_tests::production_requires_https_non_local_auth_urls --no-fail-fast
run_test frontend-safe-return 1 cargo test --offline --locked -p epsx-frontend ssr::tests::return_url_must_remain_same_origin --no-fail-fast
run_test admin-session 10 cargo test --offline --locked -p epsx-admin session_auth_tests --no-fail-fast
run_test admin-cookie-reader 3 cargo test --offline --locked -p epsx-admin auth::tests --no-fail-fast
run_test admin-production-config 1 cargo test --offline --locked -p epsx-admin configuration_tests::production_requires_https_non_local_auth_urls --no-fail-fast
run_test backend-auth-handlers 8 cargo test --offline --locked -p epsx --lib web::auth::handlers::tests --no-fail-fast
run_check route-inventory ./scripts/migration/verify-route-inventory.sh
run_check api-contract-fixtures ./scripts/migration/verify-contract-fixtures.sh

bun -e '
import { readFileSync, writeFileSync } from "node:fs";

const [manifestPath, resultsPath, reportPath, overall, passedTests] = process.argv.slice(1);
const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
const resultRows = readFileSync(resultsPath, "utf8").trim().split(/\r?\n/).filter(Boolean).map((line) => {
  const [id, status, expectedTests, passed, command] = line.split("\t");
  return { id, status, expectedTests: Number(expectedTests), passedTests: Number(passed), command };
});
const byId = new Map(resultRows.map((item) => [item.id, item]));
const cases = manifest.cases.map((item) => {
  const result = byId.get(item.id);
  if (!result) throw new Error(`missing result for ${item.id}`);
  const expectedCommand = item.command.join(" ");
  if (result.command !== expectedCommand) throw new Error(`executed command drift for ${item.id}: ${result.command}`);
  return { id: item.id, kind: item.kind, status: result.status, expectedTests: item.expectedTests, passedTests: result.passedTests, command: item.command };
});
const report = {
  schemaVersion: 1,
  gateId: manifest.gateId,
  result: overall,
  productionReady: false,
  liveWalletFlowProven: false,
  durableRefreshStoreProven: false,
  focusedTests: { expected: 71, passed: Number(passedTests) },
  fixtureChecks: { expected: 2, passed: cases.filter((item) => item.kind === "fixture-check" && item.status === "pass").length },
  blockedClaims: manifest.blockedClaims,
  cases
};
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
' -- "$MANIFEST" "$RESULTS" "$REPORT" "$OVERALL" "$PASSED_TESTS" || die "could not write deterministic report"

echo "auth-session-gate: report $REPORT"
if [[ "$OVERALL" != "pass" || "$PASSED_TESTS" != "71" ]]; then
  die "STOP — hermetic auth-session gate failed ($PASSED_TESTS/71 focused tests)"
fi

echo "auth-session-gate: PASS — 71/71 focused tests and 2/2 fixture checks"
echo "auth-session-gate: LIMIT — live wallet, durable refresh store, and production flow remain unproven"
