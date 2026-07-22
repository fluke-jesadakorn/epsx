#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-12-indexer-schema-boundary.json"
mode=integrity

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      [ "$#" -ge 2 ] || { echo "a3-12-indexer-schema-boundary: ERROR: --mode requires a value" >&2; exit 64; }
      mode=$2
      shift 2
      ;;
    --contract)
      [ "$#" -ge 2 ] || { echo "a3-12-indexer-schema-boundary: ERROR: --contract requires a value" >&2; exit 64; }
      contract=$2
      shift 2
      ;;
    *)
      echo "usage: $0 [--mode integrity|report|readiness] [--contract PATH]" >&2
      exit 64
      ;;
  esac
done

case "$mode" in integrity|report|readiness) ;; *) echo "a3-12-indexer-schema-boundary: ERROR: invalid mode" >&2; exit 64 ;; esac

command -v bun >/dev/null 2>&1 || { echo "a3-12-indexer-schema-boundary: ERROR: bun is required" >&2; exit 1; }

A3_ROOT="$repo_root" A3_CONTRACT="$contract" A3_MODE="$mode" bun -e '
import { existsSync, lstatSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";

const root = realpathSync(process.env.A3_ROOT);
const contractInput = resolve(process.env.A3_CONTRACT);
const mode = process.env.A3_MODE;
const fail = (message) => { console.error(`a3-12-indexer-schema-boundary: ERROR: ${message}`); process.exit(1); };
const exact = (label, expected, actual) => { if (JSON.stringify(expected) !== JSON.stringify(actual)) fail(`${label} drifted`); };
const sha256 = (value) => { const h = new Bun.CryptoHasher("sha256"); h.update(value); return h.digest("hex"); };
const git = (...args) => {
  const result = Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed: ${result.stderr.toString().trim()}`);
  return result.stdout.toString();
};
const gitExists = (...args) => Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" }).exitCode === 0;
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || isAbsolute(value) || value.split("/").includes("..")) fail(`${label} path is unsafe`);
  const path = resolve(root, value);
  if (path !== root && !path.startsWith(`${root}/`)) fail(`${label} escapes repository`);
  return path;
};
const regularRepoFile = (value, label) => {
  const path = safeRelative(value, label);
  if (!existsSync(path) || lstatSync(path).isSymbolicLink() || !statSync(path).isFile()) fail(`${label} is missing or unsafe`);
  return path;
};

let fixture;
try {
  if (lstatSync(contractInput).isSymbolicLink()) fail("contract must not be a symbolic link");
  fixture = JSON.parse(readFileSync(realpathSync(contractInput), "utf8"));
} catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (fixture.schemaVersion !== 1 || fixture.contractId !== "A3.12-indexer-schema-boundary") fail("unexpected schemaVersion or contractId");
if (fixture.purpose !== "offline-static-indexer-schema-boundary-only") fail("unexpected contract purpose");
if (fixture.productionReady !== false || fixture.integrityExit !== 0 || fixture.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", { service: "services/indexer", database: "epsx_indexer", schema: "public", tables: ["blocks", "transactions", "token_transfers"], status: "partial" }, fixture.scope);
const rawFixture = readFileSync(contractInput, "utf8");
if (/postgres(?:ql)?:\/\/[^\s"/]+:[^\s"@]+@/i.test(rawFixture)) fail("contract must not contain database credentials");
for (const [key, value] of Object.entries(fixture.safety ?? {})) {
  if (key !== "readinessMeaning" && value !== false) fail(`${key} must remain false`);
}

const provenance = fixture.provenance;
if (provenance.sourceRef !== "origin/development" || provenance.sourceRefRole !== "provenance-label-only" || provenance.sourceCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || provenance.scopeDecision !== "development-has-no-standalone-indexer-contract") fail("development provenance drifted");
if (!gitExists("cat-file", "-e", `${provenance.sourceCommit}^{commit}`)) fail("pinned development commit is unavailable");
exact("absent development paths", ["services/indexer", "migrated/services/indexer"], provenance.absentPaths);
for (const path of provenance.absentPaths) if (gitExists("cat-file", "-e", `${provenance.sourceCommit}:${path}`)) fail(`unexpected development indexer path exists: ${path}`);
const snapshot = provenance.removedRuntimeSnapshot;
if (snapshot.commit !== "b624f320c2db3dc24944cc0414deae7bc2d42196" || snapshot.file !== "services/indexer/src/main.rs" || snapshot.blob !== "3bb4779628eb888be9cc0a832bcf249828b2b345") fail("removed runtime snapshot pin drifted");
if (git("rev-parse", `${snapshot.commit}:${snapshot.file}`).trim() !== snapshot.blob) fail("removed runtime snapshot blob changed");
const oldRuntime = git("show", `${snapshot.commit}:${snapshot.file}`);
if (!Array.isArray(snapshot.anchors) || snapshot.anchors.length !== 5) fail("exactly five old runtime DDL anchors are required");
for (const anchor of snapshot.anchors) if (!oldRuntime.includes(anchor)) fail(`old runtime anchor is missing: ${anchor}`);

const runtime = fixture.runtimeBoundary;
if (runtime.rustRoot !== "services/indexer" || runtime.scannerFindingBefore !== 5 || runtime.scannerFindingAfter !== 0) fail("runtime finding boundary drifted");
exact("Rust inventory", [
  "services/indexer/src/ingestion/domain.rs",
  "services/indexer/src/ingestion/memory.rs",
  "services/indexer/src/ingestion/mod.rs",
  "services/indexer/src/ingestion/ports.rs",
  "services/indexer/src/ingestion/postgres/candidates.rs",
  "services/indexer/src/ingestion/postgres/codec.rs",
  "services/indexer/src/ingestion/postgres/leases.rs",
  "services/indexer/src/ingestion/postgres/mod.rs",
  "services/indexer/src/ingestion/postgres/reads.rs",
  "services/indexer/src/ingestion/selection.rs",
  "services/indexer/src/lib.rs",
  "services/indexer/src/main.rs"
], runtime.rustInventory);
exact("startup anchors", ["sqlx::PgPool::connect(&args.database_url)", "verify_schema_compatibility(&db)", "let state = AppState { db };", "tokio::net::TcpListener::bind(addr)"], runtime.startupOrderAnchors);
exact("forbidden runtime anchors", ["sync_on_start", "poll_interval", "provider_for_chain", "tokio::spawn", "sync_chain", "index_block", "ON CONFLICT (chain_id, number) DO NOTHING", "format!(\"0x{:064x}\", number)", "fetch_block_number"], runtime.forbiddenRuntimeAnchors);
exact("model/bind anchors", ["timestamp: chrono::DateTime<chrono::Utc>", "from_address: String", "to_address: Option<String>", "miner: Option<String>", "let indexer_block = indexed", ".map(u64::try_from)", "fn canonical_chain_id(value: &str)", "B256::from_str(value)", "Address::from_str(value)", "ORDER BY block_number DESC, tx_hash DESC, log_index DESC"], runtime.modelAndBindAnchors);
const rustRoot = safeRelative(runtime.rustRoot, "Rust root");
const rustFiles = [];
const visit = (directory) => {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a,b) => a.name.localeCompare(b.name))) {
    const path = resolve(directory, entry.name);
    if (entry.isSymbolicLink()) fail("symbolic links are forbidden under indexer Rust root");
    if (entry.isDirectory()) visit(path);
    else if (entry.isFile() && entry.name.endsWith(".rs")) rustFiles.push(path);
  }
};
visit(rustRoot);
rustFiles.sort();
exact("discovered Rust inventory", runtime.rustInventory, rustFiles.map((path) => path.slice(root.length + 1)));
const stripRustComments = (content) => {
  let block = false;
  return content.split(/\r?\n/).map((line) => {
    let out = "";
    for (let i = 0; i < line.length; i += 1) {
      if (block) { if (line.slice(i, i + 2) === "*/") { block = false; i += 1; } continue; }
      if (line.slice(i, i + 2) === "/*") { block = true; i += 1; continue; }
      if (line.slice(i, i + 2) === "//") break;
      out += line[i];
    }
    return out;
  });
};
const ddlPattern = /\b(?:CREATE|ALTER|DROP|TRUNCATE)\s+(?:OR\s+REPLACE\s+)?(?:TABLE|SCHEMA|INDEX|TYPE|VIEW|MATERIALIZED\s+VIEW|DATABASE)\b/i;
const ddlFindings = [];
for (const file of rustFiles) stripRustComments(readFileSync(file, "utf8")).forEach((line, index) => { if (ddlPattern.test(line)) ddlFindings.push({ file: file.slice(root.length + 1), line: index + 1 }); });
if (ddlFindings.length !== 0) fail(`indexer runtime Rust DDL scanner found ${ddlFindings.length}, expected zero`);

const lib = readFileSync(regularRepoFile("services/indexer/src/lib.rs", "indexer library"), "utf8");
const main = readFileSync(regularRepoFile("services/indexer/src/main.rs", "indexer main"), "utf8");
for (const anchor of snapshot.anchors) if (lib.includes(anchor) || main.includes(anchor)) fail(`removed runtime DDL anchor returned: ${anchor}`);

const adapter = fixture.dormantAdapterBoundary;
const adapterSourcePins = [
  { path: "services/indexer/Cargo.toml", bytes: 771, sha256: "9cd598ce3adeac3fde3ec021704ee5213b93622d6d6ff8e836e0b0c2b165a135" },
  { path: "services/indexer/src/ingestion/mod.rs", bytes: 1170, sha256: "395e589d5eb05c5d8577d9a15bf1c131f3d1c114ff3eb3289985b97424d6d547" },
  { path: "services/indexer/src/ingestion/postgres/candidates.rs", bytes: 19643, sha256: "9bccc08effb68e06593469f93d779cc2a12bad088b6698b3eded8d2de4128180" },
  { path: "services/indexer/src/ingestion/postgres/codec.rs", bytes: 5891, sha256: "693e1ddba5a8f8808251ed8be68f547b5a8da1122eec954a741ca8c0c95f9915" },
  { path: "services/indexer/src/ingestion/postgres/leases.rs", bytes: 11531, sha256: "20adcdc84b1fd970ed404d2ac9219b3de827ca01a84ece33813cfaf6ba690910" },
  { path: "services/indexer/src/ingestion/postgres/mod.rs", bytes: 568, sha256: "521534817aafdb618ebe3528cebceb3206be4e5b145c0ef2ae933794ee026d10" },
  { path: "services/indexer/src/ingestion/postgres/reads.rs", bytes: 20505, sha256: "b87bf78f4773b8f63d619a058d262462200bcb606c2c2c909337fe1a52809cce" }
];
exact("dormant adapter source pins", adapterSourcePins, adapter?.sourcePins);
const pinnedAdapterSources = new Map();
for (const pin of adapterSourcePins) {
  const source = readFileSync(regularRepoFile(pin.path, "dormant adapter pinned source"));
  if (source.length !== pin.bytes || sha256(source) !== pin.sha256) fail(`dormant adapter source bytes changed: ${pin.path}`);
  pinnedAdapterSources.set(pin.path, source.toString("utf8"));
}
exact("dormant adapter boundary", {
  status: "compiled-static-substrate",
  feature: "dormant-postgres-adapter",
  sourcePins: adapterSourcePins,
  defaultEnabled: false,
  modulePrivate: true,
  publicExport: false,
  mainCallsite: false,
  repositoryVisibility: "pub(super)",
  poolHolderOnly: true,
  parentConflictTargetOnly: true,
  strictChildInserts: true,
  fullCandidateReload: true,
  reloadRevalidation: true,
  decimalCodec: true,
  timestampCodec: true,
  outcomeCodec: true,
  databaseClockLeasePredicates: true,
  persistentLeaseFence: true,
  readSideHelpers: true,
  readModulePrivate: true,
  repeatableReadOnlyTransactions: true,
  candidateReadSingleSnapshot: true,
  snapshotReadSingleSnapshot: true,
  absentStateRejectsSelectedOrJournalOrphans: true,
  snapshotMappingChecks: true,
  snapshotRevisionChecks: true,
  selectedHashLeftJoinsChainState: true,
  selectedHashRejectsMissingStaleOrFutureState: true,
  candidateReadsLegacyProjection: false,
  usesUtcNow: false,
  usesAdvisoryLocks: false,
  providerActivated: false,
  workerActivated: false,
  routeActivated: false,
  migrationExecuted: false,
  databaseRead: false,
  databaseWrite: false,
  runtimeAdapter: false,
  executed: false,
  testEvidence: { defaultLibraryPassed: 33, featureLibraryPassed: 50, binaryPassed: 4 }
}, adapter);
const cargo = pinnedAdapterSources.get("services/indexer/Cargo.toml");
const ingestion = pinnedAdapterSources.get("services/indexer/src/ingestion/mod.rs");
const postgresModule = pinnedAdapterSources.get("services/indexer/src/ingestion/postgres/mod.rs");
const candidates = pinnedAdapterSources.get("services/indexer/src/ingestion/postgres/candidates.rs");
const codec = pinnedAdapterSources.get("services/indexer/src/ingestion/postgres/codec.rs");
const leases = pinnedAdapterSources.get("services/indexer/src/ingestion/postgres/leases.rs");
const reads = pinnedAdapterSources.get("services/indexer/src/ingestion/postgres/reads.rs");
if (!cargo.includes("[features]\ndefault = []\ndormant-postgres-adapter = []")) fail("dormant adapter feature/default boundary drifted");
if ((ingestion.match(/#\[cfg\(feature = \"dormant-postgres-adapter\"\)\]\nmod postgres;/g) ?? []).length !== 1) fail("private dormant adapter module declaration drifted");
if (/pub(?:\([^)]*\))?\s+mod\s+postgres\b/.test(ingestion) || /pub\s+use\s+(?:self::)?postgres\b/.test(ingestion)) fail("dormant adapter module became public");
if (main.includes("PostgresSelectedChainRepository") || main.includes("ingestion::postgres") || lib.includes("PostgresSelectedChainRepository") || lib.includes("ingestion::postgres")) fail("dormant adapter gained a main/library callsite or export");
for (const anchor of [
  "use sqlx::PgPool;", "mod candidates;", "mod codec;", "mod leases;", "mod reads;",
  "pub(super) struct PostgresSelectedChainRepository {", "pool: PgPool,",
  "pub(super) fn new(pool: PgPool) -> Self", "pub(super) fn pool(&self) -> &PgPool"
]) if (!postgresModule.includes(anchor)) fail(`dormant PgPool holder is missing: ${anchor}`);
if (/impl\s+SelectedChainRepository\s+for\s+PostgresSelectedChainRepository/.test(postgresModule) || /sqlx::query|\.begin\(\)|\.execute\(|\.fetch_/.test(postgresModule)) fail("dormant PgPool holder gained repository behavior");
if ((candidates.match(/\bON CONFLICT\b/g) ?? []).length !== 1 || !candidates.includes("ON CONFLICT (chain_id, block_hash) DO NOTHING")) fail("candidate conflict handling must target only the parent identity");
for (const anchor of [
  "let stored = load_candidate(transaction, identity)", "if stored == *candidate", "SelectionConflict::CandidateContent { identity }",
  "INSERT INTO public.indexer_transaction_inclusions", "INSERT INTO public.indexer_receipts", "INSERT INTO public.indexer_raw_logs",
  "FROM public.indexer_transaction_inclusions", "FROM public.indexer_receipts", "FROM public.indexer_raw_logs",
  "let candidate = validate_block(", "stored candidate failed validation", "loaded candidate hash does not match its requested identity"
]) if (!candidates.includes(anchor)) fail(`candidate persistence/reload boundary is missing: ${anchor}`);
for (const anchor of [
  "pub(super) fn decode_timestamp_seconds(", "value.timestamp_subsec_nanos() != 0", "u64::try_from(value.timestamp())",
  "pub(super) fn encode_u256_decimal(", "pub(super) fn decode_u256_decimal(", "U256::from_str_radix(value, 10)",
  "pub(super) fn encode_receipt_outcome(", "pub(super) fn decode_receipt_outcome("
]) if (!codec.includes(anchor)) fail(`adapter codec boundary is missing: ${anchor}`);
for (const anchor of [
  "LeaseFence::successor(stored.fence)?", "lease_expires_at = clock_timestamp()", "lease_expires_at > clock_timestamp()",
  "clock_timestamp() AS database_now", "FOR UPDATE", "SET lease_owner = NULL,", "RETURNING lease_fence"
]) if (!leases.includes(anchor)) fail(`database-clock fenced lease boundary is missing: ${anchor}`);
if (/Utc::now\s*\(/.test(leases)) fail("lease predicates must not use process time");
if (/advisory/i.test(stripRustComments(leases).join("\n"))) fail("lease substrate must not use advisory locks");
if ((reads.match(/begin_consistent_read\(/g) ?? []).length !== 4 || (reads.match(/\.commit\(\)/g) ?? []).length !== 4) fail("read helpers must use one consistent transaction each");
for (const anchor of [
  "pub(super) async fn snapshot(", "pub(super) async fn load_candidate(", "pub(super) async fn selected_hash(", "pub(super) async fn candidates_at_height(",
  "async fn begin_consistent_read<\x27pool>(", "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY", "candidates::load_candidate(&mut transaction, identity)",
  "verify_snapshot_mapping(&mut transaction, &snapshot)", "require_no_orphaned_selection_state(&mut transaction, chain_id)",
  "FROM public.indexer_selected_blocks", "FROM public.indexer_mutation_journal", "AS has_selected_blocks", "AS has_mutation_journal",
  "LEFT JOIN public.indexer_chain_state AS state", "state.revision AS chain_revision", "stored selected block has no chain-state row",
  "stored selected-block revision must be non-zero", "stored selected-block revision exceeds the chain revision",
  "stored selected head is not the highest exact selected-block mapping", "stored finalized selection does not match its selected-block mapping",
  "selected_revision > $2", "a selected-block mapping was written after the stored chain revision"
]) if (!reads.includes(anchor)) fail(`read-side consistency boundary is missing: ${anchor}`);
for (const legacy of ["public.blocks", "public.transactions", "public.token_transfers"]) if (reads.includes(legacy) || candidates.includes(legacy)) fail(`candidate/read helper returned to legacy projection: ${legacy}`);
const adapterCode = [postgresModule, candidates, codec, leases, reads].map((source) => stripRustComments(source).join("\n")).join("\n");
if (/impl\s+SelectedChainRepository\s+for\s+PostgresSelectedChainRepository/.test(adapterCode)) fail("dormant adapter gained a repository-port implementation");
for (const forbidden of ["tokio::spawn", "PgPool::connect", "sqlx::migrate!", "Migrator::new", "ProviderBuilder", "provider_for_chain", "Router::new", ".route("]) if (adapterCode.includes(forbidden)) fail(`dormant adapter activation returned: ${forbidden}`);
if (runtime.compatibilityQueryConstant !== "INDEXER_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility") fail("compatibility boundary names drifted");
if (runtime.structuralKeyArrayTextCastOccurrences !== 2) fail("structural key-array type contract drifted");
const queryMatch = lib.match(/pub const INDEXER_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!queryMatch) fail("compatibility query constant is missing");
const query = queryMatch[1];
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (!/^\s*WITH\s+expected_relations/i.test(query)) fail("compatibility query must begin with a read-only CTE");
if (/\b(?:INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i.test(query)) fail("compatibility query contains a mutation or command token");
const u256Max = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
for (const anchor of [
  "to_regclass(format(\x27public.%I\x27, e.table_name))", "count(*) = 27", "count(*) = 24", "count(*) = 10",
  "transactions_pkey", "ARRAY[\x27chain_id\x27,\x27hash\x27]::text[]", "foreign_key_boundary",
  "pg_catalog.pg_inherits", "NOT c.relrowsecurity AND NOT c.relforcerowsecurity", "ns.nspname <> \x27pg_catalog\x27",
  "keys.collation_oid <> a.attcollation", "a.indexprs IS NULL AND a.indpred IS NULL", "COALESCE(",
  "a.normalized_definition = e.expected_definition", "check(number>=0)",
  "check(statusisnullor(status=any(array[0,1])))", "USING btree (chain_id, \x22timestamp\x22 DESC, number DESC)",
  "SELECT att.attname::text", u256Max
]) if (!query.includes(anchor)) fail(`compatibility query is missing: ${anchor}`);
if ((query.match(/SELECT att\.attname::text/g) ?? []).length !== runtime.structuralKeyArrayTextCastOccurrences) fail("compatibility query must cast both structural key arrays from name to text");
if (query.includes("strpos(a.definition") || query.includes("required_fragment")) fail("compatibility query admits fragment-only check matching");
if ((query.match(new RegExp(u256Max, "g")) ?? []).length !== 2) fail("compatibility query must pin the U256 ceiling for both value checks");
if (!query.includes("(\x27transactions_pkey\x27, \x27transactions\x27, \x27p\x27, ARRAY[\x27chain_id\x27,\x27hash\x27]::text[]") || query.includes("(\x27transactions_pkey\x27, \x27transactions\x27, \x27p\x27, ARRAY[\x27hash\x27]::text[]")) fail("global transaction hash primary key was accepted");
const functionStart = lib.indexOf("pub async fn verify_schema_compatibility(");
const functionBody = lib.slice(functionStart, lib.indexOf("#[derive(Debug, Error)]", functionStart + 1) > functionStart ? lib.indexOf("#[derive(Debug, Error)]", functionStart + 1) : lib.length);
for (const anchor of ["sqlx::query_scalar::<_, bool>(INDEXER_SCHEMA_COMPATIBILITY_QUERY)", ".fetch_one(db)", "IndexerSchemaError::Incompatible"]) if (!functionBody.includes(anchor)) fail(`compatibility function is missing: ${anchor}`);
if (functionBody.includes(".execute(")) fail("compatibility function must remain read-only");

for (const anchor of runtime.modelAndBindAnchors) if (!main.includes(anchor)) fail(`model/bind anchor is missing: ${anchor}`);
for (const anchor of runtime.forbiddenRuntimeAnchors) if (main.includes(anchor)) fail(`unsafe runtime anchor returned: ${anchor}`);
const startupPositions = runtime.startupOrderAnchors.map((anchor) => main.indexOf(anchor));
if (startupPositions.some((position) => position < 0) || startupPositions.some((position, index) => index > 0 && position <= startupPositions[index - 1])) fail("startup compatibility ordering drifted");
for (const [qualified, expected] of Object.entries(runtime.qualifiedRelationOccurrences)) {
  const table = qualified.slice("public.".length);
  const qualifiedMatches = main.match(new RegExp(`\\b(?:INSERT\\s+INTO|UPDATE|DELETE\\s+FROM|FROM|JOIN)\\s+public\\.${table}\\b`, "gi")) ?? [];
  const unqualifiedMatches = main.match(new RegExp(`\\b(?:INSERT\\s+INTO|UPDATE|DELETE\\s+FROM|FROM|JOIN)\\s+${table}\\b`, "gi")) ?? [];
  if (qualifiedMatches.length !== expected) fail(`${qualified} runtime occurrence count is ${qualifiedMatches.length}, expected ${expected}`);
  if (unqualifiedMatches.length !== 0) fail(`unqualified runtime relation returned: ${table}`);
}

const migration = fixture.migrationRoot;
if (migration.path !== "services/indexer/migrations" || migration.runner !== null || migration.transactionOwner !== "future-reviewed-runner" || migration.orderedMigrations.length !== 2) fail("migration root boundary drifted");
exact("migration forbidden token policy", ["ALTER", "DROP", "TRUNCATE", "DELETE", "INSERT", "UPDATE", "CASCADE", "BEGIN TRANSACTION", "BEGIN WORK", "START TRANSACTION", "COMMIT", "ROLLBACK", "CREATE SCHEMA", "CREATE EXTENSION", "CREATE FUNCTION", "CREATE TRIGGER"], migration.forbiddenTokens);
const migrationFiles = readdirSync(safeRelative(migration.path, "migration root")).sort();
exact("migration inventory", [
  "20260722050000_create_indexer_projection_tables.sql",
  "20260722070000_create_indexer_fork_store.sql"
], migrationFiles);
const ordered = migration.orderedMigrations[0];
if (ordered.version !== "20260722050000" || ordered.path !== "services/indexer/migrations/20260722050000_create_indexer_projection_tables.sql" || ordered.bytes !== 4822 || ordered.sha256 !== "5d0ec77a11d2abe1303c5f9b87e7da18eadee9d2e7fa4aeda1aeaf3d76549ff8") fail("ordered migration pin drifted");
exact("migration guards", ["CREATE TABLE IF NOT EXISTS public.blocks (", "CREATE TABLE IF NOT EXISTS public.transactions (", "CREATE TABLE IF NOT EXISTS public.token_transfers (", "CREATE INDEX IF NOT EXISTS idx_blocks_timestamp", "CREATE INDEX IF NOT EXISTS idx_transactions_block", "CREATE INDEX IF NOT EXISTS idx_transfers_token", "CREATE INDEX IF NOT EXISTS idx_transfers_from", "CREATE INDEX IF NOT EXISTS idx_transfers_to"], ordered.guards);
const sql = readFileSync(regularRepoFile(ordered.path, "ordered migration"), "utf8");
if (Buffer.byteLength(sql) !== ordered.bytes || sha256(sql) !== ordered.sha256) fail("indexer migration bytes changed");
for (const guard of ordered.guards) if (!sql.includes(guard)) fail(`migration guard is missing: ${guard}`);
if ((sql.match(/CREATE TABLE IF NOT EXISTS public\./g) ?? []).length !== 3 || (sql.match(/CREATE INDEX IF NOT EXISTS /g) ?? []).length !== 5) fail("migration must contain exactly three guarded tables and five guarded indexes");
if ((sql.match(new RegExp(u256Max, "g")) ?? []).length !== 2 || (sql.match(/value::NUMERIC <= NUMERIC/g) ?? []).length !== 2) fail("migration must enforce the exact U256 ceiling for both value columns");
const forkMigration = migration.orderedMigrations[1];
if (forkMigration.version !== "20260722070000" || forkMigration.path !== "services/indexer/migrations/20260722070000_create_indexer_fork_store.sql" || forkMigration.bytes !== 23326 || forkMigration.sha256 !== "60b82188c74c5de7463610ce4c5795150970a4b760d5a81c66981cd25d9e5f00") fail("fork-store migration pin drifted");
exact("fork-store migration guards", [
  "CREATE TABLE IF NOT EXISTS public.indexer_block_candidates (",
  "CREATE TABLE IF NOT EXISTS public.indexer_transaction_inclusions (",
  "CREATE TABLE IF NOT EXISTS public.indexer_receipts (",
  "CREATE TABLE IF NOT EXISTS public.indexer_raw_logs (",
  "CREATE TABLE IF NOT EXISTS public.indexer_selected_blocks (",
  "CREATE TABLE IF NOT EXISTS public.indexer_chain_state (",
  "CREATE TABLE IF NOT EXISTS public.indexer_mutation_journal (",
  "CREATE TABLE IF NOT EXISTS public.indexer_mutation_blocks (",
  "CREATE INDEX IF NOT EXISTS idx_indexer_block_candidates_parent",
  "CREATE INDEX IF NOT EXISTS idx_indexer_transaction_inclusions_hash"
], forkMigration.guards);
const forkSql = readFileSync(regularRepoFile(forkMigration.path, "fork-store migration"), "utf8");
if (Buffer.byteLength(forkSql) !== forkMigration.bytes || sha256(forkSql) !== forkMigration.sha256) fail("fork-store migration bytes changed");
for (const guard of forkMigration.guards) if (!forkSql.includes(guard)) fail(`fork-store migration guard is missing: ${guard}`);
if ((forkSql.match(/CREATE TABLE IF NOT EXISTS public\./g) ?? []).length !== 8 || (forkSql.match(/CREATE INDEX IF NOT EXISTS /g) ?? []).length !== 2) fail("fork-store migration must contain exactly eight guarded tables and two guarded indexes");
if ((forkSql.match(/^    CONSTRAINT /gm) ?? []).length !== 101 || (forkSql.match(/ CHECK \(/g) ?? []).length !== 73) fail("fork-store constraint inventory drifted");
const firstForkCreate = forkSql.indexOf("CREATE TABLE IF NOT EXISTS public.indexer_block_candidates");
if (firstForkCreate <= 0) fail("fork-store collision preflight must precede every CREATE");
const forkPreflight = forkSql.slice(0, firstForkCreate);
if (!forkPreflight.trimStart().startsWith("DO $indexer_fork_store_preflight$") || (forkSql.match(/DO \$indexer_fork_store_preflight\$/g) ?? []).length !== 1 || (forkPreflight.match(/\bBEGIN\b/g) ?? []).length !== 1) fail("fork-store collision preflight boundary drifted");
if (/\brelkind\b/i.test(forkPreflight)) fail("fork-store collision preflight must reject every relation kind");
for (const anchor of [
  "FROM pg_catalog.pg_class rel",
  "JOIN pg_catalog.pg_namespace ns ON ns.oid = rel.relnamespace",
  "WHERE ns.nspname = \x27public\x27",
  "rel.relname = ANY (ARRAY[",
  "IF collision_names IS NOT NULL THEN",
  "RAISE EXCEPTION USING",
  "ERRCODE = \x2742P07\x27",
  "indexer fork-store fresh-create collision in public:",
  "all eight fork-store table names and both explicit index names are reserved regardless of relation kind",
  "refusing baseline adoption"
]) if (!forkPreflight.includes(anchor)) fail(`fork-store collision preflight is missing: ${anchor}`);
const preflightArray = forkPreflight.match(/rel\.relname = ANY \(ARRAY\[([\s\S]*?)\]::TEXT\[\]\)/);
if (!preflightArray) fail("fork-store collision preflight name array is missing");
const preflightNames = [...preflightArray[1].matchAll(/\x27([^\x27]+)\x27/g)].map((match) => match[1]);
exact("fork-store collision names", [
  "indexer_block_candidates",
  "indexer_transaction_inclusions",
  "indexer_receipts",
  "indexer_raw_logs",
  "indexer_selected_blocks",
  "indexer_chain_state",
  "indexer_mutation_journal",
  "indexer_mutation_blocks",
  "idx_indexer_block_candidates_parent",
  "idx_indexer_transaction_inclusions_hash"
], preflightNames);
for (const anchor of [
  "CONSTRAINT indexer_block_candidates_pkey PRIMARY KEY (chain_id, block_hash)",
  "CONSTRAINT indexer_block_candidates_chain_number_hash_key UNIQUE (chain_id, number, block_hash)",
  "CONSTRAINT indexer_transaction_inclusions_pkey PRIMARY KEY (chain_id, block_hash, transaction_index)",
  "CONSTRAINT indexer_transaction_inclusions_chain_block_tx_hash_key UNIQUE (chain_id, block_hash, transaction_hash)",
  "CONSTRAINT indexer_selected_blocks_candidate_fkey FOREIGN KEY (chain_id, number, block_hash)",
  "REFERENCES public.indexer_block_candidates (chain_id, number, block_hash)",
  "CONSTRAINT indexer_chain_state_lease_pair_check CHECK (",
  "CONSTRAINT indexer_chain_state_live_lease_fence_check CHECK (",
  "ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED",
  "AND post_state_root IS NOT NULL",
  "IS NOT DISTINCT FROM expected_finalized_selection_number",
  "IS NOT DISTINCT FROM finality_target_number",
  "result_revision = expected_revision + 1",
  "lease_fence BETWEEN 1 AND 9223372036854775807",
  "value <= NUMERIC \x27115792089237316195423570985008687907853269984665640564039457584007913129639935\x27"
]) if (!forkSql.includes(anchor)) fail(`fork-store invariant is missing: ${anchor}`);
for (const forbidden of [
  "PRIMARY KEY (chain_id, number, block_hash)",
  "UNIQUE (chain_id, transaction_hash)",
  "PRIMARY KEY (chain_id, transaction_hash)",
  "REFERENCES public.blocks",
  "REFERENCES public.transactions",
  "REFERENCES public.token_transfers",
  "octet_length(input_data)",
  "octet_length(data)",
  "mutation_fingerprint",
  "payload_fingerprint"
]) if (forkSql.includes(forbidden)) fail(`fork-store forbidden shape returned: ${forbidden}`);
const factPrefix = forkSql.slice(0, forkSql.indexOf("CREATE TABLE IF NOT EXISTS public.indexer_selected_blocks"));
if (/\b(?:canonical|finalized)\b/i.test(factPrefix)) fail("fork facts must not contain canonical or finalized flags");
for (const pattern of [
  /\bALTER\b/i, /\bDROP\b/i, /\bTRUNCATE\b/i, /\bDELETE\s+FROM\b/i, /(?:^|;)\s*INSERT\s+INTO\b/im,
  /(?:^|;)\s*UPDATE\s+\w/im, /\bCASCADE\b/i, /\bCOMMIT\b/i, /\bROLLBACK\b/i, /\bSTART\s+TRANSACTION\b/i, /\bCREATE\s+SCHEMA\b/i,
  /\bCREATE\s+EXTENSION\b/i, /\bCREATE\s+FUNCTION\b/i, /\bCREATE\s+TRIGGER\b/i
]) for (const migrationSql of [sql, forkSql]) if (pattern.test(migrationSql)) fail(`migration contains forbidden statement: ${pattern}`);
if (/\bBEGIN\b/i.test(sql) || /\bBEGIN\b/i.test(forkSql.slice(firstForkCreate))) fail("migration contains transaction control outside the exact fork-store collision preflight");

const forkStore = fixture.forkStoreContract;
if (sha256(JSON.stringify(forkStore)) !== "10695df6816d08514fe92ccbbe0dbf6dcf7eee089dcb5577634d00e6bb1f368b") fail("fork-store contract descriptors drifted");
if (forkStore.status !== "dormant-static-substrate" || forkStore.columns !== 74 || Object.keys(forkStore.tables).length !== 8 || Object.values(forkStore.tables).reduce((total, columns) => total + columns.length, 0) !== 74) fail("fork-store table/column descriptors drifted");
if (forkStore.structuralConstraints.length !== 28 || forkStore.checkConstraints !== 73 || forkStore.explicitIndexes.length !== 2) fail("fork-store constraint/index descriptors drifted");
exact("fork-store collision contract names", preflightNames, forkStore.collisionPreflightNames);
if (forkStore.freshCreateCollisionPreflight !== true || forkStore.preflightBeforeCreates !== true || forkStore.preflightRelkindRestricted !== false || forkStore.ifNotExistsAloneSafe !== false || forkStore.baselineAdoption !== false || forkStore.futureRunnerRecordsVersionAfterPreflight !== true || forkStore.proceduralBeginOnlyInCollisionPreflight !== true || forkStore.topLevelTransactionControl !== false || forkStore.candidateHeightPrimaryKey !== false || forkStore.globalTransactionHashUnique !== false || forkStore.selectedCandidateTripleForeignKey !== true || forkStore.pairedLeaseOwnerExpiry !== true || forkStore.liveLeaseRequiresPositiveFence !== true || forkStore.postStateRootOutcomeExclusive !== true || forkStore.nullSafeMutationKindChecks !== true || forkStore.sameTransactionHashAcrossBlockCandidatesAllowed !== true || forkStore.factCanonicalOrFinalizedFlags !== false || forkStore.fixedPayloadCaps !== false || forkStore.mutationFingerprint !== false || forkStore.runtimeCompatibilityProbeIncludesForkStore !== false) fail("fork-store static policy drifted");

const schema = fixture.schemaContract;
if (schema.columns !== 27) fail("schema inventory drifted");
exact("schema contract fields", [
  "checkConstraints", "columns", "extraColumnsConstraintsOrIndexesAccepted", "globalTransactionHashPrimaryKeyAccepted",
  "indexCollationDriftAccepted", "indexes", "inheritanceAccepted", "nonCatalogOperatorClassesAccepted",
  "partialExpressionOrIncludedIndexesAccepted", "rowLevelSecurityAccepted", "structuralConstraints", "tables"
], Object.keys(schema).sort());
exact("schema column descriptors", {
  blocks: [
    "chain_id:varchar(10):required", "number:int8:required", "hash:varchar(66):required",
    "parent_hash:varchar(66):required", "timestamp:timestamptz(6):required", "miner:varchar(42):nullable",
    "gas_used:int8:required", "gas_limit:int8:required", "tx_count:int4:required:default-zero"
  ],
  transactions: [
    "chain_id:varchar(10):required", "hash:varchar(66):required", "from_address:varchar(42):required",
    "to_address:varchar(42):nullable", "value:varchar(78):required", "block_number:int8:required",
    "status:int4:nullable", "timestamp:timestamptz(6):required", "input_data:bytea:required"
  ],
  token_transfers: [
    "chain_id:varchar(10):required", "tx_hash:varchar(66):required", "log_index:int4:required",
    "token_address:varchar(42):required", "from_address:varchar(42):required", "to_address:varchar(42):required",
    "value:varchar(78):required", "block_number:int8:required", "timestamp:timestamptz(6):required"
  ]
}, schema.tables);
exact("schema structural constraint descriptors", [
  "blocks_pkey:pk(chain_id,number)",
  "blocks_chain_hash_key:uq(chain_id,hash)",
  "transactions_pkey:pk(chain_id,hash)",
  "transactions_chain_hash_block_key:uq(chain_id,hash,block_number)",
  "transactions_block_fkey:fk(chain_id,block_number)->blocks(chain_id,number):no-action",
  "token_transfers_pkey:pk(chain_id,tx_hash,log_index)",
  "token_transfers_transaction_fkey:fk(chain_id,tx_hash,block_number)->transactions(chain_id,hash,block_number):no-action"
], schema.structuralConstraints);
exact("schema check constraint descriptors", [
  "blocks_chain_id_check:blocks:check(chain_id~\x27^[1-9][0-9]{0,9}$\x27)",
  "blocks_number_check:blocks:check(number>=0)",
  "blocks_hash_check:blocks:check(hash~\x27^0x[0-9a-f]{64}$\x27)",
  "blocks_parent_hash_check:blocks:check(parent_hash~\x27^0x[0-9a-f]{64}$\x27)",
  "blocks_miner_check:blocks:check(minerisnullorminer~\x27^0x[0-9a-f]{40}$\x27)",
  "blocks_gas_used_check:blocks:check(gas_used>=0)",
  "blocks_gas_limit_check:blocks:check(gas_limit>=0)",
  "blocks_gas_bounds_check:blocks:check(gas_used<=gas_limit)",
  "blocks_tx_count_check:blocks:check(tx_count>=0)",
  "transactions_chain_id_check:transactions:check(chain_id~\x27^[1-9][0-9]{0,9}$\x27)",
  "transactions_hash_check:transactions:check(hash~\x27^0x[0-9a-f]{64}$\x27)",
  "transactions_from_address_check:transactions:check(from_address~\x27^0x[0-9a-f]{40}$\x27)",
  "transactions_to_address_check:transactions:check(to_addressisnullorto_address~\x27^0x[0-9a-f]{40}$\x27)",
  `transactions_value_check:transactions:check(casewhenvalue~\x27^(0|[1-9][0-9]{0,77})$\x27thenvalue<=\x27${u256Max}\x27elsefalseend)`,
  "transactions_block_number_check:transactions:check(block_number>=0)",
  "transactions_status_check:transactions:check(statusisnullor(status=any(array[0,1])))",
  "token_transfers_chain_id_check:token_transfers:check(chain_id~\x27^[1-9][0-9]{0,9}$\x27)",
  "token_transfers_tx_hash_check:token_transfers:check(tx_hash~\x27^0x[0-9a-f]{64}$\x27)",
  "token_transfers_log_index_check:token_transfers:check(log_index>=0)",
  "token_transfers_token_address_check:token_transfers:check(token_address~\x27^0x[0-9a-f]{40}$\x27)",
  "token_transfers_from_address_check:token_transfers:check(from_address~\x27^0x[0-9a-f]{40}$\x27)",
  "token_transfers_to_address_check:token_transfers:check(to_address~\x27^0x[0-9a-f]{40}$\x27)",
  `token_transfers_value_check:token_transfers:check(casewhenvalue~\x27^(0|[1-9][0-9]{0,77})$\x27thenvalue<=\x27${u256Max}\x27elsefalseend)`,
  "token_transfers_block_number_check:token_transfers:check(block_number>=0)"
], schema.checkConstraints);
exact("schema index descriptors", [
  "blocks_pkey:blocks:unique-primary:btree(chain_id,number)",
  "blocks_chain_hash_key:blocks:unique:btree(chain_id,hash)",
  "idx_blocks_timestamp:blocks:regular:btree(chain_id,\x22timestamp\x22-desc,number-desc)",
  "transactions_pkey:transactions:unique-primary:btree(chain_id,hash)",
  "transactions_chain_hash_block_key:transactions:unique:btree(chain_id,hash,block_number)",
  "idx_transactions_block:transactions:regular:btree(chain_id,block_number-desc,hash-desc)",
  "token_transfers_pkey:token_transfers:unique-primary:btree(chain_id,tx_hash,log_index)",
  "idx_transfers_token:token_transfers:regular:btree(chain_id,token_address,block_number-desc,tx_hash-desc,log_index-desc)",
  "idx_transfers_from:token_transfers:regular:btree(chain_id,from_address,block_number-desc,tx_hash-desc,log_index-desc)",
  "idx_transfers_to:token_transfers:regular:btree(chain_id,to_address,block_number-desc,tx_hash-desc,log_index-desc)"
], schema.indexes);
exact("schema fail-closed policy", {
  globalTransactionHashPrimaryKeyAccepted: false,
  extraColumnsConstraintsOrIndexesAccepted: false,
  inheritanceAccepted: false,
  rowLevelSecurityAccepted: false,
  partialExpressionOrIncludedIndexesAccepted: false,
  nonCatalogOperatorClassesAccepted: false,
  indexCollationDriftAccepted: false
}, {
  globalTransactionHashPrimaryKeyAccepted: schema.globalTransactionHashPrimaryKeyAccepted,
  extraColumnsConstraintsOrIndexesAccepted: schema.extraColumnsConstraintsOrIndexesAccepted,
  inheritanceAccepted: schema.inheritanceAccepted,
  rowLevelSecurityAccepted: schema.rowLevelSecurityAccepted,
  partialExpressionOrIncludedIndexesAccepted: schema.partialExpressionOrIncludedIndexesAccepted,
  nonCatalogOperatorClassesAccepted: schema.nonCatalogOperatorClassesAccepted,
  indexCollationDriftAccepted: schema.indexCollationDriftAccepted
});
for (const anchor of [
  "CONSTRAINT transactions_pkey PRIMARY KEY (chain_id, hash)",
  "CONSTRAINT transactions_block_fkey FOREIGN KEY (chain_id, block_number)",
  "CONSTRAINT token_transfers_transaction_fkey FOREIGN KEY (chain_id, tx_hash, block_number)",
  "ON UPDATE NO ACTION ON DELETE NO ACTION",
  "CONSTRAINT blocks_gas_bounds_check CHECK (gas_used <= gas_limit)",
  "ORDER BY block_number DESC, tx_hash DESC, log_index DESC"
]) if (!(sql.includes(anchor) || main.includes(anchor))) fail(`schema/runtime anchor is missing: ${anchor}`);
if (!Array.isArray(fixture.blockers) || fixture.blockers.length !== 10 || fixture.blockers.some((item) => typeof item !== "string" || !item)) fail("exact ten residual blockers are required");

const report = {
  contractId: fixture.contractId,
  productionReady: false,
  readinessExit: 3,
  provenance: { sourceCommit: provenance.sourceCommit, standaloneSourceIndexer: false, removedRuntimeBlob: snapshot.blob },
  runtimeRust: { files: runtime.rustInventory.length, ddlFindings: 0, expectedDelta: -5, qualifiedRelations: runtime.qualifiedRelationOccurrences, fakeSyncAvailable: false },
  dormantAdapter: {
    status: adapter.status,
    feature: adapter.feature,
    sourcePins: adapter.sourcePins,
    defaultEnabled: false,
    privateModule: true,
    publicExport: false,
    mainCallsite: false,
    poolHolderOnly: true,
    parentConflictTargetOnly: true,
    strictChildInserts: true,
    fullCandidateReload: true,
    reloadRevalidation: true,
    databaseClockLeasePredicates: true,
    persistentLeaseFence: true,
    readSide: {
      helpers: true,
      modulePrivate: true,
      repeatableReadOnlyTransactions: true,
      candidateReadSingleSnapshot: true,
      snapshotReadSingleSnapshot: true,
      absentStateRejectsSelectedOrJournalOrphans: true,
      snapshotMappingChecks: true,
      snapshotRevisionChecks: true,
      selectedHashLeftJoinsChainState: true,
      selectedHashRejectsMissingStaleOrFutureState: true,
      candidateReadsLegacyProjection: false
    },
    tests: { defaultLibraryPassed: 33, featureLibraryPassed: 50, binaryPassed: 4 },
    databaseRead: false,
    databaseWrite: false,
    migrationExecuted: false,
    runtimeAdapter: false,
    providerActivated: false,
    workerActivated: false,
    routeActivated: false,
    executed: false
  },
  migrationRoot: {
    migrations: 2,
    runner: null,
    projection: { pinnedBytes: ordered.bytes, sha256: ordered.sha256, guardedTables: 3, guardedIndexes: 5 },
    forkStore: { pinnedBytes: forkMigration.bytes, sha256: forkMigration.sha256, guardedTables: 8, guardedIndexes: 2 }
  },
  schema: { tables: 3, columns: 27, structuralConstraints: schema.structuralConstraints.length, checkConstraints: schema.checkConstraints.length, indexes: schema.indexes.length, transactionPrimaryKey: ["chain_id", "hash"] },
  forkStore: { status: forkStore.status, tables: 8, columns: forkStore.columns, structuralConstraints: forkStore.structuralConstraints.length, checkConstraints: forkStore.checkConstraints, explicitIndexes: forkStore.explicitIndexes.length, collisionPreflight: true, collisionNames: forkStore.collisionPreflightNames.length, freshCreateOnly: true, runtimeProbe: false, executed: false },
  blockers: fixture.blockers
};
if (mode === "report") { console.log(JSON.stringify(report, null, 2)); process.exit(0); }
if (mode === "readiness") {
  console.error("a3-12-indexer-schema-boundary: STOP: ten residual A3.12 blockers remain; local static integrity is not production readiness");
  process.exit(3);
}
console.log("a3-12-indexer-schema-boundary: PASS: indexer runtime DDL 5→0; three guarded public tables, 27 exact columns and chain-scoped transaction PK pinned");
console.log("a3-12-indexer-schema-boundary: PASS: 31 exact constraints and 10 exact btree indexes fail closed on FK/check/index/inheritance/RLS/opclass/collation drift");
console.log("a3-12-indexer-schema-boundary: PASS: DateTime<Utc>, nullable fields, canonical chain/hash/address parsing and checked numeric conversion pinned");
console.log("a3-12-indexer-schema-boundary: PASS: schema probe precedes listener; autonomous provider, placeholder sync and fabricated ingestion are absent");
console.log("a3-12-indexer-schema-boundary: PASS: all four surviving runtime relations are public-qualified; only health remains reachable");
console.log("a3-12-indexer-schema-boundary: PASS: dormant fork store pins eight guarded tables, 74 columns, 101 constraints and two explicit indexes without runtime activation");
console.log("a3-12-indexer-schema-boundary: PASS: ten-name fresh-create preflight rejects every public relation-kind collision before CREATE");
console.log("a3-12-indexer-schema-boundary: PASS: seven ordered dormant-adapter source byte/SHA-256 pins are recomputed before semantic anchors");
console.log("a3-12-indexer-schema-boundary: PASS: default-off private PostgreSQL substrate pins strict candidate reload, codecs, database-clock leases and consistent read helpers without runtime activation");
'
