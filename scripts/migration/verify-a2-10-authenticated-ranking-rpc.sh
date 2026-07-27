#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="${EPSX_A2_10_REPO_ROOT:-$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)}"
EVIDENCE_ROOT_RAW="${EPSX_A2_10_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_10_STATIC_ONLY:-0}"

die() {
  echo "authenticated-ranking-rpc: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-10-authenticated-ranking-rpc.json"
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
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no database, credential, or live I/O"
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
  console.error(`authenticated-ranking-rpc: ERROR: ${message}`);
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
const exactIds = (items, ids, label) => {
  if (!Array.isArray(items) || JSON.stringify(items.map((item) => item.id)) !== JSON.stringify(ids)) fail(`${label} inventory drifted`);
  if (items.some((item) => typeof item.claim !== "string" || !item.claim)) fail(`invalid ${label} claim`);
};

let contract;
try { contract = JSON.parse(read(contractInput)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-10-authenticated-ranking-rpc" || contract.contractId !== "A2.10-authenticated-ranking-rpc") fail("unexpected contract identity");
if (contract.purpose !== "deterministic-hermetic-unwired-workload-authenticated-ranking-rpc-composition-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel drifted");
const expectedSafety = ["database", "migration", "redis", "identityNetwork", "rpcListener", "tls", "credentialVerifier", "browser", "deployment", "production"];
if (JSON.stringify(Object.keys(contract.safety)) !== JSON.stringify(expectedSafety) || expectedSafety.some((key) => contract.safety[key] !== false)) fail("safety sentinel drifted");

const expectedTarget = { ref: "migration/dioxus-microservices", commit: "60ababc75a79d173b3b217df8e9b9155795a1117" };
if (contract.targetBase?.ref !== expectedTarget.ref || contract.targetBase?.commit !== expectedTarget.commit) fail("target base drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("immutable target-base commit is missing");
const expectedBase = [
  ["base-identity-library", "shared/rust/epsx-identity-service/src/lib.rs", "536d7e4b2cafa148dace5dbfbb3d6a4574c2942c", "pub mod identity_service;"],
  ["base-identity-main", "shared/rust/epsx-identity-service/src/main.rs", "41f1ca17e4fa5520b77cbe8c8a42bde255718727", "Arc::new(FreePlanRankingOffsetService)"],
  ["base-always-free-service", "shared/rust/epsx-identity-service/src/identity_service.rs", "a08419f241788274d1e07dd75089479ae3f15455", "Ok(RankingOffset::free_plan())"],
  ["base-entitlement-resolver", "shared/rust/epsx-identity-service/src/ranking_entitlement.rs", "edb9d2bf9fbb445f97b0c67dba729275cb5657e6", "pub struct SnapshotWalletRankingOffsetQuery"],
  ["base-identity-proto", "shared/proto/identity.proto", "2e646cbf49b24b6291406af9ccaa5fbd3f44e946", "string wallet = 1;"],
  ["base-analytics-client", "apps/analytics/src/grpc_client.rs", "2d20ab78cbb019384a3d192ac49052bf6a003e00", "let request = tonic::Request::new(GetWalletRankingOffsetRequest { wallet });"],
];
if (!Array.isArray(contract.targetBase.evidence) || contract.targetBase.evidence.length !== expectedBase.length) fail("target-base evidence inventory drifted");
for (let index = 0; index < expectedBase.length; index += 1) {
  const [id, file, blob, anchor] = expectedBase[index];
  if (JSON.stringify(contract.targetBase.evidence[index]) !== JSON.stringify({ id, file, blob, anchor })) fail(`${id}: target-base tuple drifted`);
  if (git("rev-parse", `${expectedTarget.commit}:${file}`) !== blob) fail(`${id}: target-base blob drifted`);
  if (!git("show", `${expectedTarget.commit}:${file}`).includes(anchor)) fail(`${id}: target-base anchor missing`);
}

const currentInvariantIds = [
  "exported-unwired-composition", "exact-single-authorization-metadata", "strict-bearer-shape",
  "authorizer-once-before-wallet-or-query", "exact-workload-subject-and-audience",
  "canonical-evm-wallet-once", "query-once-after-authorization", "sanitized-exact-status-matrix",
  "metadata-not-protobuf-credential", "no-concrete-credential-adapter", "no-listener-network-or-tls",
  "runtime-fail-closed-unwired", "analytics-client-metadata-absent", "store-and-schema-unactivated",
  "event-infra-ui-payment-indexer-unchanged", "offline-hermetic-only",
];
const expectedInvariantIds = process.env.EPSX_A2_10_STATIC_ONLY === "1" ? contract.invariants.map((item) => item.id) : currentInvariantIds;
exactIds(contract.invariants, expectedInvariantIds, "invariant");

const currentImplementation = [
  ["impl-authenticated-ranking-rpc", "shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs", "1d9fc78e74d9959030bbd83c27989f987b7f200b8c70f8d0ad8110b36db60a58"],
  ["impl-library-export", "shared/rust/epsx-identity-service/src/lib.rs", "b98920a2d30c0ba1e9c6fd43f10ff27420863fd2f484f26fa1bce053e39781ea"],
];
const expectedImplementation = process.env.EPSX_A2_10_STATIC_ONLY === "1"
  ? (Array.isArray(contract.implementationEvidence) ? contract.implementationEvidence.map(({ id, file, sha256 }) => [id, file, sha256]) : [])
  : currentImplementation;
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("implementation evidence inventory drifted");
const contentByFile = new Map();
for (let index = 0; index < expectedImplementation.length; index += 1) {
  const [id, file, digest] = expectedImplementation[index];
  if (JSON.stringify(contract.implementationEvidence[index]) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: implementation tuple drifted`);
  const content = read(safePath(file, id));
  if (sha256(content) !== digest) fail(`${id}: implementation digest drifted`);
  contentByFile.set(file, content);
}

const currentUnchanged = [
  ["unchanged-identity-main", "shared/rust/epsx-identity-service/src/main.rs", "9a7c4185032803f6453dd4a2ab1afbc3bde06219209d0398571cb73721ac183d"],
  ["unchanged-fail-closed-service", "shared/rust/epsx-identity-service/src/identity_service.rs", "a5d64d6aa314a2f2c504836595baacc77437c0668f5e42663ee0299f43950895"],
  ["unchanged-entitlement-resolver", "shared/rust/epsx-identity-service/src/ranking_entitlement.rs", "b6cdeb6486296550b936d243c29efbe3cdf1e896e7ecd47218df79129a102507"],
  ["unchanged-analytics-client", "apps/analytics/src/grpc_client.rs", "bafac48faf1e9d03d1990ba57f922a01a350ff7763bed7ede2e70917a2c7a559"],
  ["unchanged-identity-proto", "shared/proto/identity.proto", "f33f7256048403c79219913051347d85d05238e9c62269e37e8bffdae9f69d23"],
  ["unchanged-identity-cargo", "shared/rust/epsx-identity-service/Cargo.toml", "54f7020be797a137a4c69d1e3fbccf0d21f88923a616bd48f0757aa770cf7c8f"],
  ["unchanged-workspace-lock", "Cargo.lock", "47f08101d842d66800ecc7a50ce220bf3854f7cca27345fe62d69de184379883"],
  ["unchanged-core-snapshot-adapter", "apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs", "4e3e609262aa9c7d73c2e9f01dce41ba2a8c531120d145d8ba88be03dc563f45"],
  ["unchanged-shared-snapshot-contract", "shared/rust/epsx-contracts/src/ranking_entitlement_snapshot.rs", "9ba917a2bb2646097162371e19f6c1b6f44d41f65b1f32dce5193614f5baadbe"],
  ["unchanged-emit-handler", "shared/rust/epsx-identity-service/src/emit_handler.rs", "6c9f63b1cf44141791a69cd95b94d8ee428d41bdc75bf8b1dba9fdd16303db6f"],
  ["unchanged-event-bus", "shared/rust/epsx-identity-service/src/event_bus.rs", "0cda139bf36229865801d43c28f04b7ed0c830c933d1383748182d36dd220f5f"],
  ["unchanged-sse-handler", "shared/rust/epsx-identity-service/src/sse_handler.rs", "0a91bb08d6c590940c0ec94006a32aadfcac11eed52f7a5912de7783d08ff7f8"],
];
const expectedUnchanged = process.env.EPSX_A2_10_STATIC_ONLY === "1"
  ? (Array.isArray(contract.unchangedEvidence) ? contract.unchangedEvidence.map(({ id, file, sha256 }) => [id, file, sha256]) : [])
  : currentUnchanged;
if (!Array.isArray(contract.unchangedEvidence) || contract.unchangedEvidence.length !== expectedUnchanged.length) fail("unchanged evidence inventory drifted");
for (let index = 0; index < expectedUnchanged.length; index += 1) {
  const [id, file, digest] = expectedUnchanged[index];
  if (JSON.stringify(contract.unchangedEvidence[index]) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: unchanged tuple drifted`);
  if (sha256(read(safePath(file, id))) !== digest) fail(`${id}: unchanged evidence digest drifted`);
}

const moduleSource = contentByFile.get("shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs");
const libSource = contentByFile.get("shared/rust/epsx-identity-service/src/lib.rs");
if ((libSource.match(/pub mod authenticated_ranking_rpc;/g) || []).length !== 1) fail("authenticated ranking RPC must be exported exactly once");
if (/#\[cfg\(test\)\]\s*pub mod authenticated_ranking_rpc;/.test(libSource)) fail("authenticated ranking RPC must be a normal non-test library export");
if (!libSource.includes("pub mod authenticated_ranking_rpc;\npub mod identity_service;")) fail("authenticated ranking RPC must be a normal non-test library export");

const testBoundaryToken = "#[cfg(test)]\nmod tests";
const boundaryMatches = moduleSource.match(/#\[cfg\(test\)\]\nmod tests/g) || [];
if (boundaryMatches.length !== 1) fail("authenticated ranking RPC must contain exactly one cfg(test) test boundary");
const testBoundary = moduleSource.indexOf(testBoundaryToken);
const production = moduleSource.slice(0, testBoundary);
const tests = moduleSource.slice(testBoundary);
if (!tests.startsWith(`${testBoundaryToken} {`) || !tests.trimEnd().endsWith("}")) fail("authenticated ranking RPC tests must be the final module item");

contains(production, "pub const EXPECTED_WORKLOAD_SUBJECT: &str = \"epsx-analytics-service\";", "exact workload subject");
contains(production, "pub const EXPECTED_WORKLOAD_AUDIENCE: &str = \"epsx-identity-service\";", "exact workload audience");
contains(production, "pub trait RankingWorkloadAuthorizer: Send + Sync", "workload authorizer port");
contains(production, "pub struct AuthenticatedRankingGrpcService", "authenticated composition type");
excludes(production, "impl RankingWorkloadAuthorizer for", "concrete production authorizer implementation");
for (const token of ["AccessTokenVerifier", "Jwt", "Jwks", "TcpListener", "TcpSocket", "tokio::net", "tonic::transport", "Server::builder", ".serve(", "Endpoint::", "connect_lazy", "reqwest::", "axum::", "diesel::", "PgConnection", "sql_query(", "DATABASE_URL", "TlsConnector", "ClientTlsConfig"]) excludes(production, token, `listener/network/store/credential token ${token}`);

const handlerStart = production.indexOf("impl Identity for AuthenticatedRankingGrpcService");
const handlerEnd = production.indexOf("\nfn parse_bearer(");
if (handlerStart < 0 || handlerEnd <= handlerStart) fail("authenticated RPC handler boundary is missing");
const handler = production.slice(handlerStart, handlerEnd);
const orderTokens = [
  "parse_bearer(request.metadata())?",
  ".authorize(bearer)",
  "workload.subject != EXPECTED_WORKLOAD_SUBJECT",
  "workload.audience != EXPECTED_WORKLOAD_AUDIENCE",
  "normalize_evm_wallet(&request.get_ref().wallet)?",
  ".get_wallet_ranking_offset(&normalized_wallet)",
];
let previous = -1;
for (const token of orderTokens) {
  const position = handler.indexOf(token);
  if (position < 0) fail(`authentication ordering token is missing: ${token}`);
  if (position <= previous) fail("authentication must precede subject/audience, wallet validation, and query work");
  previous = position;
  if ((handler.split(token).length - 1) !== 1) fail(`authentication ordering token must occur exactly once: ${token}`);
}

for (const token of [
  "metadata.get_all(\"authorization\").iter()",
  "if values.next().is_some()",
  ".to_str()",
  ".strip_prefix(\"Bearer \")",
  "bearer.is_empty()",
  "byte.is_ascii_whitespace()",
]) contains(production, token, `strict metadata parser token ${token}`);
for (const token of [
  "bytes.len() != 42",
  "&bytes[..2] != b\"0x\"",
  "bytes[2..].iter().all(u8::is_ascii_hexdigit)",
  "Ok(wallet.to_ascii_lowercase())",
]) contains(production, token, `canonical wallet token ${token}`);
if ((production.match(/wallet\.to_ascii_lowercase\(\)/g) || []).length !== 1) fail("wallet must be normalized to lowercase exactly once");

const expectedAuthentication = {
  metadataName: "authorization", schemePrefix: "Bearer ",
  expectedSubject: "epsx-analytics-service", expectedAudience: "epsx-identity-service",
  authorizerCallsOnAcceptedMetadata: 1, queryCallsOnSuccess: 1,
};
if (JSON.stringify(contract.authenticationContract) !== JSON.stringify(expectedAuthentication)) fail("authentication contract drifted");
const expectedWallet = {
  byteLength: 42, prefix: "0x", hexBytes: 40, mixedCaseHexAccepted: true,
  normalizedLowercaseExactlyOnce: true, trimmed: false, ensOrAlternateFormat: false,
  zeroAddressAccepted: true,
};
if (JSON.stringify(contract.walletContract) !== JSON.stringify(expectedWallet)) fail("wallet contract drifted");
const expectedStatuses = [
  ["missing-duplicate-nonascii-malformed-or-rejected-credential", "Unauthenticated", "workload authentication required"],
  ["authorizer-unavailable", "Unavailable", "workload authorization unavailable"],
  ["wrong-subject-or-audience", "PermissionDenied", "workload caller forbidden"],
  ["invalid-wallet", "InvalidArgument", "invalid wallet address"],
  ["query-unavailable", "Unavailable", "ranking authority unavailable"],
  ["query-corrupt", "Internal", "ranking authority returned invalid data"],
  ["query-unexpected", "Internal", "ranking authority failed"],
];
if (JSON.stringify(contract.statusContract) !== JSON.stringify(expectedStatuses.map(([id, code, message]) => ({ id, code, message })))) fail("status contract drifted");
for (const [, , message] of expectedStatuses) contains(production, `\"${message}\"`, `fixed status message ${message}`);
for (const token of [
  "Status::unauthenticated(AUTHENTICATION_REQUIRED)",
  "Status::unavailable(AUTHORIZATION_UNAVAILABLE)",
  "Status::permission_denied(CALLER_FORBIDDEN)",
  "Status::invalid_argument(INVALID_WALLET)",
  "ErrorKind::ServiceUnavailable => Status::unavailable(AUTHORITY_UNAVAILABLE)",
  "ErrorKind::InternalServerError => Status::internal(AUTHORITY_INVALID)",
  "_ => Status::internal(AUTHORITY_FAILED)",
]) contains(production, token, `exact status mapping ${token}`);
for (const token of ["error.message", "correlation_id", "format!(", "to_string()"] ) excludes(production.slice(production.indexOf("fn map_authorization_error")), token, `outward error detail ${token}`);

const proto = read(safePath("shared/proto/identity.proto", "identity proto"));
contains(proto, "string wallet = 1;", "request wallet field 1");
contains(proto, "int32 offset = 1;", "response offset field 1");
excludes(proto, "authorization", "protobuf authorization field");
excludes(proto, "credential", "protobuf credential field");
const main = read(safePath("shared/rust/epsx-identity-service/src/main.rs", "identity main"));
if (process.env.EPSX_A2_10_STATIC_ONLY === "1") contains(main, "Arc::new(FreePlanRankingOffsetService)", "historical runtime injection");
else contains(main, "Arc::new(UnavailableRankingOffsetService)", "fail-closed runtime injection");
excludes(main, "AuthenticatedRankingGrpcService", "authenticated composition runtime wiring");
excludes(main, "RankingWorkloadAuthorizer", "workload authorizer runtime wiring");

const currentTests = [
  "a2_10_missing_metadata_is_unauthenticated_before_authorizer_or_query",
  "a2_10_duplicate_or_malformed_bearer_is_rejected_before_authorizer_or_query",
  "a2_10_invalid_credential_precedes_wallet_validation_and_query",
  "a2_10_authorizer_unavailable_precedes_wallet_validation_and_query",
  "a2_10_wrong_subject_or_audience_is_permission_denied_before_query",
  "a2_10_invalid_evm_wallet_is_invalid_argument_without_query",
  "a2_10_mixed_case_wallet_is_normalized_once_and_queried_once",
  "a2_10_query_unavailable_maps_to_sanitized_unavailable",
  "a2_10_corrupt_query_maps_to_sanitized_internal",
  "a2_10_unexpected_query_error_is_sanitized_internal",
  "a2_10_authorization_uses_metadata_without_proto_field_changes",
];
const expectedTests = process.env.EPSX_A2_10_STATIC_ONLY === "1" ? contract.hermeticTests : currentTests;
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("hermetic test inventory drifted");
for (const name of expectedTests) {
  const matches = moduleSource.match(new RegExp(`async fn ${name}\\(`, "g")) || [];
  if (matches.length !== 1) fail(`hermetic test source is missing or ambiguous: ${name}`);
}
for (const token of [
  "assert_eq!(authorizer.calls(), 0)", "assert_eq!(authorizer.calls(), 1)",
  "assert_eq!(query.calls(), 0)", "assert_eq!(query.calls(), 1)",
  "MetadataValue::try_from(&b\"Bearer \\xff\"[..])",
  "vec![mixed.to_ascii_lowercase()]",
]) contains(tests, token, `fake counter/normalization coverage ${token}`);

const expectedScopes = [
  "apps/analytics/src/grpc_client.rs", "shared/proto/identity.proto",
  "shared/rust/epsx-identity-service/Cargo.toml", "Cargo.lock", "apps/backend/migrations",
  "services", "infrastructure", "shared/rust/dioxus_ui", "apps/frontend",
  "apps/admin-frontend", "apps/payments",
];
if (JSON.stringify(contract.unchangedPathScopes) !== JSON.stringify(expectedScopes)) fail("unchanged path-scope inventory drifted");

const currentStopIds = [
  "concrete-credential-verifier-and-issuer-trust-absent",
  "credential-issuance-storage-rotation-absent", "analytics-authorization-metadata-absent",
  "identity-workload-tls-absent", "store-schema-and-query-plan-unactivated",
  "runtime-fail-closed-until-authority-wired", "owner-delegation-binding-absent",
  "runtime-deployment-and-live-proof-absent", "production-actions-unauthorized",
];
const expectedStopIds = process.env.EPSX_A2_10_STATIC_ONLY === "1" ? contract.residualStops.map((item) => item.id) : currentStopIds;
exactIds(contract.residualStops, expectedStopIds, "residual STOP");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 9 || !contract.requiredExecutionOrder[0].startsWith("E01 ") || !contract.requiredExecutionOrder[8].startsWith("E09 ")) fail("execution order drifted");

if (!staticOnly) {
  const tracked = git("diff", "--name-only", expectedTarget.commit, "--", "shared/rust/epsx-identity-service").split("\n").filter(Boolean);
  const untracked = git("ls-files", "--others", "--exclude-standard", "--", "shared/rust/epsx-identity-service").split("\n").filter(Boolean);
  const identityDiff = [...new Set([...tracked, ...untracked])].sort();
  const expectedIdentityDiff = [
    "shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs",
    "shared/rust/epsx-identity-service/src/identity_service.rs",
    "shared/rust/epsx-identity-service/src/lib.rs",
    "shared/rust/epsx-identity-service/src/main.rs",
  ];
  if (JSON.stringify(identityDiff) !== JSON.stringify(expectedIdentityDiff)) fail("A2.10 identity implementation diff drifted from the current fail-closed boundary");
  // The historical A2.10 snapshot predates later migration-domain work. The
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
  unchangedEvidence: contract.unchangedEvidence.length,
  statusMappings: contract.statusContract.length,
  hermeticTests: contract.hermeticTests.length,
  residualStops: contract.residualStops,
}));
' "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT" "$STATIC_ONLY")" || exit 1

if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  test_list="$(cargo test --offline --locked -p epsx-identity-service --lib -- --list 2>&1)" || {
    printf '%s\n' "$test_list" >&2
    die "could not enumerate A2.10 library tests"
  }
  while IFS= read -r test_name; do
    [[ -n "$test_name" ]] || continue
    match="$(printf '%s\n' "$test_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    match_count="$(printf '%s\n' "$match" | sed '/^$/d' | wc -l | tr -d ' ')"
    [[ "$match_count" == "1" ]] || die "A2.10 hermetic test name is missing or ambiguous: $test_name"
    output="$(cargo test --offline --locked -p epsx-identity-service --lib "$match" -- --exact 2>&1)" || {
      printf '%s\n' "$output" >&2
      die "A2.10 hermetic test failed: $test_name"
    }
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" || die "A2.10 hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  check_output="$(cargo check --offline --locked -p epsx-identity-service --lib 2>&1)" || {
    printf '%s\n' "$check_output" >&2
    die "identity library offline check failed"
  }
fi

invariants="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.invariants.length));' "$CONTRACT")"
tests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.hermeticTests.length));' "$CONTRACT")"
digests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.implementationEvidence.length+c.unchangedEvidence.length));' "$CONTRACT")"
statuses="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.statusContract.length));' "$CONTRACT")"
stops="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.residualStops.length));' "$CONTRACT")"

case "$MODE" in
  integrity)
    printf 'authenticated-ranking-rpc: PASS; %s invariants; %s exact hermetic tests; %s frozen digests; %s status mappings; %s residual STOPs\n' "$invariants" "$tests" "$digests" "$statuses" "$stops"
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    printf 'authenticated-ranking-rpc: LIMIT; %s residual STOPs remain; readiness exit 3\n' "$stops" >&2
    exit 3
    ;;
esac
