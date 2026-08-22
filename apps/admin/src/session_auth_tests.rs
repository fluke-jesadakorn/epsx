use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use epsx_bff::{
    cookies::{
        CookieEnvironment, LEGACY_ACCESS_COOKIE, LEGACY_LOCAL_ACCESS_COOKIE,
        LEGACY_LOCAL_REFRESH_COOKIE, LOCAL_ADMIN_ACCESS_COOKIE as LOCAL_ACCESS_COOKIE,
        LOCAL_ADMIN_REFRESH_COOKIE as LOCAL_REFRESH_COOKIE,
    },
    refresh_outcome::{
        REFRESH_OUTCOME_HEADER, REFRESH_OUTCOME_NOT_ROTATED, REFRESH_OUTCOME_REJECTED,
        REFRESH_OUTCOME_ROTATED, SESSION_STATE_CLEARED, SESSION_STATE_HEADER,
        SESSION_STATE_PRESERVED, SESSION_STATE_ROTATED,
    },
    session::{
        AccessTokenClaims, Jwks, JwksVerifier, JwksVerifierConfig, RsaJwk, ADMIN_CLIENT_ID,
        FRONTEND_CLIENT_ID, JWKS_PATH, LOGOUT_PATH, PROFILE_PATH, REFRESH_PATH, VERIFY_PATH,
    },
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::thread_rng;
use rsa::{pkcs8::EncodePrivateKey, traits::PublicKeyParts, RsaPrivateKey};
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tower::ServiceExt;

use crate::{build_app, session_auth, AppState, SiweLoginBody};

const TEST_ISSUER: &str = "https://issuer.test";
const TEST_WALLET: &str = "0x1111111111111111111111111111111111111111";

#[test]
fn failed_cookie_clear_never_attests_cleared_session_state() {
    let response = match session_auth::try_clear_session_response(
        StatusCode::UNAUTHORIZED,
        "forced_cookie_failure",
        |_| false,
    ) {
        Ok(_) => panic!("forced cookie failure unexpectedly succeeded"),
        Err(response) => *response,
    };
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.headers().get(SESSION_STATE_HEADER).is_none());
}

fn upstream_refresh_response(mut response: Response, outcome: &'static str) -> Response {
    response
        .headers_mut()
        .insert(REFRESH_OUTCOME_HEADER, HeaderValue::from_static(outcome));
    response
}

fn assert_session_state(response: &Response, expected: &'static str) {
    assert_eq!(
        response
            .headers()
            .get(SESSION_STATE_HEADER)
            .and_then(|value| value.to_str().ok()),
        Some(expected)
    );
}

struct TestKey {
    encoding: EncodingKey,
    jwk: RsaJwk,
}

impl TestKey {
    fn generate() -> Self {
        let private = RsaPrivateKey::new(&mut thread_rng(), 2048).unwrap();
        let pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();
        let public = private.to_public_key();
        Self {
            encoding: EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap(),
            jwk: RsaJwk {
                kty: "RSA".into(),
                use_: Some("sig".into()),
                alg: Some("RS256".into()),
                kid: "admin-session-test-key".into(),
                n: URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                e: URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
            },
        }
    }

    fn access_token(&self, audience: &str, permissions: &[&str]) -> String {
        self.access_token_with_audiences(&[audience], permissions)
    }

    fn access_token_with_audiences(&self, audiences: &[&str], permissions: &[&str]) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.jwk.kid.clone());
        encode(
            &header,
            &AccessTokenClaims {
                iss: TEST_ISSUER.into(),
                sub: TEST_WALLET.into(),
                aud: audiences
                    .iter()
                    .map(|audience| (*audience).to_string())
                    .collect(),
                exp: now + 300,
                iat: now - 1,
                jti: "admin-test-jti".into(),
                scope: format!("openid permissions {}", permissions.join(" ")),
                wallet_address: TEST_WALLET.into(),
                auth_method: "web3_siwe".into(),
                auth_time: now - 1,
                nbf: None,
            },
            &self.encoding,
        )
        .unwrap()
    }
}

async fn spawn_mock(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{address}")
}

async fn unused_base_url() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    format!("http://{address}")
}

fn state(base_url: &str) -> AppState {
    let config = epsx_client::ClientConfig {
        base_url: base_url.to_string(),
        timeout: Duration::from_secs(1),
    };
    let client = Arc::new(epsx_client::ServiceClient::new(config));
    let verifier = JwksVerifierConfig::new(
        format!("{base_url}{JWKS_PATH}"),
        TEST_ISSUER,
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
        verifier: Arc::new(JwksVerifier::with_http(verifier).unwrap()),
        cookie_environment: CookieEnvironment::Local,
        api_url: base_url.to_string(),
        demo_login_enabled: false,
        dev_bypass_enabled: false,
    }
}

fn cookie_headers(cookie: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
    headers
}

fn response_cookies(response: &Response) -> Vec<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap().to_string())
        .collect()
}

fn assert_session_cleared(response: &Response) {
    let cookies = response_cookies(response);
    assert_eq!(cookies.len(), 5);
    assert_session_clear_cookie_set(&cookies);
}

fn assert_session_clear_cookie_set(cookies: &[String]) {
    for name in [
        LOCAL_ACCESS_COOKIE,
        LOCAL_REFRESH_COOKIE,
        LEGACY_LOCAL_ACCESS_COOKIE,
        LEGACY_LOCAL_REFRESH_COOKIE,
        LEGACY_ACCESS_COOKIE,
    ] {
        let cookie = cookies
            .iter()
            .find(|cookie| cookie.starts_with(&format!("{name}=")))
            .unwrap_or_else(|| panic!("missing clear cookie for {name}: {cookies:?}"));
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.contains("HttpOnly"));
    }
}

fn jwks_route(jwks: Jwks) -> Router {
    Router::new().route(
        JWKS_PATH,
        get(move || {
            let jwks = jwks.clone();
            async move { Json(jwks) }
        }),
    )
}

#[tokio::test]
async fn admin_siwe_requires_admin_audience_and_returns_no_tokens() {
    let key = TestKey::generate();
    let access = key.access_token(ADMIN_CLIENT_ID, &["admin:users:manage"]);
    let payload = json!({
        "success": true,
        "authenticated": true,
        "wallet_address": TEST_WALLET,
        "permissions": ["unsigned:stale"],
        "access_token": access,
        "refresh_token": "opaque-admin-refresh",
        "expires_in": 300,
        "refresh_expires_in": 3600
    });
    let router = jwks_route(Jwks {
        keys: vec![key.jwk.clone()],
    })
    .route(
        VERIFY_PATH,
        post(move |Json(body): Json<Value>| {
            let payload = payload.clone();
            async move {
                if body["client_id"] == ADMIN_CLIENT_ID && body["wallet_address"] == TEST_WALLET {
                    Json(payload).into_response()
                } else {
                    StatusCode::BAD_REQUEST.into_response()
                }
            }
        }),
    );
    let base_url = spawn_mock(router).await;
    let response = session_auth::siwe_login(
        State(state(&base_url)),
        Json(SiweLoginBody {
            message: "sign me".into(),
            signature: "0xsigned".into(),
            _chain_id: "56".into(),
            address: TEST_WALLET.into(),
            nonce: "nonce-1".into(),
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_cookies(&response).len(), 5);
    let bytes = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["user"]["permissions"], json!(["admin:users:manage"]));
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(!text.contains("access_token"));
    assert!(!text.contains("refresh_token"));
    assert!(!text.contains("opaque-admin-refresh"));
}

#[tokio::test]
async fn frontend_audience_token_cannot_create_admin_session() {
    let key = TestKey::generate();
    let access = key.access_token(FRONTEND_CLIENT_ID, &[]);
    let payload = json!({
        "success": true, "authenticated": true, "wallet_address": TEST_WALLET,
        "access_token": access, "refresh_token": "opaque", "expires_in": 300,
        "refresh_expires_in": 3600
    });
    let router = jwks_route(Jwks {
        keys: vec![key.jwk.clone()],
    })
    .route(
        VERIFY_PATH,
        post(move || {
            let payload = payload.clone();
            async move { Json(payload) }
        }),
    );
    let base_url = spawn_mock(router).await;
    let response = session_auth::siwe_login(
        State(state(&base_url)),
        Json(SiweLoginBody {
            message: "message".into(),
            signature: "signature".into(),
            _chain_id: "56".into(),
            address: TEST_WALLET.into(),
            nonce: "nonce".into(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert!(response_cookies(&response).is_empty());
}

#[tokio::test]
async fn refresh_reads_cookie_and_rotates_verified_pair() {
    let key = TestKey::generate();
    let access = key.access_token(ADMIN_CLIENT_ID, &["admin:payments:read"]);
    let payload = json!({
        "success": true, "authenticated": true, "access_token": access,
        "refresh_token": "rotated-admin-refresh", "expires_in": 300,
        "refresh_expires_in": 3600,
        "user": {"wallet_address": TEST_WALLET, "subject": TEST_WALLET,
                 "permissions": ["admin:payments:read"], "auth_method": "web3_siwe"}
    });
    let router = jwks_route(Jwks {
        keys: vec![key.jwk.clone()],
    })
    .route(
        REFRESH_PATH,
        post(move |Json(body): Json<Value>| {
            let payload = payload.clone();
            async move {
                assert_eq!(body["client_id"], ADMIN_CLIENT_ID);
                assert_eq!(body["refresh_token"], "browser-admin-refresh");
                upstream_refresh_response(Json(payload).into_response(), REFRESH_OUTCOME_ROTATED)
            }
        }),
    );
    let base_url = spawn_mock(router).await;
    let response = session_auth::refresh_token(
        State(state(&base_url)),
        cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_session_state(&response, SESSION_STATE_ROTATED);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("private, no-store"))
    );
    let cookies = response_cookies(&response);
    assert_eq!(cookies.len(), 5);
    assert!(cookies
        .iter()
        .any(|cookie| cookie.starts_with("epsx.admin.access_token=")));
    assert!(cookies
        .iter()
        .any(|cookie| cookie.starts_with("epsx.admin.refresh_token=rotated-admin-refresh")));
}

#[tokio::test]
async fn rejected_or_malformed_refresh_clears_canonical_and_legacy_cookies() {
    let rejected_base = spawn_mock(Router::new().route(
        REFRESH_PATH,
        post(|| async {
            upstream_refresh_response(
                StatusCode::UNAUTHORIZED.into_response(),
                REFRESH_OUTCOME_REJECTED,
            )
        }),
    ))
    .await;
    let rejected = session_auth::refresh_token(
        State(state(&rejected_base)),
        cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
    assert_session_cleared(&rejected);
    assert_session_state(&rejected, SESSION_STATE_CLEARED);

    let malformed_base = spawn_mock(Router::new().route(
        REFRESH_PATH,
        post(|| async {
            upstream_refresh_response("not-json".into_response(), REFRESH_OUTCOME_ROTATED)
        }),
    ))
    .await;
    let malformed = session_auth::refresh_token(
        State(state(&malformed_base)),
        cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
    )
    .await;
    assert_eq!(malformed.status(), StatusCode::BAD_GATEWAY);
    assert_session_cleared(&malformed);
    assert_session_state(&malformed, SESSION_STATE_CLEARED);

    for (upstream_status, outcome, expected_status, expected_state, clears) in [
        (
            StatusCode::BAD_REQUEST,
            Some(REFRESH_OUTCOME_NOT_ROTATED),
            StatusCode::BAD_REQUEST,
            SESSION_STATE_PRESERVED,
            false,
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(REFRESH_OUTCOME_NOT_ROTATED),
            StatusCode::INTERNAL_SERVER_ERROR,
            SESSION_STATE_PRESERVED,
            false,
        ),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            None,
            StatusCode::BAD_GATEWAY,
            SESSION_STATE_CLEARED,
            true,
        ),
        (
            StatusCode::REQUEST_TIMEOUT,
            Some(REFRESH_OUTCOME_NOT_ROTATED),
            StatusCode::BAD_GATEWAY,
            SESSION_STATE_CLEARED,
            true,
        ),
        (
            StatusCode::TOO_MANY_REQUESTS,
            None,
            StatusCode::BAD_GATEWAY,
            SESSION_STATE_CLEARED,
            true,
        ),
    ] {
        let retryable_base = spawn_mock(Router::new().route(
            REFRESH_PATH,
            post(move || async move {
                let response = upstream_status.into_response();
                match outcome {
                    Some(outcome) => upstream_refresh_response(response, outcome),
                    None => response,
                }
            }),
        ))
        .await;
        let retryable = session_auth::refresh_token(
            State(state(&retryable_base)),
            cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
        )
        .await;
        assert_eq!(retryable.status(), expected_status);
        assert_session_state(&retryable, expected_state);
        assert_eq!(!response_cookies(&retryable).is_empty(), clears);
    }
}

#[tokio::test]
async fn refresh_transport_failure_clears_the_unprovable_session() {
    let response = session_auth::refresh_token(
        State(state(&unused_base_url().await)),
        cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_session_cleared(&response);
    assert_session_state(&response, SESSION_STATE_CLEARED);
}

#[tokio::test]
async fn me_verifies_jwt_cross_checks_identity_and_clears_mismatch() {
    let key = TestKey::generate();
    let access = key.access_token(ADMIN_CLIENT_ID, &["token:scope"]);
    let expected_access = access.clone();
    let router = jwks_route(Jwks {
        keys: vec![key.jwk.clone()],
    })
    .route(
        PROFILE_PATH,
        get(move |headers: HeaderMap| {
            let expected_access = expected_access.clone();
            async move {
                assert_eq!(
                    headers
                        .get(header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok()),
                    Some(format!("Bearer {expected_access}").as_str())
                );
                Json(json!({"data": {
                    "wallet_address": "0x2222222222222222222222222222222222222222",
                    "subject": "0x2222222222222222222222222222222222222222",
                    "permissions": ["backend:verbatim"], "capabilities": ["admin-capability"]
                }}))
            }
        }),
    );
    let base_url = spawn_mock(router).await;
    let response = session_auth::auth_me(
        State(state(&base_url)),
        cookie_headers(&format!("epsx.admin.access_token={access}")),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert_session_cleared(&response);

    let invalid_base = unused_base_url().await;
    let invalid = session_auth::auth_me(
        State(state(&invalid_base)),
        cookie_headers("epsx.admin.access_token=not-a-jwt"),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    assert_session_cleared(&invalid);
}

#[tokio::test]
async fn me_preserves_backend_profile_permissions_and_capabilities_verbatim() {
    let key = TestKey::generate();
    let access = key.access_token(ADMIN_CLIENT_ID, &["jwt:possibly-different"]);
    let router = jwks_route(Jwks {
        keys: vec![key.jwk.clone()],
    })
    .route(
        PROFILE_PATH,
        get(|| async {
            Json(json!({"data": {
                "wallet_address": TEST_WALLET,
                "subject": TEST_WALLET,
                "permissions": ["backend:profile:permission"],
                "capabilities": ["backend-admin-capability"],
                "auth_method": "web3_siwe"
            }}))
        }),
    );
    let base_url = spawn_mock(router).await;
    let response = session_auth::auth_me(
        State(state(&base_url)),
        cookie_headers(&format!("epsx.admin.access_token={access}")),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["permissions"], json!(["backend:profile:permission"]));
    assert_eq!(body["capabilities"], json!(["backend-admin-capability"]));
}

#[tokio::test]
async fn logout_calls_delete_and_always_clears_locally() {
    let base_url = spawn_mock(Router::new().route(
        LOGOUT_PATH,
        delete(|| async { Json(json!({"success": true})) }),
    ))
    .await;
    let success = session_auth::logout(
        State(state(&base_url)),
        cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
    )
    .await;
    assert_eq!(success.status(), StatusCode::OK);
    assert_session_cleared(&success);
    assert_session_state(&success, SESSION_STATE_CLEARED);

    let unavailable = unused_base_url().await;
    let failure = session_auth::logout(
        State(state(&unavailable)),
        cookie_headers("epsx.admin.refresh_token=browser-admin-refresh"),
    )
    .await;
    assert_eq!(failure.status(), StatusCode::BAD_GATEWAY);
    assert_session_cleared(&failure);
    assert_session_state(&failure, SESSION_STATE_CLEARED);

    let mut bypass_state = state(&unavailable);
    bypass_state.dev_bypass_enabled = true;
    let bypass = session_auth::logout(State(bypass_state), HeaderMap::new()).await;
    assert_eq!(bypass.status(), StatusCode::OK);
    let bypass_cookies = response_cookies(&bypass);
    assert_eq!(bypass_cookies.len(), 6);
    assert_session_clear_cookie_set(&bypass_cookies);
    assert_session_state(&bypass, SESSION_STATE_CLEARED);
    assert!(bypass_cookies
        .iter()
        .any(|cookie| cookie.starts_with("epsx.admin.dev_bypass_disabled=1;")));
    let body = to_bytes(bypass.into_body(), 16 * 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["success"],
        true
    );
}

#[tokio::test]
async fn unauthenticated_admin_proxy_fails_before_upstream() {
    let upstream_hits = Arc::new(AtomicUsize::new(0));
    let hits = upstream_hits.clone();
    let base_url = spawn_mock(Router::new().route(
        "/api/v1/identity/users",
        get(move || {
            let hits = hits.clone();
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                Json(json!({"users": []}))
            }
        }),
    ))
    .await;
    let response = build_app(state(&base_url))
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_session_cleared(&response);
    assert_eq!(upstream_hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn dashboard_root_loads_only_for_verified_admin_root_aliases() {
    let key = TestKey::generate();
    let access = key.access_token(ADMIN_CLIENT_ID, &["admin:dashboard:view"]);
    let dashboard_hits = Arc::new(AtomicUsize::new(0));
    let hits = dashboard_hits.clone();
    let router = jwks_route(Jwks {
        keys: vec![key.jwk.clone()],
    })
    .route(
        "/api/admin/dashboard/user-status",
        get(move |headers: HeaderMap| {
            let hits = hits.clone();
            async move {
                hits.fetch_add(1, Ordering::SeqCst);
                assert!(headers.get(header::AUTHORIZATION).is_some());
                assert!(headers.get("x-request-id").is_some());
                assert!(headers.get("x-user-id").is_none());
                assert!(headers.get("x-user-address").is_none());
                Json(json!({
                    "success": true,
                    "data": {
                        "observed_at": "2026-07-23T03:04:04Z",
                        "total_users": 11,
                        "active_users": 8
                    },
                    "message": "Dashboard user status retrieved successfully",
                    "timestamp": "2026-07-23T03:04:05Z",
                    "admin_meta": {
                        "operation": "get_dashboard_user_status",
                        "performed_by": "admin"
                    }
                }))
            }
        }),
    );
    let base_url = spawn_mock(router).await;
    let app = build_app(state(&base_url));

    for path in ["/", "/index"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header(header::AUTHORIZATION, format!("Bearer {access}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    assert_eq!(dashboard_hits.load(Ordering::SeqCst), 2);

    for path in ["/?", "/?unexpected=1", "/admin/admin", "/admin/admin/index"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header(header::AUTHORIZATION, format!("Bearer {access}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::OK | StatusCode::NOT_FOUND
        ));
    }
    assert_eq!(dashboard_hits.load(Ordering::SeqCst), 2);

    let signed_out = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(signed_out.status(), StatusCode::OK);
    assert_eq!(dashboard_hits.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn wrong_or_multiple_audiences_cannot_reach_the_dashboard_upstream() {
    for audiences in [
        vec![FRONTEND_CLIENT_ID],
        vec![ADMIN_CLIENT_ID, FRONTEND_CLIENT_ID],
    ] {
        let key = TestKey::generate();
        let access = key.access_token_with_audiences(&audiences, &["admin:dashboard:view"]);
        let dashboard_hits = Arc::new(AtomicUsize::new(0));
        let hits = dashboard_hits.clone();
        let router = jwks_route(Jwks {
            keys: vec![key.jwk.clone()],
        })
        .route(
            "/api/admin/dashboard/user-status",
            get(move || {
                let hits = hits.clone();
                async move {
                    hits.fetch_add(1, Ordering::SeqCst);
                    Json(json!({"unexpected": true}))
                }
            }),
        );
        let base_url = spawn_mock(router).await;
        let response = build_app(state(&base_url))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::AUTHORIZATION, format!("Bearer {access}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(dashboard_hits.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn every_canonical_auth_route_and_login_alias_is_public() {
    let base_url = unused_base_url().await;
    let app = build_app(state(&base_url));

    for path in [
        "/api/v1/auth/challenge",
        "/api/v1/auth/siwe",
        "/api/v1/auth/login",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{path}"
        );
        assert!(response_cookies(&response).is_empty(), "{path}");
    }

    let refresh = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(refresh.status(), StatusCode::UNAUTHORIZED);
    let refresh_body = to_bytes(refresh.into_body(), 16 * 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&refresh_body).unwrap()["error"],
        "missing_refresh_token"
    );

    let me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::UNAUTHORIZED);
    assert!(response_cookies(&me).is_empty());
    let me_body = to_bytes(me.into_body(), 16 * 1024).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&me_body).unwrap()["error"],
        "missing_access_token"
    );

    let logout = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::BAD_GATEWAY);
    assert_session_cleared(&logout);

    let demo = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/demo")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(demo.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn auth_page_is_public_and_redirects_to_the_fixed_admin_root() {
    let base_url = unused_base_url().await;
    let response = build_app(state(&base_url))
        .oneshot(
            Request::builder()
                .uri("/auth?return_url=https%3A%2F%2Fevil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(response.headers()[header::LOCATION], "/");
    let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
    assert!(body.is_empty());
}
