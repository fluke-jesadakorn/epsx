#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT="$REPO_ROOT"
CONTRACT="$REPO_ROOT/docs/migration/contracts/refresh-digest-replay.json"
MIGRATION_DIR="$REPO_ROOT/apps/backend/migrations/core/20260723100000_add_refresh_token_digest_replay_state"
UP_SQL="$MIGRATION_DIR/up.sql"
DOWN_SQL="$MIGRATION_DIR/down.sql"
MODE="evidence"

die() {
  echo "refresh-digest-replay: ERROR: $*" >&2
  exit 1
}

while (( $# > 0 )); do
  case "$1" in
    --contract)
      (( $# >= 2 )) || die "--contract requires a path"
      CONTRACT="$2"
      shift 2
      ;;
    --up)
      (( $# >= 2 )) || die "--up requires a path"
      UP_SQL="$2"
      shift 2
      ;;
    --down)
      (( $# >= 2 )) || die "--down requires a path"
      DOWN_SQL="$2"
      shift 2
      ;;
    --evidence-root)
      (( $# >= 2 )) || die "--evidence-root requires a path"
      EVIDENCE_ROOT="$2"
      shift 2
      ;;
    --mode)
      (( $# >= 2 )) || die "--mode requires evidence or readiness"
      MODE="$2"
      shift 2
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

[[ "$MODE" == "evidence" || "$MODE" == "readiness" ]] || die "mode must be evidence or readiness"
command -v bun >/dev/null 2>&1 || die "bun is required"

for name in \
  DATABASE_URL TEST_DATABASE_URL REDIS_URL IDENTITY_DATABASE_URL API_URL BACKEND_URL \
  REFRESH_TOKEN_DIGEST_KEYS REFRESH_TOKEN_DIGEST_KEYS_JSON REFRESH_TOKEN_DIGEST_ACTIVE_KEY \
  REFRESH_TOKEN_HMAC_ACTIVE_KID REFRESH_TOKEN_HMAC_KEYS_JSON; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no live or secret-backed I/O"
done

bun -e '
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const [root, contractPath, upPath, downPath] = process.argv.slice(1);
const fail = (message) => {
  console.error(`refresh-digest-replay: ERROR: ${message}`);
  process.exit(1);
};
const read = (path) => {
  try { return readFileSync(path, "utf8"); }
  catch (error) { fail(`cannot read ${path}: ${error.message}`); }
};
const sha256 = (text) => createHash("sha256").update(text).digest("hex");

let contract;
try { contract = JSON.parse(read(contractPath)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A1.6-refresh-digest-replay-schema") fail("unexpected contract identity");
if (contract.status !== "partial-hermetic-schema-runtime-proof") fail("contract status must remain a partial schema/runtime hermetic proof");
if (contract.productionReady !== false || contract.databaseProof !== false || contract.runtimeHermeticProof !== true || contract.runtimeProof !== false || contract.cutoverAuthorized !== false) fail("contract must remain non-production, database-unproved, runtime-hermetic, runtime-integration-unproved, and cutover-unauthorized");
if (contract.authority?.table !== "public.openid_refresh_tokens" || contract.authority?.runtime !== "shared/rust/epsx-identity-shared") fail("canonical authority drifted");
if (contract.migration?.mode !== "additive-expand-forward-only") fail("migration mode drifted");
if (!contract.migration.legacyPolicy.includes("never request-claimed") || !contract.migration.legacyPolicy.includes("drained maintenance cutover forces reauthentication")) fail("legacy fail-closed policy drifted");
if (!contract.migration.storagePolicy.includes("Only storage_version 2") || !contract.migration.storagePolicy.includes("dual-write version 1 is intentionally unsupported")) fail("digest-only storage policy drifted");

const expectedInvariants = new Map([
  ["additive-null-expansion", "Seven nullable, default-free, non-identity, non-generated columns are added without mutating a row."],
  ["digest-metadata-atomic-shape", "Legacy rows keep all digest metadata NULL; storage-version-2 rows require digest metadata plus existing client and family bindings."],
  ["digest-only-storage-version", "Only storage_version 2 is admitted; no raw-bearing dual-write version is defined by this migration."],
  ["digest-width", "A non-NULL token digest is exactly 32 bytes."],
  ["key-selector-shape", "Digest key identifiers use 1-32 ASCII alphanumeric, underscore, or hyphen characters, and digest versions are positive."],
  ["terminal-state-shape", "Each storage-version-2 row is exactly active, consumed, or revoked; legacy rows keep every new digest and lifecycle column NULL."],
  ["lifecycle-time-order", "Consumption and revocation cannot predate row creation."],
  ["replay-after-consume", "Replay detection requires a consumed predecessor and cannot predate consumption."],
  ["digest-lookup-uniqueness", "The non-NULL key ID, digest version, and digest tuple is unique."],
  ["forward-only-security-state", "The reverse migration refuses to erase digest or replay evidence."]
]);
if (!Array.isArray(contract.invariants) || contract.invariants.length !== expectedInvariants.size) fail("ten schema invariants are required");
const observedInvariantIds = new Set();
for (const invariant of contract.invariants) {
  if (!invariant || typeof invariant.id !== "string" || typeof invariant.claim !== "string") fail("schema invariants require stable id and claim fields");
  if (observedInvariantIds.has(invariant.id)) fail(`duplicate invariant id ${invariant.id}`);
  observedInvariantIds.add(invariant.id);
  if (expectedInvariants.get(invariant.id) !== invariant.claim) fail(`schema invariant drifted for ${invariant.id}`);
}
for (const id of expectedInvariants.keys()) if (!observedInvariantIds.has(id)) fail(`missing schema invariant ${id}`);

const expectedRuntimeInvariants = new Map([
  ["opaque-credential-v1", "Refresh credentials use strict rt1 framing with exactly 32 random bytes from OsRng and canonical unpadded base64url encoding."],
  ["required-dedicated-hmac-keyring", "Both runtime factories require the dedicated refresh-token HMAC keyring from environment with no generated or default fallback, and HMAC-SHA256 uses the pinned domain separator."],
  ["digest-only-storage-and-lookup", "New rows use independent UUID storage identifiers, persist only digest material, and locate presented credentials through BYTEA digest metadata."],
  ["exact-runtime-binding-predicates", "Validation and consumption require the exact digest key, digest version, digest bytes, storage version 2, client, and non-NULL matching family."],
  ["atomic-consumed-state", "The winning consume mutation atomically sets is_revoked true and consumed_at from the PostgreSQL clock before inserting the successor."],
  ["committed-reuse-response", "Consumed-token reuse records replay and revokes active family descendants inside a transaction that returns a success outcome before the public boundary maps it to generic credential rejection."],
  ["family-lock-and-database-clock", "Rotation, logout, and replay response share the transaction-scoped family advisory lock and use PostgreSQL clock_timestamp for persisted lifecycle timestamps."],
  ["stateless-openid-composition", "Both backend factories require the HMAC keyring, inject it into OpenIDTokenService, and construct UnifiedWeb3AuthService with new_with_openid."],
  ["bearer-dto-debug-redaction", "The token response, logout request, and refresh request bearer-bearing DTOs do not derive Debug."],
  ["bounded-legacy-raw-path", "No new-row insert stores a bearer in token_id; raw UUID comparison remains only in bounded storage-version-NULL legacy logout."]
]);
if (!Array.isArray(contract.runtimeInvariants) || contract.runtimeInvariants.length !== expectedRuntimeInvariants.size) fail("ten runtime invariants are required");
const observedRuntimeInvariantIds = new Set();
for (const invariant of contract.runtimeInvariants) {
  if (!invariant || typeof invariant.id !== "string" || typeof invariant.claim !== "string") fail("runtime invariants require stable id and claim fields");
  if (observedRuntimeInvariantIds.has(invariant.id)) fail(`duplicate runtime invariant id ${invariant.id}`);
  observedRuntimeInvariantIds.add(invariant.id);
  if (expectedRuntimeInvariants.get(invariant.id) !== invariant.claim) fail(`runtime invariant drifted for ${invariant.id}`);
}
for (const id of expectedRuntimeInvariants.keys()) if (!observedRuntimeInvariantIds.has(id)) fail(`missing runtime invariant ${id}`);

const expectedStops = new Map([
  ["core-migration-version-collision", "No migration in the active core root may execute until the duplicate 00000000000001 baseline collision is reconciled."],
  ["postgres-expansion-unproved", "No disposable PostgreSQL instance has applied this expansion from every observed supported history."],
  ["forward-only-down-unproved", "No PostgreSQL proof shows that attempted down migration fails atomically without schema or data change."],
  ["digest-runtime-database-proof-absent", "Hermetic runtime evidence does not prove digest lookup, lifecycle transitions, replay response, or fail-closed startup against PostgreSQL."],
  ["maintenance-cutover-unproved", "The required drained old-to-new maintenance cutover and forced reauthentication have not been executed or proven."],
  ["legacy-zero-raw-unproved", "Legacy revocation, plaintext scrubbing, reconciliation, and zero accepted or retained raw bearers are unproved."],
  ["key-lifecycle-unproved", "Persistent digest-key secret provisioning, restart persistence, production rotation and safe retirement, and PostgreSQL integration remain unproved and unauthorized."],
  ["postgres-replay-ordering-unproved", "MVCC consume, revoke, replay detection, family-lock ordering, and forced-successor rollback are unproved."],
  ["access-token-post-logout-validity", "Already-issued access tokens remain valid until expiry after logout or refresh-family revocation."],
  ["production-actions-unauthorized", "No production secret, migration, service, browser, routing, canary, rollback, deployment, or database action is authorized by this contract."]
]);
if (!Array.isArray(contract.residualStops) || contract.residualStops.length !== expectedStops.size) fail("ten residual STOP claims are required");
const observedStopIds = new Set();
for (const stop of contract.residualStops) {
  if (!stop || typeof stop.id !== "string" || typeof stop.claim !== "string") fail("residual STOP entries require stable id and claim fields");
  if (observedStopIds.has(stop.id)) fail(`duplicate residual STOP id ${stop.id}`);
  observedStopIds.add(stop.id);
  if (expectedStops.get(stop.id) !== stop.claim) fail(`residual STOP claim drifted for ${stop.id}`);
}
for (const id of expectedStops.keys()) if (!observedStopIds.has(id)) fail(`missing residual STOP ${id}`);

const expectedEvidence = [
  { file: "apps/backend/migrations/core/20260723100000_add_refresh_token_digest_replay_state/up.sql", anchors: ["ADD COLUMN IF NOT EXISTS token_digest BYTEA", "ADD COLUMN IF NOT EXISTS digest_key_id VARCHAR(32)", "ADD COLUMN IF NOT EXISTS replay_detected_at TIMESTAMPTZ", "storage_version = 2", "OCTET_LENGTH(token_digest) = 32", "openid_refresh_tokens_terminal_state_check", "CREATE UNIQUE INDEX openid_refresh_tokens_digest_lookup_uq"] },
  { file: "apps/backend/migrations/core/20260723100000_add_refresh_token_digest_replay_state/down.sql", anchors: ["RAISE EXCEPTION", "forward-only"] },
  { file: "shared/rust/epsx-identity-shared/src/refresh_token_digest.rs", anchors: ["const TOKEN_VERSION: &str = \"rt1\";", "const TOKEN_SECRET_BYTES: usize = 32;", "const DIGEST_BYTES: usize = 32;", "const HMAC_DOMAIN_SEPARATOR: &[u8] = b\"epsx.refresh.v1\\0\";", "REFRESH_TOKEN_HMAC_ACTIVE_KID", "REFRESH_TOKEN_HMAC_KEYS_JSON", "pub fn from_env()", "OsRng.fill_bytes(&mut secret);", "pub fn digest_presented(", "Hmac::<Sha256>::new_from_slice(key)", "strict_parser_rejects_malformed_and_noncanonical_credentials", "deterministic_hmac_vector_is_stable_and_domain_separated"] },
  { file: "shared/rust/epsx-identity-shared/src/token_service.rs", anchors: ["const REFRESH_DIGEST_VERSION: i16 = 1;", "const REFRESH_STORAGE_VERSION: i16 = 2;", "enum RefreshRotationOutcome", "#[derive(Clone, Serialize, Deserialize, ToSchema)]\npub struct OpenIDTokenResponse", "let now = self.current_database_time().await?;", "let new_storage_id = Uuid::new_v4().to_string();", "let storage_id = Uuid::new_v4().to_string();", ".filter(openid_refresh_tokens::token_digest.eq(Some(old_digest.clone())))", ".filter(openid_refresh_tokens::storage_version", "client_id.eq(Some(requested_client_id.as_str()))", "family_id.eq(Some(expected_family_id))", "consumed_at.eq(Some(now))", "return Ok(RefreshRotationOutcome::ReuseDetected);", "RefreshRotationOutcome::ReuseDetected | RefreshRotationOutcome::Invalid", "pg_advisory_xact_lock(hashtextextended", "SELECT clock_timestamp() AS observed_at", "token_id.eq(&presented_token)", "storage_version.is_null()"] },
  { file: "shared/rust/epsx-identity-shared/src/auth_service.rs", anchors: ["pub fn new_with_openid(", "let new_refresh_token = openid_service.issue_refresh_token();", ".consume_refresh_token("] },
  { file: "apps/backend/src/infrastructure/container/simple_container.rs", anchors: ["RefreshTokenKeyring::from_env()", "OpenIDTokenService::new(", "UnifiedWeb3AuthService::new_with_openid("] },
  { file: "apps/backend/src/infrastructure/container/stateless_service_factory.rs", anchors: ["RefreshTokenKeyring::from_env()", "OpenIDTokenService::new(", "UnifiedWeb3AuthService::new_with_openid("] },
  { file: "shared/rust/epsx-identity-shared/src/schemas.rs", anchors: ["token_digest -> Nullable<Bytea>", "digest_key_id -> Nullable<Varchar>", "digest_version -> Nullable<Int2>", "storage_version -> Nullable<Int2>", "consumed_at -> Nullable<Timestamptz>", "revoked_at -> Nullable<Timestamptz>", "replay_detected_at -> Nullable<Timestamptz>"] },
  { file: "apps/backend/src/schemas/primary.rs", anchors: ["token_digest -> Nullable<Bytea>", "digest_key_id -> Nullable<Varchar>", "digest_version -> Nullable<Int2>", "storage_version -> Nullable<Int2>", "consumed_at -> Nullable<Timestamptz>", "revoked_at -> Nullable<Timestamptz>", "replay_detected_at -> Nullable<Timestamptz>"] },
  { file: "shared/rust/epsx-identity-shared/Cargo.toml", anchors: ["sha2 = \"0.10\"", "hmac.workspace = true", "rand = \"0.8\"", "base64 = \"0.22\""] },
  { file: "apps/backend/src/web/auth/handlers.rs", anchors: ["#[derive(Deserialize, Serialize, ToSchema)]\npub struct LogoutRequest", "#[derive(Deserialize, Serialize, ToSchema)]\npub struct TokenRefreshRequest"] },
  { file: "docs/migration/A1_5_REFRESH_CLIENT_BINDING.md", anchors: ["opaque credentials, stores keyed digests", "active-descendant revocation after detected reuse"] },
  { file: "docs/migration/contracts/migration-safety.json", anchors: ["version.core-baseline-collision", "Two active core directories share version 00000000000001."] }
];
if (JSON.stringify(contract.evidence) !== JSON.stringify(expectedEvidence)) fail("evidence inventory drifted from the exact pinned file/anchor map");
let evidenceAnchors = 0;
for (const item of expectedEvidence) {
  const contents = read(resolve(root, item.file));
  for (const anchor of item.anchors) {
    if (!contents.includes(anchor)) fail(`${item.file}: missing evidence anchor ${JSON.stringify(anchor)}`);
    evidenceAnchors += 1;
  }
}
if (evidenceAnchors !== 72) fail("evidence anchor count drifted");

const up = read(upPath);
const down = read(downPath);
if (sha256(up) !== contract.migration.upSha256) fail("up migration checksum drifted");
if (sha256(down) !== contract.migration.downSha256) fail("down migration checksum drifted");
if (/\b(DROP|DELETE|TRUNCATE|UPDATE|INSERT|MERGE)\b/i.test(up)) fail("expand migration contains destructive or data-mutating SQL");
if (/ALTER\s+COLUMN|SET\s+NOT\s+NULL/i.test(up)) fail("expand migration contains an enforcement change");

const expectedColumns = new Map([
  ["token_digest", "BYTEA"],
  ["digest_key_id", "VARCHAR(32)"],
  ["digest_version", "SMALLINT"],
  ["storage_version", "SMALLINT"],
  ["consumed_at", "TIMESTAMPTZ"],
  ["revoked_at", "TIMESTAMPTZ"],
  ["replay_detected_at", "TIMESTAMPTZ"]
]);
const addColumnMatches = [...up.matchAll(/ADD\s+COLUMN\s+IF\s+NOT\s+EXISTS\s+([a-z_]+)\s+(BYTEA|VARCHAR\(32\)|SMALLINT|TIMESTAMPTZ)/gi)];
if (addColumnMatches.length !== expectedColumns.size) fail(`expected seven additive columns, observed ${addColumnMatches.length}`);
for (const match of addColumnMatches) {
  if (expectedColumns.get(match[1]) !== match[2].toUpperCase()) fail(`unexpected column shape for ${match[1]}: ${match[2]}`);
}
for (const column of expectedColumns.keys()) if (!addColumnMatches.some((match) => match[1] === column)) fail(`missing additive column ${column}`);

for (const anchor of [
  "(\u0027token_digest\u0027, \u0027bytea\u0027, NULL::INTEGER)",
  "(\u0027digest_key_id\u0027, \u0027character varying\u0027, 32)",
  "(\u0027digest_version\u0027, \u0027smallint\u0027, NULL::INTEGER)",
  "(\u0027storage_version\u0027, \u0027smallint\u0027, NULL::INTEGER)",
  "(\u0027consumed_at\u0027, \u0027timestamp with time zone\u0027, NULL::INTEGER)",
  "(\u0027revoked_at\u0027, \u0027timestamp with time zone\u0027, NULL::INTEGER)",
  "(\u0027replay_detected_at\u0027, \u0027timestamp with time zone\u0027, NULL::INTEGER)",
  "observed_is_nullable IS DISTINCT FROM \u0027YES\u0027",
  "observed_default IS NOT NULL",
  "observed_is_identity IS DISTINCT FROM \u0027NO\u0027",
  "observed_is_generated IS DISTINCT FROM \u0027NEVER\u0027",
  "storage_version = 2",
  "OCTET_LENGTH(token_digest) = 32",
  "digest_key_id ~ \u0027^[A-Za-z0-9_-]{1,32}$\u0027",
  "digest_version IS NULL OR digest_version > 0",
  "consumed_at IS NULL OR consumed_at >= created_at",
  "revoked_at IS NULL OR revoked_at >= created_at",
  "replay_detected_at >= consumed_at"
]) if (!up.includes(anchor)) fail(`up migration missing ${JSON.stringify(anchor)}`);
if (/storage_version\s*=\s*1|storage_version\s+IN\s*\([^)]*1/i.test(up)) fail("raw-bearing storage version 1 must not be admitted");

const normalizedUp = up.replace(/\s+/g, " ");
if (!normalizedUp.includes("storage_version = 2 AND token_digest IS NOT NULL AND digest_key_id IS NOT NULL AND digest_version IS NOT NULL AND client_id IS NOT NULL AND family_id IS NOT NULL")) fail("storage-version-2 binding shape drifted");
if (!normalizedUp.includes("storage_version IS NULL AND token_digest IS NULL AND digest_key_id IS NULL AND digest_version IS NULL AND consumed_at IS NULL AND revoked_at IS NULL AND replay_detected_at IS NULL")) fail("terminal-state legacy shape drifted");
if (!normalizedUp.includes("is_revoked IS FALSE AND consumed_at IS NULL AND revoked_at IS NULL AND replay_detected_at IS NULL")) fail("terminal-state active shape drifted");
if (!normalizedUp.includes("is_revoked IS TRUE AND consumed_at IS NOT NULL AND revoked_at IS NULL")) fail("terminal-state consumed shape drifted");
if (!normalizedUp.includes("is_revoked IS TRUE AND consumed_at IS NULL AND revoked_at IS NOT NULL AND replay_detected_at IS NULL")) fail("terminal-state revoked shape drifted");

const constraintNames = [
  "openid_refresh_tokens_digest_shape_check",
  "openid_refresh_tokens_digest_size_check",
  "openid_refresh_tokens_digest_key_id_check",
  "openid_refresh_tokens_digest_version_check",
  "openid_refresh_tokens_terminal_state_check",
  "openid_refresh_tokens_consumed_order_check",
  "openid_refresh_tokens_revoked_order_check",
  "openid_refresh_tokens_replay_order_check"
];
for (const name of constraintNames) {
  const addCount = (up.match(new RegExp(`ADD\\s+CONSTRAINT\\s+${name}\\b`, "gi")) ?? []).length;
  const validateCount = (up.match(new RegExp(`VALIDATE\\s+CONSTRAINT\\s+${name}\\b`, "gi")) ?? []).length;
  if (addCount !== 1 || validateCount !== 1 || !up.includes(`\u0027${name}\u0027`)) fail(`${name}: add, validate, and catalog-refusal anchors are required exactly once`);
}
if (!/CREATE\s+UNIQUE\s+INDEX\s+openid_refresh_tokens_digest_lookup_uq\s+ON\s+public\.openid_refresh_tokens\s*\(\s*digest_key_id\s*,\s*digest_version\s*,\s*token_digest\s*\)\s+WHERE\s+token_digest\s+IS\s+NOT\s+NULL\s*;/i.test(up)) fail("partial unique digest index shape drifted");
if (!normalizedUp.includes("IF EXISTS ( SELECT 1 FROM pg_class AS relation JOIN pg_namespace AS namespace ON namespace.oid = relation.relnamespace WHERE namespace.nspname = \u0027public\u0027 AND relation.relname = \u0027openid_refresh_tokens_digest_lookup_uq\u0027 ) THEN RAISE EXCEPTION \u0027pre-existing openid_refresh_tokens_digest_lookup_uq is refused; reconcile catalog drift explicitly\u0027; END IF;")) fail("pre-existing digest index must be refused");

if (!down.includes("RAISE EXCEPTION") || !down.includes("forward-only") || /\b(DROP|DELETE|TRUNCATE|UPDATE|INSERT|MERGE|ALTER|CREATE)\b/i.test(down)) fail("down migration must only refuse destructive rollback");

const normalize = (value) => value.replace(/\s+/g, " ").trim();
const stripRustComments = (source) => {
  let output = "";
  let index = 0;
  while (index < source.length) {
    const raw = source.slice(index).match(/^(?:br|r)(#{0,32})"/);
    if (raw) {
      const terminator = `"${raw[1]}`;
      const end = source.indexOf(terminator, index + raw[0].length);
      if (end < 0) fail("unterminated Rust raw string while stripping comments");
      const next = end + terminator.length;
      output += source.slice(index, next);
      index = next;
      continue;
    }
    if (source[index] === "\"") {
      const start = index++;
      let escaped = false;
      while (index < source.length) {
        const character = source[index++];
        if (character === "\"" && !escaped) break;
        if (character === "\\" && !escaped) escaped = true;
        else escaped = false;
      }
      output += source.slice(start, index);
      continue;
    }
    if (source.startsWith("//", index)) {
      while (index < source.length && source[index] !== "\n") {
        output += " ";
        index += 1;
      }
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 1;
      output += "  ";
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          output += "  ";
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          output += "  ";
          index += 2;
        } else {
          output += source[index] === "\n" ? "\n" : " ";
          index += 1;
        }
      }
      if (depth !== 0) fail("unterminated Rust block comment while stripping comments");
      continue;
    }
    output += source[index];
    index += 1;
  }
  return output;
};
const section = (source, startAnchor, endAnchor, label) => {
  const start = source.indexOf(startAnchor);
  const end = source.indexOf(endAnchor, start + startAnchor.length);
  if (start < 0 || end < 0 || end <= start) fail(`${label} boundary is missing`);
  return source.slice(start, end);
};
const occurrenceCount = (source, value) => source.split(value).length - 1;

const digestSource = stripRustComments(read(resolve(root, "shared/rust/epsx-identity-shared/src/refresh_token_digest.rs")));
const issueCredential = normalize(section(digestSource, "pub fn issue(&self)", "pub fn digest_presented(", "opaque credential issuance"));
if (!issueCredential.includes("let mut secret = [0_u8; TOKEN_SECRET_BYTES]; OsRng.fill_bytes(&mut secret); self.issue_with_secret(secret)")) fail("rt1 issuance must use exactly TOKEN_SECRET_BYTES from OsRng");
const parseCredential = normalize(section(digestSource, "fn parse(credential: &str)", "fn validate_key_id", "strict credential parser"));
for (const anchor of [
  "credential.len() > MAX_TOKEN_BYTES || !credential.is_ascii()",
  "credential.split(\u0027.\u0027)",
  "if parts.next().is_some()",
  "if version != TOKEN_VERSION",
  "decoded_secret.len() != TOKEN_SECRET_BYTES || URL_SAFE_NO_PAD.encode(&decoded_secret) != encoded_secret"
]) if (!parseCredential.includes(anchor)) fail(`strict rt1 parser missing ${JSON.stringify(anchor)}`);
const fromEnv = normalize(section(digestSource, "pub fn from_env()", "pub fn active_key_id", "required HMAC environment keyring"));
if (!fromEnv.includes("env::var(REFRESH_TOKEN_HMAC_ACTIVE_KID_ENV)") || !fromEnv.includes("env::var(REFRESH_TOKEN_HMAC_KEYS_JSON_ENV)") || !fromEnv.includes("Self::from_json(&active_key_id, &encoded_keys_json)")) fail("dedicated refresh HMAC keyring environment contract drifted");
if (/unwrap_or|unwrap_or_else|unwrap_or_default|::default|OsRng|generate/i.test(fromEnv)) fail("refresh HMAC keyring must have no generated or default fallback");
const digestSecret = normalize(section(digestSource, "fn digest_secret(", "struct EncodedKeys", "domain-separated HMAC"));
const digestSteps = [
  "Hmac::<Sha256>::new_from_slice(key)",
  "mac.update(HMAC_DOMAIN_SEPARATOR)",
  "mac.update(key_id.as_bytes())",
  "mac.update(&[0])",
  "mac.update(secret)",
  "mac.finalize()"
];
let previousDigestStep = -1;
for (const step of digestSteps) {
  const position = digestSecret.indexOf(step);
  if (position < 0 || position <= previousDigestStep) fail("HMAC-SHA256 domain-separation sequence drifted");
  previousDigestStep = position;
}

const tokenService = stripRustComments(read(resolve(root, "shared/rust/epsx-identity-shared/src/token_service.rs")));
const normalizedTokenService = normalize(tokenService);
const createRefresh = section(tokenService, "async fn create_refresh_token(", "pub async fn validate_refresh_token(", "digest-only initial storage");
const consumeRefresh = section(tokenService, "pub(crate) async fn consume_refresh_token(", "pub async fn validate_access_token(", "atomic refresh consumption");
const revokeRefresh = section(tokenService, "pub async fn revoke_refresh_token", "pub(crate) async fn consume_refresh_token(", "refresh logout");
const replayRefresh = section(tokenService, "async fn record_refresh_replay(", "pub fn validate_client_id(", "refresh replay response");
const issueInitial = section(tokenService, "pub async fn issue_tokens_for_user(", "pub(crate) async fn issue_tokens_for_user_with_refresh_token(", "initial issuance clock");
const validateRefresh = section(tokenService, "pub async fn validate_refresh_token(", "async fn record_refresh_replay(", "digest lookup validation");
const normalizedCreate = normalize(createRefresh);
const normalizedConsume = normalize(consumeRefresh);
const normalizedRevoke = normalize(revokeRefresh);
const normalizedReplay = normalize(replayRefresh);
const normalizedValidate = normalize(validateRefresh);

if (!normalizedCreate.includes("let storage_id = Uuid::new_v4().to_string()") || !normalizedCreate.includes("openid_refresh_tokens::token_id.eq(&storage_id)") || !normalizedCreate.includes("openid_refresh_tokens::token_digest.eq(Some(token_digest))") || !normalizedCreate.includes("openid_refresh_tokens::storage_version.eq(Some(REFRESH_STORAGE_VERSION))")) fail("initial refresh insert must use an independent UUID row ID and digest-only storage");
if (normalizedCreate.includes("credential().expose") || normalizedCreate.includes("token_id.eq(refresh_token") || normalizedCreate.includes("token_id.eq(&refresh_token")) fail("initial refresh insert must not store the bearer credential");
if (!normalizedConsume.includes("let new_storage_id = Uuid::new_v4().to_string()") || !normalizedConsume.includes("openid_refresh_tokens::token_id.eq(&new_storage_id)") || !normalizedConsume.includes("openid_refresh_tokens::token_digest.eq(Some(new_digest))")) fail("successor insert must use an independent UUID row ID and digest-only storage");

for (const [label, source, anchors] of [
  ["validation", normalizedValidate, ["digest_key_id.eq(Some(digest_key_id.as_str()))", "digest_version.eq(Some(REFRESH_DIGEST_VERSION))", "token_digest.eq(Some(token_digest.clone()))", "storage_version.eq(Some(REFRESH_STORAGE_VERSION))", "client_id.eq(Some(client.as_str()))", "family_id.is_not_null()"]],
  ["consumption", normalizedConsume, ["digest_key_id .eq(Some(old_digest_key_id.as_str()))", "digest_version.eq(Some(REFRESH_DIGEST_VERSION))", "token_digest.eq(Some(old_digest.clone()))", "storage_version .eq(Some(REFRESH_STORAGE_VERSION))", "client_id.eq(Some(requested_client_id.as_str()))", "family_id.eq(Some(expected_family_id))"]]
]) for (const anchor of anchors) if (!source.includes(anchor)) fail(`${label} digest/client/family predicate missing ${JSON.stringify(anchor)}`);

if (!normalizedConsume.includes(".set(( openid_refresh_tokens::is_revoked.eq(true), openid_refresh_tokens::consumed_at.eq(Some(now)), )) .returning((")) fail("consume must atomically set the revoked/consumed tuple");
const consumeMutation = normalizedConsume.indexOf("consumed_at.eq(Some(now))");
const successorInsert = normalizedConsume.indexOf("diesel::insert_into(openid_refresh_tokens::table)");
if (consumeMutation < 0 || successorInsert < 0 || consumeMutation > successorInsert) fail("predecessor consumption must precede successor insertion");
for (const source of [issueInitial, revokeRefresh, consumeRefresh, replayRefresh]) {
  if (!source.includes("database_clock") && !source.includes("current_database_time")) fail("persisted lifecycle timestamps must use the PostgreSQL clock");
  if (source.includes("Utc::now()")) fail("persisted lifecycle state must not use the process clock");
}
if (!normalizedValidate.includes("Self::database_clock(&mut conn)") || normalizedValidate.includes("Utc::now()")) fail("refresh expiry preflight must use the PostgreSQL clock");
if (occurrenceCount(tokenService, "SELECT clock_timestamp() AS observed_at") !== 1) fail("exact PostgreSQL clock_timestamp helper drifted");

for (const [label, source] of [["logout", normalizedRevoke], ["consumption", normalizedConsume], ["replay", normalizedReplay]]) {
  if (!source.includes("Self::lock_refresh_family(conn,")) fail(`${label} must share the transaction-scoped family advisory lock`);
}
if (!normalizedTokenService.includes("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")) fail("transaction-scoped refresh-family advisory lock drifted");
if (!normalizedConsume.includes("return Ok(RefreshRotationOutcome::ReuseDetected)") || !normalizedConsume.includes("RefreshRotationOutcome::ReuseDetected | RefreshRotationOutcome::Invalid =>")) fail("reuse outcome must commit before generic public rejection");
if (!normalizedConsume.includes("replay_detected_at.eq(Some(now))") || !normalizedConsume.includes("family_id.eq(Some(expected_family_id))") || !normalizedConsume.includes("revoked_at.eq(Some(now))")) fail("consumed-token reuse must record replay and revoke active family descendants");
if (!normalizedReplay.includes("if replayed > 0") || !normalizedReplay.includes("revoked_at.eq(Some(now))")) fail("preflight replay response must commit descendant revocation after recording reuse");

const tokenIdBindings = [...tokenService.matchAll(/openid_refresh_tokens::token_id\.eq\(([^)]+)\)/g)].map((match) => match[1]);
if (JSON.stringify(tokenIdBindings) !== JSON.stringify(["&presented_token", "&new_storage_id", "&storage_id"])) fail("token_id binding inventory drifted; raw bearer storage may have appeared");
if (!normalizedRevoke.includes("token_id.eq(&presented_token)) .filter(openid_refresh_tokens::storage_version.is_null())")) fail("raw UUID comparison must remain bounded to storage-version-NULL legacy logout");

for (const [label, file] of [
  ["simple container", "apps/backend/src/infrastructure/container/simple_container.rs"],
  ["stateless factory", "apps/backend/src/infrastructure/container/stateless_service_factory.rs"]
]) {
  const factory = normalize(stripRustComments(read(resolve(root, file))));
  if (!factory.includes("RefreshTokenKeyring::from_env() .expect(\"Failed to initialize the required refresh-token HMAC keyring\")") || !factory.includes("OpenIDTokenService::new(") || !factory.includes("Arc::new(refresh_token_keyring)") || !factory.includes("UnifiedWeb3AuthService::new_with_openid(")) fail(`${label} must require and inject the HMAC keyring into stateless OpenID auth`);
}

const authService = normalize(stripRustComments(read(resolve(root, "shared/rust/epsx-identity-shared/src/auth_service.rs"))));
if (!authService.includes("pub fn new_with_openid(") || !authService.includes("let new_refresh_token = openid_service.issue_refresh_token()") || !authService.includes(".consume_refresh_token(")) fail("canonical auth service opaque rotation composition drifted");

const schemaColumns = [
  "token_digest -> Nullable<Bytea>",
  "digest_key_id -> Nullable<Varchar>",
  "digest_version -> Nullable<Int2>",
  "storage_version -> Nullable<Int2>",
  "consumed_at -> Nullable<Timestamptz>",
  "revoked_at -> Nullable<Timestamptz>",
  "replay_detected_at -> Nullable<Timestamptz>"
];
for (const [label, file] of [["shared", "shared/rust/epsx-identity-shared/src/schemas.rs"], ["backend", "apps/backend/src/schemas/primary.rs"]]) {
  const schema = stripRustComments(read(resolve(root, file)));
  const tableStart = schema.indexOf("openid_refresh_tokens (token_id) {");
  const tableEnd = schema.indexOf("diesel::table!", tableStart + 1);
  if (tableStart < 0 || tableEnd < 0) fail(`${label} openid_refresh_tokens Diesel table boundary drifted`);
  const refreshSchema = schema.slice(tableStart, tableEnd);
  for (const column of schemaColumns) if (occurrenceCount(refreshSchema, column) !== 1) fail(`${label} Diesel schema drifted for ${column}`);
}
const cargo = read(resolve(root, "shared/rust/epsx-identity-shared/Cargo.toml")).replace(/^\s*#.*$/gm, "");
for (const dependency of ["sha2 = \"0.10\"", "hmac.workspace = true", "rand = \"0.8\"", "base64 = \"0.22\""]) if (occurrenceCount(cargo, dependency) !== 1) fail(`dedicated refresh digest dependency drifted for ${dependency}`);

const deriveTraits = (source, structName) => {
  const marker = `pub struct ${structName}`;
  const position = source.indexOf(marker);
  if (position < 0) fail(`missing bearer DTO ${structName}`);
  const boundary = source.lastIndexOf("\n\n", position);
  const prefix = source.slice(Math.max(0, boundary), position);
  const matches = [...prefix.matchAll(/#\[derive\(([^)]*)\)\]/g)];
  return matches.length === 0 ? [] : matches.at(-1)[1].split(",").map((traitName) => traitName.trim());
};
const handlers = stripRustComments(read(resolve(root, "apps/backend/src/web/auth/handlers.rs")));
for (const [source, structName] of [[tokenService, "OpenIDTokenResponse"], [handlers, "LogoutRequest"], [handlers, "TokenRefreshRequest"]]) {
  if (deriveTraits(source, structName).includes("Debug")) fail(`${structName} must not derive Debug because it carries bearer material`);
}

const document = read(resolve(root, "docs/migration/A1_6_REFRESH_DIGEST_REPLAY.md"));
const normalizedDocument = document.replace(/\s+/g, " ");
for (const anchor of [
  "combined hermetic schema/runtime proof",
  "strict `rt1` credential",
  "Version 1 dual-write is deliberately not admitted",
  "There is no rolling mixed-version deployment for this wave",
  "force every pre-A1.6 session to authenticate again",
  "No active legacy row may be silently assigned to a key version",
  "No database, Redis, network, service, browser, container, Kubernetes, migration, or deployment target is contacted"
]) if (!normalizedDocument.includes(anchor)) fail(`A1.6 documentation missing ${JSON.stringify(anchor)}`);

console.log("refresh-digest-replay: PASS — schema checksums, 7/7 column guards, 8/8 constraints, 10/10 schema invariants, 10/10 runtime invariants, digest-only runtime composition, 72/72 anchors, and 10/10 STOPs verified");
' -- "$EVIDENCE_ROOT" "$CONTRACT" "$UP_SQL" "$DOWN_SQL"

if [[ "$MODE" == "readiness" ]]; then
  echo "refresh-digest-replay: STOP — core collision, PostgreSQL/MVCC, drained cutover, legacy zero-raw, persistent key lifecycle, and production proofs remain blocked" >&2
  exit 3
fi

echo "refresh-digest-replay: LIMIT — hermetic schema/runtime source evidence only; no database, secret, migration, service, browser, or production action ran"
