#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
CONTRACT="$REPO_ROOT/docs/migration/contracts/refresh-client-binding.json"
MIGRATION_DIR="$REPO_ROOT/apps/backend/migrations/core/20260723000000_bind_refresh_tokens_to_client"
UP_SQL="$MIGRATION_DIR/up.sql"
DOWN_SQL="$MIGRATION_DIR/down.sql"
MODE="evidence"

die() {
  echo "refresh-client-binding: ERROR: $*" >&2
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

for name in DATABASE_URL TEST_DATABASE_URL REDIS_URL IDENTITY_DATABASE_URL API_URL BACKEND_URL; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no live I/O"
done

bun -e '
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const [root, contractPath, upPath, downPath] = process.argv.slice(1);
const fail = (message) => {
  console.error(`refresh-client-binding: ERROR: ${message}`);
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

if (contract.schemaVersion !== 1 || contract.contractId !== "A1.5-refresh-client-binding") fail("unexpected contract identity");
if (contract.status !== "partial-hermetic-proof" || contract.productionReady !== false || contract.databaseProof !== false) fail("contract must remain partial, non-production, and database-unproved");
if (contract.authority?.runtime !== "shared/rust/epsx-identity-shared") fail("compiled runtime authority drifted");
if (contract.migration?.mode !== "additive-expand-forward-only") fail("migration mode drifted");
if (!contract.migration.legacyPolicy.includes("never guessed, backfilled, or claimed") || !contract.migration.legacyPolicy.includes("force fresh authentication")) fail("legacy fail-closed policy drifted");
const expectedInvariants = new Map([
  ["closed-client-set", "Only exact epsx-frontend and epsx-admin client values are accepted."],
  ["initial-binding-and-family", "Initial refresh issuance stores the validated client and a new per-login family identifier only after every JWT signature succeeds."],
  ["sign-before-consume", "Refresh dependency reads and every fallible JWT signature finish before the destructive rotation transition."],
  ["family-serialized-rotation", "Rotation acquires a transaction-scoped advisory lock keyed by the stored family, conditionally consumes token, wallet, client, family, active state, and expiry, then inserts the exact pre-signed successor in one transaction."],
  ["generic-fail-closed-rejection", "Cross-client, missing-family, and legacy NULL attempts mutate no row and share the invalid-refresh classification used for unknown, expired, revoked, or replayed tokens."],
  ["stored-binding-propagation", "The successor copies the client and family returned by the database transition, and JWT issuance uses the preflight stored client rather than the request DTO."],
  ["stable-auth-time", "The rotation chain preserves original authentication time rather than advancing auth_time on every refresh."],
  ["family-scoped-logout", "Logout and rotation share the transaction-scoped family advisory lock; any historical token revokes only its own active family, while a legacy NULL-family token revokes only itself."],
  ["http-refresh-boundary", "The HTTP refresh request requires an explicit client, emits a closed rotated, not_rotated, rejected, or outcome_unknown marker, is non-cacheable, and never echoes credentials in error bodies."],
  ["bff-cookie-and-status-boundary", "Frontend and admin BFFs use fixed clients and distinct local cookie names; ambiguous legacy names are clearing-only, and cookies are preserved only for an exact backend-authored not_rotated outcome."]
]);
if (!Array.isArray(contract.invariants) || contract.invariants.length !== expectedInvariants.size) fail("ten runtime invariants are required");
const observedInvariantIds = new Set();
for (const invariant of contract.invariants) {
  if (!invariant || typeof invariant.id !== "string" || typeof invariant.claim !== "string") fail("runtime invariants require stable id and claim fields");
  if (observedInvariantIds.has(invariant.id)) fail(`duplicate invariant id ${invariant.id}`);
  observedInvariantIds.add(invariant.id);
  if (expectedInvariants.get(invariant.id) !== invariant.claim) fail(`runtime invariant drifted for ${invariant.id}`);
}
for (const id of expectedInvariants.keys()) if (!observedInvariantIds.has(id)) fail(`missing runtime invariant ${id}`);
const expectedStops = new Map([
  ["core-migration-version-collision", "No migration in the active core root may be executed until its duplicate baseline-version collision is reconciled."],
  ["postgres-forward-only-migration-unproved", "No disposable PostgreSQL instance has applied this expansion from each supported history or proved that attempted down migration fails atomically without schema or data change."],
  ["legacy-row-cutover-unproved", "Populated legacy-row preservation and the later active-NULL revocation cutover are unproved."],
  ["postgres-client-and-rotation-unproved", "PostgreSQL cross-client non-consumption, MVCC single-winner rotation, and deterministic family-lock serialization are unproved."],
  ["failure-rollback-and-restart-unproved", "Rollback on forced successor-insert failure and restart persistence are unproved."],
  ["a1-6-postgres-digest-replay-unproved", "No disposable PostgreSQL instance has applied A1.6 or proven keyed-digest lookup, terminal-state transitions, replay response, and fail-closed startup."],
  ["a1-6-cutover-key-lifecycle-unproved", "The required drained forced-reauthentication cutover, legacy plaintext reconciliation, persistent secret provisioning, restart persistence, production key rotation, and safe key retirement are unproved and unauthorized."],
  ["logout-rotation-db-ordering-unproved", "The shared transaction-scoped family advisory lock has no two-connection PostgreSQL proof for logout-first, rotation-first, stale-token logout, or distinct-family isolation."],
  ["access-token-post-logout-validity", "Already-issued access tokens remain valid until expiry after logout."],
  ["production-actions-unauthorized", "No production issuer, browser, routing, canary, rollback, deployment, database, or service action is authorized by this contract."]
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
if (contract.hermeticProof?.focusedTests !== 101 || contract.hermeticProof?.commands?.length !== 5) fail("hermetic proof inventory drifted");

const up = read(upPath);
const down = read(downPath);
if (sha256(up) !== contract.migration.upSha256) fail("up migration checksum drifted");
if (sha256(down) !== contract.migration.downSha256) fail("down migration checksum drifted");
if (/\b(DROP|DELETE|TRUNCATE|UPDATE|INSERT|MERGE)\b/i.test(up)) fail("expand migration contains destructive or data-mutating SQL");
for (const anchor of [
  "ADD COLUMN IF NOT EXISTS client_id VARCHAR(32)",
  "ADD COLUMN IF NOT EXISTS family_id UUID",
  "ALTER TABLE public.openid_refresh_tokens",
  "information_schema.columns",
  "column_maximum_length IS DISTINCT FROM 32",
  "column_default IS NOT NULL",
  "column_is_identity IS DISTINCT FROM \u0027NO\u0027",
  "column_is_generated IS DISTINCT FROM \u0027NEVER\u0027",
  "column_data_type IS DISTINCT FROM \u0027uuid\u0027",
  "IF EXISTS (",
  "pre-existing openid_refresh_tokens_client_id_check is refused",
  "client_id IS NULL",
  "client_id IN (\u0027epsx-frontend\u0027, \u0027epsx-admin\u0027)",
  "NOT VALID",
  "VALIDATE CONSTRAINT openid_refresh_tokens_client_id_check",
  "AND convalidated",
  "CREATE INDEX openid_refresh_tokens_family_id_idx"
]) if (!up.includes(anchor)) fail(`up migration missing ${JSON.stringify(anchor)}`);
if (!down.includes("RAISE EXCEPTION") || !down.includes("forward-only") || /\bDROP\b/i.test(down)) fail("down migration must refuse destructive rollback");

const expectedEvidence = [
  { file: "shared/rust/epsx-identity-shared/src/token_service.rs", anchors: ["enum RefreshClient", "refresh_client_matching_fails_closed_for_cross_client_and_legacy_rows", "openid_refresh_tokens::client_id.eq(Some(client.as_str()))", "pub(crate) fn issue_refresh_token(&self) -> IssuedRefreshToken", "let family_id = Uuid::new_v4();", "issue_tokens_for_user_with_refresh_token(", ".consume_refresh_token(", "Self::lock_refresh_family(conn, expected_family_id)", "openid_refresh_tokens::family_id.eq(Some(expected_family_id))", "openid_refresh_tokens::client_id.eq(Some(stored_client_id.as_str()))", "openid_refresh_tokens::family_id.eq(Some(family_id))", "openid_refresh_tokens::created_at.eq(&created_at)", "pub async fn revoke_refresh_token", "Self::lock_refresh_family(conn, family_id)", "pg_advisory_xact_lock(hashtextextended"] },
  { file: "shared/rust/epsx-identity-shared/src/auth_service.rs", anchors: [".validate_refresh_token(refresh_token, client_id)", ".issue_tokens_for_user_with_refresh_token(", ".consume_refresh_token(", "candidate.family_id", "Web3AuthError::InvalidRefreshToken"] },
  { file: "apps/backend/src/web/auth/handlers.rs", anchors: ["pub client_id: String", "StatusCode::SERVICE_UNAVAILABLE", "AUTH_SESSION_CACHE_CONTROL", "refresh_request_requires_an_explicit_supported_client"] },
  { file: "shared/rust/bff/src/cookies.rs", anchors: ["epsx.frontend.refresh_token", "epsx.admin.refresh_token", "LEGACY_LOCAL_REFRESH_COOKIE", "local_frontend_and_admin_cookie_names_do_not_collide"] },
  { file: "apps/frontend/src/api.rs", anchors: ["client_id: FRONTEND_CLIENT_ID", "CookieClient::Frontend", "private, no-store", "response.status() == StatusCode::UNAUTHORIZED"] },
  { file: "apps/admin/src/session_auth.rs", anchors: ["client_id: ADMIN_CLIENT_ID", "CookieClient::Admin", "private, no-store", "response.status() == StatusCode::UNAUTHORIZED"] },
  { file: "shared/rust/epsx-identity-shared/src/schemas.rs", anchors: ["client_id -> Nullable<Varchar>", "family_id -> Nullable<Uuid>"] },
  { file: "apps/backend/src/schemas/primary.rs", anchors: ["client_id -> Nullable<Varchar>", "family_id -> Nullable<Uuid>"] },
  { file: "e2e/frontend/utils/auth-mock.ts", anchors: ["epsx.frontend.access_token", "epsx.frontend.refresh_token"] },
  { file: "e2e/admin/utils/auth-mock.ts", anchors: ["epsx.admin.access_token", "epsx.admin.refresh_token"] },
  { file: "e2e/frontend/fixtures/about-auth-mock.mjs", anchors: ["/api/v1/notification/unread-count"] },
  { file: "e2e/frontend/fixtures/developer-docs-auth-mock.mjs", anchors: ["/api/v1/notification/unread-count"] },
  { file: "e2e/admin/denial-runtime.spec.ts", anchors: ["epsx.admin.access_token", "page.waitForRequest(", "authRequestUrl.searchParams.get(\u0027return_url\u0027)"] }
];
if (JSON.stringify(contract.evidence) !== JSON.stringify(expectedEvidence)) fail("evidence inventory drifted from the exact pinned file/anchor map");
let evidenceAnchors = 0;
for (const item of expectedEvidence) {
  if (!item || typeof item.file !== "string" || item.file.startsWith("/") || item.file.split("/").includes("..")) fail("invalid evidence path");
  const contents = read(resolve(root, item.file));
  for (const anchor of item.anchors ?? []) {
    if (!contents.includes(anchor)) fail(`${item.file}: missing evidence anchor ${JSON.stringify(anchor)}`);
    evidenceAnchors += 1;
  }
}
if (evidenceAnchors !== 49) fail("evidence anchor count drifted");

const tokenService = read(resolve(root, "shared/rust/epsx-identity-shared/src/token_service.rs"));
const consumeStart = tokenService.indexOf("pub(crate) async fn consume_refresh_token");
const consumeEnd = tokenService.indexOf("/// Validate Access Token", consumeStart);
if (consumeStart < 0 || consumeEnd < 0) fail("canonical consume boundary is missing");
const consume = tokenService.slice(consumeStart, consumeEnd);
const clientFilter = consume.indexOf("openid_refresh_tokens::client_id");
const mutation = consume.indexOf("openid_refresh_tokens::consumed_at.eq(Some(now))");
if (clientFilter < 0 || mutation < 0 || clientFilter > mutation) fail("stored-client comparison must precede token mutation");
const normalizedConsume = consume.replace(/\s+/g, " ");
if (!normalizedConsume.includes(".set(( openid_refresh_tokens::is_revoked.eq(true), openid_refresh_tokens::consumed_at.eq(Some(now)), )) .returning((")) fail("consume must atomically publish the revoked/consumed tuple before returning predecessor state");
if (!consume.includes("requested_client_id.as_str()") || !consume.includes("expected_wallet_address")) fail("rotation predicate is not bound to client and preflight wallet");
if (!consume.includes("client_id.eq(Some(stored_client_id.as_str()))")) fail("successor does not copy the returned stored client");
const familyLock = consume.indexOf("Self::lock_refresh_family(conn, expected_family_id)");
if (familyLock < 0 || familyLock > mutation || !consume.includes("family_id.eq(Some(expected_family_id))") || !consume.includes("family_id.eq(Some(family_id))")) fail("rotation must lock, match, and propagate its stored family before mutation");

const refreshStart = tokenService.indexOf("pub async fn refresh_tokens");
const refreshEnd = tokenService.indexOf("/// Revoke", refreshStart);
const refreshFlow = tokenService.slice(refreshStart, refreshEnd);
const signBeforeConsume = refreshFlow.indexOf("issue_tokens_for_user_with_refresh_token(");
const consumeAfterSign = refreshFlow.indexOf(".consume_refresh_token(");
if (signBeforeConsume < 0 || consumeAfterSign < 0 || signBeforeConsume > consumeAfterSign) fail("canonical refresh must sign before consuming the predecessor");
const initialStart = tokenService.indexOf("pub async fn issue_tokens_for_user(");
const initialEnd = tokenService.indexOf("/// Build OpenID Connect tokens", initialStart);
const initialFlow = tokenService.slice(initialStart, initialEnd);
const initialSign = initialFlow.indexOf("issue_tokens_for_user_with_refresh_token(");
const initialPersist = initialFlow.indexOf(".create_refresh_token(");
if (initialStart < 0 || initialEnd < 0 || initialSign < 0 || initialPersist < 0 || initialSign > initialPersist) fail("initial issuance must sign before publishing the refresh row");

const revokeStart = tokenService.indexOf("pub async fn revoke_refresh_token");
const revokeEnd = tokenService.indexOf("/// Atomically consume", revokeStart);
const revokeFlow = tokenService.slice(revokeStart, revokeEnd);
if (!revokeFlow.includes("Self::lock_refresh_family(conn, family_id)") || !revokeFlow.includes("family_id.eq(Some(family_id))") || !revokeFlow.includes("token_id.eq(&presented_token)")) fail("logout must share the family lock, revoke only that family, and bound legacy logout to the exact token");
const familyHelperStart = tokenService.indexOf("async fn lock_refresh_family");
const familyHelperEnd = tokenService.indexOf("/// Store an already-generated refresh token", familyHelperStart);
const familyHelper = tokenService.slice(familyHelperStart, familyHelperEnd);
if (familyHelperStart < 0 || familyHelperEnd < 0 || !familyHelper.includes("pg_advisory_xact_lock(hashtextextended($1::text, 0))") || !familyHelper.includes(".bind::<SqlUuid, _>(family_id)") || !familyHelper.includes("WITH family_lock AS MATERIALIZED")) fail("transaction-scoped refresh-family advisory lock drifted");

const authService = read(resolve(root, "shared/rust/epsx-identity-shared/src/auth_service.rs"));
const authRefreshStart = authService.indexOf("pub async fn refresh_tokens(");
const authRefreshEnd = authService.indexOf("\n}\n\nfn map_refresh_error", authRefreshStart);
const authRefreshFlow = authService.slice(authRefreshStart, authRefreshEnd);
const authSign = authRefreshFlow.indexOf("issue_tokens_for_user_with_refresh_token(");
const authConsume = authRefreshFlow.indexOf(".consume_refresh_token(");
if (authRefreshStart < 0 || authRefreshEnd < 0 || authSign < 0 || authConsume < 0 || authSign > authConsume) fail("canonical auth service must sign before consuming the predecessor");

const handler = read(resolve(root, "apps/backend/src/web/auth/handlers.rs"));
if (handler.includes("request.client_id.unwrap_or_else")) fail("implicit frontend refresh default returned");
if (!handler.includes("pub client_id: String")) fail("refresh client is not required by the handler DTO");
if (!handler.includes("Web3AuthError::InvalidRefreshToken") || !handler.includes("StatusCode::SERVICE_UNAVAILABLE")) fail("refresh error classification drifted");
for (const outcome of ["REFRESH_OUTCOME_ROTATED", "REFRESH_OUTCOME_NOT_ROTATED", "REFRESH_OUTCOME_REJECTED", "REFRESH_OUTCOME_UNKNOWN"]) {
  if (!handler.includes(outcome)) fail(`backend refresh outcome marker missing: ${outcome}`);
}

const outcomeClassifier = read(resolve(root, "shared/rust/bff/src/refresh_outcome.rs"));
if (!outcomeClassifier.includes("Some(REFRESH_OUTCOME_NOT_ROTATED)") || !outcomeClassifier.includes("RefreshDisposition::Preserve")) fail("exact not-rotated preservation contract drifted");
if (!outcomeClassifier.includes("_ => RefreshDisposition::Clear")) fail("missing or inconsistent refresh outcomes must fail closed");

for (const file of ["apps/frontend/src/api.rs", "apps/admin/src/session_auth.rs"]) {
  const bff = read(resolve(root, file));
  if (!bff.includes("classify_refresh_outcome(status, response.headers())")) fail(`${file}: shared outcome classifier is not used`);
  if (!bff.includes("refresh_outcome_unknown") || !bff.includes("clear_session_response")) fail(`${file}: ambiguous outcomes do not clear locally`);
}

for (const [file, client] of [
  ["e2e/frontend/utils/auth-mock.ts", "frontend"],
  ["e2e/frontend/about-runtime.spec.ts", "frontend"],
  ["e2e/frontend/developer-docs-runtime.spec.ts", "frontend"],
  ["e2e/admin/utils/auth-mock.ts", "admin"],
  ["e2e/admin/denial-runtime.spec.ts", "admin"],
  ["e2e/admin/web3-auth-e2e.spec.ts", "admin"]
]) {
  const fixture = read(resolve(root, file));
  if (!fixture.includes(`epsx.${client}.access_token`) || fixture.includes("epsx.access_token") || fixture.includes("epsx.refresh_token")) fail(`${file}: local browser fixture cookie namespace drifted`);
}

const sharedSchema = read(resolve(root, "shared/rust/epsx-identity-shared/src/schemas.rs"));
const backendSchema = read(resolve(root, "apps/backend/src/schemas/primary.rs"));
for (const [name, schema] of [["shared", sharedSchema], ["backend", backendSchema]]) {
  if (!schema.includes("client_id -> Nullable<Varchar>")) fail(`${name} Diesel schema lacks nullable client expansion`);
  if (!schema.includes("family_id -> Nullable<Uuid>")) fail(`${name} Diesel schema lacks nullable family expansion`);
}

console.log(`refresh-client-binding: PASS — migration checksums/catalog guards, 10/10 pinned invariants, 49/49 pinned anchors, sign-before-consume, family-serialized rotation/logout, scoped fixtures, and stable STOP claims verified`);
' -- "$REPO_ROOT" "$CONTRACT" "$UP_SQL" "$DOWN_SQL"

if [[ "$MODE" == "readiness" ]]; then
  echo "refresh-client-binding: STOP — PostgreSQL, A1.5 legacy enforcement, A1.6 forced-reauthentication/key lifecycle, and production proofs remain blocked" >&2
  exit 3
fi

echo "refresh-client-binding: LIMIT — static/hermetic evidence only; no migration, database, service, browser, or production action ran"
