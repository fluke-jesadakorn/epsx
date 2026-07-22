#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
EVIDENCE_ROOT_RAW="${EPSX_A1_8_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
MODE="evidence"

die() {
  echo "automatic-session-recovery: ERROR: $*" >&2
  exit 1
}

REPO_ROOT="$(CDPATH= cd -- "$REPO_ROOT_RAW" && pwd -P)" || die "repository root is unavailable"
EVIDENCE_ROOT="$(CDPATH= cd -- "$EVIDENCE_ROOT_RAW" && pwd -P)" || die "evidence root is unavailable"
CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/automatic-session-recovery.json"

if (( $# == 2 )) && [[ "$1" == "--mode" ]]; then
  MODE="$2"
elif (( $# != 0 )); then
  die "usage: $0 [--mode evidence|readiness]"
fi
[[ "$MODE" == "evidence" || "$MODE" == "readiness" ]] || die "mode must be evidence or readiness"
command -v bun >/dev/null 2>&1 || die "bun is required"
command -v cargo >/dev/null 2>&1 || die "cargo is required"

for name in DATABASE_URL TEST_DATABASE_URL REDIS_URL IDENTITY_DATABASE_URL API_URL BACKEND_URL; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no live I/O"
done

STATIC_ONLY="${EPSX_A1_8_SELF_TEST_STATIC:-0}"
if [[ "$STATIC_ONLY" == "1" && "$EVIDENCE_ROOT" == "$REPO_ROOT" ]]; then
  die "static-only mode is reserved for copied self-test fixtures"
fi
if [[ "$STATIC_ONLY" != "1" && "$EVIDENCE_ROOT" != "$REPO_ROOT" ]]; then
  die "alternate evidence roots are accepted only for copied static self-test fixtures"
fi

bun -e '
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { createHash } from "node:crypto";

const [root, contractPath, staticOnly] = process.argv.slice(1);
const fail = (message) => {
  console.error(`automatic-session-recovery: ERROR: ${message}`);
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
if (contract.schemaVersion !== 1 || contract.contractId !== "A1.8-automatic-session-recovery") fail("unexpected contract identity");
if (contract.status !== "partial-hermetic-runtime-proof") fail("contract status drifted");
if (contract.productionReady !== false || contract.automaticRecoveryEntrypointProof !== true || contract.browserHermeticProof !== true || contract.browserRuntimeProof !== false || contract.databaseProof !== false || contract.deploymentAuthorized !== false) fail("contract must remain hermetic-only, non-production, database-unproved, and deployment-unauthorized");
if (contract.sourceBaseline?.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db" || contract.targetBase?.commit !== "52b7513d8adc5e08ee7fa4ff503249aa2bd715f7") fail("baseline evidence drifted");
if (JSON.stringify(contract.supersedes) !== JSON.stringify([{ contractId: "A1.7-refresh-outcome-coordination", residualStop: "automatic-refresh-entrypoint-unproved" }])) fail("A1.7 supersession drifted");

const expectedInvariants = new Map([
  ["typed-access-verification", "Optional access verification returns exactly verified, missing-or-rejected, or verifier-unavailable."],
  ["verifier-outage-never-rotates", "An observed invalid verifier configuration, unknown key ID after refresh, JWKS transport failure, or malformed JWKS outcome never permits automatic refresh recovery."],
  ["cookie-only-ssr-trigger", "A BFF emits recovery only for missing-or-rejected access plus its own HttpOnly refresh cookie."],
  ["fixed-token-free-bootstrap", "Recovery HTML contains one fixed call and interpolates no credential, identity, permission, plan, or request value."],
  ["page-lifetime-single-flight", "Duplicate recovery callers share one page-lifetime promise that is never reset after success or failure."],
  ["reload-only-after-rotation", "Automatic recovery reloads only after the hardened refresh controller returns an attested rotated success."],
  ["no-lock-no-retry", "Recovery performs no automatic retry and makes zero refresh requests when cross-tab Web Locks are unavailable."],
  ["confirmed-clear-navigation", "Rejected or ambiguous recovery navigates only after A1.7 confirms local cookie clearing."],
  ["siwe-cookie-mutation-serialized", "SIWE cookie establishment and refresh recovery share the same origin-wide session-mutation lock when Web Locks are available."],
  ["private-cache-boundary", "Recovery-bearing HTML and credential-dependent redirects are private no-store and vary on credentials while the frontend offline shell remains public, anonymous, and recovery-free."],
  ["backend-policy-authority-retained", "Automatic recovery contains no permission, plan, ranking-offset, feature-flag, subscription, or entitlement authority."]
]);
if (!Array.isArray(contract.invariants) || contract.invariants.length !== expectedInvariants.size) fail("eleven exact invariants are required");
for (const item of contract.invariants) {
  if (expectedInvariants.get(item?.id) !== item?.claim) fail(`invariant drifted: ${item?.id}`);
  expectedInvariants.delete(item.id);
}
if (expectedInvariants.size !== 0) fail("one or more invariants are missing");

const expectedStops = new Map([
  ["real-browser-matrix-unproved", "No accepted real-browser matrix proves Web Locks, BroadcastChannel, HttpOnly cookie application, reload, redirect, or multi-tab scheduling."],
  ["post-commit-fault-unproved", "No proxy or browser fault injection proves timeout or reset behavior after backend commit."],
  ["postgres-refresh-ordering-unproved", "No PostgreSQL exercise proves consume, replay response, commit acknowledgement ambiguity, or family-lock ordering."],
  ["exactly-once-delivery-unproved", "A disconnect after backend rotation but before Set-Cookie delivery still forces reauthentication and has no receipt or idempotency protocol."],
  ["bff-unreachable-clear-unproved", "JavaScript cannot prove clearing HttpOnly cookies while the BFF itself is unreachable."],
  ["access-token-post-revocation-validity", "Already-issued access tokens remain valid until expiry after logout or family revocation."],
  ["missing-access-authority-preflight-unproved", "A missing access cookie has no key ID to verify before recovery, so a later JWKS outage can still force reauthentication after rotation."],
  ["cross-document-cookie-acceptance-unproved", "No real-browser proof or server-owned attempt guard excludes repeated reloads if refresh cookies are accepted while access cookies are selectively rejected or evicted."],
  ["production-actions-unauthorized", "No production browser, proxy, TLS, routing, canary, rollback, deployment, database, secret, or service action is authorized by this contract."]
]);
if (!Array.isArray(contract.residualStops) || contract.residualStops.length !== expectedStops.size) fail("nine exact residual STOPs are required");
for (const item of contract.residualStops) {
  if (expectedStops.get(item?.id) !== item?.claim) fail(`residual STOP drifted: ${item?.id}`);
  expectedStops.delete(item.id);
}
if (expectedStops.size !== 0) fail("one or more residual STOPs are missing");

if (!Array.isArray(contract.evidence) || contract.evidence.length !== 12) fail("twelve evidence files are required");
const evidenceDigest = createHash("sha256").update(JSON.stringify(contract.evidence)).digest("hex");
if (contract.evidenceDigest !== evidenceDigest || evidenceDigest !== "4a4f37c819fd082cf6637c5ca165d308de533bdea6423f0f196c950ac9d54aee") fail("exact evidence inventory drifted");
let anchors = 0;
for (const item of contract.evidence) {
  if (!item.file || item.file.startsWith("/") || item.file.split("/").includes("..")) fail(`invalid evidence path: ${item.file}`);
  const content = read(resolve(root, item.file));
  if (!Array.isArray(item.anchors) || item.anchors.length === 0) fail(`${item.file}: anchors required`);
  for (const anchor of item.anchors) {
    if (!content.includes(anchor)) fail(`${item.file}: missing evidence anchor ${JSON.stringify(anchor)}`);
    anchors += 1;
  }
}
if (anchors !== 46) fail(`exactly 46 evidence anchors are required, found ${anchors}`);

const session = normalize(stripRustComments(read(resolve(root, "shared/rust/bff/src/session.rs"))));
const enumStart = session.indexOf("pub enum AccessVerification");
const enumEnd = session.indexOf("impl AccessVerification", enumStart);
const accessEnum = session.slice(enumStart, enumEnd);
if (enumStart < 0 || enumEnd < 0) fail("typed access outcome is missing");
for (const variant of ["Verified", "MissingOrRejected", "VerifierUnavailable"]) if (!accessEnum.includes(variant)) fail(`access outcome missing: ${variant}`);
if (!accessEnum.includes("impl fmt::Debug for AccessVerification") || !accessEnum.includes(".field(\"token\", &\"[REDACTED]\")")) fail("typed access outcome debug output can expose the bearer token");
const permitStart = session.indexOf("pub const fn permits_refresh_recovery");
const permitEnd = session.indexOf("impl AccessTokenClaims", permitStart);
const permit = session.slice(permitStart, permitEnd);
if (permitStart < 0 || permitEnd < 0 || !permit.includes("matches!(self, Self::MissingOrRejected)") || permit.includes("Self::VerifierUnavailable)")) fail("only missing-or-rejected may permit recovery");
const outageStart = session.indexOf("pub const fn is_verifier_unavailable");
const outageEnd = session.indexOf("fn validate_jwks", outageStart);
const outage = session.slice(outageStart, outageEnd);
for (const variant of ["Self::InvalidConfiguration(_)", "Self::UnknownKeyId", "Self::JwksFetch(_)", "Self::MalformedJwks(_)"]) if (!outage.includes(variant)) fail(`verifier outage class missing: ${variant}`);
for (const forbidden of ["Self::MalformedToken", "Self::WrongAlgorithm", "Self::MissingKeyId", "Self::Validation"]) if (outage.includes(forbidden)) fail(`credential rejection became verifier outage: ${forbidden}`);
const optionalStart = session.indexOf("pub async fn verify_optional_access_token");
const optionalEnd = session.indexOf("async fn refresh_cache", optionalStart);
const optional = session.slice(optionalStart, optionalEnd);
if (optionalStart < 0 || optionalEnd < 0 || !optional.includes("Err(error) if error.is_verifier_unavailable()") || !optional.includes("Err(_) => AccessVerification::MissingOrRejected")) fail("optional access verification classification drifted");

for (const file of ["apps/frontend/src/auth.rs", "apps/admin/src/auth.rs"]) {
  const auth = normalize(stripRustComments(read(resolve(root, file))));
  const start = auth.indexOf("pub async fn access_verification");
  const end = auth.indexOf("pub async fn current_user", start);
  const flow = auth.slice(start, end);
  if (start < 0 || end < 0 || !flow.includes(".verify_optional_access_token(access_token(headers, environment))")) fail(`${file}: typed optional verifier is not used`);
  if (flow.includes(".verify(&token).await.ok()")) fail(`${file}: verifier errors are collapsed to Option`);
}

const frontendSsr = normalize(stripRustComments(read(resolve(root, "apps/frontend/src/ssr.rs"))));
const frontendRecoveryStart = frontendSsr.indexOf("let recover_session = access_verification.permits_refresh_recovery()");
const frontendRecoveryEnd = frontendSsr.indexOf("let (verified_access_token, user)", frontendRecoveryStart);
const frontendRecovery = frontendSsr.slice(frontendRecoveryStart, frontendRecoveryEnd);
if (frontendRecoveryStart < 0 || frontendRecoveryEnd < 0 || !frontendRecovery.includes("auth::refresh_token(&headers, state.cookie_environment).is_some()") || !frontendRecovery.includes("path != \"/offline\"")) fail("frontend recovery predicate drifted");
if (!frontendSsr.includes("is_authenticated || recover_session") || !frontendSsr.includes("HeaderValue::from_static(\"private, no-store\")") || !frontendSsr.includes("HeaderValue::from_static(\"Cookie, Authorization\")")) fail("frontend recovery cache boundary drifted");
if ((frontendSsr.match(/return private_session_redirect\(/g) ?? []).length !== 2 || !frontendSsr.includes("fn private_session_redirect(location: String) -> Response") || !frontendSsr.includes("apply_ssr_cache_policy(&mut response, true, false, \"/auth\")")) fail("frontend credential-dependent redirects are not private");
if (!frontendSsr.includes("let offline_shell = path == \"/offline\"") || !frontendSsr.includes("let access_verification = if offline_shell { AccessVerification::MissingOrRejected")) fail("offline shell still depends on session verification");
const offlineBranch = frontendSsr.slice(frontendSsr.indexOf("if path == \"/offline\""), frontendSsr.indexOf("} else if is_authenticated || recover_session"));
if (!offlineBranch.includes("public, max-age=0, must-revalidate") || !offlineBranch.includes("offline-shell-v1")) fail("offline public exception drifted");

const adminSsr = normalize(stripRustComments(read(resolve(root, "apps/admin/src/ssr.rs"))));
const adminRecoveryStart = adminSsr.indexOf("let recover_session = access_verification.permits_refresh_recovery()");
const adminRecoveryEnd = adminSsr.indexOf("let (verified_access_token, user)", adminRecoveryStart);
const adminRecovery = adminSsr.slice(adminRecoveryStart, adminRecoveryEnd);
if (adminRecoveryStart < 0 || adminRecoveryEnd < 0 || !adminRecovery.includes("auth::refresh_token(&headers, state.cookie_environment).is_some()")) fail("admin recovery predicate drifted");
if (!adminSsr.includes("private_admin_html_response(status, doc)") || !adminSsr.includes("(\"vary\", \"Cookie, Authorization\")")) fail("admin private cache contract drifted");

for (const [name, source] of [["frontend", frontendSsr], ["admin", adminSsr]]) {
  if (!source.includes("let recovery_runtime = recover_session .then(")) fail(`${name}: recovery bootstrap is not conditional`);
  if ((source.match(/<script data-epsx-session-recovery>/g) ?? []).length !== 1) fail(`${name}: recovery marker source cardinality drifted`);
  if (!source.includes("browser_session_recovery_script()")) fail(`${name}: fixed recovery bootstrap is not used`);
}

const browserFile = stripRustComments(read(resolve(root, "shared/rust/bff/src/browser_auth.rs")));
const rawMatch = browserFile.match(/pub fn browser_auth_script\(\) -> &\u0027static str \{\s*r#"\n([\s\S]*?)\n"#\s*\}/);
if (!rawMatch) fail("browser bridge raw string missing");
const browser = rawMatch[1];
if ((browser.match(/epsxRecoverPromise = null/g) ?? []).length !== 1) fail("recovery promise must be initialized once and never reset");
if ((browser.match(/fetch\(\u0027\/api\/v1\/auth\/refresh\u0027/g) ?? []).length !== 1) fail("browser refresh fetch source cardinality drifted");
const recoverStart = browser.indexOf("function epsxRecoverSession()");
const recoverEnd = browser.indexOf("function epsxLogoutSession", recoverStart);
const recover = browser.slice(recoverStart, recoverEnd);
if (recoverStart < 0 || recoverEnd < 0 || !recover.includes("if (epsxRecoverPromise) return epsxRecoverPromise") || !recover.includes("epsxRefreshSession().then(function(session)") || (recover.match(/window\.location\.reload\(\)/g) ?? []).length !== 1) fail("one-shot recovery flow drifted");
for (const forbidden of ["fetch(", "setTimeout", "setInterval", "localStorage", "sessionStorage"]) if (recover.includes(forbidden)) fail(`recovery contains forbidden behavior: ${forbidden}`);
const siweStart = browser.indexOf("function epsxSiweLogin(");
const siweEnd = browser.indexOf("function epsxLogoutSession", siweStart);
const siwe = browser.slice(siweStart, siweEnd);
if (siweStart < 0 || siweEnd < 0 || !siwe.includes("epsxWithSessionMutation(function()") || !siwe.includes("'/api/v1/auth/siwe'") || !siwe.includes("}, false);")) fail("SIWE cookie establishment bypasses the shared session-mutation lock");
if (!browser.includes("siweLogin: epsxSiweLogin")) fail("SIWE bridge does not use the serialized cookie-establishment flow");
for (const forbidden of ["localStorage", "sessionStorage", "access_token", "refresh_token"]) if (browser.includes(forbidden)) fail(`browser bridge contains forbidden material: ${forbidden}`);
const bootstrapMatch = browserFile.match(/pub fn browser_session_recovery_script\(\) -> &\u0027static str \{\s*"([^"]+)"\s*\}/);
if (!bootstrapMatch || bootstrapMatch[1] !== "window.epsxAuth.recover().catch(function() {});") fail("fixed recovery bootstrap drifted");

const browserTests = read(resolve(root, "scripts/migration/test-browser-session-coordination.js"));
if ((browserTests.match(/\btest\("/g) ?? []).length !== 20) fail("exactly twenty browser VM cases are required");
for (const anchor of [
  "automatic recovery is one-shot and reloads once after rotation",
  "automatic recovery preserves the page on explicit non-rotation",
  "contradictory preserved success never reloads or retries",
  "missing or invalid refresh state requires confirmed best-effort clearing",
  "automatic recovery navigates only after confirmed clearing",
  "automatic recovery transport ambiguity without clear confirmation does not navigate",
  "automatic recovery without Web Locks refuses all network I/O",
  "recovery and SIWE cookie establishment use the same exclusive lock"
]) if (!browserTests.includes(anchor)) fail(`browser recovery case missing: ${anchor}`);

for (const file of ["apps/frontend/src/main.rs", "apps/admin/src/main.rs"]) {
  const source = stripRustComments(read(resolve(root, file)));
  if (!source.includes("data-epsx-session-recovery") || !source.includes("outage_html") || !source.includes("opaque-refresh")) fail(`${file}: full-router recovery evidence drifted`);
  const routingTests = source.slice(source.indexOf("mod routing_tests"));
  if (!routingTests.includes("deterministic test outage") || routingTests.includes("JwksVerifier::with_http")) fail(`${file}: recovery journey can use uncontrolled JWKS HTTP`);
}

const prefix = staticOnly === "1" ? "automatic-session-recovery: STATIC-SELF-TEST PASS" : "automatic-session-recovery: PASS";
console.log(`${prefix} 11/11 invariants, ${anchors}/${anchors} anchors, 9/9 residual STOPs`);
' -- "$EVIDENCE_ROOT" "$CONTRACT" "$STATIC_ONLY" || exit 1

if [[ "$STATIC_ONLY" != "1" ]]; then
  export CARGO_NET_OFFLINE=true
  export NO_PROXY="127.0.0.1,localhost,::1"
  unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

  run_cargo_case() {
    label="$1"
    expected="$2"
    shift 2
    output="$(cd "$REPO_ROOT" && "$@" 2>&1)" || {
      echo "$output" >&2
      die "$label failed"
    }
    grep -Fq "test result: ok. $expected passed; 0 failed" <<<"$output" || {
      echo "$output" >&2
      die "$label did not report exactly $expected passing tests"
    }
    echo "automatic-session-recovery: PASS $label ($expected/$expected)"
  }

  run_cargo_case verifier-tests 14 cargo test --offline --locked -p epsx-bff session::tests --no-fail-fast
  run_cargo_case browser-bridge-tests 8 cargo test --offline --locked -p epsx-bff browser_auth::tests --no-fail-fast
  run_cargo_case frontend-cache-test 1 cargo test --offline --locked -p epsx-frontend ssr::tests::recovery_bearing_frontend_html_is_private_and_varies_by_credentials --no-fail-fast
  run_cargo_case frontend-router-test 1 cargo test --offline --locked -p epsx-frontend routing_tests::frontend_emits_one_private_recovery_bootstrap_only_for_refresh_eligible_html --no-fail-fast
  run_cargo_case admin-router-test 1 cargo test --offline --locked -p epsx-admin routing_tests::admin_emits_one_private_recovery_bootstrap_only_with_refresh_cookie --no-fail-fast

  browser_output="$(cd "$REPO_ROOT" && bun scripts/migration/test-browser-session-coordination.js)" || {
    echo "$browser_output" >&2
    die "browser-session-coordination failed"
  }
  echo "$browser_output"
  grep -Fq "browser-session-coordination: PASS 20/20" <<<"$browser_output" || \
    die "browser-session-coordination did not report exactly 20/20 cases"
fi

if [[ "$STATIC_ONLY" == "1" ]]; then
  echo "automatic-session-recovery: STATIC-SELF-TEST ONLY — runtime gate was intentionally skipped"
  exit 0
fi

if [[ "$MODE" == "readiness" ]]; then
  echo "automatic-session-recovery: STOP — real browser, proxy fault, PostgreSQL, cookie-delivery, and production proof remain absent" >&2
  exit 3
fi

echo "automatic-session-recovery: PASS — hermetic evidence only; production readiness is not claimed"
