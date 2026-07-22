#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-11-notification-schema-boundary.json"
mode=""

die() {
  echo "a3-11 notification schema: ERROR: $*" >&2
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

for env_name in DATABASE_URL NOTIFICATIONS_DATABASE_URL REDIS_URL SMTP_HOST HTTP_PROXY HTTPS_PROXY ALL_PROXY; do
  eval "env_value=\${$env_name-}"
  [ -z "$env_value" ] || die "$env_name is set; this offline gate never contacts a database, Redis, SMTP, or network endpoint"
done
case "${EPSX_ENV-}" in
  production|prod) die "EPSX_ENV is production-looking; this gate is static-only" ;;
esac

summary=$(bun -e '
import { readFileSync, realpathSync, statSync } from "node:fs";
import { resolve, sep } from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const root = realpathSync(process.argv[1]);
const contractPath = realpathSync(process.argv[2]);
const fail = (message) => { console.error(`a3-11 notification schema: ERROR: ${message}`); process.exit(1); };
const exact = (label, expected, actual) => {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) fail(`${label} drifted`);
};
const safeRelative = (path, label) => {
  if (typeof path !== "string" || !path || path.startsWith("/") || path.split(/[\\/]+/).includes("..")) fail(`${label}: unsafe path`);
  const candidate = resolve(root, path);
  let actual;
  try { actual = realpathSync(candidate); } catch { fail(`${label}: missing file ${path}`); }
  if (actual !== root && !actual.startsWith(`${root}${sep}`)) fail(`${label}: path escapes repository`);
  return actual;
};
const gitResult = (...args) => spawnSync("git", ["-C", root, ...args], { encoding: "utf8", env: { PATH: process.env.PATH ?? "" } });
const git = (...args) => {
  const result = gitResult(...args);
  if (result.status !== 0) fail(`git ${args.join(" ")} failed: ${(result.stderr || result.stdout).trim()}`);
  return result.stdout.trim();
};
const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const occurrences = (text, needle) => text.split(needle).length - 1;
const canonicalPins = {
  compatibilityQuery: { bytes: 20887, sha256: "8733c2fd595ad6ea319dc83a5d9ece2adad0e78008a134b129faae6fcdea190e" },
  upMigration: { bytes: 1128, sha256: "788fa9500df1759d7b224c739f90f4756c2397f28a42aca1ec9af197f27290f7" },
  downMigration: { bytes: 191, sha256: "5f47cf6f1c82416ac8c60bd3e691c78b4d58f4ee78bae3f778869206350b76cc", exactBodyRequired: true },
};
const canonicalCatalogIdentifiers = {
  "information_schema.columns": 2,
  "pg_catalog.pg_namespace": 13,
  "pg_catalog.pg_class": 13,
  "pg_catalog.pg_constraint": 7,
  "pg_catalog.pg_attribute": 4,
  "pg_catalog.pg_index": 2,
  "pg_catalog.pg_type": 2,
  "pg_catalog.pg_opclass": 2,
  "pg_catalog.pg_am": 1,
  "pg_catalog.pg_inherits": 1,
  "pg_catalog.pg_policy": 1,
};
const body = (sql, table) => {
  const start = `CREATE TABLE IF NOT EXISTS public.${table} (`;
  const startAt = sql.indexOf(start);
  if (startAt < 0 || sql.indexOf(start, startAt + 1) >= 0) fail(`${table}: guarded table body is not unique`);
  const endAt = sql.indexOf("\n);", startAt);
  if (endAt < 0) fail(`${table}: table body terminator missing`);
  return sql.slice(startAt, endAt + 3);
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A3.11-notification-schema-boundary") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "offline-static-notification-schema-boundary-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", { service: "services/notification", database: "epsx_notification", schema: "public", tables: ["templates", "notifications"], status: "partial" }, contract.scope);
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("safety flags must remain false");
if (typeof contract.safety.readinessMeaning !== "string" || !contract.safety.readinessMeaning.includes("not clean-database execution")) fail("readiness meaning weakened");
exact("independent evidence pinning", {
  authority: "contract-plus-independent-verifier-constants",
  compatibilityQuery: canonicalPins.compatibilityQuery,
  upMigration: canonicalPins.upMigration,
  downMigration: canonicalPins.downMigration,
  catalogIdentifierInventory: canonicalCatalogIdentifiers,
}, contract.evidencePinning);

const provenance = contract.sourceProvenance;
if (!provenance || !provenance.development || !provenance.removedRuntimeSnapshot) fail("source provenance is incomplete");
const development = provenance.development;
if (development.ref !== "origin/development" || development.refRole !== "provenance-label-only" || development.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || development.servicePresent !== false) fail("pinned development provenance drifted");
if (!Array.isArray(development.evidence) || development.evidence.length !== 1) fail("exactly one immutable development schema record is required");
for (const item of development.evidence) {
  if (!item || !/^[a-z][a-z0-9-]+$/.test(item.id) || !/^[0-9a-f]{40}$/.test(item.blob)) fail("development evidence is malformed");
  if (git("rev-parse", `${development.commit}:${item.file}`) !== item.blob) fail(`${item.id}: pinned blob drifted`);
  const source = git("show", `${development.commit}:${item.file}`);
  if (occurrences(source, item.anchor) !== 1) fail(`${item.id}: immutable anchor drifted`);
}
if (gitResult("cat-file", "-e", `${development.commit}:services/notification/Cargo.toml`).status === 0) fail("notification service unexpectedly exists in the pinned development commit");

const removed = provenance.removedRuntimeSnapshot;
if (removed.commit !== "b624f320c2db3dc24944cc0414deae7bc2d42196" || removed.file !== "services/notification/src/main.rs" || removed.blob !== "64633151dae98bd7e5368d225f869936d3237a41") fail("removed runtime snapshot pin drifted");
if (git("rev-parse", `${removed.commit}:${removed.file}`) !== removed.blob) fail("removed runtime source blob drifted");
const removedSource = git("show", `${removed.commit}:${removed.file}`);
exact("removed runtime anchors", [
  "CREATE TABLE IF NOT EXISTS templates (",
  "CREATE TABLE IF NOT EXISTS notifications (",
  "CREATE INDEX IF NOT EXISTS idx_notif_user ON notifications",
  "CREATE INDEX IF NOT EXISTS idx_notif_status ON notifications",
  "seed_default_templates(&db).await;",
  "seed_sample_notifications(&db).await;",
  "Some(chrono::Utc::now().naive_utc())",
], removed.requiredAnchors);
for (const anchor of removed.requiredAnchors) if (occurrences(removedSource, anchor) !== 1) fail(`removed runtime anchor drifted: ${anchor}`);
const ddlPattern = /\bCREATE\s+(?:TABLE|INDEX)\b/gi;
if ((removedSource.match(ddlPattern) ?? []).length !== 4) fail("removed runtime DDL baseline is not exactly four findings");

const runtime = contract.runtimeBoundary;
exact("runtime inventory", ["services/notification/src/lib.rs", "services/notification/src/main.rs"], runtime.rustInventory);
if (runtime.rustRoot !== "services/notification" || runtime.ddlFindingsBefore !== 4 || runtime.ddlFindingsAfter !== 0) fail("runtime DDL finding delta drifted");
if (runtime.startupSeedCallsBefore !== 2 || runtime.startupSeedCallsAfter !== 0 || runtime.startupSeedWriteSitesAfter !== 0 || runtime.startupErrorSwallowSitesAfter !== 0) fail("startup seed/error boundary drifted");
const libPath = safeRelative(runtime.rustInventory[0], "notification lib");
const mainPath = safeRelative(runtime.rustInventory[1], "notification main");
const libSource = readFileSync(libPath, "utf8");
const mainSource = readFileSync(mainPath, "utf8");
const currentRust = `${libSource}\n${mainSource}`;
if ((currentRust.match(ddlPattern) ?? []).length !== 0) fail("runtime Rust still contains DDL");
for (const forbidden of ["seed_default_templates", "seed_sample_notifications", ".await.ok()", "naive_utc()"] ) if (currentRust.includes(forbidden)) fail(`runtime forbidden anchor remains: ${forbidden}`);
if (/let\s+_\s*=\s*sqlx::query/.test(mainSource)) fail("startup/query errors are still explicitly discarded");
if (!mainSource.includes("Result<(), TemplateLoadError>") || !mainSource.includes(".await?;") || !mainSource.includes("hb.register_template_string(&template.name, template.body)?;") || !mainSource.includes(".expect(\"active notification templates must load before startup\")")) fail("template cache load is not fail closed");
if (!mainSource.includes("Some(chrono::Utc::now())")) fail("TIMESTAMPTZ sent_at bind is not DateTime<Utc>");
if (!libSource.includes("sqlx::query_scalar::<_, bool>(NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY)")) fail("compatibility query is not a scalar boolean probe");

exact("qualified relation counts", { "public.templates": 8, "public.notifications": 11 }, runtime.qualifiedRelationOccurrences);
for (const [relation, expected] of Object.entries(runtime.qualifiedRelationOccurrences)) {
  if (occurrences(mainSource, relation) !== expected) fail(`${relation}: runtime occurrence count drifted`);
}
if (/\b(?:FROM|JOIN|INTO|UPDATE)\s+(?:templates|notifications)\b/i.test(mainSource) || /\bDELETE\s+FROM\s+(?:templates|notifications)\b/i.test(mainSource)) fail("unqualified notification runtime relation found");
exact("main startup sequence", [
  "sqlx::PgPool::connect(&args.database_url)",
  "verify_schema_compatibility(&db)",
  "load_templates_to_hb(&db, &mut hb)",
  "SmtpTransport::relay(&args.smtp_host)",
  "tokio::net::TcpListener::bind(addr)",
  "axum::serve(listener, app)",
], runtime.mainSequence);
let last = -1;
for (const anchor of runtime.mainSequence) {
  const at = mainSource.indexOf(anchor);
  if (at <= last) fail(`startup sequence drifted at ${anchor}`);
  last = at;
}

const queryMatch = libSource.match(/const NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!queryMatch || occurrences(libSource, "const NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY") !== 1) fail("compatibility query boundary drifted");
const query = queryMatch[1];
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (runtime.compatibilityQueryConstant !== "NOTIFICATION_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility") fail("compatibility API contract drifted");
const keyConstraintStart = query.indexOf("key_constraint_compatibility AS (");
const keyConstraintEnd = query.indexOf("not_null_catalog_exposure AS (");
if (keyConstraintStart < 0 || keyConstraintEnd <= keyConstraintStart) fail("key constraint query boundary drifted");
const keyConstraintQuery = query.slice(keyConstraintStart, keyConstraintEnd);
if (!keyConstraintQuery.includes("AND constraint_record.connoinherit") || keyConstraintQuery.includes("AND NOT constraint_record.connoinherit")) fail("key constraints must require connoinherit=true for PostgreSQL 18 fresh-schema keys");
if (!keyConstraintQuery.includes("to_jsonb(constraint_record) ? \"conperiod\"".replaceAll("\"", "\x27")) || !keyConstraintQuery.includes("->> \"conperiod\"".replaceAll("\"", "\x27"))) fail("key constraints must reject PostgreSQL 18 period semantics");
if (!Array.isArray(runtime.queryRequiredAnchors) || runtime.queryRequiredAnchors.length < 50) fail("compatibility query anchor inventory is incomplete");
for (const anchor of runtime.queryRequiredAnchors) if (!query.includes(anchor)) fail(`compatibility query missing required anchor: ${anchor}`);
if (!/^\s*WITH\s/.test(query) || !/\nSELECT\s/.test(query)) fail("compatibility query must be read-only WITH/SELECT SQL");
if (/\b(?:INSERT|UPDATE|DELETE|MERGE|COPY|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i.test(query)) fail("compatibility query contains a mutation token");
if (/\b(?:search_path|current_schema|pg_temp)\b/i.test(query)) fail("compatibility query depends on mutable schema resolution");
const actualCatalogIdentifiers = Object.fromEntries(
  Object.keys(canonicalCatalogIdentifiers).map((identifier) => [identifier, occurrences(query, identifier)]),
);
exact("compatibility query catalog identifier inventory", canonicalCatalogIdentifiers, actualCatalogIdentifiers);
if (Buffer.byteLength(query) !== canonicalPins.compatibilityQuery.bytes || sha256(query) !== canonicalPins.compatibilityQuery.sha256) fail("canonical compatibility query digest drifted independently of contract pins");

const migration = contract.migrationRoot;
if (migration.path !== "apps/backend/migrations/notifications" || migration.runnerConfig !== "apps/backend/diesel_notifications.toml" || migration.runnerDirectory !== "migrations/notifications/" || migration.historyStatus !== "blocked-preexisting-unsafe-history") fail("notification migration root contract drifted");
exact("runner print-schema filter", ["notifications", "wallet_notifications"], migration.runnerPrintSchemaFilter);
exact("runner print-schema missing tables", ["templates"], migration.runnerPrintSchemaMissing);
if (!Array.isArray(migration.preexistingStopEvidence) || migration.preexistingStopEvidence.length !== 2 || !migration.preexistingStopEvidence[1].includes("CASCADE")) fail("pre-existing migration-history STOP evidence changed");
const runnerSource = readFileSync(safeRelative(migration.runnerConfig, "notification migration runner"), "utf8");
if (!runnerSource.includes(`dir = "${migration.runnerDirectory}"`)) fail("notification migration runner directory drifted");
if (!runnerSource.includes(`only_tables = ["notifications", "wallet_notifications"]`) || runnerSource.includes(`only_tables = ["notifications", "templates", "wallet_notifications"]`)) fail("notification print-schema filter evidence drifted");
if (!Array.isArray(migration.orderedMigrations) || migration.orderedMigrations.length !== 1) fail("exactly one A3.11 migration is required");
const item = migration.orderedMigrations[0];
if (item.version !== "20260722040000" || item.directory !== `${migration.path}/${item.version}_create_notification_service_tables`) fail("ordered migration identity drifted");
if (item.up.path !== `${item.directory}/up.sql` || item.down.path !== `${item.directory}/down.sql`) fail("ordered migration path drifted");
const upPath = safeRelative(item.up.path, "A3.11 up migration");
const downPath = safeRelative(item.down.path, "A3.11 down migration");
const upSql = readFileSync(upPath, "utf8");
const downSql = readFileSync(downPath, "utf8");
for (const [label, pin, source, path] of [["up", item.up, upSql, upPath], ["down", item.down, downSql, downPath]]) {
  if (!/^[0-9a-f]{64}$/.test(pin.sha256) || statSync(path).size !== pin.bytes || sha256(source) !== pin.sha256) fail(`${label} migration bytes changed`);
}
exact("migration guards", [
  "CREATE TABLE IF NOT EXISTS public.templates (",
  "CREATE TABLE IF NOT EXISTS public.notifications (",
  "CREATE INDEX IF NOT EXISTS idx_notif_user",
  "CREATE INDEX IF NOT EXISTS idx_notif_status",
], item.guards);
for (const guard of item.guards) if (occurrences(upSql, guard) !== 1) fail(`migration guard drifted: ${guard}`);
if ((upSql.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\b/gi) ?? []).length !== 2 || (upSql.match(/\bCREATE\s+INDEX\s+IF\s+NOT\s+EXISTS\b/gi) ?? []).length !== 2) fail("migration must contain exactly two guarded tables and two guarded indexes");
if (/\b(?:DROP|ALTER|TRUNCATE|DELETE|INSERT|UPDATE|MERGE|COPY|CREATE\s+(?:EXTENSION|SCHEMA)|BEGIN|COMMIT|ROLLBACK|SAVEPOINT|DO|CALL|EXECUTE|GRANT|REVOKE|SET)\b/i.test(upSql)) fail("up migration contains a forbidden mutation/control token");
if (/\b(?:CASCADE|REFERENCES|CHECK)\b/i.test(upSql)) fail("up migration contains an unexpected cascade, foreign key, or check");
if ((upSql.match(/\bPRIMARY\s+KEY\b/gi) ?? []).length !== 2 || (upSql.match(/\bUNIQUE\b/gi) ?? []).length !== 1 || (upSql.match(/\bNOT\s+NULL\b/gi) ?? []).length !== 12) fail("migration key/explicit-nullability totals drifted");
const normalizedUp = upSql.replace(/\s+/g, " ").trim();
for (const indexSql of [
  "CREATE INDEX IF NOT EXISTS idx_notif_user ON public.notifications (user_id ASC, created_at DESC);",
  "CREATE INDEX IF NOT EXISTS idx_notif_status ON public.notifications (status ASC);",
]) if (!normalizedUp.includes(indexSql)) fail(`index definition drifted: ${indexSql}`);
if (item.down.policy !== "forward-only-refusal" || !downSql.includes("RAISE EXCEPTION \x27A3.11 notification schema migration is forward-only") || !/^DO \$forward_only\$/m.test(downSql)) fail("down migration is not the pinned forward-only refusal");
if (/\b(?:DROP|ALTER|TRUNCATE|DELETE|INSERT|UPDATE|MERGE|COPY|CREATE|GRANT|REVOKE)\b/i.test(downSql)) fail("down refusal contains destructive or data-mutating SQL");
const apostrophe = String.fromCharCode(39);
const canonicalDownBody = `DO $forward_only$\nBEGIN\n    RAISE EXCEPTION ${apostrophe}A3.11 notification schema migration is forward-only; destructive rollback requires a separately reviewed recovery migration${apostrophe};\nEND\n$forward_only$;\n`;
if (downSql !== canonicalDownBody) fail("down migration exact body or dollar-quote delimiters drifted");
if (Buffer.byteLength(upSql) !== canonicalPins.upMigration.bytes || sha256(upSql) !== canonicalPins.upMigration.sha256) fail("canonical up migration digest drifted independently of contract pins");
if (Buffer.byteLength(downSql) !== canonicalPins.downMigration.bytes || sha256(downSql) !== canonicalPins.downMigration.sha256) fail("canonical down migration digest drifted independently of contract pins");

const expectedTables = [
  ["templates", [
    [1, "id", "id VARCHAR(66) PRIMARY KEY,"], [2, "name", "name VARCHAR(100) UNIQUE NOT NULL,"],
    [3, "channel", "channel VARCHAR(20) NOT NULL,"], [4, "subject", "subject TEXT,"],
    [5, "body", "body TEXT NOT NULL,"], [6, "variables", "variables JSONB NOT NULL DEFAULT \x27{}\x27::jsonb,"],
    [7, "active", "active BOOLEAN NOT NULL DEFAULT true,"], [8, "created_at", "created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),"],
    [9, "updated_at", "updated_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW()"],
  ]],
  ["notifications", [
    [1, "id", "id VARCHAR(66) PRIMARY KEY,"], [2, "user_id", "user_id VARCHAR(66),"],
    [3, "channel", "channel VARCHAR(20) NOT NULL,"], [4, "recipient", "recipient VARCHAR(255) NOT NULL,"],
    [5, "template_id", "template_id VARCHAR(66),"], [6, "subject", "subject TEXT,"],
    [7, "body", "body TEXT NOT NULL,"], [8, "data", "data JSONB,"],
    [9, "status", "status VARCHAR(20) NOT NULL DEFAULT \x27pending\x27,"], [10, "error", "error TEXT,"],
    [11, "sent_at", "sent_at TIMESTAMPTZ(6),"], [12, "created_at", "created_at TIMESTAMPTZ(6) NOT NULL DEFAULT NOW(),"],
    [13, "read_at", "read_at TIMESTAMPTZ(6),"], [14, "title", "title TEXT,"],
    [15, "notification_type", "notification_type VARCHAR(50),"], [16, "priority", "priority VARCHAR(20),"],
    [17, "action_url", "action_url TEXT"],
  ]],
];
if (!Array.isArray(contract.requiredTables) || contract.requiredTables.length !== 2) fail("exactly two table contracts are required");
for (let tableIndex = 0; tableIndex < expectedTables.length; tableIndex++) {
  const [name, columns] = expectedTables[tableIndex];
  const actual = contract.requiredTables[tableIndex];
  if (!actual || actual.name !== name) fail(`${name}: table contract order drifted`);
  exact(`${name} columns`, columns.map(([ordinal, column, sqlAnchor]) => ({ ordinal, name: column, sqlAnchor })), actual.columns);
  const tableSql = body(upSql, name);
  for (const [, column, sqlAnchor] of columns) if (occurrences(tableSql, sqlAnchor) !== 1) fail(`${name}.${column}: migration column anchor drifted`);
}

const semantics = contract.constraintSemantics;
if (semantics.columnTotal !== 26 || semantics.notNullTotal !== 14 || semantics.relationConstraintTotalWithoutPg18NotNull !== 3 || semantics.pg18NotNullInventory !== "zero-or-exact-fourteen") fail("schema totals drifted");
exact("primary keys", ["templates.id", "notifications.id"], semantics.primaryKeys);
exact("unique keys", ["templates.name"], semantics.uniqueKeys);
exact("foreign keys", [], semantics.foreignKeys);
exact("check constraints", [], semantics.checkConstraints);
exact("key constraint policy", { local: true, inheritCount: 0, noInherit: true, pg18Period: false, validated: true, enforcedWhenExposed: true, deferrable: false, initiallyDeferred: false }, semantics.keyConstraintPolicy);
const expectedIndexes = [
  { name: "templates_pkey", table: "templates", unique: true, primary: true, keys: ["id ASC"], opclasses: ["text_ops"], collation: "column-exact" },
  { name: "templates_name_key", table: "templates", unique: true, primary: false, keys: ["name ASC"], opclasses: ["text_ops"], collation: "column-exact" },
  { name: "notifications_pkey", table: "notifications", unique: true, primary: true, keys: ["id ASC"], opclasses: ["text_ops"], collation: "column-exact" },
  { name: "idx_notif_user", table: "notifications", unique: false, primary: false, keys: ["user_id ASC", "created_at DESC"], opclasses: ["text_ops", "timestamptz_ops"], collation: "column-exact" },
  { name: "idx_notif_status", table: "notifications", unique: false, primary: false, keys: ["status ASC"], opclasses: ["text_ops"], collation: "column-exact" },
];
exact("index inventory", expectedIndexes, semantics.indexes);
exact("index policy", { exactTotal: 5, accessMethod: "btree", valid: true, ready: true, live: true, immediate: true, partialAccepted: false, expressionAccepted: false, includedColumnsAccepted: false, standaloneUniqueAccepted: false, unexpectedIndexAccepted: false }, semantics.indexPolicy);
exact("foreign key boundary", { inventoryDirection: "inbound-and-outbound", exactTotal: 0, unexpectedInboundAccepted: false, unexpectedOutboundAccepted: false }, semantics.foreignKeyBoundary);
exact("relation policy", { ordinary: true, permanent: true, partitioned: false, inheritanceAccepted: false, rowLevelSecurity: false, forceRowLevelSecurity: false, policiesAccepted: false, replicaIdentity: "default" }, semantics.relationPolicy);

if (!Array.isArray(contract.residualBlockers) || contract.residualBlockers.length !== 7) fail("exactly seven residual blockers are required");
contract.residualBlockers.forEach((blocker, index) => {
  const id = `B${String(index + 1).padStart(2, "0")}`;
  if (!blocker || blocker.id !== id || blocker.status !== "blocked" || typeof blocker.summary !== "string" || blocker.summary.length < 30) fail(`${id}: residual blocker drifted`);
});

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  evidencePinning: { authority: contract.evidencePinning.authority, catalogIdentifiers: Object.keys(canonicalCatalogIdentifiers).length, exactDownBody: true },
  source: { developmentCommit: development.commit, evidence: development.evidence.length, servicePresent: development.servicePresent, removedRuntimeCommit: removed.commit },
  runtime: { ddlBefore: runtime.ddlFindingsBefore, ddlAfter: runtime.ddlFindingsAfter, seedCallsBefore: runtime.startupSeedCallsBefore, seedCallsAfter: runtime.startupSeedCallsAfter, qualifiedRelations: runtime.qualifiedRelationOccurrences, compatibilityQueryBytes: runtime.compatibilityQueryBytes, compatibilityQuerySha256: runtime.compatibilityQuerySha256 },
  migration: { root: migration.path, runner: migration.runnerConfig, runnerPrintSchemaMissing: migration.runnerPrintSchemaMissing, migrations: migration.orderedMigrations.length, upBytes: item.up.bytes, upSha256: item.up.sha256, downBytes: item.down.bytes, downSha256: item.down.sha256, historyStatus: migration.historyStatus },
  schema: { tables: contract.requiredTables.map((table) => table.name), columns: semantics.columnTotal, notNull: semantics.notNullTotal, primaryKeys: semantics.primaryKeys.length, uniqueKeys: semantics.uniqueKeys.length, foreignKeys: semantics.foreignKeys.length, checks: semantics.checkConstraints.length, indexes: semantics.indexes.length, keyConstraintPolicy: semantics.keyConstraintPolicy },
  blockers: contract.residualBlockers.map(({ id, status }) => ({ id, status })),
  productionReady: false,
  readinessExit: 3,
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s\n' "$summary"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "a3-11 notification schema: PASS — runtime DDL 4→0 and startup seed calls 2→0; fail-closed compatibility precedes cache load/listener"
  echo "a3-11 notification schema: PASS — two guarded public tables, 26 exact columns, 14 NOT NULL columns, three exact keys, and five exact indexes pinned"
  echo "a3-11 notification schema: PASS — PostgreSQL 18 keys require connoinherit=true and reject WITHOUT OVERLAPS period semantics"
  echo "a3-11 notification schema: PASS — query/up/down evidence is independently hard-pinned; exact catalog identifiers and down delimiters are locked"
  echo "a3-11 notification schema: PASS — complete inbound/outbound FK and CHECK inventories are empty; inheritance, RLS, partial/expression/included indexes fail closed"
  echo "a3-11 notification schema: PASS — 19 runtime relations are public-qualified; TIMESTAMPTZ uses DateTime<Utc>; migration and source provenance digests are immutable"
  echo "a3-11 notification schema: LIMIT — seven blockers remain; no database, migration execution, upgrade, reconciliation, rollback, network, deployment, or production readiness was proven"
  exit 0
fi

echo "a3-11 notification schema: STOP — seven residual blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "a3-11 notification schema: LIMIT — static integrity can pass while migration-history adoption, live schema execution, legacy reconciliation, and rollback remain blocked" >&2
exit 3
