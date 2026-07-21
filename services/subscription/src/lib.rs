use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, ADMIN_AUDIENCE,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

pub const PLANS_READ_PERMISSION: &str = "admin:plans:read";
pub const PLANS_MANAGE_PERMISSION: &str = "admin:plans:manage";

#[derive(Debug, Error)]
pub enum SubscriptionConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, SubscriptionConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-subscription/1")
        .build()?;
    let config =
        JwksVerifierConfig::new(issuer, jwks_url, Duration::from_secs(5 * 60), production)?;
    Ok(Arc::new(JwksVerifier::new(config, client)))
}

#[derive(Clone)]
struct AuthState {
    verifier: Arc<dyn AccessTokenVerifier>,
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
    PlansRead,
    PlansManage,
    OwnerIdentityUnavailable,
    UnsafeVaultConfig,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if path.contains('%') || path.contains('\\') {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/subscription/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return AccessPolicy::Blocked;
    }

    match (method, segments.as_slice()) {
        (&Method::GET, ["plans"]) => AccessPolicy::PlansRead,
        (&Method::GET, ["plans", id]) if safe_dynamic_segment(id) => AccessPolicy::PlansRead,
        (&Method::POST, ["plans"]) => AccessPolicy::PlansManage,
        (&Method::GET | &Method::POST, ["subscriptions"])
        | (&Method::GET, ["subscriptions", _])
        | (&Method::POST, ["subscriptions", _, "cancel"]) => AccessPolicy::OwnerIdentityUnavailable,
        (&Method::GET, ["vault", chain_id]) if safe_dynamic_segment(chain_id) => {
            AccessPolicy::UnsafeVaultConfig
        }
        _ => AccessPolicy::Blocked,
    }
}

fn safe_dynamic_segment(segment: &str) -> bool {
    !segment.is_empty() && !matches!(segment, "." | "..")
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
        AccessPolicy::PlansRead => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(PLANS_READ_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::PlansManage => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(PLANS_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::OwnerIdentityUnavailable
        | AccessPolicy::UnsafeVaultConfig
        | AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),
    }
    next.run(request).await
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
    use epsx_service_auth::{VerifiedPrincipal, VerifyError, FRONTEND_AUDIENCE};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (audience, permissions) = match token {
                "admin-none" => (ADMIN_AUDIENCE, vec![]),
                "admin-read" => (ADMIN_AUDIENCE, vec![PLANS_READ_PERMISSION.into()]),
                "admin-manage" => (ADMIN_AUDIENCE, vec![PLANS_MANAGE_PERMISSION.into()]),
                "admin-resource-wildcard" => (ADMIN_AUDIENCE, vec!["admin:plans:*".into()]),
                "admin-domain-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-global-wildcard" => (ADMIN_AUDIENCE, vec!["*:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "frontend-read" => (FRONTEND_AUDIENCE, vec![PLANS_READ_PERMISSION.into()]),
                "frontend-manage" => (FRONTEND_AUDIENCE, vec![PLANS_MANAGE_PERMISSION.into()]),
                "other-audience" => ("epsx-other", vec![PLANS_READ_PERMISSION.into()]),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0xabc".into(),
                wallet_address: "0xabc".into(),
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

    fn app() -> (Router, Arc<Downstream>) {
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
        (protect_router(router, Arc::new(FakeVerifier)), downstream)
    }

    fn request(method: Method, path: &str, bearer: Option<&str>) -> axum::http::Request<Body> {
        let mut builder = axum::http::Request::builder().method(method).uri(path);
        if let Some(bearer) = bearer {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"));
        }
        builder.body(Body::empty()).unwrap()
    }

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn health_is_the_only_anonymous_surface_and_strips_credentials() {
        let (app, downstream) = app();
        for method in [Method::GET, Method::HEAD] {
            let mut health = request(method, "/health", Some("admin-manage"));
            health
                .headers_mut()
                .insert("x-user-id", "attacker".parse().unwrap());
            assert_eq!(status(&app, health).await, StatusCode::OK);
        }
        assert_eq!(
            status(&app, request(Method::POST, "/health", None)).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status(&app, request(Method::GET, "/health/", None)).await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 2);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn plan_reads_require_admin_audience_and_read_permission() {
        let (app, downstream) = app();
        for path in [
            "/api/v1/subscription/plans",
            "/api/v1/subscription/plans/plan-id",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, None)).await,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                status(&app, request(Method::GET, path, Some("invalid"))).await,
                StatusCode::UNAUTHORIZED
            );
            for denied in [
                "admin-none",
                "admin-manage",
                "frontend-read",
                "other-audience",
                "admin-invalid-wildcard",
            ] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(denied))).await,
                    StatusCode::FORBIDDEN
                );
            }
            for allowed in [
                "admin-read",
                "admin-resource-wildcard",
                "admin-domain-wildcard",
                "admin-global-wildcard",
            ] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(allowed))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 8);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 8);
    }

    #[tokio::test]
    async fn plan_mutation_requires_admin_audience_and_manage_permission() {
        let (app, downstream) = app();
        let path = "/api/v1/subscription/plans";
        assert_eq!(
            status(&app, request(Method::POST, path, None)).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(&app, request(Method::POST, path, Some("invalid"))).await,
            StatusCode::UNAUTHORIZED
        );
        for denied in [
            "admin-none",
            "admin-read",
            "frontend-manage",
            "other-audience",
            "admin-invalid-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::POST, path, Some(denied))).await,
                StatusCode::FORBIDDEN
            );
        }
        for allowed in [
            "admin-manage",
            "admin-resource-wildcard",
            "admin-domain-wildcard",
            "admin-global-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::POST, path, Some(allowed))).await,
                StatusCode::OK
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 4);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn allowed_admin_requests_strip_spoofable_identity_headers() {
        let (app, downstream) = app();
        let mut req = request(
            Method::GET,
            "/api/v1/subscription/plans",
            Some("admin-read"),
        );
        req.headers_mut()
            .insert("x-user-id", "attacker".parse().unwrap());
        req.headers_mut()
            .insert("x-wallet-address", "0xattacker".parse().unwrap());
        req.headers_mut()
            .insert("x-permissions", "*:*".parse().unwrap());
        assert_eq!(status(&app, req).await, StatusCode::OK);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 1);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn owner_routes_fail_closed_before_verification_or_database_work() {
        let routes = [
            (Method::POST, "/api/v1/subscription/subscriptions"),
            (Method::GET, "/api/v1/subscription/subscriptions"),
            (Method::GET, "/api/v1/subscription/subscriptions/sub-id"),
            (
                Method::POST,
                "/api/v1/subscription/subscriptions/sub-id/cancel",
            ),
        ];
        let (app, downstream) = app();
        for bearer in [
            None,
            Some("invalid"),
            Some("frontend-read"),
            Some("admin-manage"),
        ] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, bearer)).await,
                    StatusCode::NOT_FOUND
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn canned_zero_vault_is_not_exposed() {
        let (app, downstream) = app();
        for bearer in [None, Some("admin-read"), Some("admin-manage")] {
            assert_eq!(
                status(
                    &app,
                    request(Method::GET, "/api/v1/subscription/vault/56", bearer,),
                )
                .await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_paths_and_arity_fail_closed() {
        let cases = [
            (Method::HEAD, "/api/v1/subscription/plans"),
            (Method::PUT, "/api/v1/subscription/plans"),
            (Method::POST, "/api/v1/subscription/plans/plan-id"),
            (Method::GET, "/api/v1/subscription/plans/plan-id/extra"),
            (Method::GET, "/api/v1/subscription/plans/%2e%2e"),
            (Method::GET, "/api/v1/subscription//plans"),
            (
                Method::GET,
                "/api/v1/subscription/subscriptions/sub-id/cancel",
            ),
            (
                Method::POST,
                "/api/v1/subscription/subscriptions/sub-id/cancel/extra",
            ),
            (Method::GET, "/api/v1/subscription/vault/56/extra"),
            (Method::GET, "/api/v1/subscription/unknown"),
            (Method::GET, "/metrics"),
        ];
        let (app, downstream) = app();
        for (method, path) in cases {
            assert_eq!(
                status(&app, request(method, path, Some("admin-domain-wildcard"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_requires_non_local_https_identity_endpoints() {
        assert!(build_auth_verifier(
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/.well-known/jwks.json",
            false,
        )
        .is_ok());
    }
}
