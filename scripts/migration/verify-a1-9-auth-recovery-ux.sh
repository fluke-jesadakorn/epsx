#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT_RAW="${EPSX_A1_9_REPO_ROOT:-$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)}"
EVIDENCE_ROOT_RAW="${EPSX_A1_9_EVIDENCE_ROOT:-$REPO_ROOT_RAW}"
CONTRACT=""
MODE=""
STATIC_ONLY="${EPSX_A1_9_STATIC_ONLY:-0}"

die() {
  echo "auth-recovery-ux: ERROR: $*" >&2
  exit 1
}

while (( $# > 0 )); do
  case "$1" in
    --mode)
      (( $# >= 2 )) || die "--mode requires integrity, report, or readiness"
      MODE="$2"
      shift 2
      ;;
    --contract)
      (( $# >= 2 )) || die "--contract requires a local JSON file"
      CONTRACT="$2"
      shift 2
      ;;
    --evidence-root)
      (( $# >= 2 )) || die "--evidence-root requires a local directory"
      EVIDENCE_ROOT_RAW="$2"
      shift 2
      ;;
    --static-only)
      STATIC_ONLY=1
      shift
      ;;
    *) die "unsupported argument: $1" ;;
  esac
done

case "$MODE" in
  integrity|report|readiness) ;;
  *) die "--mode must be integrity, report, or readiness" ;;
esac

REPO_ROOT="$(CDPATH= cd -- "$REPO_ROOT_RAW" && pwd -P)" || die "repository root is unavailable"
EVIDENCE_ROOT="$(CDPATH= cd -- "$EVIDENCE_ROOT_RAW" && pwd -P)" || die "evidence root is unavailable"
[[ -n "$CONTRACT" ]] || CONTRACT="$EVIDENCE_ROOT/docs/migration/contracts/a1-9-auth-recovery-ux.json"
case "$CONTRACT" in
  http://*|https://*) die "contract must be a local file" ;;
esac
[[ -f "$CONTRACT" ]] || die "missing contract: $CONTRACT"
[[ "$STATIC_ONLY" == "0" || "$STATIC_ONLY" == "1" ]] || die "static-only must be 0 or 1"
if [[ "$EVIDENCE_ROOT" != "$REPO_ROOT" && "$STATIC_ONLY" != "1" ]]; then
  die "alternate evidence roots are accepted only in static-only self-tests"
fi
if [[ "$EVIDENCE_ROOT" == "$REPO_ROOT" && "$STATIC_ONLY" == "1" ]]; then
  die "static-only mode is reserved for alternate copied self-test fixtures"
fi

command -v bun >/dev/null 2>&1 || die "bun is required"
command -v git >/dev/null 2>&1 || die "git is required"
command -v cargo >/dev/null 2>&1 || die "cargo is required"

for name in \
  DATABASE_URL TEST_DATABASE_URL PRIMARY_DATABASE_URL CORE_DATABASE_URL \
  ANALYTICS_DATABASE_URL IDENTITY_DATABASE_URL PAYMENTS_DATABASE_URL \
  NOTIFICATIONS_DATABASE_URL INDEXER_DATABASE_URL REDIS_URL REDIS_CLUSTER_URL \
  OIDC_ISSUER OIDC_JWKS_URL JWKS_URL AUTH_JWKS_URL AUTH_BASE_URL BACKEND_URL \
  NEXT_PUBLIC_BACKEND_URL API_URL IDENTITY_GRPC_URL IDENTITY_SSE_URL \
  RPC_URL CHAIN_RPC_URL BSC_RPC_URL BSC_MAINNET_RPC_URL BSC_TESTNET_RPC_URL \
  ETH_RPC_URL ETHEREUM_RPC_URL POLYGON_RPC_URL WEB3_PROVIDER_URL \
  PLAYWRIGHT_WS_ENDPOINT BROWSER_WS_ENDPOINT; do
  [[ -z "${!name-}" ]] || die "$name must be unset; this verifier performs no database, credential, browser, or live I/O"
done

for name in LIVE_DATA USE_LIVE_DATA RUN_LIVE_TESTS ENABLE_LIVE_TESTS ALLOW_NETWORK RUN_BROWSER_TESTS; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    1|true|yes|on|live|enabled) die "$name enables a live-data, browser, or network path" ;;
  esac
done

for name in EPSX_ENV APP_ENV ENV ENVIRONMENT NODE_ENV RUST_ENV DEPLOY_ENV DEPLOYMENT_ENV; do
  value="${!name-}"
  normalized="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')"
  case "$normalized" in
    prod|production|prod-*|production-*|*-prod|*-production)
      die "$name identifies a production-looking environment"
      ;;
  esac
done

export CARGO_NET_OFFLINE=true
export NO_PROXY="127.0.0.1,localhost,::1"
unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

summary="$(bun -e '
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";

const [repoInput, evidenceInput, contractInput] = process.argv.slice(1);
const repo = realpathSync(repoInput);
const evidenceRoot = realpathSync(evidenceInput);
const fail = (message) => {
  console.error(`auth-recovery-ux: ERROR: ${message}`);
  process.exit(1);
};
const read = (path) => {
  try { return readFileSync(path, "utf8"); }
  catch (error) { fail(`cannot read ${path}: ${error.message}`); }
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", ...args], {
    cwd: repo,
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" },
  });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const safePath = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) fail(`${label}: unsafe evidence path`);
  if (value.split("/").some((part) => !part || part === "." || part === "..")) fail(`${label}: unsafe evidence path`);
  const candidate = resolve(evidenceRoot, value);
  let stat;
  try { stat = lstatSync(candidate); }
  catch { fail(`${label}: evidence file is missing`); }
  if (!stat.isFile() || stat.isSymbolicLink()) fail(`${label}: evidence must be a regular file`);
  const real = realpathSync(candidate);
  const rel = relative(evidenceRoot, real);
  if (!rel || rel.startsWith("..") || isAbsolute(rel)) fail(`${label}: evidence escapes root`);
  return real;
};
const sha256 = (content) => createHash("sha256").update(content).digest("hex");
const normalize = (value) => value.replace(/\s+/g, " ").trim();
const contains = (content, value, label) => { if (!content.includes(value)) fail(`missing ${label}`); };
const excludes = (content, value, label) => { if (content.includes(value)) fail(`forbidden ${label}`); };
const exactIds = (items, ids, label) => {
  if (!Array.isArray(items) || JSON.stringify(items.map((item) => item.id)) !== JSON.stringify(ids)) fail(`${label} inventory drifted`);
  if (items.some((item) => typeof item.claim !== "string" || !item.claim)) fail(`invalid ${label} claim`);
};
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
      const lifetime = source.slice(index).match(/^\u0027[A-Za-z_][A-Za-z0-9_]*/);
      if (lifetime && source[index + lifetime[0].length] !== "\u0027") {
        output += lifetime[0];
        index += lifetime[0].length;
        continue;
      }
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
try { contract = JSON.parse(read(contractInput)); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }
if (contract.schemaVersion !== 1 || contract.artifact !== "a1-9-auth-recovery-ux" || contract.contractId !== "A1.9-auth-recovery-ux") fail("unexpected contract identity");
if (contract.purpose !== "deterministic-hermetic-frontend-auth-recovery-ux-proof-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel drifted");
const safetyKeys = ["browser", "network", "database", "redis", "service", "deployment", "production"];
if (JSON.stringify(Object.keys(contract.safety)) !== JSON.stringify(safetyKeys) || safetyKeys.some((key) => contract.safety[key] !== false)) fail("safety sentinel drifted");

const sourceCommit = "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db";
const baseCommit = "346d520484e23532ec40a62d1e2fba9d7a10472c";
const predecessorCommit = "c238954cbbf9b8a5db57ef117f0be638c4613766";
if (contract.sourceBaseline?.ref !== "origin/development" || contract.sourceBaseline?.commit !== sourceCommit) fail("source baseline drifted");
if (contract.implementationBase?.ref !== "migration/dioxus-microservices" || contract.implementationBase?.commit !== baseCommit) fail("implementation base drifted");
for (const commit of [sourceCommit, baseCommit, predecessorCommit]) {
  if (git("rev-parse", `${commit}^{commit}`) !== commit) fail(`immutable commit is missing: ${commit}`);
}
const ancestry = Bun.spawnSync(["git", "merge-base", "--is-ancestor", predecessorCommit, baseCommit], { cwd: repo });
if (ancestry.exitCode !== 0) fail("A1.8 predecessor is not an ancestor of the implementation base");

const expectedPredecessor = {
  contractId: "A1.8-automatic-session-recovery",
  commit: predecessorCommit,
  contractFile: "docs/migration/contracts/automatic-session-recovery.json",
  contractBlob: "120fb172544250424498968ef2bb9b215ed33882",
  contractSha256: "a769172a61ef3ade2f90f0230a1b65b55c8abbbfe4411142bc9a01601ac19753",
  verifierFile: "scripts/migration/verify-automatic-session-recovery.sh",
  verifierBlob: "2727644bfcbf3c72baed4fa134c418e02c2c58e1",
  verifierSha256: "d0eeaa9cb39ebee92cc26e0f0de3e16d724d573d17971fc5c66152d3f41a2875",
  evidenceDigest: "4a4f37c819fd082cf6637c5ca165d308de533bdea6423f0f196c950ac9d54aee",
  expectedInvariants: 11,
  expectedAnchors: 46,
  expectedResidualStops: 9,
  replay: "historical-static",
};
if (JSON.stringify(contract.predecessor) !== JSON.stringify(expectedPredecessor)) fail("A1.8 predecessor contract drifted");
if (git("rev-parse", `${predecessorCommit}:${expectedPredecessor.contractFile}`) !== expectedPredecessor.contractBlob) fail("A1.8 contract blob drifted");
if (git("rev-parse", `${predecessorCommit}:${expectedPredecessor.verifierFile}`) !== expectedPredecessor.verifierBlob) fail("A1.8 verifier blob drifted");
if (sha256(git("show", `${predecessorCommit}:${expectedPredecessor.contractFile}`) + "\n") !== expectedPredecessor.contractSha256) fail("A1.8 contract SHA-256 drifted");
if (sha256(git("show", `${predecessorCommit}:${expectedPredecessor.verifierFile}`) + "\n") !== expectedPredecessor.verifierSha256) fail("A1.8 verifier SHA-256 drifted");

const invariantIds = [
  "historical-a1-8-replay",
  "closed-auth-page-state-classification",
  "recovering-state-announced-and-noninteractive",
  "verifier-outage-visible-and-never-recovers",
  "signed-out-state-actionable",
  "unknown-and-stale-state-fails-closed",
  "fixed-token-free-recovery-failure-event",
  "versioned-generic-actionable-failure-ui",
  "immediate-wallet-opening-feedback",
  "a1-8-session-safety-retained",
  "private-offline-and-truthful-ui-boundaries",
  "backend-policy-authority-retained",
];
exactIds(contract.invariants, invariantIds, "invariant");

const expectedImplementation = [
  ["impl-frontend-router-journey", "apps/frontend/src/main.rs", "e8d8305a2963c231e1e6f03e8f4cc17364d9ce150eff69afeb98e47aec7a491f"],
  ["impl-frontend-ssr-state", "apps/frontend/src/ssr.rs", "d6bd2440d57da7fdcf8b7000fedde3caadaae46a79b33a2b6f053d1527dabfa3"],
  ["impl-fixed-recovery-bootstrap", "shared/rust/bff/src/browser_auth.rs", "d0fec242ac4826f2584623b963dd99472f22373ee46f7f41c0667d34079785ba"],
  ["impl-auth-page-state-machine", "shared/rust/dioxus_ui/src/pages/auth_page.rs", "3c6acec87a60ee38e143d619a36d01448ebb0bd1fea5b69c5f5dde747b3247ed"],
  ["impl-fake-dom-harness", "scripts/migration/test-auth-recovery-ux.js", "dd28916051bbccd78b87060b4db859b45357dd08dd4cc246636c3f64ae97208b"],
];
if (!Array.isArray(contract.implementationEvidence) || contract.implementationEvidence.length !== expectedImplementation.length) fail("implementation evidence inventory drifted");
const contentByFile = new Map();
for (let index = 0; index < expectedImplementation.length; index += 1) {
  const [id, file, digest] = expectedImplementation[index];
  if (JSON.stringify(contract.implementationEvidence[index]) !== JSON.stringify({ id, file, sha256: digest })) fail(`${id}: implementation tuple drifted`);
  const content = read(safePath(file, id));
  if (sha256(content) !== digest) fail(`${id}: implementation digest drifted`);
  contentByFile.set(file, content);
}

const expectedTests = [
  ["T01", "cargo", "browser_auth::tests::recovery_bootstrap_emits_only_fixed_token_free_failure_state"],
  ["T02", "cargo", "pages::auth_page::tests::auth_page_signed_out_is_actionable"],
  ["T03", "cargo", "pages::auth_page::tests::auth_page_recovering_is_announced_and_disables_connect"],
  ["T04", "cargo", "pages::auth_page::tests::auth_page_verifier_unavailable_is_fixed_and_disables_connect"],
  ["T05", "cargo", "pages::auth_page::tests::auth_page_unknown_present_state_fails_closed"],
  ["T06", "cargo", "pages::auth_page::tests::auth_page_recovery_failure_event_is_fixed_actionable_and_nondisclosing"],
  ["T07", "cargo", "pages::auth_page::tests::auth_page_connect_enters_opening_wallet_busy_state_immediately"],
  ["T08", "cargo", "pages::auth_page::tests::test_pitch_content"],
  ["T09", "cargo", "ssr::tests::auth_page_session_state_is_closed_and_auth_route_only"],
  ["T10", "cargo", "ssr::tests::verifier_unavailable_auth_html_is_private_and_varies_by_credentials"],
  ["T11", "cargo", "routing_tests::frontend_emits_one_private_recovery_bootstrap_only_for_refresh_eligible_html"],
  ["T12", "bun-fake-dom", "rejected recovery emits one exact closed event"],
  ["T13", "bun-fake-dom", "resolved recovery emits no failure event"],
  ["T14", "bun-fake-dom", "invalid recovery events leave the recovering page closed"],
  ["T15", "bun-fake-dom", "valid failure becomes fixed actionable UI without reflecting payload"],
];
if (!Array.isArray(contract.hermeticTests) || contract.hermeticTests.length !== expectedTests.length) fail("hermetic test inventory drifted");
for (let index = 0; index < expectedTests.length; index += 1) {
  const [id, runner, exactName] = expectedTests[index];
  if (JSON.stringify(contract.hermeticTests[index]) !== JSON.stringify({ id, runner, exactName })) fail(`${id}: hermetic test tuple drifted`);
}

const stopIds = [
  "real-browser-matrix-unproved",
  "post-commit-fault-unproved",
  "postgres-refresh-ordering-unproved",
  "exactly-once-delivery-unproved",
  "bff-unreachable-clear-unproved",
  "access-token-post-revocation-validity",
  "missing-access-authority-preflight-unproved",
  "cross-document-cookie-acceptance-unproved",
  "assistive-technology-validation-unproved",
  "wallet-provider-network-matrix-unproved",
  "responsive-visual-regression-unproved",
  "admin-auth-recovery-ux-parity-unproved",
  "production-actions-unauthorized",
];
exactIds(contract.residualStops, stopIds, "residual STOP");

const frontendMain = normalize(stripRustComments(contentByFile.get("apps/frontend/src/main.rs")));
const frontendSsr = normalize(stripRustComments(contentByFile.get("apps/frontend/src/ssr.rs")));
const browserFile = stripRustComments(contentByFile.get("shared/rust/bff/src/browser_auth.rs"));
const authPageFile = stripRustComments(contentByFile.get("shared/rust/dioxus_ui/src/pages/auth_page.rs"));
const authPage = normalize(authPageFile);
const authPageProduction = authPage.slice(0, authPage.indexOf("#[cfg(test)]"));
const harness = contentByFile.get("scripts/migration/test-auth-recovery-ux.js");

const stateStart = frontendSsr.indexOf("fn auth_page_session_state(");
const stateEnd = frontendSsr.indexOf("fn record_notification_load(", stateStart);
const stateFlow = frontendSsr.slice(stateStart, stateEnd);
if (stateStart < 0 || stateEnd < 0) fail("auth-page state classifier missing");
for (const anchor of [
  "if path != \"/auth\" { return None;",
  "AccessVerification::MissingOrRejected if refresh_cookie_present",
  "AUTH_PAGE_SESSION_STATE_RECOVERING",
  "AccessVerification::VerifierUnavailable => AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE",
  "AccessVerification::MissingOrRejected | AccessVerification::Verified { .. } => { AUTH_PAGE_SESSION_STATE_SIGNED_OUT",
]) contains(stateFlow, anchor, `closed state classifier ${anchor}`);
contains(frontendSsr, "let recover_session = access_verification.permits_refresh_recovery() && refresh_cookie_present && path != \"/offline\";", "A1.8 recovery predicate");
contains(frontendSsr, "let auth_page_verifier_unavailable = auth_page_session_state == Some(AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE);", "verifier-outage cache classification");
contains(frontendSsr, "params.insert( AUTH_PAGE_SESSION_STATE_PARAM.to_string(), session_state.to_string(), );", "auth-page state injection");
contains(frontendSsr, "is_authenticated || recover_session || auth_page_verifier_unavailable", "private verifier-outage cache branch");
contains(frontendSsr, "HeaderValue::from_static(\"private, no-store\")", "private no-store response");
contains(frontendSsr, "HeaderValue::from_static(\"Cookie, Authorization\")", "credential Vary response");
contains(frontendSsr, "HeaderValue::from_static(\"public, max-age=0, must-revalidate\")", "public offline cache response");
contains(frontendSsr, "HeaderValue::from_static(\"offline-shell-v1\")", "offline public-shell marker");

const enumStart = authPage.indexOf("pub enum AuthPageSessionState");
const enumEnd = authPage.indexOf("impl AuthPageSessionState", enumStart);
const authEnum = authPage.slice(enumStart, enumEnd);
for (const variant of ["SignedOut", "Recovering", "VerifierUnavailable"]) contains(authEnum, variant, `auth-page enum ${variant}`);
const contextStart = authPage.indexOf("fn from_context(");
const contextEnd = authPage.indexOf("const fn as_str", contextStart);
const contextFlow = authPage.slice(contextStart, contextEnd);
for (const anchor of [
  "None | Some(AUTH_PAGE_SESSION_STATE_SIGNED_OUT) => Self::SignedOut",
  "Some(AUTH_PAGE_SESSION_STATE_RECOVERING) => Self::Recovering",
  "Some(AUTH_PAGE_SESSION_STATE_VERIFIER_UNAVAILABLE) => Self::VerifierUnavailable",
  "Some(_) => Self::VerifierUnavailable",
]) contains(contextFlow, anchor, `fail-closed UI state ${anchor}`);
contains(authPage, "\"data-auth-session-state\": session_state.as_str()", "auth session-state marker");
contains(authPage, "\"aria-busy\": if session_state == AuthPageSessionState::Recovering", "recovering aria-busy state");
contains(authPage, "disabled: session_state != AuthPageSessionState::SignedOut", "closed connect-button state");
contains(authPage, "role: \"status\"", "accessible recovery status");
contains(authPage, "role: \"alert\"", "accessible recovery alert");
contains(authPage, "tabindex: \"-1\"", "focusable recovery alert");
contains(authPage, "\"Restoring your session...\"", "fixed recovery copy");
contains(authPage, "\"Sign-in temporarily unavailable\"", "fixed verifier-outage title");
contains(authPage, "\"We cannot verify your session right now. Please try again later.\"", "fixed verifier-outage detail");
contains(authPage, "\"Wallet-based sign-in\"", "truthful wallet label");
contains(authPage, "\"Built for teams using modern data workflows\"", "truthful product-fit statement");
for (const forbidden of ["Network Secure & Operational", "2,500+"]) excludes(authPageProduction, forbidden, `unsupported UI claim ${forbidden}`);

const listenerMatch = authPageFile.match(/const WALLET_STATUS_LISTENER_SCRIPT: &str = r#"([\s\S]*?)"#;/);
if (!listenerMatch) fail("auth-page listener raw string missing");
const listener = listenerMatch[1];
const actionableStart = listener.indexOf("function authActionable()");
const actionableEnd = listener.indexOf("if (ctaBtn)", actionableStart);
const actionable = listener.slice(actionableStart, actionableEnd);
contains(actionable, "return state === \u0027signed_out\u0027 || state === \u0027recovery_failed\u0027;", "closed actionable states");
const clickStart = listener.indexOf("ctaBtn.addEventListener(\u0027click\u0027");
const clickEnd = listener.indexOf("function show(d)", clickStart);
const clickFlow = listener.slice(clickStart, clickEnd);
for (const anchor of [
  "if (!authActionable()) return;",
  "statusMsg.textContent = \u0027Opening wallet...\u0027",
  "statusEl.hidden = false",
  "errorEl.hidden = true",
  "ctaBtn.disabled = true",
  "authPage.setAttribute(\u0027aria-busy\u0027, \u0027true\u0027)",
]) contains(clickFlow, anchor, `immediate wallet feedback ${anchor}`);
for (const forbidden of ["fetch(", "eth_requestAccounts", "innerHTML", "localStorage", "sessionStorage"]) excludes(clickFlow, forbidden, `wallet click side effect ${forbidden}`);
contains(listener, "if (!d || !authActionable()) return;", "stale wallet-event guard");
const recoveryStart = listener.indexOf("document.addEventListener(\u0027epsx:auth:recovery\u0027");
const recovery = listener.slice(recoveryStart);
for (const anchor of [
  "if (d.version !== 1 || d.state !== \u0027failed\u0027) return;",
  "authPage.getAttribute(\u0027data-auth-session-state\u0027) !== \u0027recovering\u0027",
  "authPage.setAttribute(\u0027data-auth-session-state\u0027, \u0027recovery_failed\u0027)",
  "errorTitle.textContent = \u0027Session recovery failed\u0027",
  "errorMsg.textContent = \u0027We could not restore your session. Try connecting your wallet again.\u0027",
  "errorEl.focus()",
  "authPage.setAttribute(\u0027aria-busy\u0027, \u0027false\u0027)",
  "ctaBtn.disabled = false",
  "ctaLabel.textContent = \u0027Try Again\u0027",
]) contains(recovery, anchor, `recovery-failure transition ${anchor}`);
for (const forbidden of ["d.message", "e.message", "innerHTML", "fetch(", "localStorage", "sessionStorage"]) excludes(recovery, forbidden, `recovery disclosure/side effect ${forbidden}`);

const bootstrapMatch = browserFile.match(/pub fn browser_session_recovery_script\(\) -> &\u0027static str \{\s*"([^"\n]+)"\s*\}/);
const expectedBootstrap = "window.epsxAuth.recover().catch(function(){try{document.dispatchEvent(new CustomEvent(\u0027epsx:auth:recovery\u0027,{detail:{version:1,state:\u0027failed\u0027}}));}catch(_){}});";
if (!bootstrapMatch || bootstrapMatch[1] !== expectedBootstrap) fail("fixed closed recovery bootstrap drifted");
for (const forbidden of ["token", "wallet", "permission", "plan", "error.message", "function(error)", "fetch("]) excludes(bootstrapMatch[1], forbidden, `bootstrap material ${forbidden}`);
const browserRaw = browserFile.match(/pub fn browser_auth_script\(\) -> &\u0027static str \{\s*r#"\n([\s\S]*?)\n"#\s*\}/);
if (!browserRaw) fail("browser auth bridge raw string missing");
const bridge = browserRaw[1];
for (const anchor of [
  "if (epsxRecoverPromise) return epsxRecoverPromise",
  "epsxRecoverPromise = epsxRefreshSession().then(function(session)",
  "epsxWithSessionMutation(epsxRefreshOnce, true)",
  "window.location.reload();",
  "if (state === \u0027cleared\u0027)",
  "await epsxBestEffortLocalEnd(\u0027refresh_unknown\u0027)",
]) contains(bridge, anchor, `retained A1.8 bridge ${anchor}`);
if ((bridge.match(/fetch\(\u0027\/api\/v1\/auth\/refresh\u0027/g) || []).length !== 1) fail("refresh fetch cardinality drifted");
for (const forbidden of ["localStorage", "sessionStorage", "access_token", "refresh_token"]) excludes(bridge, forbidden, `browser token material ${forbidden}`);

for (const anchor of [
  "data-auth-session-state=\\\"recovering\\\"",
  "data-auth-session-state=\\\"verifier_unavailable\\\"",
  "Sign-in temporarily unavailable",
  "private, no-store",
  "Cookie, Authorization",
  "detail:{version:1,state:\u0027failed\u0027}",
]) contains(frontendMain, anchor, `full-router auth journey ${anchor}`);

const rustTestFiles = new Map([
  ["browser_auth::", browserFile],
  ["pages::auth_page::", authPageFile],
  ["ssr::", contentByFile.get("apps/frontend/src/ssr.rs")],
  ["routing_tests::", contentByFile.get("apps/frontend/src/main.rs")],
]);
for (const [, runner, exactName] of expectedTests.filter((item) => item[1] === "cargo")) {
  const sourceEntry = [...rustTestFiles].find(([prefix]) => exactName.startsWith(prefix));
  if (!sourceEntry) fail(`no source mapped for test ${exactName}`);
  const fnName = exactName.split("::").at(-1);
  if (!new RegExp(`(?:async\\s+)?fn\\s+${fnName}\\s*\\(`).test(sourceEntry[1])) fail(`exact Rust test missing: ${exactName}`);
}
const harnessNames = [...harness.matchAll(/test\("([^"]+)"/g)].map((match) => match[1]);
const expectedHarnessNames = expectedTests.filter((item) => item[1] === "bun-fake-dom").map((item) => item[2]);
if (JSON.stringify(harnessNames) !== JSON.stringify(expectedHarnessNames)) fail("exact 4/4 fake-DOM cases drifted");
for (const anchor of [
  "could not extract the browser auth bridge",
  "vm.runInContext(browserScript, context",
  "window.epsxAuth.recover()",
  "locks: {",
  "request(_name, options, operation)",
  "data-auth-session-state",
  "Sign-in temporarily unavailable",
  "Opening wallet...",
  "auth-recovery-ux: PASS ${passed}/${tests.length}",
]) contains(harness, anchor, `real-bridge fake-DOM evidence ${anchor}`);
for (const forbidden of ["playwright", "puppeteer", "DATABASE_URL", "Bun.fetch", "WebSocket("]) excludes(harness.toLowerCase(), forbidden.toLowerCase(), `fake-DOM live dependency ${forbidden}`);

const report = {
  contractId: contract.contractId,
  productionReady: false,
  integrityExit: 0,
  readinessExit: 3,
  sourceBaseline: sourceCommit,
  implementationBase: baseCommit,
  predecessor: predecessorCommit,
  predecessorReplay: "11/11 invariants, 46/46 anchors, 9/9 residual STOPs",
  invariants: contract.invariants.length,
  implementationEvidence: contract.implementationEvidence.length,
  hermeticTests: contract.hermeticTests.length,
  fakeDomCases: expectedHarnessNames.length,
  residualStops: contract.residualStops.map((item) => item.id),
};
console.log(JSON.stringify(report));
' -- "$REPO_ROOT" "$EVIDENCE_ROOT" "$CONTRACT")" || exit 1

replay_a1_8() {
  local replay_root runner evidence historical_contract evidence_list output
  replay_root="$(mktemp -d "${TMPDIR:-/tmp}/epsx-a1-9-history.XXXXXX")" || die "cannot create historical replay root"
  runner="$replay_root/runner"
  evidence="$replay_root/evidence"
  historical_contract="$replay_root/automatic-session-recovery.json"
  evidence_list="$replay_root/evidence-files.txt"
  mkdir -p "$runner/scripts/migration" "$evidence/docs/migration/contracts" || {
    rm -rf -- "$replay_root"
    die "cannot create historical replay directories"
  }
  git init -q "$runner" || {
    rm -rf -- "$replay_root"
    die "cannot initialize historical verifier root"
  }
  git -C "$REPO_ROOT" show "c238954cbbf9b8a5db57ef117f0be638c4613766:scripts/migration/verify-automatic-session-recovery.sh" >"$runner/scripts/migration/verify-automatic-session-recovery.sh" || {
    rm -rf -- "$replay_root"
    die "cannot materialize historical A1.8 verifier"
  }
  chmod +x "$runner/scripts/migration/verify-automatic-session-recovery.sh"
  git -C "$REPO_ROOT" show "c238954cbbf9b8a5db57ef117f0be638c4613766:docs/migration/contracts/automatic-session-recovery.json" >"$historical_contract" || {
    rm -rf -- "$replay_root"
    die "cannot materialize historical A1.8 contract"
  }
  bun -e '
const contract = await Bun.file(process.argv[1]).json();
const files = ["docs/migration/contracts/automatic-session-recovery.json", ...contract.evidence.map((item) => item.file)];
for (const file of [...new Set(files)]) {
  if (!file || file.startsWith("/") || file.includes("\\") || file.split("/").some((part) => !part || part === "." || part === "..")) process.exit(1);
  console.log(file);
}
' "$historical_contract" >"$evidence_list" || {
    rm -rf -- "$replay_root"
    die "historical A1.8 evidence inventory is unsafe"
  }
  while IFS= read -r file; do
    mkdir -p "$evidence/$(dirname "$file")" || {
      rm -rf -- "$replay_root"
      die "cannot create historical evidence path"
    }
    git -C "$REPO_ROOT" show "c238954cbbf9b8a5db57ef117f0be638c4613766:$file" >"$evidence/$file" || {
      rm -rf -- "$replay_root"
      die "cannot materialize historical evidence: $file"
    }
  done <"$evidence_list"
  output="$(
    EPSX_A1_8_EVIDENCE_ROOT="$evidence" \
    EPSX_A1_8_SELF_TEST_STATIC=1 \
      "$runner/scripts/migration/verify-automatic-session-recovery.sh" --mode evidence 2>&1
  )" || {
    echo "$output" >&2
    rm -rf -- "$replay_root"
    die "historical A1.8 verifier replay failed"
  }
  grep -Fq "automatic-session-recovery: STATIC-SELF-TEST PASS 11/11 invariants, 46/46 anchors, 9/9 residual STOPs" <<<"$output" || {
    echo "$output" >&2
    rm -rf -- "$replay_root"
    die "historical A1.8 replay counts drifted"
  }
  grep -Fq "automatic-session-recovery: STATIC-SELF-TEST ONLY" <<<"$output" || {
    echo "$output" >&2
    rm -rf -- "$replay_root"
    die "historical A1.8 replay did not remain static-only"
  }
  rm -rf -- "$replay_root"
}

replay_a1_8

if [[ "$MODE" == "report" ]]; then
  printf '%s\n' "$summary"
  exit 0
fi

if [[ "$MODE" == "readiness" ]]; then
  echo "auth-recovery-ux: STOP — 13 residual STOPs remain; readiness is intentionally reserved as exit 3" >&2
  exit 3
fi

if [[ "$STATIC_ONLY" == "1" ]]; then
  echo "auth-recovery-ux: STATIC PASS — historical A1.8 11/46/9; 12 invariants; 5 implementation digests; 15 exact hermetic tests including fake-DOM 4/4; 13 residual STOPs"
  exit 0
fi

run_cargo_case() {
  local label="$1"
  local expected="$2"
  shift 2
  local output
  output="$(cd "$REPO_ROOT" && "$@" 2>&1)" || {
    echo "$output" >&2
    die "$label failed"
  }
  grep -Fq "test result: ok. $expected passed; 0 failed" <<<"$output" || {
    echo "$output" >&2
    die "$label did not report exactly $expected passing tests"
  }
  echo "auth-recovery-ux: PASS $label ($expected/$expected)"
}

run_cargo_case bff-fixed-bootstrap 1 \
  cargo test --offline --locked -p epsx-bff \
  browser_auth::tests::recovery_bootstrap_emits_only_fixed_token_free_failure_state \
  -- --exact
run_cargo_case dioxus-auth-state-tests 6 \
  cargo test --offline --locked -p epsx-dioxus-ui \
  pages::auth_page::tests::auth_page_ -- --test-threads=1
run_cargo_case dioxus-truthful-pitch 1 \
  cargo test --offline --locked -p epsx-dioxus-ui \
  pages::auth_page::tests::test_pitch_content -- --exact
run_cargo_case frontend-state-classifier 1 \
  cargo test --offline --locked -p epsx-frontend \
  ssr::tests::auth_page_session_state_is_closed_and_auth_route_only -- --exact
run_cargo_case frontend-verifier-outage-cache 1 \
  cargo test --offline --locked -p epsx-frontend \
  ssr::tests::verifier_unavailable_auth_html_is_private_and_varies_by_credentials -- --exact
run_cargo_case frontend-router-journey 1 \
  cargo test --offline --locked -p epsx-frontend \
  routing_tests::frontend_emits_one_private_recovery_bootstrap_only_for_refresh_eligible_html \
  -- --exact

harness_output="$(cd "$REPO_ROOT" && bun scripts/migration/test-auth-recovery-ux.js 2>&1)" || {
  echo "$harness_output" >&2
  die "fake-DOM auth recovery harness failed"
}
echo "$harness_output"
grep -Fq "auth-recovery-ux: PASS 4/4" <<<"$harness_output" || die "fake-DOM harness did not report exactly 4/4"

echo "auth-recovery-ux: PASS — historical A1.8 11/46/9; 12 invariants; 5 implementation digests; 15 exact hermetic tests including fake-DOM 4/4; 13 residual STOPs"
