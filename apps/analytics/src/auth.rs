use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Json, Router,
};
use epsx::web::analytics::eps::cache::AnalyticsWalletContext;
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, ADMIN_AUDIENCE,
    FRONTEND_AUDIENCE,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarketAuthConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, MarketAuthConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-market-analytics/1")
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
    OptionalAuthenticated,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    match (method, path) {
        (&Method::GET | &Method::HEAD, "/health") => AccessPolicy::Public,
        (&Method::GET, "/api/analytics/filters")
        | (&Method::GET, "/api/analytics/countries")
        | (&Method::GET, "/api/analytics/available-countries")
        | (&Method::GET, "/api/analytics/sectors") => AccessPolicy::Public,
        (&Method::GET, "/api/analytics/rankings") => AccessPolicy::OptionalAuthenticated,
        _ => AccessPolicy::Blocked,
    }
}

async fn authorize_request(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    strip_spoofable_identity_headers(request.headers_mut());

    let policy = classify(request.method(), request.uri().path());
    match policy {
        AccessPolicy::Public => {
            request.headers_mut().remove(header::AUTHORIZATION);
        }
        AccessPolicy::OptionalAuthenticated => {
            if request.headers().contains_key(header::AUTHORIZATION) {
                let principal =
                    match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                        Ok(principal) => principal,
                        Err(_) => return ranking_auth_error(StatusCode::UNAUTHORIZED),
                    };
                if principal.audience != FRONTEND_AUDIENCE && principal.audience != ADMIN_AUDIENCE {
                    return ranking_auth_error(StatusCode::FORBIDDEN);
                }
                let wallet_context =
                    AnalyticsWalletContext::new(principal.wallet_address.to_lowercase());
                request.extensions_mut().insert(wallet_context);
                request.extensions_mut().insert(principal);
                request.headers_mut().remove(header::AUTHORIZATION);
            }
        }
        AccessPolicy::Blocked => return StatusCode::NOT_FOUND.into_response(),
    }

    let mut response = next.run(request).await;
    if policy == AccessPolicy::OptionalAuthenticated {
        apply_private_ranking_cache_policy(&mut response);
    }
    response
}

fn ranking_auth_error(status: StatusCode) -> Response {
    let code = if status == StatusCode::FORBIDDEN {
        "forbidden"
    } else {
        "unauthorized"
    };
    let mut response = (status, Json(serde_json::json!({ "error": code }))).into_response();
    apply_private_ranking_cache_policy(&mut response);
    response
}

fn apply_private_ranking_cache_policy(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
        .headers_mut()
        .append(header::VARY, HeaderValue::from_static("Authorization"));
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
        body::{to_bytes, Body},
        extract::Extension,
        routing::get,
    };
    use epsx_service_auth::{VerifiedPrincipal, VerifyError};
    use serde_json::Value;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    #[derive(Clone, Copy)]
    enum Verification {
        Frontend,
        Admin,
        Unsupported,
        Rejected,
    }

    struct FakeVerifier {
        result: Verification,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, _token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let audience = match self.result {
                Verification::Frontend => FRONTEND_AUDIENCE,
                Verification::Admin => ADMIN_AUDIENCE,
                Verification::Unsupported => "other-service",
                Verification::Rejected => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0xAbC".into(),
                wallet_address: "0xAbC".into(),
                audience: audience.into(),
                permissions: Vec::new(),
            })
        }
    }

    #[derive(Clone)]
    struct ProbeState {
        calls: Arc<AtomicUsize>,
    }

    async fn probe(
        State(state): State<ProbeState>,
        principal: Option<Extension<VerifiedPrincipal>>,
        wallet: Option<Extension<AnalyticsWalletContext>>,
        headers: HeaderMap,
    ) -> Json<Value> {
        state.calls.fetch_add(1, Ordering::SeqCst);
        Json(serde_json::json!({
            "audience": principal.map(|Extension(value)| value.audience),
            "wallet": wallet.map(|Extension(value)| value.wallet_address().to_string()),
            "authorization": headers.contains_key(header::AUTHORIZATION),
            "spoofed_wallet": headers.contains_key("x-wallet-address")
        }))
    }

    fn test_router(verification: Verification) -> (Router, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let verifier_calls = Arc::new(AtomicUsize::new(0));
        let handler_calls = Arc::new(AtomicUsize::new(0));
        let verifier: Arc<dyn AccessTokenVerifier> = Arc::new(FakeVerifier {
            result: verification,
            calls: verifier_calls.clone(),
        });
        let probe_state = ProbeState {
            calls: handler_calls.clone(),
        };
        let router = Router::new()
            .route("/health", get(probe))
            .route("/api/analytics/rankings", get(probe).post(probe))
            .route("/api/analytics/filters", get(probe))
            .route("/api/analytics/countries", get(probe))
            .route("/api/analytics/available-countries", get(probe))
            .route("/api/analytics/sectors", get(probe))
            .route("/v1/rankings/stream", get(probe))
            .route("/unknown", axum::routing::post(probe))
            .with_state(probe_state);
        (
            protect_router(router, verifier),
            verifier_calls,
            handler_calls,
        )
    }

    async fn request(router: Router, method: Method, path: &str, bearer: Option<&str>) -> Response {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(bearer) = bearer {
            builder = builder.header(header::AUTHORIZATION, bearer);
        }
        router
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn json(response: Response) -> Value {
        serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap()).unwrap()
    }

    fn assert_private_ranking_cache_policy(response: &Response) {
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL),
            Some(&HeaderValue::from_static("private, no-store"))
        );
        assert!(response
            .headers()
            .get_all(header::VARY)
            .iter()
            .any(|value| value == "Authorization"));
    }

    #[test]
    fn exact_route_and_method_inventory_is_closed() {
        for path in [
            "/health",
            "/api/analytics/filters",
            "/api/analytics/countries",
            "/api/analytics/available-countries",
            "/api/analytics/sectors",
        ] {
            assert_eq!(classify(&Method::GET, path), AccessPolicy::Public);
        }
        assert_eq!(classify(&Method::HEAD, "/health"), AccessPolicy::Public);
        assert_eq!(
            classify(&Method::GET, "/api/analytics/rankings"),
            AccessPolicy::OptionalAuthenticated
        );
        for (method, path) in [
            (Method::POST, "/api/analytics/rankings"),
            (Method::HEAD, "/api/analytics/rankings"),
            (Method::GET, "/rankings"),
            (Method::GET, "/api/public/analytics/rankings"),
            (Method::GET, "/api/v1/analytics/rankings"),
            (Method::GET, "/v1/rankings/stream"),
            (Method::GET, "/api/analytics/rankings/"),
            (Method::GET, "/api/analytics/%72ankings"),
        ] {
            assert_eq!(classify(&method, path), AccessPolicy::Blocked);
        }
    }

    #[tokio::test]
    async fn health_and_metadata_are_public_and_credential_omitting() {
        for path in [
            "/health",
            "/api/analytics/filters",
            "/api/analytics/countries",
            "/api/analytics/available-countries",
            "/api/analytics/sectors",
        ] {
            let (router, verifier, _) = test_router(Verification::Rejected);
            let response = request(router, Method::GET, path, Some("Bearer ignored")).await;
            assert_eq!(response.status(), StatusCode::OK);
            let body = json(response).await;
            assert_eq!(body["authorization"], false);
            assert_eq!(body["audience"], Value::Null);
            assert_eq!(verifier.load(Ordering::SeqCst), 0);
        }

        let (router, verifier, handler) = test_router(Verification::Rejected);
        let response = request(router, Method::HEAD, "/health", Some("Bearer ignored")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(verifier.load(Ordering::SeqCst), 0);
        assert_eq!(handler.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn rankings_without_credentials_remains_anonymous_free_tier_input() {
        let (router, verifier, handler) = test_router(Verification::Rejected);
        let response = request(router, Method::GET, "/api/analytics/rankings", None).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_private_ranking_cache_policy(&response);
        let body = json(response).await;
        assert_eq!(body["wallet"], Value::Null);
        assert_eq!(body["audience"], Value::Null);
        assert_eq!(verifier.load(Ordering::SeqCst), 0);
        assert_eq!(handler.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn exact_frontend_and_admin_principals_propagate_verified_wallet() {
        for (verification, expected_audience) in [
            (Verification::Frontend, FRONTEND_AUDIENCE),
            (Verification::Admin, ADMIN_AUDIENCE),
        ] {
            let (router, verifier, handler) = test_router(verification);
            let response = request(
                router,
                Method::GET,
                "/api/analytics/rankings",
                Some("Bearer signed-token"),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_private_ranking_cache_policy(&response);
            let body = json(response).await;
            assert_eq!(body["audience"], expected_audience);
            assert_eq!(body["wallet"], "0xabc");
            assert_eq!(body["authorization"], false);
            assert_eq!(verifier.load(Ordering::SeqCst), 1);
            assert_eq!(handler.load(Ordering::SeqCst), 1);
        }
    }

    #[tokio::test]
    async fn invalid_and_unsupported_credentials_fail_before_handler() {
        for (verification, expected) in [
            (Verification::Rejected, StatusCode::UNAUTHORIZED),
            (Verification::Unsupported, StatusCode::FORBIDDEN),
        ] {
            let (router, verifier, handler) = test_router(verification);
            let response = request(
                router,
                Method::GET,
                "/api/analytics/rankings",
                Some("Bearer candidate"),
            )
            .await;
            assert_eq!(response.status(), expected);
            assert_private_ranking_cache_policy(&response);
            assert_eq!(verifier.load(Ordering::SeqCst), 1);
            assert_eq!(handler.load(Ordering::SeqCst), 0);
        }
    }

    #[tokio::test]
    async fn malformed_and_duplicate_bearers_fail_before_verifier_or_handler() {
        let cases = [vec!["Basic opaque"], vec!["Bearer first", "Bearer second"]];
        for values in cases {
            let (router, verifier, handler) = test_router(Verification::Frontend);
            let mut request = Request::builder()
                .uri("/api/analytics/rankings")
                .body(Body::empty())
                .unwrap();
            for value in values {
                request.headers_mut().append(
                    header::AUTHORIZATION,
                    value.parse().expect("static authorization value"),
                );
            }
            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_private_ranking_cache_policy(&response);
            assert_eq!(verifier.load(Ordering::SeqCst), 0);
            assert_eq!(handler.load(Ordering::SeqCst), 0);
        }
    }

    #[tokio::test]
    async fn spoofable_identity_headers_are_removed_before_dispatch() {
        let (router, verifier, handler) = test_router(Verification::Frontend);
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/analytics/rankings")
                    .header(header::AUTHORIZATION, "Bearer signed-token")
                    .header("x-wallet-address", "0xattacker")
                    .header("x-permissions", "*:*:*")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json(response).await;
        assert_eq!(body["wallet"], "0xabc");
        assert_eq!(body["spoofed_wallet"], false);
        assert_eq!(verifier.load(Ordering::SeqCst), 1);
        assert_eq!(handler.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn route_drift_and_stream_are_404_before_verifier_or_handler() {
        for (method, path) in [
            (Method::POST, "/api/analytics/rankings"),
            (Method::GET, "/rankings"),
            (Method::GET, "/api/v1/analytics/rankings"),
            (Method::GET, "/v1/rankings/stream"),
            (Method::POST, "/unknown"),
        ] {
            let (router, verifier, handler) = test_router(Verification::Frontend);
            let response = request(router, method, path, Some("Bearer signed-token")).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            assert_eq!(verifier.load(Ordering::SeqCst), 0);
            assert_eq!(handler.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn production_verifier_rejects_local_or_plain_http_authorities() {
        for (issuer, jwks) in [
            (
                "http://identity.internal",
                "http://identity.internal/.well-known/jwks.json",
            ),
            (
                "https://localhost",
                "https://localhost/.well-known/jwks.json",
            ),
        ] {
            assert!(build_auth_verifier(issuer, jwks, true).is_err());
        }
    }
}
