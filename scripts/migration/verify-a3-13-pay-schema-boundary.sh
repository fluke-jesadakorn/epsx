#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-13-pay-schema-boundary.json"
mode=""

die() {
  echo "a3-13-pay-schema-boundary: ERROR: $*" >&2
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

for name in DATABASE_URL PAY_DATABASE_URL PAYMENTS_DATABASE_URL PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD RPC_URL CHAIN_RPC_URL; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts a database or chain"
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
  console.error(`a3-13-pay-schema-boundary: ERROR: ${message}`);
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
const exactKeys = (label, expected, actual) => {
  if (!actual || typeof actual !== "object" || Array.isArray(actual)) fail(`${label} must be an object`);
  exact(`${label} field inventory`, [...expected].sort(), Object.keys(actual).sort());
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) fail(`${label} must be a safe repository-relative path`);
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

if (contract.schemaVersion !== 1 || contract.contractId !== "A3.13-pay-schema-boundary") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "offline-static-pay-candidate-schema-boundary-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("top-level field inventory", [
  "authorityBoundary", "blockers", "constraintAndIndexSemantics", "contractId", "failClosedBoundary", "freshSchemaDrift",
  "integrityExit", "isolatedPostgres18Evidence", "isolatedRuntimeDelta", "migrationRoot", "nonClaims", "productionReady",
  "purpose", "readinessExit", "requiredTables", "runtimeBoundary", "rustModelContract", "safety", "schemaVersion", "scope",
].sort(), Object.keys(contract).sort());
exact("scope", {
  service: "services/pay",
  databaseCandidate: "epsx_pay",
  schema: "public",
  tables: ["pay_intents", "escrows", "pay_links", "pay_webhook_events"],
  status: "partial",
}, contract.scope);
exact("safety boundary", {
  deploymentAuthorized: false,
  databaseAccessAuthorized: false,
  migrationExecutionAuthorized: false,
  networkAccessAuthorized: false,
  providerAccessAuthorized: false,
  chainAccessAuthorized: false,
  runtimeMutationAuthorized: false,
  financialRouteEnablementAuthorized: false,
  readinessMeaning: "The integrity verifier itself proves only local source, migration bytes, read-only catalog SQL, relation qualification, and the existing fail-closed route boundary. Separately recorded disposable PostgreSQL 18 evidence is not authority, adoption, populated upgrade, reconciliation, chain, or deployed-database evidence.",
}, contract.safety);

const authority = contract.authorityBoundary;
if (!authority || authority.decision !== "unresolved-do-not-cut-over-or-dual-write") fail("database authority STOP changed");
exactKeys("authority boundary", [
  "decision", "candidateDatabaseNames", "developmentSourceRef", "developmentSourceRefRole", "developmentSourceCommit", "developmentEvidence",
], authority);
exact("candidate database names", ["epsx_payment", "epsx_pay", "epsx_payments_dev", "epsx_payments_staging", "epsx_payments_prod"], authority.candidateDatabaseNames);
if (authority.developmentSourceRef !== "origin/development" || authority.developmentSourceRefRole !== "provenance-label-only") fail("development provenance label drifted");
if (authority.developmentSourceCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") fail("pinned development source commit drifted");
if (!gitExists("cat-file", "-e", `${authority.developmentSourceCommit}^{commit}`)) fail("pinned development source commit is unavailable");
if (!Array.isArray(authority.developmentEvidence) || authority.developmentEvidence.length !== 7) fail("exactly seven immutable development evidence rows are required");
const developmentIds = ["payment-baseline", "payment-replay-index", "payment-routes", "payment-owner", "payment-owner-status", "payment-finality", "payment-browser-contract"];
exact("immutable development evidence", [
  { id: "payment-baseline", file: "apps/backend/migrations/payments/00000000000001_consolidated_baseline_v4/up.sql", blob: "02bd7c603211580e7cf39eaf29f69f96d4bd88db", anchor: "CREATE TABLE payments (" },
  { id: "payment-replay-index", file: "apps/backend/migrations/payments/20260220100000_add_unique_tx_hash_and_expiry/up.sql", blob: "eaef0ae9b844abea9e46c95ece399c68901ac2e0", anchor: "CREATE UNIQUE INDEX IF NOT EXISTS idx_payments_unique_tx_hash" },
  { id: "payment-routes", file: "apps/backend/src/web/routes/unified_router.rs", blob: "46b97779e8726757560d7946d1a47114bed28861", anchor: ".route(\"/submit\", post(submit_transaction_handler))" },
  { id: "payment-owner", file: "apps/backend/src/web/payments/submit_tx_handler.rs", blob: "089598e31a0de96ae96efe6187d4057ec2dbbb59", anchor: "let wallet_address = user_context.wallet_address.clone();" },
  { id: "payment-owner-status", file: "apps/backend/src/web/payments/get_tx_status_handler.rs", blob: "d521dfd8295af7dea2482aca9d8ccca791e421dd", anchor: ".filter(payments::wallet_address.eq(&wallet_address))" },
  { id: "payment-finality", file: "apps/backend/src/infrastructure/blockchain/tx_monitor_service.rs", blob: "decfe180641b9904831c7ebae956429447cbb7a0", anchor: "min_confirmations: if is_mainnet { 15 } else { 3 }," },
  { id: "payment-browser-contract", file: "shared/api/payments.ts", blob: "6f913d8618c12fc46b517b572b35fc51cf8ca329", anchor: "return this.client.post<TransactionStatusData>(\x27/api/payments/submit\x27, request);" },
], authority.developmentEvidence);
for (let index = 0; index < authority.developmentEvidence.length; index += 1) {
  const item = authority.developmentEvidence[index];
  exactKeys(`development evidence ${index + 1}`, ["id", "file", "blob", "anchor"], item);
  if (item.id !== developmentIds[index] || typeof item.blob !== "string" || !/^[0-9a-f]{40}$/.test(item.blob) || typeof item.anchor !== "string" || !item.anchor) fail(`development evidence ${index + 1} drifted`);
  safeRelative(item.file, `development evidence ${item.id}`);
  const actualBlob = git("rev-parse", `${authority.developmentSourceCommit}:${item.file}`).toString().trim();
  if (actualBlob !== item.blob) fail(`${item.id}: pinned source blob drifted`);
  const content = git("show", `${authority.developmentSourceCommit}:${item.file}`).toString();
  if (!content.includes(item.anchor)) fail(`${item.id}: pinned source anchor is missing`);
}

const runtime = contract.runtimeBoundary;
if (!runtime || runtime.rustRoot !== "services/pay/src" || runtime.scannerFindingBefore !== 10 || runtime.scannerFindingAfter !== 0) fail("runtime scanner boundary drifted");
exactKeys("runtime boundary", [
  "rustRoot", "rustInventory", "scannerFindingBefore", "scannerFindingAfter", "removedRuntimeSnapshot", "removedAnchors",
  "compatibilityQueryConstant", "compatibilityFunction", "compatibilityQueryBytes", "compatibilityQuerySha256",
  "qualifiedRelationOccurrences", "bindAnchors", "handlerQualificationSource", "queryRequiredAnchors", "mainSequence",
], runtime);
const expectedRustInventory = [
  "services/pay/src/db.rs",
  "services/pay/src/handlers/escrows.rs",
  "services/pay/src/handlers/intents.rs",
  "services/pay/src/handlers/mod.rs",
  "services/pay/src/handlers/pay_admin.rs",
  "services/pay/src/handlers/pay_history.rs",
  "services/pay/src/handlers/pay_links.rs",
  "services/pay/src/handlers/pay_webhooks.rs",
  "services/pay/src/lib.rs",
  "services/pay/src/main.rs",
  "services/pay/src/types.rs",
];
exact("Rust inventory", expectedRustInventory, runtime.rustInventory);
const rustRootPath = resolve(root, runtime.rustRoot);
if (!existsSync(rustRootPath) || lstatSync(rustRootPath).isSymbolicLink() || !statSync(rustRootPath).isDirectory()) fail("pay Rust root is missing or unsafe");
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
if (runtimeFindings.length !== 0) fail(`pay runtime Rust DDL scanner found ${runtimeFindings.length}, expected zero`);

exact("removed runtime snapshot", {
  commit: "526c3850fd4b1af336cb29a1a86f86b68be6c59f",
  file: "services/pay/src/db.rs",
  blob: "df3979ce0d92a0b8bbb7374d873b0cd75df71d26",
}, runtime.removedRuntimeSnapshot);
const expectedRemovedAnchors = [
  "CREATE TABLE IF NOT EXISTS pay_intents (",
  "CREATE TABLE IF NOT EXISTS escrows (",
  "CREATE TABLE IF NOT EXISTS pay_links (",
  "CREATE TABLE IF NOT EXISTS pay_webhook_events (",
  "CREATE INDEX IF NOT EXISTS idx_pay_intents_payer ON pay_intents (payer, status)",
  "CREATE INDEX IF NOT EXISTS idx_pay_intents_payee ON pay_intents (payee, status)",
  "CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows (status)",
  "CREATE INDEX IF NOT EXISTS idx_pay_links_slug ON pay_links (slug)",
  "CREATE INDEX IF NOT EXISTS idx_pay_links_intent ON pay_links (intent_id)",
  "CREATE INDEX IF NOT EXISTS idx_pay_webhook_intent ON pay_webhook_events (intent_id)",
];
exact("removed runtime anchors", expectedRemovedAnchors, runtime.removedAnchors);
if (!gitExists("cat-file", "-e", `${runtime.removedRuntimeSnapshot.commit}^{commit}`)) fail("removed runtime snapshot commit is unavailable");
const removedBlob = git("rev-parse", `${runtime.removedRuntimeSnapshot.commit}:${runtime.removedRuntimeSnapshot.file}`).toString().trim();
if (removedBlob !== runtime.removedRuntimeSnapshot.blob) fail("removed runtime snapshot blob drifted");
const removedSource = git("show", `${runtime.removedRuntimeSnapshot.commit}:${runtime.removedRuntimeSnapshot.file}`).toString();
if (!Array.isArray(runtime.removedAnchors) || runtime.removedAnchors.length !== 10) fail("exactly ten removed runtime DDL anchors are required");
for (const anchor of runtime.removedAnchors) {
  if (!removedSource.includes(anchor)) fail(`removed runtime snapshot is missing: ${anchor}`);
  for (const file of rustFiles) if (readFileSync(file, "utf8").includes(anchor)) fail(`removed runtime DDL returned: ${anchor}`);
}

const dbPath = regularRepoFile("services/pay/src/db.rs", "pay schema module");
const mainPath = regularRepoFile("services/pay/src/main.rs", "pay main");
const typesPath = regularRepoFile("services/pay/src/types.rs", "pay types");
const dbSource = readFileSync(dbPath, "utf8");
const mainSource = readFileSync(mainPath, "utf8");
const typesSource = readFileSync(typesPath, "utf8");
const queryMatch = dbSource.match(/const PAY_SCHEMA_COMPATIBILITY_QUERY: &str = r#"([\s\S]*?)"#;/);
if (!queryMatch) fail("read-only compatibility query constant is missing");
const query = queryMatch[1];
if (runtime.compatibilityQueryConstant !== "PAY_SCHEMA_COMPATIBILITY_QUERY" || runtime.compatibilityFunction !== "verify_schema_compatibility" || runtime.compatibilityQueryBytes !== 19212 || runtime.compatibilityQuerySha256 !== "a4ee6c4ad87e81e1a272d22ed22d3cd7d771f958e899d2b06e040a096f0abca7") fail("compatibility function boundary drifted");
if (Buffer.byteLength(query) !== runtime.compatibilityQueryBytes || sha256(query) !== runtime.compatibilityQuerySha256) fail("compatibility query bytes changed");
if (!/^\s*WITH\s+/i.test(query)) fail("compatibility query must begin with a read-only CTE");
if (/\b(?:INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|GRANT|REVOKE|CALL|DO)\b/i.test(query)) fail("compatibility query contains a mutation or command token");
const expectedQueryAnchors = [
  "WITH expected_tables (table_name) AS (", "COUNT(relation_oid) = 4", "relkind = \x27r\x27", "relpersistence = \x27p\x27", "NOT relispartition", "NOT relrowsecurity", "NOT relforcerowsecurity",
  "FROM pg_catalog.pg_inherits AS inheritance_record", "FROM pg_catalog.pg_policy AS policy_record", "(SELECT COUNT(*) FROM expected_columns) = 39", "(SELECT COUNT(*) FROM actual_columns) = 39",
  "pg_catalog.format_type(attribute_record.atttypid, attribute_record.atttypmod)", "pg_catalog.pg_get_expr(default_record.adbin, default_record.adrelid)", "AND COALESCE(",
  "COUNT(*) = 28", "attribute_collation = type_default_collation", "(SELECT COUNT(*) FROM expected_structural_constraints) = 5",
  "(SELECT COUNT(*) FROM structural_constraint_boundary) = 5", "cardinality(actual.conkey) = 1", "AND actual.confkey IS NULL",
  "current_setting(\x27server_version_num\x27)::integer >= 180000", "(SELECT COUNT(*) FROM expected_columns WHERE not_null) = 29",
  "(SELECT COUNT(*) FROM actual_not_null_constraints) = 29", "ELSE (SELECT COUNT(*) FROM actual_not_null_constraints) = 0",
  "(SELECT COUNT(*) FROM expected_indexes) = 11", "(SELECT COUNT(*) FROM actual_indexes) = 11", "actual.indnkeyatts = actual.indnatts", "actual.indpred IS NULL", "actual.indexprs IS NULL",
  "actual.access_method = \x27btree\x27", "actual.opclasses_compatible", "opclass_namespace.nspname = \x27pg_catalog\x27", "actual.collations_compatible", "actual.options_compatible",
  "to_regclass(\x27public.pay_intents\x27) IS NOT NULL", "to_regclass(\x27public.escrows\x27) IS NOT NULL", "to_regclass(\x27public.pay_links\x27) IS NOT NULL", "to_regclass(\x27public.pay_webhook_events\x27) IS NOT NULL",
];
exact("compatibility-query anchors", expectedQueryAnchors, runtime.queryRequiredAnchors);
for (const anchor of runtime.queryRequiredAnchors) if (typeof anchor !== "string" || !query.includes(anchor)) fail(`missing compatibility-query anchor: ${anchor}`);
if ((query.match(/to_regclass\(/g) ?? []).length !== 4) fail("compatibility query must resolve exactly four public relations");
const compatibilityStart = dbSource.indexOf("pub async fn verify_schema_compatibility(");
const compatibilityEnd = dbSource.indexOf("/// Compute the 0.3%", compatibilityStart);
if (compatibilityStart < 0) fail("compatibility function is missing");
const compatibilityBody = dbSource.slice(compatibilityStart, compatibilityEnd < 0 ? dbSource.length : compatibilityEnd);
for (const anchor of ["sqlx::query_scalar::<_, bool>(PAY_SCHEMA_COMPATIBILITY_QUERY)", ".fetch_one(db)", "PaySchemaError::Incompatible"]) if (!compatibilityBody.includes(anchor)) fail(`compatibility function is missing ${anchor}`);
if (compatibilityBody.includes(".execute(")) fail("compatibility function must remain read-only");

const relationNames = ["pay_intents", "escrows", "pay_links", "pay_webhook_events"];
const relationCounts = {};
const allRustSource = rustFiles.map((file) => readFileSync(file, "utf8")).join("\n");
exact("qualified relation occurrence contract", { "public.pay_intents": 29, "public.escrows": 21, "public.pay_links": 3, "public.pay_webhook_events": 1 }, runtime.qualifiedRelationOccurrences);
for (const relation of relationNames) {
  const qualified = allRustSource.match(new RegExp(`\\b(?:INSERT\\s+INTO|UPDATE|DELETE\\s+FROM|FROM|JOIN)\\s+public\\.${relation}\\b`, "gi")) ?? [];
  const unqualified = allRustSource.match(new RegExp(`\\b(?:INSERT\\s+INTO|UPDATE|DELETE\\s+FROM|FROM|JOIN)\\s+${relation}\\b`, "gi")) ?? [];
  const key = `public.${relation}`;
  if (qualified.length !== runtime.qualifiedRelationOccurrences[key]) fail(`${key} runtime relation count is ${qualified.length}, expected ${runtime.qualifiedRelationOccurrences[key]}`);
  if (unqualified.length !== 0) fail(`runtime SQL contains ${unqualified.length} unqualified ${relation} relation reference(s)`);
  relationCounts[key] = qualified.length;
}

const qualification = runtime.handlerQualificationSource;
if (!qualification || qualification.commit !== runtime.removedRuntimeSnapshot.commit) fail("handler qualification source drifted");
exactKeys("handler qualification source", ["commit", "comparison", "files"], qualification);
if (qualification.comparison !== "Rust tokens excluding whitespace and rustfmt trailing commas must equal the pinned source after removing public qualification before the four owned relation names") fail("handler qualification source drifted");
if (!Array.isArray(qualification.files) || qualification.files.length !== 6) fail("exactly six qualification-only handler files are required");
exact("qualification-only handler pins", [
  { path: "services/pay/src/handlers/intents.rs", blob: "ba69f4bf43c4a7c7cfa30adf47ef973724d4b3d2" },
  { path: "services/pay/src/handlers/escrows.rs", blob: "b3f07bc3d0429bc9463bd2db8432cc667b461d9a" },
  { path: "services/pay/src/handlers/pay_admin.rs", blob: "fe45fd1256d2462ef74a161ef908113c6b33f353" },
  { path: "services/pay/src/handlers/pay_history.rs", blob: "91d36a2302e3ee6318bbf2fdbec46b97c271a3a4" },
  { path: "services/pay/src/handlers/pay_links.rs", blob: "e7a22fe355a070f18bbf8cc84020c6cd661418c3" },
  { path: "services/pay/src/handlers/pay_webhooks.rs", blob: "d76bc79af90c8a6167ddc761e59dc513159d434c" },
], qualification.files);
for (let index = 0; index < qualification.files.length; index += 1) {
  exactKeys(`handler qualification file ${index + 1}`, ["path", "blob"], qualification.files[index]);
}
const normalizeRustTokens = (value) => value
  .replace(/\bpublic\.(?=(?:pay_intents|escrows|pay_links|pay_webhook_events)\b)/g, "")
  .replace(/,\s*(?=[)\]}])/g, "")
  .replace(/\s+/g, "");
for (const item of qualification.files) {
  safeRelative(item.path, "handler qualification file");
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.path}: invalid pinned handler blob`);
  const actualBlob = git("rev-parse", `${qualification.commit}:${item.path}`).toString().trim();
  if (actualBlob !== item.blob) fail(`${item.path}: pinned handler blob drifted`);
  const before = git("show", `${qualification.commit}:${item.path}`).toString();
  const after = readFileSync(regularRepoFile(item.path, "handler qualification file"), "utf8");
  if (normalizeRustTokens(before) !== normalizeRustTokens(after)) fail(`${item.path}: changes exceed public relation qualification and rustfmt whitespace`);
}

exact("SQLx bind anchors", [
  ".bind(state.chain_id.to_string())", ".bind(&req.amount)", ".bind(&req.description)", ".bind(expires_at)", ".bind(now)",
  ".bind(&intent.chain_id)", ".bind(&intent.amount)", ".bind(max_uses)", ".bind(&event.intent_id)", ".bind(&event.escrow_id)", ".bind(serde_json::json!({",
], runtime.bindAnchors);
for (const anchor of runtime.bindAnchors) if (!allRustSource.includes(anchor)) fail(`missing SQLx bind anchor: ${anchor}`);
exact("main sequence", [
  "sqlx::PgPool::connect(&args.database_url)",
  "verify_schema_compatibility(&db)",
  "build_provider(args.chain_id)",
  "let app = Router::new()",
  "tokio::net::TcpListener::bind(addr)",
  "axum::serve(listener, app)",
], runtime.mainSequence);
let previous = -1;
for (const anchor of runtime.mainSequence) {
  const index = mainSource.indexOf(anchor);
  if (index < 0) fail(`main sequence is missing: ${anchor}`);
  if (index <= previous) fail(`main sequence is out of order: ${anchor}`);
  previous = index;
}

const failClosed = contract.failClosedBoundary;
if (!failClosed || failClosed.file !== "services/pay/src/lib.rs" || failClosed.unsafeRoutesReachable !== false) fail("fail-closed boundary drifted");
exact("fail-closed policy", {
  file: "services/pay/src/lib.rs",
  unsafeMutationPolicy: "UnsafeFinancialMutation",
  internalIdentityPolicy: "InternalIdentityUnavailable",
  adminManagePolicy: "UnsafePaymentsManage",
  requiredAnchors: [
    "AccessPolicy::UnsafePaymentsManage => {", "return StatusCode::NOT_FOUND.into_response();", "AccessPolicy::UnsafeFinancialMutation",
    "| AccessPolicy::InternalIdentityUnavailable", "| AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),",
  ],
  unsafeRoutesReachable: false,
}, failClosed);
const authSource = readFileSync(regularRepoFile(failClosed.file, "pay authorization boundary"), "utf8");
if (!Array.isArray(failClosed.requiredAnchors) || failClosed.requiredAnchors.length !== 5) fail("five fail-closed anchors are required");
for (const anchor of failClosed.requiredAnchors) if (!authSource.includes(anchor)) fail(`fail-closed authorization anchor is missing: ${anchor}`);
if (!mainSource.includes("let app = protect_router(app, verifier);")) fail("pay router is not protected before listener binding");

const migrationRoot = contract.migrationRoot;
if (!migrationRoot || migrationRoot.path !== "services/pay/migrations" || migrationRoot.runner !== null || migrationRoot.transactionOwner !== "future-reviewed-runner") fail("migration-root boundary drifted");
exactKeys("migration root", ["path", "runner", "transactionOwner", "orderedMigrations"], migrationRoot);
safeRelative(migrationRoot.path, "migration root");
const migrationRootPath = resolve(root, migrationRoot.path);
if (!existsSync(migrationRootPath) || lstatSync(migrationRootPath).isSymbolicLink() || !statSync(migrationRootPath).isDirectory()) fail("migration root must be a real directory");
exact("migration-root file inventory", ["20260722060000_create_pay_store.sql"], readdirSync(migrationRootPath).sort());
if (!Array.isArray(migrationRoot.orderedMigrations) || migrationRoot.orderedMigrations.length !== 1) fail("exactly one ordered pay migration is required");
const migration = migrationRoot.orderedMigrations[0];
exactKeys("migration metadata", [
  "version", "path", "bytes", "sha256", "tableStatements", "indexStatements", "transactionControlStatements",
], migration);
exact("migration metadata", {
  version: "20260722060000",
  path: "services/pay/migrations/20260722060000_create_pay_store.sql",
  bytes: 2150,
  sha256: "b048fdefebb1c091a0d86ddcf9876a9531519f8ec4d959e863b51067826be83b",
  tableStatements: 4,
  indexStatements: 6,
  transactionControlStatements: 0,
}, migration);
if (!migration.path.split("/").at(-1).startsWith(`${migration.version}_`)) fail("migration filename/version order is inconsistent");
const migrationBytes = readFileSync(regularRepoFile(migration.path, "pay migration"));
const migrationSql = migrationBytes.toString("utf8");
if (migrationBytes.byteLength !== migration.bytes || sha256(migrationBytes) !== migration.sha256) fail("pay migration bytes changed");
if ((migrationSql.match(/;/g) ?? []).length !== 10) fail("pay migration must contain exactly ten statements");
if ((migrationSql.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.(?:pay_intents|escrows|pay_links|pay_webhook_events)\s*\(/gi) ?? []).length !== 4) fail("pay migration must contain four guarded public table creations");
if ((migrationSql.match(/\bCREATE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+idx_[a-z_]+\s+ON\s+public\.(?:pay_intents|escrows|pay_links|pay_webhook_events)\s*\(/gi) ?? []).length !== 6) fail("pay migration must contain six guarded indexes on public relations");
if (/\b(?:DROP|TRUNCATE|DELETE\s+FROM|ALTER|INSERT\s+INTO|UPDATE|MERGE)\b/i.test(migrationSql)) fail("pay migration contains a destructive, data-mutation, or alteration token");
if (/\bCREATE\s+(?:SCHEMA|EXTENSION|DATABASE|TYPE|VIEW|MATERIALIZED\s+VIEW)\b/i.test(migrationSql)) fail("pay migration contains an out-of-scope creation");
if (/\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK|SAVEPOINT)\b/i.test(migrationSql)) fail("transaction control belongs to the future reviewed runner");

if (!Array.isArray(contract.requiredTables) || contract.requiredTables.length !== 4) fail("exactly four pay tables are required");
const expectedTables = [["pay_intents", 13], ["escrows", 13], ["pay_links", 7], ["pay_webhook_events", 6]];
const expectedColumnNames = [
  ["id", "chain_id", "payer", "payee", "amount", "token_address", "status", "escrow_id", "tx_hash", "description", "expires_at", "created_at", "updated_at"],
  ["id", "chain_id", "payer", "payee", "amount", "token_address", "fee_amount", "status", "on_chain_id", "tx_hash", "dispute_reason", "created_at", "updated_at"],
  ["id", "slug", "intent_id", "max_uses", "current_uses", "expires_at", "created_at"],
  ["event_id", "intent_id", "escrow_id", "event_type", "payload", "received_at"],
];
let requiredColumnCount = 0;
for (let tableIndex = 0; tableIndex < contract.requiredTables.length; tableIndex += 1) {
  const table = contract.requiredTables[tableIndex];
  const [expectedName, expectedColumns] = expectedTables[tableIndex];
  exactKeys(`${expectedName} required table`, ["name", "columns"], table);
  if (table.name !== expectedName || !Array.isArray(table.columns) || table.columns.length !== expectedColumns) fail(`${expectedName}: required table contract drifted`);
  exact(`${expectedName} column names`, expectedColumnNames[tableIndex], table.columns.map((column) => column.name));
  const tableMatch = migrationSql.match(new RegExp(`CREATE TABLE IF NOT EXISTS public\\.${expectedName} \\(([\\s\\S]*?)\\n\\);`));
  if (!tableMatch) fail(`${expectedName}: migration table body is malformed`);
  const expectedDefinitions = [];
  for (let index = 0; index < table.columns.length; index += 1) {
    const column = table.columns[index];
    exactKeys(`${expectedName} column descriptor ${index + 1}`, ["ordinal", "name", "sqlAnchor"], column);
    if (column.ordinal !== index + 1 || typeof column.name !== "string" || typeof column.sqlAnchor !== "string") fail(`${expectedName}: column ${index + 1} drifted`);
    if ((tableMatch[1].split(column.sqlAnchor).length - 1) !== 1) fail(`${expectedName}.${column.name}: migration anchor must occur exactly once in its table`);
    expectedDefinitions.push(column.sqlAnchor.replace(/,$/, ""));
    requiredColumnCount += 1;
  }
  const actualDefinitions = tableMatch[1].split(/\r?\n/).map((line) => line.trim().replace(/,$/, "")).filter(Boolean);
  exact(`${expectedName} column definitions`, expectedDefinitions, actualDefinitions);
}
if (requiredColumnCount !== 39) fail(`required column count is ${requiredColumnCount}, expected 39`);

exact("constraint and index semantics", {
  primaryKeys: ["pay_intents.id", "escrows.id", "pay_links.id", "pay_webhook_events.event_id"],
  uniqueKeys: ["pay_links.slug"],
  foreignKeys: [],
  structuralConstraints: 5,
  postgres18NotNullConstraints: 29,
  postgres18CatalogConstraints: 34,
  pre18NotNullConstraints: 0,
  totalIndexes: 11,
  constraintBackedIndexes: 5,
  standaloneIndexes: 6,
  partialIndexes: 0,
  expressionIndexes: 0,
  includeIndexes: 0,
  inheritedOrPartitionedRelations: 0,
  rowLevelSecurityRelations: 0,
  policies: 0,
  defaultTypeCollationColumns: 28,
  accessMethod: "btree",
  opclass: "text_ops",
  opclassNamespace: "pg_catalog",
  collationPolicy: "all-varchar-and-text-columns-use-type-default-and-every-index-key-matches-its-column",
}, contract.constraintAndIndexSemantics);

const parseStruct = (name) => {
  const match = typesSource.match(new RegExp(`pub struct ${name} \\{([\\s\\S]*?)\\n\\}`));
  if (!match) fail(`${name}: Rust model is missing`);
  return match[1].split(/\r?\n/).map((line) => line.trim().match(/^pub ([a-z_]+): (.+),$/)).filter(Boolean).map((match) => `${match[1]}:${match[2]}`);
};
exact("PayIntent Rust model", [
  "id:String", "chain_id:String", "payer:String", "payee:String", "amount:String", "token_address:String", "status:String",
  "escrow_id:Option<String>", "tx_hash:Option<String>", "description:Option<String>", "expires_at:Option<chrono::DateTime<chrono::Utc>>",
  "created_at:chrono::DateTime<chrono::Utc>", "updated_at:chrono::DateTime<chrono::Utc>",
], parseStruct("PayIntent"));
exact("EscrowRecord Rust model", [
  "id:String", "chain_id:String", "payer:String", "payee:String", "amount:String", "token_address:String", "fee_amount:String", "status:String",
  "on_chain_id:Option<String>", "tx_hash:Option<String>", "dispute_reason:Option<String>",
  "created_at:chrono::DateTime<chrono::Utc>", "updated_at:chrono::DateTime<chrono::Utc>",
], parseStruct("EscrowRecord"));
exact("PayLink Rust model", [
  "id:String", "slug:String", "intent_id:String", "max_uses:i32", "current_uses:i32",
  "expires_at:Option<chrono::DateTime<chrono::Utc>>", "created_at:chrono::DateTime<chrono::Utc>",
], parseStruct("PayLink"));
const modelContract = contract.rustModelContract;
exact("Rust model contract", {
  file: "services/pay/src/types.rs",
  requiredNonOptionalDefaultColumns: [
    "PayIntent.status", "PayIntent.created_at", "PayIntent.updated_at", "EscrowRecord.fee_amount", "EscrowRecord.status",
    "EscrowRecord.created_at", "EscrowRecord.updated_at", "PayLink.max_uses", "PayLink.current_uses", "PayLink.created_at",
  ],
  requiredOptionalColumns: [
    "PayIntent.escrow_id", "PayIntent.tx_hash", "PayIntent.description", "PayIntent.expires_at", "EscrowRecord.on_chain_id",
    "EscrowRecord.tx_hash", "EscrowRecord.dispute_reason", "PayLink.expires_at",
  ],
  chainIdDecimalCapacity: 20,
}, modelContract);

const drift = contract.freshSchemaDrift;
exact("fresh-schema drift boundary", {
  guardedMigrationUpgradesPreexistingTables: false,
  driftItems: 13,
  notNullAdditions: [
    "pay_intents.status", "pay_intents.created_at", "pay_intents.updated_at", "escrows.fee_amount", "escrows.status",
    "escrows.created_at", "escrows.updated_at", "pay_links.max_uses", "pay_links.current_uses", "pay_links.created_at",
    "pay_webhook_events.received_at",
  ],
  lengthWidenings: ["pay_intents.chain_id:10->20", "escrows.chain_id:10->20"],
  stopCategories: ["database-authority", "baseline-adoption", "populated-upgrade"],
}, drift);
exact("isolated runtime delta", {
  payRuntimeDdlFindings: { before: 10, after: 0, delta: -10 },
  migrationSqlFiles: { delta: 1 },
  status: "isolated-package-only-central-a3-3-and-migration-safety-rebaseline-owned-elsewhere",
}, contract.isolatedRuntimeDelta);
exact("isolated PostgreSQL 18 evidence", {
  version: "18.4",
  distribution: "Homebrew",
  cluster: "ephemeral-local-initdb-no-production-data",
  freshMigrationProbe: true,
  catalogConstraintCounts: { total: 34, notNull: 29, structural: 5 },
  adversarialCases: { policyWithRlsDisabled: false, nonDefaultColumnCollation: false, foreignNamespaceTextOps: false },
  restoredCleanProbe: true,
  cleanupConfirmed: true,
  readinessEvidence: false,
}, contract.isolatedPostgres18Evidence);
exact("non-claims", [
  "A3.13 does not choose among the canonical backend payment schema, epsx_payment, epsx_pay, or any epsx_payments environment database.",
  "A3.13 does not adopt, alter, backfill, reconcile, read, or write any existing payment, escrow, link, webhook, subscription, credit, or entitlement row.",
  "A3.13 does not add financial idempotency, exact-money constraints, chain-event identity, receipt finality, state-machine, audit, inbox, or outbox proof.",
  "A3.13 does not make any financial, admin mutation, deposit-confirmation, or webhook route reachable; the existing uniform 404 boundary remains required.",
  "A3.13 does not authorize a migration runner, deployed or candidate database connection, provider call, chain access, deployment, production environment, or canonical inventory rebaseline; only an isolated disposable PostgreSQL 18 cluster was exercised.",
], contract.nonClaims);
exact("residual blocker inventory", [
  { id: "B01", category: "migration-runner", status: "blocked", summary: "No reviewed runner discovers, orders, records, or executes the pay candidate migration root." },
  { id: "B02", category: "database-authority", status: "blocked", summary: "The payment write authority and the relationship among all candidate database names remain deliberately unresolved." },
  { id: "B03", category: "baseline-adoption", status: "blocked", summary: "No safe version-ledger adoption flow for an already matching deployed pay candidate schema has been designed or proven." },
  { id: "B04", category: "populated-upgrade", status: "blocked", summary: "No populated source upgrade has proven the two length widenings and eleven NOT NULL additions without row loss." },
  { id: "B05", category: "reconciliation", status: "blocked", summary: "No payment, escrow, link, webhook, constraint, index, or competing-authority reconciliation has executed." },
  { id: "B06", category: "concurrent-startup", status: "blocked", summary: "No concurrent migration and pay-service startup ordering test has run against PostgreSQL." },
  { id: "B07", category: "live-database", status: "blocked", summary: "The migration and probe passed only in a disposable empty PostgreSQL 18 cluster; no candidate, deployed, populated, adoption, or upgrade database proof exists." },
  { id: "B08", category: "financial-durability", status: "blocked", summary: "The candidate schema still lacks the idempotency, chain identity, state-machine, audit, inbox, outbox, and finality proof required by A6." },
], contract.blockers);
for (const category of drift.stopCategories) if (!contract.blockers.some((blocker) => blocker.category === category && blocker.status === "blocked")) fail(`${category}: fresh-schema STOP is not blocked`);

const output = {
  schemaVersion: 1,
  contractId: contract.contractId,
  service: contract.scope.service,
  status: contract.scope.status,
  authority: { decision: authority.decision, candidateDatabaseNames: authority.candidateDatabaseNames.length, sourceCommit: authority.developmentSourceCommit, evidence: authority.developmentEvidence.length },
  runtimeRust: { files: rustFiles.length, ddlFindings: runtimeFindings.length, expectedDelta: -10, qualifiedRelations: relationCounts, bindAnchors: runtime.bindAnchors.length },
  migrationRoot: { path: migrationRoot.path, migrations: 1, statements: 10, pinnedBytes: migration.bytes, sha256: migration.sha256, runner: null },
  schema: {
    tables: contract.requiredTables.map((table) => ({ name: table.name, columns: table.columns.length })),
    columns: requiredColumnCount,
    structuralConstraints: contract.constraintAndIndexSemantics.structuralConstraints,
    postgres18NotNullConstraints: contract.constraintAndIndexSemantics.postgres18NotNullConstraints,
    postgres18CatalogConstraints: contract.constraintAndIndexSemantics.postgres18CatalogConstraints,
    primaryKeys: contract.constraintAndIndexSemantics.primaryKeys.length,
    uniqueKeys: contract.constraintAndIndexSemantics.uniqueKeys.length,
    foreignKeys: contract.constraintAndIndexSemantics.foreignKeys.length,
    indexes: contract.constraintAndIndexSemantics.totalIndexes,
    standaloneIndexes: contract.constraintAndIndexSemantics.standaloneIndexes,
    partialIndexes: contract.constraintAndIndexSemantics.partialIndexes,
    expressionIndexes: contract.constraintAndIndexSemantics.expressionIndexes,
    includeIndexes: contract.constraintAndIndexSemantics.includeIndexes,
    policies: contract.constraintAndIndexSemantics.policies,
    defaultTypeCollationColumns: contract.constraintAndIndexSemantics.defaultTypeCollationColumns,
    freshSchemaDriftItems: drift.driftItems,
  },
  isolatedPostgres18: contract.isolatedPostgres18Evidence,
  failClosed: { unsafeRoutesReachable: failClosed.unsafeRoutesReachable },
  blockers: contract.blockers.map(({ id, category, status }) => ({ id, category, status })),
  productionReady: false,
  readinessExit: 3,
};
process.stdout.write(`${JSON.stringify(output, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s\n' "$report"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "a3-13-pay-schema-boundary: PASS — pay runtime DDL 10→0; one 10-statement transaction-body migration is byte-pinned"
  echo "a3-13-pay-schema-boundary: PASS — 39 columns, five structural constraints, PG18 29-NOT-NULL/34-total constraints, zero FKs, and 11 indexes are checked"
  echo "a3-13-pay-schema-boundary: PASS — policies, partial/expression/INCLUDE, inheritance/partition, RLS, default type collation, pg_catalog text_ops, and search_path drift are rejected"
  echo "a3-13-pay-schema-boundary: PASS — 54 runtime relation references are public-qualified and six handlers differ only by qualification/rustfmt whitespace"
  echo "a3-13-pay-schema-boundary: PASS — unsafe financial, admin mutation, deposit-confirmation, and webhook routes remain fail-closed 404"
  echo "a3-13-pay-schema-boundary: LIMIT — payment database authority remains unresolved; isolated PG18 evidence is not cutover, adoption, populated upgrade, deployed database, provider, or chain proof"
  echo "a3-13-pay-schema-boundary: LIMIT — guarded fresh-schema DDL cannot apply eleven NOT NULL additions or two chain-id widenings to existing tables"
  exit 0
fi

echo "a3-13-pay-schema-boundary: STOP — eight residual A3.13 blockers remain; readiness is intentionally exit 3" >&2
echo "a3-13-pay-schema-boundary: LIMIT — static integrity is not database authority, migration, reconciliation, finality, or financial execution proof" >&2
exit 3
