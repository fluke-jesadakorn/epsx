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

pub const INDEXER_MANAGE_PERMISSION: &str = "admin:indexer:manage";

#[derive(Debug, Error)]
pub enum IndexerConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, IndexerConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-indexer/1")
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
    UnsafeProjection,
    UnsafeOperatorMutation,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if !normalized_path(path) {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/indexer/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();

    match (method, segments.as_slice()) {
        (&Method::GET, ["status", chain]) if safe_dynamic_segment(chain) => {
            AccessPolicy::UnsafeProjection
        }
        (&Method::GET, ["block", chain, number])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(number) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::GET, ["tx", chain, hash])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(hash) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::GET, ["transfers", chain, address])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(address) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::POST, ["sync"]) => AccessPolicy::UnsafeOperatorMutation,
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
        && !matches!(
            segment,
            "." | ".." | "health" | "status" | "block" | "tx" | "transfers" | "sync"
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
        AccessPolicy::UnsafeProjection => return StatusCode::NOT_FOUND.into_response(),
        AccessPolicy::UnsafeOperatorMutation => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(INDEXER_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }

            // A verified operator still cannot dispatch the current sync
            // handler: it writes number-derived placeholder hashes and uses
            // an in-memory cursor. Keep the mutation unavailable until A12
            // supplies canonical ingestion, a durable lease and replay rules.
            return StatusCode::NOT_FOUND.into_response();
        }
        AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),
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
    use axum::{
        body::Body,
        routing::{any, post},
    };
    use epsx_service_auth::{VerifiedPrincipal, VerifyError, FRONTEND_AUDIENCE};
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
                "admin-none" => (ADMIN_AUDIENCE, vec![]),
                "admin-manage" => (ADMIN_AUDIENCE, vec![INDEXER_MANAGE_PERMISSION.into()]),
                "admin-resource-wildcard" => (ADMIN_AUDIENCE, vec!["admin:indexer:*".into()]),
                "admin-domain-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "frontend-manage" => (FRONTEND_AUDIENCE, vec![INDEXER_MANAGE_PERMISSION.into()]),
                "other-audience" => ("epsx-other", vec![INDEXER_MANAGE_PERMISSION.into()]),
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

    fn app() -> (Router, Arc<Downstream>, Arc<FakeVerifier>) {
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

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn health_is_the_only_anonymous_surface_and_strips_credentials() {
        let (app, downstream, verifier) = app();
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
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 2);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn current_read_projections_fail_closed_before_auth_or_handlers() {
        let (app, downstream, verifier) = app();
        for path in [
            "/api/v1/indexer/status/56",
            "/api/v1/indexer/block/56/100",
            "/api/v1/indexer/tx/56/0xabc",
            "/api/v1/indexer/transfers/56/0xabc",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, Some("admin-manage"))).await,
                StatusCode::NOT_FOUND,
                "{path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn sync_requires_a_verified_exact_admin_audience_and_permission() {
        let (app, downstream, _) = app();
        assert_eq!(
            status(&app, request(Method::POST, "/api/v1/indexer/sync", None)).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(
                &app,
                request(Method::POST, "/api/v1/indexer/sync", Some("invalid"))
            )
            .await,
            StatusCode::UNAUTHORIZED
        );
        for bearer in [
            "admin-none",
            "frontend-manage",
            "other-audience",
            "admin-invalid-wildcard",
        ] {
            assert_eq!(
                status(
                    &app,
                    request(Method::POST, "/api/v1/indexer/sync", Some(bearer))
                )
                .await,
                StatusCode::FORBIDDEN,
                "{bearer}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn authorized_sync_still_fails_closed_before_placeholder_ingestion() {
        let (app, downstream, _) = app();
        for bearer in [
            "admin-manage",
            "admin-resource-wildcard",
            "admin-domain-wildcard",
        ] {
            assert_eq!(
                status(
                    &app,
                    request(Method::POST, "/api/v1/indexer/sync", Some(bearer))
                )
                .await,
                StatusCode::NOT_FOUND,
                "{bearer}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn spoofable_headers_never_reach_health_or_sync_handlers() {
        let (app, downstream, _) = app();
        let mut sync = request(Method::POST, "/api/v1/indexer/sync", Some("admin-manage"));
        sync.headers_mut()
            .insert("x-wallet-address", "0xattacker".parse().unwrap());
        sync.headers_mut()
            .insert("x-permissions", "admin:*:*".parse().unwrap());
        assert_eq!(status(&app, sync).await, StatusCode::NOT_FOUND);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_and_path_arities_fail_before_auth_and_handlers() {
        let (app, downstream, verifier) = app();
        for (method, path) in [
            (Method::GET, "/api/v1/indexer/sync"),
            (Method::PUT, "/api/v1/indexer/sync"),
            (Method::POST, "/api/v1/indexer/status/56"),
            (Method::GET, "/api/v1/indexer/status"),
            (Method::GET, "/api/v1/indexer/status/56/extra"),
            (Method::GET, "/api/v1/indexer/block/56"),
            (Method::GET, "/api/v1/indexer/tx/56/hash/extra"),
            (Method::GET, "/api/v1/indexer/unknown"),
        ] {
            assert_eq!(
                status(&app, request(method, path, Some("admin-manage"))).await,
                StatusCode::NOT_FOUND,
                "{path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn narrowed_runtime_mount_returns_404_instead_of_method_not_allowed() {
        let verifier = Arc::new(FakeVerifier::default());
        let router = Router::new().route("/api/v1/indexer/sync", post(|| async { StatusCode::OK }));
        let app = protect_router(router, verifier.clone());

        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/indexer/sync", Some("admin-manage"))
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/indexer/unknown", Some("admin-manage"))
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn encoded_ambiguous_and_reserved_paths_are_structurally_blocked() {
        for path in [
            "/api/v1/indexer/status/%2e%2e",
            "/api/v1/indexer/status/..",
            "/api/v1/indexer/status/sync",
            "/api/v1/indexer/block/56/status",
            "/api//v1/indexer/status/56",
            "/api/v1/indexer/status/56/",
            "/api/v1/indexer/tx/56\\hash",
        ] {
            assert_eq!(
                classify(&Method::GET, path),
                AccessPolicy::Blocked,
                "{path}"
            );
        }
    }

    #[test]
    fn the_locked_exact_policy_table_is_conservative() {
        assert_eq!(classify(&Method::GET, "/health"), AccessPolicy::Public);
        assert_eq!(
            classify(&Method::GET, "/api/v1/indexer/status/56"),
            AccessPolicy::UnsafeProjection
        );
        assert_eq!(
            classify(&Method::GET, "/api/v1/indexer/block/56/100"),
            AccessPolicy::UnsafeProjection
        );
        assert_eq!(
            classify(&Method::POST, "/api/v1/indexer/sync"),
            AccessPolicy::UnsafeOperatorMutation
        );
        assert_eq!(
            classify(&Method::GET, "/api/v1/indexer/sync"),
            AccessPolicy::Blocked
        );
    }
}
