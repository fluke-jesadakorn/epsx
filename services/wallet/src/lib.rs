use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header, HeaderMap, HeaderName, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Json, Router,
};
use epsx_service_auth::{
    authenticate_headers, AccessTokenVerifier, JwksVerifier, JwksVerifierConfig, VerifiedPrincipal,
    ADMIN_AUDIENCE, FRONTEND_AUDIENCE,
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

pub const WALLET_JSON_BODY_LIMIT_BYTES: usize = 8 * 1024;

#[derive(Debug, Error)]
pub enum WalletConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, WalletConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-wallet/1")
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
    router
        .layer(DefaultBodyLimit::max(WALLET_JSON_BODY_LIMIT_BYTES))
        .layer(middleware::from_fn_with_state(
            AuthState { verifier },
            authorize_request,
        ))
}

/// Resolve the only account owner key handlers may use. A compatibility path
/// address may agree case-insensitively, but it can never select a different
/// account. The verifier binds subject and wallet; this helper additionally
/// rejects non-canonical EVM identities before any SQL predicate is built.
pub fn canonical_owner(
    principal: &VerifiedPrincipal,
    claimed_address: Option<&str>,
) -> Result<String, StatusCode> {
    let owner = normalize_address(&principal.wallet_address).ok_or(StatusCode::FORBIDDEN)?;
    if claimed_address
        .is_some_and(|claimed| normalize_address(claimed).is_none_or(|claimed| claimed != owner))
    {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(owner)
}

fn normalize_address(address: &str) -> Option<String> {
    let bytes = address.as_bytes();
    if bytes.len() != 42
        || bytes[0] != b'0'
        || !matches!(bytes[1], b'x' | b'X')
        || !bytes[2..].iter().all(u8::is_ascii_hexdigit)
    {
        return None;
    }
    Some(format!("0x{}", address[2..].to_ascii_lowercase()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    OwnerRead,
    UnsafeProjection,
    UnsafeCustodyMutation,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if !normalized_path(path) {
        return AccessPolicy::Blocked;
    }
    let Some(tail) = path.strip_prefix("/api/v1/wallet/") else {
        return AccessPolicy::Blocked;
    };
    let segments: Vec<_> = tail.split('/').collect();

    match (method, segments.as_slice()) {
        (&Method::GET, ["accounts"]) => AccessPolicy::OwnerRead,
        (&Method::GET, ["accounts", address]) if safe_dynamic_segment(address) => {
            AccessPolicy::OwnerRead
        }
        (&Method::POST, ["verify-message"]) => AccessPolicy::Public,
        (&Method::GET, ["balance", chain, address])
            if safe_dynamic_segment(chain) && safe_dynamic_segment(address) =>
        {
            AccessPolicy::UnsafeProjection
        }
        (&Method::POST, ["accounts" | "send" | "sign-message"]) => {
            AccessPolicy::UnsafeCustodyMutation
        }
        (&Method::POST, ["estimate-gas"]) => AccessPolicy::UnsafeProjection,
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
                | "accounts"
                | "balance"
                | "send"
                | "sign-message"
                | "verify-message"
                | "estimate-gas"
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
        AccessPolicy::OwnerRead => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != FRONTEND_AUDIENCE && principal.audience != ADMIN_AUDIENCE {
                return auth_error(StatusCode::FORBIDDEN);
            }
            if normalize_address(&principal.wallet_address).is_none() {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::UnsafeProjection
        | AccessPolicy::UnsafeCustodyMutation
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
    use epsx_service_auth::{VerifiedPrincipal, VerifyError};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    const OWNER: &str = "0x1111111111111111111111111111111111111111";

    #[derive(Default)]
    struct FakeVerifier {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let (wallet, audience) = match token {
                "frontend-owner" => (OWNER, FRONTEND_AUDIENCE),
                "admin-owner" => (OWNER, ADMIN_AUDIENCE),
                "other-audience" => (OWNER, "epsx-other"),
                "malformed-wallet" => ("0xabc", FRONTEND_AUDIENCE),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: wallet.into(),
                wallet_address: wallet.into(),
                audience: audience.into(),
                permissions: vec![],
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

    async fn status(app: &Router, request: axum::http::Request<Body>) -> StatusCode {
        app.clone().oneshot(request).await.unwrap().status()
    }

    #[tokio::test]
    async fn health_and_message_verification_are_the_only_anonymous_surfaces() {
        let (app, downstream, verifier) = app();
        for method in [Method::GET, Method::HEAD] {
            let mut health = request(method, "/health", Some("frontend-owner"));
            health
                .headers_mut()
                .insert("x-user-id", "attacker".parse().unwrap());
            assert_eq!(status(&app, health).await, StatusCode::OK);
        }
        let mut verify = request(
            Method::POST,
            "/api/v1/wallet/verify-message",
            Some("frontend-owner"),
        );
        verify
            .headers_mut()
            .insert("x-wallet-address", "attacker".parse().unwrap());
        assert_eq!(status(&app, verify).await, StatusCode::OK);

        assert_eq!(downstream.hits.load(Ordering::SeqCst), 3);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn public_message_verification_rejects_an_oversized_json_body_before_handler() {
        let hits = Arc::new(AtomicUsize::new(0));
        let observed = hits.clone();
        let router = Router::new().route(
            "/api/v1/wallet/verify-message",
            axum::routing::post(move |Json(_): Json<serde_json::Value>| {
                let observed = observed.clone();
                async move {
                    observed.fetch_add(1, Ordering::SeqCst);
                    StatusCode::OK
                }
            }),
        );
        let app = protect_router(router, Arc::new(FakeVerifier::default()));
        let oversized = serde_json::json!({
            "message": "a".repeat(WALLET_JSON_BODY_LIMIT_BYTES),
            "signature": "0x00",
            "expected_address": OWNER,
        })
        .to_string();
        let req = axum::http::Request::builder()
            .method(Method::POST)
            .uri("/api/v1/wallet/verify-message")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(oversized))
            .unwrap();
        assert_eq!(status(&app, req).await, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn account_reads_require_an_exact_supported_audience_and_principal() {
        let routes = [
            "/api/v1/wallet/accounts",
            "/api/v1/wallet/accounts/0x1111111111111111111111111111111111111111",
        ];
        let (app, downstream, _) = app();
        for path in routes {
            assert_eq!(
                status(&app, request(Method::GET, path, None)).await,
                StatusCode::UNAUTHORIZED
            );
            assert_eq!(
                status(&app, request(Method::GET, path, Some("invalid"))).await,
                StatusCode::UNAUTHORIZED
            );
            for denied in ["other-audience", "malformed-wallet"] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(denied))).await,
                    StatusCode::FORBIDDEN
                );
            }
            for allowed in ["frontend-owner", "admin-owner"] {
                assert_eq!(
                    status(&app, request(Method::GET, path, Some(allowed))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 4);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn spoofable_headers_never_replace_the_verified_owner() {
        let (app, downstream, _) = app();
        let mut req = request(
            Method::GET,
            "/api/v1/wallet/accounts",
            Some("frontend-owner"),
        );
        req.headers_mut()
            .insert("x-wallet-address", "0xattacker".parse().unwrap());
        req.headers_mut()
            .insert("x-permissions", "*:*".parse().unwrap());
        assert_eq!(status(&app, req).await, StatusCode::OK);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 1);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn owner_is_canonical_and_cross_owner_or_invalid_claims_are_hidden() {
        let principal = VerifiedPrincipal {
            subject: OWNER.into(),
            wallet_address: "0x111111111111111111111111111111111111AaAa".into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: vec![],
        };
        assert_eq!(
            canonical_owner(&principal, None).unwrap(),
            "0x111111111111111111111111111111111111aaaa"
        );
        assert_eq!(
            canonical_owner(
                &principal,
                Some("0X111111111111111111111111111111111111AAAA")
            )
            .unwrap(),
            "0x111111111111111111111111111111111111aaaa"
        );
        assert_eq!(
            canonical_owner(
                &principal,
                Some("0x2222222222222222222222222222222222222222")
            ),
            Err(StatusCode::NOT_FOUND)
        );
        let invalid = VerifiedPrincipal {
            wallet_address: "0xabc".into(),
            ..principal
        };
        assert_eq!(canonical_owner(&invalid, None), Err(StatusCode::FORBIDDEN));
    }

    #[tokio::test]
    async fn unsafe_projection_and_custody_routes_fail_before_auth_or_handlers() {
        let routes = [
            (Method::POST, "/api/v1/wallet/accounts"),
            (
                Method::GET,
                "/api/v1/wallet/balance/56/0x1111111111111111111111111111111111111111",
            ),
            (Method::POST, "/api/v1/wallet/send"),
            (Method::POST, "/api/v1/wallet/sign-message"),
            (Method::POST, "/api/v1/wallet/estimate-gas"),
        ];
        let (app, downstream, verifier) = app();
        for (method, path) in routes {
            for bearer in [None, Some("invalid"), Some("frontend-owner")] {
                assert_eq!(
                    status(&app, request(method.clone(), path, bearer)).await,
                    StatusCode::NOT_FOUND,
                    "{method} {path}"
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_arities_encoded_and_reserved_paths_fail_before_auth() {
        let routes = [
            (Method::POST, "/health"),
            (Method::GET, "/health/"),
            (Method::HEAD, "/api/v1/wallet/verify-message"),
            (Method::POST, "/api/v1/wallet/accounts/owner"),
            (Method::DELETE, "/api/v1/wallet/accounts/owner"),
            (Method::GET, "/api/v1/wallet/accounts/accounts"),
            (Method::GET, "/api/v1/wallet/accounts/../send"),
            (Method::GET, "/api/v1/wallet/accounts/%2e%2e"),
            (Method::GET, "/api/v1/wallet/balance/56"),
            (Method::GET, "/api/v1/wallet/balance/56/address/extra"),
            (Method::GET, "/api/v1/wallet//accounts"),
            (Method::GET, "/api/v1/wallet/unknown"),
            (Method::GET, "/unknown"),
        ];
        let (app, downstream, verifier) = app();
        for (method, path) in routes {
            assert_eq!(
                status(&app, request(method.clone(), path, Some("frontend-owner"))).await,
                StatusCode::NOT_FOUND,
                "{method} {path}"
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn narrowed_runtime_mount_returns_404_for_wrong_method_and_unknown_path() {
        let verifier = Arc::new(FakeVerifier::default());
        let router = Router::new().route(
            "/api/v1/wallet/accounts",
            axum::routing::get(|| async { StatusCode::OK }),
        );
        let app = protect_router(router, verifier.clone());
        assert_eq!(
            status(
                &app,
                request(
                    Method::DELETE,
                    "/api/v1/wallet/accounts",
                    Some("frontend-owner")
                )
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            status(
                &app,
                request(Method::GET, "/api/v1/wallet/nope", Some("frontend-owner"))
            )
            .await,
            StatusCode::NOT_FOUND
        );
        assert_eq!(verifier.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn production_verifier_rejects_insecure_or_local_configuration() {
        assert!(matches!(
            build_auth_verifier(
                "https://identity.example.com",
                "http://identity.example.com/.well-known/jwks.json",
                true,
            ),
            Err(WalletConfigError::Auth(_))
        ));
        assert!(matches!(
            build_auth_verifier(
                "https://localhost:8443",
                "https://localhost:8443/.well-known/jwks.json",
                true,
            ),
            Err(WalletConfigError::Auth(_))
        ));
    }
}
