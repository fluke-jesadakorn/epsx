//! EPSX Frontend BFF.
//!
//! Dioxus 0.7 fullstack SSR + axum JSON API proxy.
//!
//! Strategy:
//! - The design system (Tailwind v2 CDN, glassmorphism, EPSX color tokens,
//!   global JS controllers, FOUC prevention) is injected by
//!   `epsx_templates::design_system_head` + `epsx_templates::global_js` —
//!   exactly the same as the Next.js frontend.
//! - The page body is rendered by Dioxus `rsx!` components from
//!   `epsx_dioxus_ui::pages` and serialized to HTML via `dioxus_ssr`.
//! - JSON API endpoints (`/api/*`) are kept on the same axum router and
//!   proxied to the gateway via `epsx_client::ServiceClient`.

use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use epsx_bff::{
    cookies::CookieEnvironment,
    middleware::security_headers,
    session::{JwksVerifier, JwksVerifierConfig, FRONTEND_CLIENT_ID, JWKS_PATH},
    static_assets::browser_runtime_router,
};
use epsx_client::ServiceClient;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

mod api;
mod auth;
mod chat_adapter;
mod payment_adapter;
mod ssr;
mod widgets;

use api::*;
#[derive(Clone)]
pub struct AppState {
    pub identity: Arc<ServiceClient>,
    pub notification: Arc<ServiceClient>,
    pub content: Arc<ServiceClient>,
    pub analytics: Arc<ServiceClient>,
    pub wallet: Arc<ServiceClient>,
    pub payment: Arc<ServiceClient>,
    pub subscription: Arc<ServiceClient>,
    pub verifier: Arc<JwksVerifier>,
    pub cookie_environment: CookieEnvironment,
    pub api_url: String,
    pub notification_url: String,
}

impl AppState {
    /// BIG-BANG Phase 4b: canonical session helper — callers should use this instead of
    /// constructing `JwksVerifier` + `CookieEnvironment` pairs manually.
    #[allow(dead_code)]
    pub fn session(&self) -> epsx_bff::typed_session::TypedBffSession {
        epsx_bff::typed_session::TypedBffSession::frontend(
            self.verifier.clone(),
            self.cookie_environment,
        )
    }
}

#[derive(Deserialize)]
pub struct SiweLoginBody {
    pub message: String,
    pub signature: String,
    #[serde(default)]
    pub chain_id: String,
    /// Wallet address (lowercased) that produced the signature. Wave 50b —
    /// the monolithic backend's `SignatureVerificationRequest` requires
    /// `wallet_address`, so we propagate it from the auth-page JS.
    pub address: String,
    /// Challenge nonce returned by `/api/auth/web3/challenge`. Wave 50b —
    /// the monolithic backend requires it as a separate field.
    pub nonce: String,
}

#[derive(Deserialize)]
pub struct ChallengeBody {
    pub address: String,
}

#[derive(Deserialize)]
pub struct AnalyticsTrackBody {
    pub event_name: String,
    pub properties: Option<serde_json::Value>,
    pub user_id: Option<String>,
    pub chain_id: Option<String>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-frontend");
    let state = state_from_env().expect("valid frontend authentication configuration");
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let api_url = state.api_url.clone();
    let app = build_app(state);

    if host.eq_ignore_ascii_case("localhost") {
        let ipv4 = SocketAddr::from(([127, 0, 0, 1], port));
        let ipv6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], port));
        tracing::info!(
            "Frontend BFF listening on http://localhost:{} via {} and {} (api={})",
            port,
            ipv4,
            ipv6,
            api_url
        );
        let ipv4_listener = tokio::net::TcpListener::bind(ipv4).await.unwrap();
        let ipv6_listener = tokio::net::TcpListener::bind(ipv6).await.unwrap();
        tokio::try_join!(
            axum::serve(ipv4_listener, app.clone()),
            axum::serve(ipv6_listener, app)
        )
        .unwrap();
    } else {
        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
        tracing::info!(
            "Frontend BFF listening on http://{} (api={})",
            addr,
            api_url
        );
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

fn state_from_env() -> Result<AppState, String> {
    let cookie_environment = CookieEnvironment::from_env().map_err(|error| error.to_string())?;
    let api_url = std::env::var("API_URL")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .map_err(|_| "API_URL or BACKEND_URL is required".to_string())?;
    let notification_url =
        std::env::var("NOTIFICATION_SERVICE_URL").unwrap_or_else(|_| api_url.clone());
    let content_url = std::env::var("CONTENT_SERVICE_URL").unwrap_or_else(|_| api_url.clone());
    let issuer = std::env::var("OIDC_ISSUER")
        .or_else(|_| std::env::var("BACKEND_URL"))
        .map_err(|_| "OIDC_ISSUER or BACKEND_URL is required".to_string())?;
    validate_auth_url(&api_url, cookie_environment, "API_URL/BACKEND_URL")?;
    validate_auth_url(
        &notification_url,
        cookie_environment,
        "NOTIFICATION_SERVICE_URL",
    )?;
    validate_auth_url(&content_url, cookie_environment, "CONTENT_SERVICE_URL")?;
    validate_auth_url(&issuer, cookie_environment, "OIDC_ISSUER/BACKEND_URL")?;

    let jwks_url = format!("{}{}", api_url.trim_end_matches('/'), JWKS_PATH);
    let verifier_config = JwksVerifierConfig::new(
        jwks_url,
        issuer.trim_end_matches('/'),
        FRONTEND_CLIENT_ID,
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
        base_url: notification_url.clone(),
        timeout: Duration::from_secs(15),
    };
    let content_cfg = epsx_client::ClientConfig {
        base_url: content_url,
        timeout: Duration::from_secs(15),
    };
    Ok(AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(notification_cfg)),
        content: Arc::new(ServiceClient::new(content_cfg)),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg)),
        verifier,
        cookie_environment,
        api_url,
        notification_url,
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
        assert!(validate_auth_url(
            "https://127.0.0.1:8080",
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
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/auth/siwe", post(siwe_login))
        .route("/api/v1/auth/challenge", post(auth_challenge))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/me", get(auth_me))
        .route(
            "/api/v1/notifications",
            get(notifications_api).head(|| async { axum::http::StatusCode::METHOD_NOT_ALLOWED }),
        )
        // Pinned development-branch compatibility surface.  The target
        // `/api/v1/*` routes above remain the canonical Rust contract; these
        // aliases are deliberately kept at the BFF boundary so old shared
        // clients can migrate without bypassing the verified owner context.
        .route("/api/notifications", get(legacy_notifications_api))
        .route(
            "/api/v1/notifications/unread-count",
            get(notification_unread_count)
                .head(|| async { axum::http::StatusCode::METHOD_NOT_ALLOWED }),
        )
        .route(
            "/api/notifications/unread-count",
            get(legacy_notification_unread_count),
        )
        .route(
            "/api/v1/notifications/preferences",
            get(notification_preferences_get).put(notification_preferences_put),
        )
        .route(
            "/api/notifications/preferences",
            get(legacy_notification_preferences_get)
                .put(legacy_notification_preferences_put)
                .post(legacy_notification_preferences_put),
        )
        // Development clients also used the auth-prefixed aliases. They are
        // routed through the same verified-owner adapters, never directly to
        // the notification service.
        .route(
            "/api/auth/notifications/preferences",
            get(legacy_notification_preferences_get)
                .put(legacy_notification_preferences_put)
                .post(legacy_notification_preferences_put),
        )
        .route(
            "/api/auth/notifications/unread-count",
            get(legacy_notification_unread_count),
        )
        .route(
            "/account/notification-preferences",
            post(notification_preferences_form),
        )
        .route("/api/v1/notifications/stream", get(notification_stream))
        .route("/api/notifications/stream", get(legacy_notification_stream))
        .route(
            "/api/auth/notifications/stream",
            get(legacy_notification_stream),
        )
        .route(
            "/api/v1/notifications/stream/ack",
            post(notification_stream_ack),
        )
        .route(
            "/api/v1/notifications/push",
            get(notification_push_status)
                .put(notification_push_subscribe)
                .delete(notification_push_unsubscribe),
        )
        .route(
            "/api/notifications/push/status",
            get(legacy_notification_push_status),
        )
        .route(
            "/api/notifications/push/subscribe",
            post(legacy_notification_push_subscribe),
        )
        .route(
            "/api/notifications/push/unsubscribe",
            delete(legacy_notification_push_unsubscribe),
        )
        .route(
            "/api/auth/notifications/push/status",
            get(legacy_notification_push_status),
        )
        .route(
            "/api/auth/notifications/push/subscribe",
            post(legacy_notification_push_subscribe),
        )
        .route(
            "/api/auth/notifications/push/unsubscribe",
            delete(legacy_notification_push_unsubscribe),
        )
        .route(
            "/api/v1/notifications/{id}/read",
            post(notification_read).put(notification_read),
        )
        .route(
            "/api/notifications/{id}/read",
            put(legacy_notification_read),
        )
        .route(
            "/api/v1/notifications/{id}/unread",
            post(notification_unread).put(notification_unread),
        )
        .route(
            "/api/notifications/{id}/unread",
            put(legacy_notification_unread),
        )
        .route(
            "/api/v1/notifications/{id}/acknowledge",
            put(notification_acknowledge),
        )
        .route(
            "/api/notifications/{id}/acknowledge",
            put(legacy_notification_acknowledge),
        )
        .route("/api/v1/notifications/{id}/click", post(notification_click))
        .route(
            "/api/v1/notifications/{id}/dismiss",
            post(notification_dismiss),
        )
        .route(
            "/api/v1/notifications/{id}/delete",
            post(notification_delete),
        )
        .route("/api/v1/notifications/{id}", delete(notification_delete))
        .route(
            "/api/notifications/{id}",
            delete(legacy_notification_delete),
        )
        .route(
            "/api/v1/notifications/mark-all-read",
            post(notification_mark_all).put(notification_mark_all),
        )
        .route(
            "/api/notifications/mark-all-read",
            put(legacy_notification_mark_all),
        )
        .route(
            "/api/v1/notifications/clear-all",
            post(notification_clear_all).delete(notification_clear_all),
        )
        .route(
            "/api/notifications/clear-all",
            delete(legacy_notification_clear_all),
        )
        .route("/api/v1/analytics/track", post(track_event))
        .route(
            "/api/users/watchlist",
            get(watchlist_get).post(watchlist_post),
        )
        .route("/api/users/watchlist/{symbol}", delete(watchlist_delete))
        .route(
            "/api/users/watchlist/layout",
            get(watchlist_layout_get).put(watchlist_layout_put),
        )
        .route(
            "/api/users/watchlist/groups",
            get(watchlist_layout_get).post(watchlist_group_post),
        )
        .route(
            "/api/users/watchlist/groups/{group_id}",
            put(watchlist_group_put).delete(watchlist_group_delete),
        )
        .route("/portfolio/watch", post(watchlist_add_form))
        .route("/portfolio/unwatch", post(watchlist_remove_form))
        // Ranking compatibility producers remain intentionally absent. SSR
        // reads the canonical analytics service without moving rank policy
        // into the frontend.
        .route("/api/v1/news", get(api_news))
        .route("/api/v1/news/{slug}", get(api_news_post))
        .route(
            "/api/v1/payments/submit",
            post(payment_adapter::submit_plan_payment),
        )
        .route(
            "/api/v1/payments/status/{tx_hash}",
            get(payment_adapter::payment_status),
        )
        .route("/api/v1/chat/inbox", get(chat_adapter::chat_inbox_api))
        .route(
            "/api/v1/chat/conversations",
            post(chat_adapter::chat_create_api),
        )
        .route(
            "/api/v1/chat/conversations/{id}/full",
            get(chat_adapter::chat_detail_api),
        )
        .route(
            "/api/v1/chat/conversations/{id}/messages",
            post(chat_adapter::chat_send_api),
        )
        .route("/api/v1/developer/overview", get(developer_overview))
        .route("/api/v1/developer/usage", get(developer_usage))
        .route(
            "/api/v1/developer/keys",
            get(developer_keys).post(developer_key_create),
        )
        .route(
            "/api/v1/developer/keys/{id}/revoke",
            post(developer_key_revoke),
        )
        .route("/api/v1/developer/openapi", get(developer_openapi))
        .route("/api/v1/developer/try", post(developer_try))
        .route("/developer/keys/create", post(developer_key_create_form))
        .route(
            "/developer/keys/{id}/revoke",
            post(developer_key_revoke_form),
        )
        .route(
            "/chat",
            get(ssr::ssr_handler).post(chat_adapter::chat_create_form),
        )
        .route(
            "/chat/{id}",
            get(ssr::ssr_handler).post(chat_adapter::chat_conversation_form),
        )
        // Unowned dashboard and portfolio compatibility producers are
        // intentionally absent until owner-scoped backend contracts exist.
        // Unowned wallet/session and subscription compatibility producers are
        // intentionally absent. The backend owns wallet sessions, plan
        // catalogs, eligibility, and subscription mutations.
        .merge(browser_runtime_router(&browser_runtime_dir()))
        .nest_service(
            "/public",
            tower_http::services::ServeDir::new(format!("{}/public", env!("CARGO_MANIFEST_DIR")))
                .fallback(tower_http::services::ServeFile::new(format!(
                    "{}/public/index.html",
                    env!("CARGO_MANIFEST_DIR")
                ))),
        )
        .fallback(fallback_handler)
        .layer(axum::middleware::from_fn(security_headers))
        .with_state(state)
}

fn browser_runtime_dir() -> String {
    std::env::var("EPSX_BROWSER_RUNTIME_DIR").unwrap_or_else(|_| {
        format!(
            "{}/../../target/epsx-browser-runtime",
            env!("CARGO_MANIFEST_DIR")
        )
    })
}

fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

fn api_not_found_response() -> Response {
    (
        axum::http::StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not_found",
            "message": "API route not found"
        })),
    )
        .into_response()
}

async fn fallback_handler(State(state): State<AppState>, request: Request) -> Response {
    let path = request.uri().path();
    if is_api_path(path) {
        api_not_found_response()
    } else if path
        .strip_prefix("/portfolio/")
        .is_some_and(|address| !address.is_empty() && !address.contains('/'))
    {
        (
            axum::http::StatusCode::TEMPORARY_REDIRECT,
            [(axum::http::header::LOCATION, "/portfolio")],
            "",
        )
            .into_response()
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
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use epsx_bff::session::{AccessTokenClaims, Jwks, JwksFetcher, RsaJwk, SessionError};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use rand::thread_rng;
    use rsa::{pkcs8::EncodePrivateKey, traits::PublicKeyParts, RsaPrivateKey};
    use std::{
        future::Future,
        pin::Pin,
        time::{SystemTime, UNIX_EPOCH},
    };
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

    struct StaticJwksFetcher {
        jwks: Jwks,
    }

    impl JwksFetcher for StaticJwksFetcher {
        fn fetch<'life0, 'life1, 'async_trait>(
            &'life0 self,
            _url: &'life1 str,
        ) -> Pin<Box<dyn Future<Output = Result<Jwks, SessionError>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async { Ok(self.jwks.clone()) })
        }
    }

    fn verifier_config() -> JwksVerifierConfig {
        let base_url = "http://127.0.0.1:9";
        JwksVerifierConfig::new(
            format!("{base_url}{JWKS_PATH}"),
            "https://issuer.test",
            FRONTEND_CLIENT_ID,
            Duration::from_secs(60),
        )
        .unwrap()
    }

    fn state_with_verifier(verifier: JwksVerifier) -> AppState {
        let base_url = "http://127.0.0.1:9";
        let config = epsx_client::ClientConfig {
            base_url: base_url.to_string(),
            timeout: Duration::from_millis(50),
        };
        let client = Arc::new(ServiceClient::new(config));
        AppState {
            identity: client.clone(),
            notification: client.clone(),
            content: client.clone(),
            analytics: client.clone(),
            wallet: client.clone(),
            payment: client.clone(),
            subscription: client,
            verifier: Arc::new(verifier),
            cookie_environment: CookieEnvironment::Local,
            api_url: base_url.to_string(),
            notification_url: base_url.to_string(),
        }
    }

    fn test_state() -> AppState {
        state_with_verifier(JwksVerifier::new(
            verifier_config(),
            Arc::new(FailingJwksFetcher),
        ))
    }

    fn valid_frontend_session() -> (AppState, String) {
        let private = RsaPrivateKey::new(&mut thread_rng(), 2048).unwrap();
        let pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let encoding = EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap();
        let public = private.to_public_key();
        let kid = "frontend-current";
        let jwk = RsaJwk {
            kty: "RSA".into(),
            use_: Some("sig".into()),
            alg: Some("RS256".into()),
            kid: kid.into(),
            n: URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
            e: URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let claims = AccessTokenClaims {
            iss: "https://issuer.test".into(),
            sub: "0xabc".into(),
            aud: vec![FRONTEND_CLIENT_ID.into()],
            exp: now + 300,
            iat: now,
            jti: "frontend-test-jti".into(),
            scope: "openid profile epsx:analytics:read".into(),
            wallet_address: "0xabc".into(),
            auth_method: "web3_siwe".into(),
            auth_time: now,
            nbf: None,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());
        let token = encode(&header, &claims, &encoding).unwrap();
        let verifier = JwksVerifier::new(
            verifier_config(),
            Arc::new(StaticJwksFetcher {
                jwks: Jwks { keys: vec![jwk] },
            }),
        );
        (state_with_verifier(verifier), token)
    }

    async fn request(method: Method, uri: &str) -> Response {
        request_with_cookie(method, uri, None).await
    }

    async fn request_with_cookie(method: Method, uri: &str, cookie: Option<&str>) -> Response {
        request_with_state_and_cookie(test_state(), method, uri, cookie).await
    }

    async fn request_with_state_and_cookie(
        state: AppState,
        method: Method,
        uri: &str,
        cookie: Option<&str>,
    ) -> Response {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }
        build_app(state)
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn frontend_emits_one_private_recovery_bootstrap_only_for_refresh_eligible_html() {
        let refresh_cookie = "epsx.frontend.refresh_token=opaque-refresh";
        let response = request_with_cookie(
            Method::GET,
            "/auth?return_url=%2Faccount",
            Some(refresh_cookie),
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
        assert_eq!(html.matches("epsx_browser_runtime_bootstrap.js").count(), 1);
        assert!(!html.contains("window.epsxAuth"));
        assert!(html.contains("data-auth-session-state=\"recovering\""));
        assert!(html.contains("Restoring your session..."));
        assert!(html.contains("disabled=\"true\"") || html.contains("disabled=\"disabled\""));
        assert!(!html.contains("opaque-refresh"));
        let runtime_position = html
            .find("epsx_browser_runtime_bootstrap.js")
            .expect("the generated Rust/WASM runtime module must be present");
        let recovery_position = html
            .find("data-epsx-session-recovery")
            .expect("the recovery bootstrap must be present");
        assert!(
            runtime_position < recovery_position,
            "the generated runtime must load before the recovery marker"
        );

        let wrong_client = request_with_cookie(
            Method::GET,
            "/auth",
            Some("epsx.admin.refresh_token=wrong-client"),
        )
        .await;
        let wrong_client_html = to_bytes(wrong_client.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let wrong_client_html = String::from_utf8_lossy(&wrong_client_html);
        assert!(!wrong_client_html.contains("data-epsx-session-recovery"));
        assert!(wrong_client_html.contains("data-auth-session-state=\"signed_out\""));

        let protected =
            request_with_cookie(Method::GET, "/profile?view=compact", Some(refresh_cookie)).await;
        assert_eq!(protected.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            protected.headers()[header::LOCATION],
            "/auth?return_url=%2Fprofile%3Fview%3Dcompact"
        );
        assert_eq!(
            protected.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(protected.headers()[header::VARY], "Cookie, Authorization");
        let return_location = protected.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .to_string();
        let auth_return =
            request_with_cookie(Method::GET, &return_location, Some(refresh_cookie)).await;
        let auth_return_html = to_bytes(auth_return.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&auth_return_html)
                .matches("data-epsx-session-recovery")
                .count(),
            1
        );

        let rejected = request_with_cookie(
            Method::GET,
            "/auth",
            Some(
                "epsx.frontend.access_token=malformed; epsx.frontend.refresh_token=opaque-refresh",
            ),
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
            "/auth",
            Some("epsx.frontend.access_token=eyJhbGciOiJSUzI1NiIsImtpZCI6Im91dGFnZSIsInR5cCI6IkpXVCJ9.e30.c2ln; epsx.frontend.refresh_token=opaque-refresh"),
        )
        .await;
        assert_eq!(
            verifier_outage.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(
            verifier_outage.headers()[header::VARY],
            "Cookie, Authorization"
        );
        let outage_html = to_bytes(verifier_outage.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let outage_html = String::from_utf8_lossy(&outage_html);
        assert!(!outage_html.contains("data-epsx-session-recovery"));
        assert!(outage_html.contains("data-auth-session-state=\"verifier_unavailable\""));
        assert!(outage_html.contains("Sign-in temporarily unavailable"));
        assert!(outage_html
            .contains("We cannot verify your session right now. Please try again later."));
        assert!(
            outage_html.contains("disabled=\"true\"")
                || outage_html.contains("disabled=\"disabled\"")
        );

        let (valid_state, access_token) = valid_frontend_session();
        let valid_cookie = format!(
            "epsx.frontend.access_token={access_token}; epsx.frontend.refresh_token=opaque-refresh"
        );
        let valid_auth = request_with_state_and_cookie(
            valid_state.clone(),
            Method::GET,
            "/auth?return_url=%2Fprofile",
            Some(&valid_cookie),
        )
        .await;
        assert_eq!(valid_auth.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(valid_auth.headers()[header::LOCATION], "/profile");
        assert_eq!(
            valid_auth.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(valid_auth.headers()[header::VARY], "Cookie, Authorization");

        let no_refresh = request(Method::GET, "/auth").await;
        let no_refresh_html = to_bytes(no_refresh.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let no_refresh_html = String::from_utf8_lossy(&no_refresh_html);
        assert!(!no_refresh_html.contains("data-epsx-session-recovery"));
        assert!(no_refresh_html.contains("data-auth-session-state=\"signed_out\""));

        let offline = request_with_cookie(Method::GET, "/offline", Some(refresh_cookie)).await;
        assert_eq!(
            offline.headers()[header::CACHE_CONTROL],
            "public, max-age=0, must-revalidate"
        );
        assert!(offline.headers().get(header::VARY).is_none());
        let offline_html = to_bytes(offline.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        assert!(!String::from_utf8_lossy(&offline_html).contains("data-epsx-session-recovery"));

        let anonymous_offline = request(Method::GET, "/offline").await;
        let anonymous_offline_html = to_bytes(anonymous_offline.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let authenticated_offline = request_with_state_and_cookie(
            valid_state,
            Method::GET,
            "/offline",
            Some(&valid_cookie),
        )
        .await;
        assert_eq!(
            authenticated_offline.headers()[header::CACHE_CONTROL],
            "public, max-age=0, must-revalidate"
        );
        assert!(authenticated_offline.headers().get(header::VARY).is_none());
        let authenticated_offline_html =
            to_bytes(authenticated_offline.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
        assert_eq!(authenticated_offline_html, anonymous_offline_html);
    }

    #[tokio::test]
    async fn ssr_known_unknown_and_malformed_routes_have_explicit_status() {
        assert_eq!(request(Method::GET, "/").await.status(), StatusCode::OK);

        for path in [
            "/missing-page",
            "/portfolio/",
            "/portfolio/address/extra",
            "/chat/id/extra",
            "/news/slug/extra",
            "/payment/intent",
            "/payment/intent/id/extra",
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
            assert!(
                String::from_utf8_lossy(&body).contains("Page not found"),
                "{path}"
            );
        }

        assert_eq!(
            request(Method::HEAD, "/missing-page").await.status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn ssr_invalid_and_malformed_news_route_shapes_are_404() {
        for path in ["/news/UPPER", "/news/-leading-hyphen", "/news/%2Fsecret"] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            assert!(
                String::from_utf8_lossy(&body).contains("Article not found"),
                "{path}"
            );
        }

        for path in ["/news/", "/news/live-article/", "/news/not/a-route"] {
            let response = request(Method::GET, path).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            assert!(
                String::from_utf8_lossy(&body).contains("Page not found"),
                "{path}"
            );
        }
    }

    #[tokio::test]
    async fn unknown_api_is_json_and_known_method_mismatch_stays_405() {
        for path in [
            "/api",
            "/api/",
            "/api/v1/plans/extra",
            "/api/v1/plans",
            "/api/v1/rankings",
            "/api/v1/credits",
            "/api/v1/account",
            "/api/v1/developer",
            "/api/v1/developer/docs",
            "/api/v1/analytics/summary",
            "/api/v1/dashboard",
            "/api/v1/dashboard/stats",
            "/api/v1/portfolio/0x0000000000000000000000000000000000000001",
            "/api/v1/payment/not-an-authorized-intent",
            "/api/v1/auth/oauth/google",
            "/api/v1/auth/oauth/unknown",
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

        let head = request(Method::HEAD, "/api/v1/plans/extra").await;
        assert_eq!(head.status(), StatusCode::NOT_FOUND);
        assert_eq!(head.headers()[header::CONTENT_TYPE], "application/json");

        assert_eq!(
            request(Method::GET, "/api/v1/developer/usage")
                .await
                .status(),
            StatusCode::UNAUTHORIZED,
            "the developer usage adapter must exist and require a locally verified session"
        );
        assert_eq!(
            request(Method::POST, "/api/v1/developer/try")
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "Try It must reject requests without same-origin evidence"
        );
        assert_eq!(
            request(Method::POST, "/developer/keys/create")
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "the native create fallback must exist and reject cross-origin submission"
        );

        assert_eq!(
            request(Method::POST, "/api/v1/news").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
        for method in [
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ] {
            for path in [
                "/api/v1/notifications",
                "/api/v1/notifications/unread-count",
            ] {
                assert_eq!(
                    request(method.clone(), path).await.status(),
                    StatusCode::METHOD_NOT_ALLOWED,
                    "{method} {path}"
                );
            }
        }

        assert_eq!(
            request(Method::GET, "/account/notification-preferences")
                .await
                .status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
        assert_eq!(
            request(Method::POST, "/account/notification-preferences")
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "the native preferences form route must exist and reject missing Origin/Host before auth"
        );
    }

    #[tokio::test]
    async fn watchlist_bff_requires_local_auth_same_origin_and_forwards_verified_bearer() {
        let seen = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let seen_get = Arc::clone(&seen);
        let seen_post = Arc::clone(&seen);
        let seen_delete = Arc::clone(&seen);
        let upstream = Router::new()
            .route(
                "/api/users/watchlist",
                get(move |headers: axum::http::HeaderMap| {
                    let seen_get = Arc::clone(&seen_get);
                    async move {
                        seen_get.lock().unwrap().push(format!(
                            "GET:{}",
                            headers
                                .get(header::AUTHORIZATION)
                                .and_then(|value| value.to_str().ok())
                                .unwrap_or("")
                        ));
                        Json(serde_json::json!({
                            "success": true,
                            "data": {"symbols": ["AAPL"]},
                            "error": null,
                            "meta": {"timestamp": "2026-07-27T00:00:00Z"}
                        }))
                    }
                })
                .post(
                    move |headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>| {
                        let seen_post = Arc::clone(&seen_post);
                        async move {
                            seen_post.lock().unwrap().push(format!(
                                "POST:{}:{}",
                                headers
                                    .get(header::AUTHORIZATION)
                                    .and_then(|value| value.to_str().ok())
                                    .unwrap_or(""),
                                body["symbol"].as_str().unwrap_or("")
                            ));
                            Json(serde_json::json!({
                                "success": true,
                                "data": {"symbols": ["AAPL", "MSFT"]},
                                "error": null
                            }))
                        }
                    },
                ),
            )
            .route(
                "/api/users/watchlist/{symbol}",
                delete(
                    move |headers: axum::http::HeaderMap,
                          axum::extract::Path(symbol): axum::extract::Path<String>| {
                        let seen_delete = Arc::clone(&seen_delete);
                        async move {
                            seen_delete.lock().unwrap().push(format!(
                                "DELETE:{}:{}",
                                headers
                                    .get(header::AUTHORIZATION)
                                    .and_then(|value| value.to_str().ok())
                                    .unwrap_or(""),
                                symbol
                            ));
                            Json(serde_json::json!({
                                "success": true,
                                "data": {"symbols": []},
                                "error": null
                            }))
                        }
                    },
                ),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, upstream).await.unwrap();
        });

        let (mut state, token) = valid_frontend_session();
        state.wallet = Arc::new(ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(1),
        }));
        let app = build_app(state.clone());

        let unauthenticated = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/users/watchlist")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

        let cookie = format!("epsx.frontend.access_token={token}");
        let cross_site = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/users/watchlist")
                    .header(header::COOKIE, &cookie)
                    .header(header::HOST, "epsx.test")
                    .header(header::ORIGIN, "https://evil.test")
                    .header("sec-fetch-site", "cross-site")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"symbol":"MSFT"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cross_site.status(), StatusCode::FORBIDDEN);

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/users/watchlist")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(get_response.into_body(), 16 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(get_body["data"]["symbols"], serde_json::json!(["AAPL"]));

        let post_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/users/watchlist")
                    .header(header::COOKIE, &cookie)
                    .header(header::HOST, "epsx.test")
                    .header(header::ORIGIN, "https://epsx.test")
                    .header("sec-fetch-site", "same-origin")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"symbol":"msft"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(post_response.status(), StatusCode::OK);

        let invalid_delete = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/users/watchlist/%2E%2E")
                    .header(header::COOKIE, &cookie)
                    .header(header::HOST, "epsx.test")
                    .header(header::ORIGIN, "https://epsx.test")
                    .header("sec-fetch-site", "same-origin")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            matches!(
                invalid_delete.status(),
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND
            ),
            "{}",
            invalid_delete.status()
        );

        let delete_response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/users/watchlist/AAPL")
                    .header(header::COOKIE, &cookie)
                    .header(header::HOST, "epsx.test")
                    .header(header::ORIGIN, "https://epsx.test")
                    .header("sec-fetch-site", "same-origin")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::OK);

        let seen = seen.lock().unwrap();
        assert!(seen.contains(&format!("GET:Bearer {token}")));
        assert!(seen.contains(&format!("POST:Bearer {token}:MSFT")));
        assert!(seen.contains(&format!("DELETE:Bearer {token}:AAPL")));
        assert!(seen.iter().all(|entry| !entry.contains("evil")));
    }

    #[tokio::test]
    async fn watchlist_bff_distinguishes_malformed_and_unavailable_upstreams() {
        let malformed_upstream = Router::new().route(
            "/api/users/watchlist",
            get(|| async {
                Json(serde_json::json!({
                    "success": true,
                    "data": {"symbols": "AAPL"}
                }))
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, malformed_upstream).await.unwrap();
        });

        let (mut state, token) = valid_frontend_session();
        state.wallet = Arc::new(ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{address}"),
            timeout: Duration::from_secs(1),
        }));
        let malformed = build_app(state)
            .oneshot(
                Request::builder()
                    .uri("/api/users/watchlist")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(malformed.status(), StatusCode::BAD_GATEWAY);
        assert!(malformed.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("no-store"));

        let closed_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let closed_address = closed_listener.local_addr().unwrap();
        drop(closed_listener);
        let (mut state, token) = valid_frontend_session();
        state.wallet = Arc::new(ServiceClient::new(epsx_client::ClientConfig {
            base_url: format!("http://{closed_address}"),
            timeout: Duration::from_millis(100),
        }));
        let unavailable = build_app(state)
            .oneshot(
                Request::builder()
                    .uri("/api/users/watchlist")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(unavailable.headers()[header::CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("no-store"));
    }

    #[tokio::test]
    async fn pricing_redirect_status_and_target_are_preserved() {
        let response = request(Method::GET, "/pricing?ref=test").await;
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(response.headers()[header::LOCATION], "/plans?ref=test");
    }

    #[tokio::test]
    async fn offline_shell_explicitly_declares_public_cache_contract() {
        let response = request(Method::GET, "/offline").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=0, must-revalidate"
        );
        assert_eq!(
            response.headers()["x-epsx-public-cache"],
            "offline-shell-v1"
        );
        assert!(response.headers().get(header::SET_COOKIE).is_none());

        let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(html.contains("/runtime/epsx_browser_runtime_bootstrap.js"));
        assert!(!html.contains("/service-worker.js"));
        assert!(html.contains("Open this offline help page"));
        assert!(!html.contains("View cached notifications"));
        assert!(!html.contains("Your data will sync"));
    }
}
