#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT="${EPSX_A1_7_EVIDENCE_ROOT:-$REPO_ROOT}"
CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/refresh-outcome-coordination.json"
MODE="evidence"

die() {
  echo "refresh-outcome-coordination: ERROR: $*" >&2
  exit 1
}

if (( $# == 2 )) && [[ "$1" == "--mode" ]]; then
  MODE="$2"
elif (( $# != 0 )); then
  die "usage: $0 [--mode evidence|readiness]"
fi
[[ "$MODE" == "evidence" || "$MODE" == "readiness" ]] || die "mode must be evidence or readiness"
command -v bun >/dev/null 2>&1 || die "bun is required"

for name in DATABASE_URL TEST_DATABASE_URL REDIS_URL IDENTITY_DATABASE_URL API_URL BACKEND_URL; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no live I/O"
done

bun -e '
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { createHash } from "node:crypto";

const [root, contractPath] = process.argv.slice(1);
const fail = (message) => {
  console.error(`refresh-outcome-coordination: ERROR: ${message}`);
  process.exit(1);
};
const read = (path) => {
  try { return readFileSync(path, "utf8"); }
  catch (error) { fail(`cannot read ${path}: ${error.message}`); }
};
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
    if (source[index] === "\u0027") {
      const start = index++;
      let escaped = false;
      while (index < source.length) {
        const character = source[index++];
        if (character === "\u0027" && !escaped) break;
        if (character === "\\" && !escaped) escaped = true;
        else escaped = false;
      }
      output += source.slice(start, index);
      continue;
    }
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      index = end < 0 ? source.length : end;
      continue;
    }
    if (source.startsWith("/*", index)) {
      index += 2;
      let depth = 1;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) { depth += 1; index += 2; }
        else if (source.startsWith("*/", index)) { depth -= 1; index += 2; }
        else index += 1;
      }
      if (depth !== 0) fail("unterminated Rust block comment while stripping comments");
      continue;
    }
    output += source[index++];
  }
  return output;
};

let contract;
try { contract = JSON.parse(read(contractPath)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.contractId !== "A1.7-refresh-outcome-coordination") fail("unexpected contract identity");
if (contract.status !== "partial-hermetic-runtime-proof") fail("contract status drifted");
if (contract.productionReady !== false || contract.browserHermeticProof !== true || contract.browserRuntimeProof !== false || contract.databaseProof !== false || contract.deploymentAuthorized !== false) fail("contract must remain hermetic-only, non-production, database-unproved, and deployment-unauthorized");
if (contract.sourceBaseline?.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || contract.targetBase?.commit !== "206a8fb57f982bcbc59d926e64506d4a5d654e40") fail("baseline evidence drifted");

const expectedInvariants = new Map([
  ["closed-backend-outcome", "Every canonical backend refresh response is no-store and carries exactly rotated, not_rotated, rejected, or outcome_unknown."],
  ["status-is-not-mutation-proof", "A status without an exact consistent outcome marker never proves that rotation did not commit."],
  ["preserve-only-not-rotated", "The BFF preserves cookies only for exact not_rotated responses on the closed 400, 500, or 503 matrix."],
  ["ambiguous-outcome-clears", "Rejected, outcome_unknown, missing, invalid, intermediary, transport, and malformed post-rotation outcomes clear locally without refresh retry."],
  ["verified-rotation-replaces-pair", "Only a rotated 200 whose body, JWT, identity, and cookie lifetimes all validate replaces the access and refresh pair."],
  ["credential-redirects-disabled", "Credential-bearing authentication requests do not follow upstream redirects."],
  ["shared-origin-mutation-lock", "Refresh and logout use one origin-scoped exclusive Web Lock, and unsupported locks refuse refresh before fetch."],
  ["single-flight-no-retry", "One same-window refresh promise performs exactly one refresh fetch and has no automatic retry path."],
  ["token-free-cross-tab-events", "Cross-tab session messages use a fixed versioned schema and contain no credential, identity, permission, or entitlement data."],
  ["confirmed-logout-navigation", "Logout redirects and broadcasts session end only after the BFF confirms local cookie clearing."],
  ["shared-controller-reachability", "Authenticated customer mobile/desktop actions, both active admin chrome variants, the connected-wallet hook, and admin denial actions delegate to the shared session controller."],
  ["backend-policy-authority-retained", "Browser and BFF changes contain no permission, plan, ranking-offset, feature-flag, subscription, or entitlement authority."]
]);
if (!Array.isArray(contract.invariants) || contract.invariants.length !== expectedInvariants.size) fail("twelve exact invariants are required");
for (const item of contract.invariants) {
  if (expectedInvariants.get(item?.id) !== item?.claim) fail(`invariant drifted: ${item?.id}`);
  expectedInvariants.delete(item.id);
}
if (expectedInvariants.size !== 0) fail("one or more invariants are missing");

const expectedStops = new Map([
  ["real-browser-matrix-unproved", "No accepted real-browser matrix proves Web Locks, BroadcastChannel, cookie application, navigation, or multi-tab behavior."],
  ["post-commit-fault-unproved", "No proxy or browser fault injection proves timeout or reset behavior after backend commit."],
  ["postgres-refresh-ordering-unproved", "No PostgreSQL exercise proves consume, replay response, commit acknowledgement ambiguity, or family-lock ordering."],
  ["exactly-once-delivery-unproved", "A disconnect after backend rotation but before Set-Cookie delivery still forces reauthentication and has no receipt or idempotency protocol."],
  ["bff-unreachable-clear-unproved", "JavaScript cannot prove clearing HttpOnly cookies while the BFF itself is unreachable."],
  ["automatic-refresh-entrypoint-unproved", "The shared refresh controller has no automatic customer/admin runtime caller, so expired-cookie recovery still requires explicit reauthentication."],
  ["access-token-post-revocation-validity", "Already-issued access tokens remain valid until expiry after logout or family revocation."],
  ["production-actions-unauthorized", "No production browser, proxy, TLS, routing, canary, rollback, deployment, database, secret, or service action is authorized by this contract."]
]);
if (!Array.isArray(contract.residualStops) || contract.residualStops.length !== expectedStops.size) fail("eight exact residual STOPs are required");
for (const item of contract.residualStops) {
  if (expectedStops.get(item?.id) !== item?.claim) fail(`residual STOP drifted: ${item?.id}`);
  expectedStops.delete(item.id);
}
if (expectedStops.size !== 0) fail("one or more residual STOPs are missing");

let anchors = 0;
if (!Array.isArray(contract.evidence) || contract.evidence.length !== 17) fail("seventeen evidence files are required");
const evidenceDigest = createHash("sha256").update(JSON.stringify(contract.evidence)).digest("hex");
if (evidenceDigest !== "01df307f939518812f0185b1834281b114197843f19d31b46860e009dda6450d") fail("exact evidence file/anchor inventory drifted");
for (const item of contract.evidence) {
  if (!item.file || item.file.startsWith("/") || item.file.split("/").includes("..")) fail(`invalid evidence path: ${item.file}`);
  const content = read(resolve(root, item.file));
  if (!Array.isArray(item.anchors) || item.anchors.length === 0) fail(`${item.file}: anchors required`);
  for (const anchor of item.anchors) {
    if (!content.includes(anchor)) fail(`${item.file}: missing evidence anchor ${JSON.stringify(anchor)}`);
    anchors += 1;
  }
}
if (anchors !== 65) fail(`exactly 65 evidence anchors are required, found ${anchors}`);

const handler = normalize(stripRustComments(read(resolve(root, "apps/backend/src/web/auth/handlers.rs"))));
for (const marker of ["REFRESH_OUTCOME_ROTATED", "REFRESH_OUTCOME_NOT_ROTATED", "REFRESH_OUTCOME_REJECTED", "REFRESH_OUTCOME_UNKNOWN"]) {
  if (!handler.includes(marker)) fail(`backend marker missing: ${marker}`);
}
const dependencyStart = handler.indexOf("Err(Web3AuthError::DatabaseError(error) | Web3AuthError::BlockchainError(error))");
const signingStart = handler.indexOf("Err(Web3AuthError::TokenGenerationFailed(error))", dependencyStart);
if (dependencyStart < 0 || signingStart < 0) fail("backend dependency/signing outcome branches missing");
const dependencyBranch = handler.slice(dependencyStart, signingStart);
if (!dependencyBranch.includes("REFRESH_OUTCOME_UNKNOWN") || dependencyBranch.includes("REFRESH_OUTCOME_NOT_ROTATED")) fail("database or blockchain ambiguity must never claim not_rotated");
const signingBranch = handler.slice(signingStart, handler.indexOf("Err(error)", signingStart));
if (!signingBranch.includes("REFRESH_OUTCOME_NOT_ROTATED")) fail("pre-rotation signing failure must remain explicitly not_rotated");

const classifier = normalize(stripRustComments(read(resolve(root, "shared/rust/bff/src/refresh_outcome.rs"))));
if (!classifier.includes("StatusCode::BAD_REQUEST | StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE, Some(REFRESH_OUTCOME_NOT_ROTATED), ) => RefreshDisposition::Preserve")) fail("closed preserve matrix drifted");
if (!classifier.includes("_ => RefreshDisposition::Clear")) fail("unknown classifier outcomes must clear");
if (!classifier.includes("StatusCode::OK, Some(REFRESH_OUTCOME_ROTATED)")) fail("rotated success classifier drifted");
if (!classifier.includes("headers.get_all(REFRESH_OUTCOME_HEADER).iter()") || !classifier.includes("values.next().is_none().then_some(outcome)")) fail("refresh outcome must require exactly one header value");

for (const file of ["apps/frontend/src/api.rs", "apps/admin/src/session_auth.rs"]) {
  const app = normalize(stripRustComments(read(resolve(root, file))));
  const start = app.indexOf("pub async fn refresh_token(");
  const end = app.indexOf("fn refresh_response(", start);
  const flow = app.slice(start, end);
  if (start < 0 || end < 0 || !flow.includes("classify_refresh_outcome(status, response.headers())")) fail(`${file}: shared classifier missing`);
  const sendStart = flow.indexOf(".send() .await");
  const sendErrorStart = flow.indexOf("Err(error) =>", sendStart);
  const sendErrorEnd = flow.indexOf("} };", sendErrorStart);
  const sendError = flow.slice(sendErrorStart, sendErrorEnd);
  if (sendStart < 0 || sendErrorStart < 0 || sendErrorEnd < 0 || !sendError.includes("refresh_outcome_unknown") || !sendError.includes("clear_refresh_session_response")) fail(`${file}: transport ambiguity does not clear`);
  if (!flow.includes("RefreshDisposition::Preserve") || !flow.includes("refresh_not_rotated")) fail(`${file}: exact preserve response missing`);
  if (flow.includes("is_connect()") || flow.includes(".retry")) fail(`${file}: inferred pre-dispatch preservation or retry returned`);
  if (!app.includes(".auth_client()")) fail(`${file}: redirect-disabled auth client is not used`);
  const clearStart = app.indexOf("fn clear_refresh_session_response(");
  const clearEnd = app.indexOf("fn try_clear_session_response(", clearStart);
  const clearFlow = app.slice(clearStart, clearEnd);
  if (clearStart < 0 || clearEnd < 0 || !clearFlow.includes("Ok(response) => refresh_response(response, RefreshDisposition::Clear)") || !clearFlow.includes("Err(error) => error")) fail(`${file}: cleared marker can be emitted without successful cookie clearing`);
}

const browserFile = stripRustComments(read(resolve(root, "shared/rust/bff/src/browser_auth.rs")));
const rawMatch = browserFile.match(/pub fn browser_auth_script\(\) -> &\u0027static str \{\s*r#"\n([\s\S]*?)\n"#\s*\}/);
if (!rawMatch) fail("browser bridge raw string missing");
const browser = rawMatch[1];
if ((browser.match(/fetch\(\u0027\/api\/v1\/auth\/refresh\u0027/g) ?? []).length !== 1) fail("browser refresh fetch must occur exactly once in source");
for (const anchor of [
  "navigator.locks.request(epsxSessionLockName, { mode: \u0027exclusive\u0027 }, operation)",
  "epsxWithSessionMutation(epsxRefreshOnce, true)",
  "function epsxLogoutSession(target)",
  "Session refresh requires cross-tab coordination",
  "if (epsxRefreshPromise) return epsxRefreshPromise",
  "{ version: 1, type: type }",
  "response.headers.get(\u0027x-epsx-session-state\u0027)",
  "event.target.closest(\u0027[data-epsx-logout]\u0027)",
  "try { epsxSessionChannel.postMessage(event); } catch (_) {}",
  "if (response.headers.get(\u0027x-epsx-session-state\u0027) !== \u0027cleared\u0027) return false"
]) if (!browser.includes(anchor)) fail(`browser coordination anchor missing: ${anchor}`);
for (const forbidden of ["localStorage", "sessionStorage", "access_token", "refresh_token"]) if (browser.includes(forbidden)) fail(`browser bridge contains forbidden material: ${forbidden}`);
const clearCheck = browser.indexOf("!== \u0027cleared\u0027");
const logoutRedirect = browser.indexOf("epsxEndLocalSession(\u0027logout\u0027, safeTarget)");
if (clearCheck < 0 || logoutRedirect < clearCheck) fail("logout redirects before local clear confirmation");

const client = normalize(stripRustComments(read(resolve(root, "shared/rust/client/src/lib.rs"))));
const clientNewStart = client.indexOf("pub fn new(config: ClientConfig) -> Self");
const authClientStart = client.indexOf("pub fn auth_client(&self) -> reqwest::Client");
const authClientEnd = client.indexOf("pub fn base_url", authClientStart);
if (clientNewStart < 0 || authClientStart < 0 || authClientEnd < 0 || !client.slice(clientNewStart, authClientStart).includes("redirect(reqwest::redirect::Policy::none())") || !client.slice(authClientStart, authClientEnd).includes("self.auth_inner.clone()")) fail("pooled auth client redirect refusal drifted");

const stripJavaScriptComments = (source) => source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/[^\n]*/gm, "$1");
const adminSsr = stripRustComments(read(resolve(root, "apps/admin/src/ssr.rs")));
const denialStart = adminSsr.indexOf("fn admin_denial_runtime_script()");
const denialEnd = adminSsr.indexOf("#[cfg(test)]", denialStart);
const denial = normalize(stripJavaScriptComments(adminSsr.slice(denialStart, denialEnd)));
if (!denial.includes("window.epsxAuth.logout(target)") || denial.includes("fetch(\u0027/api/v1/auth/logout\u0027")) fail("admin denial action bypasses the shared controller");

const template = normalize(stripRustComments(read(resolve(root, "shared/rust/templates/src/lib.rs"))));
const headerStart = template.indexOf("pub fn epsx_header_for_session(is_authenticated: bool)");
const headerEnd = template.indexOf("pub fn page_shell(", headerStart);
const header = template.slice(headerStart, headerEnd);
if (headerStart < 0 || headerEnd < 0) fail("truthful authenticated header function boundary drifted");
if (!header.includes("data-epsx-logout")) fail("authenticated header logout hook drifted");
if (!header.includes("href=\"/account\"")) fail("authenticated header account link drifted");
if (!header.includes("data-epsx-authenticated=\"{authenticated}\"")) fail("mobile session marker drifted");
for (const forbidden of ["permission", "ranking_offset", "feature_flag", "subscription", "entitlement"]) if (header.includes(forbidden)) fail(`header acquired backend-only policy authority: ${forbidden}`);

const adminShell = normalize(stripRustComments(read(resolve(root, "shared/rust/dioxus_ui/src/layout/admin_shell.rs"))));
if (!adminShell.includes("Header {") || !adminShell.includes("html.contains(\"data-epsx-logout=\\\"true\\\"\")")) {
  fail("shared/rust/dioxus_ui/src/layout/admin_shell.rs: composed authenticated admin chrome logout hook drifted");
}
const walletButton = normalize(stripRustComments(read(resolve(root, "shared/rust/dioxus_ui/src/auth/wallet_button.rs"))));
if (!walletButton.includes("\"data-epsx-logout\": \"true\"")) {
  fail("shared/rust/dioxus_ui/src/auth/wallet_button.rs: authenticated admin chrome logout hook drifted");
}

console.log(`refresh-outcome-coordination: PASS 12/12 invariants, ${anchors}/${anchors} anchors, 8/8 residual STOPs`);
' -- "$EVIDENCE_ROOT" "$CONTRACT" || exit 1

bun "$EVIDENCE_ROOT/scripts/migration/test-browser-session-coordination.js" || exit 1

if [[ "$MODE" == "readiness" ]]; then
  echo "refresh-outcome-coordination: STOP — real browser, proxy fault, PostgreSQL, cookie-delivery, and production proof remain absent" >&2
  exit 3
fi

echo "refresh-outcome-coordination: PASS — hermetic evidence only; production readiness is not claimed"
