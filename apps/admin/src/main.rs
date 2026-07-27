use axum::{
    body::to_bytes,
    extract::{Path as AxPath, RawQuery, Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post},
    Json, Router,
};
use epsx_bff::{
    cookies::CookieEnvironment,
    middleware::security_headers,
    session::{JwksVerifier, JwksVerifierConfig, ADMIN_CLIENT_ID, JWKS_PATH},
};
use epsx_client::{RequestContext, ServiceClient};
use epsx_dioxus_ui::pages::admin_pages::payments::decode_admin_payment_intent_list;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

mod analytics_admin_adapter;
mod audit_log_adapter;
mod auth;
mod chat_admin_adapter;
mod commerce_admin_adapter;
mod dashboard_user_status_adapter;
mod developer_portal_adapter;
mod media_adapter;
mod news_adapter;
mod notification_admin_adapter;
mod session_auth;
#[cfg(test)]
mod session_auth_tests;
mod settings_adapter;
mod ssr;
mod wallet_stats_adapter;

#[derive(Clone)]
struct AppState {
    identity: Arc<ServiceClient>,
    wallet: Arc<ServiceClient>,
    payment: Arc<ServiceClient>,
    subscription: Arc<ServiceClient>,
    content: Arc<ServiceClient>,
    notification: Arc<ServiceClient>,
    analytics: Arc<ServiceClient>,
    indexer: Arc<ServiceClient>,
    verifier: Arc<JwksVerifier>,
    cookie_environment: CookieEnvironment,
    api_url: String,
    demo_login_enabled: bool,
}

const ADMIN_NOTIFICATION_FORM_MAX: usize = 20 * 1024;
const ADMIN_NOTIFICATION_FLASH_COOKIE: &str = "epsx.admin.notification_send";

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdminNotificationSendForm {
    recipient_wallet_address: String,
    title: String,
    message: String,
}

fn same_origin_admin_notification_form(headers: &HeaderMap) -> bool {
    let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let Some(host) = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let origin_host = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))
        .and_then(|value| value.split('/').next())
        .unwrap_or("");
    !origin_host.is_empty()
        && origin_host == host
        && headers
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok())
            .is_none_or(|value| matches!(value, "same-origin" | "same-site"))
}

fn valid_admin_wallet(value: &str) -> bool {
    value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_admin_form_text(value: &str, max_bytes: usize, required: bool) -> bool {
    (!required || !value.trim().is_empty())
        && value.len() <= max_bytes
        && !value.chars().any(char::is_control)
}

fn parse_admin_notification_form(body: &[u8]) -> Result<AdminNotificationSendForm, ()> {
    let mut fields = std::collections::BTreeMap::new();
    for (key, value) in url::form_urlencoded::parse(body) {
        if key.len() > 64
            || value.len() > 16 * 1024
            || fields
                .insert(key.into_owned(), value.into_owned())
                .is_some()
        {
            return Err(());
        }
    }
    if fields.keys().any(|key| {
        !matches!(
            key.as_str(),
            "recipient_wallet_address" | "title" | "message"
        )
    }) {
        return Err(());
    }
    let recipient_wallet_address = fields.remove("recipient_wallet_address").ok_or(())?;
    let title = fields.remove("title").ok_or(())?;
    let message = fields.remove("message").ok_or(())?;
    if !valid_admin_wallet(&recipient_wallet_address)
        || !valid_admin_form_text(&title, 255, true)
        || !valid_admin_form_text(&message, 16 * 1024, true)
    {
        return Err(());
    }
    Ok(AdminNotificationSendForm {
        recipient_wallet_address: recipient_wallet_address.to_ascii_lowercase(),
        title,
        message,
    })
}

fn admin_notification_form_redirect(state: &'static str) -> Response {
    let state = matches!(state, "accepted")
        .then_some("accepted")
        .unwrap_or("error");
    let mut response = Redirect::to("/notifications/manage?send=accepted").into_response();
    if state == "error" {
        response = Redirect::to("/notifications/manage?send=error").into_response();
    }
    let cookie = format!(
        "{ADMIN_NOTIFICATION_FLASH_COOKIE}={state}; Path=/notifications; Max-Age=30; HttpOnly; SameSite=Lax"
    );
    response.headers_mut().append(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("notification flash cookie is bounded"),
    );
    response
}

#[derive(Deserialize)]
struct SiweLoginBody {
    message: String,
    signature: String,
    #[serde(default)]
    #[serde(rename = "chain_id")]
    _chain_id: String,
    address: String,
    nonce: String,
}

#[derive(Deserialize)]
struct ChallengeBody {
    address: String,
}

#[derive(Deserialize)]
struct DemoLoginBody {
    #[serde(rename = "address")]
    _address: Option<String>,
    #[serde(rename = "chain_id")]
    _chain_id: Option<String>,
}

fn ctx_from(headers: &HeaderMap) -> RequestContext {
    RequestContext::from_headers(headers)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PaymentIntentQuery {
    pub(crate) payer: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) limit: u64,
    pub(crate) offset: u64,
}

/// Bound deep pagination well below the pay service's signed `i64` extractor
/// limit. Larger offsets are treated as invalid rather than being forwarded and
/// silently interpreted differently upstream.
const MAX_PAYMENT_INTENT_OFFSET: u64 = 10_000_000;

impl PaymentIntentQuery {
    pub(crate) fn from_raw(raw_query: &str) -> Result<Self, ()> {
        let mut parsed = Self {
            payer: None,
            status: None,
            limit: 20,
            offset: 0,
        };
        let mut payer_seen = false;
        let mut status_seen = false;
        let mut limit_seen = false;
        let mut offset_seen = false;
        let mut url = reqwest::Url::parse("http://admin.invalid/")
            .expect("the fixed payment query base URL is valid");
        url.set_query((!raw_query.is_empty()).then_some(raw_query));

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "payer" => {
                    if payer_seen {
                        return Err(());
                    }
                    payer_seen = true;
                    parsed.payer = if value.is_empty() {
                        None
                    } else {
                        Some(safe_payment_query_value(&value, 128).ok_or(())?)
                    };
                }
                "status" => {
                    if status_seen {
                        return Err(());
                    }
                    status_seen = true;
                    parsed.status = if value.is_empty() {
                        None
                    } else {
                        Some(safe_payment_query_value(&value, 32).ok_or(())?)
                    };
                }
                "limit" => {
                    if limit_seen {
                        return Err(());
                    }
                    limit_seen = true;
                    let limit = value.parse::<u64>().map_err(|_| ())?;
                    if !(1..=100).contains(&limit) {
                        return Err(());
                    }
                    parsed.limit = limit;
                }
                "offset" => {
                    if offset_seen {
                        return Err(());
                    }
                    offset_seen = true;
                    let offset = value.parse::<u64>().map_err(|_| ())?;
                    if offset > MAX_PAYMENT_INTENT_OFFSET {
                        return Err(());
                    }
                    parsed.offset = offset;
                }
                _ => {}
            }
        }
        Ok(parsed)
    }

    pub(crate) fn upstream_path(&self) -> String {
        let mut pairs = Vec::with_capacity(4);
        if let Some(payer) = &self.payer {
            pairs.push(format!("payer={payer}"));
        }
        if let Some(status) = &self.status {
            pairs.push(format!("status={status}"));
        }
        pairs.push(format!("limit={}", self.limit));
        pairs.push(format!("offset={}", self.offset));
        format!("/api/v1/admin/pay/intents?{}", pairs.join("&"))
    }
}

fn safe_payment_query_value(value: &str, max_len: usize) -> Option<String> {
    (!value.is_empty()
        && value.len() <= max_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-')))
    .then(|| value.to_string())
}

pub(crate) fn payment_tab(raw_query: &str) -> Result<&'static str, ()> {
    let mut url = reqwest::Url::parse("http://admin.invalid/")
        .expect("the fixed payment query base URL is valid");
    url.set_query((!raw_query.is_empty()).then_some(raw_query));
    let mut tab = None;
    for (key, value) in url.query_pairs() {
        if key != "tab" {
            continue;
        }
        if tab.is_some() {
            return Err(());
        }
        tab = Some(match value.as_ref() {
            "payments" => "payments",
            "user-access" => "user-access",
            "payment-links" => "payment-links",
            _ => return Err(()),
        });
    }
    Ok(tab.unwrap_or("payments"))
}

fn safe_upstream_status(status: u16) -> Option<StatusCode> {
    match status {
        400 => Some(StatusCode::BAD_REQUEST),
        401 => Some(StatusCode::UNAUTHORIZED),
        403 => Some(StatusCode::FORBIDDEN),
        404 => Some(StatusCode::NOT_FOUND),
        409 => Some(StatusCode::CONFLICT),
        422 => Some(StatusCode::UNPROCESSABLE_ENTITY),
        429 => Some(StatusCode::TOO_MANY_REQUESTS),
        502 => Some(StatusCode::BAD_GATEWAY),
        503 => Some(StatusCode::SERVICE_UNAVAILABLE),
        504 => Some(StatusCode::GATEWAY_TIMEOUT),
        _ => None,
    }
}

fn err_to_status(e: epsx_client::ClientError) -> StatusCode {
    use epsx_client::ClientError::*;
    match e {
        Unauthorized => StatusCode::UNAUTHORIZED,
        NotFound => StatusCode::NOT_FOUND,
        Timeout => StatusCode::GATEWAY_TIMEOUT,
        Http(error) if error.is_timeout() => StatusCode::GATEWAY_TIMEOUT,
        Http(error) if error.is_connect() => StatusCode::SERVICE_UNAVAILABLE,
        UpstreamStatus(status) => safe_upstream_status(status).unwrap_or(StatusCode::BAD_GATEWAY),
        Http(_) | Service(_) | Serde(_) => StatusCode::BAD_GATEWAY,
    }
}

#[cfg(test)]
mod upstream_error_mapping_tests {
    use super::*;
    use epsx_client::ClientError;

    #[test]
    fn preserves_the_closed_safe_upstream_status_set() {
        for (code, expected) in [
            ("400", StatusCode::BAD_REQUEST),
            ("401", StatusCode::UNAUTHORIZED),
            ("403", StatusCode::FORBIDDEN),
            ("404", StatusCode::NOT_FOUND),
            ("409", StatusCode::CONFLICT),
            ("422", StatusCode::UNPROCESSABLE_ENTITY),
            ("429", StatusCode::TOO_MANY_REQUESTS),
            ("502", StatusCode::BAD_GATEWAY),
            ("503", StatusCode::SERVICE_UNAVAILABLE),
            ("504", StatusCode::GATEWAY_TIMEOUT),
        ] {
            let error = ClientError::UpstreamStatus(code.parse().unwrap());
            assert_eq!(err_to_status(error), expected, "upstream status {code}");
        }
    }

    #[test]
    fn rejects_arbitrary_or_unsafe_upstream_statuses() {
        for code in [200, 402, 418, 500, 599, 4030] {
            assert_eq!(
                err_to_status(ClientError::UpstreamStatus(code)),
                StatusCode::BAD_GATEWAY,
                "{code}"
            );
        }
    }

    #[test]
    fn typed_auth_resource_and_timeout_errors_keep_their_semantics() {
        assert_eq!(
            err_to_status(ClientError::Unauthorized),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(err_to_status(ClientError::NotFound), StatusCode::NOT_FOUND);
        assert_eq!(
            err_to_status(ClientError::Timeout),
            StatusCode::GATEWAY_TIMEOUT
        );
    }

    #[test]
    fn legacy_service_details_are_never_parsed_or_exposed_as_statuses() {
        let sensitive = "Bearer secret\r\nset-cookie: stolen=yes";
        assert_eq!(
            err_to_status(ClientError::Service(format!(
                "status 403 Forbidden: {sensitive}"
            ))),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(
            err_to_status(ClientError::Service(format!(
                "status 500 Internal Server Error: {sensitive}"
            ))),
            StatusCode::BAD_GATEWAY
        );
    }
}

#[cfg(test)]
mod payment_intent_adapter_tests {
    use super::*;

    #[test]
    fn payment_query_forwards_only_the_read_allowlist() {
        let query = PaymentIntentQuery::from_raw(
            "payer=0xabc&status=pending&limit=50&offset=100&tab=payments&force=cancel",
        )
        .unwrap();
        assert_eq!(query.payer.as_deref(), Some("0xabc"));
        assert_eq!(query.status.as_deref(), Some("pending"));
        assert_eq!(query.limit, 50);
        assert_eq!(query.offset, 100);
        assert_eq!(
            query.upstream_path(),
            "/api/v1/admin/pay/intents?payer=0xabc&status=pending&limit=50&offset=100"
        );
        assert!(!query.upstream_path().contains("force"));
    }

    #[test]
    fn payment_query_rejects_invalid_recognized_values_instead_of_broadening() {
        for raw in [
            "payer=%26admin%3Dtrue",
            "status=%0D%0Aauthorization",
            "limit=0",
            "limit=999",
            "offset=-1",
            "offset=10000001",
        ] {
            assert!(PaymentIntentQuery::from_raw(raw).is_err(), "{raw}");
        }
        assert!(MAX_PAYMENT_INTENT_OFFSET < i64::MAX as u64);
    }

    #[test]
    fn payment_query_uses_bounded_defaults_only_when_values_are_absent() {
        let query = PaymentIntentQuery::from_raw("payer=&status=&tab=payments").unwrap();
        assert_eq!(query.payer, None);
        assert_eq!(query.status, None);
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert_eq!(
            query.upstream_path(),
            "/api/v1/admin/pay/intents?limit=20&offset=0"
        );
    }

    #[test]
    fn payment_query_rejects_duplicate_recognized_parameters() {
        for raw in [
            "payer=0xfirst&payer=0xsecond",
            "status=pending&status=released",
            "limit=20&limit=50",
            "offset=0&offset=20",
        ] {
            assert!(PaymentIntentQuery::from_raw(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn payment_tab_accepts_only_known_read_surfaces() {
        assert_eq!(payment_tab(""), Ok("payments"));
        assert_eq!(payment_tab("tab=payments"), Ok("payments"));
        assert_eq!(payment_tab("tab=user-access"), Ok("user-access"));
        assert_eq!(payment_tab("tab=payment-links"), Ok("payment-links"));
        assert!(payment_tab("tab=create-link").is_err());
        assert!(payment_tab("tab=%0D%0Aevil").is_err());
        assert!(payment_tab("tab=payments&tab=payment-links").is_err());
    }
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-admin");

    // Wave 21 — dev auth bypass banner. Always evaluated (cheap) so the
    // log line is honest about the process state. Default is OFF; the
    // env var must be set to "1" to flip it on.
    if epsx_bff::dev_bypass::is_dev_bypass_enabled() {
        tracing::warn!(
            "EPSX_DEV_AUTH_BYPASS=1 — every request is treated as logged in as dev admin (0x...d3v1). NEVER enable in production."
        );
    }

    let state = state_from_env().expect("valid admin authentication configuration");
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let api_url = state.api_url.clone();
    let app = build_app(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Admin BFF listening on http://{} (api={})", addr, api_url);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn state_from_env() -> Result<AppState, String> {
    let cookie_environment = CookieEnvironment::from_env().map_err(|error| error.to_string())?;
    let api_url = std::env::var("API_URL")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .map_err(|_| "API_URL or BACKEND_URL is required".to_string())?;
    let notification_url =
        std::env::var("NOTIFICATION_SERVICE_URL").unwrap_or_else(|_| api_url.clone());
    let issuer = std::env::var("OIDC_ISSUER")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .map_err(|_| "OIDC_ISSUER or BACKEND_URL is required".to_string())?;
    validate_auth_url(&api_url, cookie_environment, "API_URL/BACKEND_URL")?;
    validate_auth_url(
        &notification_url,
        cookie_environment,
        "NOTIFICATION_SERVICE_URL",
    )?;
    validate_auth_url(&issuer, cookie_environment, "OIDC_ISSUER/BACKEND_URL")?;

    let demo_login_enabled = std::env::var("EPSX_ENABLE_DEMO_LOGIN").ok().as_deref() == Some("1");
    let dev_bypass_enabled = std::env::var("EPSX_DEV_AUTH_BYPASS").ok().as_deref() == Some("1");
    if cookie_environment == CookieEnvironment::Production
        && (demo_login_enabled || dev_bypass_enabled)
    {
        return Err("demo login and auth bypass are forbidden in production".to_string());
    }

    let verifier_config = JwksVerifierConfig::new(
        format!("{}{}", api_url.trim_end_matches('/'), JWKS_PATH),
        issuer.trim_end_matches('/'),
        ADMIN_CLIENT_ID,
        Duration::from_secs(300),
    )
    .map_err(|error| error.to_string())?;
    let verifier =
        Arc::new(JwksVerifier::with_http(verifier_config).map_err(|error| error.to_string())?);
    let cfg = epsx_client::ClientConfig {
        base_url: api_url.clone(),
        timeout: Duration::from_secs(15),
    };
    let notification_cfg = epsx_client::ClientConfig {
        base_url: notification_url,
        timeout: Duration::from_secs(15),
    };
    Ok(AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(notification_cfg)),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        indexer: Arc::new(ServiceClient::new(cfg)),
        verifier,
        cookie_environment,
        api_url,
        demo_login_enabled,
    })
}

fn validate_auth_url(
    value: &str,
    environment: CookieEnvironment,
    label: &str,
) -> Result<(), String> {
    let url = reqwest::Url::parse(value).map_err(|_| format!("{label} must be an absolute URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(format!("{label} has forbidden URL components"));
    }
    if environment == CookieEnvironment::Production && url.scheme() != "https" {
        return Err(format!("{label} must use HTTPS in production"));
    }
    if environment == CookieEnvironment::Production {
        let host = url
            .host_str()
            .ok_or_else(|| format!("{label} must include a host"))?;
        let local_host = host.eq_ignore_ascii_case("localhost")
            || host.ends_with(".localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback());
        if local_host {
            return Err(format!("{label} must not use a local host in production"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn production_requires_https_non_local_auth_urls() {
        assert!(validate_auth_url(
            "https://api.epsx.io",
            CookieEnvironment::Production,
            "API_URL"
        )
        .is_ok());
        assert!(validate_auth_url(
            "http://api.epsx.io",
            CookieEnvironment::Production,
            "API_URL"
        )
        .is_err());
        assert!(validate_auth_url(
            "https://localhost:8080",
            CookieEnvironment::Production,
            "API_URL"
        )
        .is_err());
    }

    #[test]
    fn local_mode_allows_http_but_rejects_ambiguous_components() {
        assert!(
            validate_auth_url("http://localhost:8080", CookieEnvironment::Local, "API_URL").is_ok()
        );
        assert!(validate_auth_url(
            "http://localhost:8080?issuer=other",
            CookieEnvironment::Local,
            "API_URL"
        )
        .is_err());
    }

    #[test]
    fn template_audit_projection_is_bounded_and_versioned() {
        let payload = serde_json::from_value::<BackendTemplateAuditEnvelope>(serde_json::json!({
            "items": [{
                "id": "template-audit-1",
                "template_id": "0xtemplate",
                "action": "rollback",
                "from_version": 2,
                "to_version": 3,
                "actor_subject": "admin-subject",
                "metadata": {"restored_version": 2, "new_version": 3},
                "created_at": "2026-07-24T00:00:00Z"
            }]
        }))
        .unwrap();
        assert!(valid_template_audit_payload(&payload));

        let mut unknown_action = payload;
        unknown_action.items[0].action = "preview".into();
        assert!(!valid_template_audit_payload(&unknown_action));
        let mut malformed_metadata =
            serde_json::from_value::<BackendTemplateAuditEnvelope>(serde_json::json!({
                "items": [{
                    "id": "template-audit-1",
                    "template_id": "0xtemplate",
                    "action": "updated",
                    "from_version": null,
                    "to_version": 1,
                    "actor_subject": "admin-subject",
                    "metadata": [],
                    "created_at": "2026-07-24T00:00:00Z"
                }]
            }))
            .unwrap();
        assert!(!valid_template_audit_payload(&malformed_metadata));
        malformed_metadata.items[0].action = "rollback".into();
        malformed_metadata.items[0].metadata =
            serde_json::json!({"restored_version": 1, "new_version": 2, "secret": "nope"});
        assert!(!valid_template_audit_payload(&malformed_metadata));
        malformed_metadata.items[0].to_version = Some(0);
        assert!(!valid_template_audit_payload(&malformed_metadata));
    }

    #[test]
    fn template_list_projection_is_strict_bounded_and_typed() {
        let payload = serde_json::from_value::<BackendTemplateList>(serde_json::json!({
            "items": [{
                "id": "0xtemplate",
                "name": "welcome",
                "channel": "in_app",
                "subject": "Welcome",
                "body": "Hello {{name}}",
                "variables": {"name": {"type": "string", "required": true}},
                "active": true,
                "created_at": "2026-07-24T00:00:00Z",
                "updated_at": "2026-07-24T00:00:00Z"
            }],
            "total": 1
        }))
        .unwrap();
        assert!(valid_template_list_payload(&payload));

        let mut negative = payload;
        negative.total = -1;
        assert!(!valid_template_list_payload(&negative));

        let mut invalid = serde_json::from_value::<BackendTemplateList>(serde_json::json!({
            "items": [{
                "id": "0xtemplate",
                "name": "welcome",
                "channel": "sms",
                "subject": null,
                "body": "Hello",
                "variables": {"name": {"type": "string", "unknown": true}},
                "active": true,
                "created_at": "2026-07-24T00:00:00Z",
                "updated_at": "2026-07-24T00:00:00Z"
            }],
            "total": 1
        }))
        .unwrap();
        assert!(!valid_template_list_payload(&invalid));
        invalid.items[0].channel = "in_app".into();
        invalid.items[0].variables = serde_json::json!({
            "name": {"type": "string", "description": "line\ncontrol"}
        });
        assert!(!valid_template_list_payload(&invalid));

        assert!(
            serde_json::from_value::<BackendTemplateList>(serde_json::json!({
                "items": [], "total": 0, "private": "nope"
            }))
            .is_err()
        );
    }

    #[test]
    fn notification_metrics_projection_is_non_negative_and_channel_bounded() {
        let payload = BackendNotificationMetrics {
            queue_depth: 1,
            queue_age_seconds: Some(2),
            suppressed: 0,
            retry_wait: 0,
            terminal_failed: 0,
            dead_lettered: 0,
            provider_accepted: 1,
            attempting: 0,
            channel_outcomes: BTreeMap::from([(String::from("in_app"), 1)]),
            provider_events: 1,
            delivery_attempts: 1,
            replay_cursors: 1,
            replay_cursor_age_seconds: Some(3),
            active_streams: 1,
            stream_connections_total: 2,
            stream_reconnects_total: 1,
            stream_replayed_events_total: 1,
            stream_lag_seconds: Some(1),
            stream_query_failures_total: 0,
        };
        assert!(valid_notification_metrics(&payload));
        let mut negative = payload;
        negative.queue_depth = -1;
        assert!(!valid_notification_metrics(&negative));
        let mut unknown_channel = negative;
        unknown_channel.queue_depth = 0;
        unknown_channel.channel_outcomes.insert("sms".into(), 1);
        assert!(!valid_notification_metrics(&unknown_channel));
    }
}

fn build_app(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/api/health", get(api_health))
        // Auth
        .route("/api/v1/auth/challenge", post(session_auth::auth_challenge))
        .route("/api/v1/auth/siwe", post(session_auth::siwe_login))
        .route("/api/v1/auth/login", post(session_auth::siwe_login))
        .route("/api/v1/auth/refresh", post(session_auth::refresh_token))
        .route("/api/v1/auth/demo", post(session_auth::demo_login))
        .route("/api/v1/auth/me", get(session_auth::auth_me))
        .route("/api/v1/auth/logout", post(session_auth::logout))
        // The legacy admin login page no longer owns authentication. Keep the
        // browser entry point deterministic while the canonical auth flow is
        // reconciled, and never let query input choose the redirect target.
        .route("/auth", get(admin_auth_redirect))
        // Wave 43 T1 A3 — prod-mirror middleware: redirect
        // `/wallet-management` → `/wallet-management/wallets` to
        // match prod's Vercel middleware (3-hop redirect chain).
        .route("/wallet-management", get(wallet_redirect_to_wallets))
        // Users (identity)
        .route("/api/v1/users", get(list_users).post(create_user))
        .route(
            "/api/v1/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
        // Payments: the migration slice exposes only the canonical admin-wide
        // read contract. Owner reads and financial mutations stay unregistered.
        .route("/api/v1/payments", get(list_payments))
        // Subscriptions
        .route("/api/v1/subscriptions", get(list_subscriptions))
        .route("/api/v1/subscriptions/{id}", get(get_subscription))
        .route(
            "/api/v1/subscriptions/{id}/cancel",
            post(cancel_subscription),
        )
        .route(
            "/api/v1/subscription/plans",
            get(list_plans).post(create_plan),
        )
        .route("/api/v1/subscription/plans/{id}", get(get_plan))
        // Content
        .route("/api/v1/pages", get(list_pages).post(create_page))
        .route("/api/v1/pages/{slug}", get(get_page).put(update_page))
        .route("/api/v1/pages/{slug}/publish", post(publish_page))
        .route("/api/v1/themes", get(list_themes).post(create_theme))
        .route("/api/v1/themes/{id}", get(get_theme).put(update_theme))
        .route("/api/v1/blocks", get(list_blocks))
        .route("/api/v1/content/navigation", get(content_navigation))
        .route("/api/v1/content/site", get(content_site))
        // Notifications
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/{id}/read", post(mark_read))
        .route("/api/v1/notifications/{id}", delete(delete_notification))
        .route(
            "/api/v1/notifications/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/v1/notifications/templates/{id}",
            delete(delete_template),
        )
        .route(
            "/api/v1/notifications/templates/{id}/preview",
            post(preview_template),
        )
        .route(
            "/api/v1/notifications/templates/{id}/rollback",
            post(rollback_template),
        )
        .route(
            "/api/v1/notifications/templates/{id}/audit",
            get(template_audit),
        )
        .route("/api/v1/notifications/metrics", get(notification_metrics))
        .route("/api/v1/notifications/send", post(send_notification))
        // The source app exposes `/notifications/create` as a page and the
        // form posts back to the same URL. Keep both methods on one route so
        // a browser GET renders the Dioxus page instead of Axum returning a
        // method-mismatch 405 before the SSR fallback can run.
        .route(
            "/notifications/create",
            get(fallback_handler).post(submit_notification_form),
        )
        // Analytics
        .route("/api/v1/analytics/events", get(list_events))
        .route("/api/v1/analytics/metrics/{metric}", get(get_metrics))
        .route("/api/v1/analytics/revenue", get(revenue))
        .route("/api/v1/analytics/track", post(track_event))
        // Indexer
        .route("/api/v1/indexer/status/{chain}", get(chain_status))
        .route("/api/v1/indexer/block/{chain}/{number}", get(get_block))
        .route("/api/v1/indexer/tx/{chain}/{hash}", get(get_tx))
        .route(
            "/api/v1/indexer/transfers/{chain}/{address}",
            get(get_transfers),
        )
        // Wallet
        .route("/api/v1/wallet/accounts", get(list_accounts))
        .route("/api/v1/wallet/accounts/{address}", get(get_account))
        // Static assets
        .nest_service(
            "/public",
            tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR")))
                .fallback(tower_http::services::ServeFile::new(format!(
                    "{}/public/index.html",
                    env!("CARGO_MANIFEST_DIR")
                ))),
        )
        // SSR fallback (Dioxus)
        .fallback(fallback_handler)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_verified_admin_session,
        ))
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state)
}

async fn require_verified_admin_session(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if is_api_path(path) && !is_known_admin_api_path(path) {
        return api_not_found_response();
    }
    if !is_known_protected_admin_api_path(path) {
        return next.run(request).await;
    }
    if !is_allowed_protected_admin_api_method(request.method(), path) {
        // The path is registered, but this method is not. Let Axum's
        // MethodRouter return its canonical 405 without demanding a session.
        return next.run(request).await;
    }

    let Some((token, _user)) = auth::verified_access_token(
        request.headers(),
        state.verifier.as_ref(),
        state.cookie_environment,
    )
    .await
    else {
        return session_auth::clear_session_response(
            &state,
            StatusCode::UNAUTHORIZED,
            "invalid_access_token",
        );
    };
    let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) else {
        return session_auth::clear_session_response(
            &state,
            StatusCode::UNAUTHORIZED,
            "invalid_access_token",
        );
    };
    request.headers_mut().insert(header::AUTHORIZATION, value);
    next.run(request).await
}

fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn api_segments(path: &str) -> Option<Vec<&str>> {
    if !is_api_path(path) || path.ends_with('/') {
        return None;
    }
    let segments: Vec<_> = path.trim_start_matches('/').split('/').collect();
    segments
        .iter()
        .all(|segment| !segment.is_empty())
        .then_some(segments)
}

fn is_known_public_admin_api_path(path: &str) -> bool {
    let Some(segments) = api_segments(path) else {
        return false;
    };
    matches!(
        segments.as_slice(),
        ["api", "health"]
            | ["api", "v1", "auth", "challenge"]
            | ["api", "v1", "auth", "siwe"]
            | ["api", "v1", "auth", "login"]
            | ["api", "v1", "auth", "refresh"]
            | ["api", "v1", "auth", "demo"]
            | ["api", "v1", "auth", "me"]
            | ["api", "v1", "auth", "logout"]
            | ["api", "v1", "blocks"]
            | ["api", "v1", "content", "navigation"]
            | ["api", "v1", "content", "site"]
    )
}

/// Exact path-shape allowlist for registered protected admin API routes. The
/// authentication middleware uses this before verification so an unknown API
/// miss cannot be converted into a 401 or SSR HTML response.
fn is_known_protected_admin_api_path(path: &str) -> bool {
    let Some(segments) = api_segments(path) else {
        return false;
    };
    matches!(
        segments.as_slice(),
        ["api", "v1", "users"]
            | ["api", "v1", "users", _]
            | ["api", "v1", "payments"]
            | ["api", "v1", "subscriptions"]
            | ["api", "v1", "subscriptions", _]
            | ["api", "v1", "subscriptions", _, "cancel"]
            | ["api", "v1", "subscription", "plans"]
            | ["api", "v1", "subscription", "plans", _]
            | ["api", "v1", "pages"]
            | ["api", "v1", "pages", _]
            | ["api", "v1", "pages", _, "publish"]
            | ["api", "v1", "themes"]
            | ["api", "v1", "themes", _]
            | ["api", "v1", "notifications"]
            | ["api", "v1", "notifications", "templates"]
            | ["api", "v1", "notifications", "templates", _]
            | ["api", "v1", "notifications", "templates", _, "preview"]
            | ["api", "v1", "notifications", "templates", _, "rollback"]
            | ["api", "v1", "notifications", "templates", _, "audit"]
            | ["api", "v1", "notifications", "send"]
            | ["api", "v1", "notifications", _]
            | ["api", "v1", "notifications", _, "read"]
            | ["api", "v1", "analytics", "events"]
            | ["api", "v1", "analytics", "metrics", _]
            | ["api", "v1", "analytics", "revenue"]
            | ["api", "v1", "analytics", "track"]
            | ["api", "v1", "indexer", "status", _]
            | ["api", "v1", "indexer", "block", _, _]
            | ["api", "v1", "indexer", "tx", _, _]
            | ["api", "v1", "indexer", "transfers", _, _]
            | ["api", "v1", "wallet", "accounts"]
            | ["api", "v1", "wallet", "accounts", _]
    )
}

fn is_allowed_protected_admin_api_method(method: &Method, path: &str) -> bool {
    let Some(segments) = api_segments(path) else {
        return false;
    };
    let is_read = method == Method::GET || method == Method::HEAD;
    match segments.as_slice() {
        ["api", "v1", "users"] => is_read || method == Method::POST,
        ["api", "v1", "users", _] => is_read || method == Method::PUT || method == Method::DELETE,
        ["api", "v1", "payments"]
        | ["api", "v1", "subscriptions"]
        | ["api", "v1", "subscriptions", _]
        | ["api", "v1", "subscription", "plans", _]
        | ["api", "v1", "analytics", "events"]
        | ["api", "v1", "analytics", "metrics", _]
        | ["api", "v1", "analytics", "revenue"]
        | ["api", "v1", "indexer", "status", _]
        | ["api", "v1", "indexer", "block", _, _]
        | ["api", "v1", "indexer", "tx", _, _]
        | ["api", "v1", "indexer", "transfers", _, _]
        | ["api", "v1", "wallet", "accounts"]
        | ["api", "v1", "wallet", "accounts", _] => is_read,
        ["api", "v1", "subscriptions", _, "cancel"]
        | ["api", "v1", "pages", _, "publish"]
        | ["api", "v1", "notifications", _, "read"]
        | ["api", "v1", "notifications", "send"]
        | ["api", "v1", "notifications", "templates", _, "preview"]
        | ["api", "v1", "notifications", "templates", _, "rollback"] => method == Method::POST,
        ["api", "v1", "notifications", "templates", _, "audit"] => is_read,
        ["api", "v1", "notifications", "metrics"] => is_read,
        ["api", "v1", "analytics", "track"] => method == Method::POST,
        ["api", "v1", "subscription", "plans"]
        | ["api", "v1", "pages"]
        | ["api", "v1", "themes"]
        | ["api", "v1", "notifications", "templates"] => is_read || method == Method::POST,
        ["api", "v1", "pages", _] | ["api", "v1", "themes", _] => is_read || method == Method::PUT,
        ["api", "v1", "notifications"] => is_read,
        ["api", "v1", "notifications", "templates", _] | ["api", "v1", "notifications", _] => {
            method == Method::DELETE
        }
        _ => false,
    }
}

fn is_known_admin_api_path(path: &str) -> bool {
    is_known_public_admin_api_path(path) || is_known_protected_admin_api_path(path)
}

fn api_not_found_response() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "message": "API route not found"
        })),
    )
        .into_response()
}

async fn fallback_handler(State(state): State<AppState>, request: Request) -> Response {
    if is_api_path(request.uri().path()) {
        api_not_found_response()
    } else {
        ssr::ssr_handler(State(state), request).await
    }
}

#[cfg(test)]
mod routing_tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{header, Method, Request, StatusCode},
    };
    use epsx_bff::session::{Jwks, JwksFetcher, SessionError};
    use std::{future::Future, pin::Pin};
    use tower::ServiceExt;

    struct FailingJwksFetcher;

    impl JwksFetcher for FailingJwksFetcher {
        fn fetch<'life0, 'life1, 'async_trait>(
            &'life0 self,
            _url: &'life1 str,
        ) -> Pin<Box<dyn Future<Output = Result<Jwks, SessionError>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async { Err(SessionError::JwksFetch("deterministic test outage".into())) })
        }
    }

    fn test_state() -> AppState {
        let base_url = "http://127.0.0.1:9";
        let config = epsx_client::ClientConfig {
            base_url: base_url.to_string(),
            timeout: Duration::from_millis(50),
        };
        let client = Arc::new(ServiceClient::new(config));
        let verifier_config = JwksVerifierConfig::new(
            format!("{base_url}{JWKS_PATH}"),
            "https://issuer.test",
            ADMIN_CLIENT_ID,
            Duration::from_secs(60),
        )
        .unwrap();
        AppState {
            identity: client.clone(),
            wallet: client.clone(),
            payment: client.clone(),
            subscription: client.clone(),
            content: client.clone(),
            notification: client.clone(),
            analytics: client.clone(),
            indexer: client,
            verifier: Arc::new(JwksVerifier::new(
                verifier_config,
                Arc::new(FailingJwksFetcher),
            )),
            cookie_environment: CookieEnvironment::Local,
            api_url: base_url.to_string(),
            demo_login_enabled: false,
        }
    }

    async fn request(method: Method, uri: &str) -> Response {
        request_with_cookie(method, uri, None).await
    }

    async fn request_with_cookie(method: Method, uri: &str, cookie: Option<&str>) -> Response {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }
        build_app(test_state())
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[test]
    fn notification_form_parser_is_closed_bounded_and_canonicalizes_wallets() {
        let wallet = "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let body =
            format!("recipient_wallet_address={wallet}&title=Hello+admin&message=Queued+message");
        let parsed = parse_admin_notification_form(body.as_bytes()).unwrap();
        assert_eq!(parsed.recipient_wallet_address, wallet.to_ascii_lowercase());
        assert_eq!(parsed.title, "Hello admin");
        assert_eq!(parsed.message, "Queued message");

        for invalid in [
            format!("{body}&message=duplicate"),
            format!("{body}&broadcast=true"),
            format!("recipient_wallet_address=0x123&title=x&message=y"),
            format!("recipient_wallet_address={wallet}&title=&message=y"),
        ] {
            assert!(
                parse_admin_notification_form(invalid.as_bytes()).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn notification_form_requires_same_origin_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("admin.test"));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://admin.test"),
        );
        assert!(same_origin_admin_notification_form(&headers));

        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://evil.test"),
        );
        assert!(!same_origin_admin_notification_form(&headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://admin.test"),
        );
        headers.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
        assert!(!same_origin_admin_notification_form(&headers));
    }

    #[test]
    fn notification_form_redirect_is_cookie_paired_and_closed() {
        let accepted = admin_notification_form_redirect("accepted");
        assert_eq!(accepted.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            accepted.headers()[header::LOCATION],
            "/notifications/manage?send=accepted"
        );
        assert!(accepted.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("epsx.admin.notification_send=accepted"));

        let invalid = admin_notification_form_redirect("unexpected");
        assert_eq!(
            invalid.headers()[header::LOCATION],
            "/notifications/manage?send=error"
        );
        assert!(invalid.headers()[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .contains("epsx.admin.notification_send=error"));
    }

    #[tokio::test]
    async fn admin_emits_one_private_recovery_bootstrap_only_with_refresh_cookie() {
        let response = request_with_cookie(
            Method::GET,
            "/analytics",
            Some("epsx.admin.refresh_token=opaque-refresh"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(response.headers()[header::VARY], "Cookie, Authorization");
        let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body);
        assert_eq!(html.matches("data-epsx-session-recovery").count(), 1);
        assert_eq!(html.matches("window.epsxAuth.recover()").count(), 1);
        assert!(!html.contains("opaque-refresh"));
        let bridge_position = html
            .find("window.epsxAuth =")
            .expect("the shared auth bridge must be present");
        let recovery_position = html
            .find("data-epsx-session-recovery")
            .expect("the recovery bootstrap must be present");
        assert!(
            bridge_position < recovery_position,
            "the shared bridge must be defined before recovery runs"
        );

        let wrong_client = request_with_cookie(
            Method::GET,
            "/analytics",
            Some("epsx.frontend.refresh_token=wrong-client"),
        )
        .await;
        let wrong_client_html = to_bytes(wrong_client.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        assert!(!String::from_utf8_lossy(&wrong_client_html).contains("data-epsx-session-recovery"));

        let rejected = request_with_cookie(
            Method::GET,
            "/analytics",
            Some("epsx.admin.access_token=malformed; epsx.admin.refresh_token=opaque-refresh"),
        )
        .await;
        let rejected_html = String::from_utf8_lossy(
            &to_bytes(rejected.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap(),
        )
        .into_owned();
        assert_eq!(
            rejected_html.matches("data-epsx-session-recovery").count(),
            1
        );

        let verifier_outage = request_with_cookie(
            Method::GET,
            "/analytics",
            Some("epsx.admin.access_token=eyJhbGciOiJSUzI1NiIsImtpZCI6Im91dGFnZSIsInR5cCI6IkpXVCJ9.e30.c2ln; epsx.admin.refresh_token=opaque-refresh"),
        )
        .await;
        let outage_html = to_bytes(verifier_outage.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        assert!(!String::from_utf8_lossy(&outage_html).contains("data-epsx-session-recovery"));

        let no_refresh = request(Method::GET, "/analytics").await;
        let no_refresh_html = to_bytes(no_refresh.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        assert!(!String::from_utf8_lossy(&no_refresh_html).contains("data-epsx-session-recovery"));
    }

    #[test]
    fn protected_api_predicate_covers_every_registered_shape() {
        for path in [
            "/api/v1/users",
            "/api/v1/users/id",
            "/api/v1/payments",
            "/api/v1/subscriptions",
            "/api/v1/subscriptions/id",
            "/api/v1/subscriptions/id/cancel",
            "/api/v1/subscription/plans",
            "/api/v1/subscription/plans/id",
            "/api/v1/pages",
            "/api/v1/pages/slug",
            "/api/v1/pages/slug/publish",
            "/api/v1/themes",
            "/api/v1/themes/id",
            "/api/v1/notifications",
            "/api/v1/notifications/id",
            "/api/v1/notifications/id/read",
            "/api/v1/notifications/templates",
            "/api/v1/notifications/templates/id",
            "/api/v1/notifications/templates/id/preview",
            "/api/v1/notifications/templates/id/rollback",
            "/api/v1/notifications/templates/id/audit",
            "/api/v1/notifications/metrics",
            "/api/v1/notifications/send",
            "/api/v1/analytics/events",
            "/api/v1/analytics/metrics/usage",
            "/api/v1/analytics/revenue",
            "/api/v1/analytics/track",
            "/api/v1/indexer/status/bsc",
            "/api/v1/indexer/block/bsc/1",
            "/api/v1/indexer/tx/bsc/hash",
            "/api/v1/indexer/transfers/bsc/address",
            "/api/v1/wallet/accounts",
            "/api/v1/wallet/accounts/address",
        ] {
            assert!(is_known_protected_admin_api_path(path), "{path}");
        }

        for path in [
            "/api/v1/users/id/extra",
            "/api/v1/auth/unknown",
            "/api/v1/indexer/block/bsc",
            "/api/v1/notifications/templates/id/extra",
            "/api/v1/payments/id",
            "/api/v1/payments/id/confirm",
            "/api/v1/payments/id/cancel",
            "/api/v1/escrows",
            "/api/v1/escrows/id/release",
        ] {
            assert!(!is_known_admin_api_path(path), "{path}");
        }
    }

    #[tokio::test]
    async fn ssr_unknown_and_malformed_routes_bypass_unauth_skeleton() {
        assert_eq!(request(Method::GET, "/").await.status(), StatusCode::OK);

        for path in [
            "/missing-page",
            "/chat/",
            "/chat/id/extra",
            "/news//edit",
            "/news/id/edit/extra",
            "/wallet-management/address/extra",
            "/wallet-management/access/plans/id/extra",
            "/wallet-management/wallets/address/disable/extra",
        ] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "text/html; charset=utf-8",
                "{path}"
            );
            let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            let html = String::from_utf8_lossy(&body);
            assert!(html.contains("Page not found"), "{path}");
            assert!(!html.contains("admin-skeleton"), "{path}");
        }

        assert_eq!(
            request(Method::HEAD, "/missing-page").await.status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn unknown_api_is_json_before_auth_and_known_route_remains_protected() {
        for path in [
            "/api",
            "/api/",
            "/api/v1/users/id/extra",
            "/api/v1/auth/unknown",
            "/api/v1/payments/id",
            "/api/v1/payments/id/confirm",
            "/api/v1/payments/id/cancel",
            "/api/v1/escrows",
            "/api/v1/escrows/id/release",
        ] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                "application/json",
                "{path}"
            );
            let body: serde_json::Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap())
                    .unwrap();
            assert_eq!(body["error"], "not_found", "{path}");
        }

        let head = request(Method::HEAD, "/api/v1/users/id/extra").await;
        assert_eq!(head.status(), StatusCode::NOT_FOUND);
        assert_eq!(head.headers()[header::CONTENT_TYPE], "application/json");

        assert_eq!(
            request(Method::GET, "/api/v1/users").await.status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(Method::HEAD, "/api/v1/users").await.status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(Method::POST, "/api/v1/users/id").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
        assert_eq!(
            request(Method::GET, "/api/v1/auth/challenge")
                .await
                .status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn admin_redirect_targets_are_fixed_but_transport_drift_remains_visible() {
        for uri in [
            "/auth",
            "/auth?next=https%3A%2F%2Fevil.example%2Fsteal",
            "/auth?return_url=%2F%2Fevil.example%2Fsteal",
            "/auth?return_url=%2Fwallet-management%2Fwallets&next=%2Fanalytics",
        ] {
            let auth = request(Method::GET, uri).await;
            assert_eq!(auth.status(), StatusCode::TEMPORARY_REDIRECT, "{uri}");
            assert_eq!(auth.headers()[header::LOCATION], "/", "{uri}");
            let body = to_bytes(auth.into_body(), 16 * 1024).await.unwrap();
            let body = String::from_utf8_lossy(&body);
            assert!(!body.contains("evil.example"), "{uri}");
            assert!(!body.contains("wallet-management"), "{uri}");
            assert!(!body.contains("analytics"), "{uri}");
        }

        let auth_head = request(Method::HEAD, "/auth?return_url=https%3A%2F%2Fevil.example").await;
        assert_eq!(auth_head.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(auth_head.headers()[header::LOCATION], "/");
        assert!(to_bytes(auth_head.into_body(), 16 * 1024)
            .await
            .unwrap()
            .is_empty());

        let auth_post = request(Method::POST, "/auth?return_url=%2Fanalytics").await;
        assert_eq!(auth_post.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert!(auth_post.headers().get(header::LOCATION).is_none());

        for uri in [
            "/wallet-management",
            "/wallet-management?next=https%3A%2F%2Fevil.example%2Fsteal",
        ] {
            let wallet = request(Method::GET, uri).await;
            assert_eq!(wallet.status(), StatusCode::PERMANENT_REDIRECT, "{uri}");
            assert_eq!(
                wallet.headers()[header::LOCATION],
                "/wallet-management/wallets",
                "{uri}"
            );
        }

        for uri in [
            "/notifications",
            "/notifications?next=https%3A%2F%2Fevil.example%2Fsteal",
        ] {
            let notifications = request(Method::GET, uri).await;
            assert_eq!(notifications.status(), StatusCode::OK, "{uri}");
            let body = to_bytes(notifications.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            let html = String::from_utf8_lossy(&body);
            assert!(
                html.contains("window.location.replace('/notifications/manage');"),
                "{uri}"
            );
            assert!(!html.contains("evil.example"), "{uri}");
        }

        // The notification composer is a real page whose form posts back to
        // the same path. A GET must therefore reach SSR rather than the
        // POST-only form handler returning 405.
        let create = request(Method::GET, "/notifications/create").await;
        assert_eq!(create.status(), StatusCode::OK);
        let body = to_bytes(create.into_body(), 2 * 1024 * 1024).await.unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(html.contains("Sign in required"));
    }
}

async fn api_health() -> &'static str {
    "ok"
}

async fn admin_auth_redirect() -> Response {
    Redirect::temporary("/").into_response()
}

// Wave 43 T1 A3 — mirror prod's Vercel middleware:
// `/wallet-management` → `/wallet-management/wallets` (308 Permanent).
// Prod uses Vercel middleware that fires before the page handler;
// dev's `Router::fallback(ssr_handler)` would otherwise render
// `wallet_redirect::render()` which is NOT what prod shows for the
// bare `/wallet-management` URL (prod redirects to /wallets).
async fn wallet_redirect_to_wallets() -> Response {
    Redirect::permanent("/wallet-management/wallets").into_response()
}

// ===== Users =====
async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .identity
        .get_with_ctx("/api/v1/identity/users", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .identity
        .post_with_ctx("/api/v1/identity/users", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/identity/users/{}", id);
    state
        .identity
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/identity/users/{}", id);
    state
        .identity
        .put_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/identity/users/{}", id);
    state
        .identity
        .delete_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Payments =====
async fn list_payments(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let query = PaymentIntentQuery::from_raw(raw_query.as_deref().unwrap_or_default())
        .map_err(|()| StatusCode::BAD_REQUEST)?;
    let value = state
        .payment
        .get_with_ctx(&query.upstream_path(), &ctx)
        .await
        .map_err(err_to_status)?;
    let payload = decode_admin_payment_intent_list(value).ok_or(StatusCode::BAD_GATEWAY)?;
    Ok(Json(payload).into_response())
}

// ===== Subscriptions =====
async fn list_subscriptions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .subscription
        .get_with_ctx("/api/v1/subscription/subscriptions", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/subscription/subscriptions/{}", id);
    state
        .subscription
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn cancel_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/subscription/subscriptions/{}/cancel", id);
    state
        .subscription
        .post_with_ctx(&path, &serde_json::json!({}), &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_plans(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .subscription
        .get_with_ctx("/api/v1/subscription/plans", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .subscription
        .post_with_ctx("/api/v1/subscription/plans", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/subscription/plans/{}", id);
    state
        .subscription
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Content =====
async fn list_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .get_with_ctx("/api/v1/content/pages", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .post_with_ctx("/api/v1/content/pages", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/pages/{}", slug);
    state
        .content
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn update_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(slug): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/pages/{}", slug);
    state
        .content
        .put_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn publish_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/pages/{}/publish", slug);
    state
        .content
        .post_with_ctx(&path, &serde_json::json!({}), &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_themes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .get_with_ctx("/api/v1/content/themes", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .content
        .post_with_ctx("/api/v1/content/themes", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn get_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/themes/{}", id);
    state
        .content
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn update_theme(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/content/themes/{}", id);
    state
        .content
        .put_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_blocks(State(state): State<AppState>) -> Result<Response, StatusCode> {
    state
        .content
        .get_plain("/api/v1/content/blocks")
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn content_navigation(State(state): State<AppState>) -> Result<Response, StatusCode> {
    state
        .content
        .get_plain("/api/v1/content/navigation")
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn content_site(State(state): State<AppState>) -> Result<Response, StatusCode> {
    state
        .content
        .get_plain("/api/v1/content/site")
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Notifications =====
async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .get_with_ctx("/api/v1/notification/list", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn mark_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/{}/read", id);
    state
        .notification
        .post_with_ctx(&path, &serde_json::json!({}), &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn delete_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/{}", id);
    state
        .notification
        .delete_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .get_with_ctx("/api/v1/notification/templates", &ctx)
        .await
        .and_then(|value| {
            let payload = serde_json::from_value::<BackendTemplateList>(value)
                .map_err(|_| epsx_client::ClientError::Service("malformed template list".into()))?;
            if !valid_template_list_payload(&payload) {
                return Err(epsx_client::ClientError::Service(
                    "malformed template list".into(),
                ));
            }
            let encoded = serde_json::to_vec(&payload)
                .map_err(|_| epsx_client::ClientError::Service("malformed template list".into()))?;
            if encoded.len() > 512 * 1024 {
                return Err(epsx_client::ClientError::Service(
                    "template list response too large".into(),
                ));
            }
            Ok(serde_json::to_value(payload).expect("validated template list is serializable"))
        })
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .post_with_ctx("/api/v1/notification/templates", &body, &ctx)
        .await
        .map(|v| (StatusCode::CREATED, Json(v)).into_response())
        .map_err(err_to_status)
}

async fn delete_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/templates/{}", id);
    state
        .notification
        .delete_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn preview_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/templates/{id}/preview");
    state
        .notification
        .post_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn rollback_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/templates/{id}/rollback");
    state
        .notification
        .post_with_ctx(&path, &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

const TEMPLATE_AUDIT_MAX_ITEMS: usize = 100;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendTemplateAuditEnvelope {
    items: Vec<BackendTemplateAuditEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendTemplateAuditEntry {
    id: String,
    template_id: String,
    action: String,
    from_version: Option<i32>,
    to_version: Option<i32>,
    actor_subject: String,
    metadata: serde_json::Value,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendTemplateList {
    items: Vec<BackendTemplate>,
    total: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendTemplate {
    id: String,
    name: String,
    channel: String,
    subject: Option<String>,
    body: String,
    variables: serde_json::Value,
    active: bool,
    created_at: String,
    updated_at: String,
}

fn valid_template_list_text(value: &str, min_chars: usize, max_chars: usize) -> bool {
    let count = value.chars().count();
    (min_chars..=max_chars).contains(&count) && !value.chars().any(char::is_control)
}

fn valid_template_list_variables(value: &serde_json::Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.len() <= 128
        && serde_json::to_vec(value).is_ok_and(|encoded| encoded.len() <= 64 * 1024)
        && object.iter().all(|(name, definition)| {
            (1..=64).contains(&name.len())
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
                && definition.as_object().is_some_and(|definition| {
                    definition.len() <= 3
                        && definition
                            .get("type")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|kind| {
                                matches!(
                                    kind,
                                    "string"
                                        | "number"
                                        | "integer"
                                        | "boolean"
                                        | "object"
                                        | "array"
                                )
                            })
                        && definition
                            .get("required")
                            .is_none_or(serde_json::Value::is_boolean)
                        && definition
                            .get("description")
                            .and_then(serde_json::Value::as_str)
                            .is_none_or(|description| valid_template_list_text(description, 0, 512))
                        && definition
                            .keys()
                            .all(|key| matches!(key.as_str(), "type" | "required" | "description"))
                })
        })
}

fn valid_template_list_rfc3339(value: &str) -> bool {
    value.len() <= 64
        && !value.is_empty()
        && !value.chars().any(char::is_control)
        && chrono::DateTime::parse_from_rfc3339(value).is_ok()
}

fn valid_template_list_payload(payload: &BackendTemplateList) -> bool {
    payload.total >= 0
        && payload.items.len() <= 128
        && usize::try_from(payload.total)
            .ok()
            .is_some_and(|total| total >= payload.items.len())
        && payload.items.iter().all(|template| {
            valid_template_list_text(&template.id, 1, 66)
                && valid_template_list_text(&template.name, 1, 100)
                && matches!(template.channel.as_str(), "email" | "in_app" | "push")
                && template
                    .subject
                    .as_deref()
                    .is_none_or(|subject| valid_template_list_text(subject, 0, 255))
                && valid_template_list_text(&template.body, 1, 64 * 1024)
                && valid_template_list_variables(&template.variables)
                && valid_template_list_rfc3339(&template.created_at)
                && valid_template_list_rfc3339(&template.updated_at)
        })
}

fn valid_template_audit_text(value: &str, max_chars: usize) -> bool {
    !value.is_empty() && value.chars().count() <= max_chars && !value.chars().any(char::is_control)
}

fn valid_template_audit_metadata(action: &str, metadata: &serde_json::Value) -> bool {
    let Some(object) = metadata.as_object() else {
        return false;
    };
    match action {
        "created" | "updated" | "deleted" => {
            object.len() == 1
                && object
                    .get("template_name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|name| valid_template_audit_text(name, 100))
        }
        "rollback" => {
            object.len() == 2
                && object
                    .get("restored_version")
                    .and_then(serde_json::Value::as_i64)
                    .is_some_and(|version| version > 0)
                && object
                    .get("new_version")
                    .and_then(serde_json::Value::as_i64)
                    .is_some_and(|version| version > 0)
        }
        _ => false,
    }
}

fn valid_template_audit_payload(payload: &BackendTemplateAuditEnvelope) -> bool {
    payload.items.len() <= TEMPLATE_AUDIT_MAX_ITEMS
        && payload.items.iter().all(|entry| {
            valid_template_audit_text(&entry.id, 128)
                && valid_template_audit_text(&entry.template_id, 66)
                && matches!(
                    entry.action.as_str(),
                    "created" | "updated" | "deleted" | "rollback"
                )
                && entry.from_version.is_none_or(|version| version > 0)
                && entry.to_version.is_none_or(|version| version > 0)
                && valid_template_audit_text(&entry.actor_subject, 255)
                && valid_template_audit_metadata(&entry.action, &entry.metadata)
                && serde_json::to_vec(&entry.metadata).is_ok_and(|bytes| bytes.len() <= 16 * 1024)
                && chrono::DateTime::parse_from_rfc3339(&entry.created_at).is_ok()
                && entry.created_at.len() <= 64
        })
}

async fn template_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/notification/templates/{id}/audit");
    state
        .notification
        .get_with_ctx(&path, &ctx)
        .await
        .and_then(|value| {
            let payload = serde_json::from_value::<BackendTemplateAuditEnvelope>(value)?;
            if !valid_template_audit_payload(&payload) {
                return Err(epsx_client::ClientError::Service(
                    "malformed template audit response".into(),
                ));
            }
            Ok(serde_json::to_value(payload)?)
        })
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BackendNotificationMetrics {
    queue_depth: i64,
    queue_age_seconds: Option<i64>,
    suppressed: i64,
    retry_wait: i64,
    terminal_failed: i64,
    dead_lettered: i64,
    provider_accepted: i64,
    attempting: i64,
    channel_outcomes: BTreeMap<String, i64>,
    provider_events: i64,
    delivery_attempts: i64,
    replay_cursors: i64,
    replay_cursor_age_seconds: Option<i64>,
    active_streams: usize,
    stream_connections_total: u64,
    stream_reconnects_total: u64,
    stream_replayed_events_total: u64,
    stream_lag_seconds: Option<u64>,
    stream_query_failures_total: u64,
}

fn valid_notification_metrics(payload: &BackendNotificationMetrics) -> bool {
    let non_negative = |value: i64| value >= 0;
    non_negative(payload.queue_depth)
        && payload.queue_age_seconds.is_none_or(non_negative)
        && non_negative(payload.suppressed)
        && non_negative(payload.retry_wait)
        && non_negative(payload.terminal_failed)
        && non_negative(payload.dead_lettered)
        && non_negative(payload.provider_accepted)
        && non_negative(payload.attempting)
        && non_negative(payload.provider_events)
        && non_negative(payload.delivery_attempts)
        && non_negative(payload.replay_cursors)
        && payload.replay_cursor_age_seconds.is_none_or(non_negative)
        && payload.active_streams <= 256
        && payload.channel_outcomes.iter().all(|(channel, count)| {
            matches!(channel.as_str(), "email" | "in_app" | "push") && *count >= 0
        })
}

async fn notification_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .get_with_ctx("/api/v1/notification/admin/metrics", &ctx)
        .await
        .and_then(|value| {
            let payload = serde_json::from_value::<BackendNotificationMetrics>(value)?;
            if !valid_notification_metrics(&payload) {
                return Err(epsx_client::ClientError::Service(
                    "malformed notification metrics response".into(),
                ));
            }
            Ok(serde_json::to_value(payload)?)
        })
        .map(|value| Json(value).into_response())
        .map_err(err_to_status)
}

async fn send_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .notification
        .post_with_ctx_status("/api/v1/notification/send", &body, &ctx)
        .await
        .map(|(status, v)| (status, Json(v)).into_response())
        .map_err(err_to_status)
}

/// Accept the small, source-compatible notification compose surface rendered
/// by the Dioxus admin page. Authentication and authorization remain owned by
/// the verified session and notification service; this handler only translates
/// the bounded browser form into the service's canonical in-app request.
async fn submit_notification_form(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if !same_origin_admin_notification_form(&parts.headers) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "notification_origin_rejected"
            })),
        )
            .into_response();
    }

    let Some((token, _user)) = auth::verified_access_token(
        &parts.headers,
        state.verifier.as_ref(),
        state.cookie_environment,
    )
    .await
    else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !content_type.split(';').next().is_some_and(|value| {
        value
            .trim()
            .eq_ignore_ascii_case("application/x-www-form-urlencoded")
    }) {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }

    let body = match to_bytes(body, ADMIN_NOTIFICATION_FORM_MAX).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let form = match parse_admin_notification_form(&body) {
        Ok(form) => form,
        Err(()) => return admin_notification_form_redirect("error"),
    };

    let mut request_context = RequestContext::from_headers(&parts.headers);
    request_context.auth_token = Some(token);
    let payload = serde_json::json!({
        "user_id": form.recipient_wallet_address,
        "channel": "in_app",
        "recipient": form.recipient_wallet_address,
        "template_id": null,
        "subject": form.title,
        "body": form.message,
        "data": null,
        "expires_at": null,
    });
    match state
        .notification
        .post_with_ctx_status("/api/v1/notification/send", &payload, &request_context)
        .await
    {
        Ok((status, _)) if status.is_success() => admin_notification_form_redirect("accepted"),
        _ => admin_notification_form_redirect("error"),
    }
}

// ===== Analytics =====
async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .analytics
        .get_with_ctx("/api/v1/analytics/events", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(metric): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/analytics/metrics/{}", metric);
    state
        .analytics
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn revenue(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .analytics
        .get_with_ctx("/api/v1/analytics/revenue", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn track_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .analytics
        .post_with_ctx("/api/v1/analytics/track", &body, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Indexer =====
async fn chain_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(chain): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/status/{}", chain);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_block(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath((chain, number)): AxPath<(String, String)>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/block/{}/{}", chain, number);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_tx(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath((chain, hash)): AxPath<(String, String)>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/tx/{}/{}", chain, hash);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_transfers(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath((chain, address)): AxPath<(String, String)>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/indexer/transfers/{}/{}", chain, address);
    state
        .indexer
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

// ===== Wallet =====
async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    state
        .wallet
        .get_with_ctx("/api/v1/wallet/accounts", &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}

async fn get_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxPath(address): AxPath<String>,
) -> Result<Response, StatusCode> {
    let ctx = ctx_from(&headers);
    let path = format!("/api/v1/wallet/accounts/{}", address);
    state
        .wallet
        .get_with_ctx(&path, &ctx)
        .await
        .map(|v| Json(v).into_response())
        .map_err(err_to_status)
}
