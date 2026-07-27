use axum::{
    extract::{Request, State},
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

pub const PAYMENTS_VIEW_PERMISSION: &str = "admin:payments:view";
pub const PAYMENTS_MANAGE_PERMISSION: &str = "admin:payments:manage";
pub const PAYMENT_LINKS_VIEW_PERMISSION: &str = "admin:payment-links:view";
pub const PAYMENT_LINKS_MANAGE_PERMISSION: &str = "admin:payment-links:manage";

#[derive(Debug, Error)]
pub enum PayConfigError {
    #[error("HTTP client configuration failed")]
    Http(#[from] reqwest::Error),
    #[error("OIDC verifier configuration failed")]
    Auth(#[from] epsx_service_auth::VerifyError),
}

pub fn build_auth_verifier(
    issuer: &str,
    jwks_url: &str,
    production: bool,
) -> Result<Arc<JwksVerifier>, PayConfigError> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .pool_idle_timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("epsx-pay/1")
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

/// The verified wallet is the only owner key accepted by payment handlers.
/// Compatibility path/body values may agree with it but can never select a
/// different wallet's financial records.
pub fn canonical_owner(
    principal: &VerifiedPrincipal,
    claimed_wallet: Option<&str>,
) -> Result<String, StatusCode> {
    if claimed_wallet.is_some_and(|claimed| {
        claimed.trim().is_empty() || !claimed.eq_ignore_ascii_case(&principal.wallet_address)
    }) {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(principal.wallet_address.to_ascii_lowercase())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessPolicy {
    Public,
    AdminPermission(&'static str),
    OwnerRead,
    PaymentsRead,
    UnsafePaymentsManage,
    UnsafeFinancialMutation,
    InternalIdentityUnavailable,
    Blocked,
}

fn classify(method: &Method, path: &str) -> AccessPolicy {
    if matches!(method, &Method::GET | &Method::HEAD) && path == "/health" {
        return AccessPolicy::Public;
    }
    if path.contains('%') || path.contains('\\') {
        return AccessPolicy::Blocked;
    }

    if path == "/api/v1/admin/pay" || path.starts_with("/api/v1/admin/pay/") {
        let tail = path.strip_prefix("/api/v1/admin/pay/").unwrap_or_default();
        let segments: Vec<_> = tail.split('/').collect();
        if segments.iter().any(|segment| segment.is_empty()) {
            return AccessPolicy::Blocked;
        }
        return match (method, segments.as_slice()) {
            (&Method::GET, ["links"]) => {
                AccessPolicy::AdminPermission(PAYMENT_LINKS_VIEW_PERMISSION)
            }
            (&Method::POST, ["links"]) => {
                AccessPolicy::AdminPermission(PAYMENT_LINKS_MANAGE_PERMISSION)
            }
            (&Method::POST, ["links", id, "disable"]) if safe_dynamic_segment(id) => {
                AccessPolicy::AdminPermission(PAYMENT_LINKS_MANAGE_PERMISSION)
            }
            (&Method::POST, ["intents", id, "cancel"]) if safe_dynamic_segment(id) => {
                AccessPolicy::AdminPermission(PAYMENTS_MANAGE_PERMISSION)
            }
            _ => {
                // Preserve the existing read/force operation classifier below.
                return match (method, segments.as_slice()) {
                    (&Method::GET, ["intents"]) => AccessPolicy::PaymentsRead,
                    (&Method::POST, ["intents", id, "force-cancel"])
                        if safe_dynamic_segment(id) =>
                    {
                        AccessPolicy::UnsafePaymentsManage
                    }
                    (&Method::POST, ["escrows", id, "force-release" | "force-refund"])
                        if safe_dynamic_segment(id) =>
                    {
                        AccessPolicy::UnsafePaymentsManage
                    }
                    _ => AccessPolicy::Blocked,
                };
            }
        };
    }

    if let Some(tail) = path.strip_prefix("/api/v1/admin/pay/") {
        let Some(segments) = safe_segments(tail) else {
            return AccessPolicy::Blocked;
        };
        return match (method, segments.as_slice()) {
            (&Method::GET, ["intents"]) => AccessPolicy::PaymentsRead,
            (&Method::POST, ["intents", id, "force-cancel"]) if safe_dynamic_segment(id) => {
                AccessPolicy::UnsafePaymentsManage
            }
            (&Method::POST, ["escrows", id, "force-release" | "force-refund"])
                if safe_dynamic_segment(id) =>
            {
                AccessPolicy::UnsafePaymentsManage
            }
            _ => AccessPolicy::Blocked,
        };
    }

    let Some(tail) = path.strip_prefix("/api/v1/pay/") else {
        return AccessPolicy::Blocked;
    };
    let Some(segments) = safe_segments(tail) else {
        return AccessPolicy::Blocked;
    };

    match (method, segments.as_slice()) {
        (&Method::GET, ["links", slug]) if safe_dynamic_segment(slug) => AccessPolicy::Public,
        (&Method::GET, ["intents" | "escrows"]) => AccessPolicy::OwnerRead,
        (&Method::GET, ["intents" | "escrows", id]) if safe_dynamic_segment(id) => {
            AccessPolicy::OwnerRead
        }
        (&Method::GET, ["history", wallet]) if safe_dynamic_segment(wallet) => {
            AccessPolicy::OwnerRead
        }
        (&Method::POST, ["escrows", id, "resolve"]) if safe_dynamic_segment(id) => {
            AccessPolicy::UnsafePaymentsManage
        }
        (&Method::POST, ["intents"]) | (&Method::POST, ["links"]) => {
            AccessPolicy::UnsafeFinancialMutation
        }
        (&Method::POST, ["intents", id, "confirm" | "cancel"])
        | (&Method::POST, ["escrows", id, "release" | "refund" | "dispute"])
        | (&Method::POST, ["links", id, "redeem"])
            if safe_dynamic_segment(id) =>
        {
            AccessPolicy::UnsafeFinancialMutation
        }
        (&Method::POST, ["escrows", id, "confirm-deposit"]) if safe_dynamic_segment(id) => {
            AccessPolicy::InternalIdentityUnavailable
        }
        (&Method::POST, ["webhooks", "on-chain"]) => AccessPolicy::InternalIdentityUnavailable,
        _ => AccessPolicy::Blocked,
    }
}

fn safe_segments(path: &str) -> Option<Vec<&str>> {
    let segments: Vec<_> = path.split('/').collect();
    if segments.iter().any(|segment| {
        segment.is_empty()
            || matches!(*segment, "." | "..")
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    }) {
        None
    } else {
        Some(segments)
    }
}

fn safe_dynamic_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        && !segment.starts_with("force-")
        && !matches!(
            segment,
            "health"
                | "pay"
                | "admin"
                | "intents"
                | "escrows"
                | "links"
                | "history"
                | "webhooks"
                | "on-chain"
                | "sync"
                | "confirm"
                | "cancel"
                | "release"
                | "refund"
                | "dispute"
                | "resolve"
                | "confirm-deposit"
                | "redeem"
                | "force-cancel"
                | "force-release"
                | "force-refund"
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
        AccessPolicy::AdminPermission(required) => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal
                    .permissions
                    .iter()
                    .any(|permission| permission == required)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
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
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::PaymentsRead => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(PAYMENTS_VIEW_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            request.extensions_mut().insert(principal);
        }
        AccessPolicy::UnsafePaymentsManage => {
            let principal =
                match authenticate_headers(state.verifier.as_ref(), request.headers()).await {
                    Ok(principal) => principal,
                    Err(_) => return auth_error(StatusCode::UNAUTHORIZED),
                };
            if principal.audience != ADMIN_AUDIENCE
                || !principal.has_permission(PAYMENTS_MANAGE_PERMISSION)
            {
                return auth_error(StatusCode::FORBIDDEN);
            }
            // Force operations remain intentionally unavailable even to an
            // authorized admin; do not forward them to a downstream handler.
            let _ = principal;
            return StatusCode::NOT_FOUND.into_response();
        }
        AccessPolicy::UnsafeFinancialMutation => {
            // Owner-facing financial mutations remain deliberately hidden until
            // their full typed, audited, finality-aware contract is available.
            // Do not authenticate or expose a downstream side effect here.
            return StatusCode::NOT_FOUND.into_response();
        }
        AccessPolicy::InternalIdentityUnavailable | AccessPolicy::Blocked => {
            return StatusCode::NOT_FOUND.into_response()
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
    use axum::{body::Body, routing::any};
    use epsx_service_auth::VerifyError;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    struct FakeVerifier;

    #[async_trait]
    impl AccessTokenVerifier for FakeVerifier {
        async fn verify(&self, token: &str) -> Result<VerifiedPrincipal, VerifyError> {
            let (audience, permissions) = match token {
                "frontend-owner" => (FRONTEND_AUDIENCE, vec![]),
                "admin-owner" => (ADMIN_AUDIENCE, vec![]),
                "admin-view" => (ADMIN_AUDIENCE, vec![PAYMENTS_VIEW_PERMISSION.into()]),
                "admin-manage" => (ADMIN_AUDIENCE, vec![PAYMENTS_MANAGE_PERMISSION.into()]),
                "admin-resource-wildcard" => (ADMIN_AUDIENCE, vec!["admin:payments:*".into()]),
                "admin-domain-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:*".into()]),
                "admin-global-wildcard" => (ADMIN_AUDIENCE, vec!["*:*".into()]),
                "admin-invalid-wildcard" => (ADMIN_AUDIENCE, vec!["admin:*:manage".into()]),
                "frontend-view" => (FRONTEND_AUDIENCE, vec![PAYMENTS_VIEW_PERMISSION.into()]),
                "frontend-manage" => (FRONTEND_AUDIENCE, vec![PAYMENTS_MANAGE_PERMISSION.into()]),
                "other-audience" => ("epsx-other", vec![]),
                _ => return Err(VerifyError::Validation),
            };
            Ok(VerifiedPrincipal {
                subject: "0xAbC".into(),
                wallet_address: "0xAbC".into(),
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
    async fn health_and_safe_link_lookup_are_the_only_public_surfaces() {
        let (app, downstream) = app();
        for (method, path) in [
            (Method::GET, "/health"),
            (Method::HEAD, "/health"),
            (Method::GET, "/api/v1/pay/links/epsx-abc123"),
        ] {
            let mut req = request(method, path, Some("admin-manage"));
            req.headers_mut()
                .insert("x-wallet-address", "attacker".parse().unwrap());
            assert_eq!(status(&app, req).await, StatusCode::OK);
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 3);
        assert_eq!(downstream.authorization_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn owner_reads_require_an_exact_supported_audience() {
        let routes = [
            (Method::GET, "/api/v1/pay/intents"),
            (Method::GET, "/api/v1/pay/intents/intent-id"),
            (Method::GET, "/api/v1/pay/escrows"),
            (Method::GET, "/api/v1/pay/escrows/escrow-id"),
            (Method::GET, "/api/v1/pay/history/0xabc"),
        ];
        let (app, downstream) = app();
        for bearer in ["frontend-owner", "admin-owner"] {
            for (method, path) in &routes {
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(bearer))).await,
                    StatusCode::OK
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 10);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 10);

        for bearer in [None, Some("invalid"), Some("other-audience")] {
            let expected = if bearer == Some("other-audience") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            };
            assert_eq!(
                status(&app, request(Method::GET, "/api/v1/pay/intents", bearer),).await,
                expected
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn admin_read_requires_view_and_exact_admin_audience() {
        let path = "/api/v1/admin/pay/intents";
        let (app, downstream) = app();
        for denied in [
            None,
            Some("invalid"),
            Some("admin-owner"),
            Some("admin-manage"),
            Some("frontend-view"),
            Some("admin-invalid-wildcard"),
        ] {
            let expected = if matches!(denied, None | Some("invalid")) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::FORBIDDEN
            };
            assert_eq!(
                status(&app, request(Method::GET, path, denied)).await,
                expected
            );
        }
        for allowed in [
            "admin-view",
            "admin-resource-wildcard",
            "admin-domain-wildcard",
            "admin-global-wildcard",
        ] {
            assert_eq!(
                status(&app, request(Method::GET, path, Some(allowed))).await,
                StatusCode::OK
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn admin_mutations_require_manage_but_remain_unavailable() {
        let routes = [
            (Method::POST, "/api/v1/pay/escrows/escrow-id/resolve"),
            (
                Method::POST,
                "/api/v1/admin/pay/intents/intent-id/force-cancel",
            ),
            (
                Method::POST,
                "/api/v1/admin/pay/escrows/escrow-id/force-release",
            ),
            (
                Method::POST,
                "/api/v1/admin/pay/escrows/escrow-id/force-refund",
            ),
        ];
        let (app, downstream) = app();
        for (method, path) in &routes {
            for denied in [
                None,
                Some("invalid"),
                Some("admin-owner"),
                Some("admin-view"),
                Some("frontend-manage"),
                Some("admin-invalid-wildcard"),
            ] {
                let expected = if matches!(denied, None | Some("invalid")) {
                    StatusCode::UNAUTHORIZED
                } else {
                    StatusCode::FORBIDDEN
                };
                assert_eq!(
                    status(&app, request(method.clone(), path, denied)).await,
                    expected
                );
            }
            for allowed in [
                "admin-manage",
                "admin-resource-wildcard",
                "admin-domain-wildcard",
                "admin-global-wildcard",
            ] {
                assert_eq!(
                    status(&app, request(method.clone(), path, Some(allowed))).await,
                    StatusCode::NOT_FOUND
                );
            }
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
        assert_eq!(downstream.principal_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn all_owner_financial_and_internal_mutations_fail_before_verification() {
        let routes = [
            (Method::POST, "/api/v1/pay/intents"),
            (Method::POST, "/api/v1/pay/intents/intent-id/confirm"),
            (Method::POST, "/api/v1/pay/intents/intent-id/cancel"),
            (Method::POST, "/api/v1/pay/escrows/escrow-id/release"),
            (Method::POST, "/api/v1/pay/escrows/escrow-id/refund"),
            (Method::POST, "/api/v1/pay/escrows/escrow-id/dispute"),
            (
                Method::POST,
                "/api/v1/pay/escrows/escrow-id/confirm-deposit",
            ),
            (Method::POST, "/api/v1/pay/links"),
            (Method::POST, "/api/v1/pay/links/link-id/redeem"),
            (Method::POST, "/api/v1/pay/webhooks/on-chain"),
        ];
        let (app, downstream) = app();
        for bearer in [None, Some("frontend-owner"), Some("admin-manage")] {
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
    async fn spoofable_headers_are_removed_before_authorized_dispatch() {
        let (app, downstream) = app();
        let mut req = request(Method::GET, "/api/v1/pay/intents", Some("frontend-owner"));
        req.headers_mut()
            .insert("x-user-id", "attacker".parse().unwrap());
        req.headers_mut()
            .insert("x-wallet-address", "attacker".parse().unwrap());
        req.headers_mut()
            .insert("x-permissions", "*:*".parse().unwrap());
        assert_eq!(status(&app, req).await, StatusCode::OK);
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 1);
        assert_eq!(downstream.spoofed_identity_seen.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn unknown_methods_arity_encoded_and_reserved_paths_are_hidden() {
        let paths = [
            (Method::POST, "/health"),
            (Method::GET, "/health/"),
            (Method::DELETE, "/api/v1/pay/intents/intent-id"),
            (Method::GET, "/api/v1/pay/intents/one/two"),
            (Method::GET, "/api/v1/pay/links/epsx%2Fhidden"),
            (Method::GET, "/api/v1/pay/history/.."),
            (Method::GET, "/api/v1/pay/links/intents"),
            (Method::GET, "/api/v1/pay/history/sync"),
            (Method::GET, "/api/v1/pay/intents/escrows"),
            (Method::POST, "/api/v1/pay/escrows/refund/resolve"),
            (Method::POST, "/api/v1/admin/pay/intents/force-cancel"),
            (Method::GET, "/api/v1/pay/private"),
        ];
        let (app, downstream) = app();
        for (method, path) in paths {
            assert_eq!(
                status(&app, request(method, path, Some("admin-global-wildcard"))).await,
                StatusCode::NOT_FOUND
            );
        }
        assert_eq!(downstream.hits.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn owner_is_derived_from_verified_wallet_and_cross_owner_is_hidden() {
        let principal = VerifiedPrincipal {
            subject: "0xAbC".into(),
            wallet_address: "0xAbC".into(),
            audience: FRONTEND_AUDIENCE.into(),
            permissions: vec![],
        };
        assert_eq!(canonical_owner(&principal, None).unwrap(), "0xabc");
        assert_eq!(canonical_owner(&principal, Some("0xabc")).unwrap(), "0xabc");
        assert_eq!(
            canonical_owner(&principal, Some("0xdef")),
            Err(StatusCode::NOT_FOUND)
        );
        assert_eq!(
            canonical_owner(&principal, Some("")),
            Err(StatusCode::NOT_FOUND)
        );
    }

    #[test]
    fn admin_resource_prefix_is_a_strict_path_boundary() {
        assert!(matches!(
            classify(&Method::GET, "/api/v1/admin/pay/links"),
            AccessPolicy::AdminPermission(PAYMENT_LINKS_VIEW_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/pay/links"),
            AccessPolicy::AdminPermission(PAYMENT_LINKS_MANAGE_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/pay/links/link-id/disable"),
            AccessPolicy::AdminPermission(PAYMENT_LINKS_MANAGE_PERMISSION)
        ));
        assert!(matches!(
            classify(&Method::GET, "/api/v1/admin/pay/intents"),
            AccessPolicy::PaymentsRead
        ));
        assert!(matches!(
            classify(&Method::POST, "/api/v1/admin/pay/intents/intent-id/cancel"),
            AccessPolicy::AdminPermission(PAYMENTS_MANAGE_PERMISSION)
        ));
        for path in [
            "/api/v1/admin/payfoo",
            "/api/v1/admin/pay/linksfoo",
            "/api/v1/admin/pay/links/../disable",
            "/api/v1/admin/pay/links/link.id/disable",
            "/api/v1/admin/pay/links/link%2Did/disable",
        ] {
            assert_eq!(
                classify(&Method::POST, path),
                AccessPolicy::Blocked,
                "{path}"
            );
        }
    }

    #[test]
    fn production_identity_urls_reject_plaintext_and_local_hosts() {
        assert!(build_auth_verifier(
            "https://identity.epsx.io",
            "https://identity.epsx.io/.well-known/jwks.json",
            true
        )
        .is_ok());
        assert!(build_auth_verifier(
            "http://identity.epsx.io",
            "https://identity.epsx.io/.well-known/jwks.json",
            true
        )
        .is_err());
        assert!(build_auth_verifier(
            "https://identity.epsx.io",
            "http://127.0.0.1:8080/.well-known/jwks.json",
            true
        )
        .is_err());
    }
}
