#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="${EPSX_A2_11_REPO_ROOT:-$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)}"
EVIDENCE_ROOT_RAW="${EPSX_A2_11_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_11_STATIC_ONLY:-0}"

die() {
  echo "ranking-store-extraction: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-11-ranking-store-extraction.json"
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
  console.error(`ranking-store-extraction: ERROR: ${message}`);
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
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-11-ranking-store-extraction" || contract.contractId !== "A2.11-ranking-store-extraction") fail("unexpected contract identity");
if (contract.purpose !== "deterministic-hermetic-library-only-ranking-store-extraction-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel drifted");
const safetyKeys = ["database", "schemaCatalog", "migration", "redis", "identityNetwork", "rpc", "browser", "serviceListener", "deployment", "production"];
if (JSON.stringify(Object.keys(contract.safety)) !== JSON.stringify(safetyKeys) || safetyKeys.some((key) => contract.safety[key] !== false)) fail("safety sentinel drifted");

const expectedSource = { ref: "origin/development", commit: "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" };
if (contract.sourceBaseline?.ref !== expectedSource.ref || contract.sourceBaseline?.commit !== expectedSource.commit) fail("source baseline drifted");
if (git("rev-parse", `${expectedSource.commit}^{commit}`) !== expectedSource.commit) fail("immutable source commit is missing");
if (!Array.isArray(contract.sourceBaseline.evidence) || contract.sourceBaseline.evidence.length !== 5) fail("source evidence inventory drifted");
for (const item of contract.sourceBaseline.evidence) {
  if (git("rev-parse", `${expectedSource.commit}:${item.file}`) !== item.blob) fail(`${item.id}: source blob drifted`);
  if (!git("show", `${expectedSource.commit}:${item.file}`).includes(item.anchor)) fail(`${item.id}: source anchor missing`);
}

const expectedTarget = { ref: "migration/dioxus-microservices", commit: "005a604542271050279a6190fc00eada00f32137" };
if (contract.targetBase?.ref !== expectedTarget.ref || contract.targetBase?.commit !== expectedTarget.commit) fail("target base drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("immutable target-base commit is missing");
if (!Array.isArray(contract.targetBase.evidence) || contract.targetBase.evidence.length !== 11) fail("target-base evidence inventory drifted");
for (const item of contract.targetBase.evidence) {
  if (git("rev-parse", `${expectedTarget.commit}:${item.file}`) !== item.blob) fail(`${item.id}: target-base blob drifted`);
  if (!git("show", `${expectedTarget.commit}:${item.file}`).includes(item.anchor)) fail(`${item.id}: target-base anchor missing`);
}

const invariantIds = [
  "library-only-store-package", "actual-repository-contract-preserved", "workspace-member-once",
  "minimal-store-dependencies", "owned-cloneable-pool", "backend-compatibility-reexport",
  "sql-shape-preserved", "strict-decoder-preserved", "twelve-exact-tests-moved",
  "identity-no-store-dependency", "no-schema-migration-runtime-change", "offline-hermetic-only",
];
exactIds(contract.invariants, invariantIds, "invariant");

const expectedImplementation = [
  ["impl-workspace-manifest", "Cargo.toml", "bc46c71ec8efc6d7c7af81ffe0c4e0813bfcb1427d6e8b616b3933178c6a947a"],
  ["impl-workspace-lock", "Cargo.lock", "a3a40aabc05feb2a9ecb2d5e4aab7e689eb51764cf8147bff7759a366f5ba95c"],
  ["impl-backend-manifest", "apps/backend/Cargo.toml", "4b4cc24581b0b575fe5dc6cada9165f27b9cbd60c0d260a2d4f6488b961a936c"],
  ["impl-backend-compatibility-reexport", "apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs", "4e3e609262aa9c7d73c2e9f01dce41ba2a8c531120d145d8ba88be03dc563f45"],
  ["impl-ranking-store-manifest", "shared/rust/epsx-ranking-store/Cargo.toml", "c2d0bb14dd1b97a1ef8fe99b31af83acd833bf7ffac05d32b4ca35e67cfe225b"],
  ["impl-ranking-store-library", "shared/rust/epsx-ranking-store/src/lib.rs", "8c2228460a82e54972d0200991ee4bea73d27bcc1380fb67925e4323b93c5ee1"],
  ["impl-shared-repository-contract", "shared/rust/epsx-contracts/src/ranking_entitlement_snapshot.rs", "9ba917a2bb2646097162371e19f6c1b6f44d41f65b1f32dce5193614f5baadbe"],
  ["impl-identity-manifest-unchanged", "shared/rust/epsx-identity-service/Cargo.toml", "54f7020be797a137a4c69d1e3fbccf0d21f88923a616bd48f0757aa770cf7c8f"],
  ["impl-identity-auth-composition-unchanged", "shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs", "1d9fc78e74d9959030bbd83c27989f987b7f200b8c70f8d0ad8110b36db60a58"],
  ["impl-identity-runtime-main-unchanged", "shared/rust/epsx-identity-service/src/main.rs", "e3b9980bf99434d67c3276b7dec329ba9453b226e72992d390f11c03bfaa78bc"],
  ["impl-identity-free-service-unchanged", "shared/rust/epsx-identity-service/src/identity_service.rs", "f172df2a9998cc773b4299e2760164ee9d9a6c225680260b6c17bdc27f8da320"],
  ["impl-identity-proto-unchanged", "shared/proto/identity.proto", "f33f7256048403c79219913051347d85d05238e9c62269e37e8bffdae9f69d23"],
];
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("implementation evidence inventory drifted");
const contentByFile = new Map();
for (let index = 0; index < expectedImplementation.length; index += 1) {
  const [id, file, digest] = expectedImplementation[index];
  if (JSON.stringify(contract.implementationEvidence[index]) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: implementation tuple drifted`);
  const content = read(safePath(file, id));
  if (sha256(content) !== digest) fail(`${id}: implementation digest drifted`);
  contentByFile.set(file, content);
}

const fixturePath = safePath(contract.fixtureEvidence?.file, "fixture");
const fixture = read(fixturePath);
if (contract.fixtureEvidence.sha256 !== "3f8ddaa93047f999459239a148da8dfbdf26a2d338a58b7c5a6a7f4481ae79fa" || sha256(fixture) !== contract.fixtureEvidence.sha256) fail("fixture digest drifted");
let fixtureJson;
try { fixtureJson = JSON.parse(fixture); } catch { fail("fixture JSON is invalid"); }
if (!Array.isArray(fixtureJson.cases) || fixtureJson.cases.length !== 21 || contract.fixtureEvidence.fixtureCount !== 21) fail("fixture case inventory drifted");

const workspace = contentByFile.get("Cargo.toml");
const lock = contentByFile.get("Cargo.lock");
const backendManifest = contentByFile.get("apps/backend/Cargo.toml");
const shim = contentByFile.get("apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs");
const storeManifest = contentByFile.get("shared/rust/epsx-ranking-store/Cargo.toml");
const store = contentByFile.get("shared/rust/epsx-ranking-store/src/lib.rs");
const repositoryContract = contentByFile.get("shared/rust/epsx-contracts/src/ranking_entitlement_snapshot.rs");
const identityManifest = contentByFile.get("shared/rust/epsx-identity-service/Cargo.toml");
const identityAuth = contentByFile.get("shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs");
const identityMain = contentByFile.get("shared/rust/epsx-identity-service/src/main.rs");
const identityService = contentByFile.get("shared/rust/epsx-identity-service/src/identity_service.rs");
const proto = contentByFile.get("shared/proto/identity.proto");

if ((workspace.match(/"shared\/rust\/epsx-ranking-store"/g) || []).length !== 1) fail("workspace must contain epsx-ranking-store exactly once");
contains(backendManifest, "epsx-ranking-store = { path = \"../../shared/rust/epsx-ranking-store\" }", "backend ranking-store dependency");
contains(lock, "name = \"epsx-ranking-store\"", "ranking-store lock package");
if ((lock.match(/name = "epsx-ranking-store"/g) || []).length !== 1) fail("lockfile must contain one ranking-store package");

const expectedRuntimeDeps = ["async-trait", "diesel", "diesel-async", "epsx-contracts", "epsx-database-pools", "serde_json"];
if (contract.packageContract?.name !== "epsx-ranking-store" || JSON.stringify(contract.packageContract.runtimeDependencies) !== JSON.stringify(expectedRuntimeDeps) || JSON.stringify(contract.packageContract.devDependencies) !== JSON.stringify(["serde"]) || contract.packageContract.binaryTargets !== 0) fail("package contract drifted");
for (const dep of expectedRuntimeDeps) contains(storeManifest, dep, `store dependency ${dep}`);
contains(storeManifest, "[dev-dependencies]", "store dev-dependency section");
contains(storeManifest, "serde.workspace = true", "test-only serde dependency");
for (const token of ["[[bin]]", "[features]", "tokio", "axum", "tonic", "reqwest", "dotenv", "config", "identity-service"]) excludes(storeManifest, token, `store runtime surface ${token}`);

const expectedShim = "//! Compatibility exports for the extracted ranking-entitlement snapshot store.\n\npub use epsx_ranking_store::{\n    PostgresRankingEntitlementSnapshotRepository, RANKING_ENTITLEMENT_SNAPSHOT_SQL,\n};\n";
if (shim !== expectedShim) fail("backend compatibility module must be an exact two-symbol re-export");
contains(repositoryContract, "pub trait RankingEntitlementSnapshotRepository: Send + Sync", "actual repository trait");
contains(repositoryContract, ") -> Result<RankingEntitlementSnapshot, RankingEntitlementSnapshotError>;", "actual repository result contract");
for (const token of contract.repositoryContract.forbiddenInventedTypes) excludes(store, token, `invented repository type ${token}`);

contains(store, "use epsx_database_pools::TlsPool;", "shared pool import");
contains(store, "pool: TlsPool,", "owned pool field");
contains(store, "pub fn new(pool: TlsPool) -> Self", "owned pool constructor");
excludes(store, "Arc<&\u0027static TlsPool>", "leaked static pool wrapper");
excludes(store, "std::sync::Arc", "monolith Arc pool ownership");

const baseAdapter = git("show", `${expectedTarget.commit}:apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs`);
const normalizedBase = `${baseAdapter}\n`
  .replace("use std::{collections::BTreeMap, sync::Arc};", "use std::collections::BTreeMap;")
  .replace("use crate::prelude::TlsPool;", "use epsx_database_pools::TlsPool;")
  .replaceAll("Arc<&\u0027static TlsPool>", "TlsPool")
  .replace("../../../../../../docs/migration/fixtures/a2-8-ranking-entitlement-rows.json", "../../../../docs/migration/fixtures/a2-8-ranking-entitlement-rows.json")
  .replace("use serde_json::Value;\n\nuse epsx_database_pools::TlsPool;", "use epsx_database_pools::TlsPool;\nuse serde_json::Value;");
if (normalizedBase !== store) fail("extracted adapter drifted from the normalized immutable source");

const sqlAnchor = "pub const RANKING_ENTITLEMENT_SNAPSHOT_SQL: &str = r#\"";
const sqlStart = store.indexOf(sqlAnchor);
if (sqlStart < 0) fail("ranking SQL constant is missing");
const sqlBodyStart = sqlStart + sqlAnchor.length;
const sqlEnd = store.indexOf("\"#;", sqlBodyStart);
if (sqlEnd < 0) fail("ranking SQL terminator is missing");
const sql = store.slice(sqlBodyStart, sqlEnd);
if (contract.sqlEvidence?.sha256 !== "18d04cc6456e545d1a39c22ab85983ac4de4a26922882e2128721ffa2f418574" || sha256(sql) !== contract.sqlEvidence.sha256) fail("ranking SQL digest drifted");
if (sql.includes(";") || (sql.match(/LEFT JOIN public\./g) || []).length !== 4 || (sql.match(/\$1/g) || []).length !== 1) fail("ranking SQL shape drifted");
for (const table of contract.sqlEvidence.qualifiedTables) contains(sql, table, `schema-qualified table ${table}`);
for (const token of [" INSERT ", " UPDATE ", " DELETE ", " MERGE ", " FOR UPDATE", " WHERE "]) excludes(` ${sql.toUpperCase()} `, token, `mutating/filter SQL token ${token}`);

const expectedTests = [
  "a2_8_sql_is_one_read_only_schema_qualified_statement",
  "a2_8_sentinel_empty_wallet_maps_to_empty_snapshot",
  "a2_8_grouping_and_permissions_are_stable",
  "a2_8_equivalent_duplicate_rows_are_idempotent",
  "a2_8_zero_rows_are_corrupt_not_empty_success",
  "a2_8_conflicting_duplicate_rows_are_corrupt",
  "a2_8_missing_plan_is_preserved_without_invented_facts",
  "a2_8_inactive_expired_and_inactive_permission_facts_are_preserved",
  "a2_8_metadata_shapes_remain_missing_integer_or_invalid",
  "a2_8_dangling_and_partial_rows_are_corrupt",
  "a2_8_sentinel_cardinality_and_mixed_rows_are_corrupt",
  "a2_8_wallet_and_observation_inconsistency_are_corrupt",
];
if (JSON.stringify(contract.hermeticTests) !== JSON.stringify(expectedTests)) fail("hermetic test inventory drifted");
for (const name of expectedTests) {
  if ((store.match(new RegExp(`fn ${name}\\(`, "g")) || []).length !== 1) fail(`hermetic test source is missing or ambiguous: ${name}`);
}

for (const token of ["epsx-ranking-store", "epsx_ranking_store"]) excludes(identityManifest, token, "identity store dependency");
contains(identityAuth, "pub struct AuthenticatedRankingGrpcService", "unchanged authenticated composition");
contains(identityMain, "Arc::new(FreePlanRankingOffsetService)", "unchanged always-Free runtime injection");
contains(identityService, "Ok(RankingOffset::free_plan())", "unchanged always-Free implementation");
contains(proto, "rpc GetWalletRankingOffset(GetWalletRankingOffsetRequest)", "unchanged identity RPC");

const stopIds = [
  "schema-adoption-uncertified", "generated-schema-filter-gap", "lower-wallet-functional-index-absent",
  "database-execution-absent", "query-plan-and-bound-absent", "mvcc-concurrency-unproved",
  "reconciliation-unproved", "identity-store-wiring-absent", "a2-10-concrete-auth-absent",
  "identity-runtime-still-always-free", "ranking-event-durability-absent", "ui-bff-readiness-unproved",
  "runtime-deployment-proof-absent", "forbidden-domain-changes-absent", "production-actions-unauthorized",
];
exactIds(contract.residualStops, stopIds, "residual STOP");
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 10 || !contract.requiredExecutionOrder[0].startsWith("E01 ") || !contract.requiredExecutionOrder[9].startsWith("E10 ")) fail("execution order drifted");

if (!staticOnly) {
  const scope = ["Cargo.toml", "Cargo.lock", "apps/backend/Cargo.toml", "apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs", "shared/rust/epsx-ranking-store"];
  const tracked = git("diff", "--name-only", expectedTarget.commit, "--", ...scope).split("\n").filter(Boolean);
  const untracked = git("ls-files", "--others", "--exclude-standard", "--", ...scope).split("\n").filter(Boolean);
  const actual = [...new Set([...tracked, ...untracked])].sort();
  const expected = [
    "Cargo.lock", "Cargo.toml", "apps/backend/Cargo.toml",
    "apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs",
    "shared/rust/epsx-ranking-store/Cargo.toml", "shared/rust/epsx-ranking-store/src/lib.rs",
  ];
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail("A2.11 implementation diff inventory drifted");
  for (const prefix of contract.forbiddenChangePrefixes) {
    const changed = git("diff", "--name-only", expectedTarget.commit, "--", prefix);
    const others = git("ls-files", "--others", "--exclude-standard", "--", prefix);
    if (changed || others) fail(`forbidden A2.11 path changed: ${prefix}`);
  }
}

process.stdout.write(JSON.stringify({
  artifact: contract.artifact,
  targetBase: contract.targetBase.commit,
  productionReady: contract.productionReady,
  readinessExit: contract.readinessExit,
  sourceEvidence: contract.sourceBaseline.evidence.length,
  baseEvidence: contract.targetBase.evidence.length,
  invariants: contract.invariants.length,
  implementationEvidence: contract.implementationEvidence.length,
  fixtureCases: contract.fixtureEvidence.fixtureCount,
  hermeticTests: contract.hermeticTests.length,
  residualStops: contract.residualStops,
}));
' "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT" "$STATIC_ONLY")" || exit 1

if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  test_list="$(cargo test --offline --locked -p epsx-ranking-store --lib -- --list 2>&1)" || {
    printf '%s\n' "$test_list" >&2
    die "could not enumerate ranking-store tests"
  }
  while IFS= read -r test_name; do
    [[ -n "$test_name" ]] || continue
    match="$(printf '%s\n' "$test_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    match_count="$(printf '%s\n' "$match" | sed '/^$/d' | wc -l | tr -d ' ')"
    [[ "$match_count" == "1" ]] || die "A2.11 hermetic test name is missing or ambiguous: $test_name"
    output="$(cargo test --offline --locked -p epsx-ranking-store --lib "$match" -- --exact 2>&1)" || {
      printf '%s\n' "$output" >&2
      die "A2.11 hermetic test failed: $test_name"
    }
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" || die "A2.11 hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  store_check="$(cargo check --offline --locked -p epsx-ranking-store --lib 2>&1)" || {
    printf '%s\n' "$store_check" >&2
    die "ranking-store offline check failed"
  }
  backend_check="$(cargo check --offline --locked -p epsx --lib 2>&1)" || {
    printf '%s\n' "$backend_check" >&2
    die "backend compatibility offline check failed"
  }
fi

invariants="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.invariants.length));' "$CONTRACT")"
tests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.hermeticTests.length));' "$CONTRACT")"
evidence="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.implementationEvidence.length));' "$CONTRACT")"
stops="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.residualStops.length));' "$CONTRACT")"

case "$MODE" in
  integrity)
    printf 'ranking-store-extraction: PASS; %s invariants; %s exact hermetic tests; %s implementation digests; %s residual STOPs\n' "$invariants" "$tests" "$evidence" "$stops"
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    printf 'ranking-store-extraction: LIMIT; %s residual STOPs remain; readiness exit 3\n' "$stops" >&2
    exit 3
    ;;
esac
