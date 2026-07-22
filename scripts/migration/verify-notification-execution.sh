#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
contract="$repo_root/docs/migration/contracts/notification-execution.json"
mode=""

die() {
  echo "notification-execution: ERROR: $*" >&2
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

for name in DATABASE_URL NOTIFICATIONS_DATABASE_URL REDIS_URL REDIS_PASSWORD SMTP_HOST SMTP_URL SMTP_USER SMTP_PASSWORD SENDGRID_API_KEY RESEND_API_KEY VAPID_PRIVATE_KEY VAPID_PUBLIC_KEY INTERNAL_SERVICE_TOKEN KUBECONFIG; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier never contacts databases, Redis, SMTP, push providers, Kubernetes, or internal services"
done

for name in HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy NOTIFICATION_SERVICE_URL NOTIFICATION_NETWORK_ACCESS; do
  eval "value=\${$name-}"
  [ -z "$value" ] || die "$name must be unset; this verifier refuses external network configuration"
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

summary=$(bun -e '
import { readFileSync, realpathSync } from "node:fs";
import { isAbsolute, resolve, sep } from "node:path";

const [rootInput, contractPath] = process.argv.slice(1);
const root = realpathSync(rootInput);
const fail = (message) => {
  console.error(`notification-execution: ERROR: ${message}`);
  process.exit(1);
};
const git = (...args) => {
  const result = Bun.spawnSync(["git", ...args], { cwd: root, stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) fail(`git ${args.join(" ")} failed`);
  return result.stdout.toString().trim();
};
const safeRelative = (value, label) => {
  if (typeof value !== "string" || !value || value.includes("\0") || value.includes("\\") || isAbsolute(value)) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
  const segments = value.split("/");
  if (segments.some((segment) => !segment || segment === "." || segment === "..")) {
    fail(`unsafe evidence path for ${label}: ${JSON.stringify(value)}`);
  }
};
const anchored = (content, item, scope) => {
  if (typeof item.anchor !== "string" || item.anchor.length < 4) fail(`${scope} ${item.id}: invalid anchor`);
  if (!content.includes(item.anchor)) fail(`missing ${scope} anchor ${item.id} in ${item.file}`);
};

let contract;
try { contract = JSON.parse(readFileSync(contractPath, "utf8")); }
catch (error) { fail(`invalid contract JSON: ${error.message}`); }

if (contract.schemaVersion !== 1 || contract.contractId !== "A11.0-notification-execution") fail("unexpected schemaVersion or contractId");
if (contract.purpose !== "deterministic-audit-and-readiness-stop") fail("unexpected contract purpose");
if (contract.productionReady !== false || contract.integrityExit !== 0 || contract.readinessExit !== 3) fail("readiness sentinel changed");
if (!contract.safety || Object.entries(contract.safety).filter(([key]) => key !== "readinessMeaning").some(([, value]) => value !== false)) fail("safety flags must remain false");

const source = contract.source;
if (!source || source.ref !== "origin/development" || !/^[0-9a-f]{40}$/.test(source.commit)) fail("invalid pinned source ref/commit");
const resolvedRef = git("rev-parse", `${source.ref}^{commit}`);
if (resolvedRef !== source.commit) fail(`stale source ref/commit: ${source.ref}=${resolvedRef}, contract=${source.commit}`);
if (source.commit !== "373bd231cb7a616c3d4c0ddc1d60e0099a88a5db") fail("source commit is not the reviewed A11 pin");
if (!Array.isArray(source.evidence) || source.evidence.length !== 14) fail("exactly 14 pinned source evidence records are required");

const evidenceIds = new Set();
for (const item of source.evidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  if (!/^[0-9a-f]{40}$/.test(item.blob)) fail(`${item.id}: invalid source blob`);
  const actualBlob = git("rev-parse", `${source.commit}:${item.file}`);
  if (actualBlob !== item.blob) fail(`${item.id}: stale source blob for ${item.file}`);
  const content = git("show", `${source.commit}:${item.file}`);
  anchored(content, item, "source");
}

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length !== 53) fail("exactly 53 target evidence records are required");
const targetContents = new Map();
for (const item of contract.targetEvidence) {
  if (!item || typeof item.id !== "string" || !/^[a-z][a-z0-9-]+$/.test(item.id) || evidenceIds.has(item.id)) fail(`invalid or duplicate evidence id: ${item?.id}`);
  evidenceIds.add(item.id);
  safeRelative(item.file, item.id);
  const candidate = resolve(root, item.file);
  let actual;
  try { actual = realpathSync(candidate); }
  catch { fail(`missing target evidence file ${item.file}`); }
  if (actual !== root && !actual.startsWith(`${root}${sep}`)) fail(`unsafe evidence path for ${item.id}: ${JSON.stringify(item.file)}`);
  const content = readFileSync(actual, "utf8");
  targetContents.set(item.id, content);
  anchored(content, item, "target");
}
const exactNotificationAnchors = {
  "tgt-frontend-get-only-list": ["apps/frontend/src/main.rs", "get(notifications_api)"],
  "tgt-frontend-query-contract": ["apps/frontend/src/api.rs", "const NOTIFICATION_LIST_LIMIT_MAX: u16 = 100;"],
  "tgt-frontend-body-limits": ["apps/frontend/src/api.rs", "const NOTIFICATION_LIST_BODY_MAX: usize = 2 * 1024 * 1024;"],
  "tgt-frontend-owner-cross-check": ["apps/frontend/src/api.rs", ".validate(&user.wallet_address, query.limit)"],
  "tgt-frontend-bearer-only": ["apps/frontend/src/api.rs", "let req = client.get(&url).bearer_auth(&token);"],
  "tgt-frontend-unread-contract": ["apps/frontend/src/api.rs", "struct NotificationUnreadCount {"],
  "tgt-frontend-ssr-list": ["apps/frontend/src/ssr.rs", ".get_with_ctx(\"/api/v1/notification/list\", &request_context)"],
  "tgt-frontend-ssr-ok": ["apps/frontend/src/ssr.rs", "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"ok\".into());"],
  "tgt-frontend-ssr-error": ["apps/frontend/src/ssr.rs", "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"error\".into());"],
  "tgt-user-ui-target-dto": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "struct ServiceNotification {"],
  "tgt-user-ui-no-sample": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "fn malformed_and_upstream_states_are_truthful_and_sample_free()"],
  "tgt-user-ui-neutral-title": ["shared/rust/dioxus_ui/src/pages/notifications.rs", ".unwrap_or_else(|| \"Notification\".to_string());"],
  "tgt-user-ui-auth-only": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "fn authenticated_owner_needs_no_frontend_permission_grant()"],
  "tgt-user-ui-read-only-count": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "let unread_label = format!(\"{unread_count} unread in loaded list\");"],
  "tgt-user-ui-disabled-controls": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "fn lifecycle_delivery_and_unapproved_navigation_controls_are_absent()"],
  "tgt-dormant-nav-unavailable": ["apps/frontend/src/ui.rs", "data-state=\"unavailable\""],
  "tgt-active-header-mount": ["apps/frontend/src/ssr.rs", "epsx_templates::epsx_header()"],
  "tgt-active-header-auth-runtime": ["apps/frontend/src/ssr.rs", "let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);"],
  "tgt-active-header-offline-exclusion": ["apps/frontend/src/ssr.rs", "if !is_authenticated || path == \"/offline\" {"],
  "tgt-active-header-endpoint": ["apps/frontend/src/ssr.rs", "var endpoint = \u0027/api/v1/notifications/unread-count\u0027;"],
  "tgt-active-header-exact-validation": ["apps/frontend/src/ssr.rs", "if (!Number.isSafeInteger(payload.count) || payload.count < 0) return null;"],
  "tgt-active-header-race-guard": ["apps/frontend/src/ssr.rs", "if (generation === requestGeneration && !document.hidden) setUnavailable();"],
  "tgt-active-header-initial-dom": ["shared/rust/templates/src/lib.rs", "data-epsx-notification-unread-badge=\"true\" data-state=\"unavailable\" aria-hidden=\"true\" hidden></span>"],
  "tgt-active-header-accessibility": ["apps/frontend/src/ssr.rs", "target.setAttribute(\u0027aria-label\u0027, \u0027Notifications, \u0027 + String(count) + \u0027 unread\u0027);"],
  "tgt-active-header-text-only": ["apps/frontend/src/ssr.rs", "badge.textContent = count > 99 ? \u002799+\u0027 : String(count);"],
};
const targetById = new Map(contract.targetEvidence.map((item) => [item.id, item]));
for (const [id, [file, anchor]] of Object.entries(exactNotificationAnchors)) {
  const item = targetById.get(id);
  if (!item || item.file !== file || item.anchor !== anchor) fail(`${id}: notification semantic anchor drifted`);
}
for (const id of ["tgt-schema-compatibility", "tgt-additive-service-migration", "tgt-startup-no-seeds"]) {
  if (!targetContents.has(id)) fail(`missing A3.11 target evidence ${id}`);
}
for (const stale of [
  "tgt-runtime-template-ddl", "tgt-runtime-notification-ddl", "tgt-runtime-sample-seed",
  "tgt-user-ui-sample-fallback", "tgt-user-ui-inert-bulk", "tgt-browser-permission-simulated",
  "tgt-user-preferences-inert", "tgt-frontend-any-method", "tgt-frontend-list-bff",
  "tgt-nav-count-first-page", "tgt-active-header-no-badge"
]) {
  if (evidenceIds.has(stale)) fail(`stale pre-A3.11 target evidence remains: ${stale}`);
}
const notificationMain = targetContents.get("tgt-service-routes");
if (/\bCREATE\s+(?:TABLE|INDEX)\b/i.test(notificationMain)) fail("notification runtime DDL reappeared after A3.11");
if (notificationMain.includes("seed_default_templates") || notificationMain.includes("seed_sample_notifications")) fail("notification startup sample/default seed path reappeared after A3.11");
const frontendSsr = targetContents.get("tgt-frontend-ssr-list");
for (const anchor of [
  "params.remove(NOTIFICATIONS_DATA_PARAM);",
  "params.insert(NOTIFICATIONS_DATA_PARAM.into(), value.to_string());",
  "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"ok\".into());",
  "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"error\".into());"
]) if (!frontendSsr.includes(anchor)) fail(`notification SSR outcome contract drifted: ${anchor}`);
const frontendMain = targetContents.get("tgt-frontend-get-only-list");
const notificationRoutesStart = frontendMain.indexOf("\"/api/v1/notifications\",");
const notificationMutationsStart = frontendMain.indexOf(".route(\"/api/v1/notifications/{id}/read\"", notificationRoutesStart);
if (notificationRoutesStart < 0 || notificationMutationsStart < 0) fail("notification BFF read-route boundary markers drifted");
const notificationReadRoutes = frontendMain.slice(notificationRoutesStart, notificationMutationsStart);
for (const anchor of [
  "get(notifications_api)",
  "\"/api/v1/notifications/unread-count\"",
  "get(notification_unread_count)",
  ".head(|| async { axum::http::StatusCode::METHOD_NOT_ALLOWED })"
]) if (!notificationReadRoutes.includes(anchor)) fail(`notification GET-only route contract drifted: ${anchor}`);
if ((notificationReadRoutes.match(/\.head\(\|\| async \{ axum::http::StatusCode::METHOD_NOT_ALLOWED \}\)/g) ?? []).length !== 2) fail("notification list and unread routes must each reject HEAD explicitly");
for (const forbidden of [
  ".route(\"/api/v1/notifications\", any(notifications_api))",
  ".route(\"/api/v1/notifications\", post(notifications_api))",
  "any(notification_unread_count)",
  "post(notification_unread_count)"
]) if (notificationReadRoutes.includes(forbidden)) fail(`notification read route widened beyond GET: ${forbidden}`);

const frontendApi = targetContents.get("tgt-frontend-query-contract");
const queryStart = frontendApi.indexOf("const NOTIFICATION_LIST_LIMIT_MAX: u16 = 100;");
const listStart = frontendApi.indexOf("pub async fn notifications_api(", queryStart);
const unreadStructStart = frontendApi.lastIndexOf("#[derive(Debug, Deserialize, Serialize)]", frontendApi.indexOf("struct NotificationUnreadCount {"));
const unreadFunctionStart = frontendApi.indexOf("pub async fn notification_unread_count(", listStart);
const mutationStart = frontendApi.indexOf("pub async fn notification_read(", unreadFunctionStart);
if ([queryStart, listStart, unreadStructStart, unreadFunctionStart, mutationStart].some((offset) => offset < 0)) fail("notification BFF read boundary markers drifted");
const queryContract = frontendApi.slice(queryStart, unreadStructStart);
const listFunction = frontendApi.slice(listStart, unreadFunctionStart);
const unreadFunction = frontendApi.slice(unreadFunctionStart, mutationStart);
const unreadBoundary = frontendApi.slice(unreadStructStart, mutationStart);
const privateResponseStart = frontendApi.indexOf("fn private_notification_response(", unreadStructStart);
const privateResponseEnd = frontendApi.indexOf("async fn read_notification_body_limited(", privateResponseStart);
if (privateResponseStart < 0 || privateResponseEnd < 0 || privateResponseStart >= privateResponseEnd) fail("notification private response boundary drifted");
const privateResponse = frontendApi.slice(privateResponseStart, privateResponseEnd);
for (const anchor of [
  "header::CACHE_CONTROL",
  "HeaderValue::from_static(\"private, no-store\")"
]) if (!privateResponse.includes(anchor)) fail(`notification private response policy drifted: ${anchor}`);
if (!listFunction.includes("private_notification_response(notifications_api_inner(state, headers, raw_query).await)")) fail("notification list response no longer wraps every outcome in the private cache policy");
if (!unreadFunction.includes("private_notification_response(notification_unread_count_inner(state, headers).await)")) fail("notification unread response no longer wraps every outcome in the private cache policy");
for (const anchor of [
  "const NOTIFICATION_LIST_OFFSET_MAX: u32 = 1_000_000;",
  "const NOTIFICATION_LIST_BODY_MAX: usize = 2 * 1024 * 1024;",
  "const NOTIFICATION_UNREAD_BODY_MAX: usize = 4 * 1024;",
  "if !seen.insert(key.to_string())",
  "if !(1..=NOTIFICATION_LIST_LIMIT_MAX).contains(&value)",
  "if value > NOTIFICATION_LIST_OFFSET_MAX",
  "if !matches!(value.as_ref(), \"pending\" | \"sent\" | \"failed\")",
  "_ => return Err(())",
  "user_id: RequiredNullable<String>",
  ".is_some_and(|user_id| user_id.eq_ignore_ascii_case(owner))"
]) if (!queryContract.includes(anchor)) fail(`notification list query/owner contract drifted: ${anchor}`);
if ((queryContract.match(/#\[serde\(deny_unknown_fields\)\]/g) ?? []).length !== 2) fail("notification list and row wire DTOs must both reject unknown fields");
for (const anchor of [
  "struct NotificationListWire {",
  "items: Vec<NotificationWire>,",
  "total: i64,",
  "struct NotificationWire {",
  "id: String,",
  "user_id: RequiredNullable<String>,",
  "channel: String,",
  "recipient: String,",
  "template_id: RequiredNullable<String>,",
  "subject: RequiredNullable<String>,",
  "body: String,",
  "data: RequiredNullable<serde_json::Value>,",
  "status: String,",
  "error: RequiredNullable<String>,",
  "sent_at: RequiredNullable<DateTime<chrono::Utc>>",
  "created_at: DateTime<chrono::Utc>,",
  "read_at: RequiredNullable<DateTime<chrono::Utc>>",
  "title: RequiredNullable<String>,",
  "notification_type: RequiredNullable<String>,",
  "priority: RequiredNullable<String>,",
  "action_url: RequiredNullable<String>"
]) if (!queryContract.includes(anchor)) fail(`notification exact list wire DTO drifted: ${anchor}`);
for (const anchor of [
  "let (token, user) = match verified_bearer_and_user(&state, &headers).await",
  "query.upstream_suffix()",
  "let req = client.get(&url).bearer_auth(&token);",
  "read_notification_body_limited(response, NOTIFICATION_LIST_BODY_MAX)",
  ".validate(&user.wallet_address, query.limit)"
]) if (!listFunction.includes(anchor)) fail(`notification list forwarding/validation contract drifted: ${anchor}`);
for (const anchor of [
  "async fn read_notification_body_limited(",
  ".content_length()",
  "while let Some(chunk) = response.chunk().await.map_err(|_| ())?",
  "let next_len = body.len().checked_add(chunk.len()).ok_or(())?;",
  "if next_len > limit",
  "let value = serde_json::from_slice::<serde_json::Value>(&body);",
  "let payload = serde_json::from_slice::<NotificationListWire>(&body);",
  "match (value, payload)"
]) if (!unreadBoundary.includes(anchor)) fail(`notification bounded-body parsing contract drifted: ${anchor}`);
for (const forbidden of ["response.json::<serde_json::Value>()", "value.clone()", "payload.clone()"]) {
  if (listFunction.includes(forbidden)) fail(`notification list reintroduced unbounded or cloned parsing: ${forbidden}`);
}
for (const identityForward of ["x-user-id", "x-user-address", "x-wallet-address", ".header("]) {
  if (listFunction.toLowerCase().includes(identityForward) || unreadFunction.toLowerCase().includes(identityForward)) fail(`notification read BFF forwards unreviewed identity metadata: ${identityForward}`);
}
for (const anchor of [
  "#[serde(deny_unknown_fields)]",
  "struct NotificationUnreadCount {",
  "count: i64,",
  "let token = match verified_bearer(&state, &headers).await",
  "\"{}/api/v1/notification/unread-count\"",
  ".bearer_auth(&token)",
  "read_notification_body_limited(response, NOTIFICATION_UNREAD_BODY_MAX)",
  "serde_json::from_slice::<NotificationUnreadCount>(&body)",
  "Ok(payload) if payload.count >= 0 => Json(payload).into_response()"
]) if (!unreadBoundary.includes(anchor)) fail(`notification unread-count contract drifted: ${anchor}`);

const dormantNav = targetContents.get("tgt-dormant-nav-unavailable");
if (!dormantNav.includes("data-state=\"unavailable\"")) fail("dormant notification badge is not explicitly unavailable");
for (const forbidden of ["fetch(" + String.fromCharCode(39) + "/api/v1/notifications", "/api/v1/notifications?limit=1", "items.filter", ">0</span>"]) {
  if (dormantNav.includes(forbidden)) fail(`dormant notification badge reintroduced fabricated count behavior: ${forbidden}`);
}
const activeSsr = targetContents.get("tgt-active-header-mount");
const activeNavStart = activeSsr.indexOf("let nav_html = if path == \"/auth\" {");
const activeNavEnd = activeSsr.indexOf("// === Wave 49+ — re-enable footer ===", activeNavStart);
if (activeNavStart < 0 || activeNavEnd < 0 || !activeSsr.slice(activeNavStart, activeNavEnd).includes("epsx_templates::epsx_header()")) fail("active SSR shell no longer mounts the reviewed shared header");
for (const id of [
  "tgt-active-header-auth-runtime", "tgt-active-header-offline-exclusion", "tgt-active-header-endpoint",
  "tgt-active-header-exact-validation", "tgt-active-header-race-guard", "tgt-active-header-accessibility",
  "tgt-active-header-text-only"
]) {
  if (targetContents.get(id) !== activeSsr) fail(`${id}: active SSR evidence must share the mounted-header source`);
}
const authDerivation = activeSsr.indexOf("let is_authenticated = user.is_some();");
const runtimeInjection = activeSsr.indexOf("let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);");
const bodyInjection = activeSsr.indexOf("{route_runtime}{authenticated_header_runtime}</body>", runtimeInjection);
const badgeRuntimeStart = activeSsr.indexOf("fn notification_badge_runtime(is_authenticated: bool, path: &str) -> &\u0027static str {");
const badgeRuntimeEnd = activeSsr.indexOf("/// Minimal URL-encoder for the `next=` query parameter.", badgeRuntimeStart);
if ([authDerivation, runtimeInjection, bodyInjection, badgeRuntimeStart, badgeRuntimeEnd].some((offset) => offset < 0) || !(authDerivation < runtimeInjection && runtimeInjection < bodyInjection && bodyInjection < badgeRuntimeStart && badgeRuntimeStart < badgeRuntimeEnd)) fail("active notification badge auth/injection boundaries drifted");
const badgeRuntime = activeSsr.slice(badgeRuntimeStart, badgeRuntimeEnd);
const authGate = badgeRuntime.indexOf("if !is_authenticated || path == \"/offline\" {");
const emptyReturn = badgeRuntime.indexOf("return \"\";", authGate);
const browserScript = badgeRuntime.indexOf("data-epsx-notification-badge-runtime", emptyReturn);
if (authGate < 0 || emptyReturn < 0 || browserScript < 0 || !(authGate < emptyReturn && emptyReturn < browserScript)) fail("signed-out/offline badge runtime gate drifted");
for (const anchor of [
  "var endpoint = \u0027/api/v1/notifications/unread-count\u0027;",
  "cache: \u0027no-store\u0027",
  "credentials: \u0027include\u0027",
  "method: \u0027GET\u0027",
  "Object.getPrototypeOf(payload) !== Object.prototype",
  "keys.length !== 1 || keys[0] !== \u0027count\u0027",
  "Object.prototype.hasOwnProperty.call(payload, \u0027count\u0027)",
  "Number.isSafeInteger(payload.count)",
  "payload.count < 0",
  "if (generation !== requestGeneration || document.hidden || !response.ok) return;",
  "if (generation !== requestGeneration || document.hidden) return;",
  "if (generation === requestGeneration && !document.hidden) setUnavailable();",
  "if (count === 0)",
  "badge.textContent = count > 99 ? \u002799+\u0027 : String(count);",
  "target.setAttribute(\u0027aria-label\u0027, \u0027Notifications, \u0027 + String(count) + \u0027 unread\u0027);",
  "badge.hidden = true;",
  "badge.setAttribute(\u0027data-state\u0027, \u0027unavailable\u0027);",
  "fetch(endpoint, {"
]) if (!badgeRuntime.includes(anchor)) fail(`active notification badge runtime contract drifted: ${anchor}`);
if ((badgeRuntime.match(/fetch\(/g) ?? []).length !== 1) fail("active notification badge must have exactly one read fetch");
for (const forbidden of [
  "innerHTML", "insertAdjacentHTML", "document.write", "method: \u0027POST\u0027", "method: \u0027PUT\u0027",
  "method: \u0027PATCH\u0027", "method: \u0027DELETE\u0027", "/api/v1/notifications?", "limit=1", "items.filter"
]) if (badgeRuntime.includes(forbidden)) fail(`active notification badge reintroduced mutation, injection, or fabricated-count behavior: ${forbidden}`);
const sharedTemplates = targetContents.get("tgt-active-header-initial-dom");
const sharedHeaderStart = sharedTemplates.indexOf("pub fn epsx_header() -> String {");
const sharedHeaderEnd = sharedTemplates.indexOf("/// A standard page shell.", sharedHeaderStart);
if (sharedHeaderStart < 0 || sharedHeaderEnd < 0) fail("active shared header boundary markers drifted");
const sharedHeader = sharedTemplates.slice(sharedHeaderStart, sharedHeaderEnd);
for (const anchor of [
  "href=\"/notifications\"",
  "aria-label=\"Notifications\"",
  "data-epsx-notification-badge-target=\"true\"",
  "data-epsx-notification-unread-badge=\"true\" data-state=\"unavailable\" aria-hidden=\"true\" hidden></span>"
]) if (!sharedHeader.includes(anchor)) fail(`active shared-header initial badge contract drifted: ${anchor}`);
if ((sharedHeader.match(/data-epsx-notification-badge-target=\"true\"/g) ?? []).length !== 1 || (sharedHeader.match(/data-epsx-notification-unread-badge=\"true\"/g) ?? []).length !== 1) fail("active shared header must expose exactly one badge target and one badge");
for (const forbidden of [">0</span>", "innerHTML", "fetch(", "/api/v1/notifications/unread-count"]) if (sharedHeader.includes(forbidden)) fail(`active shared header must start inert and unavailable: ${forbidden}`);
const badgeCssStart = sharedTemplates.indexOf(".epsx-notification-badge {{");
const badgeCssEnd = sharedTemplates.indexOf(".epsx-notification-badge[hidden]", badgeCssStart);
if (badgeCssStart < 0 || badgeCssEnd < 0 || badgeCssStart >= badgeCssEnd) fail("active notification badge CSS boundary drifted");
const badgeCss = sharedTemplates.slice(badgeCssStart, badgeCssEnd);
if (!badgeCss.includes("background: #dc2626; color: white;")) fail("active notification badge lost its reviewed AA text contrast color");
if (badgeCss.includes("background: #ef4444; color: white;")) fail("active notification badge restored the sub-AA text contrast color");
const cachePolicyStart = activeSsr.indexOf("fn apply_ssr_cache_policy(response: &mut Response, is_authenticated: bool, path: &str) {");
const cachePolicyEnd = activeSsr.indexOf("/// Fetch page-specific data", cachePolicyStart);
if (cachePolicyStart < 0 || cachePolicyEnd < 0 || cachePolicyStart >= cachePolicyEnd) fail("authenticated SSR cache policy boundary drifted");
const cachePolicy = activeSsr.slice(cachePolicyStart, cachePolicyEnd);
for (const anchor of [
  "if path == \"/offline\" {",
  "HeaderValue::from_static(\"public, max-age=0, must-revalidate\")",
  "} else if is_authenticated {",
  "HeaderValue::from_static(\"private, no-store\")"
]) if (!cachePolicy.includes(anchor)) fail(`authenticated SSR cache policy drifted: ${anchor}`);
if (!activeSsr.includes("apply_ssr_cache_policy(&mut response, is_authenticated, &path);")) fail("SSR response no longer applies the reviewed private/public cache split");
const ownerUi = targetContents.get("tgt-user-ui-target-dto");
const ownerUiRuntime = ownerUi.split("#[cfg(test)]", 1)[0];
for (const anchor of [
  "enum RequiredNullable<T> {",
  "Missing,",
  "Present(Option<T>),",
  "fn require(self) -> Result<Option<T>, ()>",
  "subject: RequiredNullable<String>,",
  "created_at: DateTime<Utc>,",
  "read_at: RequiredNullable<DateTime<Utc>>",
  "title: RequiredNullable<String>,",
  "notification_type: RequiredNullable<String>,",
  "priority: RequiredNullable<String>,",
  "_action_url: RequiredNullable<String>,",
  "impl TryFrom<ServiceNotification> for Notification {",
  "let subject = value.subject.require()?;",
  "let read_at = value.read_at.require()?;",
  "let title = value.title.require()?;",
  "let notification_type = value.notification_type.require()?;",
  "let priority = value.priority.require()?;",
  "let _action_url = value._action_url.require()?;",
  "Some(\"error\") | None => NotificationLoad::UpstreamError",
  "return NotificationLoad::Malformed;",
  ".unwrap_or_else(|| \"Notification\".to_string());",
  "let unread_label = format!(\"{unread_count} unread in loaded list\");",
  "a { class: \"btn btn-sm btn-outline\", href: \"/notifications\", \"Try again\" }"
]) if (!ownerUiRuntime.includes(anchor)) fail(`notification owner read-only contract drifted: ${anchor}`);
for (const forbidden of [
  "notifications:read", "sample_notifications", "Mark all read", "Clear all", "Mark read", "Delete",
  "Enable Browser Notifications", "Test Notification", "Notification Settings", "SwitchInput",
  "BrowserNotificationsPrompt", "NotificationSettingsSection", "notification-action", "use_signal(",
  "onclick:", "notifications-filters"
]) if (ownerUiRuntime.includes(forbidden)) fail(`notification owner UI reintroduced blocked behavior: ${forbidden}`);
const serviceMigration = targetContents.get("tgt-additive-service-migration");
if ((serviceMigration.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\./gi) ?? []).length !== 2) fail("A3.11 guarded public-table evidence drifted");
if (/\b(?:DROP|ALTER|TRUNCATE|DELETE|INSERT|UPDATE|MERGE|COPY|CASCADE)\b/i.test(serviceMigration)) fail("A3.11 evidence contains destructive or data-mutating migration SQL");
if (/^\s*-\s+notification\//m.test(targetContents.get("tgt-kustomize-without-notification"))) fail("notification Kubernetes resource appeared; refresh the A11 deployment audit");

const auth = contract.directAuthPrerequisite;
if (!auth || auth.status !== "partial" || !Array.isArray(auth.proven) || auth.proven.length !== 5 || !Array.isArray(auth.notProven) || auth.notProven.length !== 5) fail("A2.3c direct auth must remain a narrowly scoped partial prerequisite");
if (!Array.isArray(auth.evidenceIds) || auth.evidenceIds.length < 2) fail("direct auth evidence is incomplete");
for (const id of auth.evidenceIds) if (!evidenceIds.has(id)) fail(`direct auth prerequisite: unknown evidence id ${id}`);

const expectedSurfaces = [
  "owner-list-and-count", "owner-lifecycle-mutations", "owner-preferences",
  "realtime-sse-and-offline-replay", "browser-push", "admin-send-broadcast-schedule",
  "template-lifecycle", "email-delivery", "inapp-delivery", "internal-publishers",
  "admin-history-stats-delete", "migration-cutover-operations"
];
if (!Array.isArray(contract.surfaceContracts) || contract.surfaceContracts.length !== expectedSurfaces.length) fail("exactly 12 notification surface contracts are required");
const surfaceIds = new Set();
for (const surface of contract.surfaceContracts) {
  if (!surface || !expectedSurfaces.includes(surface.id) || surfaceIds.has(surface.id)) fail(`invalid or duplicate surface contract: ${surface?.id}`);
  surfaceIds.add(surface.id);
  if (surface.status !== "blocked" || typeof surface.ownerKey !== "string" || !surface.ownerKey) fail(`${surface.id}: surface must remain blocked with an owner key`);
  if (typeof surface.source !== "string" || !surface.source || typeof surface.targetObserved !== "string" || !surface.targetObserved) fail(`${surface.id}: source and target observations are required`);
  if (!Array.isArray(surface.blockerIds) || surface.blockerIds.length === 0) fail(`${surface.id}: blocker references are required`);
}
if (expectedSurfaces.some((id) => !surfaceIds.has(id))) fail("notification surface inventory drifted");
const ownerReadSurface = contract.surfaceContracts.find((surface) => surface.id === "owner-list-and-count");
const exactOwnerReadObservation = "The frontend exposes GET-only list and unread-count BFF routes with explicit HEAD-to-405 overrides; all other non-GET methods are rejected. List permits only bounded status/limit/offset, forwards only a verified bearer, streams at most 2 MiB, parses the same bytes into exact wire and passthrough JSON values, and cross-checks every current-target row against the principal wallet; unread-count streams at most 4 KiB and requires the exact non-negative current-target DTO. SSR and the read-only page preserve explicit outcomes without sample fallback or lifecycle controls. The active shared header starts empty, hidden, and unavailable; only a server-verified authenticated non-offline response receives its read-only browser controller. Authenticated HTML and every list/count BFF outcome are private/no-store, and the fetch bypasses caches. The controller uses the exact unread-count route with credentials, exact-object/non-negative-safe-integer validation, stale-response generation guards, zero/error hiding, AA badge contrast, a 99+ visual cap with the exact count in the accessible label, and text-only DOM writes. Source method/query/envelope/broadcast/expiry/read parity and live browser/runtime proof remain blocked.";
if (ownerReadSurface?.targetObserved !== exactOwnerReadObservation) fail("owner-list-and-count target observation or shared-header residual blockers drifted");

const ruleSections = { ownershipRules: 5, deliveryRules: 8, idempotencyRules: 5, privacyRules: 5 };
for (const [section, expected] of Object.entries(ruleSections)) {
  const rules = contract[section];
  if (!Array.isArray(rules) || rules.length !== expected) fail(`${section} must contain exactly ${expected} rules`);
  const ids = new Set();
  for (const rule of rules) {
    if (!rule || typeof rule.id !== "string" || ids.has(rule.id) || rule.status !== "required-unproven" || typeof rule.rule !== "string" || !rule.rule) fail(`${section}: invalid rule ${rule?.id}`);
    ids.add(rule.id);
  }
}

if (!Array.isArray(contract.migrationRequirements) || contract.migrationRequirements.length !== 7) fail("exactly seven migration requirements are required");
if (!Array.isArray(contract.observabilityRequirements) || contract.observabilityRequirements.length !== 6) fail("exactly six observability requirements are required");
if (!Array.isArray(contract.cutoverRequirements) || contract.cutoverRequirements.length !== 6) fail("exactly six cutover requirements are required");
for (const [name, values] of Object.entries({ migrationRequirements: contract.migrationRequirements, observabilityRequirements: contract.observabilityRequirements, cutoverRequirements: contract.cutoverRequirements })) {
  if (values.some((value) => typeof value !== "string" || !value)) fail(`${name} contains an invalid requirement`);
}

if (!Array.isArray(contract.blockers) || contract.blockers.length !== 22) fail("exactly 22 stop blockers are required");
const blockerIds = new Set();
for (const blocker of contract.blockers) {
  if (!blocker || !/^B[0-9]{2}$/.test(blocker.id) || blockerIds.has(blocker.id)) fail(`invalid or duplicate blocker: ${blocker?.id}`);
  blockerIds.add(blocker.id);
  if (blocker.severity !== "stop" || blocker.status !== "blocked") fail(`${blocker.id}: stop blocker state changed without readiness proof`);
  if (typeof blocker.category !== "string" || !blocker.category || typeof blocker.summary !== "string" || !blocker.summary || typeof blocker.resolution !== "string" || !blocker.resolution) fail(`${blocker.id}: category, summary, and resolution are required`);
  if (!Array.isArray(blocker.evidenceIds) || blocker.evidenceIds.length === 0) fail(`${blocker.id}: evidence references are required`);
  for (const id of blocker.evidenceIds) if (!evidenceIds.has(id)) fail(`${blocker.id}: unknown evidence id ${id}`);
}
for (const surface of contract.surfaceContracts) for (const id of surface.blockerIds) if (!blockerIds.has(id)) fail(`${surface.id}: unknown blocker ${id}`);

if (!Array.isArray(contract.requiredExecutionOrder) || contract.requiredExecutionOrder.length !== 8) fail("exactly eight execution batches are required");
contract.requiredExecutionOrder.forEach((batch, index) => {
  const expected = `N${index + 1}`;
  if (!batch || batch.batch !== expected || typeof batch.name !== "string" || !batch.name || typeof batch.exit !== "string" || !batch.exit) fail(`invalid execution batch ${expected}`);
});

const report = {
  schemaVersion: 1,
  contractId: contract.contractId,
  source: { ref: source.ref, commit: source.commit, evidence: source.evidence.length },
  targetEvidence: contract.targetEvidence.length,
  schemaBoundary: { status: "partial-static", runtimeDdlFindings: 0, startupSeedCalls: 0 },
  directAuthPrerequisite: auth.status,
  surfaces: contract.surfaceContracts.map((item) => ({ id: item.id, status: item.status })),
  rules: {
    ownership: contract.ownershipRules.length,
    delivery: contract.deliveryRules.length,
    idempotency: contract.idempotencyRules.length,
    privacy: contract.privacyRules.length
  },
  requirements: {
    migration: contract.migrationRequirements.length,
    observability: contract.observabilityRequirements.length,
    cutover: contract.cutoverRequirements.length
  },
  batches: contract.requiredExecutionOrder.map((item) => item.batch),
  blockers: contract.blockers.map((item) => ({ id: item.id, category: item.category, status: item.status })),
  productionReady: false,
  readinessExit: 3
};
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
' -- "$repo_root" "$contract") || exit 1

if [ "$mode" = "report" ]; then
  printf '%s\n' "$summary"
  exit 0
fi

if [ "$mode" = "integrity" ]; then
  echo "notification-execution: PASS — 14 source records, 53 target anchors, 12 surfaces, and 22 stop blockers verified"
  echo "notification-execution: LIMIT — A2.3c auth and A3.11 schema boundary remain partial; no database, upgrade, reconciliation, Redis, SMTP, push, network, deployment, or production readiness was proven"
  exit 0
fi

echo "notification-execution: STOP — 22 stop blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "notification-execution: LIMIT — integrity may pass while notification lifecycle and delivery remain non-production" >&2
exit 3
