#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-6-analytics-schema-boundary.json"
mode=""

die() {
  echo "a3-6-analytics-schema-boundary: ERROR: $*" >&2
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

for name in DATABASE_URL ANALYTICS_DATABASE_URL PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD; do
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
  console.error(`a3-6-analytics-schema-boundary: ERROR: ${message}`);
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

let contract;
try {
  if (lstatSync(contractInput).isSymbolicLink()) fail("contract must be a regular non-symbolic-link file");
  const contractPath = realpathSync(contractInput);
  if (!statSync(contractPath).isFile()) fail("contract must be a regular non-symbolic-link file");
  contract = JSON.parse(readFileSync(contractPath, "utf8"));
} catch (error) {
  fail(`invalid contract JSON: ${error.message}`);
}

if (contract.schemaVersion !== 1 || contract.contractId !== "A3.6-analytics-schema-boundary") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "offline-static-analytics-schema-boundary-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", {
  service: "services/analytics",
  database: "epsx_analytics",
  schema: "public",
  table: "events",
  status: "partial",
}, contract.scope);
if (!contract.safety || typeof contract.safety.readinessMeaning !== "string") fail("safety boundary is required");
for (const [key, value] of Object.entries(contract.safety)) {
  if (key !== "readinessMeaning" && value !== false) fail(`${key} must remain false`);
}

const runtime = contract.runtimeBoundary;
if (!runtime || runtime.rustRoot !== "services/analytics" || runtime.scannerFindingBefore !== 1 || runtime.scannerFindingAfter !== 0) fail("runtime scanner boundary drifted");
if (runtime.removedAnchor !== "CREATE TABLE IF NOT EXISTS events (" || runtime.compatibilityQueryConstant !== "ANALYTICS_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility" || runtime.compatibilityQueryBytes !== 2824 || runtime.compatibilityQuerySha256 !== "3d8a007ad47b8c57cf3f2b45c8d1e5bcadf9ceebfeded89e69c0127725978739" || runtime.qualifiedEventsRelation !== "public.events" || runtime.qualifiedEventsSqlOccurrences !== 5) fail("runtime boundary anchors or query pin drifted");

const rustRootPath = resolve(root, runtime.rustRoot);
if (!existsSync(rustRootPath) || !statSync(rustRootPath).isDirectory()) fail("analytics Rust root is missing");
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
if (runtimeFindings.length !== 0) fail(`analytics runtime Rust DDL scanner found ${runtimeFindings.length}, expected zero`);

const libPath = regularRepoFile("services/analytics/src/lib.rs", "analytics library");
const mainPath = regularRepoFile("services/analytics/src/main.rs", "analytics main");
const lib = readFileSync(libPath, "utf8");
const main = readFileSync(mainPath, "utf8");
if (lib.includes(runtime.removedAnchor)) fail("removed runtime schema-mutation anchor returned");
const queryMatch = lib.match(/const ANALYTICS_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!queryMatch) fail("read-only compatibility query constant is missing");
const query = queryMatch[1];
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (!/^\s*WITH\s+/i.test(query)) fail("compatibility query must start with a read-only CTE");
const queryMutation = /\b(?:INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i;
if (queryMutation.test(query)) fail("compatibility query contains a mutation or command token");
if (!Array.isArray(runtime.queryRequiredAnchors) || runtime.queryRequiredAnchors.length !== 6) fail("exactly six compatibility-query anchors are required");
for (const anchor of runtime.queryRequiredAnchors) if (typeof anchor !== "string" || !query.includes(anchor)) fail(`missing compatibility-query anchor: ${anchor}`);
if (!/AND\s+COALESCE\(\s*CASE expected\.default_kind[\s\S]*?END,\s*false\s*\)/.test(query)) fail("required default comparisons must coalesce NULL to false");

const unqualifiedEventsSql = lib.match(/\b(?:INSERT\s+INTO|DELETE\s+FROM|UPDATE|FROM|JOIN)\s+events\b/gi) ?? [];
if (unqualifiedEventsSql.length !== 0) fail(`analytics runtime SQL contains ${unqualifiedEventsSql.length} unqualified events relation reference(s)`);
const qualifiedEventsSql = lib.match(/\b(?:INSERT\s+INTO|DELETE\s+FROM|UPDATE|FROM|JOIN)\s+public\.events\b/gi) ?? [];
if (qualifiedEventsSql.length !== runtime.qualifiedEventsSqlOccurrences) fail(`analytics runtime SQL has ${qualifiedEventsSql.length} public.events relation references, expected ${runtime.qualifiedEventsSqlOccurrences}`);

const functionStart = lib.indexOf("pub async fn verify_schema_compatibility(");
const functionEnd = lib.indexOf("#[async_trait]", functionStart);
if (functionStart < 0 || functionEnd < 0) fail("compatibility function boundary is missing");
const functionBody = lib.slice(functionStart, functionEnd);
for (const anchor of [
  "sqlx::query_scalar::<_, bool>(ANALYTICS_SCHEMA_COMPATIBILITY_QUERY)",
  ".fetch_one(db)",
  "AnalyticsSchemaError::Incompatible",
]) if (!functionBody.includes(anchor)) fail(`compatibility function is missing ${anchor}`);
if (functionBody.includes(".execute(")) fail("compatibility function must not execute a mutation statement");

if (!Array.isArray(runtime.mainSequence) || runtime.mainSequence.length !== 4) fail("main sequence must contain exactly four anchors");
let previous = -1;
for (const anchor of runtime.mainSequence) {
  const index = main.indexOf(anchor);
  if (index < 0) fail(`main sequence is missing: ${anchor}`);
  if (index <= previous) fail(`main sequence is out of order: ${anchor}`);
  previous = index;
}

const migrationRoot = contract.migrationRoot;
if (!migrationRoot || migrationRoot.path !== "services/analytics/migrations" || migrationRoot.runner !== null || migrationRoot.transactionOwner !== "future-reviewed-runner") fail("migration-root boundary drifted");
safeRelative(migrationRoot.path, "migration root");
const migrationRootPath = resolve(root, migrationRoot.path);
if (!existsSync(migrationRootPath) || lstatSync(migrationRootPath).isSymbolicLink() || !statSync(migrationRootPath).isDirectory()) fail("migration root must be a real directory");
if (!Array.isArray(migrationRoot.orderedMigrations) || migrationRoot.orderedMigrations.length !== 2) fail("exactly two ordered analytics migrations are required");
const rootEntries = readdirSync(migrationRootPath).sort();
exact("migration-root file inventory", ["20260722000000_create_events.sql", "20260727010000_add_event_subject.sql"], rootEntries);
const migration = migrationRoot.orderedMigrations[0];
if (migration.version !== "20260722000000" || migration.path !== "services/analytics/migrations/20260722000000_create_events.sql" || migration.bytes !== 260 || migration.sha256 !== "03a154e1d8761e412face94c4cd848616e9e2c8ca43d8d5ffb44c52701c2e7dd" || migration.guard !== "CREATE TABLE IF NOT EXISTS public.events (") fail("ordered migration pin drifted");
if (!migration.path.split("/").at(-1).startsWith(`${migration.version}_`)) fail("migration filename/version order is inconsistent");
const migrationPath = regularRepoFile(migration.path, "analytics migration");
const migrationBytes = readFileSync(migrationPath);
const migrationSql = migrationBytes.toString("utf8");
if (migrationBytes.byteLength !== migration.bytes || sha256(migrationBytes) !== migration.sha256) fail("analytics migration bytes changed");
if ((migrationSql.match(/;/g) ?? []).length !== 1) fail("analytics migration must contain exactly one statement");
if ((migrationSql.match(/\bCREATE\b/gi) ?? []).length !== 1 || !/^\s*CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.events\s*\(/i.test(migrationSql)) fail("analytics migration must contain exactly one guarded table creation");
if (/\b(?:DROP|TRUNCATE|DELETE|ALTER|INSERT|UPDATE|MERGE|CASCADE)\b/i.test(migrationSql)) fail("analytics migration contains a destructive, data-mutation, or alteration token");
if (/\bCREATE\s+(?:SCHEMA|EXTENSION|INDEX|DATABASE|TYPE|VIEW)\b/i.test(migrationSql)) fail("analytics migration contains an out-of-scope creation");
if (/\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(migrationSql)) fail("transaction control belongs to the future reviewed runner");

const subjectMigration = migrationRoot.orderedMigrations[1];
if (subjectMigration.version !== "20260727010000" || subjectMigration.path !== "services/analytics/migrations/20260727010000_add_event_subject.sql" || subjectMigration.bytes !== 415 || subjectMigration.sha256 !== "df2e7da938b0b80f42bddc71c5f5c22f27fb41af01bce08823c1c7e4e510c812") fail("ordered subject migration pin drifted");
const subjectBytes = readFileSync(regularRepoFile(subjectMigration.path, "analytics subject migration"));
const subjectSql = subjectBytes.toString("utf8");
if (subjectBytes.byteLength !== subjectMigration.bytes || sha256(subjectBytes) !== subjectMigration.sha256) fail("analytics subject migration bytes changed");
if ((subjectSql.match(/;/g) ?? []).length !== 3 || !/ALTER\s+TABLE\s+public\.events\s+ADD\s+COLUMN\s+IF\s+NOT\s+EXISTS\s+subject\s+VARCHAR\(128\)/i.test(subjectSql) || !/CREATE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+events_subject_created_at_idx/i.test(subjectSql)) fail("analytics subject migration is not the reviewed additive shape");
if (/\b(?:DROP|TRUNCATE|DELETE|INSERT|UPDATE|MERGE|CASCADE|CREATE\s+(?:SCHEMA|EXTENSION|DATABASE|TYPE|VIEW))\b/i.test(subjectSql) || /\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(subjectSql)) fail("analytics subject migration contains a destructive token or transaction control");

if (!Array.isArray(contract.requiredColumns) || contract.requiredColumns.length !== 6) fail("exactly six legacy columns are required");
const expectedColumnNames = ["id", "user_id", "event_name", "properties_json", "chain_id", "created_at"];
const expectedDefinitions = [];
for (let index = 0; index < contract.requiredColumns.length; index += 1) {
  const column = contract.requiredColumns[index];
  if (column.ordinal !== index + 1 || column.name !== expectedColumnNames[index] || typeof column.sqlAnchor !== "string") fail(`required column ${index + 1} drifted`);
  if ((migrationSql.split(column.sqlAnchor).length - 1) !== 1) fail(`${column.name}: migration anchor must occur exactly once`);
  expectedDefinitions.push(column.sqlAnchor.replace(/,$/, ""));
}
const tableMatch = migrationSql.match(/^\s*CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.events\s*\(([\s\S]*)\);\s*$/i);
if (!tableMatch) fail("analytics migration table body is malformed");
const actualDefinitions = tableMatch[1].split(/\r?\n/).map((line) => line.trim().replace(/,$/, "")).filter(Boolean);
exact("events column definitions", expectedDefinitions, actualDefinitions);

exact("isolated scanner delta", {
  runtimeRustDdlFindings: { before: 39, after: 38, delta: -1 },
  actionableFindings: { before: 33, after: 32, delta: -1 },
  analyticsFindings: { before: 1, after: 0, delta: -1 },
  reviewedExceptions: { before: 6, after: 6, delta: 0 },
  status: "isolated-projection-only-canonical-rebaseline-owned-elsewhere",
}, contract.isolatedScannerDelta);
if (!Array.isArray(contract.nonClaims) || contract.nonClaims.length !== 4 || contract.nonClaims.some((item) => typeof item !== "string" || item.length < 40)) fail("four substantive non-claims are required");
const blockerCategories = ["migration-runner", "baseline-adoption", "populated-upgrade", "reconciliation", "concurrent-startup", "live-database"];
if (!Array.isArray(contract.blockers) || contract.blockers.length !== 6) fail("exactly six residual blockers are required");
for (let index = 0; index < contract.blockers.length; index += 1) {
  const blocker = contract.blockers[index];
  const id = `B${String(index + 1).padStart(2, "0")}`;
  if (blocker.id !== id || blocker.category !== blockerCategories[index] || blocker.status !== "blocked" || typeof blocker.summary !== "string" || blocker.summary.length < 40) fail(`${id}: residual blocker drifted`);
}

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  service: contract.scope.service,
  status: contract.scope.status,
  runtimeRust: {
    files: rustFiles.length,
    ddlFindings: runtimeFindings.length,
    expectedDelta: -1,
    qualifiedEventsRelation: runtime.qualifiedEventsRelation,
    qualifiedEventsSqlOccurrences: qualifiedEventsSql.length,
  },
  migrationRoot: { path: migrationRoot.path, migrations: 2, pinnedBytes: migration.bytes + subjectMigration.bytes, sha256: subjectMigration.sha256, runner: null },
  requiredColumns: expectedColumnNames,
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
  echo "a3-6-analytics-schema-boundary: PASS — analytics runtime DDL 1→0, baseline plus additive subject migration pinned, seven-column compatibility boundary verified"
  echo "a3-6-analytics-schema-boundary: LIMIT — no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran"
  exit 0
fi

echo "a3-6-analytics-schema-boundary: STOP — six residual A3.6 blockers remain; readiness is intentionally exit 3" >&2
echo "a3-6-analytics-schema-boundary: LIMIT — static integrity is not migration or database execution evidence" >&2
exit 3
