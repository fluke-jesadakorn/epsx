#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT_RAW="${EPSX_A2_6_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_6_STATIC_ONLY:-0}"

die() {
  echo "ranking-authority-failure-boundary: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-6-ranking-authority-failure-boundary.json"
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
  console.error(`ranking-authority-failure-boundary: ERROR: ${message}`);
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
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-6-ranking-authority-failure-boundary") fail("unexpected contract schema/artifact");
if (contract.contractId !== "A2.6-ranking-authority-failure-boundary" || contract.purpose !== "deterministic-hermetic-fail-closed-authority-boundary-and-readiness-stop") fail("unexpected contract identity/purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
const expectedSafety = { database: false, redis: false, identityNetwork: false, marketProvider: false, rpc: false, browser: false, serviceListener: false, deployment: false, production: false };
if (JSON.stringify(contract.safety) !== JSON.stringify(expectedSafety)) fail("all exact safety flags must remain false");

const expectedSource = {
  ref: "origin/development",
  commit: "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db",
  interpretation: "The monolith plan query and authenticated free-tier downgrade are compatibility evidence and findings; an authority failure is not an entitlement decision and is not preserved as a production target.",
  evidence: [
    { id: "src-plan-ranking-query", file: "apps/backend/src/auth/unified_permission_service.rs", blob: "17becb76341ca3dfc87dedbc71e9653063a86f9e", anchor: "pub async fn get_wallet_ranking_offset(" },
    { id: "src-authenticated-free-downgrade", file: "apps/backend/src/web/analytics/eps/cache.rs", blob: "13f8c39b24a248186d6c24f3298023ba678bc210", anchor: "warn!(\"Analytics API: Failed to get offset for {}: {}, using free tier\", wallet, e);" },
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
  commit: "a7f7ed0c0d0d3b07cb43414b1e3cd2a5f64bd5d1",
  interpretation: "Immutable post-A2.5 snapshot before lazy identity transport and fail-closed authenticated authority handling.",
  evidence: [
    { id: "base-eager-fallback-client", file: "apps/analytics/src/grpc_client.rs", blob: "67d4ebf964cc23f5dcbe1dc7dcdbc6596f84bbc8" },
    { id: "base-fallback-runtime-wiring", file: "apps/analytics/src/main.rs", blob: "e99af6fdfb5d3eaaab98cce96f57e3d2cd96de5f" },
    { id: "base-authority-downgrade-handler", file: "apps/backend/src/web/analytics/eps/cache.rs", blob: "0cf30cb873a568cd33f16e47af8f997d09946c8c" },
    { id: "base-a2-5-provider-contract", file: "docs/migration/contracts/a2-5-market-provider-boundary.json", blob: "15b753ffb6df99489d1a3ece739ed121065b6388" },
  ],
};
if (JSON.stringify(contract.targetBase) !== JSON.stringify(expectedTarget)) fail("target base or evidence tuples drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("target base commit is missing");
for (const item of expectedTarget.evidence) {
  if (git("rev-parse", `${expectedTarget.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale target-base blob`);
}

const expectedInvariantIds = [
  "anonymous-free-without-authority", "authenticated-authority-required",
  "authority-denial-before-provider", "lazy-client-construction",
  "single-attempt-single-deadline", "no-free-fallback", "strict-wire-offset",
  "opaque-authority-failure", "successful-offset-preserved", "non-production-evidence",
];
if (JSON.stringify(contract.invariants.map((item) => item.id)) !== JSON.stringify(expectedInvariantIds)) fail("invariant inventory drifted");
for (const item of contract.invariants) {
  if (typeof item.claim !== "string" || item.claim.length < 40 || /production ready|deployment authorized/i.test(item.claim)) fail(`${item.id}: invalid invariant meaning`);
}
const expectedTests = [
  "a2_6_grpc_success_returns_strictly_validated_offset_once",
  "a2_6_grpc_status_is_opaque_without_fallback_or_retry",
  "a2_6_grpc_timeout_is_opaque_without_fallback_or_retry",
  "a2_6_invalid_wire_offset_is_rejected_not_clamped",
  "a2_6_lazy_constructor_accepts_unreachable_uri_without_dialing",
  "a2_6_constructor_rejects_malformed_uri_opaquely",
  "a2_6_anonymous_bypasses_authority_and_keeps_free_input",
  "a2_6_authenticated_authoritative_offset_proceeds",
  "a2_6_authenticated_no_plan_is_explicit_free_success",
  "a2_6_authenticated_authority_errors_stop_before_provider_work",
];
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("hermetic test inventory drifted");

const expectedImplementation = [
  ["impl-fail-closed-grpc-client", "apps/analytics/src/grpc_client.rs", "bafac48faf1e9d03d1990ba57f922a01a350ff7763bed7ede2e70917a2c7a559"],
  ["impl-lazy-runtime-wiring", "apps/analytics/src/main.rs", "76daa6108d37f2c09b76109437e271f3393f9917cef631ccaed198c17858fda4"],
  ["impl-fail-closed-ranking-handler", "apps/backend/src/web/analytics/eps/cache.rs", "282b0be9a63d8e25cfd84970e90da03ea6db1f3a0582f0cf2659c67eaf7b57c6"],
];
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("three implementation records are required");
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

const expectedStops = [
  ["identity-success-always-free", "The identity gRPC server still returns the Free Plan offset for every successful request, so paid entitlement success is not authoritative."],
  ["identity-db-authority-absent", "The identity service has no reviewed database repository, atomic plan snapshot, schema probe, assignment reconciliation or live plan authority."],
  ["identity-workload-auth-tls-absent", "The internal identity RPC has no authenticated workload identity, owner binding, transport TLS or authorization policy."],
  ["identity-sse-emit-unauthenticated", "Identity ranking SSE and emit remain unauthenticated, global, ephemeral and disconnected from transactional entitlement changes."],
  ["source-parity-shadow-unproved", "Free, paid, expired, overlapping, revoked and admin assignment parity has no reconciled fixture ledger or source-shadow evidence."],
  ["route-owner-runtime-cutover-unproved", "The monolith remains canonical and no gateway, Cloudflare, Kubernetes, image, configuration, canary or rollback cutover is authorized."],
  ["live-deployment-evidence-absent", "No database, RPC, service listener, browser, staging, load, observability, deployment or rollback evidence exists for A2.6."],
  ["production-actions-unauthorized", "Passing A2.6 integrity is not production readiness and never authorizes a live service, secret, database, provider or deployment action."],
];
if (JSON.stringify(contract.residualStops.map((item) => [item.id, item.claim])) !== JSON.stringify(expectedStops)) fail("residual STOP inventory or meaning drifted");

const expectedOrderPrefixes = ["E01 ", "E02 ", "E03 ", "E04 ", "E05 ", "E06 ", "E07 ", "E08 "];
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== expectedOrderPrefixes.length) fail("execution order drifted");
contract.requiredExecutionOrder.forEach((item, index) => {
  if (typeof item !== "string" || !item.startsWith(expectedOrderPrefixes[index]) || (index < 7 && /deploy first/i.test(item))) fail(`invalid execution step E0${index + 1}`);
});

const grpc = contentByFile.get("apps/analytics/src/grpc_client.rs");
const main = contentByFile.get("apps/analytics/src/main.rs");
const handler = contentByFile.get("apps/backend/src/web/analytics/eps/cache.rs");
contains(grpc, "Endpoint::from_shared(endpoint)", "lazy endpoint parsing");
contains(grpc, "endpoint.connect_lazy()", "lazy tonic channel");
contains(grpc, "tokio::time::timeout(", "authority deadline");
contains(grpc, "GRPC_TIMEOUT", "fixed authority deadline");
contains(grpc, "RankingOffset::new(raw_offset)", "strict wire validation");
contains(grpc, "ErrorKind::ServiceUnavailable", "opaque authority error kind");
excludes(grpc, "IdentityClient::connect(", "eager identity dial");
excludes(grpc, "RankingOffset::from(raw_offset)", "lossy wire clamping");
excludes(grpc, "fallback: Arc<dyn WalletRankingOffsetQuery>", "free-plan fallback field");
contains(main, "GrpcWalletRankingOffsetQuery::new(grpc_endpoint)", "lazy runtime construction");
excludes(main, "let fallback: Arc<dyn WalletRankingOffsetQuery>", "runtime free-plan fallback wiring");
contains(handler, "resolve_market_ranking_offset(", "authority resolution helper");
contains(handler, "wallet_address.as_deref()", "verified wallet authority input");
contains(handler, "ErrorKind::ServiceUnavailable", "handler fail-closed error");
excludes(handler, "using free tier", "authenticated authority downgrade log");
const authorityIndex = handler.indexOf("let rank_offset = resolve_market_ranking_offset(");
const providerIndex = handler.indexOf("fetch_market_rankings(");
if (authorityIndex < 0 || providerIndex < 0 || authorityIndex >= providerIndex) fail("authority must resolve before provider work");
for (const name of expectedTests) if (!grpc.includes(name) && !handler.includes(name)) fail(`missing hermetic test source: ${name}`);

process.stdout.write(JSON.stringify({
  artifact: contract.artifact,
  productionReady: contract.productionReady,
  readinessExit: contract.readinessExit,
  invariants: contract.invariants.length,
  hermeticTests: contract.hermeticTests.length,
  implementationEvidence: contract.implementationEvidence.length,
  residualStops: contract.residualStops,
}));
' "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT")" || exit 1

if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  analytics_list="$(cargo test --offline --locked -p epsx-analytics-service --bin epsx-analytics-service -- --list 2>&1)" || {
    printf '%s\n' "$analytics_list" >&2
    die "could not enumerate analytics hermetic tests"
  }
  backend_list="$(cargo test --offline --locked -p epsx --lib -- --list 2>&1)" || {
    printf '%s\n' "$backend_list" >&2
    die "could not enumerate backend hermetic tests"
  }
  while IFS= read -r test_name; do
    analytics_match="$(printf '%s\n' "$analytics_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    backend_match="$(printf '%s\n' "$backend_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    match_count="$(( $(printf '%s\n' "$analytics_match" "$backend_match" | sed '/^$/d' | wc -l | tr -d ' ') ))"
    [[ "$match_count" == "1" ]] || die "hermetic test name is missing or ambiguous: $test_name"
    if [[ -n "$analytics_match" ]]; then
      output="$(cargo test --offline --locked -p epsx-analytics-service --bin epsx-analytics-service "$analytics_match" -- --exact 2>&1)" || {
        printf '%s\n' "$output" >&2
        die "hermetic test failed: $test_name"
      }
    else
      output="$(cargo test --offline --locked -p epsx --lib "$backend_match" -- --exact 2>&1)" || {
        printf '%s\n' "$output" >&2
        die "hermetic test failed: $test_name"
      }
    fi
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" || die "hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  check_output="$(cargo check --offline --locked -p epsx-analytics-service 2>&1)" || {
    printf '%s\n' "$check_output" >&2
    die "epsx-analytics-service offline check failed"
  }
fi

case "$MODE" in
  integrity)
    printf 'ranking-authority-failure-boundary: PASS; 10 invariants; 10 hermetic tests; 3 implementation digests; 8 residual STOPs\n'
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    printf 'ranking-authority-failure-boundary: LIMIT; 8 residual STOPs remain; readiness exit 3\n'
    exit 3
    ;;
esac
