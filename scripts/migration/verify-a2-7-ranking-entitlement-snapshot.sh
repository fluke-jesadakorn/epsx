#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT_RAW="${EPSX_A2_7_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_7_STATIC_ONLY:-0}"

die() {
  echo "ranking-entitlement-snapshot: ERROR: $*" >&2
  exit 1
}

while (( $# > 0 )); do
  case "$1" in
    --mode)
      (( $# >= 2 )) || die "--mode requires integrity, report, or readiness"
      MODE="$2"
      shift 2
      ;;
    --contract)
      (( $# >= 2 )) || die "--contract requires a local JSON file"
      CONTRACT="$2"
      shift 2
      ;;
    --evidence-root)
      (( $# >= 2 )) || die "--evidence-root requires a local directory"
      EVIDENCE_ROOT_RAW="$2"
      shift 2
      ;;
    --static-only)
      STATIC_ONLY=1
      shift
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

case "$MODE" in
  integrity|report|readiness) ;;
  *) die "--mode must be integrity, report, or readiness" ;;
esac

REPO_ROOT="$(CDPATH= cd -- "$REPO_ROOT_RAW" && pwd -P)" || die "repository root is unavailable"
EVIDENCE_ROOT="$(CDPATH= cd -- "$EVIDENCE_ROOT_RAW" && pwd -P)" || die "evidence root is unavailable"
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-7-ranking-entitlement-snapshot.json"
case "$CONTRACT" in
  http://*|https://*) die "contract must be a local file" ;;
esac
[[ -f "$CONTRACT" ]] || die "missing contract: $CONTRACT"
[[ "$STATIC_ONLY" == "0" || "$STATIC_ONLY" == "1" ]] || die "static-only must be 0 or 1"
if [[ "$EVIDENCE_ROOT" != "$REPO_ROOT" && "$STATIC_ONLY" != "1" ]]; then
  die "alternate evidence roots are accepted only in static-only self-tests"
fi
if [[ "$EVIDENCE_ROOT" == "$REPO_ROOT" && "$STATIC_ONLY" == "1" ]]; then
  die "static-only mode is reserved for alternate copied self-test fixtures"
fi

command -v bun >/dev/null 2>&1 || die "bun is required"
command -v git >/dev/null 2>&1 || die "git is required"
if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  command -v cargo >/dev/null 2>&1 || die "cargo is required"
fi

for name in \
  DATABASE_URL TEST_DATABASE_URL ANALYTICS_DATABASE_URL IDENTITY_DATABASE_URL \
  REDIS_URL REDIS_CLUSTER_URL OIDC_ISSUER OIDC_JWKS_URL JWKS_URL AUTH_JWKS_URL \
  AUTH_BASE_URL BACKEND_URL NEXT_PUBLIC_BACKEND_URL API_URL IDENTITY_GRPC_URL \
  IDENTITY_SSE_URL TRADINGVIEW_AUTH_TOKEN TRADINGVIEW_URL TRADINGVIEW_BASE_URL \
  TRADINGVIEW_WEBSOCKET_URL MARKET_DATA_URL MARKET_DATA_API_KEY LIVE_DATA_URL \
  RPC_URL CHAIN_RPC_URL BSC_RPC_URL BSC_MAINNET_RPC_URL BSC_TESTNET_RPC_URL \
  ETH_RPC_URL ETHEREUM_RPC_URL POLYGON_RPC_URL WEB3_PROVIDER_URL; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no live I/O"
done

for name in LIVE_DATA USE_LIVE_DATA ANALYTICS_LIVE_DATA RUN_LIVE_TESTS ENABLE_LIVE_TESTS ALLOW_NETWORK INDEXER_SYNC_ON_START SYNC_ON_START; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    1|true|yes|on|live|enabled) die "$name enables a live-data, network, or sync path" ;;
  esac
done

for name in EPSX_ENV APP_ENV ENV ENVIRONMENT NODE_ENV RUST_ENV DEPLOY_ENV DEPLOYMENT_ENV; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    prod|production|prod-*|production-*|*-prod|*-production)
      die "$name identifies a production-looking environment"
      ;;
  esac
done

export CARGO_NET_OFFLINE=true
export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

summary="$(bun -e '
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";

const [repoInput, evidenceInput, contractInput] = process.argv.slice(1);
const repo = realpathSync(repoInput);
const evidenceRoot = realpathSync(evidenceInput);
const fail = (message) => {
  console.error(`ranking-entitlement-snapshot: ERROR: ${message}`);
  process.exit(1);
};
const read = (path) => {
  try { return readFileSync(path, "utf8"); }
  catch (error) { fail(`cannot read ${path}: ${error.message}`); }
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", ...args], {
    cwd: repo,
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" },
  });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const safePath = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) fail(`${label}: unsafe path`);
  if (value.split("/").some((part) => !part || part === "." || part === "..")) fail(`${label}: unsafe path`);
  const candidate = resolve(evidenceRoot, value);
  let stat;
  try { stat = lstatSync(candidate); }
  catch { fail(`${label}: evidence file is missing`); }
  if (stat.isSymbolicLink()) fail(`${label}: symlinks are forbidden`);
  const actual = realpathSync(candidate);
  const rel = relative(evidenceRoot, actual);
  if (rel.startsWith("..") || isAbsolute(rel)) fail(`${label}: escaped evidence root`);
  return actual;
};
const contains = (content, value, label) => {
  if (!content.includes(value)) fail(`missing ${label}: ${value}`);
};
const excludes = (content, value, label) => {
  if (content.includes(value)) fail(`forbidden ${label}: ${value}`);
};

let contract;
try { contract = JSON.parse(read(contractInput)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-7-ranking-entitlement-snapshot") fail("unexpected contract schema/artifact");
if (contract.contractId !== "A2.7-ranking-entitlement-snapshot" || contract.purpose !== "deterministic-hermetic-ranking-entitlement-snapshot-and-readiness-stop") fail("unexpected contract identity/purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
const expectedSafety = { database: false, redis: false, identityNetwork: false, marketProvider: false, rpc: false, browser: false, serviceListener: false, deployment: false, production: false };
if (JSON.stringify(contract.safety) !== JSON.stringify(expectedSafety)) fail("all exact safety flags must remain false");

const expectedSource = {
  ref: "origin/development",
  commit: "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db",
  interpretation: "The monolith ranking query is compatibility authority for this pure resolver: active joined assignments, exclusive expiry, Free Plan seed and minimum valid ranking candidate. Conflicting grace, scheduling, direct-grant, token-claim and PlanFeatures behavior remains unresolved rather than invented.",
  evidence: [
    { id: "src-ranking-offset-query", file: "apps/backend/src/auth/unified_permission_service.rs", blob: "17becb76341ca3dfc87dedbc71e9653063a86f9e", anchor: "pub async fn get_wallet_ranking_offset(" },
    { id: "src-no-plan-free-success", file: "apps/backend/src/auth/unified_permission_service.rs", blob: "17becb76341ca3dfc87dedbc71e9653063a86f9e", anchor: "if rows.is_empty() {" },
    { id: "src-subscription-grace-conflict", file: "apps/backend/src/web/payments/subscription_handlers.rs", blob: "a051eb373cc94258cfe001cb66c90f3bbb9646d2", anchor: "OR (wga.expires_at + (g.grace_period_hours || '\'' hours'\'')::INTERVAL) > NOW())" },
    { id: "src-locked-rank-presentation", file: "apps/frontend/components/analytics/plan-status-bar.tsx", blob: "cdf7696cdb1290389653c3d520cb3b270cbbe8e1", anchor: "locked ranks = offset - 1" },
    { id: "src-core-assignment-schema", file: "apps/backend/migrations/core/00000000000001_consolidated_baseline_v6/up.sql", blob: "3cf683bc589f7252fd1be64ab62b69958fc292de", anchor: "CREATE TABLE wallet_plan_assignments (" },
    { id: "src-plan-features-alternative", file: "apps/backend/src/domain/subscription_management/value_objects/plan_features.rs", blob: "666c4f52d2d4121c1a59b15d631c79c61094cd5e", anchor: "ranking_offset: Option<i32>," },
  ],
};
if (JSON.stringify(contract.sourceBaseline) !== JSON.stringify(expectedSource)) fail("source baseline or evidence tuples drifted");
if (git("rev-parse", `${expectedSource.ref}^{commit}`) !== expectedSource.commit) fail("source ref is stale");
for (const item of expectedSource.evidence) {
  if (git("rev-parse", `${expectedSource.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale source blob`);
  contains(git("show", `${expectedSource.commit}:${item.file}`), item.anchor, `${item.id} source anchor`);
}

const expectedTarget = {
  ref: "migration/dioxus-microservices",
  commit: "395db722e2d71ff73a606d7eac14d6c4ef9d972d",
  interpretation: "Immutable completed post-A2.6 snapshot before adding the pure entitlement-snapshot resolver.",
  evidence: [
    { id: "base-identity-free-stub", file: "shared/rust/epsx-identity-service/src/identity_service.rs", blob: "a08419f241788274d1e07dd75089479ae3f15455", anchor: "Ok(RankingOffset::free_plan())" },
    { id: "base-runtime-free-wiring", file: "shared/rust/epsx-identity-service/src/main.rs", blob: "e05d9e320fdaed761cd7ea3aaccd2ed120a20143", anchor: "Arc::new(FreePlanRankingOffsetService)" },
    { id: "base-strict-ranking-offset", file: "shared/rust/epsx-contracts/src/value_objects/ranking_offset.rs", blob: "eed39fba8e1fdbe8aa2e8f920ace95eff16953cb", anchor: "pub const RANKING_OFFSET_MAX: i32 = 1000;" },
    { id: "base-a2-6-authority-contract", file: "docs/migration/contracts/a2-6-ranking-authority-failure-boundary.json", blob: "19410d9a46a4e50a73c404be5ba8a80e13553534", anchor: "\"contractId\": \"A2.6-ranking-authority-failure-boundary\"" },
  ],
};
if (JSON.stringify(contract.targetBase) !== JSON.stringify(expectedTarget)) fail("target base or evidence tuples drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("target base commit is missing");
for (const item of expectedTarget.evidence) {
  if (git("rev-parse", `${expectedTarget.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale target-base blob`);
  contains(git("show", `${expectedTarget.commit}:${item.file}`), item.anchor, `${item.id} target-base anchor`);
}

const expectedInvariantIds = [
  "fixed-observed-at", "active-assignment-present-active-plan",
  "exclusive-expiry-or-permanent", "strict-relevant-candidate-validation",
  "inactive-and-unrelated-permissions-ignored", "minimum-seeded-free",
  "typed-success-provenance", "order-and-duplicate-invariance",
  "errors-distinct-from-free", "runtime-always-free-remains-unwired",
  "no-scheduling-or-grace-invention", "offline-non-production",
];
if (JSON.stringify(contract.invariants.map((item) => item.id)) !== JSON.stringify(expectedInvariantIds)) fail("invariant inventory drifted");
for (const item of contract.invariants) {
  if (typeof item.claim !== "string" || item.claim.length < 40 || /production ready|deployment authorized/i.test(item.claim)) fail(`${item.id}: invalid invariant meaning`);
}

const expectedStops = [
  "core-owned-adapter-absent", "atomic-sql-snapshot-unproved",
  "schema-adoption-reconciliation-unproved", "identity-runtime-still-always-free",
  "scheduling-grace-policy-unresolved", "alternate-entitlement-sources-unresolved",
  "identity-workload-auth-tls-absent", "ranking-event-durability-absent",
  "ui-bff-readiness-unproved", "live-parity-observability-unproved",
  "route-owner-cutover-unproved", "production-actions-unauthorized",
];
if (JSON.stringify(contract.residualStops.map((item) => item.id)) !== JSON.stringify(expectedStops)) fail("residual STOP inventory drifted");
for (const item of contract.residualStops) if (typeof item.claim !== "string" || item.claim.length < 50) fail(`${item.id}: residual STOP meaning is incomplete`);

const expectedOrderPrefixes = ["E01 ", "E02 ", "E03 ", "E04 ", "E05 ", "E06 ", "E07 ", "E08 ", "E09 "];
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== expectedOrderPrefixes.length) fail("execution order drifted");
contract.requiredExecutionOrder.forEach((item, index) => {
  if (typeof item !== "string" || !item.startsWith(expectedOrderPrefixes[index]) || (index < 8 && /deploy first/i.test(item))) fail(`invalid execution step E${String(index + 1).padStart(2, "0")}`);
});

const fixture = contract.fixtureEvidence;
if (!fixture || fixture.file !== "docs/migration/fixtures/a2-7-ranking-entitlement-snapshot.json") fail("fixture evidence path drifted");
if (!/^[0-9a-f]{64}$/.test(fixture.sha256) || !Number.isInteger(fixture.fixtureCount) || fixture.fixtureCount < 1) fail("fixture evidence is not frozen");
if (!Array.isArray(fixture.fixtureIds) || fixture.fixtureIds.length !== fixture.fixtureCount || new Set(fixture.fixtureIds).size !== fixture.fixtureCount) fail("fixture inventory drifted");
const fixturePath = safePath(fixture.file, "fixture-ledger");
const fixtureContent = read(fixturePath);
if (createHash("sha256").update(fixtureContent).digest("hex") !== fixture.sha256) fail("fixture ledger digest drifted");
let fixtureLedger;
try { fixtureLedger = JSON.parse(fixtureContent); }
catch (error) { fail(`invalid fixture ledger JSON: ${error.message}`); }
if (fixtureLedger.schemaVersion !== 1 || !Array.isArray(fixtureLedger.cases) || JSON.stringify(fixtureLedger.cases.map((item) => item.id)) !== JSON.stringify(fixture.fixtureIds)) fail("fixture ledger IDs drifted");

const expectedTests = [
  "a2_7_fixture_ledger_matches_pure_decisions",
  "a2_7_query_is_dyn_compatible",
  "a2_7_query_normalizes_wallet_and_calls_repository_once",
  "a2_7_repository_corrupt_maps_to_opaque_internal_error",
  "a2_7_repository_unavailable_maps_to_opaque_service_unavailable",
  "a2_7_resolution_corruption_maps_to_opaque_internal_error",
  "a2_7_snapshot_wallet_mismatch_fails_opaquely",
];
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("hermetic test inventory drifted");

const expectedImplementation = [
  ["impl-pure-ranking-entitlement-resolver", "shared/rust/epsx-identity-service/src/ranking_entitlement.rs", "3c42711783a14f6ff6d7ebeb813a03fa556aa466e3f15b08895ae59faef68bb8"],
  ["impl-resolver-library-export", "shared/rust/epsx-identity-service/src/lib.rs", "4853eddb05a16f39968d2b812ddbb61ba872d5c407be51bd65524d8a78a6d208"],
  ["impl-unwired-runtime-main", "shared/rust/epsx-identity-service/src/main.rs", "2508a73e31b65556970dab3a3a97d71d7a427d8d3f3db7ebb8b39dd71a643e1e"],
  ["impl-always-free-runtime-service", "shared/rust/epsx-identity-service/src/identity_service.rs", "f172df2a9998cc773b4299e2760164ee9d9a6c225680260b6c17bdc27f8da320"],
];
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("implementation evidence inventory drifted");
const contentByFile = new Map();
for (let index = 0; index < expectedImplementation.length; index += 1) {
  const [id, file, sha256] = expectedImplementation[index];
  const item = contract.implementationEvidence[index];
  if (JSON.stringify(item) !== JSON.stringify({ id, file, sha256 })) fail(`${id}: implementation tuple drifted`);
  if (!/^[0-9a-f]{64}$/.test(sha256)) fail(`${id}: implementation digest is not frozen`);
  const path = safePath(file, id);
  const content = read(path);
  if (createHash("sha256").update(content).digest("hex") !== sha256) fail(`${id}: implementation digest drifted`);
  contentByFile.set(file, content);
}

if (expectedImplementation.length > 0) {
  const resolverEntry = [...contentByFile.entries()].find(([file]) => file.endsWith("epsx-identity-service/src/ranking_entitlement.rs"));
  const libEntry = [...contentByFile.entries()].find(([file]) => file.endsWith("epsx-identity-service/src/lib.rs"));
  const mainEntry = [...contentByFile.entries()].find(([file]) => file.endsWith("epsx-identity-service/src/main.rs"));
  const serviceEntry = [...contentByFile.entries()].find(([file]) => file.endsWith("epsx-identity-service/src/identity_service.rs"));
  if (!resolverEntry || !libEntry || !mainEntry || !serviceEntry) fail("resolver, lib export and unwired runtime evidence are required");
  const resolver = resolverEntry[1];
  const lib = libEntry[1];
  const main = mainEntry[1];
  const service = serviceEntry[1];
  contains(resolver, "observed_at", "fixed snapshot instant");
  contains(resolver, "Unix epoch microseconds", "snapshot timestamp unit");
  contains(resolver, "RankingOffset::new", "strict ranking candidate validation");
  contains(resolver, "NoEffectivePlan", "typed no-plan outcome");
  contains(resolver, "EffectivePlansWithoutGrant", "typed no-grant outcome");
  contains(resolver, "PlanGrant", "typed plan-grant outcome");
  excludes(resolver, "assigned_at", "invented scheduled-start policy");
  excludes(resolver, "grace_period", "invented grace policy");
  excludes(resolver, "Utc::now", "process clock access");
  excludes(resolver, "SystemTime::now", "process clock access");
  excludes(resolver, "diesel", "database access");
  excludes(resolver, "sqlx", "database access");
  contains(lib, "pub mod ranking_entitlement;", "resolver library export");
  contains(main, "Arc::new(FreePlanRankingOffsetService)", "always-Free runtime wiring");
  contains(service, "Ok(RankingOffset::free_plan())", "always-Free identity service");
  for (const name of expectedTests) if (!resolver.includes(name)) fail(`missing hermetic test source: ${name}`);
}

process.stdout.write(JSON.stringify({
  artifact: contract.artifact,
  productionReady: contract.productionReady,
  readinessExit: contract.readinessExit,
  invariants: contract.invariants.length,
  fixtures: fixture.fixtureCount,
  hermeticTests: contract.hermeticTests.length,
  implementationEvidence: contract.implementationEvidence.length,
  residualStops: contract.residualStops,
}));
' "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT")" || exit 1

if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  test_list="$(cargo test --offline --locked -p epsx-identity-service --lib -- --list 2>&1)" || {
    printf '%s\n' "$test_list" >&2
    die "could not enumerate identity hermetic tests"
  }
  while IFS= read -r test_name; do
    [[ -n "$test_name" ]] || continue
    match="$(printf '%s\n' "$test_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    match_count="$(printf '%s\n' "$match" | sed '/^$/d' | wc -l | tr -d ' ')"
    [[ "$match_count" == "1" ]] || die "hermetic test name is missing or ambiguous: $test_name"
    output="$(cargo test --offline --locked -p epsx-identity-service --lib "$match" -- --exact 2>&1)" || {
      printf '%s\n' "$output" >&2
      die "hermetic test failed: $test_name"
    }
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" || die "hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  check_output="$(cargo check --offline --locked -p epsx-identity-service --lib 2>&1)" || {
    printf '%s\n' "$check_output" >&2
    die "epsx-identity-service offline library check failed"
  }
fi

invariants="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.invariants.length));' "$CONTRACT")"
fixtures="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.fixtureEvidence.fixtureCount));' "$CONTRACT")"
tests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.hermeticTests.length));' "$CONTRACT")"
digests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.implementationEvidence.length + 1));' "$CONTRACT")"
stops="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.residualStops.length));' "$CONTRACT")"

case "$MODE" in
  integrity)
    printf 'ranking-entitlement-snapshot: PASS; %s invariants; %s fixtures; %s hermetic tests; %s frozen digests; %s residual STOPs\n' "$invariants" "$fixtures" "$tests" "$digests" "$stops"
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    printf 'ranking-entitlement-snapshot: LIMIT; %s residual STOPs remain; readiness exit 3\n' "$stops" >&2
    exit 3
    ;;
esac
