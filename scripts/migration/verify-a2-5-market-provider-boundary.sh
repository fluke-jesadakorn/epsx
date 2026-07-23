#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT_RAW="${EPSX_A2_5_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_5_STATIC_ONLY:-0}"

die() {
  echo "market-provider-boundary: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-5-market-provider-boundary.json"
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

for name in DATABASE_URL TEST_DATABASE_URL ANALYTICS_DATABASE_URL IDENTITY_DATABASE_URL REDIS_URL REDIS_CLUSTER_URL OIDC_ISSUER OIDC_JWKS_URL JWKS_URL AUTH_JWKS_URL AUTH_BASE_URL BACKEND_URL NEXT_PUBLIC_BACKEND_URL API_URL IDENTITY_GRPC_URL IDENTITY_SSE_URL TRADINGVIEW_AUTH_TOKEN TRADINGVIEW_URL TRADINGVIEW_BASE_URL TRADINGVIEW_WEBSOCKET_URL MARKET_DATA_URL MARKET_DATA_API_KEY LIVE_DATA_URL RPC_URL CHAIN_RPC_URL BSC_RPC_URL BSC_MAINNET_RPC_URL BSC_TESTNET_RPC_URL ETH_RPC_URL ETHEREUM_RPC_URL POLYGON_RPC_URL WEB3_PROVIDER_URL; do
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
  console.error(`market-provider-boundary: ERROR: ${message}`);
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
  const parts = value.split("/");
  if (parts.some((part) => !part || part === "." || part === "..")) fail(`${label}: unsafe path`);
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
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-5-market-provider-boundary") fail("unexpected contract schema/artifact");
if (contract.contractId !== "A2.5-market-rankings-provider-boundary" || contract.purpose !== "deterministic-hermetic-resource-boundary-and-readiness-stop") fail("unexpected contract identity/purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
const expectedSafety = { database: false, redis: false, identityNetwork: false, marketProvider: false, rpc: false, browser: false, serviceListener: false, deployment: false, production: false };
if (JSON.stringify(contract.safety) !== JSON.stringify(expectedSafety)) fail("all exact safety flags must remain false");

const expectedSource = {
  ref: "origin/development",
  commit: "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db",
  interpretation: "Compatibility evidence is preserved where safe; client-only bounds, silent filter behavior, fallback authentication and unbounded provider work are findings rather than production targets.",
  evidence: [
    { id: "src-public-ranking-cap", file: "shared/api/analytics.ts", blob: "95e6457879199083da6f9cf27034ce0ebc6b1815", anchor: "limit: Math.min(filters.limit ?? 10, 10), // Public API limit: max 10" },
    { id: "src-auth-ranking-client", file: "shared/api/analytics.ts", blob: "95e6457879199083da6f9cf27034ce0ebc6b1815", anchor: "async getAuthenticatedRankings(filters: AnalyticsFilters = {}): Promise<CardDashboardResponse> {" },
    { id: "src-per-request-provider", file: "apps/backend/src/web/analytics/eps/cache.rs", blob: "13f8c39b24a248186d6c24f3298023ba678bc210", anchor: "TradingViewApiService::new(" },
    { id: "src-provider-retry", file: "apps/backend/src/infrastructure/adapters/services/tradingview/rest.rs", blob: "4ac774f151801c8c81f590a0e162c4914a134ecd", anchor: "pub async fn execute_custom_request(" },
    { id: "src-provider-three-retries", file: "apps/backend/src/infrastructure/adapters/services/tradingview/api_service.rs", blob: "ef5c58e3e8127b944a3f97df9b893ba66cf1ee57", anchor: "execute_custom_request(payload, 3)" },
    { id: "src-unchecked-provider-range", file: "apps/backend/src/infrastructure/adapters/services/tradingview/scanner.rs", blob: "8b30975ea466a70f17163ce8b32aa091fc92be97", anchor: "let range_end = skip + limit;" },
    { id: "src-existing-port-area", file: "apps/backend/src/domain/market_analytics/repository_ports/market_data_scanner_port.rs", blob: "9dfb7d102cfb7a087f2bc351e34cf84507bca08e", anchor: "pub trait MarketDataScannerPort: Send + Sync" },
  ],
};
if (JSON.stringify(contract.sourceBaseline) !== JSON.stringify(expectedSource)) fail("source baseline or evidence tuples drifted");
if (git("rev-parse", `${expectedSource.ref}^{commit}`) !== expectedSource.commit) fail("source ref is stale");
for (const item of expectedSource.evidence) {
  if (git("rev-parse", `${expectedSource.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale source blob`);
  const content = git("show", `${expectedSource.commit}:${item.file}`);
  contains(content, item.anchor, `${item.id} source anchor`);
}

const expectedTarget = {
  ref: "migration/dioxus-microservices",
  commit: "e5b6552481de8fda72ffdd56b435789c7049392b",
  interpretation: "Immutable post-A2.4 snapshot before provider extraction, request bounds, shared concurrency ownership, single-attempt transport and deterministic provider tests.",
  evidence: [
    { id: "base-ranking-handler", file: "apps/backend/src/web/analytics/eps/cache.rs", blob: "c1bfd92a3976788fc7dcb2cdaf7e20b310082577" },
    { id: "base-provider-rest", file: "apps/backend/src/infrastructure/adapters/services/tradingview/rest.rs", blob: "3bdb98f5d077749e95f9ca0a5008ba5439dadcc4" },
    { id: "base-provider-api", file: "apps/backend/src/infrastructure/adapters/services/tradingview/api_service.rs", blob: "ef5c58e3e8127b944a3f97df9b893ba66cf1ee57" },
    { id: "base-provider-scanner", file: "apps/backend/src/infrastructure/adapters/services/tradingview/scanner.rs", blob: "a79b38fd0e28f76c0f0a009c1327399b205243b9" },
    { id: "base-provider-port-area", file: "apps/backend/src/domain/market_analytics/repository_ports/market_data_scanner_port.rs", blob: "00ea0febe2928393372ae921b8d0a20405d54c15" },
    { id: "base-provider-adapter", file: "apps/backend/src/infrastructure/adapters/services/tradingview/tradingview_adapter.rs", blob: "eb78f0fc455f8194df3ee81f3d717ed5e3ea839e" },
    { id: "base-standalone-wiring", file: "apps/analytics/src/main.rs", blob: "d79023471643f90c366f5ca11e0c42b091814287" },
    { id: "base-monolith-wiring", file: "apps/backend/src/web/routes/unified_router.rs", blob: "a4e18b46623d28243345e011e7bba426a1313d9a" },
  ],
};
if (JSON.stringify(contract.targetBase) !== JSON.stringify(expectedTarget)) fail("target base or evidence tuples drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("target base commit is missing");
for (const item of expectedTarget.evidence) {
  if (git("rev-parse", `${expectedTarget.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale target blob`);
}

const expectedInvariantIds = [
  "canonical-port-only-handler", "process-shared-provider", "anonymous-cap-ten",
  "authenticated-cap-one-hundred", "checked-pagination", "shared-concurrency-budget",
  "bounded-transient-retry", "single-total-deadline", "single-attempt-live-adapter",
  "opaque-provider-failures", "bounded-provider-response", "card-contract-preserved",
  "authority-dependency-honest", "non-production-evidence",
];
if (JSON.stringify(contract.invariants.map((item) => item.id)) !== JSON.stringify(expectedInvariantIds)) fail("invariant inventory drifted");
for (const item of contract.invariants) {
  if (typeof item.claim !== "string" || item.claim.length < 40 || /production ready|deployment authorized/i.test(item.claim)) fail(`${item.id}: invalid invariant meaning`);
}
const expectedTests = [
  "a2_5_invalid_pagination_never_calls_inner_provider",
  "a2_5_transient_failures_succeed_within_three_total_attempts",
  "a2_5_transient_failures_stop_after_three_total_attempts",
  "a2_5_permanent_failure_is_not_retried_and_is_sanitized",
  "a2_5_invalid_provider_page_is_rejected",
  "a2_5_timeout_is_sanitized_and_releases_the_permit",
  "a2_5_shared_concurrency_peaks_at_five_and_saturation_fails_fast",
  "a2_5_provider_response_body_is_bounded",
  "a2_5_http_status_retry_classification_is_explicit",
  "a2_5_missing_provider_total_preserves_current_page_extent",
  "a2_5_anonymous_limit_is_capped_at_ten",
  "a2_5_authenticated_limit_is_capped_at_one_hundred",
  "a2_5_checked_pagination_overflow_fails_before_provider_call",
  "a2_5_sort_aliases_normalize_and_supported_fields_are_preserved",
  "a2_5_unknown_sort_is_rejected_before_provider_call",
  "a2_5_accessible_pagination_excludes_locked_ranks",
  "a2_5_provider_error_is_sanitized",
  "a2_5_successful_mapping_preserves_quarterly_dto",
];
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("hermetic test inventory drifted");

const expectedImplementation = [
  ["impl-rankings-port", "apps/backend/src/domain/market_analytics/repository_ports/market_rankings_provider_port.rs", "8c465441d8d904a72c74dcb8a2abe030a88ca6ab1cc5b061c453f0e8332002c8"],
  ["impl-rankings-port-module", "apps/backend/src/domain/market_analytics/repository_ports/mod.rs", "6591ca8a6db2be02abfc55db0075a1869edd8b144cac311cb25aadf8d83e6782"],
  ["impl-rankings-domain-export", "apps/backend/src/domain/market_analytics/mod.rs", "9d8f0c74f4c299021647ec3cdd5d097ce1faf584f37abcda70834a44c86359ca"],
  ["impl-bounded-provider", "apps/backend/src/infrastructure/adapters/services/tradingview/bounded_rankings_provider.rs", "281df061a40db3f127c46df2184cd0b5801c397e970c8c08858b6ecfca351578"],
  ["impl-provider-module", "apps/backend/src/infrastructure/adapters/services/tradingview/mod.rs", "6d44e0865b0dc339f47af77323142954128f2565867f187f307e27ffd44888df"],
  ["impl-provider-types", "apps/backend/src/infrastructure/adapters/services/tradingview/types.rs", "85d4248f68897cd69b80a1aac07c02249a65aae72ccc7b5ee20dd5b1a883ccd2"],
  ["impl-provider-rest", "apps/backend/src/infrastructure/adapters/services/tradingview/rest.rs", "a730498205af3956382a99292c1580827847ced39825df10f7fa37220e185913"],
  ["impl-provider-api", "apps/backend/src/infrastructure/adapters/services/tradingview/api_service.rs", "70312e3451c576aa39c4ea734b2ba6f687bc001d57b177c1c492e2b9f32ab952"],
  ["impl-provider-adapter", "apps/backend/src/infrastructure/adapters/services/tradingview/tradingview_adapter.rs", "604cb1136508dcef1f4550950d1c58d3795722de2ef8ac972a2408601c8311bf"],
  ["impl-ranking-handler", "apps/backend/src/web/analytics/eps/cache.rs", "917ec1d5df3547b99287403c5cbda9137e7e80138f77fc57fd054056c6447e20"],
  ["impl-standalone-wiring", "apps/analytics/src/main.rs", "5abec572f6d7cef0128aff75299a051098b2261c401df15c6e90b5b9b7aace57"],
  ["impl-monolith-wiring", "apps/backend/src/web/routes/unified_router.rs", "7578a2479445b68a5c85adc9afdcef7b7dd085c7d1292b56d7709e387851507b"],
];
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("twelve implementation records are required");
const contentByFile = new Map();
for (let index = 0; index < expectedImplementation.length; index += 1) {
  const [id, file, sha256] = expectedImplementation[index];
  const item = contract.implementationEvidence[index];
  if (JSON.stringify(item) !== JSON.stringify({ id, file, sha256 })) fail(`${id}: implementation tuple drifted`);
  const path = safePath(file, id);
  const content = read(path);
  const actual = createHash("sha256").update(content).digest("hex");
  if (actual !== sha256) fail(`${id}: implementation digest drifted`);
  contentByFile.set(file, content);
}

const expectedStopIds = [
  "route-owner-cutover-unproved", "public-auth-compatibility-incomplete",
  "plan-offset-authority-always-free", "authority-outage-fallback-unproved",
  "provider-commercial-data-semantics-unproved", "distributed-resilience-unproved",
  "legacy-provider-callers-unbounded", "internal-identity-untrusted",
  "upstream-ranking-events-unsafe", "runtime-config-readiness-unproved",
  "market-ui-unproved", "live-runtime-unproved", "production-actions-unauthorized",
];
if (JSON.stringify(contract.residualStops.map((item) => item.id)) !== JSON.stringify(expectedStopIds)) fail("residual STOP inventory drifted");
for (const item of contract.residualStops) {
  if (typeof item.claim !== "string" || item.claim.length < 40 || /production ready|deployment authorized/i.test(item.claim)) fail(`${item.id}: invalid STOP meaning`);
}
const expectedOrderPrefixes = ["E01 ", "E02 ", "E03 ", "E04 ", "E05 ", "E06 ", "E07 ", "E08 "];
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== expectedOrderPrefixes.length) fail("execution order drifted");
contract.requiredExecutionOrder.forEach((item, index) => {
  if (typeof item !== "string" || !item.startsWith(expectedOrderPrefixes[index]) || (index < 7 && /deploy first/i.test(item))) fail(`invalid execution step E0${index + 1}`);
});

const port = contentByFile.get("apps/backend/src/domain/market_analytics/repository_ports/market_rankings_provider_port.rs");
const bounded = contentByFile.get("apps/backend/src/infrastructure/adapters/services/tradingview/bounded_rankings_provider.rs");
const types = contentByFile.get("apps/backend/src/infrastructure/adapters/services/tradingview/types.rs");
const rest = contentByFile.get("apps/backend/src/infrastructure/adapters/services/tradingview/rest.rs");
const api = contentByFile.get("apps/backend/src/infrastructure/adapters/services/tradingview/api_service.rs");
const adapter = contentByFile.get("apps/backend/src/infrastructure/adapters/services/tradingview/tradingview_adapter.rs");
const handler = contentByFile.get("apps/backend/src/web/analytics/eps/cache.rs");
const standalone = contentByFile.get("apps/analytics/src/main.rs");
const monolith = contentByFile.get("apps/backend/src/web/routes/unified_router.rs");
contains(port, "pub trait MarketRankingsProviderPort: Send + Sync", "rankings port");
contains(bounded, "try_acquire_owned()", "fail-fast shared semaphore");
contains(bounded, "MAX_TOTAL_ATTEMPTS: usize = 3", "three total attempts");
contains(bounded, "error.is_retryable()", "transient-only retry");
contains(bounded, "time::timeout(self.total_deadline", "one total deadline");
contains(bounded, "MAX_CONCURRENT_REQUESTS, MAX_PAGE_SIZE", "shared provider constants");
contains(bounded, "validate_provider_page(&page, &request)", "provider page bound");
excludes(bounded, "TcpListener", "provider listener");
excludes(bounded, "reqwest", "provider test transport");
contains(types, "HttpStatus(u16)", "structured provider HTTP status");
contains(rest, "pub async fn execute_custom_request_once(", "single-attempt REST path");
contains(rest, "MAX_CUSTOM_RESPONSE_BYTES", "bounded provider response bytes");
excludes(rest, "Response text (first 500 chars)", "provider response-body logging");
contains(api, ".execute_custom_request_once(payload)", "single-attempt API path");
contains(api, "resolve_market_rankings_total(response.total_count, skip, response.data.len())", "missing provider total normalization");
contains(adapter, "impl MarketRankingsProviderPort for TradingViewAdapter", "live provider adapter");
contains(adapter, "MarketDataError::HttpStatus(429 | 500..=599)", "transient HTTP status mapping");
contains(handler, "Extension(rankings_provider): Extension<Arc<dyn MarketRankingsProviderPort>>", "handler provider injection");
contains(handler, "let limit_cap = if is_authenticated { 100 } else { 10 };", "public/auth bounds");
contains(handler, ".checked_mul(limit)", "checked pagination");
contains(handler, "accessible_pagination(total, rank_start, page, limit)", "accessible pagination totals");
excludes(handler, "TradingViewApiService::new(", "per-request provider construction");
excludes(handler, "enhance_with_websocket_data", "canonical WebSocket enhancement");
contains(standalone, "BoundedMarketRankingsProvider::new(", "standalone shared provider");
contains(standalone, ".layer(axum::Extension(market_rankings_provider))", "standalone provider injection");
contains(monolith, "BoundedMarketRankingsProvider::new(", "monolith shared provider");
contains(monolith, ".layer(Extension(market_rankings_provider))", "monolith provider injection");
const testSources = [bounded, rest, api, adapter, handler];
for (const name of expectedTests) if (!testSources.some((source) => source.includes(name))) fail(`missing hermetic test source: ${name}`);

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
  test_list="$(cargo test --offline --locked -p epsx --lib -- --list 2>&1)" || {
    printf '%s\n' "$test_list" >&2
    die "could not enumerate hermetic tests"
  }
  while IFS= read -r test_name; do
    qualified_test="$(printf '%s\n' "$test_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    [[ "$(printf '%s\n' "$qualified_test" | sed '/^$/d' | wc -l | tr -d ' ')" == "1" ]] ||
      die "hermetic test name is missing or ambiguous: $test_name"
    output="$(cargo test --offline --locked -p epsx --lib "$qualified_test" -- --exact 2>&1)" || {
      printf '%s\n' "$output" >&2
      die "hermetic test failed: $test_name"
    }
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" ||
      die "hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  check_output="$(cargo check --offline --locked -p epsx-analytics-service 2>&1)" || {
    printf '%s\n' "$check_output" >&2
    die "epsx-analytics-service offline check failed"
  }
fi

case "$MODE" in
  integrity)
    printf 'market-provider-boundary: PASS; 14 invariants; 18 hermetic tests; 12 implementation digests; 13 residual STOPs\n'
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    stop_count="$(bun -e 'const r = JSON.parse(process.argv[1]); console.log(r.residualStops.length);' "$summary")"
    printf 'market-provider-boundary: LIMIT; %s residual STOPs remain; readiness exit 3\n' "$stop_count"
    exit 3
    ;;
esac
