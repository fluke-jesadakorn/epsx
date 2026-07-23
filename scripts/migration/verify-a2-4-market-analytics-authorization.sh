#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT_RAW="${EPSX_A2_4_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_4_STATIC_ONLY:-0}"

die() {
  echo "market-analytics-authorization: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-4-market-analytics-authorization.json"
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

for name in LIVE_DATA USE_LIVE_DATA ANALYTICS_LIVE_DATA INDEXER_SYNC_ON_START SYNC_ON_START; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    1|true|yes|on|live|enabled) die "$name enables a live-data or sync path" ;;
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
import { readFileSync, realpathSync } from "node:fs";
import { createHash } from "node:crypto";
import { isAbsolute, relative, resolve } from "node:path";

const [repoInput, evidenceInput, contractInput] = process.argv.slice(1);
const repo = realpathSync(repoInput);
const evidenceRoot = realpathSync(evidenceInput);
const fail = (message) => {
  console.error(`market-analytics-authorization: ERROR: ${message}`);
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
  let actual;
  try { actual = realpathSync(candidate); }
  catch { fail(`${label}: evidence file is missing`); }
  const rel = relative(evidenceRoot, actual);
  if (rel.startsWith("..") || isAbsolute(rel)) fail(`${label}: unsafe path`);
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
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-4-market-analytics-authorization") fail("unexpected contract schema/artifact");
if (contract.contractId !== "A2.4-market-analytics-direct-service-authorization" || contract.purpose !== "deterministic-hermetic-boundary-and-readiness-stop") fail("unexpected contract identity/purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
const expectedSafety = { database: false, redis: false, identityNetwork: false, marketProvider: false, rpc: false, browser: false, serviceListener: false, deployment: false, production: false };
if (JSON.stringify(contract.safety) !== JSON.stringify(expectedSafety)) fail("all exact safety flags must remain false");

const source = contract.sourceBaseline;
if (!source || source.ref !== "origin/development" || source.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") fail("source baseline drifted");
if (source.interpretation !== "Compatibility evidence only; weak invalid-bearer fallback and API-key behavior are not automatically safe production targets.") fail("source interpretation drifted");
if (git("rev-parse", `${source.ref}^{commit}`) !== source.commit) fail("source ref is stale");
const expectedSourceEvidence = [
  { id: "src-market-client-contract", file: "shared/api/analytics.ts", blob: "95e6457879199083da6f9cf27034ce0ebc6b1815", anchor: "/api/analytics/rankings" },
  { id: "src-market-action-fallback", file: "apps/frontend/app/actions/analytics.ts", blob: "8058d826e5fcec8cb5b1a14e7019da23b5bebf6a", anchor: "return await analytics.getPublicRankings(mergedFilters);" },
  { id: "src-market-router", file: "apps/backend/src/web/routes/unified_router.rs", blob: "46b97779e8726757560d7946d1a47114bed28861", anchor: ".nest(\"/analytics\"" },
  { id: "src-ranking-offset-authority", file: "apps/backend/src/web/analytics/eps/cache.rs", blob: "13f8c39b24a248186d6c24f3298023ba678bc210", anchor: "get_wallet_ranking_offset" },
  { id: "src-market-metadata", file: "apps/backend/src/web/analytics/eps/metadata.rs", blob: "fa00324d37a1400519264aa2877b65a1563239e1", anchor: "get_all_valid_countries" },
];
if (JSON.stringify(source.evidence) !== JSON.stringify(expectedSourceEvidence)) fail("source evidence tuples drifted");
if (new Set(source.evidence.map((item) => item.file)).size !== source.evidence.length) fail("source evidence paths must be unique");
for (const item of source.evidence) {
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid source blob`);
  if (git("rev-parse", `${source.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale source blob`);
  contains(git("show", `${source.commit}:${item.file}`), item.anchor, `source anchor ${item.id}`);
}

const target = contract.targetBase;
if (!target || target.ref !== "migration/dioxus-microservices" || target.commit !== "c238954cbbf9b8a5db57ef117f0be638c4613766") fail("target base drifted");
if (target.interpretation !== "Immutable pre-A2.4 snapshot; root routes, absent direct auth, always-free authority and global SSE were findings to remediate or retain as STOPs.") fail("target-base interpretation drifted");
if (git("rev-parse", `${target.commit}^{commit}`) !== target.commit) fail("target base commit is missing");
const expectedTargetEvidence = [
  { id: "base-market-manifest", file: "apps/analytics/Cargo.toml", blob: "249c1d9561ce24a1c106c04633b0885579534a68" },
  { id: "base-market-router", file: "apps/analytics/src/main.rs", blob: "96b485a1accda0329ae4a9ad170ad805325ff2ae" },
  { id: "base-market-grpc", file: "apps/analytics/src/grpc_client.rs", blob: "67d4ebf964cc23f5dcbe1dc7dcdbc6596f84bbc8" },
  { id: "base-market-sse", file: "apps/analytics/src/sse_consumer.rs", blob: "2fc91c9eb39e04068097cf79d648c17966c00fe0" },
  { id: "base-shared-verifier", file: "shared/rust/epsx-service-auth/src/lib.rs", blob: "2111f6fef250677c0589bf1ece8388a38b57d75b" },
];
if (JSON.stringify(target.evidence) !== JSON.stringify(expectedTargetEvidence)) fail("target-base evidence tuples drifted");
if (new Set(target.evidence.map((item) => item.file)).size !== target.evidence.length) fail("target-base evidence paths must be unique");
for (const item of target.evidence) {
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid target-base blob`);
  if (git("rev-parse", `${target.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale target-base blob`);
}

const expectedRoutes = [
  ["market.health", ["GET", "HEAD"], "/health", "public-credential-omitting", null],
  ["market.rankings", ["GET"], "/api/analytics/rankings", "anonymous-or-strict-supplied-bearer", "verified-principal-only"],
  ["market.filters", ["GET"], "/api/analytics/filters", "public-credential-omitting", null],
  ["market.countries", ["GET"], "/api/analytics/countries", "public-credential-omitting", null],
  ["market.available-countries", ["GET"], "/api/analytics/available-countries", "public-credential-omitting", null],
  ["market.sectors", ["GET"], "/api/analytics/sectors", "public-credential-omitting", null],
];
if (!Array.isArray(contract.routes) || contract.routes.length !== expectedRoutes.length) fail("exactly six route records are required");
for (const [index, expected] of expectedRoutes.entries()) {
  const route = contract.routes[index];
  if (!route || route.id !== expected[0] || JSON.stringify(route.methods) !== JSON.stringify(expected[1]) || route.path !== expected[2] || route.policy !== expected[3] || route.walletSource !== expected[4]) fail(`route policy drifted: ${route?.id}`);
}

const expectedInvariants = [
  { id: "canonical-market-namespace", claim: "Only /health and the five canonical /api/analytics market routes are dispatched; raw, public-duplicate, event-analytics and SSE aliases are blocked before dependencies." },
  { id: "public-credentials-omitted", claim: "Health and metadata remain public, strip Authorization and spoofable identity headers, and never invoke the verifier." },
  { id: "anonymous-rankings-retained", claim: "Rankings without Authorization remain anonymous free-tier input for source-compatible public usability." },
  { id: "supplied-credentials-strict", claim: "Any supplied rankings Authorization value must be one strict bearer and verify successfully before handler dispatch." },
  { id: "exact-browser-audiences", claim: "Only verified epsx-frontend or epsx-admin browser audiences can establish a rankings principal." },
  { id: "server-derived-wallet", claim: "The standalone rankings wallet comes only from VerifiedPrincipal and is propagated through a server-owned AnalyticsWalletContext." },
  { id: "principal-cache-isolation", claim: "Anonymous, authenticated and denied rankings responses are private no-store and vary on Authorization so intermediaries cannot reuse a principal-dependent response." },
  { id: "spoof-and-bearer-stripping", claim: "Spoofable identity headers and the raw bearer are removed before handler dispatch." },
  { id: "denial-before-handler", claim: "Invalid, duplicate, malformed, wrong-audience, unknown-path and wrong-method requests fail before handler or provider work." },
  { id: "sse-runtime-disabled", claim: "The A2.4 candidate has no global ranking-offset SSE startup task or downstream route; active pre-candidate deployment manifests remain outside this code-only slice." },
  { id: "backend-policy-authority-retained", claim: "Analytics never derives ranking offset, plan, feature, subscription or entitlement authority from token permission strings." },
];
if (JSON.stringify(contract.invariants) !== JSON.stringify(expectedInvariants)) fail("exact invariant claims drifted");

const expectedTests = [
  "exact_route_and_method_inventory_is_closed",
  "health_and_metadata_are_public_and_credential_omitting",
  "rankings_without_credentials_remains_anonymous_free_tier_input",
  "exact_frontend_and_admin_principals_propagate_verified_wallet",
  "invalid_and_unsupported_credentials_fail_before_handler",
  "malformed_and_duplicate_bearers_fail_before_verifier_or_handler",
  "spoofable_identity_headers_are_removed_before_dispatch",
  "route_drift_and_stream_are_404_before_verifier_or_handler",
  "production_verifier_rejects_local_or_plain_http_authorities",
  "test_canonical_route_inventory_and_blocked_aliases",
];
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("ten-test inventory drifted");

const implementation = [
  ["impl-market-manifest", "apps/analytics/Cargo.toml", "8edb05b7fc9f4eee4313c59e434c7a0d874f0f3efe0eaea1df26938a15497f3f"],
  ["impl-market-auth", "apps/analytics/src/auth.rs", "4cb018b22cf510302b20b7c21f546083d07fc39b3b94593212ddc328d55a3471"],
  ["impl-market-router", "apps/analytics/src/main.rs", "5abec572f6d7cef0128aff75299a051098b2261c401df15c6e90b5b9b7aace57"],
  ["impl-market-wallet-bridge", "apps/backend/src/web/analytics/eps/cache.rs", "917ec1d5df3547b99287403c5cbda9137e7e80138f77fc57fd054056c6447e20"],
];
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== implementation.length) fail("four implementation records are required");
const contents = new Map();
for (const [index, [id, file, digest]] of implementation.entries()) {
  const item = contract.implementationEvidence[index];
  if (!item || item.id !== id || item.file !== file || item.sha256 !== digest) fail(`${id}: implementation record drifted`);
  const content = read(safePath(file, id));
  const actual = createHash("sha256").update(content).digest("hex");
  if (actual !== digest) fail(`${id}: implementation digest drifted`);
  contents.set(file, content);
}

const manifest = contents.get("apps/analytics/Cargo.toml");
contains(manifest, "epsx-service-auth = { path = \"../../shared/rust/epsx-service-auth\" }", "shared auth dependency");

const auth = contents.get("apps/analytics/src/auth.rs");
for (const anchor of [
  "enum AccessPolicy", "AccessPolicy::Public", "AccessPolicy::OptionalAuthenticated", "AccessPolicy::Blocked",
  "authenticate_headers(state.verifier.as_ref(), request.headers()).await", "AnalyticsWalletContext::new(principal.wallet_address.to_lowercase())",
  "request.extensions_mut().insert(principal)", "request.headers_mut().remove(header::AUTHORIZATION)",
  "strip_spoofable_identity_headers(request.headers_mut())", "StatusCode::NOT_FOUND.into_response()",
  "HeaderValue::from_static(\"private, no-store\")", "HeaderValue::from_static(\"Authorization\")",
]) contains(auth, anchor, "auth structure");
excludes(auth, "principal.permissions", "token permission authority");
for (const name of expectedTests.slice(0, 9)) contains(auth, `fn ${name}(`, `auth test ${name}`);

const main = contents.get("apps/analytics/src/main.rs");
const builderStart = main.indexOf("pub fn build_analytics_router");
const builderEnd = main.indexOf("async fn health_handler", builderStart);
if (builderStart < 0 || builderEnd < 0) fail("production router builder is missing");
const builder = main.slice(builderStart, builderEnd);
for (const path of ["/health", "/api/analytics/rankings", "/api/analytics/filters", "/api/analytics/countries", "/api/analytics/available-countries", "/api/analytics/sectors"]) contains(builder, `\"${path}\"`, `canonical route ${path}`);
for (const path of ["/rankings", "/filters", "/countries", "/available-countries", "/sectors", "/v1/rankings/stream", "/api/v1/analytics"]) excludes(builder, `.route(\"${path}\"`, `route alias ${path}`);
contains(builder, "protect_router(router, verifier)", "production auth layer");
const production = main.slice(0, main.indexOf("#[cfg(test)]\nmod tests"));
for (const forbidden of ["run_sse_consumer(", "rankings_stream_handler", ".route(\"/v1/rankings/stream\""]) excludes(production, forbidden, "production SSE runtime");
contains(main, "#[cfg(test)]\nmod sse_consumer;", "test-only SSE module");
contains(main, "OIDC_ISSUER is required", "required OIDC issuer");
for (const marker of ["APP_ENV", "ENVIRONMENT", "DEPLOYMENT_ENV", "starts_with(\"production-\")", "ends_with(\"-production\")"]) contains(main, marker, "production environment classification");
contains(main, "test_canonical_route_inventory_and_blocked_aliases", "canonical router canary");

const cache = contents.get("apps/backend/src/web/analytics/eps/cache.rs");
for (const anchor of ["pub struct AnalyticsWalletContext", "analytics_wallet_ext: Option<Extension<AnalyticsWalletContext>>", ".map(|context| context.wallet_address().to_lowercase())", ".or_else(|| user_context.as_ref().map(|ctx| ctx.wallet_address.to_lowercase()))"]) contains(cache, anchor, "wallet bridge");

const expectedStops = [
  { id: "route-owner-cutover-unproved", claim: "The monolith still owns canonical routes and no gateway, Cloudflare or workload cutover is authorized or proven." },
  { id: "public-auth-compatibility-incomplete", claim: "Legacy public duplicate paths, API-key behavior, implicit HEAD, wrong-method 405 behavior, query caps, filters, envelopes and status compatibility are not fully adjudicated." },
  { id: "plan-offset-authority-always-free", claim: "The identity gRPC server still returns free-plan offset for every wallet, so premium behavior is not authoritative." },
  { id: "authority-outage-fallback-unproved", claim: "Authenticated authority failures still silently fall back to free and can continue into provider work instead of a truthful fail-closed response." },
  { id: "provider-boundary-unproved", claim: "TradingView provenance, licensing, quotas, timeout, retry, normalization, freshness, cache and sanitized failure semantics are unproven by A2.4." },
  { id: "provider-amplification-unbounded", claim: "A2.4 does not prove a public request-size, rate, concurrency or circuit-breaker boundary; later provider slices must carry that evidence independently." },
  { id: "internal-identity-untrusted", claim: "The gRPC identity query has no authenticated service identity, authoritative owner binding, NetworkPolicy or immutable non-dev image proof." },
  { id: "upstream-ranking-events-unsafe", claim: "The downstream SSE path is disabled, but identity still exposes unauthenticated emit/global stream endpoints and the candidate retains dormant historical SSE source, tests and dependencies pending archive cleanup." },
  { id: "runtime-config-readiness-unproved", claim: "Checked-in overlays still select pre-A2.4 images/configuration; no reviewed candidate OIDC wiring exists and health does not check identity or market-provider readiness." },
  { id: "market-ui-unproved", claim: "No BFF loader or responsive Dioxus loading, empty, error, stale, filter, pagination, watchlist and entitlement journey consumes this contract." },
  { id: "live-runtime-unproved", claim: "No accepted real JWT, browser, provider, identity, proxy, listener, staging or fault-injection exercise exists." },
  { id: "production-actions-unauthorized", claim: "No build, image publication, database, secret, service restart, canary, rollback or production deployment is authorized by this contract." },
];
if (JSON.stringify(contract.residualStops) !== JSON.stringify(expectedStops)) fail("exact residual STOP claims drifted");
const expectedSteps = [
  "E01 Extract and independently verify a provider port with bounded public request size, concurrency, retry and timeout behavior.",
  "E02 Make identity ranking offsets authoritative from active backend plan assignments and fail authenticated authority errors truthfully before provider work.",
  "E03 Define authenticated internal-service identity for ranking queries; keep public emit and global SSE unavailable until a durable owner-safe protocol exists.",
  "E04 Lock public/auth query, envelope, status, cap, API-key and freshness compatibility against the pinned source contract.",
  "E05 Select one production market route owner and add explicit gateway or BFF routing without colliding with event analytics.",
  "E06 Add the typed frontend BFF loader and Dioxus loading, empty, error, stale, filter, sort, pagination, watchlist and entitlement states.",
  "E07 Run hermetic full-story tests, then separately authorized disposable and staging exercises with observability, shadow parity and rollback evidence.",
  "E08 Request production deployment authorization only after all residual STOPs are closed.",
];
if (JSON.stringify(contract.requiredExecutionOrder) !== JSON.stringify(expectedSteps)) fail("exact execution order drifted");

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  targetBase: { ref: target.ref, commit: target.commit, evidence: target.evidence.length },
  routes: contract.routes.map(({ id, methods, path, policy, walletSource }) => ({ id, methods, path, policy, walletSource })),
  invariants: contract.invariants.length,
  hermeticTests: contract.hermeticTests.length,
  implementationEvidence: contract.implementationEvidence.length,
  residualStops: contract.residualStops.map((item) => item.id),
  executionSteps: contract.requiredExecutionOrder.length,
  productionReady: false,
  readinessExit: 3,
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT")" || exit 1

if [[ "$MODE" == "report" ]]; then
  printf '%s\n' "$summary"
  exit 0
fi

if [[ "$MODE" == "readiness" ]]; then
  echo "market-analytics-authorization: STOP readiness (12 residual STOPs remain)" >&2
  echo "market-analytics-authorization: LIMIT — hermetic boundary evidence is not production readiness or deployment authorization" >&2
  exit 3
fi

if [[ "$STATIC_ONLY" != "1" ]]; then
  temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a2-4-verifier.XXXXXX")" || die "cannot create temporary directory"
  trap 'rm -rf -- "$temp_dir"' EXIT HUP INT TERM
  run_cargo() {
    local label="$1"
    shift
    if ! "$@" >"$temp_dir/$label.out" 2>&1; then
      sed -n '1,240p' "$temp_dir/$label.out" >&2
      die "$label failed"
    fi
  }
  run_cargo check cargo check --offline --locked -p epsx-analytics-service
  run_cargo auth-tests cargo test --offline --locked -p epsx-analytics-service auth::tests --no-fail-fast
  run_cargo router-canary cargo test --offline --locked -p epsx-analytics-service tests::test_canonical_route_inventory_and_blocked_aliases --no-fail-fast
  grep -q "test result: ok. 9 passed; 0 failed" "$temp_dir/auth-tests.out" || die "auth test count drifted"
  grep -q "test result: ok. 1 passed; 0 failed" "$temp_dir/router-canary.out" || die "router canary count drifted"
fi

echo "market-analytics-authorization: PASS integrity (5 source pins; 5 target-base pins; 6 routes; 11 invariants; 10 hermetic tests; 12 residual STOPs)"
echo "market-analytics-authorization: LIMIT — no database, Redis, identity, provider, RPC, browser, listener, deployment, or production readiness was proven"
