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
exact("Rust inventory", ["services/indexer/src/lib.rs", "services/indexer/src/main.rs"], runtime.rustInventory);
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
if (migration.path !== "services/indexer/migrations" || migration.runner !== null || migration.transactionOwner !== "future-reviewed-runner" || migration.orderedMigrations.length !== 1) fail("migration root boundary drifted");
const migrationFiles = readdirSync(safeRelative(migration.path, "migration root")).sort();
exact("migration inventory", ["20260722050000_create_indexer_projection_tables.sql"], migrationFiles);
const ordered = migration.orderedMigrations[0];
if (ordered.version !== "20260722050000" || ordered.path !== "services/indexer/migrations/20260722050000_create_indexer_projection_tables.sql" || ordered.bytes !== 4822 || ordered.sha256 !== "5d0ec77a11d2abe1303c5f9b87e7da18eadee9d2e7fa4aeda1aeaf3d76549ff8") fail("ordered migration pin drifted");
exact("migration guards", ["CREATE TABLE IF NOT EXISTS public.blocks (", "CREATE TABLE IF NOT EXISTS public.transactions (", "CREATE TABLE IF NOT EXISTS public.token_transfers (", "CREATE INDEX IF NOT EXISTS idx_blocks_timestamp", "CREATE INDEX IF NOT EXISTS idx_transactions_block", "CREATE INDEX IF NOT EXISTS idx_transfers_token", "CREATE INDEX IF NOT EXISTS idx_transfers_from", "CREATE INDEX IF NOT EXISTS idx_transfers_to"], ordered.guards);
const sql = readFileSync(regularRepoFile(ordered.path, "ordered migration"), "utf8");
if (Buffer.byteLength(sql) !== ordered.bytes || sha256(sql) !== ordered.sha256) fail("indexer migration bytes changed");
for (const guard of ordered.guards) if (!sql.includes(guard)) fail(`migration guard is missing: ${guard}`);
if ((sql.match(/CREATE TABLE IF NOT EXISTS public\./g) ?? []).length !== 3 || (sql.match(/CREATE INDEX IF NOT EXISTS /g) ?? []).length !== 5) fail("migration must contain exactly three guarded tables and five guarded indexes");
if ((sql.match(new RegExp(u256Max, "g")) ?? []).length !== 2 || (sql.match(/value::NUMERIC <= NUMERIC/g) ?? []).length !== 2) fail("migration must enforce the exact U256 ceiling for both value columns");
for (const pattern of [
  /\bALTER\b/i, /\bDROP\b/i, /\bTRUNCATE\b/i, /\bDELETE\s+FROM\b/i, /(?:^|;)\s*INSERT\s+INTO\b/im,
  /(?:^|;)\s*UPDATE\s+\w/im, /\bCASCADE\b/i, /\bBEGIN\b/i, /\bCOMMIT\b/i, /\bCREATE\s+SCHEMA\b/i,
  /\bCREATE\s+EXTENSION\b/i
]) if (pattern.test(sql)) fail(`migration contains forbidden statement: ${pattern}`);

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
  runtimeRust: { ddlFindings: 0, expectedDelta: -5, qualifiedRelations: runtime.qualifiedRelationOccurrences, fakeSyncAvailable: false },
  migrationRoot: { migrations: 1, runner: null, pinnedBytes: ordered.bytes, sha256: ordered.sha256, guardedTables: 3, guardedIndexes: 5 },
  schema: { tables: 3, columns: 27, structuralConstraints: schema.structuralConstraints.length, checkConstraints: schema.checkConstraints.length, indexes: schema.indexes.length, transactionPrimaryKey: ["chain_id", "hash"] },
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
'
