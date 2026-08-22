use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, ADMIN_AUDIENCE,
    FRONTEND_AUDIENCE,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

pub const USERS_READ_PERMISSION: &str = "admin:users:read";
pub const USERS_CREATE_PERMISSION: &str = "admin:users:create";
pub const USERS_UPDATE_PERMISSION: &str = "admin:users:update";
pub const USERS_DELETE_PERMISSION: &str = "admin:users:delete";

#[derive(Debug, Error)]
pub enum IdentityConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, IdentityConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-identity/1")
        .build()?;
    let config =
        JwksVerifierConfig::new(issuer, jwks_url, Duration::from_secs(5 * 60), production)?;
    Ok(Arc::new(JwksVerifier::new(config, client)))
}

#[derive(Clone)]
struct AuthState {
    verifier: Arc<dyn AccessTokenVerifier>,
}

/// Build the intentionally narrow A2.3i identity surface.
///
/// Health is live. All unsafe lifecycle routes are hidden before dispatch.
/// `/auth/me` and administrative user routes prove only their authentication
/// boundary, then deliberately return 404 before candidate identity, body, or
/// persistence semantics can run.
pub fn build_router(verifier: Arc<dyn AccessTokenVerifier>) -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .route("/api/v1/identity/auth/challenge", post(not_found))
        .route("/api/v1/identity/auth/siwe", post(not_found))
        .route("/api/v1/identity/auth/refresh", post(not_found))
        .route("/api/v1/identity/auth/me", get(not_found))
        .route("/api/v1/identity/auth/demo", post(not_found))
        .route("/api/v1/identity/users", get(not_found).post(not_found))
        .route(
            "/api/v1/identity/users/{id}",
            get(not_found).put(not_found).delete(not_found),
        )
        .fallback(not_found);
    protect_router(router, verifier)
}

pub fn protect_router(router: Router, verifier: Arc<dyn AccessTokenVerifier>) -> Router {
    router.layer(middleware::from_fn_with_state(
        AuthState { verifier },
        authorize_request,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    UnsafeLifecycle,
    AuthenticatedCandidate,
    AdminPermission(&'static str),
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if !normalized_path(path) {
        return AccessPolicy::Blocked;
    }
    match (method, path) {
        (&Method::GET | &Method::HEAD, "/health") => AccessPolicy::Public,
        (&Method::POST, "/api/v1/identity/auth/challenge")
        | (&Method::POST, "/api/v1/identity/auth/siwe")
        | (&Method::POST, "/api/v1/identity/auth/refresh")
        | (&Method::POST, "/api/v1/identity/auth/demo") => AccessPolicy::UnsafeLifecycle,
        (&Method::GET, "/api/v1/identity/auth/me") => AccessPolicy::AuthenticatedCandidate,
        (&Method::GET, "/api/v1/identity/users") => {
            AccessPolicy::AdminPermission(USERS_READ_PERMISSION)
        }
        (&Method::POST, "/api/v1/identity/users") => {
            AccessPolicy::AdminPermission(USERS_CREATE_PERMISSION)
        }
        _ => classify_user_detail(method, path),
    }
}

fn classify_user_detail(method: &Method, path: &str) -> AccessPolicy {
    let Some(id) = path.strip_prefix("/api/v1/identity/users/") else {
        return AccessPolicy::Blocked;
    };
    if !safe_dynamic_segment(id) {
        return AccessPolicy::Blocked;
    }
    match *method {
        Method::GET => AccessPolicy::AdminPermission(USERS_READ_PERMISSION),
        Method::PUT => AccessPolicy::AdminPermission(USERS_UPDATE_PERMISSION),
        Method::DELETE => AccessPolicy::AdminPermission(USERS_DELETE_PERMISSION),
        _ => AccessPolicy::Blocked,
    }
}

fn normalized_path(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= 2048
        && !path.contains('%')
        && !path.contains('\\')
        && !path.contains("//")
        && !path.ends_with('/')
}

fn safe_dynamic_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && !matches!(
            segment,
            "." | ".."
                | "health"
                | "auth"
                | "challenge"
                | "siwe"
                | "refresh"
                | "me"
                | "demo"
                | "users"
        )
}

async fn authorize_request(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    strip_spoofable_identity_headers(request.headers_mut());
    match classify(request.method(), request.uri().path()) {
        AccessPolicy::Public => {
            request.headers_mut().remove(header::AUTHORIZATION);
        }
        AccessPolicy::UnsafeLifecycle | AccessPolicy::Blocked => {
            return StatusCode::NOT_FOUND.into_response();
        }
        AccessPolicy::AuthenticatedCandidate => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != FRONTEND_AUDIENCE && principal.audience != ADMIN_AUDIENCE {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::AdminPermission(required) => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE || !principal.has_permission(required) {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
    }
    next.run(request).await
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

fn auth_error(status: StatusCode) -> Response {
    let code = if status == StatusCode::FORBIDDEN {
        "forbidden"
    } else {
        "unauthorized"
    };
    (status, Json(serde_json::json!({ "error": code }))).into_response()
}

fn strip_spoofable_identity_headers(headers: &mut HeaderMap) {
    let names: Vec<HeaderName> = headers
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::{body::Body, routing::any};
    use epsx_service_auth::{VerifiedPrincipal, VerifyError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    #[derive(Default)]
    struct FakeVerifier {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (audience, permissions) = match token {
                "frontend" => (FRONTEND_AUDIENCE, vec![]),
                "admin" => (ADMIN_AUDIENCE, vec![]),
                "admin-read" => (ADMIN_AUDIENCE, vec![USERS_READ_PERMISSION.into()]),
                "admin-create" => (ADMIN_AUDIENCE, vec![USERS_CREATE_PERMISSION.into()]),
                "admin-update" => (ADMIN_AUDIENCE, vec![USERS_UPDATE_PERMISSION.into()]),
                "admin-delete" => (ADMIN_AUDIENCE, vec![USERS_DELETE_PERMISSION.into()]),
                "admin-resource-wildcard" => (ADMIN_AUDIENCE, vec!["admin:users:*".into()]),
                "admin-domain-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-global-wildcard" => (ADMIN_AUDIENCE, vec!["*:*".into()]),
                "frontend-read" => (FRONTEND_AUDIENCE, vec![USERS_READ_PERMISSION.into()]),
                "other" => ("epsx-other", vec![USERS_READ_PERMISSION.into()]),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0x1111111111111111111111111111111111111111".into(),
                wallet_address: "0x1111111111111111111111111111111111111111".into(),
                audience: audience.into(),
                permissions,
            })
        }
    }

    #[derive(Default)]
    struct Downstream {
        hits: AtomicUsize,
        authorization_seen: AtomicUsize,
        spoofed_identity_seen: AtomicUsize,
        principal_seen: AtomicUsize,
    }

    fn observed_app() -> (Router, Arc<Downstream>, Arc<FakeVerifier>) {
        let downstream = Arc::new(Downstream::default());
        let observed = downstream.clone();
        let router = Router::new().fallback(any(move |request: Request| {
            let observed = observed.clone();
            async move {
                observed.hits.fetch_add(1, Ordering::SeqCst);
                if request.headers().contains_key(header::AUTHORIZATION) {
                    observed.authorization_seen.fetch_add(1, Ordering::SeqCst);
                }
                if request.headers().contains_key("x-user-id")
                    || request.headers().contains_key("x-wallet-address")
                    || request.headers().contains_key("x-permissions")
                {
                    observed
                        .spoofed_identity_seen
                        .fetch_add(1, Ordering::SeqCst);
                }
                if request.extensions().get::<VerifiedPrincipal>().is_some() {
                    observed.principal_seen.fetch_add(1, Ordering::SeqCst);
                }
                StatusCode::OK
            }
        }));
        let verifier = Arc::new(FakeVerifier::default());
        (
            protect_router(router, verifier.clone()),
            downstream,
            verifier,
        )
    }

    fn request(method: Method, path: &str, bearer: Option<&str>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder().method(method).uri(path);
        if let Some(bearer) = bearer {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
        }
        builder.body(Body::empty()).unwrap()
    }

    fn json_request(method: Method, path: &str, bearer: Option<&str>) -> axum::http::Request<Body> {
        let mut request = request(method, path, bearer);
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            "application/json".parse().expect("valid content type"),
        );
        *request.body_mut() = Body::from("{ definitely-not-json");
        request
    }

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn health_is_the_only_anonymous_surface_and_strips_untrusted_headers() {
        let (app, downstream, verifier) = observed_app();
        for method in [Method::GET, Method::HEAD] {
            let mut health = request(method, "/health", Some("admin-read"));
            health
                .headers_mut()
                .insert("x-user-id", "attacker".parse().unwrap());
            health
                .headers_mut()
                .insert("x-wallet-address", "attacker".parse().unwrap());
            health
                .headers_mut()
                .insert("x-permissions", "*:*".parse().unwrap());
            assert_eq!(status(&app, health).await, StatusCode::OK);
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 2);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unsafe_lifecycle_routes_are_hidden_before_auth_body_or_handler() {
        let routes = [
            "/api/v1/identity/auth/challenge",
            "/api/v1/identity/auth/siwe",
            "/api/v1/identity/auth/refresh",
            "/api/v1/identity/auth/demo",
        ];
        let hits = Arc::new(AtomicUsize::new(0));
        let router = routes.iter().fold(Router::new(), |router, path| {
            let hits = hits.clone();
            router.route(
                path,
                post(move |Json(_): Json<serde_json::Value>| {
                    let hits = hits.clone();
                    async move {
                        hits.fetch_add(1, Ordering::SeqCst);
                        StatusCode::OK
                    }
                }),
            )
        });
        let verifier = Arc::new(FakeVerifier::default());
        let app = protect_router(router, verifier.clone());
        for path in routes {
            for bearer in [None, Some("invalid"), Some("admin-read")] {
                assert_eq!(
                    status(&app, json_request(Method::POST, path, bearer)).await,
                    StatusCode::NOT_FOUND,
                    "POST {path}"
                );
            }
        }
        assert_eq!(hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn auth_me_accepts_only_the_two_bff_audiences_then_remains_hidden() {
        let verifier = Arc::new(FakeVerifier::default());
        let app = build_router(verifier.clone());
        assert_eq!(
            status(&app, request(Method::GET, "/api/v1/identity/auth/me", None)).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/identity/auth/me", Some("invalid"))
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/identity/auth/me", Some("other"))
            )
            .await,
            StatusCode::FORBIDDEN
        );
        for bearer in ["frontend", "admin"] {
            assert_eq!(
                status(
                    &app,
                    request(Method::GET, "/api/v1/identity/auth/me", Some(bearer))
                )
                .await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn user_routes_require_admin_audience_and_canonical_permissions() {
        let cases = [
            (Method::GET, "/api/v1/identity/users", "admin-read"),
            (Method::POST, "/api/v1/identity/users", "admin-create"),
            (
                Method::GET,
                "/api/v1/identity/users/00000000-0000-0000-0000-000000000001",
                "admin-read",
            ),
            (
                Method::PUT,
                "/api/v1/identity/users/00000000-0000-0000-0000-000000000001",
                "admin-update",
            ),
            (
                Method::DELETE,
                "/api/v1/identity/users/00000000-0000-0000-0000-000000000001",
                "admin-delete",
            ),
        ];
        let app = build_router(Arc::new(FakeVerifier::default()));
        for (method, path, allowed) in cases {
            assert_eq!(
                status(&app, request(method.clone(), path, None)).await,
                StatusCode::UNAUTHORIZED,
                "missing token for {method} {path}"
            );
            for denied in ["invalid", "admin", "frontend-read", "other"] {
                let expected = if denied == "invalid" {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::FORBIDDEN
                };
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(denied))).await,
                    expected,
                    "{denied} for {method} {path}"
                );
            }
            for permitted in [
                allowed,
                "admin-resource-wildcard",
                "admin-domain-wildcard",
                "admin-global-wildcard",
            ] {
                let allowed_request = if matches!(method, Method::POST | Method::PUT) {
                    json_request(method.clone(), path, Some(permitted))
                } else {
                    request(method.clone(), path, Some(permitted))
                };
                assert_eq!(
                    status(&app, allowed_request).await,
                    StatusCode::NOT_FOUND,
                    "canonical permission must reach candidate semantics for {method} {path}"
                );
            }
        }
    }

    #[tokio::test]
    async fn malformed_bearer_forms_never_reach_candidate_handlers() {
        let verifier = Arc::new(FakeVerifier::default());
        let app = build_router(verifier.clone());
        for value in ["bearer frontend", "Bearer", "Bearer ", "Bearer front end"] {
            let request = axum::http::Request::builder()
                .method(Method::GET)
                .uri("/api/v1/identity/auth/me")
                .header(header::AUTHORIZATION, value)
                .body(Body::empty())
                .unwrap();
            assert_eq!(status(&app, request).await, StatusCode::UNAUTHORIZED);
        }
        let duplicate = axum::http::Request::builder()
            .method(Method::GET)
            .uri("/api/v1/identity/auth/me")
            .header(header::AUTHORIZATION, "Bearer frontend")
            .header(header::AUTHORIZATION, "Bearer admin")
            .body(Body::empty())
            .unwrap();
        assert_eq!(status(&app, duplicate).await, StatusCode::UNAUTHORIZED);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn wrong_methods_arities_encoded_reserved_and_unknown_paths_stop_before_auth() {
        let cases = [
            (Method::POST, "/health"),
            (Method::GET, "/health/"),
            (Method::GET, "/api/v1/identity/auth/challenge"),
            (Method::HEAD, "/api/v1/identity/auth/me"),
            (Method::GET, "/api/v1/identity/users/"),
            (Method::GET, "/api/v1/identity/users/users"),
            (Method::GET, "/api/v1/identity/users/auth"),
            (Method::GET, "/api/v1/identity/users/id/extra"),
            (Method::GET, "/api/v1/identity/users/%2e%2e"),
            (Method::GET, "/api/v1/identity//users"),
            (Method::PATCH, "/api/v1/identity/users/id"),
            (Method::GET, "/api/v1/identity/unknown"),
            (Method::GET, "/unknown"),
        ];
        let (app, downstream, verifier) = observed_app();
        for (method, path) in cases {
            assert_eq!(
                status(&app, request(method.clone(), path, Some("admin-read"))).await,
                StatusCode::NOT_FOUND,
                "{method} {path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn classifier_inventory_is_exact_and_every_other_method_is_blocked() {
        let detail = "/api/v1/identity/users/00000000-0000-0000-0000-000000000001";
        let paths = [
            "/health",
            "/api/v1/identity/auth/challenge",
            "/api/v1/identity/auth/siwe",
            "/api/v1/identity/auth/refresh",
            "/api/v1/identity/auth/me",
            "/api/v1/identity/auth/demo",
            "/api/v1/identity/users",
            detail,
        ];
        let methods = [
            Method::GET,
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
            Method::CONNECT,
            Method::TRACE,
            Method::from_bytes(b"BREW").expect("valid extension method"),
        ];

        for path in paths {
            for method in &methods {
                let expected = match (method, path) {
                    (&Method::GET | &Method::HEAD, "/health") => AccessPolicy::Public,
                    (&Method::POST, "/api/v1/identity/auth/challenge")
                    | (&Method::POST, "/api/v1/identity/auth/siwe")
                    | (&Method::POST, "/api/v1/identity/auth/refresh")
                    | (&Method::POST, "/api/v1/identity/auth/demo") => {
                        AccessPolicy::UnsafeLifecycle
                    }
                    (&Method::GET, "/api/v1/identity/auth/me") => {
                        AccessPolicy::AuthenticatedCandidate
                    }
                    (&Method::GET, "/api/v1/identity/users") => {
                        AccessPolicy::AdminPermission(USERS_READ_PERMISSION)
                    }
                    (&Method::POST, "/api/v1/identity/users") => {
                        AccessPolicy::AdminPermission(USERS_CREATE_PERMISSION)
                    }
                    (&Method::GET, value) if value == detail => {
                        AccessPolicy::AdminPermission(USERS_READ_PERMISSION)
                    }
                    (&Method::PUT, value) if value == detail => {
                        AccessPolicy::AdminPermission(USERS_UPDATE_PERMISSION)
                    }
                    (&Method::DELETE, value) if value == detail => {
                        AccessPolicy::AdminPermission(USERS_DELETE_PERMISSION)
                    }
                    _ => AccessPolicy::Blocked,
                };
                assert_eq!(classify(method, path), expected, "{method} {path}");
            }
        }

        for (method, path) in [
            (Method::GET, "/healthz"),
            (Method::HEAD, "/api/v1/identity/auth/me"),
            (Method::POST, "/api/v1/identity/auth/logout"),
            (Method::GET, "/api/v1/identity/users/health"),
            (Method::GET, "/api/v1/identity/users/id/extra"),
            (Method::GET, "/api/v1/identity/users/id%2fextra"),
            (Method::GET, "/api/v1/identity//users"),
            (Method::GET, "/api/v1/identity/unknown"),
            (Method::GET, "/"),
        ] {
            assert_eq!(
                classify(&method, path),
                AccessPolicy::Blocked,
                "{method} {path}"
            );
        }
    }

    #[test]
    fn production_verifier_rejects_insecure_or_local_configuration() {
        assert!(matches!(
            build_auth_verifier(
                "https://identity.example.com",
                "http://identity.example.com/.well-known/jwks.json",
                true,
            ),
            Err(IdentityConfigError::Auth(_))
        ));
        assert!(matches!(
            build_auth_verifier(
                "https://localhost:8443",
                "https://localhost:8443/.well-known/jwks.json",
                true,
            ),
            Err(IdentityConfigError::Auth(_))
        ));
        assert!(build_auth_verifier(
            "https://identity.example.com",
            "https://identity.example.com/.well-known/jwks.json",
            true,
        )
        .is_ok());
    }
}
