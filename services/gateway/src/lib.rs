pub mod auth;
pub mod policy;

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, VerifiedPrincipal, ADMIN_AUDIENCE,
};
use policy::{classify, AccessPolicy};
use serde_json::json;
use std::{net::IpAddr, sync::Arc, time::Duration};
use thiserror::Error;
use tower_http::trace::TraceLayer;

pub const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;
const MAX_REQUEST_ID_BYTES: usize = 128;
const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Clone)]
pub struct GatewayUrls {
    pub identity: String,
    pub wallet: String,
    pub payment: String,
    pub subscription: String,
    pub content: String,
    pub notification: String,
    pub analytics: String,
    pub indexer: String,
}

#[derive(Debug, Error)]
pub enum GatewayConfigError {
    #[error("invalid upstream URL for {0}")]
    InvalidUpstream(&'static str),
}

impl GatewayUrls {
    fn validate(mut self, production: bool) -> Result<Self, GatewayConfigError> {
        self.identity = validate_upstream("identity", &self.identity, production)?;
        self.wallet = validate_upstream("wallet", &self.wallet, production)?;
        self.payment = validate_upstream("payment", &self.payment, production)?;
        self.subscription = validate_upstream("subscription", &self.subscription, production)?;
        self.content = validate_upstream("content", &self.content, production)?;
        self.notification = validate_upstream("notification", &self.notification, production)?;
        self.analytics = validate_upstream("analytics", &self.analytics, production)?;
        self.indexer = validate_upstream("indexer", &self.indexer, production)?;
        Ok(self)
    }
}

fn validate_upstream(
    name: &'static str,
    raw: &str,
    production: bool,
) -> Result<String, GatewayConfigError> {
    if raw.trim() != raw {
        return Err(GatewayConfigError::InvalidUpstream(name));
    }
    let url = reqwest::Url::parse(raw).map_err(|_| GatewayConfigError::InvalidUpstream(name))?;
    let host = url
        .host_str()
        .ok_or(GatewayConfigError::InvalidUpstream(name))?;
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host.to_ascii_lowercase().ends_with(".localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || raw.contains("/./")
        || raw.contains("/../")
        || (production && loopback)
    {
        return Err(GatewayConfigError::InvalidUpstream(name));
    }
    Ok(url.origin().ascii_serialization())
}

#[derive(Clone)]
pub struct AppState {
    urls: GatewayUrls,
    client: reqwest::Client,
    verifier: Arc<dyn AccessTokenVerifier>,
}

impl AppState {
    pub fn new(
        urls: GatewayUrls,
        client: reqwest::Client,
        verifier: Arc<dyn AccessTokenVerifier>,
        production: bool,
    ) -> Result<Self, GatewayConfigError> {
        Ok(Self {
            urls: urls.validate(production)?,
            client,
            verifier,
        })
    }
}

pub fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-gateway/1")
        .build()
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/identity/{*path}", any(proxy_identity))
        .route("/api/v1/admin/wallets/{*path}", any(proxy_wallet))
        .route("/api/v1/admin/credits/{*path}", any(proxy_wallet))
        .route("/api/v1/admin/pay/{*path}", any(proxy_payment))
        .route(
            "/api/v1/admin/subscription/{*path}",
            any(proxy_subscription),
        )
        .route("/api/v1/wallet/{*path}", any(proxy_wallet))
        .route("/api/v1/payment/{*path}", any(proxy_payment))
        .route("/api/v1/pay/{*path}", any(proxy_payment))
        .route("/api/v1/subscription/{*path}", any(proxy_subscription))
        .route("/api/v1/content/{*path}", any(proxy_content))
        .route("/api/v1/news", any(proxy_news))
        .route("/api/v1/news/{*path}", any(proxy_news))
        .route("/api/v1/portfolio/{*path}", any(proxy_portfolio))
        .route("/api/v1/plans", any(proxy_plans))
        .route("/api/v1/plans/{*path}", any(proxy_plans))
        .route("/api/v1/rankings", any(proxy_rankings))
        .route("/api/v1/rankings/{*path}", any(proxy_rankings))
        .route("/api/v1/notification/{*path}", any(proxy_notification))
        .route("/api/v1/analytics/{*path}", any(proxy_analytics))
        .route("/api/v1/indexer/{*path}", any(proxy_indexer))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

macro_rules! proxy_fn {
    ($name:ident, $field:ident) => {
        async fn $name(State(state): State<AppState>, req: Request) -> Response {
            let base_url = state.urls.$field.clone();
            proxy_to_service(state, &base_url, req, None).await
        }
    };
}

macro_rules! proxy_rewrite_fn {
    ($name:ident, $state:ident, $field:ident, $from:literal, $to:literal) => {
        async fn $name(State($state): State<AppState>, req: Request) -> Response {
            let base_url = $state.urls.$field.clone();
            proxy_to_service($state, &base_url, req, Some(($from, $to))).await
        }
    };
}

proxy_fn!(proxy_identity, identity);
proxy_fn!(proxy_wallet, wallet);
proxy_fn!(proxy_payment, payment);
proxy_fn!(proxy_subscription, subscription);
proxy_fn!(proxy_content, content);
proxy_rewrite_fn!(
    proxy_news,
    state,
    content,
    "/api/v1/news",
    "/api/v1/content/news"
);
proxy_rewrite_fn!(
    proxy_portfolio,
    state,
    content,
    "/api/v1/portfolio",
    "/api/v1/content/portfolio"
);
proxy_rewrite_fn!(
    proxy_plans,
    state,
    content,
    "/api/v1/plans",
    "/api/v1/content/plans"
);
proxy_rewrite_fn!(
    proxy_rankings,
    state,
    content,
    "/api/v1/rankings",
    "/api/v1/content/rankings"
);
proxy_fn!(proxy_notification, notification);
proxy_fn!(proxy_analytics, analytics);
proxy_fn!(proxy_indexer, indexer);

#[derive(Debug)]
enum GatewayError {
    Unauthorized,
    Forbidden,
    NotFound,
    PayloadTooLarge,
    BadGateway,
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large"),
            Self::BadGateway => (StatusCode::BAD_GATEWAY, "bad_gateway"),
        };
        (status, Json(json!({ "error": code }))).into_response()
    }
}

async fn authorize_request(
    verifier: &dyn AccessTokenVerifier,
    method: &Method,
    path: &str,
    headers: &mut HeaderMap,
) -> Result<Option<VerifiedPrincipal>, GatewayError> {
    strip_spoofable_identity_headers(headers);
    match classify(method, path) {
        AccessPolicy::Public | AccessPolicy::CredentialExchange => {
            // Anonymous/credential-exchange endpoints never forward a caller's
            // access token, valid or invalid, to avoid a confused deputy.
            headers.remove(header::AUTHORIZATION);
            Ok(None)
        }
        AccessPolicy::Authenticated => authenticate_headers(verifier, headers)
            .await
            .map(Some)
            .map_err(|_| GatewayError::Unauthorized),
        AccessPolicy::Permission(permission) => {
            let principal = authenticate_headers(verifier, headers)
                .await
                .map_err(|_| GatewayError::Unauthorized)?;
            if principal.audience != ADMIN_AUDIENCE || !principal.has_permission(permission) {
                return Err(GatewayError::Forbidden);
            }
            Ok(Some(principal))
        }
        AccessPolicy::AudiencePermission {
            audience,
            permission,
        } => {
            let principal = authenticate_headers(verifier, headers)
                .await
                .map_err(|_| GatewayError::Unauthorized)?;
            if principal.audience != audience || !principal.has_permission(permission) {
                return Err(GatewayError::Forbidden);
            }
            Ok(Some(principal))
        }
        AccessPolicy::InternalOnly | AccessPolicy::Blocked => Err(GatewayError::NotFound),
    }
}

async fn proxy_to_service(
    state: AppState,
    base_url: &str,
    mut req: Request,
    rewrite: Option<(&str, &str)>,
) -> Response {
    let method = req.method().clone();
    let raw_path = req.uri().path().to_string();
    if let Err(error) = authorize_request(
        state.verifier.as_ref(),
        &method,
        &raw_path,
        req.headers_mut(),
    )
    .await
    {
        return error.into_response();
    }

    if content_length_exceeds(req.headers(), MAX_REQUEST_BODY_BYTES) {
        return GatewayError::PayloadTooLarge.into_response();
    }

    let path = match rewrite {
        Some((from, to)) if raw_path == from || raw_path.starts_with(&format!("{from}/")) => {
            format!("{to}{}", &raw_path[from.len()..])
        }
        _ => raw_path,
    };
    let query = req
        .uri()
        .query()
        .map(|query| format!("?{query}"))
        .unwrap_or_default();
    let url = format!("{}{}{}", base_url.trim_end_matches('/'), path, query);

    let mut headers = std::mem::take(req.headers_mut());
    sanitize_proxy_headers(&mut headers);
    ensure_request_id(&mut headers);
    let request_id = headers
        .get(&REQUEST_ID)
        .cloned()
        .expect("request id is inserted before proxying");
    let body = match to_bytes(req.into_body(), MAX_REQUEST_BODY_BYTES).await {
        Ok(body) => body,
        Err(_) => return GatewayError::PayloadTooLarge.into_response(),
    };

    let mut response = match state
        .client
        .request(method, &url)
        .headers(headers)
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            let error_class = if error.is_timeout() {
                "timeout"
            } else if error.is_connect() {
                "connect"
            } else if error.is_request() {
                "request"
            } else {
                "other"
            };
            tracing::warn!(error_class, "gateway upstream request failed");
            return GatewayError::BadGateway.into_response();
        }
    };

    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BODY_BYTES as u64)
    {
        return GatewayError::BadGateway.into_response();
    }
    let status = response.status();
    let mut response_headers = response.headers().clone();
    sanitize_proxy_headers(&mut response_headers);
    // Candidate services are bearer-token APIs. Do not let an upstream
    // service create a browser cookie outside the frontend/admin BFF policy.
    response_headers.remove(header::SET_COOKIE);
    response_headers.insert(REQUEST_ID.clone(), request_id);
    let mut body = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk))
                if body.len().saturating_add(chunk.len()) <= MAX_RESPONSE_BODY_BYTES =>
            {
                body.extend_from_slice(&chunk);
            }
            Ok(Some(_)) | Err(_) => return GatewayError::BadGateway.into_response(),
            Ok(None) => break,
        }
    }

    let mut response = Response::builder().status(status);
    if let Some(headers) = response.headers_mut() {
        *headers = response_headers;
    } else {
        return GatewayError::BadGateway.into_response();
    }
    response
        .body(Body::from(body))
        .unwrap_or_else(|_| GatewayError::BadGateway.into_response())
}

fn content_length_exceeds(headers: &HeaderMap, limit: usize) -> bool {
    headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > limit as u64)
}

fn strip_spoofable_identity_headers(headers: &mut HeaderMap) {
    let names: Vec<_> = headers
        .keys()
        .filter(|name| {
            let name = name.as_str();
            name.starts_with("x-user-")
                || name.starts_with("x-wallet-")
                || name.starts_with("x-auth-")
                || name.starts_with("x-epsx-")
                || matches!(
                    name,
                    "x-user"
                        | "x-subject"
                        | "x-principal"
                        | "x-wallet"
                        | "x-address"
                        | "x-chain-id"
                        | "x-client-id"
                        | "x-permissions"
                        | "x-role"
                        | "x-roles"
                        | "x-scope"
                        | "x-forwarded-user"
                )
        })
        .cloned()
        .collect();
    for name in names {
        headers.remove(name);
    }
}

fn sanitize_proxy_headers(headers: &mut HeaderMap) {
    let connection_named: Vec<HeaderName> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .filter_map(|name| HeaderName::from_bytes(name.trim().as_bytes()).ok())
        .collect();
    for name in connection_named {
        headers.remove(name);
    }

    for name in [
        header::CONNECTION,
        header::CONTENT_LENGTH,
        header::HOST,
        header::PROXY_AUTHENTICATE,
        header::PROXY_AUTHORIZATION,
        header::TE,
        header::TRAILER,
        header::TRANSFER_ENCODING,
        header::UPGRADE,
        header::COOKIE,
        HeaderName::from_static("keep-alive"),
        HeaderName::from_static("forwarded"),
        HeaderName::from_static("x-forwarded-for"),
        HeaderName::from_static("x-forwarded-host"),
        HeaderName::from_static("x-forwarded-proto"),
        HeaderName::from_static("x-real-ip"),
    ] {
        headers.remove(name);
    }
}

fn ensure_request_id(headers: &mut HeaderMap) {
    let valid = headers
        .get(&REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= MAX_REQUEST_ID_BYTES
                && value.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
                })
        })
        .map(str::to_string);
    let request_id = valid.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    headers.insert(
        REQUEST_ID.clone(),
        HeaderValue::from_str(&request_id).expect("validated request id must be a header value"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::routing::any;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    #[derive(Debug, Clone)]
    struct CapturedRequest {
        uri: String,
        path: String,
        headers: HeaderMap,
        body: Vec<u8>,
    }

    #[derive(Default)]
    struct Capture {
        hits: AtomicUsize,
        requests: Mutex<Vec<CapturedRequest>>,
    }

    async fn capture_upstream(State(capture): State<Arc<Capture>>, req: Request) -> Response {
        capture.hits.fetch_add(1, Ordering::SeqCst);
        let uri = req.uri().to_string();
        let path = req.uri().path().to_string();
        let oversized_response = path.ends_with("/large-response");
        let headers = req.headers().clone();
        let body = to_bytes(req.into_body(), MAX_REQUEST_BODY_BYTES)
            .await
            .unwrap()
            .to_vec();
        capture.requests.lock().await.push(CapturedRequest {
            uri,
            path,
            headers,
            body,
        });
        if oversized_response {
            return Body::from(vec![0_u8; MAX_RESPONSE_BODY_BYTES + 1]).into_response();
        }
        (
            StatusCode::NO_CONTENT,
            [(header::SET_COOKIE, "legacy_service_session=unsafe")],
        )
            .into_response()
    }

    struct FakeVerifier {
        calls: AtomicUsize,
    }

    impl FakeVerifier {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(
            &self,
            token: &str,
        ) -> Result<VerifiedPrincipal, epsx_service_auth::VerifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (audience, permissions) = match token {
                "front" => (epsx_service_auth::FRONTEND_AUDIENCE, vec![]),
                "admin-no-scope" => (ADMIN_AUDIENCE, vec![]),
                "admin-users" => (ADMIN_AUDIENCE, vec!["admin:users:read".into()]),
                "admin-audit" => (ADMIN_AUDIENCE, vec!["admin:audit:read".into()]),
                "admin-global" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "publisher" => (
                    policy::NOTIFICATION_PUBLISHER_AUDIENCE,
                    vec![policy::NOTIFICATION_PUBLISH_PERMISSION.into()],
                ),
                "provider" => (
                    policy::NOTIFICATION_PROVIDER_AUDIENCE,
                    vec![policy::NOTIFICATION_PROVIDER_EVENTS_PERMISSION.into()],
                ),
                "provider-wrong-permission" => (
                    policy::NOTIFICATION_PROVIDER_AUDIENCE,
                    vec!["internal:notifications:read".into()],
                ),
                _ => return Err(epsx_service_auth::VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0xabc".into(),
                wallet_address: "0xabc".into(),
                audience: audience.into(),
                permissions,
            })
        }
    }

    async fn test_app() -> (Router, Arc<Capture>, Arc<FakeVerifier>) {
        let capture = Arc::new(Capture::default());
        let upstream = Router::new()
            .fallback(any(capture_upstream))
            .with_state(capture.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, upstream).await.unwrap();
        });
        let url = format!("http://{address}");
        let verifier = Arc::new(FakeVerifier::new());
        let state = AppState::new(
            GatewayUrls {
                identity: url.clone(),
                wallet: url.clone(),
                payment: url.clone(),
                subscription: url.clone(),
                content: url.clone(),
                notification: url.clone(),
                analytics: url.clone(),
                indexer: url,
            },
            build_http_client().unwrap(),
            verifier.clone(),
            false,
        )
        .unwrap();
        (build_router(state), capture, verifier)
    }

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn public_and_credential_routes_strip_bearer_and_spoofable_headers() {
        let (app, capture, verifier) = test_app().await;
        for (method, path) in [
            (Method::GET, "/api/v1/wallet/balance/56/0xabc"),
            (Method::POST, "/api/v1/identity/auth/refresh"),
        ] {
            let request = axum::http::Request::builder()
                .method(method)
                .uri(path)
                .header(header::AUTHORIZATION, "Bearer invalid")
                .header("x-user-id", "attacker")
                .header("x-wallet-address", "attacker")
                .header("x-subject", "attacker")
                .header(header::CONNECTION, "x-leak")
                .header("x-leak", "secret")
                .header("x-request-id", "safe-request-1")
                .body(Body::empty())
                .unwrap();
            assert_eq!(status(&app, request).await, StatusCode::NO_CONTENT);
        }

        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
        let requests = capture.requests.lock().await;
        assert_eq!(requests.len(), 2);
        for request in requests.iter() {
            assert!(!request.headers.contains_key(header::AUTHORIZATION));
            assert!(!request.headers.contains_key("x-user-id"));
            assert!(!request.headers.contains_key("x-wallet-address"));
            assert!(!request.headers.contains_key("x-subject"));
            assert!(!request.headers.contains_key("x-leak"));
            assert_eq!(request.headers["x-request-id"], "safe-request-1");
        }
    }

    #[tokio::test]
    async fn denial_never_contacts_upstream() {
        let (app, capture, _) = test_app().await;
        let cases = [
            (
                Method::POST,
                "/api/v1/wallet/send",
                None,
                StatusCode::UNAUTHORIZED,
            ),
            (
                Method::GET,
                "/api/v1/identity/users",
                Some("Bearer front"),
                StatusCode::FORBIDDEN,
            ),
            (
                Method::GET,
                "/api/v1/identity/users",
                Some("Bearer admin-no-scope"),
                StatusCode::FORBIDDEN,
            ),
            (
                Method::GET,
                "/api/v1/analytics/metrics/prometheus",
                Some("Bearer admin-global"),
                StatusCode::NOT_FOUND,
            ),
            (
                Method::POST,
                "/api/v1/payment/intents",
                Some("Bearer front"),
                StatusCode::NOT_FOUND,
            ),
            (
                Method::POST,
                "/api/v1/pay/intents",
                Some("Bearer front"),
                StatusCode::NOT_FOUND,
            ),
            (
                Method::POST,
                "/api/v1/identity/auth/demo",
                None,
                StatusCode::NOT_FOUND,
            ),
            (
                Method::GET,
                "/api/v1/indexer/sync",
                Some("Bearer admin-global"),
                StatusCode::NOT_FOUND,
            ),
        ];

        for (method, path, authorization, expected) in cases {
            let mut builder = axum::http::Request::builder().method(method).uri(path);
            if let Some(authorization) = authorization {
                builder = builder.header(header::AUTHORIZATION, authorization);
            }
            assert_eq!(
                status(&app, builder.body(Body::empty()).unwrap()).await,
                expected
            );
        }
        assert_eq!(capture.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn owner_history_is_authenticated_and_forwarded_without_rewrite() {
        let (app, capture, verifier) = test_app().await;
        let wallet = "0x1111111111111111111111111111111111111111";
        let uri = format!("/api/v1/pay/history/{wallet}?limit=10&offset=0");
        let request = axum::http::Request::builder()
            .method(Method::GET)
            .uri(&uri)
            .header(header::AUTHORIZATION, "Bearer front")
            .body(Body::empty())
            .unwrap();

        assert_eq!(status(&app, request).await, StatusCode::NO_CONTENT);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 1);
        assert_eq!(capture.hits.load(Ordering::SeqCst), 1);
        let requests = capture.requests.lock().await;
        assert_eq!(requests[0].uri, uri);
        assert_eq!(requests[0].headers[header::AUTHORIZATION], "Bearer front");
    }

    #[tokio::test]
    async fn owner_history_requires_authentication_before_upstream() {
        let (app, capture, _) = test_app().await;
        let request = axum::http::Request::builder()
            .method(Method::GET)
            .uri("/api/v1/pay/history/0x1111111111111111111111111111111111111111")
            .body(Body::empty())
            .unwrap();

        assert_eq!(status(&app, request).await, StatusCode::UNAUTHORIZED);
        assert_eq!(capture.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn other_pay_shapes_are_not_authenticated_or_forwarded() {
        let (app, capture, verifier) = test_app().await;
        let wallet = "0x1111111111111111111111111111111111111111";
        let oversized = "a".repeat(policy::MAX_WALLET_SEGMENT_BYTES + 1);
        let cases = [
            (Method::HEAD, format!("/api/v1/pay/history/{wallet}")),
            (Method::POST, format!("/api/v1/pay/history/{wallet}")),
            (Method::GET, "/api/v1/pay/history".into()),
            (Method::GET, format!("/api/v1/pay/history/{wallet}/extra")),
            (Method::GET, format!("/api/v1/pay/history/{wallet}/")),
            (Method::GET, "/api/v1/pay/history/0xabc%2Fextra".into()),
            (Method::GET, "/api/v1/pay/history/0xabc%252Fextra".into()),
            (Method::GET, "/api/v1/pay/history/wallet:0xabc".into()),
            (Method::GET, "/api/v1/pay/history/history".into()),
            (Method::GET, "/api/v1/pay/history/force-release".into()),
            (Method::GET, format!("/api/v1/pay/history/{oversized}")),
            (Method::GET, format!("/api/v1/payment/history/{wallet}")),
            (Method::GET, "/api/v1/pay/intents".into()),
            (Method::POST, "/api/v1/pay/intents".into()),
        ];

        for (method, uri) in cases {
            let request = axum::http::Request::builder()
                .method(method.clone())
                .uri(&uri)
                .header(header::AUTHORIZATION, "Bearer front")
                .body(Body::empty())
                .unwrap();
            assert_eq!(
                status(&app, request).await,
                StatusCode::NOT_FOUND,
                "{method} {uri}"
            );
        }

        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
        assert_eq!(capture.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn duplicate_authorization_is_rejected_before_upstream() {
        let (app, capture, _) = test_app().await;
        let mut request = axum::http::Request::builder()
            .method(Method::POST)
            .uri("/api/v1/wallet/send")
            .body(Body::empty())
            .unwrap();
        request.headers_mut().append(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer front"),
        );
        request.headers_mut().append(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer admin-global"),
        );
        assert_eq!(status(&app, request).await, StatusCode::UNAUTHORIZED);
        assert_eq!(capture.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn audit_route_requires_admin_audience_and_permission_before_upstream() {
        let (app, capture, _) = test_app().await;
        let path = "/api/v1/analytics/admin/audit-log";
        for (bearer, expected) in [
            (None, StatusCode::UNAUTHORIZED),
            (Some("front"), StatusCode::FORBIDDEN),
            (Some("admin-no-scope"), StatusCode::FORBIDDEN),
        ] {
            let mut builder = axum::http::Request::builder().method(Method::GET).uri(path);
            if let Some(bearer) = bearer {
                builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
            }
            assert_eq!(
                status(&app, builder.body(Body::empty()).unwrap()).await,
                expected
            );
        }
        assert_eq!(capture.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn authenticated_and_granular_admin_bearers_forward_unchanged() {
        let (app, capture, _) = test_app().await;
        let cases = [
            (Method::POST, "/api/v1/wallet/send", "Bearer front"),
            (Method::GET, "/api/v1/identity/users", "Bearer admin-users"),
            (
                Method::POST,
                "/api/v1/notification/send",
                "Bearer admin-global",
            ),
            (
                Method::GET,
                "/api/v1/analytics/admin/audit-log",
                "Bearer admin-audit",
            ),
        ];
        for (method, path, authorization) in cases {
            let request = axum::http::Request::builder()
                .method(method)
                .uri(path)
                .header(header::AUTHORIZATION, authorization)
                .body(Body::from("ok"))
                .unwrap();
            assert_eq!(status(&app, request).await, StatusCode::NO_CONTENT);
        }
        let requests = capture.requests.lock().await;
        assert_eq!(requests.len(), 4);
        assert_eq!(requests[0].headers[header::AUTHORIZATION], "Bearer front");
        assert_eq!(
            requests[1].headers[header::AUTHORIZATION],
            "Bearer admin-users"
        );
        assert_eq!(
            requests[2].headers[header::AUTHORIZATION],
            "Bearer admin-global"
        );
        assert_eq!(
            requests[3].headers[header::AUTHORIZATION],
            "Bearer admin-audit"
        );
        assert!(requests.iter().all(|request| request.body == b"ok"));
    }

    #[tokio::test]
    async fn notification_service_audience_routes_require_their_internal_identity() {
        let (app, capture, _) = test_app().await;
        for (path, bearer) in [
            ("/api/v1/notification/publish", "publisher"),
            ("/api/v1/notification/provider-events", "provider"),
        ] {
            let request = axum::http::Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {bearer}"))
                .body(Body::from("ok"))
                .unwrap();
            assert_eq!(status(&app, request).await, StatusCode::NO_CONTENT);
        }

        for (path, bearer, expected) in [
            (
                "/api/v1/notification/publish",
                "front",
                StatusCode::FORBIDDEN,
            ),
            (
                "/api/v1/notification/publish",
                "admin-global",
                StatusCode::FORBIDDEN,
            ),
            (
                "/api/v1/notification/provider-events",
                "publisher",
                StatusCode::FORBIDDEN,
            ),
            (
                "/api/v1/notification/provider-events",
                "provider-wrong-permission",
                StatusCode::FORBIDDEN,
            ),
        ] {
            let request = axum::http::Request::builder()
                .method(Method::POST)
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {bearer}"))
                .body(Body::empty())
                .unwrap();
            assert_eq!(status(&app, request).await, expected);
        }
        assert_eq!(capture.hits.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn request_body_is_capped_before_upstream() {
        let (app, capture, _) = test_app().await;
        let request = axum::http::Request::builder()
            .method(Method::POST)
            .uri("/api/v1/wallet/send")
            .header(header::AUTHORIZATION, "Bearer front")
            .body(Body::from(vec![0_u8; MAX_REQUEST_BODY_BYTES + 1]))
            .unwrap();
        assert_eq!(status(&app, request).await, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(capture.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn upstream_response_body_is_capped() {
        let (app, capture, _) = test_app().await;
        let request = axum::http::Request::builder()
            .method(Method::GET)
            .uri("/api/v1/news/large-response")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status(&app, request).await, StatusCode::BAD_GATEWAY);
        assert_eq!(capture.hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn invalid_request_id_is_replaced_and_alias_is_rewritten() {
        let (app, capture, _) = test_app().await;
        let request = axum::http::Request::builder()
            .method(Method::GET)
            .uri("/api/v1/news/story")
            .header("x-request-id", "contains spaces")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status(&app, request).await, StatusCode::NO_CONTENT);
        let requests = capture.requests.lock().await;
        assert_eq!(requests[0].path, "/api/v1/content/news/story");
        let request_id = requests[0].headers["x-request-id"].to_str().unwrap();
        assert_ne!(request_id, "contains spaces");
        assert!(request_id.len() <= MAX_REQUEST_ID_BYTES);
    }

    #[tokio::test]
    async fn upstream_service_cookies_are_not_exposed() {
        let (app, _, _) = test_app().await;
        let request = axum::http::Request::builder()
            .method(Method::GET)
            .uri("/api/v1/content/site")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(!response.headers().contains_key(header::SET_COOKIE));
    }

    #[test]
    fn upstream_configuration_rejects_unsafe_and_prod_local_urls() {
        assert!(validate_upstream("test", "ftp://service.example", false).is_err());
        assert!(validate_upstream("test", "http://user@service.example", false).is_err());
        assert!(validate_upstream("test", "http://service.example/base", false).is_err());
        assert!(validate_upstream("test", "http://service.example?x=1", false).is_err());
        assert!(validate_upstream("test", "http://localhost:8101", true).is_err());
        assert!(validate_upstream("test", "http://identity.localhost:8101", true).is_err());
        assert!(validate_upstream("test", "http://127.0.0.1:8101", true).is_err());
        assert_eq!(
            validate_upstream("test", "http://identity.epsx.svc:8101", true).unwrap(),
            "http://identity.epsx.svc:8101"
        );
    }
}
