#!/bin/sh
set -eu

usage() {
  echo "usage: $0 [--mode integrity|report|readiness] [--contract PATH]" >&2
  exit 64
}

mode=integrity
contract_path=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      [ "$#" -ge 2 ] || usage
      mode=$2
      shift 2
      ;;
    --contract)
      [ "$#" -ge 2 ] || usage
      contract_path=$2
      shift 2
      ;;
    *) usage ;;
  esac
done
case "$mode" in integrity|report|readiness) ;; *) usage ;; esac

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
[ -n "$contract_path" ] || contract_path="$repo_root/docs/migration/contracts/a3-9-wallet-schema-boundary.json"

command -v bun >/dev/null 2>&1 || {
  echo "a3-9-wallet-schema-boundary: ERROR: bun is required" >&2
  exit 1
}

if [ "${EPSX_ENV:-}" = production ] || [ "${NODE_ENV:-}" = production ]; then
  echo "a3-9-wallet-schema-boundary: ERROR: refusing a production-looking environment" >&2
  exit 1
fi
if [ -n "${WALLET_DATABASE_URL:-}" ]; then
  echo "a3-9-wallet-schema-boundary: ERROR: this verifier never contacts a database; unset WALLET_DATABASE_URL" >&2
  exit 1
fi

report=$(bun -e '
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, lstatSync, readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { resolve, sep } from "node:path";

const [rootInput, contractInput] = process.argv.slice(1);
const root = realpathSync(rootInput);
const contractPath = resolve(contractInput);
const fail = (message) => {
  console.error(`a3-9-wallet-schema-boundary: ERROR: ${message}`);
  process.exit(1);
};
const exact = (label, expected, actual) => {
  if (JSON.stringify(expected) !== JSON.stringify(actual)) fail(`${label} drifted`);
};
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const safeRelative = (value, label) => {
  if (typeof value !== "string" || value.length === 0 || value.startsWith("/") || value.split("/").includes("..")) fail(`${label} must be repository-relative`);
};
const repoPath = (relative, label) => {
  safeRelative(relative, label);
  const path = resolve(root, relative);
  if (path !== root && !path.startsWith(`${root}${sep}`)) fail(`${label} escapes repository root`);
  return path;
};
const regularRepoFile = (relative, label) => {
  const path = repoPath(relative, label);
  if (!existsSync(path) || lstatSync(path).isSymbolicLink() || !statSync(path).isFile()) fail(`${label} must be a regular file`);
  if (realpathSync(path) !== path) fail(`${label} may not traverse symbolic links`);
  return path;
};

let contract;
try {
  if (!existsSync(contractPath) || lstatSync(contractPath).isSymbolicLink() || !statSync(contractPath).isFile()) fail("contract must be a regular file");
  contract = JSON.parse(readFileSync(contractPath, "utf8"));
} catch (error) {
  fail(`invalid contract: ${error.message}`);
}

if (contract.schemaVersion !== 1 || contract.contractId !== "a3.9.wallet-schema-boundary.v1") fail("contract identity drifted");
if (contract.purpose !== "offline-static-wallet-schema-boundary-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", {
  service: "services/wallet",
  database: "epsx_wallet",
  schema: "public",
  tables: ["accounts", "nonces", "signed_transactions"],
  status: "partial",
}, contract.scope);
if (!contract.safety || typeof contract.safety.readinessMeaning !== "string") fail("safety boundary is required");
for (const [key, value] of Object.entries(contract.safety)) {
  if (key !== "readinessMeaning" && value !== false) fail(`${key} must remain false`);
}

const development = contract.developmentBaseline;
if (!development || development.sourceRefLabel !== "origin/development" || development.targetCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || development.candidatePath !== "services/wallet" || development.candidateServicePresent !== false || development.status !== "blocked") fail("development baseline drifted");
const commitCheck = spawnSync("git", ["-C", root, "cat-file", "-e", `${development.targetCommit}^{commit}`], { stdio: "ignore" });
if (commitCheck.status !== 0) fail("immutable development commit is unavailable locally");
const candidateCheck = spawnSync("git", ["-C", root, "cat-file", "-e", `${development.targetCommit}:${development.candidatePath}`], { stdio: "ignore" });
if (candidateCheck.status === 0) fail("development candidate unexpectedly exists");

const runtime = contract.runtimeBoundary;
if (!runtime || runtime.rustRoot !== "services/wallet" || runtime.scannerFindingBefore !== 3 || runtime.scannerFindingAfter !== 0) fail("runtime scanner boundary drifted");
exact("runtime Rust inventory", ["services/wallet/src/lib.rs", "services/wallet/src/main.rs"], runtime.rustInventory);
exact("removed runtime DDL anchors", [
  "CREATE TABLE IF NOT EXISTS accounts (",
  "CREATE TABLE IF NOT EXISTS nonces (",
  "CREATE TABLE IF NOT EXISTS signed_transactions (",
], runtime.removedAnchors);
if (runtime.compatibilityQueryConstant !== "WALLET_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility" || runtime.compatibilityQueryBytes !== 22561 || runtime.compatibilityQuerySha256 !== "a46ba81e71d77d13f35c40437e79ff0f45e3365efe6898908e5b18177082c71d") fail("compatibility query boundary or pin drifted");
exact("qualified relation counts", {
  "public.accounts": 3,
  "public.nonces": 1,
  "public.signed_transactions": 1,
}, runtime.qualifiedRelationOccurrences);

const rustRoot = repoPath(runtime.rustRoot, "wallet Rust root");
if (!existsSync(rustRoot) || lstatSync(rustRoot).isSymbolicLink() || !statSync(rustRoot).isDirectory()) fail("wallet Rust root is missing");
const rustFiles = [];
const visit = (directory) => {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const path = resolve(directory, entry.name);
    if (entry.isSymbolicLink()) fail("symbolic links are forbidden under wallet Rust root");
    if (entry.isDirectory()) visit(path);
    else if (entry.isFile() && entry.name.endsWith(".rs")) rustFiles.push(path);
  }
};
visit(rustRoot);
rustFiles.sort();
exact("observed Rust inventory", runtime.rustInventory, rustFiles.map((file) => file.slice(root.length + 1)));

const stripRustComments = (content) => {
  let inBlock = false;
  return content.split(/\r?\n/).map((line) => {
    let result = "";
    for (let index = 0; index < line.length; index += 1) {
      if (inBlock) {
        if (line.slice(index, index + 2) === "*/") { inBlock = false; index += 1; }
        continue;
      }
      if (line.slice(index, index + 2) === "/*") { inBlock = true; index += 1; continue; }
      if (line.slice(index, index + 2) === "//") break;
      result += line[index];
    }
    return result;
  });
};
const runtimeDdlPattern = /\b(?:CREATE|ALTER|DROP|TRUNCATE)\s+(?:OR\s+REPLACE\s+)?(?:TABLE|SCHEMA|INDEX|TYPE|VIEW|MATERIALIZED\s+VIEW|DATABASE)\b/i;
const runtimeFindings = [];
for (const file of rustFiles) {
  const relative = file.slice(root.length + 1);
  stripRustComments(readFileSync(file, "utf8")).forEach((line, index) => {
    const match = line.match(runtimeDdlPattern);
    if (match) runtimeFindings.push({ file: relative, line: index + 1, kind: match[0] });
  });
}
if (runtimeFindings.length !== 0) fail(`wallet runtime Rust DDL scanner found ${runtimeFindings.length}, expected zero`);

const lib = readFileSync(regularRepoFile("services/wallet/src/lib.rs", "wallet library"), "utf8");
const main = readFileSync(regularRepoFile("services/wallet/src/main.rs", "wallet main"), "utf8");
for (const anchor of runtime.removedAnchors) if (lib.includes(anchor) || main.includes(anchor)) fail(`removed runtime schema anchor returned: ${anchor}`);
const queryStartAnchor = `const ${runtime.compatibilityQueryConstant}: &str = r#"`;
const queryStart = lib.indexOf(queryStartAnchor);
const queryEnd = lib.indexOf("\"#;", queryStart + queryStartAnchor.length);
if (queryStart < 0 || queryEnd < 0) fail("read-only compatibility query constant is missing");
const query = lib.slice(queryStart + queryStartAnchor.length, queryEnd);
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (!/^\s*WITH\s+/i.test(query)) fail("compatibility query must start with a read-only CTE");
if (/\b(?:INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i.test(query)) fail("compatibility query contains a mutation or command token");
if (!Array.isArray(runtime.queryRequiredAnchors) || runtime.queryRequiredAnchors.length !== 41) fail("exactly forty-one compatibility-query anchors are required");
for (const anchor of runtime.queryRequiredAnchors) if (typeof anchor !== "string" || !query.includes(anchor)) fail(`missing compatibility-query anchor: ${anchor}`);
if (!query.includes("bool_and(COALESCE(") || !query.includes("END,\n                false")) fail("bool_and/default comparisons must be NULL-safe");
const apostrophe = String.fromCharCode(39);
const allowedSerialDefaults = [
  `nextval(${apostrophe}signed_transactions_id_seq${apostrophe}::regclass)`,
  `nextval(${apostrophe}public.signed_transactions_id_seq${apostrophe}::regclass)`,
];
exact("adversarial query guards", {
  constraintInventoryJoin: "LEFT JOIN pg_catalog.pg_index AS constraint_index",
  inheritanceInventory: "FROM pg_catalog.pg_inherits AS inheritance_record",
  relationNotExistsStructure: "exactly-one-not-exists-before-pg-inherits",
  notNullCatalogExposure: `pg_catalog.current_setting(${apostrophe}server_version_num${apostrophe})::integer >= 180000`,
  prePg18NoRowPath: "NOT exposure.exposed",
  notNullEnforcement: `pg_catalog.to_jsonb(constraint_record) ->> ${apostrophe}conenforced${apostrophe}`,
  nullableColumnRule: "expected nullable columns have no contype=n constraint",
  allowedSerialDefaultExpressions: allowedSerialDefaults,
  rejectedSerialDefaultShape: "double-nextval-expression",
}, runtime.adversarialGuards);
if (query.includes("actual.column_default LIKE")) fail("serial default must not use a wildcard expression check");
if (query.includes("\n        JOIN pg_catalog.pg_index AS constraint_index")) fail("constraint inventory must not inner-join away CHECK/FK rows");
if (!query.includes("inheritance_record.inhrelid = relation_record.oid") || !query.includes("inheritance_record.inhparent = relation_record.oid")) fail("inheritance child/parent rejection drifted");
const normalizedQuery = query.replace(/\s+/g, " ");
if (normalizedQuery.includes("AND NOT EXISTS ( AND NOT EXISTS (")) fail("relation compatibility contains a duplicate NOT EXISTS opener");
const relationStart = query.indexOf("relation_compatibility AS (");
const relationEnd = query.indexOf("expected_primary_keys (table_name, key_columns) AS (", relationStart);
if (relationStart < 0 || relationEnd < 0) fail("relation compatibility CTE boundary drifted");
const relationCompatibility = query.slice(relationStart, relationEnd);
if ((relationCompatibility.match(/AND\s+NOT\s+EXISTS\s*\(/gi) || []).length !== 1) fail("relation compatibility must contain exactly one inheritance NOT EXISTS guard");
if ((relationCompatibility.match(/FROM\s+pg_catalog\.pg_inherits\s+AS\s+inheritance_record/gi) || []).length !== 1) fail("relation compatibility must inspect inheritance exactly once");
if (query.includes("constraint_record.conenforced")) fail("pre-PG18 parsing must not reference a missing conenforced column directly");
for (const anchor of [
  "NOT exposure.exposed",
  `expected.is_nullable = ${apostrophe}YES${apostrophe}`,
  `expected.is_nullable = ${apostrophe}NO${apostrophe}`,
  "OR NOT constraint_record.convalidated",
  "OR constraint_record.condeferrable",
  "OR constraint_record.condeferred",
  `pg_catalog.to_jsonb(constraint_record) ->> ${apostrophe}conenforced${apostrophe}`,
]) if (!query.includes(anchor)) fail(`PG18 NOT NULL guard is missing: ${anchor}`);
for (const expression of allowedSerialDefaults) {
  const sqlExpression = expression.replaceAll(apostrophe, `${apostrophe}${apostrophe}`);
  if ((query.split(sqlExpression).length - 1) !== 2) fail(`single-nextval allowlist occurrence drifted: ${expression}`);
}
const doubleNextval = `${allowedSerialDefaults[0]} + ${allowedSerialDefaults[0]}`;
if (allowedSerialDefaults.includes(doubleNextval)) fail("double-nextval expression entered the allowlist");
for (const table of contract.scope.tables) {
  const quotedTable = `${String.fromCharCode(39)}${table}${String.fromCharCode(39)}`;
  if (!query.includes(`(${quotedTable},`) && !query.includes(`(${quotedTable}, `)) fail(`expected table is missing from compatibility query: ${table}`);
}

for (const [relation, expected] of Object.entries(runtime.qualifiedRelationOccurrences)) {
  const bare = relation.split(".")[1];
  const escapedBare = bare.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const escapedRelation = relation.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const operation = "(?:INSERT\\s+INTO|DELETE\\s+FROM|UPDATE|FROM|JOIN)";
  const unqualified = main.match(new RegExp(`\\b${operation}\\s+${escapedBare}\\b`, "gi")) ?? [];
  if (unqualified.length !== 0) fail(`runtime SQL contains ${unqualified.length} unqualified ${bare} reference(s)`);
  const qualified = main.match(new RegExp(`\\b${operation}\\s+${escapedRelation}\\b`, "gi")) ?? [];
  if (qualified.length !== expected) fail(`runtime SQL has ${qualified.length} ${relation} references, expected ${expected}`);
}

const functionStart = lib.indexOf(`pub async fn ${runtime.compatibilityFunction}(`);
const functionEnd = lib.indexOf("#[derive(Debug, Error)]\npub enum WalletConfigError", functionStart);
if (functionStart < 0 || functionEnd < 0) fail("compatibility function boundary is missing");
const functionBody = lib.slice(functionStart, functionEnd);
for (const anchor of [
  "sqlx::query_scalar::<_, bool>(WALLET_SCHEMA_COMPATIBILITY_QUERY)",
  ".fetch_one(db)",
  "WalletSchemaError::Incompatible",
]) if (!functionBody.includes(anchor)) fail(`compatibility function is missing ${anchor}`);
if (functionBody.includes(".execute(")) fail("compatibility function must not execute a mutation statement");

if (!Array.isArray(runtime.mainSequence) || runtime.mainSequence.length !== 4) fail("main sequence must contain four anchors");
let previous = -1;
for (const anchor of runtime.mainSequence) {
  const index = main.indexOf(anchor);
  if (index < 0) fail(`main sequence is missing: ${anchor}`);
  if (index <= previous) fail(`main sequence is out of order: ${anchor}`);
  previous = index;
}

const migrationRoot = contract.migrationRoot;
if (!migrationRoot || migrationRoot.path !== "services/wallet/migrations" || migrationRoot.runner !== null || migrationRoot.transactionOwner !== "future-reviewed-runner") fail("migration-root boundary drifted");
const migrationRootPath = repoPath(migrationRoot.path, "migration root");
if (!existsSync(migrationRootPath) || lstatSync(migrationRootPath).isSymbolicLink() || !statSync(migrationRootPath).isDirectory()) fail("migration root must be a real directory");
exact("migration-root inventory", ["20260722020000_create_wallet_store.sql"], readdirSync(migrationRootPath).sort());
if (!Array.isArray(migrationRoot.orderedMigrations) || migrationRoot.orderedMigrations.length !== 1) fail("exactly one ordered wallet migration is required");
const migration = migrationRoot.orderedMigrations[0];
if (migration.version !== "20260722020000" || migration.path !== "services/wallet/migrations/20260722020000_create_wallet_store.sql" || migration.bytes !== 775 || migration.sha256 !== "cf79bdb4e999d4cfb54648ba8d82e845af7c5feaccd20d5ca2143ff673ca1731") fail("ordered migration pin drifted");
exact("migration guards", [
  "CREATE TABLE IF NOT EXISTS public.accounts (",
  "CREATE TABLE IF NOT EXISTS public.nonces (",
  "CREATE TABLE IF NOT EXISTS public.signed_transactions (",
], migration.guards);
const migrationBytes = readFileSync(regularRepoFile(migration.path, "wallet migration"));
const migrationSql = migrationBytes.toString("utf8");
if (migrationBytes.byteLength !== migration.bytes || sha256(migrationBytes) !== migration.sha256) fail("wallet migration bytes changed");
if ((migrationSql.match(/;/g) ?? []).length !== 3 || (migrationSql.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\./gi) ?? []).length !== 3) fail("wallet migration must contain exactly three guarded public table statements");
for (const guard of migration.guards) if ((migrationSql.split(guard).length - 1) !== 1) fail(`migration guard must occur once: ${guard}`);
if (/\b(?:DROP|TRUNCATE|DELETE|ALTER|INSERT|UPDATE|MERGE|CASCADE)\b/i.test(migrationSql)) fail("wallet migration contains a destructive, data-mutation, or alteration token");
if (/\bCREATE\s+(?:SCHEMA|EXTENSION|INDEX|DATABASE|TYPE|VIEW)\b/i.test(migrationSql)) fail("wallet migration contains an out-of-scope creation");
if (/\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(migrationSql)) fail("transaction control belongs to the future reviewed runner");

const expectedTables = [
  {
    name: "accounts",
    columns: [
      { ordinal: 1, name: "address", databaseType: "varchar(42)", nullable: false, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "address VARCHAR(42) NOT NULL," },
      { ordinal: 2, name: "chain_id", databaseType: "varchar(10)", nullable: false, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "chain_id VARCHAR(10) NOT NULL," },
      { ordinal: 3, name: "label", databaseType: "text", nullable: true, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "label TEXT," },
      { ordinal: 4, name: "role", databaseType: "varchar(50)", nullable: true, default: `${apostrophe}user${apostrophe}::character varying`, datetimePrecision: null, collation: "database-default", sqlAnchor: `role VARCHAR(50) DEFAULT ${apostrophe}user${apostrophe},` },
      { ordinal: 5, name: "encrypted_pk", databaseType: "text", nullable: true, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "encrypted_pk TEXT," },
      { ordinal: 6, name: "created_at", databaseType: "timestamptz", nullable: true, default: "now()", datetimePrecision: 6, collation: null, sqlAnchor: "created_at TIMESTAMPTZ DEFAULT NOW()," },
    ],
    primaryKey: ["address", "chain_id"],
    primaryKeyConstraint: { keyColumns: ["address", "chain_id"], deferrable: false, initiallyDeferred: false, validated: true, indexBinding: "onlyIndex" },
    onlyIndex: { method: "btree", unique: true, primary: true, immediate: true, keyColumns: ["address", "chain_id"], collation: "matches-key-column-collation", operatorClasses: ["text_ops", "text_ops"], operatorClassNamespaces: ["pg_catalog", "pg_catalog"] },
  },
  {
    name: "nonces",
    columns: [
      { ordinal: 1, name: "address", databaseType: "varchar(42)", nullable: false, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "address VARCHAR(42) NOT NULL," },
      { ordinal: 2, name: "chain_id", databaseType: "varchar(10)", nullable: false, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "chain_id VARCHAR(10) NOT NULL," },
      { ordinal: 3, name: "nonce", databaseType: "bigint", nullable: false, default: "0", datetimePrecision: null, collation: null, sqlAnchor: "nonce BIGINT NOT NULL DEFAULT 0," },
      { ordinal: 4, name: "updated_at", databaseType: "timestamptz", nullable: true, default: "now()", datetimePrecision: 6, collation: null, sqlAnchor: "updated_at TIMESTAMPTZ DEFAULT NOW()," },
    ],
    primaryKey: ["address", "chain_id"],
    primaryKeyConstraint: { keyColumns: ["address", "chain_id"], deferrable: false, initiallyDeferred: false, validated: true, indexBinding: "onlyIndex" },
    onlyIndex: { method: "btree", unique: true, primary: true, immediate: true, keyColumns: ["address", "chain_id"], collation: "matches-key-column-collation", operatorClasses: ["text_ops", "text_ops"], operatorClassNamespaces: ["pg_catalog", "pg_catalog"] },
  },
  {
    name: "signed_transactions",
    columns: [
      { ordinal: 1, name: "id", databaseType: "serial/integer", nullable: false, default: "nextval exact owned public.signed_transactions_id_seq OID", datetimePrecision: null, collation: null, sqlAnchor: "id SERIAL PRIMARY KEY," },
      { ordinal: 2, name: "chain_id", databaseType: "varchar(10)", nullable: false, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "chain_id VARCHAR(10) NOT NULL," },
      { ordinal: 3, name: "sender", databaseType: "varchar(42)", nullable: false, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "sender VARCHAR(42) NOT NULL," },
      { ordinal: 4, name: "recipient", databaseType: "varchar(42)", nullable: true, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "recipient VARCHAR(42)," },
      { ordinal: 5, name: "value", databaseType: "varchar(78)", nullable: true, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "value VARCHAR(78)," },
      { ordinal: 6, name: "data_hash", databaseType: "varchar(66)", nullable: true, default: null, datetimePrecision: null, collation: "database-default", sqlAnchor: "data_hash VARCHAR(66)," },
      { ordinal: 7, name: "created_at", databaseType: "timestamptz", nullable: true, default: "now()", datetimePrecision: 6, collation: null, sqlAnchor: "created_at TIMESTAMPTZ DEFAULT NOW()" },
    ],
    primaryKey: ["id"],
    primaryKeyConstraint: { keyColumns: ["id"], deferrable: false, initiallyDeferred: false, validated: true, indexBinding: "onlyIndex" },
    onlyIndex: { method: "btree", unique: true, primary: true, immediate: true, keyColumns: ["id"], collation: "matches-key-column-collation", operatorClasses: ["int4_ops"], operatorClassNamespaces: ["pg_catalog"] },
    serialSequence: { name: "public.signed_transactions_id_seq", type: "integer", start: 1, increment: 1, min: 1, max: 2147483647, cache: 1, cycle: false, ownedBy: "public.signed_transactions.id", defaultDependency: "exact-sequence-oid-only" },
  },
];
exact("required wallet table/column/constraint/index contract", expectedTables, contract.requiredTables);
const expectedColumnCounts = [6, 4, 7];
let totalColumns = 0;
let nullableColumns = 0;
for (let tableIndex = 0; tableIndex < contract.requiredTables.length; tableIndex += 1) {
  const table = contract.requiredTables[tableIndex];
  if (!Array.isArray(table.columns) || table.columns.length !== expectedColumnCounts[tableIndex]) fail(`${table.name}: column count drifted`);
  const escapedTable = table.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const tableMatch = migrationSql.match(new RegExp(`CREATE\\s+TABLE\\s+IF\\s+NOT\\s+EXISTS\\s+public\\.${escapedTable}\\s*\\(([\\s\\S]*?)\\n\\);`, "i"));
  if (!tableMatch) fail(`${table.name}: migration table body is malformed`);
  const tableSql = tableMatch[1];
  for (let index = 0; index < table.columns.length; index += 1) {
    const column = table.columns[index];
    if (column.ordinal !== index + 1) fail(`${table.name}: column ${index + 1} order drifted`);
    if ((tableSql.split(column.sqlAnchor).length - 1) !== 1) fail(`${table.name}.${column.name}: SQL anchor must occur once in its table`);
    totalColumns += 1;
    if (column.nullable) nullableColumns += 1;
  }
  exact(`${table.name}: PK constraint/key alignment`, table.primaryKey, table.primaryKeyConstraint.keyColumns);
  exact(`${table.name}: PK/index key alignment`, table.primaryKey, table.onlyIndex.keyColumns);
}
if (totalColumns !== 17 || nullableColumns !== 9) fail("wallet column/nullability coverage drifted");
const sequence = contract.requiredTables[2].serialSequence;
exact("serial sequence", expectedTables[2].serialSequence, sequence);

const model = contract.modelBoundary;
if (!model || model.source !== "services/wallet/src/main.rs" || model.uuidFields !== 0 || model.nullableResponseFields !== 2 || model.hermeticBinaryTests !== 4) fail("model boundary drifted");
const checkSlice = (label, boundary) => {
  const start = main.indexOf(boundary.start);
  const end = main.indexOf(boundary.end, start);
  if (start < 0 || end < 0) fail(`${label} source slice anchors are missing`);
  const slice = main.slice(start, end);
  if (Buffer.byteLength(slice) !== boundary.bytes || sha256(slice) !== boundary.sha256) fail(`${label} source slice bytes changed`);
  return slice;
};
checkSlice("account model", model.accountSlice);
checkSlice("transaction model", model.transactionSlice);
const sendOperation = checkSlice("send operation", model.sendOperationSlice);
const bindHelpers = checkSlice("bind helpers", model.bindHelperSlice);
exact("account response model", [{
  name: "AccountResponse",
  fields: [
    { name: "address", rustType: "String", databaseType: "varchar(42)", nullable: false },
    { name: "chain_id", rustType: "String", databaseType: "varchar(10)", nullable: false },
    { name: "label", rustType: "Option<String>", databaseType: "text", nullable: true },
    { name: "role", rustType: "Option<String>", databaseType: "varchar(50)", nullable: true },
  ],
}], model.responseModels);
exact("database scalar types", [
  { relationColumn: "public.nonces.nonce", rustType: "i64", databaseType: "bigint", sourceAnchor: "let nonce: i64 = sqlx::query_scalar(" },
  { responseField: "SendTxResponse.nonce", rustType: "u64", conversionAnchor: "u64::try_from(nonce)" },
], model.databaseScalarTypes);
for (const item of model.databaseScalarTypes) {
  const anchor = item.sourceAnchor ?? item.conversionAnchor;
  if (!main.includes(anchor)) fail(`database scalar anchor is missing: ${anchor}`);
}
exact("bounded bind anchors", [
  "canonical_evm_address(provided)?.1",
  "canonical_evm_address(&req.from)?",
  "canonical_evm_address(&req.to)?",
  "canonical_transaction_value(&req.value)?",
  "database_chain_id(req.chain_id)?",
  "role.chars().count() > 50",
  "bytes.len() > 32",
], model.boundedBindAnchors);
for (const anchor of model.boundedBindAnchors) if (!main.includes(anchor)) fail(`bounded bind anchor is missing: ${anchor}`);
exact("transaction boundary", {
  beginAnchor: ".db\n        .begin()",
  nonceFetchAnchor: ".fetch_one(&mut *transaction)",
  checkedNonceAnchor: "u64::try_from(nonce)",
  signedInsertAnchor: ".execute(&mut *transaction)",
  commitAnchor: ".commit()",
  executorOccurrences: 2,
  status: "static-source-pinned-routes-remain-disabled",
}, model.transactionBoundary);
let operationPrevious = -1;
for (const anchor of [
  "canonical_evm_address(&req.from)?",
  "canonical_evm_address(&req.to)?",
  "canonical_transaction_value(&req.value)?",
  model.transactionBoundary.beginAnchor,
  model.transactionBoundary.nonceFetchAnchor,
  model.transactionBoundary.checkedNonceAnchor,
  model.transactionBoundary.signedInsertAnchor,
  model.transactionBoundary.commitAnchor,
]) {
  const index = sendOperation.indexOf(anchor);
  if (index < 0 || index <= operationPrevious) fail(`send transaction sequence drifted: ${anchor}`);
  operationPrevious = index;
}
const transactionExecutors = (sendOperation.match(/\.(?:fetch_one|execute)\(&mut \*transaction\)/g) ?? []).length;
if (transactionExecutors !== model.transactionBoundary.executorOccurrences) fail("transaction executor occurrence count drifted");
if (sendOperation.includes(".fetch_one(&state.db)") || sendOperation.includes(".execute(&state.db)")) fail("send operation escaped the SQL transaction");
for (const anchor of [
  "Address::from_str(value)",
  "format!(\"{address:#x}\").to_ascii_lowercase()",
  "U256::from_str_radix(hex, 16)",
  "U256::from_str_radix(value, 10)",
  "Ok(parsed.to_string())",
]) if (!bindHelpers.includes(anchor)) fail(`canonical bind helper is missing: ${anchor}`);
for (const anchor of [".bind(&sender)", ".bind(&recipient)", ".bind(&value)"]) if (!sendOperation.includes(anchor)) fail(`canonical transaction bind is missing: ${anchor}`);
if (!main.includes("canonical_evm_address(provided)?.1") || !main.includes(".bind(&address)")) fail("create-account address is not parsed/canonicalized before bind");
if (!lib.includes("(&Method::POST, [\"accounts\" | \"send\" | \"sign-message\"])")) fail("custody routes no longer share the disabled policy");
if (!lib.includes("AccessPolicy::UnsafeCustodyMutation\n        | AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response()")) fail("disabled custody routes no longer fail closed");
exact("request bind fields", ["address", "chain_id", "label", "role", "from", "to", "value", "data"], model.requestBindFields);
if (!main.includes("role: Option<String>,") || !main.includes("role: Some(role),")) fail("nullable role response mapping drifted");
if (!main.includes("#[derive(Serialize, Deserialize, FromRow)]\nstruct AccountResponse")) fail("account response must retain FromRow");

exact("isolated scanner delta", {
  runtimeRustDdlFindings: { before: 35, after: 32, delta: -3 },
  actionableFindings: { before: 29, after: 26, delta: -3 },
  walletFindings: { before: 3, after: 0, delta: -3 },
  reviewedExceptions: { before: 6, after: 6, delta: 0 },
  findingIdsAtInspection: ["finding.033", "finding.034", "finding.035"],
  assignedHistoricalRange: ["finding.035", "finding.036", "finding.037"],
  status: "isolated-projection-only-canonical-rebaseline-owned-elsewhere",
}, contract.isolatedScannerDelta);
if (!Array.isArray(contract.nonClaims) || contract.nonClaims.length !== 4 || contract.nonClaims.some((item) => typeof item !== "string" || item.length < 50)) fail("four substantive non-claims are required");
const blockerCategories = ["migration-runner", "baseline-adoption", "populated-upgrade", "reconciliation", "concurrent-startup", "live-database"];
if (!Array.isArray(contract.blockers) || contract.blockers.length !== 6) fail("exactly six residual blockers are required");
for (let index = 0; index < contract.blockers.length; index += 1) {
  const blocker = contract.blockers[index];
  const id = `B${String(index + 1).padStart(2, "0")}`;
  if (blocker.id !== id || blocker.category !== blockerCategories[index] || blocker.status !== "blocked" || typeof blocker.summary !== "string" || blocker.summary.length < 50) fail(`${id}: residual blocker drifted`);
}

const result = {
  schemaVersion: 1,
  contractId: contract.contractId,
  service: contract.scope.service,
  status: contract.scope.status,
  developmentMapping: { candidateServicePresent: false, status: "blocked", targetCommit: development.targetCommit },
  runtimeRust: {
    files: rustFiles.length,
    ddlFindings: runtimeFindings.length,
    expectedDelta: -3,
    qualifiedRelationOccurrences: runtime.qualifiedRelationOccurrences,
    compatibilityQueryBytes: Buffer.byteLength(query),
    compatibilityQuerySha256: sha256(query),
  },
  migrationRoot: { path: migrationRoot.path, migrations: 1, pinnedBytes: migration.bytes, sha256: migration.sha256, runner: null },
  schema: { tables: 3, columns: totalColumns, nullableColumns, expectedNotNullColumns: totalColumns - nullableColumns, pg18NotNullInventory: true, prePg18NoRowPath: true, constraints: 3, indexes: 3, serialSequences: 1, exactDefaultDependencies: 1, datetimePrecisionColumns: 3, databaseDefaultCollationColumns: 12 },
  models: { responseFields: 4, nullableResponseFields: 2, uuidFields: 0, boundedBindAnchors: model.boundedBindAnchors.length, hermeticBinaryTests: model.hermeticBinaryTests, atomicTransactionExecutors: transactionExecutors },
  blockers: contract.blockers.map(({ id, category, status }) => ({ id, category, status })),
  productionReady: false,
  readinessExit: 3,
};
process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
' -- "$repo_root" "$contract_path") || exit 1

if [ "$mode" = report ]; then
  printf '%s\n' "$report"
  exit 0
fi
if [ "$mode" = integrity ]; then
  echo "a3-9-wallet-schema-boundary: PASS — wallet runtime DDL 3→0, one 775-byte migration pinned, three tables/17 columns and Rust bind models verified"
  echo "a3-9-wallet-schema-boundary: LIMIT — no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran"
  exit 0
fi

echo "a3-9-wallet-schema-boundary: STOP — six residual A3.9 blockers remain; readiness is intentionally exit 3" >&2
echo "a3-9-wallet-schema-boundary: LIMIT — static integrity is not migration or database execution evidence" >&2
exit 3
