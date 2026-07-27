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

for name in DATABASE_URL NOTIFICATIONS_DATABASE_URL REDIS_URL REDIS_PASSWORD SMTP_HOST SMTP_URL SMTP_USER SMTP_PASSWORD NOTIFICATION_SMTP_HOST NOTIFICATION_SMTP_USER NOTIFICATION_SMTP_PASSWORD SENDGRID_API_KEY RESEND_API_KEY VAPID_PRIVATE_KEY VAPID_PUBLIC_KEY NOTIFICATION_VAPID_PUBLIC_KEY INTERNAL_SERVICE_TOKEN NOTIFICATION_ADAPTER NOTIFICATION_PROVIDER_SIGNING_SECRET NOTIFICATION_PROVIDER_SIGNING_SECRET_PREVIOUS NOTIFICATION_PROVIDER_SIGNING_SECRETS KUBECONFIG; do
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
import { existsSync, readFileSync, realpathSync } from "node:fs";
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
const pinnedSourceFilterFiles = [
  git("show", `${source.commit}:shared/api/notifications.ts`),
  git("show", `${source.commit}:shared/components/notifications/types.ts`),
  git("show", `${source.commit}:shared/components/notifications/schemas.ts`),
];
for (const value of [
  "system", "security", "permission", "wallet_management", "wallet", "payment",
  "general", "announcement", "advertisement", "chat", "low", "normal", "high",
  "critical", "urgent", "start_date", "end_date", "page", "limit", "status",
]) {
  if (!pinnedSourceFilterFiles.some((content) => content.includes(value))) {
    fail(`pinned source notification filter vocabulary drifted: ${value}`);
  }
}

if (!Array.isArray(contract.targetEvidence) || contract.targetEvidence.length !== 88) fail("exactly 88 target evidence records are required");
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
  "tgt-realtime-redis-wakeup": ["services/notification/src/main.rs", "notify.notify_waiters();"],
  "tgt-broadcast-row-materialization": ["services/notification/src/main.rs", "const BROADCAST_NOTIFICATION_INSERT_SQL: &str ="],
  "tgt-internal-subject-boundary": ["services/notification/src/lib.rs", "|| !valid_internal_subject(&principal.subject)"],
  "tgt-expiry-filter": ["services/notification/src/main.rs", "(x.expires_at IS NULL OR x.expires_at > NOW())"],
  "tgt-expiry-sweeper": ["services/notification/src/delivery.rs", "const EXPIRE_DUE_SQL: &str ="],
  "tgt-expiry-notification-sweeper": ["services/notification/src/delivery.rs", "const EXPIRE_NOTIFICATIONS_SQL: &str ="],
  "tgt-template-link-url-policy": ["services/notification/src/main.rs", "&& template_link_urls_are_safe(body)"],
  "tgt-template-tag-policy": ["services/notification/src/main.rs", "&& template_tags_are_allowlisted(body)"],
  "tgt-frontend-get-only-list": ["apps/frontend/src/main.rs", "get(notifications_api)"],
  "tgt-frontend-query-contract": ["apps/frontend/src/api.rs", "const NOTIFICATION_LIST_LIMIT_MAX: u16 = 100;"],
  "tgt-frontend-body-limits": ["apps/frontend/src/api.rs", "const NOTIFICATION_LIST_BODY_MAX: usize = 2 * 1024 * 1024;"],
  "tgt-frontend-owner-cross-check": ["apps/frontend/src/api.rs", "payload.validate(owner, query)"],
  "tgt-frontend-bearer-only": ["apps/frontend/src/api.rs", ".bearer_auth(bearer)"],
  "tgt-frontend-unread-contract": ["apps/frontend/src/api.rs", "struct NotificationUnreadCount {"],
  "tgt-frontend-ssr-list": ["apps/frontend/src/ssr.rs", "crate::api::load_owner_notifications("],
  "tgt-frontend-ssr-ok": ["apps/frontend/src/ssr.rs", "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"ok\".into());"],
  "tgt-frontend-ssr-error": ["apps/frontend/src/ssr.rs", "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"error\".into());"],
  "tgt-user-ui-target-dto": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "struct ServiceNotification {"],
  "tgt-user-ui-no-sample": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "fn malformed_and_upstream_states_are_truthful_and_sample_free()"],
  "tgt-user-ui-neutral-title": ["shared/rust/dioxus_ui/src/pages/notifications.rs", ".unwrap_or_else(|| \"Notification\".to_string());"],
  "tgt-user-ui-auth-only": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "fn authenticated_owner_needs_no_frontend_permission_grant()"],
  "tgt-user-ui-read-only-count": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "let unread_label = format!(\"{unread_count} unread on this page\");"],
  "tgt-user-ui-disabled-controls": ["shared/rust/dioxus_ui/src/pages/notifications.rs", "fn lifecycle_delivery_and_unapproved_navigation_controls_remain_absent()"],
  "tgt-dormant-nav-unavailable": ["apps/frontend/src/ui.rs", "data-state=\"unavailable\""],
  "tgt-active-header-mount": ["apps/frontend/src/ssr.rs", "epsx_templates::epsx_header_for_session_and_wallet_with_network("],
  "tgt-active-header-auth-runtime": ["apps/frontend/src/ssr.rs", "let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);"],
  "tgt-active-header-offline-exclusion": ["apps/frontend/src/ssr.rs", "if !is_authenticated || path == \"/offline\" {"],
  "tgt-active-header-endpoint": ["apps/frontend/src/ssr.rs", "var endpoint = \u0027/api/v1/notifications/unread-count\u0027;"],
  "tgt-active-header-exact-validation": ["apps/frontend/src/ssr.rs", "if (!Number.isSafeInteger(payload.count) || payload.count < 0) return null;"],
  "tgt-active-header-race-guard": ["apps/frontend/src/ssr.rs", "if (generation === requestGeneration && !document.hidden) setUnavailable();"],
  "tgt-active-header-initial-dom": ["shared/rust/templates/src/lib.rs", "data-epsx-notification-unread-badge=\"true\" data-state=\"unavailable\" aria-hidden=\"true\" hidden></span>"],
  "tgt-active-header-accessibility": ["apps/frontend/src/ssr.rs", "target.setAttribute(\u0027aria-label\u0027, \u0027Notifications, \u0027 + String(count) + \u0027 unread\u0027);"],
  "tgt-active-header-text-only": ["apps/frontend/src/ssr.rs", "badge.textContent = count > 99 ? \u002799+\u0027 : String(count);"],
  "tgt-admin-global-list-handler": ["services/notification/src/main.rs", "async fn list_admin_notifications("],
  "tgt-admin-global-list-service-policy": ["services/notification/src/lib.rs", "(&Method::GET, [\"admin\", \"list\" | \"metrics\"]) => AccessPolicy::NotificationsAdmin"],
  "tgt-admin-global-list-gateway-policy": ["services/gateway/src/policy.rs", "(&Method::GET, [\"api\", \"v1\", \"notification\", \"admin\", \"list\" | \"metrics\"])"],
  "tgt-admin-global-list-adapter": ["apps/admin/src/notification_admin_adapter.rs", "pub(crate) async fn load_admin_notifications("],
  "tgt-admin-global-list-ssr": ["apps/admin/src/ssr.rs", "if route_path == \"/notifications/manage\" {"],
  "tgt-admin-global-list-ui": ["shared/rust/dioxus_ui/src/pages/admin_pages/notifications.rs", "pub fn decode_admin_notification_projection("],
};
const targetById = new Map(contract.targetEvidence.map((item) => [item.id, item]));
for (const [id, [file, anchor]] of Object.entries(exactNotificationAnchors)) {
  const item = targetById.get(id);
  if (!item || item.file !== file || item.anchor !== anchor) fail(`${id}: notification semantic anchor drifted`);
}
const ownerFilterContracts = [
  [targetContents.get("tgt-frontend-ssr-list"), "const NOTIFICATIONS_STATUS_PARAM: &str ="],
  [targetContents.get("tgt-frontend-ssr-list"), "matches!(value.as_ref(), \"all\" | \"read\" | \"unread\")"],
  [targetContents.get("tgt-frontend-ssr-list"), "const NOTIFICATIONS_TYPE_VALUES: &[&str]"],
  [targetContents.get("tgt-frontend-ssr-list"), "\"wallet_management\""],
  [targetContents.get("tgt-frontend-ssr-list"), "const NOTIFICATIONS_PRIORITY_VALUES: &[&str]"],
  [targetContents.get("tgt-frontend-ssr-list"), "\"start_date\" | \"end_date\""],
  [targetContents.get("tgt-frontend-query-contract"), "pub(crate) fn for_ssr_page_and_status("],
  [targetContents.get("tgt-frontend-query-contract"), "pub(crate) fn for_ssr_page_and_filters("],
  [targetContents.get("tgt-frontend-query-contract"), "pub(crate) fn for_ssr_page_and_filters_and_dates("],
  [targetContents.get("tgt-user-ui-target-dto"), "enum NotificationStatusFilter"],
  [targetContents.get("tgt-user-ui-target-dto"), "enum NotificationTypeFilter"],
  [targetContents.get("tgt-user-ui-target-dto"), "enum NotificationPriorityFilter"],
  [targetContents.get("tgt-user-ui-target-dto"), "data-notification-status-filters"],
  [targetContents.get("tgt-user-ui-target-dto"), "data-notification-type-filters"],
  [targetContents.get("tgt-user-ui-target-dto"), "data-notification-priority-filters"],
];
for (const [content, anchor] of ownerFilterContracts) {
  if (typeof content !== "string" || !content.includes(anchor)) fail(`owner filter contract drifted: ${anchor}`);
}
for (const id of ["tgt-schema-compatibility", "tgt-additive-service-migration", "tgt-startup-no-seeds"]) {
  if (!targetContents.has(id)) fail(`missing A3.11 target evidence ${id}`);
}
for (const stale of [
  "tgt-runtime-template-ddl", "tgt-runtime-notification-ddl", "tgt-runtime-sample-seed",
  "tgt-user-ui-sample-fallback", "tgt-user-ui-inert-bulk", "tgt-browser-permission-simulated",
  "tgt-user-preferences-inert", "tgt-frontend-any-method", "tgt-frontend-list-bff",
  "tgt-nav-count-first-page", "tgt-active-header-no-badge", "tgt-admin-ui-unavailable"
]) {
  if (evidenceIds.has(stale)) fail(`stale pre-A3.11 target evidence remains: ${stale}`);
}
const notificationMain = targetContents.get("tgt-service-routes");
if (/\bCREATE\s+(?:TABLE|INDEX)\b/i.test(notificationMain)) fail("notification runtime DDL reappeared after A3.11");
if (notificationMain.includes("seed_default_templates") || notificationMain.includes("seed_sample_notifications")) fail("notification startup sample/default seed path reappeared after A3.11");
for (const anchor of [
  "\"/api/v1/notification/admin/metrics\"",
  "async fn admin_metrics(",
  "queue_age_seconds",
  "active_streams",
  "channel_outcomes",
  "provider_events",
  "delivery_attempts",
  "replay_cursor_age_seconds",
  "stream_connections_total",
  "stream_reconnects_total",
  "stream_replayed_events_total",
  "stream_lag_seconds",
  "stream_query_failures_total"
]) if (!notificationMain.includes(anchor)) fail(`notification metrics route drifted: ${anchor}`);
for (const anchor of [
  "\"/api/v1/notification/provider-events\"",
  "async fn record_provider_event(",
  "fn validate_provider_event_request(",
  "fn verify_provider_signature(",
  "NOTIFICATION_PROVIDER_SIGNING_SECRET",
  "NOTIFICATION_PROVIDER_SIGNING_SECRET_PREVIOUS",
  "NOTIFICATION_PROVIDER_SIGNING_SECRETS",
  "fn provider_signing_secrets_from_env(",
  "fn provider_signing_secrets_from_values<I, S>(",
  ".provider_signing_secrets",
  ".any(|secret| verify_provider_signature",
  "ON CONFLICT (provider, provider_event_id) DO NOTHING"
]) if (!notificationMain.includes(anchor)) fail(`provider event reconciliation route drifted: ${anchor}`);
const emailDelivery = targetContents.get("tgt-email-blocking-send");
for (const anchor of ["SmtpTransport::relay(host)", "SmtpTransport::starttls_relay(host)", "const SMTP_TRANSPORT_TIMEOUT_SECONDS: u64 = 30;", ".timeout(Some(std::time::Duration::from_secs(", "tokio::task::spawn_blocking(move || smtp.send(&msg))"]) {
  if (!emailDelivery.includes(anchor)) fail(`SMTP transport safety contract drifted: ${anchor}`);
}
const templateDelivery = targetContents.get("tgt-template-nonstrict");
if (!templateDelivery.includes("let t = template.ok_or(StatusCode::NOT_FOUND)?;")) fail("template send path restored a raw-body fallback for missing template IDs");
for (const anchor of [
  "fn valid_template_variables(",
  "fn validate_template_data(",
  "fn template_uses_only_escaped_output(",
  "fn template_markup_is_safe(",
  "fn template_contains_event_handler(",
  "\"<meta\"",
  "\"srcdoc=\"",
  "async fn preview_template(",
  "async fn rollback_template(",
  "async fn list_template_audit(",
  "/api/v1/notification/templates/{id}/rollback",
  "/api/v1/notification/templates/{id}/audit",
  "validate_template_data(&t.variables, &data_map)?;",
  "notification_template_audit",
  "INSERT INTO public.notification_template_audit"
]) {
  if (!templateDelivery.includes(anchor)) fail(`typed template contract drifted: ${anchor}`);
}
const privacyLogSources = [
  ["apps/backend/src/web/notifications/sse_handlers.rs", readFileSync(resolve(root, "apps/backend/src/web/notifications/sse_handlers.rs"), "utf8")],
  ["apps/backend/src/web/notifications/offline_queue.rs", readFileSync(resolve(root, "apps/backend/src/web/notifications/offline_queue.rs"), "utf8")],
  ["apps/backend/src/web/admin/notification_handlers/notification_admin.rs", readFileSync(resolve(root, "apps/backend/src/web/admin/notification_handlers/notification_admin.rs"), "utf8")],
  ["services/notification/src/main.rs", readFileSync(resolve(root, "services/notification/src/main.rs"), "utf8")],
  ["services/notification/src/delivery.rs", readFileSync(resolve(root, "services/notification/src/delivery.rs"), "utf8")]
];
for (const [file, content] of privacyLogSources) {
  for (const forbidden of ["wallet={},", "wallet: {}", "for wallet: {}", "title=", "\"recipient_wallet_address\": request.recipient_wallet_address", "\"message\": request.message"]) {
    if (content.includes(forbidden)) fail(`notification privacy telemetry drifted in ${file}: ${forbidden}`);
  }
  const sensitiveFields = ["wallet", "recipient", "email", "subject", "body", "message", "payload", "token", "title"];
  const logLines = content.split("\n").filter((line) => line.includes("tracing::"));
  for (const line of logLines) {
    for (const field of sensitiveFields) {
      if (new RegExp(`\\b${field}\\s*=|\\{${field}\\}`).test(line)) {
        fail(`notification privacy telemetry interpolates ${field} in ${file}`);
      }
    }
  }
}
const adminNotificationHandler = privacyLogSources[2][1];
for (const anchor of ["fn valid_notification_text(", "fn valid_notification_url(", "\"title_chars\"", "\"has_image_url\""]) {
  if (!adminNotificationHandler.includes(anchor)) fail(`admin notification privacy boundary drifted: ${anchor}`);
}
const notificationPoolFactory = readFileSync(resolve(root, "apps/backend/src/infrastructure/database/diesel_connection_manager.rs"), "utf8");
if (notificationPoolFactory.includes("NOTIFICATIONS_DATABASE_URL not set, using main database pool")) {
  fail("notification pool factory restored a primary-database fallback");
}
for (const [file, content] of [
  ["apps/backend/src/web/admin/notification_handlers/notification_user.rs", readFileSync(resolve(root, "apps/backend/src/web/admin/notification_handlers/notification_user.rs"), "utf8")],
  ["apps/backend/src/web/admin/notification_handlers/notification_admin.rs", adminNotificationHandler]
]) {
  for (const forbidden of ["app_state.db_pool.clone()", "&**app_state.db_pool", "falling back to main pool"]) {
    if (content.includes(forbidden)) fail(`notification handler ${file} restored a primary-pool fallback: ${forbidden}`);
  }
  if (!content.includes("require_notifications_pool().await")) fail(`notification handler ${file} lost strict notification-pool acquisition`);
}
const sendHandler = targetContents.get("tgt-durable-channel-job");
const sendStart = sendHandler.indexOf("async fn send_notification(");
const sendEnd = sendHandler.indexOf("async fn send_email(", sendStart);
if (sendStart < 0 || sendEnd < 0 || !sendHandler.slice(sendStart, sendEnd).includes("StatusCode::ACCEPTED")) fail("durable notification enqueue no longer returns 202 Accepted");
for (const anchor of [
  "Extension(principal): Extension<VerifiedPrincipal>",
  "require_admin_notifications(&principal)?",
  "fn valid_wallet_address(",
  "fn valid_email_recipient(",
  "fn valid_push_recipient(",
  "if t.channel != req.channel",
  "request.recipient == \"all\""
]) if (!sendHandler.includes(anchor)) fail(`admin recipient binding contract drifted: ${anchor}`);
const adminMain = targetContents.get("tgt-admin-bff-send");
const adminSendStart = adminMain.indexOf("async fn send_notification(");
const adminSendEnd = adminMain.indexOf("// ===== Analytics =====", adminSendStart);
if (adminSendStart < 0 || adminSendEnd < 0 || !adminMain.slice(adminSendStart, adminSendEnd).includes("post_with_ctx_status")) fail("admin notification send adapter no longer preserves the durable enqueue status");
for (const anchor of [
  "get(fallback_handler).post(submit_notification_form)",
  "async fn submit_notification_form(",
  "same_origin_admin_notification_form(&parts.headers)",
  "parse_admin_notification_form(&body)",
  "ADMIN_NOTIFICATION_CREATE_COOKIE",
  "send_admin_notification(",
  "post_with_ctx_status(\"/api/v1/notification/send\""
]) if (!adminMain.includes(anchor)) fail(`admin notification compose adapter drifted: ${anchor}`);
for (const anchor of [
  "BackendTemplateList",
  "valid_template_list_payload(&payload)",
  "template list response too large",
  "valid_template_list_variables"
]) if (!adminMain.includes(anchor)) fail(`admin template list projection drifted: ${anchor}`);
const frontendSsr = targetContents.get("tgt-frontend-ssr-list");
for (const anchor of [
  "params.remove(NOTIFICATIONS_DATA_PARAM);",
  "params.insert(NOTIFICATIONS_DATA_PARAM.into(), value.to_string());",
  "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"ok\".into());",
  "params.insert(NOTIFICATIONS_STATE_PARAM.into(), \"error\".into());"
]) if (!frontendSsr.includes(anchor)) fail(`notification SSR outcome contract drifted: ${anchor}`);
const frontendMain = targetContents.get("tgt-frontend-get-only-list");
const notificationRoutesStart = frontendMain.indexOf("\"/api/v1/notifications\",");
const notificationMutationsStart = frontendMain.indexOf("\"/api/v1/notifications/{id}/read\"", notificationRoutesStart);
if (notificationRoutesStart < 0 || notificationMutationsStart < 0) fail("notification BFF read-route boundary markers drifted");
for (const anchor of [
  "\"/api/v1/notifications/{id}/acknowledge\"",
  "put(notification_acknowledge)"
]) if (!frontendMain.includes(anchor)) fail(`notification acknowledgement BFF route drifted: ${anchor}`);
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
const unreadStruct = frontendApi.indexOf("struct NotificationUnreadCount {", queryStart);
const unreadStructStart = frontendApi.lastIndexOf("#[derive(", unreadStruct);
const unreadFunctionStart = frontendApi.indexOf("pub async fn notification_unread_count(", listStart);
const mutationStart = frontendApi.indexOf("pub async fn notification_read(", unreadFunctionStart);
if ([queryStart, listStart, unreadStruct, unreadStructStart, unreadFunctionStart, mutationStart].some((offset) => offset < 0)) fail("notification BFF read boundary markers drifted");
const queryContract = frontendApi.slice(queryStart, unreadStructStart);
const listFunction = frontendApi.slice(listStart, unreadFunctionStart);
const unreadFunction = frontendApi.slice(unreadFunctionStart, mutationStart);
const unreadBoundary = frontendApi.slice(listStart, mutationStart);
const privateResponseStart = frontendApi.indexOf("fn private_notification_response(", queryStart);
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
  "\"pending\" | \"sent\" | \"failed\" | \"suppressed\" | \"read\" | \"unread\" | \"all\"",
  "\"type\" | \"notification_type\"",
  "DateTime::parse_from_rfc3339(value.as_ref())",
  "DateTime::parse_from_rfc3339(start).ok() > DateTime::parse_from_rfc3339(end).ok()",
  "_ => return Err(())",
  "user_id: RequiredNullable<String>",
  "bounded_notification_text(user_id, NOTIFICATION_ID_MAX, false)",
  "user_id.eq_ignore_ascii_case(owner)",
  "NOTIFICATION_RECIPIENT_MAX",
  "NOTIFICATION_BODY_MAX",
  "NOTIFICATION_DATA_MAX",
  "NOTIFICATION_ACTION_URL_MAX"
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
  ".bearer_auth(bearer)",
  "read_notification_body_limited(response, NOTIFICATION_LIST_BODY_MAX)",
  "payload.validate(owner, query)"
]) if (!(queryContract.includes(anchor) || listFunction.includes(anchor))) fail(`notification list forwarding/validation contract drifted: ${anchor}`);
for (const anchor of [
  "pub async fn notification_acknowledge(",
  "{}/api/v1/notification/{}/acknowledge",
  ".put(&url)",
  "pub async fn notification_unread(",
  "{}/api/v1/notification/{}/unread",
  ".header(\"x-request-id\"",
  "async fn notification_engagement_event(",
  "pub async fn notification_click(",
  "pub async fn notification_dismiss(",
  "event:",
  "{}/api/v1/notification/{}/{}"
]) if (!frontendApi.includes(anchor)) fail(`notification acknowledgement BFF forwarding contract drifted: ${anchor}`);
const frontendMainSource = readFileSync(resolve(root, "apps/frontend/src/main.rs"), "utf8");
for (const anchor of [
  "post(notification_read).put(notification_read)",
  "post(notification_unread).put(notification_unread)",
  "delete(notification_delete)",
  "post(notification_mark_all).put(notification_mark_all)",
  "post(notification_clear_all).delete(notification_clear_all)",
  ".route(\"/api/v1/notifications/{id}/click\", post(notification_click))",
  "/api/v1/notifications/{id}/dismiss",
  "post(notification_dismiss)"
]) if (!frontendMainSource.includes(anchor)) fail(`notification engagement BFF route drifted: ${anchor}`);

const notificationService = targetContents.get("tgt-service-routes");
for (const anchor of [
  "fn parse_owner_notification_query(",
  "\"pending\" | \"sent\" | \"failed\" | \"suppressed\" | \"read\" | \"unread\" | \"all\"",
  "const OWNER_NOTIFICATION_FILTER_SQL: &str =",
  "(0..=1_000_000).contains(value)",
  "async fn acknowledge_notification(",
  "async fn mark_clicked(",
  "async fn mark_dismissed(",
  "notification_engagement",
  "acknowledged_at",
  "clicked_at = COALESCE",
  "dismissed_at = COALESCE",
  "ON CONFLICT (notification_id, owner_id)"
]) if (!notificationService.includes(anchor)) fail(`notification acknowledgement service contract drifted: ${anchor}`);
const notificationAdapter = readFileSync(resolve(root, "apps/backend/src/infrastructure/adapters/notification/mod.rs"), "utf8");
for (const anchor of [
  "fn notification_adapter_is_remote(",
  "fn notification_adapter_value_is_supported(",
  "value.eq_ignore_ascii_case(\"remote\")",
  "fn notification_adapter_required_for_values",
  "production"
]) if (!notificationAdapter.includes(anchor)) fail(`notification adapter startup contract drifted: ${anchor}`);
const httpNotificationAdapter = readFileSync(resolve(root, "apps/backend/src/infrastructure/adapters/notification/http_adapter.rs"), "utf8");
for (const anchor of [
  "fn validate_event_id(event_id: &str)",
  "async fn send_request_with_event_id(",
  "async fn broadcast_request_with_event_id(",
  "idempotency_key: event_id",
  ".header(\"x-request-id\", request.event_id)",
  "const PUBLISH_RESPONSE_MAX_BYTES: usize = 8 * 1024;",
  ".content_length()",
  "response.bytes().await",
  "if bytes.len() > PUBLISH_RESPONSE_MAX_BYTES",
  "async fn send_with_event_id(",
  "async fn broadcast_with_event_id("
]) if (!httpNotificationAdapter.includes(anchor)) fail(`stable notification event identity contract drifted: ${anchor}`);
const stablePublisherSources = [
  ["apps/backend/src/web/payments/credit_handlers.rs", "payment.credit.grant:"],
  ["apps/backend/src/web/payments/submit_tx_handler.rs", "payment.confirmed:"],
  ["apps/backend/src/web/admin/permissions/assignments/create.rs", "permission.assignment.created:"],
  ["apps/backend/src/web/admin/permissions/assignments/remove.rs", "permission.assignment.removed:"],
  ["apps/backend/src/web/user/chat_handlers.rs", "chat.message:"],
  ["apps/backend/src/web/admin/chat_handlers.rs", "chat.message:"],
  ["apps/backend/src/infrastructure/services/plan_expiration_service.rs", "subscription.expiry:"]
];
for (const [file, identityPrefix] of stablePublisherSources) {
  const content = readFileSync(resolve(root, file), "utf8");
  if (!content.includes("send_with_event_id_retry(")) fail(`producer lacks bounded stable notification event retry: ${file}`);
  if (!content.includes(identityPrefix)) fail(`producer identity prefix drifted: ${file}`);
}
if (!readFileSync(resolve(root, "apps/backend/src/web/user/chat_handlers.rs"), "utf8").includes("broadcast_with_event_id_retry(")) fail("chat broadcast lacks bounded stable notification event retry");
const notificationServicePolicy = readFileSync(resolve(root, "services/notification/src/lib.rs"), "utf8");
if (!notificationServicePolicy.includes("(&Method::PUT, [id, \"acknowledge\"])") || !notificationServicePolicy.includes("\"acknowledge\"")) fail("notification acknowledgement service policy drifted");
const notificationGatewayPolicy = readFileSync(resolve(root, "services/gateway/src/policy.rs"), "utf8");
if (!notificationGatewayPolicy.includes("(&Method::PUT, [\"api\", \"v1\", \"notification\", _, \"acknowledge\"])") ) fail("notification acknowledgement gateway policy drifted");
for (const anchor of [
  "async fn read_notification_body_limited(",
  ".content_length()",
  "while let Some(chunk) = response",
  "let next_len = body",
  "if next_len > limit",
  "let value = match serde_json::from_slice::<serde_json::Value>(&body)",
  "let payload = match serde_json::from_slice::<NotificationListWire>(&body)",
  "payload.items.is_empty()"
]) if (!(unreadBoundary.includes(anchor) || frontendApi.includes(anchor))) fail(`notification bounded-body parsing contract drifted: ${anchor}`);
for (const forbidden of ["response.json::<serde_json::Value>()", "value.clone()", "payload.clone()"]) {
  if (listFunction.includes(forbidden)) fail(`notification list reintroduced unbounded or cloned parsing: ${forbidden}`);
}
for (const identityForward of ["x-user-id", "x-user-address", "x-wallet-address"]) {
  if (listFunction.toLowerCase().includes(identityForward) || unreadFunction.toLowerCase().includes(identityForward)) fail(`notification read BFF forwards unreviewed identity metadata: ${identityForward}`);
}
for (const anchor of [
  "#[serde(deny_unknown_fields)]",
  "struct NotificationUnreadCount {",
  "count: u64,",
  "let token = match verified_bearer(&state, &headers).await",
  "\"{}/api/v1/notification/unread-count\"",
  ".bearer_auth(bearer)",
  "read_notification_body_limited(response, NOTIFICATION_UNREAD_BODY_MAX)",
  "serde_json::from_slice::<NotificationUnreadCount>(&body)",
  "Ok(payload) if payload.count <= NOTIFICATION_UNREAD_JS_SAFE_MAX"
]) if (!(unreadBoundary.includes(anchor) || frontendApi.includes(anchor))) fail(`notification unread-count contract drifted: ${anchor}`);

const dormantNav = targetContents.get("tgt-dormant-nav-unavailable");
if (!dormantNav.includes("data-state=\"unavailable\"")) fail("dormant notification badge is not explicitly unavailable");
for (const forbidden of ["fetch(" + String.fromCharCode(39) + "/api/v1/notifications", "/api/v1/notifications?limit=1", "items.filter", ">0</span>"]) {
  if (dormantNav.includes(forbidden)) fail(`dormant notification badge reintroduced fabricated count behavior: ${forbidden}`);
}
const activeSsr = targetContents.get("tgt-active-header-mount");
const activeNavStart = activeSsr.indexOf("fn frontend_navigation_html(");
const activeNavEnd = activeSsr.indexOf("fn safe_return_url(", activeNavStart);
if (activeNavStart < 0 || activeNavEnd < 0 || !activeSsr.slice(activeNavStart, activeNavEnd).includes("epsx_templates::epsx_header_for_session_and_wallet_with_network(")) fail("active SSR shell no longer mounts the reviewed shared header");
for (const id of [
  "tgt-active-header-auth-runtime", "tgt-active-header-offline-exclusion", "tgt-active-header-endpoint",
  "tgt-active-header-exact-validation", "tgt-active-header-race-guard", "tgt-active-header-accessibility",
  "tgt-active-header-text-only"
]) {
  if (targetContents.get(id) !== activeSsr) fail(`${id}: active SSR evidence must share the mounted-header source`);
}
const authDerivation = activeSsr.indexOf("let is_authenticated = user.is_some();");
const runtimeInjection = activeSsr.indexOf("let authenticated_header_runtime = notification_badge_runtime(is_authenticated, &path);");
const bodyInjection = activeSsr.indexOf("{route_runtime}{authenticated_header_runtime}{chat_widget_html}</body>", runtimeInjection);
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
const cachePolicyStart = activeSsr.indexOf("fn apply_ssr_cache_policy(");
const cachePolicyEnd = activeSsr.indexOf("/// Fetch page-specific data", cachePolicyStart);
if (cachePolicyStart < 0 || cachePolicyEnd < 0 || cachePolicyStart >= cachePolicyEnd) fail("authenticated SSR cache policy boundary drifted");
const cachePolicy = activeSsr.slice(cachePolicyStart, cachePolicyEnd);
for (const anchor of [
  "if path == \"/offline\" {",
  "HeaderValue::from_static(\"public, max-age=0, must-revalidate\")",
  "} else if is_authenticated || recover_session || auth_page_verifier_unavailable {",
  "HeaderValue::from_static(\"private, no-store\")"
]) if (!cachePolicy.includes(anchor)) fail(`authenticated SSR cache policy drifted: ${anchor}`);
if (!activeSsr.includes("apply_ssr_cache_policy(\n        &mut response,\n        is_authenticated,\n        recover_session,\n        auth_page_verifier_unavailable,\n        &path,\n    );")) fail("SSR response no longer applies the reviewed private/public cache split");
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
  "return NotificationLoad::Malformed(Some(page));",
  ".unwrap_or_else(|| \"Notification\".to_string());",
  "let unread_label = format!(\"{unread_count} unread on this page\");",
  "href: \"{retry_href}\"",
  "fn NotificationMutationToolbar(has_unread: bool)",
  "data-notification-mutation-toolbar",
  "data-notification-mutation\": \"acknowledge\"",
  "data-notification-mutation\": \"delete\""
]) if (!ownerUiRuntime.includes(anchor)) fail(`notification owner read-only contract drifted: ${anchor}`);
for (const forbidden of [
  "notifications:read", "sample_notifications",
  "Enable Browser Notifications", "Test Notification", "Notification Settings", "SwitchInput",
  "BrowserNotificationsPrompt", "NotificationSettingsSection", "notification-action-url", "use_signal(",
  "onclick:", "notifications-filters"
]) if (ownerUiRuntime.includes(forbidden)) fail(`notification owner UI reintroduced blocked behavior: ${forbidden}`);

const adminService = targetContents.get("tgt-admin-global-list-handler");
const adminHandlerStart = adminService.indexOf("async fn list_admin_notifications(");
const adminHandlerEnd = adminService.indexOf("async fn get_notification(", adminHandlerStart);
const adminHandler = adminService.slice(adminHandlerStart, adminHandlerEnd);
for (const anchor of [
  "AdminNotificationQuery::parse(raw_query.as_deref())?",
  "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY",
  "sqlx::query_scalar(ADMIN_NOTIFICATION_COUNT_SQL)",
  ".bind(query.wallet_address.as_deref())",
  "sqlx::query_as(ADMIN_NOTIFICATION_LIST_SQL)",
  "admin_notification_cardinality_is_valid(total, query.limit, query.offset, rows.len())",
  ".collect::<Option<Vec<_>>>()",
  "StatusCode::INTERNAL_SERVER_ERROR"
]) if (!adminHandler.includes(anchor)) fail(`admin notification global-list handler drifted: ${anchor}`);
const adminSqlStart = adminService.indexOf("const ADMIN_NOTIFICATION_LIST_SQL");
const adminSqlEnd = adminService.indexOf(";", adminSqlStart);
const adminSql = adminService.slice(adminSqlStart, adminSqlEnd);
for (const anchor of [
  "SELECT id, title, subject, channel, CASE WHEN n.read_",
  "notification_type, priority, sent_at, created_at",
  "ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2"
]) if (!adminSql.includes(anchor)) fail(`admin notification redacted SQL drifted: ${anchor}`);
for (const forbidden of ["user_id", "recipient", "template_id", "body", "data", "error", "read_at", "action_url"]) {
  if (adminSql.includes(forbidden)) fail(`admin notification SQL selected private field: ${forbidden}`);
}
const adminServicePolicy = targetContents.get("tgt-admin-global-list-service-policy");
for (const anchor of [
  "(&Method::GET, [\"admin\", \"list\" | \"metrics\"]) => AccessPolicy::NotificationsAdmin",
  "principal.audience != ADMIN_AUDIENCE",
  "!principal.has_permission(NOTIFICATIONS_MANAGE_PERMISSION)"
]) if (!adminServicePolicy.includes(anchor)) fail(`admin notification direct policy drifted: ${anchor}`);
const adminGatewayPolicy = targetContents.get("tgt-admin-global-list-gateway-policy");
for (const anchor of [
  "(&Method::GET, [\"api\", \"v1\", \"notification\", \"admin\", \"list\" | \"metrics\"])",
  "AccessPolicy::Permission(\"admin:notifications:manage\")"
]) if (!adminGatewayPolicy.includes(anchor)) fail(`admin notification gateway policy drifted: ${anchor}`);
const gatewayMain = readFileSync(resolve(root, "services/gateway/src/main.rs"), "utf8");
for (const anchor of [
  "with_additional_audience(NOTIFICATION_PUBLISHER_AUDIENCE)",
  "with_additional_audience(NOTIFICATION_PROVIDER_AUDIENCE)"
]) if (!gatewayMain.includes(anchor)) fail(`gateway service-audience verifier drifted: ${anchor}`);
for (const anchor of [
  "AudiencePermission",
  "NOTIFICATION_PUBLISHER_AUDIENCE",
  "NOTIFICATION_PROVIDER_AUDIENCE",
  "NOTIFICATION_PROVIDER_EVENTS_PERMISSION"
]) if (!adminGatewayPolicy.includes(anchor)) fail(`gateway notification internal policy drifted: ${anchor}`);
const adminAdapter = targetContents.get("tgt-admin-global-list-adapter").split("#[cfg(test)]", 1)[0];
for (const anchor of [
  "MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES: usize = 256 * 1024",
  "#[serde(deny_unknown_fields)]",
  "clone_for_bearer()",
  ".bearer_auth(token)",
  "read_response_body_limited(response, MAX_ADMIN_NOTIFICATION_RESPONSE_BYTES)",
  "/api/v1/notification/admin/list?limit={ADMIN_NOTIFICATION_LIMIT}&offset={}"
]) if (!adminAdapter.includes(anchor)) fail(`admin notification strict adapter drifted: ${anchor}`);
for (const anchor of [
  "load_admin_notification_metrics(",
  "/api/v1/notification/admin/metrics",
  "MAX_ADMIN_NOTIFICATION_METRICS_RESPONSE_BYTES: usize = 32 * 1024",
  "decode_admin_notification_metrics(value)"
]) if (!adminAdapter.includes(anchor)) fail(`admin notification metrics adapter drifted: ${anchor}`);
const adminSsr = targetContents.get("tgt-admin-global-list-ssr");
for (const anchor of [
  "if route_path == \"/notifications/manage\" {",
  "load_admin_notifications(",
  "record_admin_notification_load(&mut params, &notification_query, load);",
  "load_admin_notification_metrics(",
  "record_admin_notification_metrics_load(&mut params, load);"
]) if (!adminSsr.includes(anchor)) fail(`admin notification SSR integration drifted: ${anchor}`);
const adminUi = targetContents.get("tgt-admin-global-list-ui").split("#[cfg(test)]", 1)[0];
for (const anchor of [
  "pub fn decode_admin_notification_projection(",
  "NotificationLoad::Forbidden",
  "NotificationLoad::Unavailable",
  "NotificationLoad::Malformed",
  "data-admin-notifications-page-state",
  "decode_admin_notification_metrics(",
  "data-admin-notifications-surface\": \"create\"",
  "form { class: \"mt-6 grid gap-5\", method: \"post\", action: \"/notifications/create\"",
  "name: \"idempotency_key\"",
  "name: \"recipient_wallet_address\"",
  "name: \"title\"",
  "name: \"message\"",
  "Send notification",
  "NotificationSendFeedback",
  "NotificationMetricsPanel",
  "Operational queue snapshot"
]) if (!adminUi.includes(anchor)) fail(`admin notification read-only UI drifted: ${anchor}`);
for (const forbidden of [
  "onclick:",
  "name: \"broadcast\"", "name: \"plan_id\"", "name: \"schedule\"",
  "name: \"image_url\"", "name: \"action_url\"", "name: \"data\""
]) {
  if (adminUi.includes(forbidden)) fail(`admin notification UI reintroduced mutation surface: ${forbidden}`);
}
const serviceMigration = targetContents.get("tgt-additive-service-migration");
if ((serviceMigration.match(/\bCREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+public\./gi) ?? []).length !== 2) fail("A3.11 guarded public-table evidence drifted");
if (/\b(?:DROP|ALTER|TRUNCATE|DELETE|INSERT|UPDATE|MERGE|COPY|CASCADE)\b/i.test(serviceMigration)) fail("A3.11 evidence contains destructive or data-mutating migration SQL");
const notificationKustomization = targetContents.get("tgt-kustomize-notification-service");
if (!notificationKustomization.includes("- ../../base/notification") || !notificationKustomization.includes("name: epsx-notification") || !notificationKustomization.includes("newTag: dev")) fail("notification Kubernetes deployment/service is missing from the dev resource inventory");
const notificationStagingKustomization = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/staging/kustomization.yaml"), "utf8");
if (!notificationStagingKustomization.includes("- ../../base/notification") || !notificationStagingKustomization.includes("name: epsx-notification") || !notificationStagingKustomization.includes("newTag: staging")) fail("notification Kubernetes deployment/service is missing from the staging resource inventory");
const notificationBaseKustomization = readFileSync(resolve(root, "infrastructure/kubernetes/base/kustomization.yaml"), "utf8");
const notificationProdKustomization = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/prod/kustomization.yaml"), "utf8");
if (notificationBaseKustomization.includes("notification/") || notificationProdKustomization.includes("epsx-notification")) fail("notification Kubernetes resources must remain absent from the production resource inventory");
for (const file of ["infrastructure/kubernetes/base/notification/deployment.yaml", "infrastructure/kubernetes/base/notification/service.yaml", "services/notification/Dockerfile"]) {
  if (!existsSync(resolve(root, file))) fail(`notification deployment artifact is missing: ${file}`);
}
const notificationDeployment = readFileSync(resolve(root, "infrastructure/kubernetes/base/notification/deployment.yaml"), "utf8");
for (const anchor of [
  "name: epsx-notification",
  "name: EPSX_ENV",
  "envFrom:\n            - secretRef:\n                name: epsx-notification",
  "image: epsx-notification:prod",
  "imagePullPolicy: IfNotPresent",
  "path: /health",
  "path: /ready",
  "runAsNonRoot: true",
  "readOnlyRootFilesystem: true",
  "containerPort: 8106"
]) if (!notificationDeployment.includes(anchor)) fail(`notification deployment contract drifted: ${anchor}`);
const notificationSecrets = readFileSync(resolve(root, "infrastructure/kubernetes/scripts/create-secrets.sh"), "utf8");
for (const key of [
  "--from-literal=DATABASE_URL=",
  "--from-literal=OIDC_ISSUER=",
  "--from-literal=OIDC_JWKS_URL=",
  "--from-literal=SMTP_HOST=",
  "--from-literal=REDIS_URL=",
  "--from-literal=NOTIFICATION_PLAN_DATABASE_URL=",
  "--from-literal=NOTIFICATION_PROVIDER_SIGNING_SECRET=",
  "--from-literal=NOTIFICATION_VAPID_PUBLIC_KEY=",
  "--from-literal=NOTIFICATION_VAPID_PREVIOUS_PRIVATE_KEY="
]) if (!notificationSecrets.includes(key)) fail(`notification managed-secret schema drifted: ${key}`);
const devNotificationKustomization = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/dev/kustomization.yaml"), "utf8");
if (!devNotificationKustomization.includes("patches/notification-environment.yaml")) fail("dev notification overlay must select its non-production environment patch");
const devNotificationEnvironment = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/dev/patches/notification-environment.yaml"), "utf8");
if (!devNotificationEnvironment.includes("name: EPSX_ENV") || !devNotificationEnvironment.includes("value: development")) fail("dev notification overlay must set EPSX_ENV=development");
const stagingNotificationKustomization = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/staging/kustomization.yaml"), "utf8");
if (!stagingNotificationKustomization.includes("patches/notification-environment.yaml")) fail("staging notification overlay must select its non-production environment patch");
const stagingNotificationEnvironment = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/staging/patches/notification-environment.yaml"), "utf8");
if (!stagingNotificationEnvironment.includes("name: EPSX_ENV") || !stagingNotificationEnvironment.includes("value: staging")) fail("staging notification overlay must set EPSX_ENV=staging");
const prodNotificationKustomization = readFileSync(resolve(root, "infrastructure/kubernetes/overlays/prod/kustomization.yaml"), "utf8");
if (prodNotificationKustomization.includes("epsx-notification") || prodNotificationKustomization.includes("notification-environment.yaml")) fail("production notification overlay must remain absent");
const lifecycleConstraints = readFileSync(resolve(root, "apps/backend/migrations/notifications/20260723140000_add_notification_lifecycle_constraints/up.sql"), "utf8");
for (const anchor of [
  "ALTER TABLE public.templates",
  "ALTER TABLE public.notifications",
  "ALTER TABLE public.notification_preferences",
  "ALTER TABLE public.notification_channel_jobs",
  "notification_replay_cursors_owner_check",
  "notification_request_idempotency_hash_check"
]) if (!lifecycleConstraints.includes(anchor)) fail(`notification lifecycle constraint migration drifted: ${anchor}`);

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
const exactOwnerReadObservation = "The frontend exposes GET-only list and unread-count BFF routes with explicit HEAD-to-405 overrides; all other non-GET methods are rejected. List permits only bounded status/limit/offset, forwards only a verified bearer, streams at most 2 MiB, parses the same bytes into exact wire and passthrough JSON values, applies bounded identity/recipient/text/data/action fields, and cross-checks every current-target row against the principal wallet; unread-count streams at most 4 KiB and requires the exact non-negative current-target DTO. SSR preserves explicit outcomes without sample fallback, admits only canonical all/read/unread status plus all ten bounded source notification types, five priorities, RFC3339 start/end dates, and derived page/offset semantics, while the Dioxus page renders native status/type/priority links, preserves date filters through pagination, and exposes bounded owner actions through the Rust BFF without inferring delivery. Cookie-backed mutations require same-origin context, bearer integrations remain cryptographically verified, and the route-scoped controller accepts only a closed action map with bounded IDs, exact {ok:true} responses, no-store credentials, and truthful error/reload states. The active shared header starts empty, hidden, and unavailable; only a server-verified authenticated non-offline response receives its unread-count controller. Authenticated HTML and every list/count BFF outcome are private/no-store, and the fetch bypasses caches. The unread controller uses the exact route with credentials, exact-object/non-negative-safe-integer validation, stale-response generation guards, zero/error hiding, AA badge contrast, a 99+ visual cap with the exact count in the accessible label, and text-only DOM writes. Local authenticated and responsive browser smoke now prove owner-route authentication and native layout; source method/query/envelope/broadcast/expiry/read parity and non-local runtime proof remain blocked.";
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
  cargo xtask notification-compatibility-audit --strict >/dev/null
  cargo xtask notification-producer-audit --strict >/dev/null
  echo "notification-execution: PASS — 14 source records, 88 target anchors, 12 surfaces, and 22 stop blockers verified"
  echo "notification-execution: LIMIT — A2.3c auth and A3.11 schema boundary remain partial; this static gate performs no database access, and no production adoption, source reconciliation, Redis, SMTP, push, network, deployment, or cutover readiness was proven"
  exit 0
fi

echo "notification-execution: STOP — 22 stop blockers remain; readiness is intentionally reserved as exit 3" >&2
echo "notification-execution: LIMIT — integrity may pass while notification lifecycle and delivery remain non-production" >&2
exit 3
