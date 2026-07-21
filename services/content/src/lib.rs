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

const CONTENT_MANAGE_PERMISSION: &str = "admin:content:manage";

#[derive(Debug, Error)]
pub enum ContentConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, ContentConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-content/1")
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
    ContentAdmin,
    EditorIdentityRequired,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if path.contains('%') || path.contains('\\') {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/content/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return AccessPolicy::Blocked;
    }

    match (method, segments.as_slice()) {
        (&Method::GET, ["pages", slug, "render"]) if safe_dynamic_segment(slug) => {
            AccessPolicy::Public
        }
        (
            &Method::GET,
            ["themes" | "blocks" | "navigation" | "site" | "news" | "plans" | "rankings"],
        ) => AccessPolicy::Public,
        (&Method::GET, ["themes" | "blocks" | "news" | "portfolio", value])
            if safe_dynamic_segment(value) =>
        {
            AccessPolicy::Public
        }
        (&Method::GET, ["pages"]) | (&Method::POST, ["pages" | "themes"]) => {
            AccessPolicy::ContentAdmin
        }
        (&Method::GET | &Method::PUT, ["pages", slug]) if safe_dynamic_segment(slug) => {
            AccessPolicy::ContentAdmin
        }
        (&Method::POST, ["pages", id, "publish"]) if safe_dynamic_segment(id) => {
            AccessPolicy::ContentAdmin
        }
        (&Method::PUT, ["themes", id]) if safe_dynamic_segment(id) => AccessPolicy::ContentAdmin,
        (&Method::POST, ["edit", "start" | "commit"]) | (&Method::GET, ["edit", "sessions"]) => {
            AccessPolicy::EditorIdentityRequired
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
        AccessPolicy::ContentAdmin => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(CONTENT_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::EditorIdentityRequired | AccessPolicy::Blocked => {
            return StatusCode::NOT_FOUND.into_response();
        }
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
    use axum::body::Body;
    use epsx_service_auth::{VerifiedPrincipal, VerifyError, FRONTEND_AUDIENCE};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (audience, permissions) = match token {
                "frontend-content" => (FRONTEND_AUDIENCE, vec![CONTENT_MANAGE_PERMISSION.into()]),
                "admin" => (ADMIN_AUDIENCE, vec![]),
                "admin-content" => (ADMIN_AUDIENCE, vec![CONTENT_MANAGE_PERMISSION.into()]),
                "admin-wildcard" => (ADMIN_AUDIENCE, vec!["admin:content:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "other-audience" => ("epsx-other", vec![CONTENT_MANAGE_PERMISSION.into()]),
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
    }

    fn app() -> (Router, Arc<Downstream>) {
        let downstream = Arc::new(Downstream::default());
        let observed = downstream.clone();
        let router = Router::new().fallback(move |request: Request| {
            let observed = observed.clone();
            async move {
                observed.hits.fetch_add(1, Ordering::SeqCst);
                if request.headers().contains_key(header::AUTHORIZATION) {
                    observed.authorization_seen.fetch_add(1, Ordering::SeqCst);
                }
                if request.headers().contains_key("x-user-id")
                    || request.headers().contains_key("x-permissions")
                {
                    observed
                        .spoofed_identity_seen
                        .fetch_add(1, Ordering::SeqCst);
                }
                StatusCode::OK
            }
        });
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
    async fn exact_public_allowlist_is_anonymous() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::GET, "/health"),
            (Method::HEAD, "/health"),
            (Method::GET, "/api/v1/content/pages/welcome/render"),
            (Method::GET, "/api/v1/content/themes"),
            (Method::GET, "/api/v1/content/themes/theme-id"),
            (Method::GET, "/api/v1/content/blocks"),
            (Method::GET, "/api/v1/content/blocks/hero"),
            (Method::GET, "/api/v1/content/navigation"),
            (Method::GET, "/api/v1/content/site"),
            (Method::GET, "/api/v1/content/news"),
            (Method::GET, "/api/v1/content/news/launch"),
            (Method::GET, "/api/v1/content/plans"),
            (Method::GET, "/api/v1/content/rankings"),
            (Method::GET, "/api/v1/content/portfolio/0xabc"),
        ] {
            assert_eq!(
                status(&app, request(method, path, None)).await,
                StatusCode::OK
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 14);
    }

    #[tokio::test]
    async fn public_requests_strip_bearer_and_spoofable_identity_headers() {
        let (app, downstream) = app();
        let mut public = request(
            Method::GET,
            "/api/v1/content/navigation",
            Some("admin-content"),
        );
        public
            .headers_mut()
            .insert("x-user-id", "attacker".parse().unwrap());
        public
            .headers_mut()
            .insert("x-permissions", CONTENT_MANAGE_PERMISSION.parse().unwrap());
        assert_eq!(status(&app, public).await, StatusCode::OK);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn content_admin_routes_follow_canonical_backend_permission_grammar() {
        let (app, downstream) = app();
        let routes = [
            (Method::GET, "/api/v1/content/pages/article"),
            (Method::PUT, "/api/v1/content/pages/article"),
            (Method::POST, "/api/v1/content/pages"),
            (Method::GET, "/api/v1/content/pages"),
            (Method::POST, "/api/v1/content/pages/page-id/publish"),
            (Method::POST, "/api/v1/content/themes"),
            (Method::PUT, "/api/v1/content/themes/theme-id"),
        ];
        for bearer in ["admin-content", "admin-wildcard"] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(bearer))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 14);
    }

    #[tokio::test]
    async fn wrong_audience_missing_permission_and_spoof_headers_are_denied() {
        let (app, downstream) = app();
        for (bearer, expected) in [
            (None, StatusCode::UNAUTHORIZED),
            (Some("invalid"), StatusCode::UNAUTHORIZED),
            (Some("admin"), StatusCode::FORBIDDEN),
            (Some("admin-invalid-wildcard"), StatusCode::FORBIDDEN),
            (Some("frontend-content"), StatusCode::FORBIDDEN),
            (Some("other-audience"), StatusCode::FORBIDDEN),
        ] {
            assert_eq!(
                status(&app, request(Method::GET, "/api/v1/content/pages", bearer),).await,
                expected
            );
        }

        let mut spoofed = request(Method::GET, "/api/v1/content/pages", Some("admin"));
        spoofed
            .headers_mut()
            .insert("x-permissions", CONTENT_MANAGE_PERMISSION.parse().unwrap());
        assert_eq!(status(&app, spoofed).await, StatusCode::FORBIDDEN);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn editor_identity_routes_remain_fail_closed() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::POST, "/api/v1/content/edit/start"),
            (Method::POST, "/api/v1/content/edit/commit"),
            (Method::GET, "/api/v1/content/edit/sessions"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-content"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn strict_arity_unknown_and_unapproved_methods_fail_before_downstream() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::GET, "/api/v1/content/pages/a/b/render"),
            (Method::GET, "/api/v1/content/themes/a/b"),
            (Method::GET, "/api/v1/content/blocks/a/b"),
            (Method::GET, "/api/v1/content/news/a/b"),
            (Method::GET, "/api/v1/content/portfolio/a/b"),
            (Method::GET, "/api/v1/content/pages/%2e%2e/render"),
            (Method::POST, "/api/v1/content/navigation"),
            (Method::DELETE, "/api/v1/content/pages/article"),
            (Method::GET, "/api/v1/content/unknown"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-content"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_auth_requires_https_non_local_identity_urls() {
        assert!(build_auth_verifier(
            "http://issuer.example",
            "https://issuer.example/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://issuer.localhost",
            "https://issuer.example/.well-known/jwks.json",
            true,
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://issuer.example",
            "https://127.0.0.1/.well-known/jwks.json",
            true,
        )
        .is_err());
    }
}
