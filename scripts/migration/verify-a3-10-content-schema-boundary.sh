#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-10-content-schema-boundary.json"
mode=""

die() {
  echo "a3-10-content-schema-boundary: ERROR: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      [ "$#" -ge 2 ] || die "--mode requires integrity, readiness, or report"
      mode=$2
      shift 2
      ;;
    --contract)
      [ "$#" -ge 2 ] || die "--contract requires a local JSON file"
      contract=$2
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

case "$mode" in
  integrity|readiness|report) ;;
  *) die "--mode must be integrity, readiness, or report" ;;
esac

case "$contract" in
  http://*|https://*) die "contract must be a local file" ;;
esac
[ -f "$contract" ] || die "missing contract: $contract"
command -v bun >/dev/null 2>&1 || die "bun is required"
command -v git >/dev/null 2>&1 || die "git is required"

for name in DATABASE_URL CONTENT_DATABASE_URL PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts a database"
done

for name in EPSX_ENV APP_ENV ENVIRONMENT NODE_ENV RUST_ENV DEPLOY_ENV; do
  eval "value=\${$name-}"
  normalized=$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')
  case "$normalized" in
    prod|production|prod-*|production-*|*-prod|*-production)
      die "$name identifies a production-looking environment"
      ;;
  esac
done

export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

report=$(bun -e '
import {
  existsSync,
  lstatSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
} from "node:fs";
import { isAbsolute, resolve, sep } from "node:path";

const [rootInput, contractInput] = process.argv.slice(1);
const root = realpathSync(rootInput);
const fail = (message) => {
  console.error(`a3-10-content-schema-boundary: ERROR: ${message}`);
  process.exit(1);
};
const sha256 = (value) => {
  const hasher = new Bun.CryptoHasher("sha256");
  hasher.update(value);
  return hasher.digest("hex");
};
const stable = (value) => {
  if (Array.isArray(value)) return `[${value.map(stable).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.entries(value).sort(([a], [b]) => a.localeCompare(b)).map(([key, item]) => `${JSON.stringify(key)}:${stable(item)}`).join(",")}}`;
  }
  return JSON.stringify(value);
};
const exact = (label, expected, actual) => {
  if (stable(expected) !== stable(actual)) fail(`${label} drifted`);
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) {
    fail(`${label} must be a safe repository-relative path`);
  }
  const parts = value.split("/");
  if (parts.some((part) => !part || part === "." || part === "..")) fail(`${label} must be a safe repository-relative path`);
};
const regularRepoFile = (relative, label) => {
  safeRelative(relative, label);
  const candidate = resolve(root, relative);
  if (!existsSync(candidate) || lstatSync(candidate).isSymbolicLink() || !statSync(candidate).isFile()) fail(`${label} must be a regular file: ${relative}`);
  const actual = realpathSync(candidate);
  if (actual !== root && !actual.startsWith(`${root}${sep}`)) fail(`${label} escapes the repository: ${relative}`);
  return actual;
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed: ${result.stderr.toString().trim()}`);
  return result.stdout;
};
const gitExists = (...args) => Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" }).exitCode === 0;

let contract;
try {
  if (lstatSync(contractInput).isSymbolicLink()) fail("contract must be a regular non-symbolic-link file");
  const contractPath = realpathSync(contractInput);
  if (!statSync(contractPath).isFile()) fail("contract must be a regular non-symbolic-link file");
  contract = JSON.parse(readFileSync(contractPath, "utf8"));
} catch (error) {
  fail(`invalid contract JSON: ${error.message}`);
}

if (contract.schemaVersion !== 1 || contract.contractId !== "A3.10-content-schema-boundary") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "offline-static-content-schema-boundary-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", {
  service: "services/content",
  database: "epsx_content",
  schema: "public",
  tables: ["themes", "pages", "block_types", "edit_sessions"],
  status: "partial",
}, contract.scope);
if (!contract.safety || typeof contract.safety.readinessMeaning !== "string") fail("safety boundary is required");
for (const [key, value] of Object.entries(contract.safety)) {
  if (key !== "readinessMeaning" && value !== false) fail(`${key} must remain false`);
}

const source = contract.developmentNewsBoundary;
if (!source || source.sourceRef !== "origin/development" || source.sourceRefRole !== "provenance-label-only" || source.scopeDecision !== "legacy-news-authority-inspected-but-not-imported-by-this-content-schema-slice") fail("development news boundary drifted");
if (source.sourceCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") fail("pinned development source commit drifted");
if (!gitExists("cat-file", "-e", `${source.sourceCommit}^{commit}`)) fail("pinned development source commit is unavailable");
if (!Array.isArray(source.evidence) || source.evidence.length !== 7) fail("exactly seven pinned development news evidence rows are required");
const sourceIds = ["news-table", "news-indexes", "news-pin-columns", "news-model", "news-repository", "news-public-api", "news-admin-api"];
for (let index = 0; index < source.evidence.length; index += 1) {
  const item = source.evidence[index];
  if (item.id !== sourceIds[index] || typeof item.blob !== "string" || !/^[0-9a-f]{40}$/.test(item.blob) || typeof item.anchor !== "string" || !item.anchor) fail(`development evidence ${index + 1} drifted`);
  safeRelative(item.file, `development evidence ${item.id}`);
  const actualBlob = git("rev-parse", `${source.sourceCommit}:${item.file}`).toString().trim();
  if (actualBlob !== item.blob) fail(`${item.id}: pinned blob drifted`);
  const content = git("show", `${source.sourceCommit}:${item.file}`).toString();
  if (!content.includes(item.anchor)) fail(`${item.id}: pinned source anchor is missing`);
}

const runtime = contract.runtimeBoundary;
if (!runtime || runtime.rustRoot !== "services/content" || runtime.scannerFindingBefore !== 4 || runtime.scannerFindingAfter !== 0) fail("runtime scanner boundary drifted");
exact("Rust inventory", ["services/content/src/lib.rs", "services/content/src/main.rs"], runtime.rustInventory);
exact("removed runtime anchors", [
  "CREATE TABLE IF NOT EXISTS pages (",
  "CREATE TABLE IF NOT EXISTS themes (",
  "CREATE TABLE IF NOT EXISTS block_types (",
  "CREATE TABLE IF NOT EXISTS edit_sessions (",
], runtime.removedAnchors);
if (runtime.compatibilityQueryConstant !== "CONTENT_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility" || runtime.compatibilityQueryBytes !== 15196 || runtime.compatibilityQuerySha256 !== "65a6e45346adc594b4a87f9090a346a924fe666a89c06715be145e46886ced61") fail("compatibility query boundary or pin drifted");
exact("qualified relation occurrence contract", {
  "public.pages": 8,
  "public.themes": 5,
  "public.block_types": 3,
  "public.edit_sessions": 3,
}, runtime.qualifiedRelationOccurrences);
if (runtime.jsonbTextProjectionOccurrences !== 38 || runtime.returningStarOccurrences !== 0) fail("runtime projection contract drifted");

const rustRootPath = resolve(root, runtime.rustRoot);
if (!existsSync(rustRootPath) || lstatSync(rustRootPath).isSymbolicLink() || !statSync(rustRootPath).isDirectory()) fail("content Rust root is missing or unsafe");
const rustFiles = [];
const visit = (directory) => {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const path = resolve(directory, entry.name);
    if (entry.isSymbolicLink()) fail(`symbolic links are not allowed under ${runtime.rustRoot}`);
    if (entry.isDirectory()) visit(path);
    else if (entry.isFile() && entry.name.endsWith(".rs")) rustFiles.push(path);
  }
};
visit(rustRootPath);
rustFiles.sort();
exact("discovered Rust inventory", runtime.rustInventory, rustFiles.map((file) => file.slice(root.length + 1)));

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
    if (match) runtimeFindings.push({ file: relative, line: index + 1, kind: match[0].trim().replace(/\s+/g, " ").toUpperCase() });
  });
}
if (runtimeFindings.length !== 0) fail(`content runtime Rust DDL scanner found ${runtimeFindings.length}, expected zero`);

const libPath = regularRepoFile("services/content/src/lib.rs", "content library");
const mainPath = regularRepoFile("services/content/src/main.rs", "content main");
const lib = readFileSync(libPath, "utf8");
const main = readFileSync(mainPath, "utf8");
for (const anchor of runtime.removedAnchors) if (lib.includes(anchor) || main.includes(anchor)) fail(`removed runtime mutation anchor returned: ${anchor}`);
const queryMatch = lib.match(/const CONTENT_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!queryMatch) fail("read-only compatibility query constant is missing");
const query = queryMatch[1];
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (!/^\s*WITH\s+/i.test(query)) fail("compatibility query must start with a read-only CTE");
const queryMutation = /\b(?:INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i;
if (queryMutation.test(query)) fail("compatibility query contains a mutation or command token");
if (!Array.isArray(runtime.queryRequiredAnchors) || runtime.queryRequiredAnchors.length !== 31) fail("exactly thirty-one compatibility-query anchors are required");
for (const anchor of runtime.queryRequiredAnchors) if (typeof anchor !== "string" || !query.includes(anchor)) fail(`missing compatibility-query anchor: ${anchor}`);
if (!/AND\s+COALESCE\(\s*CASE expected\.default_kind[\s\S]*?END,\s*false\s*\)/.test(query)) fail("required default comparisons must coalesce SQL NULL to false");
if ((query.match(/to_regclass\(/g) ?? []).length !== 4) fail("compatibility query must resolve exactly four public relations");

const functionStart = lib.indexOf("pub async fn verify_schema_compatibility(");
const functionEnd = lib.indexOf("#[derive(Debug, Error)]", functionStart + 1);
if (functionStart < 0) fail("compatibility function is missing");
const functionBody = lib.slice(functionStart, functionEnd < 0 ? lib.length : functionEnd);
for (const anchor of [
  "sqlx::query_scalar::<_, bool>(CONTENT_SCHEMA_COMPATIBILITY_QUERY)",
  ".fetch_one(db)",
  "ContentSchemaError::Incompatible",
]) if (!functionBody.includes(anchor)) fail(`compatibility function is missing ${anchor}`);
if (functionBody.includes(".execute(")) fail("compatibility function must not execute a mutation statement");

if (!Array.isArray(runtime.modelAndBindAnchors) || runtime.modelAndBindAnchors.length !== 11) fail("exactly eleven model/bind anchors are required");
for (const anchor of runtime.modelAndBindAnchors) if (typeof anchor !== "string" || !main.includes(anchor)) fail(`missing model/bind anchor: ${anchor}`);
if (main.includes(".bind(colors.to_string())") || main.includes(".bind(fonts.to_string())") || main.includes(".bind(spacing.to_string())")) fail("JSONB sync values must not be bound as PostgreSQL text");
const jsonbTextProjectionCount = (main.match(/\b(?:blocks_json|seo_json|colors_json|fonts_json|spacing_json|breakpoints_json|radius_json|schema_json|default_props_json)::text\s+AS\s+(?:blocks_json|seo_json|colors_json|fonts_json|spacing_json|breakpoints_json|radius_json|schema_json|default_props_json)\b/g) ?? []).length;
if (jsonbTextProjectionCount !== runtime.jsonbTextProjectionOccurrences) fail(`JSONB text projection count is ${jsonbTextProjectionCount}, expected ${runtime.jsonbTextProjectionOccurrences}`);
const returningStars = main.match(/\bRETURNING\s+\*/gi) ?? [];
if (returningStars.length !== runtime.returningStarOccurrences) fail(`RETURNING * count is ${returningStars.length}, expected zero`);

const relationCounts = {};
for (const [qualifiedRelation, expectedCount] of Object.entries(runtime.qualifiedRelationOccurrences)) {
  const table = qualifiedRelation.slice("public.".length);
  const escapedTable = table.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const qualified = main.match(new RegExp(`\\b(?:INSERT\\s+INTO|UPDATE|DELETE\\s+FROM|FROM|JOIN)\\s+public\\.${escapedTable}\\b`, "gi")) ?? [];
  const unqualified = main.match(new RegExp(`\\b(?:INSERT\\s+INTO|UPDATE|DELETE\\s+FROM|FROM|JOIN)\\s+${escapedTable}\\b`, "gi")) ?? [];
  if (qualified.length !== expectedCount) fail(`${qualifiedRelation} runtime reference count is ${qualified.length}, expected ${expectedCount}`);
  if (unqualified.length !== 0) fail(`runtime SQL contains ${unqualified.length} unqualified ${table} relation reference(s)`);
  relationCounts[qualifiedRelation] = qualified.length;
}

if (!Array.isArray(runtime.mainSequence) || runtime.mainSequence.length !== 6) fail("main sequence must contain exactly six anchors");
let previous = -1;
for (const anchor of runtime.mainSequence) {
  const index = main.indexOf(anchor);
  if (index < 0) fail(`main sequence is missing: ${anchor}`);
  if (index <= previous) fail(`main sequence is out of order: ${anchor}`);
  previous = index;
}

const migrationRoot = contract.migrationRoot;
if (!migrationRoot || migrationRoot.path !== "services/content/migrations" || migrationRoot.runner !== null || migrationRoot.transactionOwner !== "future-reviewed-runner") fail("migration-root boundary drifted");
safeRelative(migrationRoot.path, "migration root");
const migrationRootPath = resolve(root, migrationRoot.path);
if (!existsSync(migrationRootPath) || lstatSync(migrationRootPath).isSymbolicLink() || !statSync(migrationRootPath).isDirectory()) fail("migration root must be a real directory");
if (!Array.isArray(migrationRoot.orderedMigrations) || migrationRoot.orderedMigrations.length !== 1) fail("exactly one ordered content migration is required");
exact("migration-root file inventory", ["20260722030000_create_content_tables.sql"], readdirSync(migrationRootPath).sort());
const migration = migrationRoot.orderedMigrations[0];
if (migration.version !== "20260722030000" || migration.path !== "services/content/migrations/20260722030000_create_content_tables.sql" || migration.bytes !== 1656 || migration.sha256 !== "b4eaf9ec57b1a823e0dad8a2a5fdb1b574488d6c7ebceb1187784cd505bba24d") fail("ordered migration pin drifted");
exact("migration guards", [
  "CREATE TABLE IF NOT EXISTS public.themes (",
  "CREATE TABLE IF NOT EXISTS public.pages (",
  "CREATE TABLE IF NOT EXISTS public.block_types (",
  "CREATE TABLE IF NOT EXISTS public.edit_sessions (",
], migration.guards);
if (!migration.path.split("/").at(-1).startsWith(`${migration.version}_`)) fail("migration filename/version order is inconsistent");
const migrationPath = regularRepoFile(migration.path, "content migration");
const migrationBytes = readFileSync(migrationPath);
const migrationSql = migrationBytes.toString("utf8");
if (migrationBytes.byteLength !== migration.bytes || sha256(migrationBytes) !== migration.sha256) fail("content migration bytes changed");
if ((migrationSql.match(/;/g) ?? []).length !== 4) fail("content migration must contain exactly four statements");
if ((migrationSql.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.(?:themes|pages|block_types|edit_sessions)\s*\(/gi) ?? []).length !== 4) fail("content migration must contain exactly four guarded public table creations");
for (const guard of migration.guards) if ((migrationSql.split(guard).length - 1) !== 1) fail(`migration guard must occur exactly once: ${guard}`);
if (/\b(?:DROP|TRUNCATE|DELETE\s+FROM|ALTER|INSERT|UPDATE|MERGE)\b/i.test(migrationSql)) fail("content migration contains a destructive, data-mutation, or alteration token");
if (/\bCREATE\s+(?:SCHEMA|EXTENSION|INDEX|DATABASE|TYPE|VIEW)\b/i.test(migrationSql)) fail("content migration contains an out-of-scope creation");
if (/\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(migrationSql)) fail("transaction control belongs to the future reviewed runner");

if (!Array.isArray(contract.requiredTables) || contract.requiredTables.length !== 4) fail("exactly four content tables are required");
const expectedTables = [
  ["themes", 8],
  ["pages", 11],
  ["block_types", 9],
  ["edit_sessions", 6],
];
let requiredColumnCount = 0;
for (let tableIndex = 0; tableIndex < contract.requiredTables.length; tableIndex += 1) {
  const table = contract.requiredTables[tableIndex];
  const [expectedName, expectedColumns] = expectedTables[tableIndex];
  if (table.name !== expectedName || !Array.isArray(table.columns) || table.columns.length !== expectedColumns) fail(`${expectedName}: required table contract drifted`);
  const tableMatch = migrationSql.match(new RegExp(`CREATE TABLE IF NOT EXISTS public\\.${expectedName} \\(([\\s\\S]*?)\\n\\);`));
  if (!tableMatch) fail(`${expectedName}: migration table body is malformed`);
  const expectedDefinitions = [];
  for (let index = 0; index < table.columns.length; index += 1) {
    const column = table.columns[index];
    if (column.ordinal !== index + 1 || typeof column.name !== "string" || typeof column.sqlAnchor !== "string") fail(`${expectedName}: column ${index + 1} drifted`);
    if ((tableMatch[1].split(column.sqlAnchor).length - 1) !== 1) fail(`${expectedName}.${column.name}: migration anchor must occur exactly once in its table`);
    expectedDefinitions.push(column.sqlAnchor.replace(/,$/, ""));
    requiredColumnCount += 1;
  }
  const actualDefinitions = tableMatch[1].split(/\r?\n/).map((line) => line.trim().replace(/,$/, "")).filter(Boolean);
  exact(`${expectedName} column definitions`, expectedDefinitions, actualDefinitions);
}
if (requiredColumnCount !== 34) fail(`required column count is ${requiredColumnCount}, expected 34`);

const drift = contract.freshSchemaDrift;
if (!drift || drift.guardedMigrationUpgradesPreexistingTables !== false || drift.driftItems !== 18) fail("fresh-schema drift boundary drifted");
exact("removed runtime snapshot", {
  commit: "c0339d663123cb26ecd682aeea28e9917cf05b7f",
  file: "services/content/src/main.rs",
  blob: "9d623fdd83780e3a13f18e439383e4fcba72b601",
}, drift.removedRuntimeSnapshot);
if (!gitExists("cat-file", "-e", `${drift.removedRuntimeSnapshot.commit}^{commit}`)) fail("removed runtime snapshot commit is unavailable");
const removedRuntimeBlob = git("rev-parse", `${drift.removedRuntimeSnapshot.commit}:${drift.removedRuntimeSnapshot.file}`).toString().trim();
if (removedRuntimeBlob !== drift.removedRuntimeSnapshot.blob) fail("removed runtime snapshot blob drifted");
const removedRuntimeSource = git("show", `${drift.removedRuntimeSnapshot.commit}:${drift.removedRuntimeSnapshot.file}`).toString();
const tableBody = (content, prefix, table, suffix) => {
  const startAnchor = `${prefix}${table} (`;
  const start = content.indexOf(startAnchor);
  if (start < 0) fail(`${table}: table start is missing from drift evidence`);
  const bodyStart = start + startAnchor.length;
  const end = content.indexOf(suffix, bodyStart);
  if (end < 0) fail(`${table}: table end is missing from drift evidence`);
  return content.slice(bodyStart, end);
};
const oldBodies = Object.fromEntries(expectedTables.map(([table]) => [table, tableBody(removedRuntimeSource, "CREATE TABLE IF NOT EXISTS ", table, "\n        )\"")]));
const newBodies = Object.fromEntries(expectedTables.map(([table]) => [table, tableBody(migrationSql, "CREATE TABLE IF NOT EXISTS public.", table, "\n);")]));
if (!Array.isArray(drift.notNullAdditions) || drift.notNullAdditions.length !== 17) fail("exactly seventeen intentional NOT NULL additions are required");
const expectedNotNullKeys = [
  "pages.locale", "pages.status", "pages.blocks_json", "pages.created_at", "pages.updated_at",
  "themes.colors_json", "themes.fonts_json", "themes.spacing_json", "themes.breakpoints_json", "themes.is_default",
  "block_types.schema_json", "block_types.default_props_json", "block_types.admin_only", "block_types.updated_at",
  "edit_sessions.page_id", "edit_sessions.status", "edit_sessions.started_at",
];
const actualNotNullKeys = [];
for (const item of drift.notNullAdditions) {
  if (!item || typeof item.table !== "string" || typeof item.column !== "string" || typeof item.oldSqlAnchor !== "string" || typeof item.newSqlAnchor !== "string") fail("NOT NULL drift item is malformed");
  const key = `${item.table}.${item.column}`;
  actualNotNullKeys.push(key);
  if (!oldBodies[item.table] || !newBodies[item.table]) fail(`${key}: drift table is not in scope`);
  if ((oldBodies[item.table].split(item.oldSqlAnchor).length - 1) !== 1) fail(`${key}: old nullable anchor must occur once`);
  if ((newBodies[item.table].split(item.newSqlAnchor).length - 1) !== 1) fail(`${key}: new NOT NULL anchor must occur once`);
  if (!/\bNOT\s+NULL\b/i.test(item.newSqlAnchor) || /\bNOT\s+NULL\b/i.test(item.oldSqlAnchor)) fail(`${key}: NOT NULL drift direction is invalid`);
}
exact("NOT NULL drift key inventory", expectedNotNullKeys, actualNotNullKeys);
if (new Set(actualNotNullKeys).size !== 17) fail("NOT NULL drift inventory contains duplicates");
if (!Array.isArray(drift.uniqueAdditions) || drift.uniqueAdditions.length !== 1) fail("exactly one intentional unique addition is required");
const uniqueDrift = drift.uniqueAdditions[0];
if (uniqueDrift.table !== "themes" || uniqueDrift.column !== "name" || uniqueDrift.oldSqlAnchor !== "name VARCHAR(100) NOT NULL," || uniqueDrift.newSqlAnchor !== "name VARCHAR(100) UNIQUE NOT NULL,") fail("themes.name unique drift item changed");
if ((oldBodies.themes.split(uniqueDrift.oldSqlAnchor).length - 1) !== 1 || (newBodies.themes.split(uniqueDrift.newSqlAnchor).length - 1) !== 1) fail("themes.name unique drift anchors are not exact");
exact("fresh-schema drift STOP categories", ["baseline-adoption", "populated-upgrade"], drift.stopCategories);

exact("constraint semantics", {
  primaryKeys: ["themes.id", "pages.id", "block_types.id", "edit_sessions.id"],
  uniqueKeys: ["themes.name", "pages.slug", "block_types.block_type"],
  foreignKeys: ["edit_sessions.page_id->public.pages.id ON UPDATE NO ACTION ON DELETE CASCADE"],
  backingIndexes: 7,
  nameOnlyIdempotenceAccepted: false,
  uniqueConstraintPolicy: {
    exactDistinctSet: true,
    oneConstraintPerKey: true,
    validated: true,
    deferrable: false,
    initiallyDeferred: false,
    backingIndexUnique: true,
    backingIndexValid: true,
    backingIndexReady: true,
    backingIndexImmediate: true,
    backingIndexPartial: false,
    backingIndexExpression: false,
  },
  foreignKeyBoundary: {
    inventoryDirection: "inbound-and-outbound",
    exactTotal: 1,
    unexpectedInboundAccepted: false,
    unexpectedOutboundAccepted: false,
  },
  uniqueIndexInventory: {
    catalogDriver: "pg_catalog.pg_index",
    inventoryPredicate: "indisunique",
    exactTotal: 7,
    constraintBackedOnly: true,
    standaloneAccepted: false,
    partialAccepted: false,
    expressionAccepted: false,
    accessMethod: "btree",
    collationPolicy: "exact-indexed-column-collation",
    bindings: [
      { table: "themes", column: "id", constraintType: "p", opclass: "uuid_ops" },
      { table: "themes", column: "name", constraintType: "u", opclass: "text_ops" },
      { table: "pages", column: "id", constraintType: "p", opclass: "uuid_ops" },
      { table: "pages", column: "slug", constraintType: "u", opclass: "text_ops" },
      { table: "block_types", column: "id", constraintType: "p", opclass: "uuid_ops" },
      { table: "block_types", column: "block_type", constraintType: "u", opclass: "text_ops" },
      { table: "edit_sessions", column: "id", constraintType: "p", opclass: "uuid_ops" },
    ],
  },
}, contract.constraintSemantics);
if ((migrationSql.match(/\bPRIMARY\s+KEY\b/gi) ?? []).length !== 4) fail("migration must define exactly four primary keys");
if ((migrationSql.match(/\bUNIQUE\b/gi) ?? []).length !== 3) fail("migration must define exactly three unique keys");
if ((migrationSql.match(/\bREFERENCES\s+public\.pages\s*\(id\)\s+ON\s+DELETE\s+CASCADE\b/gi) ?? []).length !== 1) fail("migration must preserve the exact edit-session page foreign key action");
if (/\bCREATE\s+(?:UNIQUE\s+)?INDEX\b/i.test(migrationSql)) fail("name-only index idempotence is not accepted in this migration");

exact("legacy cascade finding", {
  anchor: "page_id UUID NOT NULL REFERENCES public.pages(id) ON DELETE CASCADE,",
  kind: "CASCADE",
  count: 1,
  classification: "reviewed-lexical-safety-stop",
  rationale: "The target migration preserves the existing edit-session child cleanup action; the read-only probe validates the exact FK semantics instead of trusting an object name.",
  executionClaimed: false,
}, contract.legacyCascadeFinding);
if ((migrationSql.match(/\bCASCADE\b/gi) ?? []).length !== 1 || (migrationSql.split(contract.legacyCascadeFinding.anchor).length - 1) !== 1) fail("legacy CASCADE finding must remain exact and singular");

exact("isolated scanner delta", {
  runtimeRustDdlFindings: { before: 35, after: 31, delta: -4 },
  actionableFindings: { before: 29, after: 25, delta: -4 },
  contentFindings: { before: 4, after: 0, delta: -4 },
  reviewedRuntimeExceptions: { before: 6, after: 6, delta: 0 },
  migrationSqlFiles: { before: 169, projectedAfter: 170, delta: 1 },
  destructiveLexicalFindings: { before: 510, projectedAfter: 511, delta: 1 },
  projectedMigrationSqlSha256: "cda0fbb7411db38cc02a4c4d7ec97d26b15aaff5a5faa9281ff96e3e763e9132",
  status: "isolated-authoring-snapshot-canonical-rebaseline-owned-elsewhere",
}, contract.isolatedScannerDelta);
if (!Array.isArray(contract.nonClaims) || contract.nonClaims.length !== 5 || contract.nonClaims.some((item) => typeof item !== "string" || item.length < 60)) fail("five substantive non-claims are required");
const blockerCategories = ["migration-runner", "baseline-adoption", "populated-upgrade", "reconciliation", "concurrent-startup", "live-database"];
if (!Array.isArray(contract.blockers) || contract.blockers.length !== 6) fail("exactly six residual blockers are required");
for (let index = 0; index < contract.blockers.length; index += 1) {
  const blocker = contract.blockers[index];
  const id = `B${String(index + 1).padStart(2, "0")}`;
  if (blocker.id !== id || blocker.category !== blockerCategories[index] || blocker.status !== "blocked" || typeof blocker.summary !== "string" || blocker.summary.length < 40) fail(`${id}: residual blocker drifted`);
}
for (const category of drift.stopCategories) {
  if (!contract.blockers.some((blocker) => blocker.category === category && blocker.status === "blocked")) fail(`${category}: fresh-schema drift STOP is not blocked`);
}

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  service: contract.scope.service,
  status: contract.scope.status,
  developmentNews: { sourceCommit: source.sourceCommit, evidence: source.evidence.length, imported: false },
  runtimeRust: {
    files: rustFiles.length,
    ddlFindings: runtimeFindings.length,
    expectedDelta: -4,
    qualifiedRelations: relationCounts,
    jsonbTextProjections: jsonbTextProjectionCount,
    returningStars: returningStars.length,
  },
  migrationRoot: {
    path: migrationRoot.path,
    migrations: 1,
    pinnedBytes: migration.bytes,
    sha256: migration.sha256,
    runner: null,
    guardedTables: migration.guards.length,
    lexicalCascadeFindings: 1,
  },
  schema: {
    tables: contract.requiredTables.map((table) => ({ name: table.name, columns: table.columns.length })),
    columns: requiredColumnCount,
    primaryKeys: 4,
    uniqueKeys: 3,
    foreignKeys: 1,
    backingIndexes: 7,
    inventoriedUniqueIndexes: contract.constraintSemantics.uniqueIndexInventory.exactTotal,
    standaloneUniqueIndexesAccepted: contract.constraintSemantics.uniqueIndexInventory.standaloneAccepted,
    partialUniqueIndexesAccepted: contract.constraintSemantics.uniqueIndexInventory.partialAccepted,
    expressionUniqueIndexesAccepted: contract.constraintSemantics.uniqueIndexInventory.expressionAccepted,
    freshSchemaDriftItems: drift.driftItems,
    notNullAdditions: drift.notNullAdditions.length,
    uniqueAdditions: drift.uniqueAdditions.length,
  },
  blockers: contract.blockers.map(({ id, category, status }) => ({ id, category, status })),
  productionReady: false,
  readinessExit: 3,
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s\n' "$report"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "a3-10-content-schema-boundary: PASS — content runtime DDL 4→0, four guarded public tables and 34 exact columns pinned"
  echo "a3-10-content-schema-boundary: PASS — three exact immediate unique keys and the complete inbound/outbound FK boundary are pinned"
  echo "a3-10-content-schema-boundary: PASS — all seven pg_index unique rows are constraint-bound; standalone, partial, and expression unique indexes fail closed"
  echo "a3-10-content-schema-boundary: PASS — JSONB/UUID/timestamptz models audited; 19 runtime relations are public-qualified"
  echo "a3-10-content-schema-boundary: LIMIT — guarded fresh-schema DDL cannot apply 17 NOT NULL and one UNIQUE drift item to pre-existing tables"
  echo "a3-10-content-schema-boundary: LIMIT — one preserved ON DELETE CASCADE lexical finding remains explicit; no migration or database ran"
  echo "a3-10-content-schema-boundary: LIMIT — no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran"
  exit 0
fi

echo "a3-10-content-schema-boundary: STOP — six residual A3.10 blockers remain; readiness is intentionally exit 3" >&2
echo "a3-10-content-schema-boundary: LIMIT — static integrity is not migration, reconciliation, or database execution evidence" >&2
exit 3
