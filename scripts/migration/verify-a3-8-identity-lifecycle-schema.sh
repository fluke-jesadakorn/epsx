#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/a3-8-identity-lifecycle-schema.json"
mode=""

die() {
  echo "a3-8-identity-lifecycle-schema: ERROR: $*" >&2
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

for name in DATABASE_URL IDENTITY_DATABASE_URL REDIS_URL PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts a database or Redis"
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
  console.error(`a3-8-identity-lifecycle-schema: ERROR: ${message}`);
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
const git = (args, label) => {
  const result = Bun.spawnSync(["git", "-C", root, ...args], { stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`${label}: ${result.stderr.toString().trim() || "git command failed"}`);
  return result.stdout.toString();
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

if (contract.schemaVersion !== 1 || contract.contractId !== "A3.8-identity-lifecycle-schema") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "offline-static-additive-identity-lifecycle-schema-only") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
exact("scope", {
  service: "services/identity",
  database: "epsx_identity",
  schema: "public",
  status: "partial-schema-only",
  routesEnabled: false,
}, contract.scope);
if (!contract.safety || typeof contract.safety.readinessMeaning !== "string") fail("safety boundary is required");
for (const [key, value] of Object.entries(contract.safety)) {
  if (key !== "readinessMeaning" && value !== false) fail(`${key} must remain false`);
}

const expectedAuthorities = {
  historicalAudit: {
    path: "docs/migration/contracts/a2-3-identity-authorization.json",
    sha256: "7b71f7adda0767bea01aabb8d9ff8dd0a0ed6d3c9982375fce16c78ae74b3027",
    contractId: "A2.3h-identity-direct-service-authorization-audit",
  },
  failClosedRuntime: {
    path: "docs/migration/contracts/a2-3-identity-fail-closed-runtime.json",
    sha256: "5cf05d13c06c11d0ad785b4b1329356d0e8a793f54752d4b12c43b22d2a352a2",
    contractId: "A2.3i-identity-direct-service-fail-closed-runtime",
  },
};
if (!contract.authority || contract.authority.interpretation !== "A3.8 may define additive persistence shape only. A2.3i continues to hide every identity lifecycle route.") fail("authority interpretation drifted");
for (const [name, expected] of Object.entries(expectedAuthorities)) {
  exact(`${name} authority`, expected, contract.authority[name]);
  const authorityPath = regularRepoFile(expected.path, `${name} authority`);
  const bytes = readFileSync(authorityPath);
  if (sha256(bytes) !== expected.sha256) fail(`${name} authority bytes changed`);
  let parsed;
  try { parsed = JSON.parse(bytes.toString("utf8")); } catch (error) { fail(`${name} authority is invalid JSON: ${error.message}`); }
  if (parsed.contractId !== expected.contractId) fail(`${name} authority contractId changed`);
}

const source = contract.sourceEvidence;
if (!source || source.developmentCommit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || source.developmentRefLabel !== "origin/development" || source.historicalCandidateCommit !== "0cdd7ba1967d52e299000b7290873cd4d19dfd09" || source.failClosedCommit !== "f5aa5b393cc0856a1ab2cba42746daa05b8c25c1") fail("source commit boundary drifted");
const expectedEvidence = [
  [source.developmentCommit, "apps/backend/migrations/core/00000000000001_consolidated_baseline_v6/up.sql", "3cf683bc589f7252fd1be64ab62b69958fc292de", ["CREATE TABLE wallet_users (", "CREATE TABLE web3_auth_nonces (", "CREATE TABLE openid_refresh_tokens ("]],
  [source.developmentCommit, "apps/backend/migrations/core/20260427000000_allow_multiple_web3_nonces/up.sql", "c970600c13978919ab6cc88fc9702511f9f50e1a", ["ADD CONSTRAINT web3_auth_nonces_pkey PRIMARY KEY (nonce)", "CREATE INDEX IF NOT EXISTS idx_web3_auth_nonces_wallet_address"]],
  [source.developmentCommit, "apps/backend/src/auth/auth_service.rs", "1a8414c6996b74eb8c200911309431955186cf3c", ["self.cleanup_nonce(&wallet_address, &request.nonce).await?;", "Signature verification failed"]],
  [source.developmentCommit, "apps/backend/src/auth/challenge_service.rs", "a39f95fdd498d09e69638182fa47fb5f2f607f1b", ["chain_id: std::env::var(\"CHAIN_ID\")", ".and_then(|v| v.parse::<u64>().ok())"]],
  [source.developmentCommit, "apps/backend/src/auth/token_service.rs", "fb88b092bde0de2e558fb463a135f66cf2eb0f39", ["Atomically consume a refresh token and create its replacement.", "openid_refresh_tokens::token_id.eq(&old_token)", "openid_refresh_tokens::is_revoked.eq(true)"]],
  [source.developmentCommit, "apps/backend/migrations/_archive/core/20251126010000_consolidated_schema/up.sql", "e4af122b12a9f775d528ad0b62fecb34e8b6dfbb", ["CREATE TABLE sessions (", "access_token TEXT NOT NULL,", "refresh_token TEXT,"]],
  [source.historicalCandidateCommit, "services/identity/src/main.rs", "012e1321aba54280d5b0cdaa81ee133a28ac3dfc", ["CREATE TABLE IF NOT EXISTS users (", "let stored: Option<String> = conn.get(&key).await", "let _: i64 = conn.del(&key).await", "verify_token(&req.refresh_token)", "let chain_id_num: u64 = req.chain_id.parse()"]],
  [source.failClosedCommit, "services/identity/src/lib.rs", "b6a8f64a7afb78b149455e898184ad8042fd33b3", [".route(\"/api/v1/identity/auth/challenge\", post(not_found))", ".route(\"/api/v1/identity/auth/siwe\", post(not_found))", ".route(\"/api/v1/identity/auth/refresh\", post(not_found))", "AccessPolicy::UnsafeLifecycle | AccessPolicy::Blocked"]],
  [source.failClosedCommit, "services/identity/src/main.rs", "c8363672f6a50202e9ed4dd7046fb772e67e271a", ["Identity persistence, SIWE nonce consumption, refresh rotation", "let app = build_router(verifier);", "identity service listening with lifecycle routes disabled"]],
].map(([commit, path, blob, anchors]) => ({ commit, path, blob, anchors }));
exact("immutable source evidence", expectedEvidence, source.items);
for (const [index, item] of source.items.entries()) {
  safeRelative(item.path, `source evidence ${index + 1}`);
  const commit = git(["rev-parse", "--verify", `${item.commit}^{commit}`], `source evidence ${index + 1} commit`).trim();
  if (commit !== item.commit) fail(`source evidence ${index + 1} commit is not exact`);
  const blob = git(["rev-parse", `${item.commit}:${item.path}`], `source evidence ${index + 1} blob`).trim();
  if (blob !== item.blob) fail(`source evidence ${index + 1} blob changed`);
  const content = git(["show", `${item.commit}:${item.path}`], `source evidence ${index + 1} content`);
  for (const anchor of item.anchors) if ((content.split(anchor).length - 1) < 1) fail(`source evidence ${index + 1} missing anchor: ${anchor}`);
}

if (source.schemaHistoryMatch !== "identity-table-create-alter-drop-lexical-inventory" || !Array.isArray(source.schemaHistoryPaths) || source.schemaHistoryPaths.length !== 22) fail("schema-history inventory boundary drifted");
const treePaths = git(["ls-tree", "-r", "--name-only", source.developmentCommit], "development tree inventory").trim().split("\n").filter(Boolean);
const historyPattern = /\b(?:CREATE|ALTER|DROP)\s+TABLE\s+(?:IF\s+(?:NOT\s+)?EXISTS\s+)?(?:public\.)?(?:users|wallet_users|web3_auth_nonces|openid_refresh_tokens|sessions)\b/i;
const observedHistory = [];
for (const relative of treePaths) {
  if (!relative.endsWith(".sql")) continue;
  const content = git(["show", `${source.developmentCommit}:${relative}`], `schema-history ${relative}`);
  if (historyPattern.test(content)) observedHistory.push(relative);
}
observedHistory.sort();
exact("schema-history inventory", source.schemaHistoryPaths, observedHistory);

const currentLib = readFileSync(regularRepoFile("services/identity/src/lib.rs", "current identity library"), "utf8");
const currentMain = readFileSync(regularRepoFile("services/identity/src/main.rs", "current identity main"), "utf8");
for (const anchor of [
  ".route(\"/api/v1/identity/auth/challenge\", post(not_found))",
  ".route(\"/api/v1/identity/auth/siwe\", post(not_found))",
  ".route(\"/api/v1/identity/auth/refresh\", post(not_found))",
  "AccessPolicy::UnsafeLifecycle | AccessPolicy::Blocked",
]) if (!currentLib.includes(anchor)) fail(`current fail-closed runtime is missing: ${anchor}`);
for (const anchor of ["let app = build_router(verifier);", "identity service listening with lifecycle routes disabled"]) if (!currentMain.includes(anchor)) fail(`current fail-closed main is missing: ${anchor}`);
if (/\b(?:sqlx|deadpool_redis|RedisPool|PgPool|Migrator|migrate!)\b/.test(`${currentLib}\n${currentMain}`)) fail("identity runtime persistence was integrated outside A3.8 scope");

const migrationRoot = contract.migrationRoot;
if (!migrationRoot || migrationRoot.path !== "services/identity/migrations" || migrationRoot.runner !== null || migrationRoot.transactionOwner !== "future-reviewed-runner") fail("migration-root boundary drifted");
safeRelative(migrationRoot.path, "migration root");
const migrationRootPath = resolve(root, migrationRoot.path);
if (!existsSync(migrationRootPath) || lstatSync(migrationRootPath).isSymbolicLink() || !statSync(migrationRootPath).isDirectory()) fail("migration root must be a real directory");
exact("migration-root file inventory", ["20260722000000_create_identity_lifecycle.sql"], readdirSync(migrationRootPath).sort());
if (!Array.isArray(migrationRoot.orderedMigrations) || migrationRoot.orderedMigrations.length !== 1) fail("exactly one ordered identity migration is required");
const migration = migrationRoot.orderedMigrations[0];
exact("ordered migration pin", {
  version: "20260722000000",
  path: "services/identity/migrations/20260722000000_create_identity_lifecycle.sql",
  bytes: 6417,
  sha256: "6415b7621d424b639e1f4692c924d4f42539fbf810d774024bc8bbbd152d008c",
  statementCount: 10,
  guardedTableCount: 4,
  guardedIndexCount: 6,
}, migration);
if (!migration.path.split("/").at(-1).startsWith(`${migration.version}_`)) fail("migration filename/version order is inconsistent");
const migrationBytes = readFileSync(regularRepoFile(migration.path, "identity lifecycle migration"));
const sql = migrationBytes.toString("utf8");
if (migrationBytes.byteLength !== migration.bytes || sha256(migrationBytes) !== migration.sha256) fail("identity lifecycle migration bytes changed");

const semicolonCount = (sql.match(/;/g) ?? []).length;
const guardedTables = sql.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\.identity_[a-z_]+\s*\(/gi) ?? [];
const guardedIndexes = sql.match(/\bCREATE\s+(?:UNIQUE\s+)?INDEX\s+IF\s+NOT\s+EXISTS\s+identity_[a-z_]+/gi) ?? [];
if (semicolonCount !== migration.statementCount || guardedTables.length !== migration.guardedTableCount || guardedIndexes.length !== migration.guardedIndexCount) fail("migration statement or guarded object count changed");
if (/\bCREATE\s+TABLE\b(?!\s+IF\s+NOT\s+EXISTS)/i.test(sql) || /\bCREATE\s+(?:UNIQUE\s+)?INDEX\b(?!\s+IF\s+NOT\s+EXISTS)/i.test(sql)) fail("every identity table and index creation must remain guarded");
if (/\bCREATE\s+(?:SCHEMA|EXTENSION|DATABASE|TYPE|VIEW|MATERIALIZED\s+VIEW|FUNCTION|TRIGGER)\b/i.test(sql)) fail("migration contains out-of-scope object creation");
const commands = sql.replace(/\bON\s+DELETE\s+RESTRICT\b/gi, "");
if (/\b(?:ALTER|DROP|TRUNCATE|DELETE|INSERT|UPDATE|MERGE|CASCADE|GRANT|REVOKE|CALL|DO)\b/i.test(commands)) fail("migration contains a destructive, data-mutation, or alteration command");
if (/\b(?:BEGIN|START\s+TRANSACTION|COMMIT|ROLLBACK)\b/i.test(sql)) fail("transaction control belongs to the future reviewed runner");

const design = contract.schemaDesign;
const expectedTables = ["public.identity_users", "public.identity_siwe_challenges", "public.identity_refresh_families", "public.identity_refresh_sessions"];
if (!design) fail("schema design is missing");
exact("schema tables", expectedTables, design.tables);
const observedTables = guardedTables.map((entry) => entry.match(/public\.identity_[a-z_]+/i)[0].toLowerCase());
exact("guarded table order", expectedTables, observedTables);
const expectedAnchors = [
  "CREATE TABLE IF NOT EXISTS public.identity_users (",
  "user_id UUID PRIMARY KEY,",
  "wallet_address VARCHAR(42) NOT NULL,",
  "CONSTRAINT identity_users_wallet_address_unique UNIQUE (wallet_address)",
  "wallet_address ~ \u0027^0x[0-9a-f]{40}$\u0027",
  "CREATE TABLE IF NOT EXISTS public.identity_siwe_challenges (",
  "challenge_id UUID PRIMARY KEY,",
  "client_id VARCHAR(64) NOT NULL,",
  "CONSTRAINT identity_siwe_challenges_client_id CHECK (",
  "client_id IN (\u0027epsx-frontend\u0027, \u0027epsx-admin\u0027)",
  "chain_id VARCHAR(20) NOT NULL,",
  "WHEN chain_id ~ \u0027^(0|[1-9][0-9]{0,19})$\u0027",
  "THEN chain_id::NUMERIC <= 18446744073709551615",
  "nonce_hash BYTEA NOT NULL,",
  "message_hash BYTEA NOT NULL,",
  "consumed_at TIMESTAMPTZ,",
  "CONSTRAINT identity_siwe_challenges_nonce_hash_unique UNIQUE (nonce_hash)",
  "OCTET_LENGTH(nonce_hash) = 32",
  "OCTET_LENGTH(message_hash) = 32",
  "CREATE INDEX IF NOT EXISTS identity_siwe_challenges_active_lookup_idx",
  "ON public.identity_siwe_challenges (wallet_address, client_id, nonce_hash)",
  "WHERE consumed_at IS NULL;",
  "CREATE TABLE IF NOT EXISTS public.identity_refresh_families (",
  "CONSTRAINT identity_refresh_families_ownership_unique UNIQUE (family_id, user_id, client_id)",
  "CREATE TABLE IF NOT EXISTS public.identity_refresh_sessions (",
  "family_id UUID NOT NULL,",
  "parent_session_id UUID,",
  "client_id IN (\u0027epsx-frontend\u0027, \u0027epsx-admin\u0027)",
  "token_hash BYTEA NOT NULL,",
  "hash_key_id VARCHAR(64) NOT NULL,",
  "hash_version SMALLINT NOT NULL DEFAULT 1,",
  "generation INTEGER NOT NULL DEFAULT 0,",
  "CONSTRAINT identity_refresh_sessions_family_fk",
  "FOREIGN KEY (family_id, user_id, client_id)",
  "REFERENCES public.identity_refresh_families (family_id, user_id, client_id)",
  "CONSTRAINT identity_refresh_sessions_lineage_parent_key",
  "UNIQUE (session_id, user_id, family_id, client_id)",
  "CONSTRAINT identity_refresh_sessions_parent_lineage_fk",
  "FOREIGN KEY (parent_session_id, user_id, family_id, client_id)",
  "REFERENCES public.identity_refresh_sessions (session_id, user_id, family_id, client_id)",
  "CONSTRAINT identity_refresh_sessions_parent_unique UNIQUE (parent_session_id)",
  "CONSTRAINT identity_refresh_sessions_not_self_parent CHECK (",
  "parent_session_id IS NULL OR parent_session_id <> session_id",
  "CONSTRAINT identity_refresh_sessions_token_hash_unique UNIQUE (hash_key_id, token_hash)",
  "ON DELETE RESTRICT",
  "WHERE consumed_at IS NULL AND revoked_at IS NULL;",
  "CREATE UNIQUE INDEX IF NOT EXISTS identity_refresh_sessions_one_root_per_family_idx",
  "WHERE parent_session_id IS NULL;",
];
exact("required SQL anchors", expectedAnchors, design.requiredAnchors);
for (const anchor of design.requiredAnchors) if (!sql.includes(anchor)) fail(`migration is missing required anchor: ${anchor}`);
exact("forbidden authority terms", ["roles", "permissions", "plans", "tier_level", "subscription"], design.forbiddenAuthorityTerms);
exact("forbidden secret columns", ["access_token", "refresh_token", "raw_token", "jwt_secret", "password", "signature"], design.forbiddenSecretColumns);
for (const term of [...design.forbiddenAuthorityTerms, ...design.forbiddenSecretColumns]) {
  if (new RegExp(`\\b${term}\\b`, "i").test(sql)) fail(`migration contains forbidden authority or secret term: ${term}`);
}
if (/\b(?:digest_algorithm|hash_algorithm|sha256|sha-256|blake2|blake3)\b/i.test(sql)) fail("schema must not claim a digest algorithm");
const tableBody = (name) => {
  const match = sql.match(new RegExp(`CREATE\\s+TABLE\\s+IF\\s+NOT\\s+EXISTS\\s+public\\.${name}\\s*\\(([\\s\\S]*?)\\n\\);`, "i"));
  if (!match) fail(`${name} table body is missing`);
  return match[1];
};
const challenges = tableBody("identity_siwe_challenges");
const families = tableBody("identity_refresh_families");
const sessions = tableBody("identity_refresh_sessions");
if (!/client_id\s+VARCHAR\(64\)\s+NOT\s+NULL/i.test(challenges) || !/CONSTRAINT\s+identity_siwe_challenges_client_id\s+CHECK\s*\(\s*client_id\s+IN\s*\(\u0027epsx-frontend\u0027,\s*\u0027epsx-admin\u0027\)\s*\)/i.test(challenges)) fail("SIWE challenge client binding is missing");
if (!/identity_siwe_challenges\s*\(wallet_address,\s*client_id,\s*nonce_hash\)\s*WHERE\s+consumed_at\s+IS\s+NULL/i.test(sql)) fail("SIWE active lookup must include the stored client");
if (!/chain_id\s+VARCHAR\(20\)\s+NOT\s+NULL/i.test(challenges) || !/CONSTRAINT\s+identity_siwe_challenges_chain_id_format\s+CHECK\s*\(\s*CASE\s+WHEN\s+chain_id\s*~\s*\u0027\^\(0\|\[1-9\]\[0-9\]\{0,19\}\)\$\u0027\s+THEN\s+chain_id::NUMERIC\s*<=\s*18446744073709551615\s+ELSE\s+FALSE\s+END\s*\)/i.test(challenges)) fail("chain_id must be a canonical decimal within the exact u64 upper bound");
if (!/FOREIGN\s+KEY\s*\(user_id\)\s+REFERENCES\s+public\.identity_users\s*\(user_id\)\s+ON\s+DELETE\s+RESTRICT/i.test(families)) fail("refresh family user ownership must be restrictively linked");
if (!/UNIQUE\s*\(family_id,\s*user_id,\s*client_id\)/i.test(families) || !/client_id\s+IN\s*\(\u0027epsx-frontend\u0027,\s*\u0027epsx-admin\u0027\)/i.test(families)) fail("refresh family ownership key or client constraint is missing");
if (!/FOREIGN\s+KEY\s*\(family_id,\s*user_id,\s*client_id\)\s+REFERENCES\s+public\.identity_refresh_families\s*\(family_id,\s*user_id,\s*client_id\)\s+ON\s+DELETE\s+RESTRICT/i.test(sessions)) fail("refresh session family ownership binding is missing");
if (!/UNIQUE\s*\(session_id,\s*user_id,\s*family_id,\s*client_id\)/i.test(sessions)) fail("refresh parent-side composite key is missing");
if (!/FOREIGN\s+KEY\s*\(parent_session_id,\s*user_id,\s*family_id,\s*client_id\)\s+REFERENCES\s+public\.identity_refresh_sessions\s*\(session_id,\s*user_id,\s*family_id,\s*client_id\)\s+ON\s+DELETE\s+RESTRICT/i.test(sessions)) fail("refresh parent lineage binding is missing");
if (!/parent_session_id\s+IS\s+NULL\s+OR\s+parent_session_id\s*<>\s*session_id/i.test(sessions)) fail("refresh self-parent denial is missing");
if (!/\(parent_session_id IS NULL AND generation = 0\)[\s\S]*?\(parent_session_id IS NOT NULL AND generation > 0\)/.test(sessions)) fail("refresh generation/root shape is missing");
if (!/CREATE\s+UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+identity_refresh_sessions_one_root_per_family_idx\s+ON\s+public\.identity_refresh_sessions\s*\(family_id\)\s+WHERE\s+parent_session_id\s+IS\s+NULL/i.test(sql)) fail("refresh family must have at most one root session");

const lifecycle = contract.lifecycleDesign;
if (!lifecycle || lifecycle.runtimeImplemented !== false || lifecycle.concurrencyProven !== false || lifecycle.crossClientReplayProven !== false || lifecycle.exactGenerationIncrementProven !== false || lifecycle.revokeVsRotateProven !== false) fail("lifecycle non-claim drifted");
for (const key of ["identity", "siweChallenge", "crossClientReplay", "refreshRotation", "refreshRevocationRace", "refreshStorage"]) if (typeof lifecycle[key] !== "string" || lifecycle[key].length < 100) fail(`${key} lifecycle design is not substantive`);
if (!lifecycle.siweChallenge.includes("stored client") || !lifecycle.siweChallenge.includes("token issuance") || !lifecycle.crossClientReplay.includes("other client")) fail("cross-client replay requirement drifted");
if (!lifecycle.refreshRotation.includes("generation equals parent generation plus one remains a transactional concurrency STOP")) fail("exact refresh generation limitation drifted");
if (!lifecycle.refreshRevocationRace.includes("atomic row-locking or serializable boundary") || !lifecycle.refreshRevocationRace.includes("require revoked_at IS NULL") || !lifecycle.refreshRevocationRace.includes("failing rotation before parent consume or child insert")) fail("revoke-vs-rotate requirement drifted");
if (!lifecycle.refreshStorage.includes("does not claim or enforce a particular digest algorithm")) fail("digest-algorithm non-claim drifted");
exact("negative static cases", [
  "challenge-client-binding-removed",
  "challenge-cross-client-replay-authorized",
  "chain-id-u64-upper-bound-removed",
  "refresh-family-owner-mismatch",
  "refresh-parent-lineage-mismatch",
  "refresh-self-parent-authorized",
  "refresh-single-successor-unique-removed",
  "refresh-one-root-index-removed",
  "refresh-revoke-vs-rotate-overclaim",
  "if-not-exists-catalog-adoption-overclaim",
  "digest-algorithm-overclaim",
], contract.negativeStaticCases);
if (!Array.isArray(contract.nonClaims) || contract.nonClaims.length !== 7 || contract.nonClaims.some((item) => typeof item !== "string" || item.length < 80)) fail("seven substantive non-claims are required");
if (!contract.nonClaims.some((item) => item.startsWith("IF NOT EXISTS provides name idempotence only:"))) fail("name-idempotence/catalog non-claim is missing");
if (!contract.nonClaims.some((item) => item.startsWith("Thirty-two-byte digest columns"))) fail("digest-algorithm non-claim is missing");
if (!contract.nonClaims.some((item) => item.startsWith("The schema does not prove row-lock ordering"))) fail("revoke-vs-rotate non-claim is missing");
const blockerCategories = ["migration-runner", "baseline-mapping", "backfill", "populated-upgrade", "concurrency", "reconciliation", "runtime-integration", "external-jwks", "live-database", "catalog-compatibility"];
if (!Array.isArray(contract.blockers) || contract.blockers.length !== blockerCategories.length) fail("exactly ten residual blockers are required");
for (let index = 0; index < contract.blockers.length; index += 1) {
  const blocker = contract.blockers[index];
  const id = `B${String(index + 1).padStart(2, "0")}`;
  if (blocker.id !== id || blocker.category !== blockerCategories[index] || blocker.status !== "blocked" || typeof blocker.summary !== "string" || blocker.summary.length < 70) fail(`${id}: residual blocker drifted`);
}

const output = {
  schemaVersion: 1,
  contractId: contract.contractId,
  service: contract.scope.service,
  status: contract.scope.status,
  source: {
    developmentCommit: source.developmentCommit,
    evidenceItems: source.items.length,
    schemaHistoryPaths: observedHistory.length,
  },
  migrationRoot: {
    path: migrationRoot.path,
    migrations: 1,
    pinnedBytes: migration.bytes,
    sha256: migration.sha256,
    statements: semicolonCount,
    guardedTables: guardedTables.length,
    guardedIndexes: guardedIndexes.length,
    runner: null,
  },
  lifecycle: {
    identityMapping: "uuid-subject-to-lowercase-wallet",
    challengeStorage: "32-byte-digests-client-bound-single-consume-shape",
    refreshStorage: "32-byte-keyed-digest-family-lineage-shape",
    routesEnabled: false,
    runtimeImplemented: false,
    concurrencyProven: false,
    crossClientReplayProven: false,
    exactGenerationIncrementProven: false,
    revokeVsRotateProven: false,
    catalogCompatibilityProven: false,
    digestAlgorithmClaimed: false,
  },
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
  echo "a3-8-identity-lifecycle-schema: PASS — 6,417-byte additive migration pinned; 4 tables, 6 indexes, client/lineage/u64 constraints, and immutable source history verified"
  echo "a3-8-identity-lifecycle-schema: LIMIT — routes remain disabled; no runner, catalog adoption, backfill, upgrade, concurrency, JWKS, runtime, or database proof ran"
  exit 0
fi

echo "a3-8-identity-lifecycle-schema: STOP — ten residual A3.8 blockers remain; readiness is intentionally exit 3" >&2
echo "a3-8-identity-lifecycle-schema: LIMIT — static schema integrity is not migration or lifecycle execution evidence" >&2
exit 3
