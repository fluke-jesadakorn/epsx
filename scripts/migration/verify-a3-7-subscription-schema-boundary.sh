#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-7-subscription-schema-boundary.json"
mode=""

die() {
  echo "a3-7-subscription-schema-boundary: ERROR: $*" >&2
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

for name in DATABASE_URL SUBSCRIPTION_DATABASE_URL PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD; do
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
  console.error(`a3-7-subscription-schema-boundary: ERROR: ${message}`);
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

if (contract.schemaVersion !== 1 || contract.contractId !== "A3.7-subscription-schema-boundary") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "offline-static-subscription-schema-boundary-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", {
  service: "services/subscription",
  database: "epsx_subscription",
  schema: "public",
  tables: ["subscription_plans", "subscriptions"],
  status: "partial",
}, contract.scope);
if (!contract.safety || typeof contract.safety.readinessMeaning !== "string") fail("safety boundary is required");
for (const [key, value] of Object.entries(contract.safety)) {
  if (key !== "readinessMeaning" && value !== false) fail(`${key} must remain false`);
}

const baseline = contract.developmentBaseline;
if (!baseline || baseline.sourceRef !== "origin/development" || baseline.sourceCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || baseline.candidateServicePath !== "services/subscription" || baseline.candidateServicePresent !== false || baseline.mappingStatus !== "blocked") fail("development baseline boundary drifted");
if (git("rev-parse", baseline.sourceRef).toString().trim() !== baseline.sourceCommit) fail("origin/development no longer resolves to the pinned source commit");
if (gitExists("cat-file", "-e", `${baseline.sourceCommit}:${baseline.candidateServicePath}`)) fail("candidate subscription service unexpectedly exists in the development baseline");
if (!Array.isArray(baseline.evidence) || baseline.evidence.length !== 2) fail("exactly two development baseline evidence files are required");
const expectedBaseline = [
  {
    path: "apps/backend/migrations/payments/00000000000001_consolidated_baseline_v4/up.sql",
    bytesSha256: "832ff33f0138e67bf9c1e6a0a58a5ae99d61493913b80785cf71f177b8bf23fa",
    anchors: ["CREATE TABLE subscriptions (", "wallet_address VARCHAR(42) NOT NULL", "expires_at TIMESTAMPTZ NOT NULL"],
  },
  {
    path: "apps/backend/src/web/payments/subscription_handlers.rs",
    bytesSha256: "81c3e8ea1622703ebe81cd4733b17d9f0c77d8577af442fd9843ba7f16557ad1",
    anchors: ["Uses wallet_plan_assignments table for all plan access data", "WHERE LOWER(wga.wallet_address) = LOWER($1)", "FREE_PLAN_RANKING_OFFSET"],
  },
];
exact("development baseline evidence", expectedBaseline, baseline.evidence);
for (const evidence of baseline.evidence) {
  safeRelative(evidence.path, "development evidence path");
  const bytes = git("show", `${baseline.sourceCommit}:${evidence.path}`);
  if (sha256(bytes) !== evidence.bytesSha256) fail(`development evidence bytes changed: ${evidence.path}`);
  const content = bytes.toString();
  for (const anchor of evidence.anchors) if (!content.includes(anchor)) fail(`development evidence anchor is missing: ${evidence.path}: ${anchor}`);
}

const runtime = contract.runtimeBoundary;
if (!runtime || runtime.rustRoot !== "services/subscription" || runtime.scannerFindingBefore !== 2 || runtime.scannerFindingAfter !== 0) fail("runtime scanner boundary drifted");
exact("Rust inventory", ["services/subscription/src/admin.rs", "services/subscription/src/lib.rs", "services/subscription/src/main.rs"], runtime.rustInventory);
exact("removed runtime anchors", ["CREATE TABLE IF NOT EXISTS subscription_plans (", "CREATE TABLE IF NOT EXISTS subscriptions ("], runtime.removedAnchors);
if (runtime.compatibilityQueryConstant !== "SUBSCRIPTION_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility" || runtime.compatibilityQueryBytes !== 7950 || runtime.compatibilityQuerySha256 !== "5235b7509d102d0ca68f508ee1f48041c78846878c557a25af82846e07ae6a9d") fail("compatibility query boundary or pin drifted");
exact("qualified relation counts", { "public.subscription_plans": 3, "public.subscriptions": 4 }, runtime.qualifiedRelationOccurrences);

const rustRoot = resolve(root, runtime.rustRoot);
if (!existsSync(rustRoot) || lstatSync(rustRoot).isSymbolicLink() || !statSync(rustRoot).isDirectory()) fail("subscription Rust root must be a real directory");
const rustFiles = [];
const visit = (directory) => {
  for (const entry of readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const path = resolve(directory, entry.name);
    if (entry.isSymbolicLink()) fail(`symbolic links are not allowed under ${runtime.rustRoot}`);
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
    if (match) runtimeFindings.push({ file: relative, line: index + 1, kind: match[0].trim().replace(/\s+/g, " ").toUpperCase() });
  });
}
if (runtimeFindings.length !== 0) fail(`subscription runtime Rust DDL scanner found ${runtimeFindings.length}, expected zero`);

const lib = readFileSync(regularRepoFile("services/subscription/src/lib.rs", "subscription library"), "utf8");
const main = readFileSync(regularRepoFile("services/subscription/src/main.rs", "subscription main"), "utf8");
for (const anchor of runtime.removedAnchors) if (lib.includes(anchor) || main.includes(anchor)) fail(`removed runtime schema-mutation anchor returned: ${anchor}`);
const queryStartAnchor = `const ${runtime.compatibilityQueryConstant}: &str = r#"`;
const queryStart = lib.indexOf(queryStartAnchor);
const queryEnd = lib.indexOf("\"#;", queryStart + queryStartAnchor.length);
if (queryStart < 0 || queryEnd < 0) fail("read-only compatibility query constant is missing");
const query = lib.slice(queryStart + queryStartAnchor.length, queryEnd);
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (!/^\s*WITH\s+/i.test(query)) fail("compatibility query must start with a read-only CTE");
if (/\b(?:INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i.test(query)) fail("compatibility query contains a mutation or command token");
if (!Array.isArray(runtime.queryRequiredAnchors) || runtime.queryRequiredAnchors.length !== 12) fail("exactly twelve compatibility-query anchors are required");
for (const anchor of runtime.queryRequiredAnchors) if (typeof anchor !== "string" || !query.includes(anchor)) fail(`missing compatibility-query anchor: ${anchor}`);
if (!/AND\s+COALESCE\(\s*CASE expected\.default_kind[\s\S]*?END,\s*false\s*\)/.test(query)) fail("required default comparisons must coalesce NULL to false");

for (const [relation, expected] of Object.entries(runtime.qualifiedRelationOccurrences)) {
  const bare = relation.split(".")[1];
  const escapedBare = bare.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const escapedRelation = relation.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const command = "(?:INSERT\\s+INTO|DELETE\\s+FROM|UPDATE|FROM|JOIN)";
  const unqualified = main.match(new RegExp(`\\b${command}\\s+${escapedBare}\\b`, "gi")) ?? [];
  if (unqualified.length !== 0) fail(`runtime SQL contains ${unqualified.length} unqualified ${bare} relation reference(s)`);
  const qualified = main.match(new RegExp(`\\b${command}\\s+${escapedRelation}\\b`, "gi")) ?? [];
  if (qualified.length !== expected) fail(`runtime SQL has ${qualified.length} ${relation} references, expected ${expected}`);
}

const functionStart = lib.indexOf(`pub async fn ${runtime.compatibilityFunction}(`);
const functionEnd = lib.indexOf("pub enum SubscriptionConfigError", functionStart);
if (functionStart < 0 || functionEnd < 0) fail("compatibility function boundary is missing");
const functionBody = lib.slice(functionStart, functionEnd);
for (const anchor of [
  "sqlx::query_scalar::<_, bool>(SUBSCRIPTION_SCHEMA_COMPATIBILITY_QUERY)",
  ".fetch_one(db)",
  "SubscriptionSchemaError::Incompatible",
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

const modelBoundary = contract.modelBoundary;
const expectedResponseModels = [
  {
    name: "SubscriptionPlan",
    fields: [
      { name: "id", rustType: "Uuid", databaseType: "uuid", nullable: false },
      { name: "merchant_id", rustType: "Uuid", databaseType: "uuid", nullable: false },
      { name: "name", rustType: "String", databaseType: "varchar(100)", nullable: false },
      { name: "description", rustType: "Option<String>", databaseType: "text", nullable: true },
      { name: "amount", rustType: "String", databaseType: "varchar(78)", nullable: false },
      { name: "currency", rustType: "String", databaseType: "varchar(10)", nullable: false },
      { name: "chain_id", rustType: "String", databaseType: "varchar(10)", nullable: false },
      { name: "interval", rustType: "i32", databaseType: "integer", nullable: false },
      { name: "active", rustType: "Option<bool>", databaseType: "boolean", nullable: true },
      { name: "created_at", rustType: "Option<chrono::DateTime<chrono::Utc>>", databaseType: "timestamptz", nullable: true },
    ],
  },
  {
    name: "Subscription",
    fields: [
      { name: "id", rustType: "Uuid", databaseType: "uuid", nullable: false },
      { name: "user_id", rustType: "Uuid", databaseType: "uuid", nullable: false },
      { name: "plan_id", rustType: "Option<Uuid>", databaseType: "uuid", nullable: true },
      { name: "status", rustType: "Option<String>", databaseType: "varchar(20)", nullable: true },
      { name: "account_id", rustType: "Option<String>", databaseType: "varchar(42)", nullable: true },
      { name: "payment_token", rustType: "Option<String>", databaseType: "varchar(42)", nullable: true },
      { name: "vault_position_id", rustType: "Option<String>", databaseType: "varchar(100)", nullable: true },
      { name: "current_period_start", rustType: "Option<chrono::DateTime<chrono::Utc>>", databaseType: "timestamptz", nullable: true },
      { name: "current_period_end", rustType: "Option<chrono::DateTime<chrono::Utc>>", databaseType: "timestamptz", nullable: true },
      { name: "created_at", rustType: "Option<chrono::DateTime<chrono::Utc>>", databaseType: "timestamptz", nullable: true },
    ],
  },
];
const expectedRequestModels = [
  {
    name: "CreatePlanRequest",
    fields: [
      { name: "merchant_id", rustType: "Uuid" },
      { name: "name", rustType: "String" },
      { name: "description", rustType: "Option<String>" },
      { name: "amount", rustType: "String" },
      { name: "currency", rustType: "String" },
      { name: "chain_id", rustType: "String" },
      { name: "interval", rustType: "i32" },
    ],
  },
  {
    name: "CreateSubscriptionRequest",
    fields: [
      { name: "user_id", rustType: "Uuid" },
      { name: "plan_id", rustType: "Uuid" },
      { name: "account_id", rustType: "Option<String>" },
      { name: "payment_token", rustType: "Option<String>" },
    ],
  },
];
if (!modelBoundary || modelBoundary.source !== "services/subscription/src/main.rs" || modelBoundary.sliceStart !== "#[derive(Serialize, FromRow)]\nstruct SubscriptionPlan" || modelBoundary.sliceEnd !== "\n\n#[tokio::main]" || modelBoundary.bytes !== 1265 || modelBoundary.sha256 !== "c6d87859984b684de8d30619d4a4b49a4332f65605a49aee9fa04e351c00fcb7") fail("model source boundary or pin drifted");
exact("response model/schema mapping", expectedResponseModels, modelBoundary.responseModels);
exact("request model mapping", expectedRequestModels, modelBoundary.requestModels);
exact("required plan input mapping", {
  column: "public.subscriptions.plan_id",
  responseType: "Option<Uuid>",
  createRequestType: "Uuid",
  status: "explicit-required-create-input",
  rationaleAnchor: "New subscriptions require an explicit target even though legacy rows",
}, modelBoundary.requiredPlanInputMapping);
exact("query model occurrences", { SubscriptionPlan: 3, Subscription: 4 }, modelBoundary.queryAsOccurrences);
exact("SQL operation occurrences", {
  planInsertReturning: 1,
  planSelect: 2,
  subscriptionInsertReturning: 1,
  subscriptionSelect: 2,
  subscriptionUpdateReturning: 1,
}, modelBoundary.sqlOperationOccurrences);
if (modelBoundary.pathUuidExtractorAnchor !== "axum::extract::Path(id): axum::extract::Path<Uuid>" || modelBoundary.pathUuidExtractorOccurrences !== 3 || modelBoundary.idBindOccurrences !== 3 || modelBoundary.responseSchemaFieldsCovered !== 20 || modelBoundary.nullableResponseFieldsCovered !== 11) fail("model coverage counts drifted");
exact("request bind fields", ["merchant_id", "name", "description", "amount", "currency", "chain_id", "interval", "user_id", "plan_id", "account_id", "payment_token"], modelBoundary.requestBindFields);

const modelStart = main.indexOf(modelBoundary.sliceStart);
const modelEnd = main.indexOf(modelBoundary.sliceEnd, modelStart);
if (modelStart < 0 || modelEnd < 0) fail("model source slice anchors are missing");
const modelSlice = main.slice(modelStart, modelEnd);
if (Buffer.byteLength(modelSlice) !== modelBoundary.bytes || sha256(modelSlice) !== modelBoundary.sha256) fail("model source slice bytes changed");
if (!main.includes("use uuid::Uuid;")) fail("subscription main must import uuid::Uuid");
if (!main.includes(modelBoundary.requiredPlanInputMapping.rationaleAnchor)) fail("required create-plan input rationale is missing");

const parseStructFields = (name) => {
  const match = main.match(new RegExp(`struct\\s+${name}\\s*\\{([\\s\\S]*?)\\n\\}`, "m"));
  if (!match) fail(`model struct is missing: ${name}`);
  return match[1].split(/\r?\n/).map((line) => line.trim()).map((line) => line.match(/^([a-z_][a-z0-9_]*):\s*(.+),$/)).filter(Boolean).map((field) => ({ name: field[1], rustType: field[2] }));
};
for (const model of [...expectedResponseModels, ...expectedRequestModels]) {
  exact(`${model.name} Rust fields`, model.fields.map(({ name, rustType }) => ({ name, rustType })), parseStructFields(model.name));
}
if (!main.includes("#[derive(Serialize, FromRow)]\nstruct SubscriptionPlan") || !main.includes("#[derive(Serialize, FromRow)]\nstruct Subscription")) fail("response models must retain Serialize and FromRow derives");
if (!main.includes("#[derive(Deserialize)]\nstruct CreatePlanRequest") || !main.includes("#[derive(Deserialize)]\nstruct CreateSubscriptionRequest")) fail("request models must retain Deserialize derives");

const responseFieldCount = expectedResponseModels.reduce((count, model) => count + model.fields.length, 0);
const nullableFieldCount = expectedResponseModels.flatMap((model) => model.fields).filter((field) => field.nullable).length;
if (responseFieldCount !== modelBoundary.responseSchemaFieldsCovered || nullableFieldCount !== modelBoundary.nullableResponseFieldsCovered) fail("response model schema/nullability coverage is incomplete");

const countMatches = (pattern) => (main.match(pattern) ?? []).length;
if (countMatches(/sqlx::query_as::<_, SubscriptionPlan>/g) !== modelBoundary.queryAsOccurrences.SubscriptionPlan || countMatches(/sqlx::query_as::<_, Subscription>/g) !== modelBoundary.queryAsOccurrences.Subscription) fail("query_as response model occurrence count drifted");
const observedSqlOperations = {
  planInsertReturning: countMatches(/INSERT INTO public\.subscription_plans[^"\n]*RETURNING \*/g),
  planSelect: countMatches(/SELECT \* FROM public\.subscription_plans/g),
  subscriptionInsertReturning: countMatches(/INSERT INTO public\.subscriptions[^"\n]*RETURNING \*/g),
  subscriptionSelect: countMatches(/SELECT \* FROM public\.subscriptions/g),
  subscriptionUpdateReturning: countMatches(/UPDATE public\.subscriptions[^"\n]*RETURNING \*/g),
};
exact("observed SQL operation inventory", modelBoundary.sqlOperationOccurrences, observedSqlOperations);
if (main.split(modelBoundary.pathUuidExtractorAnchor).length - 1 !== modelBoundary.pathUuidExtractorOccurrences) fail("UUID path extractor occurrence count drifted");
if ((main.match(/\.bind\(&id\)/g) ?? []).length !== modelBoundary.idBindOccurrences) fail("UUID path bind occurrence count drifted");
for (const field of modelBoundary.requestBindFields) {
  const anchor = `.bind(&req.${field})`;
  if (main.split(anchor).length - 1 !== 1) fail(`request bind must occur exactly once: ${field}`);
}

const migrationRoot = contract.migrationRoot;
if (!migrationRoot || migrationRoot.path !== "services/subscription/migrations" || migrationRoot.runner !== null || migrationRoot.transactionOwner !== "future-reviewed-runner") fail("migration-root boundary drifted");
safeRelative(migrationRoot.path, "migration root");
const migrationRootPath = resolve(root, migrationRoot.path);
if (!existsSync(migrationRootPath) || lstatSync(migrationRootPath).isSymbolicLink() || !statSync(migrationRootPath).isDirectory()) fail("migration root must be a real directory");
if (!Array.isArray(migrationRoot.orderedMigrations) || migrationRoot.orderedMigrations.length !== 3) fail("exactly three ordered subscription migrations are required");
exact("migration-root file inventory", ["20260722010000_create_subscription_tables.sql", "20260727000000_create_admin_access_operations.sql", "20260727010000_create_plan_state.sql"], readdirSync(migrationRootPath).sort());
const migration = migrationRoot.orderedMigrations[0];
if (migration.version !== "20260722010000" || migration.path !== "services/subscription/migrations/20260722010000_create_subscription_tables.sql" || migration.bytes !== 844 || migration.sha256 !== "20f38597d2d64bad3589036c2fe20aab2be89e5d240c540d401b46713c701349") fail("ordered migration pin drifted");
exact("migration guards", ["CREATE TABLE IF NOT EXISTS public.subscription_plans (", "CREATE TABLE IF NOT EXISTS public.subscriptions ("], migration.guards);
if (!migration.path.split("/").at(-1).startsWith(`${migration.version}_`)) fail("migration filename/version order is inconsistent");
const migrationBytes = readFileSync(regularRepoFile(migration.path, "subscription migration"));
const migrationSql = migrationBytes.toString("utf8");
if (migrationBytes.byteLength !== migration.bytes || sha256(migrationBytes) !== migration.sha256) fail("subscription migration bytes changed");
if ((migrationSql.match(/;/g) ?? []).length !== 2) fail("subscription migration must contain exactly two statements");
if ((migrationSql.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.(?:subscription_plans|subscriptions)\s*\(/gi) ?? []).length !== 2) fail("subscription migration must contain exactly two guarded table creations");
if (/\b(?:DROP|TRUNCATE|DELETE|ALTER|INSERT|UPDATE|MERGE|CASCADE)\b/i.test(migrationSql)) fail("subscription migration contains a destructive, data-mutation, or alteration token");
if (/\bCREATE\s+(?:SCHEMA|EXTENSION|INDEX|DATABASE|TYPE|VIEW)\b/i.test(migrationSql)) fail("subscription migration contains an out-of-scope creation");
if (/\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(migrationSql)) fail("transaction control belongs to the future reviewed runner");

const adminMigration = migrationRoot.orderedMigrations[1];
if (adminMigration.version !== "20260727000000" || adminMigration.path !== "services/subscription/migrations/20260727000000_create_admin_access_operations.sql" || adminMigration.bytes !== 1524 || adminMigration.sha256 !== "dbc244c1e24c0216adb3dc9dd0313534676e2555cabe8b663c9c88f4eada87b0") fail("ordered admin-access migration pin drifted");
const adminBytes = readFileSync(regularRepoFile(adminMigration.path, "subscription admin-access migration"));
const adminSql = adminBytes.toString("utf8");
if (adminBytes.byteLength !== adminMigration.bytes || sha256(adminBytes) !== adminMigration.sha256) fail("subscription admin-access migration bytes changed");
if ((adminSql.match(/;/g) ?? []).length !== 4 || !/CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.subscription_access_assignments/i.test(adminSql) || !/CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.subscription_admin_operations/i.test(adminSql) || /\b(?:DROP|TRUNCATE|INSERT|UPDATE|MERGE|ON\s+DELETE\s+CASCADE|ALTER)\b/i.test(adminSql)) fail("subscription admin-access migration is not a safe additive shape");

const stateMigration = migrationRoot.orderedMigrations[2];
if (stateMigration.version !== "20260727010000" || stateMigration.path !== "services/subscription/migrations/20260727010000_create_plan_state.sql" || stateMigration.bytes !== 511 || stateMigration.sha256 !== "9122ca4f8f43bfe524d4784c7b211abc0e58843d732b991b4d47d2109aa60783") fail("ordered plan-state migration pin drifted");
const stateBytes = readFileSync(regularRepoFile(stateMigration.path, "subscription plan-state migration"));
const stateSql = stateBytes.toString("utf8");
if (stateBytes.byteLength !== stateMigration.bytes || sha256(stateBytes) !== stateMigration.sha256) fail("subscription plan-state migration bytes changed");
if ((stateSql.match(/;/g) ?? []).length !== 1 || !/CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.subscription_plan_state\s*\(/i.test(stateSql) || !/PRIMARY\s+KEY\s+REFERENCES\s+public\.subscription_plans\(id\)\s+ON\s+DELETE\s+RESTRICT/i.test(stateSql) || !/CHECK\s*\(version\s+>=\s+0\)/i.test(stateSql)) fail("subscription plan-state migration is not the reviewed additive shape");
if (/\b(?:DROP|TRUNCATE|INSERT|UPDATE|MERGE|ON\s+DELETE\s+CASCADE|ALTER)\b/i.test(stateSql) || /\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(stateSql)) fail("subscription plan-state migration contains a destructive token or transaction control");

if (!Array.isArray(contract.requiredTables) || contract.requiredTables.length !== 2) fail("exactly two legacy tables are required");
const expectedTableNames = ["subscription_plans", "subscriptions"];
for (let tableIndex = 0; tableIndex < contract.requiredTables.length; tableIndex += 1) {
  const table = contract.requiredTables[tableIndex];
  if (table.name !== expectedTableNames[tableIndex] || !Array.isArray(table.columns) || table.columns.length !== 10) fail(`required table ${tableIndex + 1} drifted`);
  const bodyMatch = migrationSql.match(new RegExp(`CREATE\\s+TABLE\\s+IF\\s+NOT\\s+EXISTS\\s+public\\.${table.name}\\s*\\(([\\s\\S]*?)\\n\\);`, "i"));
  if (!bodyMatch) fail(`${table.name}: migration table body is malformed`);
  const actualDefinitions = bodyMatch[1].split(/\r?\n/).map((line) => line.trim().replace(/,$/, "")).filter(Boolean);
  const expectedDefinitions = [];
  for (let columnIndex = 0; columnIndex < table.columns.length; columnIndex += 1) {
    const column = table.columns[columnIndex];
    if (column.ordinal !== columnIndex + 1 || typeof column.name !== "string" || !column.name || typeof column.sqlAnchor !== "string") fail(`${table.name}: required column ${columnIndex + 1} drifted`);
    expectedDefinitions.push(column.sqlAnchor.replace(/,$/, ""));
  }
  exact(`${table.name} column definitions`, expectedDefinitions, actualDefinitions);
}

exact("isolated scanner delta", {
  runtimeRustDdlFindings: { before: 37, after: 35, delta: -2 },
  actionableFindings: { before: 31, after: 29, delta: -2 },
  subscriptionFindings: { before: 2, after: 0, delta: -2 },
  reviewedExceptions: { before: 6, after: 6, delta: 0 },
  status: "isolated-projection-only-canonical-rebaseline-owned-elsewhere",
}, contract.isolatedScannerDelta);
if (!Array.isArray(contract.nonClaims) || contract.nonClaims.length !== 4 || contract.nonClaims.some((item) => typeof item !== "string" || item.length < 60)) fail("four substantive non-claims are required");
const blockerCategories = ["migration-runner", "baseline-adoption", "populated-upgrade", "reconciliation", "concurrent-startup", "live-database"];
if (!Array.isArray(contract.blockers) || contract.blockers.length !== 6) fail("exactly six residual blockers are required");
for (let index = 0; index < contract.blockers.length; index += 1) {
  const blocker = contract.blockers[index];
  const id = `B${String(index + 1).padStart(2, "0")}`;
  if (blocker.id !== id || blocker.category !== blockerCategories[index] || blocker.status !== "blocked" || typeof blocker.summary !== "string" || blocker.summary.length < 50) fail(`${id}: residual blocker drifted`);
}

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  service: contract.scope.service,
  status: contract.scope.status,
  developmentMapping: { sourceCommit: baseline.sourceCommit, candidateServicePresent: false, status: baseline.mappingStatus },
  runtimeRust: { files: rustFiles.length, ddlFindings: runtimeFindings.length, expectedDelta: -2, qualifiedRelationOccurrences: runtime.qualifiedRelationOccurrences },
  models: {
    responseFields: responseFieldCount,
    nullableResponseFields: nullableFieldCount,
    requestFields: expectedRequestModels.reduce((count, model) => count + model.fields.length, 0),
    uuidPathExtractors: modelBoundary.pathUuidExtractorOccurrences,
    queryAsOccurrences: modelBoundary.queryAsOccurrences,
  },
  migrationRoot: { path: migrationRoot.path, migrations: 1, pinnedBytes: migration.bytes, sha256: migration.sha256, runner: null },
  requiredTables: contract.requiredTables.map((table) => ({ name: table.name, columns: table.columns.length })),
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
  echo "a3-7-subscription-schema-boundary: PASS — subscription runtime DDL 2→0, baseline plus additive plan-state migration pinned, and response fields verified"
  echo "a3-7-subscription-schema-boundary: LIMIT — no runner, baseline adoption, populated upgrade, reconciliation, concurrent startup, or live database proof ran"
  exit 0
fi

echo "a3-7-subscription-schema-boundary: STOP — six residual A3.7 blockers remain; readiness is intentionally exit 3" >&2
echo "a3-7-subscription-schema-boundary: LIMIT — static integrity is not migration, development-data mapping, or database execution evidence" >&2
exit 3
