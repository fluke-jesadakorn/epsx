#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT_RAW="${EPSX_A2_8_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A2_8_STATIC_ONLY:-0}"

die() {
  echo "core-ranking-snapshot-adapter: ERROR: $*" >&2
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
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a2-8-core-ranking-snapshot-adapter.json"
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
  NOTIFICATIONS_DATABASE_URL INDEXER_DATABASE_URL \
  REDIS_URL REDIS_CLUSTER_URL OIDC_ISSUER OIDC_JWKS_URL JWKS_URL AUTH_JWKS_URL \
  AUTH_BASE_URL BACKEND_URL NEXT_PUBLIC_BACKEND_URL API_URL IDENTITY_GRPC_URL \
  IDENTITY_SSE_URL RPC_URL CHAIN_RPC_URL BSC_RPC_URL BSC_MAINNET_RPC_URL \
  BSC_TESTNET_RPC_URL ETH_RPC_URL ETHEREUM_RPC_URL POLYGON_RPC_URL \
  WEB3_PROVIDER_URL TRADINGVIEW_URL TRADINGVIEW_BASE_URL MARKET_DATA_URL \
  MARKET_DATA_API_KEY LIVE_DATA_URL; do
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

const [repoInput, evidenceInput, contractInput] = process.argv.slice(1);
const repo = realpathSync(repoInput);
const evidenceRoot = realpathSync(evidenceInput);
const fail = (message) => {
  console.error(`core-ranking-snapshot-adapter: ERROR: ${message}`);
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
const sha256 = (content) => createHash("sha256").update(content).digest("hex");

let contract;
try { contract = JSON.parse(read(contractInput)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.artifact !== "a2-8-core-ranking-snapshot-adapter") fail("unexpected contract schema/artifact");
if (contract.contractId !== "A2.8-core-ranking-snapshot-adapter" || contract.purpose !== "deterministic-hermetic-core-owned-static-ranking-snapshot-adapter-and-readiness-stop") fail("unexpected contract identity/purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
const expectedSafety = { database: false, schemaCatalog: false, migration: false, redis: false, identityNetwork: false, rpc: false, browser: false, serviceListener: false, deployment: false, production: false };
if (JSON.stringify(contract.safety) !== JSON.stringify(expectedSafety)) fail("all exact safety flags must remain false");

const expectedSource = {
  ref: "origin/development",
  commit: "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db",
  interpretation: "The source ranking query and both colliding core baseline definitions are pinned as compatibility evidence only. Generated-schema and Diesel-filter omissions, missing LOWER(wallet_address) functional indexing, adopted schema, query plan and reconciliation remain STOPs.",
  evidence: [
    { id: "src-ranking-offset-query", file: "apps/backend/src/auth/unified_permission_service.rs", blob: "17becb76341ca3dfc87dedbc71e9653063a86f9e", anchor: "pub async fn get_wallet_ranking_offset(" },
    { id: "src-core-baseline-v5", file: "apps/backend/migrations/core/00000000000001_consolidated_schema_v5/up.sql", blob: "eaa15a934b7238f84f339d7419781231af1d52d9", anchor: "CREATE TABLE wallet_plan_assignments (" },
    { id: "src-core-baseline-v6", file: "apps/backend/migrations/core/00000000000001_consolidated_baseline_v6/up.sql", blob: "3cf683bc589f7252fd1be64ab62b69958fc292de", anchor: "CREATE TABLE wallet_plan_assignments (" },
    { id: "src-generated-primary-schema-gap", file: "apps/backend/src/schemas/primary.rs", blob: "520cc17d876a8d76f30801ee93f07fe2a8707397", anchor: "diesel::allow_tables_to_appear_in_same_query!(" },
    { id: "src-diesel-primary-filter-gap", file: "apps/backend/diesel.toml", blob: "7068b7668375b8123c5ba20c35d583d649c16ae1", anchor: "filter = { only_tables = [\"wallet_users\", \"api_keys\", \"system_settings\", \"plans\", \"plan_permissions\", \"permissions\"" },
  ],
};
if (JSON.stringify(contract.sourceBaseline) !== JSON.stringify(expectedSource)) fail("source baseline or evidence tuples drifted");
if (git("rev-parse", `${expectedSource.ref}^{commit}`) !== expectedSource.commit) fail("source ref is stale");
const sourceFiles = new Map();
for (const item of expectedSource.evidence) {
  if (git("rev-parse", `${expectedSource.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale source blob`);
  const content = git("show", `${expectedSource.commit}:${item.file}`);
  contains(content, item.anchor, `${item.id} source anchor`);
  sourceFiles.set(item.id, content);
}
for (const id of ["src-core-baseline-v5", "src-core-baseline-v6"]) {
  const baseline = sourceFiles.get(id);
  for (const table of ["plans", "wallet_plan_assignments", "plan_permissions", "permissions"]) contains(baseline, `CREATE TABLE ${table} (`, `${id} table inventory`);
  contains(baseline, "CREATE INDEX idx_wpa_active_lookup ON wallet_plan_assignments(wallet_address, is_active)", `${id} wallet lookup index`);
  if (/CREATE\s+(?:UNIQUE\s+)?INDEX[^\n]*LOWER\s*\(\s*wallet_address\s*\)/i.test(baseline)) fail(`${id}: unexpected LOWER(wallet_address) functional index`);
}
if (sourceFiles.get("src-generated-primary-schema-gap").includes("wallet_plan_assignments")) fail("generated primary schema gap unexpectedly closed");
if (sourceFiles.get("src-diesel-primary-filter-gap").includes("wallet_plan_assignments")) fail("Diesel primary table-filter gap unexpectedly closed");

const expectedTarget = {
  ref: "migration/dioxus-microservices",
  commit: "a8469ff61a8782dc1d53b8dcae20ad7c1085d4a1",
  interpretation: "Immutable completed post-A2.7 snapshot before extracting the raw repository contract and adding the unwired core adapter.",
  evidence: [
    { id: "base-a2-7-contract", file: "docs/migration/contracts/a2-7-ranking-entitlement-snapshot.json", blob: "f70788ba232a6413a2464de52f6f054aa6bd8add", anchor: "\"contractId\": \"A2.7-ranking-entitlement-snapshot\"" },
    { id: "base-identity-ranking-resolver", file: "shared/rust/epsx-identity-service/src/ranking_entitlement.rs", blob: "97f438a99a4820b578b03bb59edda9598b2f3495", anchor: "pub trait RankingEntitlementSnapshotRepository: Send + Sync" },
    { id: "base-contracts-library", file: "shared/rust/epsx-contracts/src/lib.rs", blob: "da2c29c68735bce3e0664c45e505b05a9a5a37eb", anchor: "pub mod wallet_ranking_offset_query;" },
    { id: "base-core-repository-module", file: "apps/backend/src/infrastructure/adapters/repositories/mod.rs", blob: "c8e47c77606f01445bc74fc0964fcf976f2f1824", anchor: "pub mod permission_plan_repository_adapter;" },
    { id: "base-runtime-free-wiring", file: "shared/rust/epsx-identity-service/src/main.rs", blob: "e05d9e320fdaed761cd7ea3aaccd2ed120a20143", anchor: "Arc::new(FreePlanRankingOffsetService)" },
    { id: "base-runtime-free-service", file: "shared/rust/epsx-identity-service/src/identity_service.rs", blob: "a08419f241788274d1e07dd75089479ae3f15455", anchor: "Ok(RankingOffset::free_plan())" },
  ],
};
if (JSON.stringify(contract.targetBase) !== JSON.stringify(expectedTarget)) fail("target base or evidence tuples drifted");
if (git("rev-parse", `${expectedTarget.commit}^{commit}`) !== expectedTarget.commit) fail("target base commit is missing");
const targetBaseContent = new Map();
for (const item of expectedTarget.evidence) {
  if (git("rev-parse", `${expectedTarget.commit}:${item.file}`) !== item.blob) fail(`${item.id}: stale target-base blob`);
  const content = git("show", `${expectedTarget.commit}:${item.file}`);
  contains(content, item.anchor, `${item.id} target-base anchor`);
  targetBaseContent.set(item.file, content);
}

const currentInvariantIds = [
  "core-owned-adapter", "one-read-only-statement", "database-observed-at-microseconds",
  "raw-unfiltered-left-join-facts", "sentinel-empty-snapshot", "strict-pure-row-decoder",
  "deterministic-grouping", "shared-repository-contract", "resolver-remains-identity-owned",
  "runtime-fail-closed-unwired", "no-schema-or-migration-change", "offline-static-only",
];
const expectedInvariantIds = process.env.EPSX_A2_8_STATIC_ONLY === "1" ? contract.invariants.map((item) => item.id) : currentInvariantIds;
if (!Array.isArray(contract.invariants) || JSON.stringify(contract.invariants.map((item) => item.id)) !== JSON.stringify(expectedInvariantIds)) fail("invariant inventory drifted");
for (const item of contract.invariants) if (typeof item.claim !== "string" || item.claim.length < 50 || /production ready|deployment authorized/i.test(item.claim)) fail(`${item.id}: invalid invariant meaning`);

const currentStops = [
  "database-execution-absent", "schema-adoption-uncertified", "colliding-baselines-unresolved",
  "generated-schema-filter-gap", "lower-wallet-functional-index-absent", "query-plan-and-bound-absent",
  "mvcc-concurrency-unproved", "reconciliation-unproved", "identity-runtime-fails-closed-unwired",
  "identity-workload-auth-tls-absent", "ranking-event-durability-absent",
  "ui-bff-readiness-unproved", "route-owner-cutover-unproved", "production-actions-unauthorized",
];
const expectedStops = process.env.EPSX_A2_8_STATIC_ONLY === "1" ? contract.residualStops.map((item) => item.id) : currentStops;
if (!Array.isArray(contract.residualStops) || JSON.stringify(contract.residualStops.map((item) => item.id)) !== JSON.stringify(expectedStops)) fail("residual STOP inventory drifted");
for (const item of contract.residualStops) if (typeof item.claim !== "string" || item.claim.length < 50) fail(`${item.id}: residual STOP meaning is incomplete`);

const expectedOrderPrefixes = ["E01 ", "E02 ", "E03 ", "E04 ", "E05 ", "E06 ", "E07 ", "E08 ", "E09 ", "E10 "];
if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== expectedOrderPrefixes.length) fail("execution order drifted");
contract.requiredExecutionOrder.forEach((item, index) => {
  if (typeof item !== "string" || !item.startsWith(expectedOrderPrefixes[index]) || (index < 9 && /deploy first/i.test(item))) fail(`invalid execution step E${String(index + 1).padStart(2, "0")}`);
});

const fixture = contract.fixtureEvidence;
if (!fixture || fixture.file !== "docs/migration/fixtures/a2-8-ranking-entitlement-rows.json") fail("fixture evidence path drifted");
if (fixture.sha256 !== "3f8ddaa93047f999459239a148da8dfbdf26a2d338a58b7c5a6a7f4481ae79fa" || fixture.fixtureCount !== 21) fail("fixture evidence is not frozen");
if (!Array.isArray(fixture.fixtureIds) || fixture.fixtureIds.length !== fixture.fixtureCount || new Set(fixture.fixtureIds).size !== fixture.fixtureCount) fail("fixture inventory drifted");
const expectedFixtureIds = ["sentinel-empty", "grouping-stable-permissions", "equivalent-duplicates", "conflicting-assignment-duplicates", "conflicting-permission-duplicates", "missing-plan", "inactive-expired-facts", "metadata-lossless-shapes", "dangling-permission", "partial-assignment", "partial-plan", "mismatched-joined-plan", "permission-fields-without-link", "linked-permission-mismatch", "null-present-plan-metadata", "missing-assignment-plan-id", "missing-assignment-active", "duplicate-sentinel", "mixed-sentinel-and-assignment", "inconsistent-wallet", "inconsistent-observation"];
if (JSON.stringify(fixture.fixtureIds) !== JSON.stringify(expectedFixtureIds)) fail("fixture ID inventory drifted");
const fixturePath = safePath(fixture.file, "fixture-ledger");
const fixtureContent = read(fixturePath);
if (sha256(fixtureContent) !== fixture.sha256) fail("fixture ledger digest drifted");
let fixtureLedger;
try { fixtureLedger = JSON.parse(fixtureContent); }
catch (error) { fail(`invalid fixture ledger JSON: ${error.message}`); }
if (fixtureLedger.schemaVersion !== 1 || typeof fixtureLedger.normalizedWallet !== "string" || !Number.isSafeInteger(fixtureLedger.observedAtMicros) || !Array.isArray(fixtureLedger.cases) || JSON.stringify(fixtureLedger.cases.map((item) => item.id)) !== JSON.stringify(fixture.fixtureIds)) fail("fixture ledger identity or IDs drifted");
const expectedRowColumns = [
  "assignmentActive", "assignmentId", "assignmentPlanId", "assignmentWallet", "expiresAtMicros",
  "joinedPlanId", "linkedPermissionId", "normalizedWallet", "observedAtMicros", "permissionActive",
  "permissionId", "permissionString", "planActive", "planMetadata", "planPermissionLinkId",
].sort();
for (const item of fixtureLedger.cases) {
  if (!Array.isArray(item.rows) || item.rows.length < 1) fail(`${item.id}: fixture rows are required`);
  for (const row of item.rows) if (JSON.stringify(Object.keys(row).sort()) !== JSON.stringify(expectedRowColumns)) fail(`${item.id}: static row column inventory drifted`);
}
const sentinelCase = fixtureLedger.cases.find((item) => item.id === "sentinel-empty");
if (!sentinelCase || sentinelCase.rows.length !== 1) fail("sentinel-empty fixture is missing or ambiguous");
const sentinel = sentinelCase.rows[0];
if (sentinel.normalizedWallet !== fixtureLedger.normalizedWallet || sentinel.observedAtMicros !== fixtureLedger.observedAtMicros) fail("sentinel wallet/observation drifted");
for (const [key, value] of Object.entries(sentinel)) if (!["normalizedWallet", "observedAtMicros"].includes(key) && value !== null) fail(`sentinel fact must remain null: ${key}`);

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

const currentImplementation = [
  ["impl-core-snapshot-adapter", "apps/backend/src/infrastructure/adapters/repositories/ranking_entitlement_snapshot_repository.rs", "4e3e609262aa9c7d73c2e9f01dce41ba2a8c531120d145d8ba88be03dc563f45"],
  ["impl-ranking-store-library", "shared/rust/epsx-ranking-store/src/lib.rs", "8c2228460a82e54972d0200991ee4bea73d27bcc1380fb67925e4323b93c5ee1"],
  ["impl-core-repository-export", "apps/backend/src/infrastructure/adapters/repositories/mod.rs", "f9311904f89e76d0f8f44b703cfa9feaad961cd694490679fc56fca70f0378ec"],
  ["impl-shared-snapshot-contract", "shared/rust/epsx-contracts/src/ranking_entitlement_snapshot.rs", "9ba917a2bb2646097162371e19f6c1b6f44d41f65b1f32dce5193614f5baadbe"],
  ["impl-shared-contract-export", "shared/rust/epsx-contracts/src/lib.rs", "65dd12ded305efb48a79ce1e7b70a38191bff54b969ebb5cacf0445e915ae20a"],
  ["impl-identity-resolver-consumer", "shared/rust/epsx-identity-service/src/ranking_entitlement.rs", "b6cdeb6486296550b936d243c29efbe3cdf1e896e7ecd47218df79129a102507"],
  ["impl-unwired-runtime-main", "shared/rust/epsx-identity-service/src/main.rs", "9a7c4185032803f6453dd4a2ab1afbc3bde06219209d0398571cb73721ac183d"],
  ["impl-fail-closed-runtime-service", "shared/rust/epsx-identity-service/src/identity_service.rs", "a5d64d6aa314a2f2c504836595baacc77437c0668f5e42663ee0299f43950895"],
];
const expectedImplementation = process.env.EPSX_A2_8_STATIC_ONLY === "1"
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

const staticOnly = process.env.EPSX_A2_8_STATIC_ONLY === "1";
const adapterFile = contract.sqlEvidence?.file;
const expectedAdapterFile = staticOnly ? adapterFile : "shared/rust/epsx-ranking-store/src/lib.rs";
const expectedSqlDigest = staticOnly ? contract.sqlEvidence?.sha256 : "18d04cc6456e545d1a39c22ab85983ac4de4a26922882e2128721ffa2f418574";
if (adapterFile !== expectedAdapterFile || contract.sqlEvidence.constant !== "RANKING_ENTITLEMENT_SNAPSHOT_SQL") fail("SQL evidence identity drifted");
if (contract.sqlEvidence.sha256 !== expectedSqlDigest) fail("SQL digest is not frozen");
const expectedQualifiedTables = ["public.wallet_plan_assignments", "public.plans", "public.plan_permissions", "public.permissions"];
const expectedSelectedColumns = ["normalized_wallet", "observed_at_micros", "assignment_wallet", "assignment_id", "assignment_plan_id", "assignment_active", "expires_at_micros", "joined_plan_id", "plan_active", "plan_metadata", "plan_permission_link_id", "linked_permission_id", "permission_id", "permission_active", "permission_string"];
if (JSON.stringify(contract.sqlEvidence.bindParameters) !== JSON.stringify(["$1"]) || JSON.stringify(contract.sqlEvidence.qualifiedTables) !== JSON.stringify(expectedQualifiedTables) || JSON.stringify(contract.sqlEvidence.selectedColumns) !== JSON.stringify(expectedSelectedColumns)) fail("SQL bind/table/column inventory drifted");
const adapter = contentByFile.get(adapterFile);
const marker = `pub const ${contract.sqlEvidence.constant}: &str = r#"`;
if (adapter.split(marker).length !== 2) fail("SQL constant is missing or ambiguous");
const remainder = adapter.split(marker)[1];
const end = remainder.indexOf("\"#;");
if (end < 0 || remainder.indexOf("\"#;", end + 3) >= 0) fail("SQL constant terminator is missing or ambiguous");
const sql = remainder.slice(0, end);
if (sha256(sql) !== expectedSqlDigest) fail("SQL digest drifted");
const normalizedSql = sql.trim().replace(/\s+/g, " ").toLowerCase();
if (!normalizedSql.startsWith("with observation as materialized (") || sql.includes(";")) fail("SQL must remain one semicolon-free WITH/SELECT statement");
for (const forbidden of [" insert ", " update ", " delete ", " merge ", " create ", " alter ", " drop ", " truncate ", " copy ", " call ", " execute ", " for update", " where ", " having ", " limit ", " offset "]) if (` ${normalizedSql} `.includes(forbidden)) fail(`SQL contains forbidden read/filter/boundary token: ${forbidden.trim()}`);
if ((normalizedSql.match(/statement_timestamp\(\)/g) || []).length !== 1 || !normalizedSql.includes("extract(epoch from observation.observed_at)") || !normalizedSql.includes("* 1000000")) fail("database observation microsecond expression drifted");
if ((normalizedSql.match(/\$[0-9]+/g) || []).join(",") !== "$1" || !normalizedSql.includes("lower($1::text)")) fail("SQL wallet bind inventory drifted");
if ((normalizedSql.match(/left join public\./g) || []).length !== 4) fail("SQL must retain four raw LEFT JOINs");
const actualQualifiedTables = [...normalizedSql.matchAll(/(?:from|join)\s+(public\.[a-z_]+)/g)].map((match) => match[1]);
if (JSON.stringify(actualQualifiedTables) !== JSON.stringify(expectedQualifiedTables)) fail("SQL public-qualified table inventory drifted");
for (const forbiddenPolicy of ["assignment.is_active =", "plan.is_active =", "permission.is_active =", "expires_at >", "ranking_offset", "epsx:rankings:offset", "grace_period"]) if (normalizedSql.includes(forbiddenPolicy)) fail(`SQL must preserve raw policy facts: ${forbiddenPolicy}`);
const selectMatch = sql.match(/\)\s*SELECT\s+([\s\S]*?)\nFROM\s+observation\n/i);
if (!selectMatch) fail("main SQL projection is missing");
const actualColumns = selectMatch[1].split(",").map((expression) => {
  const compact = expression.trim();
  const alias = compact.match(/\s+AS\s+([a-z_]+)$/i);
  if (alias) return alias[1].toLowerCase();
  const terminal = compact.match(/\.([a-z_]+)$/i);
  return terminal?.[1]?.toLowerCase();
});
if (JSON.stringify(actualColumns) !== JSON.stringify(expectedSelectedColumns)) fail("SQL static selected-column inventory drifted");
if (!normalizedSql.includes("from observation left join public.wallet_plan_assignments") || !normalizedSql.includes("order by") || normalizedSql.includes(" inner join ")) fail("sentinel/raw LEFT JOIN or deterministic ordering structure drifted");

const coreMod = contentByFile.get("apps/backend/src/infrastructure/adapters/repositories/mod.rs");
const sharedContract = contentByFile.get("shared/rust/epsx-contracts/src/ranking_entitlement_snapshot.rs");
const sharedLib = contentByFile.get("shared/rust/epsx-contracts/src/lib.rs");
const identityResolver = contentByFile.get("shared/rust/epsx-identity-service/src/ranking_entitlement.rs");
const runtimeMain = contentByFile.get("shared/rust/epsx-identity-service/src/main.rs");
const runtimeService = contentByFile.get("shared/rust/epsx-identity-service/src/identity_service.rs");
contains(adapter, "diesel::sql_query(RANKING_ENTITLEMENT_SNAPSHOT_SQL)", "one Diesel statement execution site");
contains(adapter, ".bind::<diesel::sql_types::Text, _>", "single wallet bind");
if ((adapter.match(/diesel::sql_query\(/g) || []).length !== 1 || (adapter.match(/\.bind::<diesel::sql_types::Text, _>/g) || []).length !== 1) fail("adapter must retain exactly one Diesel statement site and one wallet bind");
contains(adapter, "snapshot_from_rows", "pure row decoder");
contains(adapter, "is_clean_sentinel", "strict sentinel decoder");
contains(coreMod, "pub mod ranking_entitlement_snapshot_repository;", "core repository module ownership");
contains(coreMod, "PostgresRankingEntitlementSnapshotRepository", "core adapter export");
for (const anchor of ["pub struct RankingEntitlementSnapshot", "pub enum RankingEntitlementSnapshotError", "pub trait RankingEntitlementSnapshotRepository"]) contains(sharedContract, anchor, "shared snapshot contract");
for (const forbidden of ["diesel", "sql_query", "statement_timestamp", "RankingOffset::new", "FreePlan"]) excludes(sharedContract, forbidden, "storage or ranking policy in shared contract");
contains(sharedLib, "pub mod ranking_entitlement_snapshot;", "shared contract export");
contains(identityResolver, "pub use epsx_contracts::ranking_entitlement_snapshot", "identity resolver shared-contract consumer");
contains(identityResolver, "RankingOffset::new", "identity-owned ranking policy");
const expectedRuntimeMainDigest = staticOnly ? sha256(runtimeMain) : "9a7c4185032803f6453dd4a2ab1afbc3bde06219209d0398571cb73721ac183d";
const expectedRuntimeServiceDigest = staticOnly ? sha256(runtimeService) : "a5d64d6aa314a2f2c504836595baacc77437c0668f5e42663ee0299f43950895";
if (sha256(runtimeMain) !== expectedRuntimeMainDigest || sha256(runtimeService) !== expectedRuntimeServiceDigest) fail("identity runtime safety boundary drifted");
if (staticOnly) contains(runtimeMain, "Arc::new(FreePlanRankingOffsetService)", "historical runtime wiring");
else contains(runtimeMain, "Arc::new(UnavailableRankingOffsetService)", "fail-closed runtime wiring");
if (staticOnly) contains(runtimeService, "Ok(RankingOffset::free_plan())", "historical runtime service");
else contains(runtimeService, "ranking authority is unavailable", "fail-closed runtime service");
for (const name of expectedTests) if (!adapter.includes(name)) fail(`missing hermetic test source: ${name}`);
for (const item of contract.implementationEvidence) if (item.file.includes("/migrations/") || item.file.endsWith("schema.rs") || item.file.endsWith("diesel.toml")) fail("A2.8 implementation evidence must not include a migration or schema regeneration");

process.stdout.write(JSON.stringify({
  artifact: contract.artifact,
  productionReady: contract.productionReady,
  readinessExit: contract.readinessExit,
  invariants: contract.invariants.length,
  fixtures: fixture.fixtureCount,
  hermeticTests: contract.hermeticTests.length,
  implementationEvidence: contract.implementationEvidence.length,
  sqlDigest: contract.sqlEvidence.sha256,
  residualStops: contract.residualStops,
}));
' "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT")" || exit 1

if [[ "$MODE" == "integrity" && "$STATIC_ONLY" != "1" ]]; then
  test_list="$(cargo test --offline --locked -p epsx-ranking-store --lib -- --list 2>&1)" || {
    printf '%s\n' "$test_list" >&2
    die "could not enumerate backend hermetic tests"
  }
  while IFS= read -r test_name; do
    [[ -n "$test_name" ]] || continue
    match="$(printf '%s\n' "$test_list" | sed -n "/::${test_name}: test$/s/: test$//p")"
    match_count="$(printf '%s\n' "$match" | sed '/^$/d' | wc -l | tr -d ' ')"
    [[ "$match_count" == "1" ]] || die "hermetic test name is missing or ambiguous: $test_name"
    output="$(cargo test --offline --locked -p epsx-ranking-store --lib "$match" -- --exact 2>&1)" || {
      printf '%s\n' "$output" >&2
      die "hermetic test failed: $test_name"
    }
    grep -q "test result: ok. 1 passed; 0 failed" <<<"$output" || die "hermetic test did not run exactly once: $test_name"
  done < <(bun -e 'const c = await Bun.file(process.argv[1]).json(); for (const name of c.hermeticTests) console.log(name);' "$CONTRACT")

  check_output="$(cargo check --offline --locked -p epsx-ranking-store --lib 2>&1)" || {
    printf '%s\n' "$check_output" >&2
    die "epsx offline library check failed"
  }
fi

invariants="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.invariants.length));' "$CONTRACT")"
fixtures="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.fixtureEvidence.fixtureCount));' "$CONTRACT")"
tests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.hermeticTests.length));' "$CONTRACT")"
digests="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.implementationEvidence.length + 2));' "$CONTRACT")"
stops="$(bun -e 'const c=await Bun.file(process.argv[1]).json(); process.stdout.write(String(c.residualStops.length));' "$CONTRACT")"

case "$MODE" in
  integrity)
    printf 'core-ranking-snapshot-adapter: PASS; %s invariants; %s fixtures; %s hermetic tests; %s frozen digests; %s residual STOPs\n' "$invariants" "$fixtures" "$tests" "$digests" "$stops"
    ;;
  report)
    printf '%s\n' "$summary"
    ;;
  readiness)
    printf 'core-ranking-snapshot-adapter: LIMIT; %s residual STOPs remain; readiness exit 3\n' "$stops" >&2
    exit 3
    ;;
esac
