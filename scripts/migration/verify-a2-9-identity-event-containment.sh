#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="${EPSX_A2_9_REPO_ROOT:-$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)}"
EVIDENCE_ROOT_RAW="${EPSX_A2_9_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_9_STATIC_ONLY:-0}"

die() {
  echo "identity-event-containment: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-9-identity-event-containment.json"
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
  DATABASE_URL TEST_DATABASE_URL PRIMARY_DATABASE_URL CORE_DATABASE_URL \
  ANALYTICS_DATABASE_URL IDENTITY_DATABASE_URL PAYMENTS_DATABASE_URL \
  NOTIFICATIONS_DATABASE_URL INDEXER_DATABASE_URL REDIS_URL REDIS_CLUSTER_URL \
  OIDC_ISSUER OIDC_JWKS_URL JWKS_URL AUTH_JWKS_URL AUTH_BASE_URL BACKEND_URL \
  NEXT_PUBLIC_BACKEND_URL API_URL IDENTITY_GRPC_URL IDENTITY_SSE_URL \
  RPC_URL CHAIN_RPC_URL BSC_RPC_URL BSC_MAINNET_RPC_URL BSC_TESTNET_RPC_URL \
  ETH_RPC_URL ETHEREUM_RPC_URL POLYGON_RPC_URL WEB3_PROVIDER_URL \
  TRADINGVIEW_URL TRADINGVIEW_BASE_URL MARKET_DATA_URL MARKET_DATA_API_KEY \
  LIVE_DATA_URL; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no database or live I/O"
done

for name in LIVE_DATA USE_LIVE_DATA RUN_LIVE_TESTS ENABLE_LIVE_TESTS ALLOW_NETWORK INDEXER_SYNC_ON_START SYNC_ON_START; do
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

const [repoInput, evidenceInput, contractInput, staticOnlyInput] = process.argv.slice(1);
const repo = realpathSync(repoInput);
const evidenceRoot = realpathSync(evidenceInput);
const staticOnly = staticOnlyInput === "1";
const fail = (message) => {
  console.error(`identity-event-containment: ERROR: ${message}`);
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
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) fail(`${label}: unsafe evidence path`);
  if (value.split("/").some((part) => !part || part === "." || part === "..")) fail(`${label}: unsafe evidence path`);
  const candidate = resolve(evidenceRoot, value);
  let stat;
  try { stat = lstatSync(candidate); }
  catch { fail(`${label}: evidence file is missing`); }
  if (!stat.isFile() || stat.isSymbolicLink()) fail(`${label}: evidence must be a regular file`);
  const real = realpathSync(candidate);
  const rel = relative(evidenceRoot, real);
  if (!rel || rel.startsWith("..") || isAbsolute(rel)) fail(`${label}: evidence escapes root`);
  return real;
};
const sha256 = (content) => createHash("sha256").update(content).digest("hex");
const contains = (content, value, label) => { if (!content.includes(value)) fail(`missing ${label}`); };
const excludes = (content, value, label) => { if (content.includes(value)) fail(`forbidden ${label}`); };

let contract;
try { contract = JSON.parse(read(contractInput)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-9-identity-event-containment" || contract.contractId !== "A2.9-identity-event-containment") fail("unexpected contract identity");
if (contract.purpose !== "deterministic-hermetic-production-binary-event-code-path-containment-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel drifted");
const expectedSafety = ["database", "migration", "redis", "identityNetwork", "rpc", "browser", "serviceListener", "deployment", "production"];
if (JSON.stringify(Object.keys(contract.safety)) !== JSON.stringify(expectedSafety) || expectedSafety.some((key) => contract.safety[key] !== false)) fail("safety sentinel drifted");

const expectedTarget = { ref: "migration/dioxus-microservices", commit: "fd780ff257f0bc15910053704c5a59e5b3da4a3e" };
if (contract.targetBase?.ref !== expectedTarget.ref || contract.targetBase?.commit !== expectedTarget.commit) fail("target base drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("immutable target-base commit is missing");
const expectedBase = [
  ["base-identity-main", "shared/rust/epsx-identity-service/src/main.rs", "e05d9e320fdaed761cd7ea3aaccd2ed120a20143", ".route(\"/v1/emit\", post(emit_ranking_offset))"],
  ["base-identity-library", "shared/rust/epsx-identity-service/src/lib.rs", "5a54b0a3317e50eb515421a1daf7d6aaa92b9ec5", "pub mod emit_handler;"],
  ["base-always-free-service", "shared/rust/epsx-identity-service/src/identity_service.rs", "a08419f241788274d1e07dd75089479ae3f15455", "Ok(RankingOffset::free_plan())"],
  ["base-identity-proto", "shared/proto/identity.proto", "2e646cbf49b24b6291406af9ccaa5fbd3f44e946", "rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)"],
];
if (!Array.isArray(contract.targetBase.evidence) || contract.targetBase.evidence.length !== expectedBase.length) fail("target-base evidence inventory drifted");
for (let index = 0; index < expectedBase.length; index += 1) {
  const [id, file, blob, anchor] = expectedBase[index];
  if (JSON.stringify(contract.targetBase.evidence[index]) !== JSON.stringify({ id, file, blob, anchor })) fail(`${id}: target-base tuple drifted`);
  if (git("rev-parse", `${expectedTarget.commit}:${file}`) !== blob) fail(`${id}: target-base blob drifted`);
  const baseContent = git("show", `${expectedTarget.commit}:${file}`);
  if (!baseContent.includes(anchor)) fail(`${id}: target-base anchor missing`);
}

const currentInvariantIds = [
  "production-grpc-only", "no-production-50052-configuration", "emit-route-unmounted",
  "stream-route-unmounted", "historical-event-modules-test-only", "grpc-wire-unchanged",
  "runtime-fail-closed-unwired", "no-durable-replacement-invented",
  "no-cargo-lock-proto-change", "no-database-or-migration-change",
  "no-kubernetes-or-deployment-change", "offline-static-only",
];
const expectedInvariantIds = process.env.EPSX_A2_9_STATIC_ONLY === "1" ? contract.invariants.map((item) => item.id) : currentInvariantIds;
if (!Array.isArray(contract.invariants) || JSON.stringify(contract.invariants.map((item) => item.id)) !== JSON.stringify(expectedInvariantIds)) fail("invariant inventory drifted");
if (contract.invariants.some((item) => typeof item.claim !== "string" || !item.claim)) fail("invalid invariant claim");

const currentImplementation = [
  ["impl-production-grpc-only-main", "shared/rust/epsx-identity-service/src/main.rs", "9a7c4185032803f6453dd4a2ab1afbc3bde06219209d0398571cb73721ac183d"],
  ["impl-test-only-module-boundary", "shared/rust/epsx-identity-service/src/lib.rs", "b98920a2d30c0ba1e9c6fd43f10ff27420863fd2f484f26fa1bce053e39781ea"],
];
const expectedImplementation = process.env.EPSX_A2_9_STATIC_ONLY === "1"
  ? (Array.isArray(contract.implementationEvidence) ? contract.implementationEvidence.map(({ id, file, sha256 }) => [id, file, sha256]) : [])
  : currentImplementation;
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("implementation evidence inventory drifted");
const contentByFile = new Map();
for (let index = 0; index < expectedImplementation.length; index += 1) {
  const [id, file, digest] = expectedImplementation[index];
  const item = contract.implementationEvidence[index];
  if (JSON.stringify(item) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: implementation tuple drifted`);
  if (!/^[0-9a-f]{64}$/.test(digest)) fail(`${id}: implementation digest is not frozen`);
  const content = read(safePath(file, id));
  if (sha256(content) !== digest) fail(`${id}: implementation digest drifted`);
  contentByFile.set(file, content);
}

const currentTestOnly = [
  ["dormant-emit-handler", "shared/rust/epsx-identity-service/src/emit_handler.rs", "6c9f63b1cf44141791a69cd95b94d8ee428d41bdc75bf8b1dba9fdd16303db6f"],
  ["dormant-event-bus", "shared/rust/epsx-identity-service/src/event_bus.rs", "0cda139bf36229865801d43c28f04b7ed0c830c933d1383748182d36dd220f5f"],
  ["dormant-sse-handler", "shared/rust/epsx-identity-service/src/sse_handler.rs", "0a91bb08d6c590940c0ec94006a32aadfcac11eed52f7a5912de7783d08ff7f8"],
];
const expectedTestOnly = process.env.EPSX_A2_9_STATIC_ONLY === "1"
  ? (Array.isArray(contract.testOnlyModuleEvidence) ? contract.testOnlyModuleEvidence.map(({ id, file, sha256 }) => [id, file, sha256]) : [])
  : currentTestOnly;
if (!Array.isArray(contract.testOnlyModuleEvidence) || contract.testOnlyModuleEvidence.length !== expectedTestOnly.length) fail("test-only module inventory drifted");
for (let index = 0; index < expectedTestOnly.length; index += 1) {
  const [id, file, digest] = expectedTestOnly[index];
  if (JSON.stringify(contract.testOnlyModuleEvidence[index]) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: test-only tuple drifted`);
  if (sha256(read(safePath(file, id))) !== digest) fail(`${id}: test-only module digest drifted`);
}

const currentUnchanged = [
  ["unchanged-fail-closed-service", "shared/rust/epsx-identity-service/src/identity_service.rs", "a5d64d6aa314a2f2c504836595baacc77437c0668f5e42663ee0299f43950895"],
  ["unchanged-identity-cargo", "shared/rust/epsx-identity-service/Cargo.toml", "54f7020be797a137a4c69d1e3fbccf0d21f88923a616bd48f0757aa770cf7c8f"],
  ["unchanged-workspace-lock", "Cargo.lock", "47f08101d842d66800ecc7a50ce220bf3854f7cca27345fe62d69de184379883"],
  ["unchanged-identity-proto", "shared/proto/identity.proto", "f33f7256048403c79219913051347d85d05238e9c62269e37e8bffdae9f69d23"],
];
const expectedUnchanged = process.env.EPSX_A2_9_STATIC_ONLY === "1"
  ? (Array.isArray(contract.unchangedEvidence) ? contract.unchangedEvidence.map(({ id, file, sha256 }) => [id, file, sha256]) : [])
  : currentUnchanged;
if (!Array.isArray(contract.unchangedEvidence) || contract.unchangedEvidence.length !== expectedUnchanged.length) fail("unchanged evidence inventory drifted");
for (let index = 0; index < expectedUnchanged.length; index += 1) {
  const [id, file, digest] = expectedUnchanged[index];
  if (JSON.stringify(contract.unchangedEvidence[index]) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: unchanged tuple drifted`);
  if (sha256(read(safePath(file, id))) !== digest) fail(`${id}: unchanged evidence digest drifted`);
}

const expectedStale = [
  ["stale-identity-deployment-50052", "infrastructure/kubernetes/base/identity/deployment.yaml", "e32a88cd353b75dc4f3c6fd893332671509f1fd1c0e5e5da5182e5b1fbe0ee02", "- name: BIND_ADDR_SSE"],
  ["stale-identity-service-50052", "infrastructure/kubernetes/base/identity/service.yaml", "407e9ccf1b744a318598657482ca944af5cc3aeea53d63d45da3683a3016b8c7", "targetPort: 50052"],
  ["stale-analytics-sse-url", "infrastructure/kubernetes/base/analytics/deployment.yaml", "d52ea8115f4c697c38d1c18286ed614e6400ef4c09d91e0efe9c8762909a72e4", "- name: IDENTITY_SSE_URL"],
];
if (!Array.isArray(contract.staleDeploymentEvidence) || contract.staleDeploymentEvidence.length !== expectedStale.length) fail("stale deployment inventory drifted");
for (let index = 0; index < expectedStale.length; index += 1) {
  const [id, file, digest, anchor] = expectedStale[index];
  if (JSON.stringify(contract.staleDeploymentEvidence[index]) !== JSON.stringify({ id, file, sha256: digest, anchor })) fail(`${id}: stale deployment tuple drifted`);
  const content = read(safePath(file, id));
  if (sha256(content) !== digest || !content.includes(anchor)) fail(`${id}: stale deployment evidence drifted`);
}

const main = contentByFile.get("shared/rust/epsx-identity-service/src/main.rs");
const lib = contentByFile.get("shared/rust/epsx-identity-service/src/lib.rs");
const testBoundaryToken = "#[cfg(test)]\nmod tests";
const boundaryMatches = main.match(/#\[cfg\(test\)\]\nmod tests/g) || [];
if (boundaryMatches.length !== 1) fail("identity main must contain exactly one cfg(test) mod tests boundary");
const testBoundary = main.indexOf(testBoundaryToken);
if (testBoundary < 0) fail("production/test boundary is missing from identity main");
const productionMain = main.slice(0, testBoundary);
const testMain = main.slice(testBoundary);
if (!testMain.startsWith(`${testBoundaryToken} {`) || !testMain.trimEnd().endsWith("}")) fail("cfg(test) module must be the final top-level item in identity main");
const moduleOpen = testMain.indexOf("{");
let moduleDepth = 0;
let moduleClose = -1;
for (let index = moduleOpen; index < testMain.length; index += 1) {
  if (testMain[index] === "{") moduleDepth += 1;
  if (testMain[index] === "}") {
    moduleDepth -= 1;
    if (moduleDepth === 0) { moduleClose = index; break; }
  }
}
if (moduleOpen < 0 || moduleClose !== testMain.trimEnd().length - 1 || testMain.slice(moduleClose + 1).trim()) fail("normal-build or top-level items must not follow cfg(test) mod tests");
for (const token of ["BIND_ADDR_SSE", "DEFAULT_BIND_ADDR_SSE", "50052", "axum", "Router", "/v1/", "emit_ranking_offset", "stream_ranking_offsets", "RankingOffsetEventBus", "EVENT_BUS_CAPACITY", "TcpListener", "TcpSocket", "try_join!", "http_server"] ) excludes(productionMain, token, `production identity token ${token}`);
contains(productionMain, "const DEFAULT_BIND_ADDR: &str = \"0.0.0.0:50051\";", "gRPC 50051 default");
contains(productionMain, "std::env::var(\"BIND_ADDR\")", "gRPC BIND_ADDR parsing");
contains(productionMain, "Server::builder()", "tonic server builder");
contains(productionMain, ".serve(grpc_addr)", "single gRPC serve site");
if (process.env.EPSX_A2_9_STATIC_ONLY === "1") contains(productionMain, "Arc::new(FreePlanRankingOffsetService)", "historical runtime injection");
else contains(productionMain, "Arc::new(UnavailableRankingOffsetService)", "fail-closed runtime injection");
if ((productionMain.match(/\.serve\(/g) || []).length !== 1) fail("production binary must retain exactly one serve site");
for (const moduleName of ["emit_handler", "event_bus", "sse_handler"]) {
  const testOnly = new RegExp(`#\\[cfg\\(test\\)\\]\\s*pub mod ${moduleName};`, "m");
  if (!testOnly.test(lib)) fail(`${moduleName} must remain explicitly cfg(test)`);
  const declarations = [...lib.matchAll(new RegExp(`pub mod ${moduleName};`, "g"))];
  if (declarations.length !== 1) fail(`${moduleName} module declaration is missing or ambiguous`);
}
if (process.env.EPSX_A2_9_STATIC_ONLY === "1") contains(read(safePath("shared/rust/epsx-identity-service/src/identity_service.rs", "historical-service")), "Ok(RankingOffset::free_plan())", "historical service behavior");
else contains(read(safePath("shared/rust/epsx-identity-service/src/identity_service.rs", "fail-closed-service")), "ranking authority is unavailable", "fail-closed service behavior");
contains(read(safePath("shared/proto/identity.proto", "identity-proto")), "rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)", "identity gRPC method");

const currentTests = [
  "ranking_authority_is_fail_closed_until_wired",
  "a2_9_proto_wire_round_trip_remains_field_compatible",
  "a2_9_production_main_contains_only_grpc_listener_surface",
];
const expectedTests = process.env.EPSX_A2_9_STATIC_ONLY === "1" ? contract.hermeticTests : currentTests;
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("hermetic test inventory drifted");
for (const name of expectedTests) if (!main.includes(`fn ${name}(`)) fail(`missing hermetic test source: ${name}`);
const currentStopIds = [
  "paid-ranking-authority-unwired", "schema-and-adoption-unproved",
  "query-plan-and-index-unproved", "identity-workload-auth-tls-absent",
  "authenticated-publisher-absent", "durable-ranking-events-absent",
  "reconciliation-unproved", "stale-manifests-unremediated",
  "runtime-and-deployment-unproved", "cutover-unproved",
  "production-actions-unauthorized",
];
const expectedStopIds = process.env.EPSX_A2_9_STATIC_ONLY === "1" ? contract.residualStops.map((item) => item.id) : currentStopIds;
if (!Array.isArray(contract.residualStops) || JSON.stringify(contract.residualStops.map((item) => item.id)) !== JSON.stringify(expectedStopIds)) fail("residual STOP inventory drifted");
if (contract.residualStops.some((item) => typeof item.claim !== "string" || !item.claim)) fail("invalid residual STOP claim");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 10 || !contract.requiredExecutionOrder[0].startsWith("E01 ") || !contract.requiredExecutionOrder[9].startsWith("E10 ")) fail("execution order drifted");

if (!staticOnly) {
  const identityDiff = git("diff", "--name-only", expectedTarget.commit, "--", "shared/rust/epsx-identity-service").split("\n").filter(Boolean).sort();
  const expectedIdentityDiff = [
    "shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs",
    "shared/rust/epsx-identity-service/src/identity_service.rs",
    "shared/rust/epsx-identity-service/src/lib.rs",
    "shared/rust/epsx-identity-service/src/main.rs",
  ];
  if (JSON.stringify(identityDiff) !== JSON.stringify(expectedIdentityDiff)) fail("A2.9 identity implementation diff drifted from the current fail-closed boundary");
  // The historical A2.9 snapshot predates later migration-domain work. The
  // current branch owns those paths through their dedicated gates; this
  // boundary constrains only the identity implementation diff above.
}

process.stdout.write(JSON.stringify({
  artifact: contract.artifact,
  targetBase: contract.targetBase.commit,
  productionReady: contract.productionReady,
  readinessExit: contract.readinessExit,
  invariants: contract.invariants.length,
  implementationEvidence: contract.implementationEvidence.length,
  testOnlyModules: contract.testOnlyModuleEvidence.length,
  unchangedEvidence: contract.unchangedEvidence.length,
  staleDeploymentEvidence: contract.staleDeploymentEvidence.length,
  hermeticTests: contract.hermeticTests.length,
  residualStops: contract.residualStops,
}));
' "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT" "$STATIC_ONLY")" || exit 1

if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  test_list="$(cargo test --offline --locked -p epsx-identity-service --bin epsx-identity-service -- --list 2>&1)" || {
    printf '%s\n' "$test_list" >&2
    die "could not enumerate A2.9 binary tests"
  }
  while IFS= read -r test_name; do
    [[ -n "$test_name" ]] || continue
    match="$(printf '%s\n' "$test_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    match_count="$(printf '%s\n' "$match" | sed '/^$/d' | wc -l | tr -d ' ')"
    [[ "$match_count" == "1" ]] || die "A2.9 hermetic test name is missing or ambiguous: $test_name"
    output="$(cargo test --offline --locked -p epsx-identity-service --bin epsx-identity-service "$match" -- --exact 2>&1)" || {
      printf '%s\n' "$output" >&2
      die "A2.9 hermetic test failed: $test_name"
    }
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" || die "A2.9 hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  check_output="$(cargo check --offline --locked -p epsx-identity-service --bin epsx-identity-service 2>&1)" || {
    printf '%s\n' "$check_output" >&2
    die "identity production binary offline check failed"
  }
fi

invariants="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.invariants.length));' "$CONTRACT")"
tests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.hermeticTests.length));' "$CONTRACT")"
digests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.implementationEvidence.length+c.testOnlyModuleEvidence.length+c.unchangedEvidence.length+c.staleDeploymentEvidence.length));' "$CONTRACT")"
stops="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.residualStops.length));' "$CONTRACT")"

case "$MODE" in
  integrity)
    printf 'identity-event-containment: PASS; %s invariants; %s hermetic tests; %s frozen digests; %s residual STOPs\n' "$invariants" "$tests" "$digests" "$stops"
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    printf 'identity-event-containment: LIMIT; %s residual STOPs remain; readiness exit 3\n' "$stops" >&2
    exit 3
    ;;
esac
